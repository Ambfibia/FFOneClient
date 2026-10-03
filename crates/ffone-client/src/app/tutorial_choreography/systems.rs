use super::*;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_tutorial_choreography_events(
    mut commands: Commands,
    drive: TutorialChoreographyDrive,
) {
    let TutorialChoreographyDrive {
        mut choreography_player,
        mut execution,
        mut presentation,
        mut issues,
        mut actor_commands,
        actor_registry,
        actor_transforms,
        cameras,
        mut players,
        mut tutorial,
        mut runtime,
        language,
        mut voice_subtitles,
        mut effect_runtime,
        mut projectile_random,
        tutorial_voices,
        tutorial_music_audio,
        mut ambient,
        mut auxiliary_presentation,
        mut nano_commands,
        mut gameplay_nano_commands,
        nano_state,
        mut player_commands,
    } = drive;
    let mut events = VecDeque::from(std::mem::take(&mut execution.pending));
    let Ok((mut player_transform, mut player_controller)) = players.single_mut() else {
        return;
    };
    let camera_transform = cameras
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| Transform::from_translation(player_transform.translation));
    let mut frame = TutorialChoreographyFrame {
        player: player_transform.clone(),
        camera: camera_transform,
        start: Transform::from_translation(execution.player_start),
        nano: nano_state.live_transform(),
    };

    while let Some(event) = events.pop_front() {
        match event {
            ChoreographyPlaybackEvent::Action {
                source_line,
                action,
                ..
            } => match action {
                ChoreographyAction::Npc(action) => apply_choreography_npc_action(
                    action,
                    source_line,
                    &mut frame,
                    &execution.camera_captures,
                    &actor_registry,
                    &actor_transforms,
                    &mut actor_commands,
                    &mut nano_commands,
                    &mut issues,
                ),
                ChoreographyAction::Player(action) => match action {
                    PlayerAction::Animation(clip) => {
                        let queued = tutorial
                            .character()
                            .and_then(|character| {
                                tutorial_player_gender_from_protocol(character.style.gender).ok()
                            })
                            .and_then(|gender| player_commands.avatar_emote(gender, clip).ok())
                            .is_some();
                        if queued {
                            execution.player_pose = Some(clip);
                        } else {
                            issues.push(TutorialChoreographyIssue::RigAnimationUnavailable {
                                source_line,
                                target: RigAnimationTarget::Player,
                                clip,
                            });
                        }
                    }
                    PlayerAction::StandForce => {
                        execution.player_pose = Some("stand1");
                        player_controller.velocity = Vec3::ZERO;
                        if let Some(gender) = tutorial.character().and_then(|character| {
                            tutorial_player_gender_from_protocol(character.style.gender).ok()
                        }) {
                            player_commands.stand_force(gender);
                        } else {
                            issues.push(TutorialChoreographyIssue::RigAnimationUnavailable {
                                source_line,
                                target: RigAnimationTarget::Player,
                                clip: "stand1",
                            });
                        }
                    }
                    PlayerAction::Warp(target) => {
                        if let Some(target) = resolve_choreography_position(
                            target,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        ) {
                            player_transform.translation = target;
                            frame.player = player_transform.clone();
                            player_controller.velocity = Vec3::ZERO;
                        }
                    }
                    PlayerAction::Face(target) => {
                        if let Some(target) = resolve_choreography_entity_position(
                            target,
                            source_line,
                            &frame,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        ) && let Some(heading) =
                            tutorial_player_heading_toward(&player_transform, target)
                        {
                            execution.player_target_heading = Some(heading);
                            player_controller.yaw_degrees = heading.degrees();
                        }
                    }
                    PlayerAction::FacePosition(target) => {
                        if let Some(target) = resolve_choreography_position(
                            target,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        ) && let Some(heading) =
                            tutorial_player_heading_toward(&player_transform, target)
                        {
                            execution.player_target_heading = Some(heading);
                            player_controller.yaw_degrees = heading.degrees();
                        }
                    }
                    PlayerAction::FaceAngle(angle) => {
                        // `TutorialSceneAngle(float)` consumes a direct Unity
                        // visual heading, not OpenFusion's packet `iAngle`.
                        let heading = tutorial_scene_angle_heading(angle);
                        execution.player_target_heading = Some(heading);
                        player_controller.yaw_degrees = heading.degrees();
                    }
                    PlayerAction::SetTemporaryNanoAbsent => {
                        gameplay_nano_commands.dismiss();
                        frame.nano = None;
                    }
                    PlayerAction::Hide | PlayerAction::Show => {}
                },
                ChoreographyAction::Camera(action) => match action {
                    CameraAction::CurrentTarget(target) => {
                        presentation.camera.resolved_current_target = resolve_choreography_position(
                            target,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        );
                    }
                    CameraAction::Target(target) => {
                        presentation.camera.resolved_target = resolve_choreography_position(
                            target,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        );
                    }
                    CameraAction::Start(start) => {
                        presentation.camera.resolved_start = resolve_choreography_position(
                            start,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        );
                    }
                    CameraAction::StoredStart(start) => {
                        presentation.camera.resolved_stored_start = resolve_choreography_position(
                            start,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        );
                    }
                    CameraAction::TargetRotation(rotation) => {
                        presentation.camera.resolved_target_rotation =
                            resolve_choreography_rotation(
                                rotation,
                                source_line,
                                frame.camera.translation,
                                &frame,
                                &execution.camera_captures,
                                &actor_registry,
                                &actor_transforms,
                                &mut issues,
                            );
                    }
                    CameraAction::LookAt(target) => {
                        presentation.camera.resolved_look_at = resolve_choreography_position(
                            target,
                            source_line,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        );
                    }
                    CameraAction::CaptureTransform(slot) => {
                        execution
                            .camera_captures
                            .capture(slot, frame.camera.clone());
                    }
                    CameraAction::Mode(_)
                    | CameraAction::FreezeCurrentTarget
                    | CameraAction::Distance(_)
                    | CameraAction::ForwardInterpolation(_) => {}
                },
                ChoreographyAction::VoiceOff => {
                    stop_tutorial_voice(&mut commands, &tutorial_voices);
                    voice_subtitles.clear();
                }
                // `StopBGM` owns the source music channel. InitStartChapter
                // sets that channel to `none`; it must never stop ambient,
                // voice or overlapping UI/world one-shots.
                ChoreographyAction::StopBgm => {
                    for entity in &tutorial_music_audio {
                        commands.entity(entity).despawn();
                    }
                }
                ChoreographyAction::Effect(action) => apply_choreography_effect_action(
                    action,
                    source_line,
                    &frame,
                    &execution.camera_captures,
                    &language,
                    &actor_registry,
                    &actor_transforms,
                    &mut issues,
                    &mut effect_runtime,
                ),
                ChoreographyAction::Projectile(action) => {
                    apply_choreography_projectile_action(
                        action,
                        source_line,
                        &frame,
                        &actor_registry,
                        &actor_transforms,
                        &mut issues,
                        &mut effect_runtime,
                        &mut projectile_random,
                    );
                }
                ChoreographyAction::Picture(action) => {
                    apply_choreography_picture_action(action, &mut auxiliary_presentation);
                }
                ChoreographyAction::Sound(TutorialSoundAction::Ambient { cue }) => {
                    if let Err(error) = ambient.request_legacy_cue(cue) {
                        runtime.message = format!("Tutorial ambient cue rejected: {error}");
                    }
                }
                // Presenter owns the proven UI/world one-shots. Ambient is
                // exclusively routed through TutorialAmbientRuntime above.
                ChoreographyAction::Sound(
                    TutorialSoundAction::UiOneShot { .. }
                    | TutorialSoundAction::WorldOneShot { .. },
                ) => {}
                ChoreographyAction::Nano(action) => match action {
                    NanoAction::Equip {
                        nano_id,
                        skill_id,
                        slot,
                        stamina,
                    } => {
                        if let Ok(index) = usize::try_from(slot) {
                            equip_tutorial_nano(
                                &mut runtime,
                                &mut gameplay_nano_commands,
                                index,
                                nano_id,
                                skill_id,
                                stamina,
                            );
                        }
                    }
                    NanoAction::Call => nano_commands.call(),
                    NanoAction::Happy => nano_commands.happy(),
                    NanoAction::Stand => nano_commands.set_stand_motion(),
                    NanoAction::Emote(clip) => nano_commands.play_emote(clip),
                    NanoAction::SetVoiceDisabled(disabled) => {
                        nano_commands.set_voice_disabled(disabled);
                    }
                    NanoAction::DestroyPresentationObject => {
                        nano_commands.destroy();
                        frame.nano = None;
                    }
                },
                ChoreographyAction::Equipment(EquipmentAction::TutorialWeaponByLocaleAndClass) => {
                    let class = tutorial
                        .character()
                        .map_or(0, |character| character.style.class);
                    // Retrobution keeps `localized.local` fixed to US even when the displayed
                    // language changes. The old language check selected the class-matched
                    // Korean branch for Russian UI; OpenFusion does not persist iClass, so
                    // that branch returned no tutorial weapon at all.
                    let weapon_id = player_commands
                        .tutorial_weapon(true, class)
                        .map(|request| i32::from(request.item.item_id));
                    execution.tutorial_weapon_id = weapon_id;
                    runtime.tutorial_weapon_id = weapon_id;
                }
                ChoreographyAction::Progress(ProgressAction::InitChapter { chapter, step }) => {
                    if tutorial.init_chapter(chapter).is_ok() && step != 0 {
                        tutorial.write_step_preserving_state(step);
                    }
                }
                // Presenter owns all scene audio. LoopAction is retained by the
                // choreography player solely for exact lifecycle/skip cleanup.
                ChoreographyAction::Loop(LoopAction::Start(_))
                | ChoreographyAction::Loop(LoopAction::StartIfAbsent(_))
                | ChoreographyAction::Loop(LoopAction::StopIfPresent)
                | ChoreographyAction::Hud(_)
                | ChoreographyAction::SpatialAudio(_)
                | ChoreographyAction::EventScene(_)
                | ChoreographyAction::Cinematic(_)
                | ChoreographyAction::FadeEnabled(_)
                | ChoreographyAction::GameFadeIn
                | ChoreographyAction::SubtitleClear
                | ChoreographyAction::Sequence(_)
                | ChoreographyAction::Pan(_)
                | ChoreographyAction::Unresolved(_) => {}
            },
            ChoreographyPlaybackEvent::SequenceSample {
                source_line,
                sample,
            } => match sample {
                FrameSequenceSample::NpcDelta {
                    id,
                    frame: sequence_frame,
                    cumulative_translation,
                    rotation_each_frame,
                } => {
                    let divisor = f32::from(sequence_frame.saturating_add(1));
                    actor_commands.translate_native(
                        id,
                        choreography_client_vector(cumulative_translation) / divisor,
                    );
                    if let Some(rotation) = rotation_each_frame
                        && let Some(origin) = actor_registry
                            .entity(id)
                            .and_then(|entity| actor_transforms.get(entity).ok())
                            .map(|transform| transform.translation)
                        && let Some(rotation) = resolve_choreography_rotation(
                            rotation,
                            source_line,
                            origin,
                            &frame,
                            &execution.camera_captures,
                            &actor_registry,
                            &actor_transforms,
                            &mut issues,
                        )
                    {
                        actor_commands.set_native_rotation(id, rotation);
                    }
                }
                FrameSequenceSample::NpcLerp { id, value, .. } => {
                    actor_commands.warp_native(id, choreography_client_vector(value));
                }
                FrameSequenceSample::PlayerPathAndFade {
                    player_position: target,
                    ..
                } => {
                    let target = choreography_client_vector(target);
                    player_transform.translation = target;
                    frame.player = player_transform.clone();
                    player_controller.velocity = Vec3::ZERO;
                }
                FrameSequenceSample::Fade { .. }
                | FrameSequenceSample::SubtitleTypewriter { .. }
                | FrameSequenceSample::CameraShake { .. } => {}
            },
            ChoreographyPlaybackEvent::BlockingWaitReached { .. } => {
                // The initial InstantiateEffect/PreloadEffect actions precede
                // the source `while` in this same event batch. Process their
                // real disposition now so an immediate success does not gain
                // an artificial frame and an immediate failure can issue its
                // first retry before yielding 0.1 seconds.
                effect_runtime.process_pending();
                let resumed = choreography_player.tick_with_wait_resolution(0.0, false, |wait| {
                    tutorial_blocking_wait_is_resolved(wait, &effect_runtime)
                });
                for resumed_event in &resumed {
                    presentation.apply(*resumed_event);
                }
                events.extend(resumed);
            }
            ChoreographyPlaybackEvent::FinalSkipFade { .. }
            | ChoreographyPlaybackEvent::Finished { .. } => {}
        }
    }
}

pub(in super::super) fn apply_tutorial_player_cinematic_turn(
    time: Res<Time>,
    presentation: Res<TutorialChoreographyPresentation>,
    mut execution: ResMut<TutorialChoreographyExecution>,
    mut players: Query<&mut Transform, With<LocalPlayer>>,
) {
    if !presentation.event_scene {
        execution.player_target_heading = None;
        return;
    }
    let Some(target) = execution.player_target_heading else {
        return;
    };
    let Ok(mut transform) = players.single_mut() else {
        return;
    };
    transform.rotation = tutorial_cinematic_turn_rotation(
        transform.rotation,
        target.native_root_rotation(),
        time.delta_secs(),
    );
}

pub(in super::super) fn apply_tutorial_choreography_camera(
    time: Res<Time>,
    presentation: Res<TutorialChoreographyPresentation>,
    mut cameras: Query<(&mut Transform, &mut LegacyOrbitCamera), With<Camera3d>>,
    mut memory: Local<TutorialCameraPoseMemory>,
) {
    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };
    let camera = &presentation.camera;

    // `cnPlayerCamera.LookAtPosition` temporarily aims MainCamera only to copy
    // its horizontal euler angle into RotationY, restores the old transform,
    // and resets RotationX to zero. It is a one-shot orbit update, not a
    // persistent direct Transform.LookAt.
    if camera.look_at_revision != memory.applied_look_at_revision
        && let Some(look_at) = camera.resolved_look_at
    {
        let direction = look_at - transform.translation;
        if direction.length_squared() > f32::EPSILON {
            let unity = native_to_unity_vector(direction);
            orbit.yaw_degrees = unity.x.atan2(unity.z).to_degrees();
        }
        orbit.pitch_degrees = 0.0;
        memory.applied_look_at_revision = camera.look_at_revision;
    }

    if camera.mode == CameraMode::None {
        memory.mode = None;
        memory.translation = None;
        memory.target = None;
        memory.authored_target = None;
        memory.authored_current_target = None;
        memory.stored_start = None;
        memory.frozen_target = None;
        return;
    }
    let mode_changed = memory.mode != Some(camera.mode);
    if mode_changed || memory.translation.is_none() {
        let previous_target = memory.target;
        memory.mode = Some(camera.mode);
        memory.translation = Some(match camera.mode {
            CameraMode::FromToCamera => camera
                .resolved_stored_start
                .unwrap_or(transform.translation),
            // EventCameraControll lerps the live MainCamera transform. Its
            // TempPosition field is used only by the non-forward target path.
            CameraMode::EventCameraControl => transform.translation,
            CameraMode::None => unreachable!("normal orbit camera returned above"),
        });
        memory.target = match camera.mode {
            CameraMode::FromToCamera => camera
                .resolved_current_target
                .or(previous_target)
                .or(camera.resolved_target),
            CameraMode::EventCameraControl => camera.resolved_target.or(previous_target),
            CameraMode::None => unreachable!("normal orbit camera returned above"),
        };
        memory.authored_current_target = camera.resolved_current_target;
        memory.authored_target = camera.resolved_target;
        memory.stored_start = camera.resolved_stored_start;
    } else {
        // `TempPosition` is an independently authored current position in the
        // original coroutine.  It is not a fallback for kNewStartPosition:
        // assigning it must immediately reseed the FromTo interpolation.
        if camera.resolved_stored_start != memory.stored_start {
            if let Some(stored_start) = camera.resolved_stored_start {
                memory.translation = Some(stored_start);
            }
            memory.stored_start = camera.resolved_stored_start;
        }
        if camera.resolved_target != memory.authored_target {
            memory.authored_target = camera.resolved_target;
            memory.frozen_target = None;
            if memory.target.is_none() {
                memory.target = camera.resolved_target;
            }
        }
        // `kCTargetPosition` is independent from `kNewTargetPosition`.
        // Assigning it is an authored cut/reseed of the current focus, while
        // changing only the destination must preserve interpolation continuity.
        if camera.mode == CameraMode::FromToCamera
            && camera.resolved_current_target != memory.authored_current_target
        {
            if let Some(current_target) = camera.resolved_current_target {
                memory.target = Some(current_target);
            }
            memory.authored_current_target = camera.resolved_current_target;
        }
    }
    if camera.freeze_target_revision != memory.applied_freeze_target_revision {
        memory.frozen_target = memory.target;
        memory.applied_freeze_target_revision = camera.freeze_target_revision;
    }
    let amount = time.delta_secs().clamp(0.0, 1.0);
    match camera.mode {
        CameraMode::None => unreachable!("normal orbit camera returned above"),
        CameraMode::FromToCamera => {
            if let Some(desired) = camera.resolved_start {
                if camera.forward_interpolation {
                    let next = memory
                        .translation
                        .unwrap_or(transform.translation)
                        .lerp(desired, amount);
                    memory.translation = Some(next);
                } else {
                    memory.translation = Some(desired);
                }
            }
            if let Some(desired_target) = memory.frozen_target.or(camera.resolved_target) {
                let next_target = if camera.forward_interpolation {
                    memory
                        .target
                        .unwrap_or(desired_target)
                        .lerp(desired_target, amount)
                } else {
                    desired_target
                };
                memory.target = Some(next_target);
            }
            if let Some(translation) = memory.translation {
                transform.translation = translation + camera.shake_start_offset;
            }
            if let Some(target) = memory.target {
                transform.look_at(target + camera.shake_target_offset, Vec3::Y);
            }
        }
        CameraMode::EventCameraControl => {
            if let Some(target) = camera.resolved_target {
                let rotation = camera
                    .resolved_target_rotation
                    .unwrap_or(transform.rotation);
                if camera.forward_interpolation {
                    let desired = target - rotation.mul_vec3(Vec3::Z) * camera.distance;
                    let next = memory
                        .translation
                        .unwrap_or(transform.translation)
                        .lerp(desired, amount);
                    memory.translation = Some(next);
                    memory.target = Some(target);
                    transform.translation = next + camera.shake_start_offset;
                    transform.look_at(target + camera.shake_target_offset, Vec3::Y);
                } else {
                    let smoothed_target = memory.target.unwrap_or(target).lerp(target, amount);
                    memory.target = Some(smoothed_target);
                    let position = smoothed_target - rotation.mul_vec3(Vec3::Z) * camera.distance;
                    memory.translation = Some(position);
                    transform.translation = position + camera.shake_start_offset;
                    transform.look_at(smoothed_target + camera.shake_target_offset, Vec3::Y);
                }
            }
        }
    }
}

pub(in super::super) fn sync_tutorial_choreography_visibility(
    presentation: Res<TutorialChoreographyPresentation>,
    runtime: Option<Res<RuntimeStatus>>,
    model: Res<MissionUiModel>,
    guide_ui: Option<Res<GuideUiModel>>,
    guide_production: Option<Res<GuideProductionRuntime>>,
    bank_ui: Option<Res<BankUiState>>,
    bank_production: Option<Res<BankProductionRuntime0104>>,
    vendor_ui: Option<Res<VendorUiState>>,
    vendor_production: Option<Res<VendorProductionRuntime0104>>,
    rule_ui: Option<Res<RuleUiModel>>,
    rule_runtime: Option<Res<RuleRuntime>>,
    nano_free_tuning: Option<Res<NanoFreeTuningModel>>,
    nano_free_tuning_production: Option<Res<NanoFreeTuningProductionRuntime>>,
    mode_visibility: TutorialChoreographyModeVisibility,
    mut hud: Query<&mut Visibility, With<GameplayHud>>,
    mut player_scenes: Query<&mut Visibility, (With<LocalCharacterScene>, Without<GameplayHud>)>,
) {
    let rule_modal = rule_ui.as_ref().is_some_and(|model| {
        rule_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.modal_active(model))
    });
    let bank_modal = bank_ui.as_ref().is_some_and(|state| {
        state.phase != BankLifecyclePhase::Hidden
            || bank_production
                .as_ref()
                .is_some_and(|production| production.modal_active())
    });
    let vendor_modal = vendor_ui.as_ref().is_some_and(|state| {
        state.phase != VendorLifecyclePhase::Hidden
            || vendor_production
                .as_ref()
                .is_some_and(|production| production.modal_active())
    });
    let nano_free_tuning_modal = nano_free_tuning.as_ref().is_some_and(|model| {
        nano_free_tuning_production
            .as_ref()
            .is_some_and(|production| production.modal_active(model))
    });
    let barber_modal = mode_visibility.barber.as_ref().is_some_and(|model|model.active());
    let combi_modal = mode_visibility
        .combi_runtime
        .as_ref()
        .is_some_and(|runtime| runtime.modal_active());
    let email_modal = mode_visibility
        .email_runtime
        .as_ref()
        .is_some_and(|runtime| runtime.modal_active());
    let enchant_modal = mode_visibility
        .enchant_runtime
        .as_ref()
        .is_some_and(|runtime| runtime.is_active());
    // One owner combines every reason to hide the HUD. GM presentation must
    // not restore it while a cinematic or modal still owns the screen.
    let gm_hidden = runtime.as_ref().is_some_and(|runtime| {
        runtime.player_id.is_some() && runtime.user_level <= 50 && runtime.chat.gm.hide_ui
    });
    let hud_visibility = if gm_hidden
        || presentation.hud_hide_depth > 0
        || model.npc_letterbox_visible()
        || bank_modal
        || vendor_modal
        || rule_modal
        || nano_free_tuning_modal
        || email_modal
        || combi_modal
        || barber_modal
        || enchant_modal
    {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut hud {
        visibility.set_if_neq(hud_visibility);
    }
    // `cnPlayerCamera.SetSubTarget` excludes the local Avatar layer and hides
    // its PrintName until EndSubTarget. FFOne has no separate local
    // world-space PrintName entity; LocalCharacterScene is the exact visual
    // root for the assembled player rig and its equipment.
    let guide_subtarget = guide_ui.as_ref().is_some_and(|guide| {
        guide_production
            .as_ref()
            .is_some_and(|production| guide.visible && production.active_source_npc.is_some())
    });
    let vendor_subtarget = vendor_ui.as_ref().is_some_and(|vendor| {
        vendor.phase != VendorLifecyclePhase::Hidden
            && vendor_production
                .as_ref()
                .is_some_and(|production| production.active_source_npc().is_some())
    });
    let bank_subtarget = bank_production
        .as_ref()
        .is_some_and(|production| production.session().is_some() || production.opening().is_some());
    let player_visibility = if presentation.player_hidden
        || model.npc_subtarget_actor_id().is_some()
        || guide_subtarget
        || bank_subtarget
        || vendor_subtarget
        || mode_visibility
            .race_production
            .as_ref()
            .is_some_and(|production| production.camera_subtarget_npc_id().is_some())
        || combi_modal
        || barber_modal
        || enchant_modal
        || rule_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.camera_sub_target_npc_id().is_some())
    {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut player_scenes {
        visibility.set_if_neq(player_visibility);
    }
}
