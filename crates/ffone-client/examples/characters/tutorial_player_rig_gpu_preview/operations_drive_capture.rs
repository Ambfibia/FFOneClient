use super::*;

pub(super) fn drive_capture(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    mut presentation: ResMut<TutorialPlayerPresentationCommandQueue>,
    rigs: Query<(&TutorialSelectedPlayerRigStatus, &NativePlayerRigBones)>,
    transforms: Query<&Transform>,
    applied: Query<&TutorialPlayerAnimationApplied>,
    mut players: Query<&mut AnimationPlayer>,
    mut controllers: Query<&mut LegacyPlayerController>,
    materials: Res<Assets<LegacyModelMaterial>>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    scene_entities: Query<(Entity, &Name, &GlobalTransform)>,
    travel: PreviewTravelResources,
    mut emote_continuation: ResMut<ffone_client::player_emote::PlayerEmoteContinuation>,
    weapons: Query<(&TutorialPlayerWeaponAttachment, &Visibility)>,
    mut exit: MessageWriter<AppExit>,
) {
    let PreviewTravelResources {
        mut skyway,
        mut vehicle,
    } = travel;
    state.frames = state.frames.saturating_add(1);
    if config.case == PreviewCase::WeaponSwap {
        return;
    }
    if let Some(error) = material_errors.iter().next() {
        eprintln!("material error: {}", error.0);
        exit.write(AppExit::error());
        return;
    }
    let Ok((status, bones)) = rigs.get(config.rig_root) else {
        if state.frames >= TIMEOUT_FRAMES {
            eprintln!("tutorial player rig components did not become available");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    };
    if let TutorialSelectedPlayerRigStatus::Blocked(error) = status {
        eprintln!("tutorial player rig blocked: {error}");
        exit.write(AppExit::error());
        return;
    }
    if !matches!(status, TutorialSelectedPlayerRigStatus::Ready) {
        if state.frames >= TIMEOUT_FRAMES {
            eprintln!("tutorial player rig timed out in status {status:?}");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    // Addressed custom-emote proof through the production animation adapter.
    if let Ok(code) = env::var("FFONE_CUSTOM_EMOTE_CODE") {
        let code = code.parse::<i32>().expect("numeric custom emote code");
        assert!((31..=44).contains(&code));
        let expected = TutorialPlayerClip::from_avatar_emote_code(code).unwrap();
        if !state.emote_started {
            presentation.avatar_emote_code(config.gender, code).unwrap();
            state.emote_started = true;
            state.ready_frame = Some(state.frames);
        }
        if let Ok(pose) = applied.get(config.rig_root) {
            if pose.clip == expected {
                let player = players.get_mut(pose.animation_player).unwrap();
                let seek = player.animation(pose.animation_node).unwrap().seek_time();
                if seek > 0.25 {
                    let rig_position = scene_entities.get(config.rig_root).unwrap().2.translation();
                    for name in [
                        "Bip01 Pelvis",
                        "Bip01 Spine",
                        "Bip01 Head",
                        "Bip01 L Thigh",
                        "Bip01 R Hand",
                    ] {
                        let entity = bones.unique_by_true_name(name).unwrap();
                        let local = transforms.get(entity).unwrap();
                        assert!(
                            local.scale.is_finite() && local.scale.max_element() < 1.8,
                            "emote {code} {name}: doubled body scale {:?}",
                            local.scale
                        );
                        let global = scene_entities.get(entity).unwrap().2;
                        assert!(
                            global.translation().distance(rig_position) < 4.0,
                            "emote {code} {name}: stretched hierarchy {:?}",
                            global.translation()
                        );
                    }
                    if !state.capture_issued && seek > 0.35 {
                        state.capture_issued = true;
                        commands
                            .spawn(Screenshot::primary_window())
                            .observe(save_screenshot);
                    }
                    if state.capture_saved
                        && env::var_os("FFONE_CUSTOM_EMOTE_INTERRUPT").is_some()
                        && state.emote_interrupt_frame.is_none()
                    {
                        controllers
                            .get_mut(config.controller_root)
                            .unwrap()
                            .set_current_direction_key(1);
                        state.emote_interrupt_frame = Some(state.frames);
                    }
                }
            } else if state.capture_saved && state.frames > state.ready_frame.unwrap() + 10 {
                eprintln!(
                    "custom emote proof: code {code}, {}, scales bounded, capture saved, {} to {}",
                    expected.name(),
                    if state.emote_interrupt_frame.is_some() {
                        "interrupted"
                    } else {
                        "completed"
                    },
                    pose.clip.name()
                );
                exit.write(AppExit::Success);
                return;
            }
        }
        if state.frames > state.ready_frame.unwrap() + TIMEOUT_FRAMES {
            panic!("custom emote {code} did not finish/capture");
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    if matches!(config.case, PreviewCase::Dance | PreviewCase::Beach) {
        if !state.emote_started {
            if env::var_os("FFONE_EMOTE_ATTACK_PROBE").is_some() {
                presentation.tutorial_weapon(true, 1).unwrap();
            }
            presentation
                .avatar_emote_code(
                    config.gender,
                    if config.case == PreviewCase::Dance {
                        6
                    } else {
                        24
                    },
                )
                .unwrap();
            state.emote_started = true;
        }
        // Deterministic delayed server-echo fixture. Waiting frames must not
        // enqueue duplicate requests or release the emote to idle.
        for (owner, code) in emote_continuation.pending.drain(..) {
            assert_eq!(owner, config.controller_root);
            assert!(if config.case == PreviewCase::Dance {
                [6, 17, 18, 19, 20].contains(&code)
            } else {
                code == 24
            });
            assert!(
                state.emote_echo.is_none(),
                "duplicate request before the server echo"
            );
            state.emote_echo = Some((code, state.frames + 20));
            presentation.record_emote_continuation_sent(code);
        }
        if let Some((code, frame)) = state.emote_echo
            && state.frames >= frame
        {
            state.emote_echo = None;
            let accepted = presentation.accept_avatar_emote_echo(code);
            if env::var_os("FFONE_EMOTE_ATTACK_PROBE").is_some()
                && state.emote_interrupt_frame.is_some()
            {
                assert!(!accepted, "cancelled repeat echo must be discarded");
                eprintln!("cancelled continuation echo discarded: code {code}");
            }
            if accepted {
                presentation.avatar_emote_code(config.gender, code).unwrap();
                state.emote_cycles += 1;
                eprintln!("emote continuation {}: code {code}", state.emote_cycles);
            }
        }
        if let Ok(pose) = applied.get(config.rig_root) {
            if let Some(interrupt_frame) = state.emote_interrupt_frame {
                if state.frames > interrupt_frame + 4 && !pose.clip.is_avatar_emote_code_clip() {
                    if env::var_os("FFONE_EMOTE_ATTACK_PROBE").is_some() {
                        assert!(state.attack_dispatched);
                        if !state.emote_attack_seen {
                            assert!(
                                weapons
                                    .iter()
                                    .any(|(weapon, visibility)| weapon.rig_root == config.rig_root
                                        && *visibility != Visibility::Hidden),
                                "attack must restore the hand weapon"
                            );
                            assert!(
                                matches!(
                                    pose.clip,
                                    TutorialPlayerClip::RifleAttack1
                                        | TutorialPlayerClip::RifleAttack1Upper
                                ),
                                "expected actual attack playback, got {}",
                                pose.clip.name()
                            );
                            state.emote_attack_seen = true;
                            state.capture_saved = false;
                            commands
                                .spawn(Screenshot::primary_window())
                                .observe(save_screenshot);
                        }
                        if state.frames <= interrupt_frame + 40 || !state.capture_saved {
                            std::thread::sleep(Duration::from_millis(16));
                            return;
                        }
                        assert!(
                            state.emote_echo.is_none(),
                            "late echo must be delivered before success"
                        );
                    }
                    eprintln!(
                        "emote proof: {} cycles, capture saved, interruption released {}",
                        state.emote_cycles,
                        pose.clip.name()
                    );
                    exit.write(AppExit::Success);
                    return;
                }
            } else if state.emote_cycles >= 2 {
                assert!(
                    pose.clip.is_avatar_emote_code_clip(),
                    "persistent emote returned to idle"
                );
                if !state.capture_issued
                    && players
                        .get(pose.animation_player)
                        .ok()
                        .and_then(|p| p.animation(pose.animation_node))
                        // Exclude the outgoing terminal sample on the echo
                        // frame; capture a settled pose in the renewed clip.
                        .is_some_and(|a| (1.0..1.5).contains(&a.seek_time()))
                {
                    state.capture_issued = true;
                    commands
                        .spawn(Screenshot::primary_window())
                        .observe(save_screenshot);
                }
                if state.capture_saved {
                    if env::var_os("FFONE_EMOTE_ATTACK_PROBE").is_some() {
                        state.attack_pending = true;
                        let code = if config.case == PreviewCase::Dance {
                            6
                        } else {
                            24
                        };
                        assert!(state.emote_echo.is_none());
                        presentation.record_emote_continuation_sent(code);
                        state.emote_echo = Some((code, state.frames + 20));
                    } else {
                        controllers
                            .get_mut(config.controller_root)
                            .unwrap()
                            .set_current_direction_key(1);
                    }
                    state.emote_interrupt_frame = Some(state.frames);
                }
            }
        }
        if state.frames >= 1800 {
            eprintln!("emote proof timed out after {} cycles", state.emote_cycles);
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    if matches!(
        config.case,
        PreviewCase::VehicleBoard | PreviewCase::VehicleScooter
    ) {
        let frame = state.frames;
        let ready = *state.ready_frame.get_or_insert(frame);
        let attachment = scene_entities
            .iter()
            .find(|(_, name, _)| name.as_str().starts_with("Personal vehicle "));
        let clip_ready = applied
            .get(config.rig_root)
            .is_ok_and(|pose| pose.clip.is_vehicle());
        if frame > ready + 120 && !state.capture_issued && clip_ready {
            assert!(
                scene_entities
                    .iter()
                    .any(|(_, name, _)| name.as_str() == "Vehicle exhaust trail"),
                "vehicle must render trails"
            );
            let (_, _, transform) = attachment.expect("vehicle must attach to active rig");
            let socket = bones
                .unique_by_true_name("vehicle")
                .expect("vehicle socket");
            let (_, _, socket_world) = scene_entities.get(socket).unwrap();
            assert!(transform.translation().distance(socket_world.translation()) < 0.0001);
            assert!(transform.translation().is_finite());
            assert!(weapons.iter().all(|(_, v)| *v == Visibility::Hidden));
            state.capture_issued = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_screenshot);
        }
        if state.capture_saved {
            vehicle.item_id = None;
            vehicle.family = ffone_client::avatar_action::LegacyVehiclePresentationFamily::None;
            commands
                .entity(config.controller_root)
                .insert(LegacyAvatarPresentationContext::default());
            if attachment.is_none()
                && !scene_entities
                    .iter()
                    .any(|(_, name, _)| name.as_str() == "Vehicle exhaust trail")
            {
                if env::var_os("FFONE_PLAYER_PREVIEW_DISMOUNT").is_some() {
                    if state.dismount_frame.is_none() {
                        println!("vehicle attachment and trails removed after dismount");
                    }
                    let dismount_frame = *state.dismount_frame.get_or_insert(frame);
                    if frame > dismount_frame + 60 && !clip_ready && !state.dismount_capture_issued
                    {
                        state.dismount_capture_issued = true;
                        commands
                            .spawn(Screenshot::primary_window())
                            .observe(save_dismount_screenshot);
                    }
                    if state.dismount_capture_saved {
                        exit.write(AppExit::Success);
                    }
                } else {
                    println!("vehicle attachment and trails removed after dismount");
                    exit.write(AppExit::Success);
                }
            }
        }
        if frame >= TIMEOUT_FRAMES {
            eprintln!("vehicle preview timed out; clip_ready={clip_ready}");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    if matches!(config.case, PreviewCase::Skyway | PreviewCase::Zipline) {
        let frame = state.frames;
        let ready = *state.ready_frame.get_or_insert(frame);
        let elapsed = frame - ready;
        let attachment_name = if config.case == PreviewCase::Skyway {
            "Monkey Skyway attachment"
        } else {
            "Zipline trolley attachment"
        };
        let attachment = scene_entities
            .iter()
            .find(|(_, name, _)| name.as_str() == attachment_name);
        let clip_ready = applied.get(config.rig_root).is_ok_and(|pose| {
            if config.case == PreviewCase::Skyway {
                matches!(
                    pose.clip,
                    TutorialPlayerClip::Mount1 | TutorialPlayerClip::Mount2
                )
            } else {
                pose.clip == TutorialPlayerClip::RopeDown
            }
        });
        if elapsed >= 120 && !state.capture_issued && clip_ready && !materials.is_empty() {
            let (_, _, transform) =
                attachment.expect("traversal attachment must spawn on the active rig");
            println!(
                "traversal={:?}, attachment={:?}",
                config.case,
                transform.to_scale_rotation_translation()
            );
            if config.case == PreviewCase::Skyway {
                let socket = bones
                    .unique_by_true_name("Bip01 Broomstick")
                    .expect("Skyway socket");
                let (_, _, socket_world) = scene_entities.get(socket).unwrap();
                assert!(
                    transform.translation().distance(socket_world.translation()) < 0.0001,
                    "Monkey must preserve the original shoulder-level socket position"
                );
            }
            assert!(transform.translation().is_finite());
            state.capture_issued = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_screenshot);
        }
        if state.capture_saved {
            skyway.active = false;
            commands
                .entity(config.controller_root)
                .insert(LegacyAvatarPresentationContext::default());
            if attachment.is_none() {
                println!("traversal attachment removed after dismount");
                exit.write(AppExit::Success);
            }
        }
        if frame >= TIMEOUT_FRAMES {
            eprintln!("traversal preview timed out; clip_ready={clip_ready}");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    if matches!(
        config.case,
        PreviewCase::Standup | PreviewCase::StartupLanding
    ) && !state.staying_queued
    {
        presentation
            .avatar_emote(config.gender, TutorialPlayerClip::Staying.name())
            .expect("standup GPU proof must resolve staying");
        state.staying_queued = true;
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    let Ok(applied) = applied.get(config.rig_root) else {
        std::thread::sleep(Duration::from_millis(16));
        return;
    };
    if matches!(
        config.case,
        PreviewCase::Standup | PreviewCase::StartupLanding
    ) {
        let Ok(player) = players.get_mut(applied.animation_player) else {
            eprintln!("standup GPU proof AnimationPlayer disappeared");
            exit.write(AppExit::error());
            return;
        };
        match applied.clip {
            TutorialPlayerClip::Staying => {
                let Some(active) = player.animation(applied.animation_node) else {
                    eprintln!("staying node disappeared before standup dispatch");
                    exit.write(AppExit::error());
                    return;
                };
                if !state.standup_queued && active.seek_time() >= 0.1 {
                    presentation
                        .avatar_emote(config.gender, TutorialPlayerClip::Standup.name())
                        .expect("standup GPU proof must resolve standup");
                    state.standup_queued = true;
                }
            }
            TutorialPlayerClip::Standup => {
                state.standup_seen = true;
                state.standup_node = Some(applied.animation_node);
                let Some(active) = player.animation(applied.animation_node) else {
                    eprintln!("standup node disappeared before its completion edge");
                    exit.write(AppExit::error());
                    return;
                };
                state.standup_last_seek_time = active.seek_time();
                if config.case == PreviewCase::StartupLanding
                    && !state.startup_landing_grounded
                    && active.seek_time() >= 0.1
                {
                    let mut controller = controllers
                        .get_mut(config.controller_root)
                        .expect("startup landing proof controller disappeared");
                    controller.set_grounded(true);
                    state.startup_landing_grounded = true;
                }
            }
            TutorialPlayerClip::JumpEnd if config.case == PreviewCase::StartupLanding => {
                state.jumpend_seen = true;
                state.jumpend_node = Some(applied.animation_node);
                let Some(active) = player.animation(applied.animation_node) else {
                    eprintln!("jumpend node disappeared before its completion edge");
                    exit.write(AppExit::error());
                    return;
                };
                state.jumpend_last_seek_time = active.seek_time();
            }
            TutorialPlayerClip::Stand1
                if state.standup_seen
                    && (config.case == PreviewCase::Standup || state.jumpend_seen) =>
            {
                let Some(active) = player.animation(applied.animation_node) else {
                    eprintln!("stand1 node disappeared after standup completion");
                    exit.write(AppExit::error());
                    return;
                };
                let frames = state.frames;
                let return_frame = *state.stand1_return_frame.get_or_insert(frames);
                if state.frames.saturating_sub(return_frame) >= 30 {
                    if state.standup_last_seek_time
                        > PRIMARY_STANDUP_END_EVENT_SECONDS + MAX_END_EVENT_FRAME_OVERSHOOT_SECONDS
                    {
                        eprintln!(
                            "standup missed its primary end event: lastSeek={:.6}",
                            state.standup_last_seek_time
                        );
                        exit.write(AppExit::error());
                        return;
                    }
                    if state
                        .standup_node
                        .is_some_and(|node| player.animation(node).is_some())
                    {
                        eprintln!("standup node still owns the pose after the return fade");
                        exit.write(AppExit::error());
                        return;
                    }
                    if config.case == PreviewCase::StartupLanding {
                        let expected_jumpend_event = match config.gender {
                            PlayerRigGender::Male => 0.183_333_34,
                            PlayerRigGender::Female => 0.216_666_67,
                        };
                        if state.jumpend_last_seek_time
                            > expected_jumpend_event + MAX_END_EVENT_FRAME_OVERSHOOT_SECONDS
                        {
                            eprintln!(
                                "jumpend missed its primary end event: lastSeek={:.6}",
                                state.jumpend_last_seek_time
                            );
                            exit.write(AppExit::error());
                            return;
                        }
                        if state
                            .jumpend_node
                            .is_some_and(|node| player.animation(node).is_some())
                        {
                            eprintln!("jumpend node still owns the pose after the return fade");
                            exit.write(AppExit::error());
                            return;
                        }
                    }
                    if active.is_paused() || (active.weight() - 1.0).abs() > 0.000_001 {
                        eprintln!(
                            "stand1 did not take full ownership after standup: paused={}, weight={}",
                            active.is_paused(),
                            active.weight()
                        );
                        exit.write(AppExit::error());
                        return;
                    }
                    if !state.capture_issued && !materials.is_empty() {
                        state.capture_issued = true;
                        println!(
                            "ready={status:?}, case={:?}, standupLastSeekTime={:.6}, jumpendLastSeekTime={:.6}, stand1SeekTime={:.6}",
                            config.case,
                            state.standup_last_seek_time,
                            state.jumpend_last_seek_time,
                            active.seek_time()
                        );
                        commands
                            .spawn(Screenshot::primary_window())
                            .observe(save_screenshot);
                    }
                }
            }
            clip => {
                eprintln!("standup GPU proof reached unexpected clip {}", clip.name());
                exit.write(AppExit::error());
                return;
            }
        }
        if state.capture_saved {
            exit.write(AppExit::Success);
        } else if state.frames >= TIMEOUT_FRAMES {
            eprintln!(
                "standup GPU proof timed out: case={:?}, stayingQueued={}, standupQueued={}, standupSeen={}, jumpendSeen={}, standupLastSeek={:.6}, jumpendLastSeek={:.6}",
                config.case,
                state.staying_queued,
                state.standup_queued,
                state.standup_seen,
                state.jumpend_seen,
                state.standup_last_seek_time,
                state.jumpend_last_seek_time
            );
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    if config.case.captures_attack() && applied.clip == TutorialPlayerClip::Attack1Upper {
        let Ok(player) = players.get_mut(applied.animation_player) else {
            eprintln!("unarmed attack AnimationPlayer disappeared");
            exit.write(AppExit::error());
            return;
        };
        let Some(active) = player.animation(applied.animation_node) else {
            std::thread::sleep(Duration::from_millis(16));
            return;
        };
        if !state.capture_issued && active.seek_time() >= 0.25 && !materials.is_empty() {
            state.capture_issued = true;
            println!(
                "ready={status:?}, case={:?}, attackSeekTime={:.6}",
                config.case,
                active.seek_time(),
            );
            for true_name in [
                "Bip01",
                "Bip01 NonAccum",
                "Bip01 Pelvis",
                "Bip01 Spine",
                "Bip01 Spine1",
                "Bip01 R Clavicle",
                "Bip01 R UpperArm",
                "Bip01 R Forearm",
            ] {
                let entity = bones
                    .unique_by_true_name(true_name)
                    .unwrap_or_else(|| panic!("missing attack audit target {true_name}"));
                let transform = transforms
                    .get(entity)
                    .unwrap_or_else(|_| panic!("missing attack audit transform {true_name}"));
                println!("{true_name}: {:?}", transform.rotation);
            }
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_screenshot);
        }
        if state.capture_saved {
            exit.write(AppExit::Success);
        } else if state.frames >= TIMEOUT_FRAMES {
            eprintln!(
                "unarmed attack GPU proof timed out in case {:?}",
                config.case
            );
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    if applied.clip != config.expected_clip {
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    let Ok(mut player) = players.get_mut(applied.animation_player) else {
        eprintln!(
            "tutorial {:?} AnimationPlayer disappeared",
            applied.clip.name()
        );
        exit.write(AppExit::error());
        return;
    };
    let Some(active) = player.animation(applied.animation_node) else {
        if let Some(stop_frame) = state.stop_injected_frame {
            let missing_frames = state.frames.saturating_sub(stop_frame);
            if missing_frames <= MAX_MISSING_BASE_FRAMES {
                std::thread::sleep(Duration::from_millis(16));
                return;
            }
            eprintln!(
                "tutorial {:?} base node was not restored for {missing_frames} frames",
                applied.clip.name()
            );
        } else {
            eprintln!(
                "tutorial {:?} animation node disappeared",
                applied.clip.name()
            );
        }
        exit.write(AppExit::error());
        return;
    };
    if state.stop_injected_frame.is_some() {
        state.stop_recovered = true;
    }
    if active.is_paused() {
        eprintln!(
            "tutorial {:?} animation remained paused after the runtime watchdog",
            applied.clip.name()
        );
        exit.write(AppExit::error());
        return;
    }
    let seek_time = active.seek_time();
    let frames = state.frames;
    let run_start_frame = *state.ready_frame.get_or_insert(frames);
    if config.case.captures_attack() {
        if !state.attack_dispatched && frames.saturating_sub(run_start_frame) >= 5 {
            state.attack_pending = true;
        }
        if state.frames >= TIMEOUT_FRAMES {
            eprintln!(
                "unarmed attack was not applied in GPU case {:?}",
                config.case
            );
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    let last_motion_frame = *state.last_run_motion_frame.get_or_insert(frames);
    if state
        .last_run_seek_time
        .is_some_and(|last| (last - seek_time).abs() > 0.000_001)
    {
        state.last_run_motion_frame = Some(frames);
        state.motion_observed = true;
    }
    state.last_run_seek_time = Some(seek_time);
    let frozen_frames = frames.saturating_sub(last_motion_frame);
    state.max_frozen_run_frames = state.max_frozen_run_frames.max(frozen_frames);
    if frozen_frames > MAX_FROZEN_RUN_FRAMES {
        eprintln!("tutorial run froze at seekTime={seek_time:.6} for {frozen_frames} frames");
        exit.write(AppExit::error());
        return;
    }
    if !state.pause_injected
        && frames.saturating_sub(run_start_frame) >= INJECT_PAUSE_AFTER_RUN_FRAMES
    {
        player
            .animation_mut(applied.animation_node)
            .expect("run node was checked above")
            .pause();
        state.pause_injected = true;
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    if state.pause_injected {
        state.pause_recovered = true;
    }
    if state.stop_injected_frame.is_none()
        && frames.saturating_sub(run_start_frame) >= INJECT_STOP_AFTER_RUN_FRAMES
    {
        player.stop(applied.animation_node);
        state.stop_injected_frame = Some(frames);
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    let spine = bones
        .unique_by_true_name("Bip01 Spine")
        .expect("shared actor skeleton must have one Bip01 Spine");
    let rotation = transforms
        .get(spine)
        .expect("animated spine must retain Transform")
        .rotation;
    let initial = *state.initial_spine_rotation.get_or_insert(rotation);
    if initial.angle_between(rotation) > 0.000_01 {
        state.motion_observed = true;
    }

    if !state.capture_issued
        && state.motion_observed
        && state.pause_recovered
        && state.stop_recovered
        && state.frames.saturating_sub(run_start_frame) >= READY_WARMUP_FRAMES
        && !materials.is_empty()
    {
        state.capture_issued = true;
        println!(
            "ready={status:?}, bodyCompositionMotionRadians={:.8}, \
            pauseRecovered={}, stopRecovered={}, maxFrozenRunFrames={}",
            initial.angle_between(rotation),
            state.pause_recovered,
            state.stop_recovered,
            state.max_frozen_run_frames,
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "tutorial player rig GPU proof timed out: status={status:?}, motionObserved={}",
            state.motion_observed
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(16));
}

pub(super) fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}
