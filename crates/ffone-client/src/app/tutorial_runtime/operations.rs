use super::*;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn drive_tutorial_auxiliary_choreography(
    mut commands: Commands,
    time: Res<Time>,
    tutorial: Res<TutorialSession>,
    asset_server: Res<AssetServer>,
    catalog: Res<NativeAudioCatalog>,
    language: Res<Language>,
    voice_language: Res<VoiceLanguage>,
    localization: Res<Localization>,
    tutorial_content: Res<TutorialMissionContent>,
    mut voice_subtitles: ResMut<TutorialVoiceSubtitleState>,
    mut auxiliary: TutorialAuxiliaryDriveState,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
    mut effect_runtime: ResMut<TutorialEffectRuntime>,
    mut runtime: ResMut<RuntimeStatus>,
    tutorial_voices: Query<Entity, With<TutorialVoiceAudio>>,
) {
    if tutorial.completion_requested {
        return;
    }
    let emissions = auxiliary.mission_runtime.auxiliary.tick(time.delta_secs());
    let Ok(emissions) = emissions else {
        runtime.message = "Tutorial auxiliary timeline rejected a non-finite delta".to_owned();
        return;
    };
    for emission in emissions {
        apply_tutorial_auxiliary_emission(
            emission,
            &mut commands,
            &asset_server,
            &catalog,
            &language,
            &voice_language.effective,
            &localization,
            &tutorial_content,
            &mut voice_subtitles,
            time.elapsed_secs_f64(),
            &mut auxiliary.mission_runtime,
            &mut auxiliary.presentation,
            &mut actor_commands,
            &mut effect_runtime,
            &mut auxiliary.pending_actor_effects,
            &mut runtime,
            &tutorial_voices,
            &mut auxiliary.nanocom_messages,
        );
    }
}

pub(in super::super) fn tutorial_actor_effect_world_position(actor_position: Vec3, offset: [f32; 3]) -> Vec3 {
    actor_position + unity_to_native_vector(Vec3::from_array(offset))
}

pub(in super::super) fn tutorial_movement_stage_effects(
    stage: TutorialStage,
) -> Vec<TutorialMovementStageEffect> {
    use TutorialMovementStageEffect::{Clear, Marker, Preload};

    let start = TUTORIAL_START_UNITY_POSITION;
    match stage {
        TutorialStage::Movement(MovementStage::PostIntro) => {
            vec![Preload { source_line: 2845 }]
        }
        TutorialStage::Movement(MovementStage::LookRight) => vec![
            Clear { source_line: 2875 },
            Marker {
                unity_position: start + Vec3::new(0.0, 2.0, -8.0),
                source_line: 2876,
            },
        ],
        TutorialStage::Movement(MovementStage::LookLeft) => vec![
            Clear { source_line: 2890 },
            Marker {
                unity_position: start + Vec3::new(8.0, 2.0, 0.0),
                source_line: 2891,
            },
        ],
        TutorialStage::Movement(MovementStage::LookUp) => vec![
            Clear { source_line: 2906 },
            Marker {
                unity_position: start + Vec3::new(7.0, 5.0, 0.0),
                source_line: 2907,
            },
        ],
        TutorialStage::Movement(MovementStage::LookDown) => vec![
            Clear { source_line: 2921 },
            Marker {
                unity_position: start + Vec3::new(4.0, 1.0, 0.0),
                source_line: 2922,
            },
        ],
        TutorialStage::Movement(MovementStage::MoveForward) => vec![
            Clear { source_line: 2933 },
            Marker {
                unity_position: start + Vec3::new(9.0, 1.0, 0.0),
                source_line: 2934,
            },
        ],
        TutorialStage::Movement(MovementStage::MoveBackward) => {
            vec![Clear { source_line: 2947 }]
        }
        TutorialStage::Movement(MovementStage::MoveAndSteer) => {
            vec![Clear { source_line: 2959 }]
        }
        TutorialStage::Movement(MovementStage::ReachLedge) => vec![
            Clear { source_line: 2966 },
            Marker {
                unity_position: Vec3::new(555.82, -104.0, 654.2),
                source_line: 2970,
            },
        ],
        TutorialStage::Movement(MovementStage::JumpAndLand) => vec![
            Clear { source_line: 2981 },
            Marker {
                unity_position: Vec3::new(566.0, -99.0, 665.0),
                source_line: 2983,
            },
        ],
        _ => Vec::new(),
    }
}

pub(in super::super) fn tutorial_movement_stage_camera_target(stage: TutorialStage) -> Option<Vec3> {
    match stage {
        TutorialStage::Movement(MovementStage::MoveForward) => {
            Some(TUTORIAL_START_UNITY_POSITION + Vec3::new(9.0, 1.0, 0.0))
        }
        TutorialStage::Movement(MovementStage::JumpAndLand) => Some(Vec3::new(566.0, -99.0, 665.0)),
        _ => None,
    }
}

pub(in super::super) fn tutorial_movement_timeout_restarts(
    stage: MovementStage,
    elapsed_seconds: f32,
) -> bool {
    elapsed_seconds > 25.0
        && matches!(
            stage,
            MovementStage::LookRight
                | MovementStage::LookLeft
                | MovementStage::LookUp
                | MovementStage::LookDown
                | MovementStage::JumpAndLand
        )
}

pub(super) fn tutorial_gameplay_nano_matches(nano: &TutorialNanoGameplayState) -> bool {
    nano.loadout()
        == Some(TutorialNanoGameplayLoadout {
            nano_id: ffone_client::tutorial_nano_gameplay::TUTORIAL_BUTTERCUP_NANO_ID,
            skill_id: ffone_client::tutorial_nano_gameplay::TUTORIAL_BUTTERCUP_SKILL_ID,
        })
        && nano.world_presentation().is_none()
}

/// Commit both halves of the local grant. Timeline playback and its skip
/// finisher can deliver the same grant in either order.
pub(in super::super) fn equip_tutorial_nano(
    runtime: &mut RuntimeStatus,
    queue: &mut TutorialNanoGameplayCommandQueue,
    slot: usize,
    nano_id: i16,
    skill_id: i16,
    stamina: i16,
) -> bool {
    let Some(target) = runtime.nano_slots.get_mut(slot) else {
        return false;
    };
    if target.nano_id != Some(nano_id) || target.skill_id != skill_id {
        *target = RuntimeNanoSlot {
            nano_id: Some(nano_id),
            skill_id,
            stamina,
            active: false,
        };
    }
    queue.equip(nano_id, skill_id, i32::from(stamina));
    true
}

/// Reconcile a resumed stage or a lost slot before accepting gameplay input.
/// The chapter is the local grant authority; shard inventory is still the
/// pre-tutorial inventory until tutorial completion has been saved.
pub(in super::super) fn ensure_tutorial_nano_loadout(
    stage: TutorialStage,
    runtime: &mut RuntimeStatus,
    nano: &TutorialNanoGameplayState,
    queue: &mut TutorialNanoGameplayCommandQueue,
) {
    if !matches!(
        stage,
        TutorialStage::Infection(InfectionStage::ExitPrompt | InfectionStage::WarpOut)
            | TutorialStage::NanoPower(_)
    ) {
        return;
    }
    let expected = TutorialNanoGameplayLoadout {
        nano_id: 1,
        skill_id: 1,
    };
    let gameplay_matches = tutorial_gameplay_nano_matches(nano);
    let slot = &mut runtime.nano_slots[0];
    if slot.nano_id != Some(expected.nano_id) || slot.skill_id != expected.skill_id {
        *slot = RuntimeNanoSlot {
            nano_id: Some(expected.nano_id),
            skill_id: expected.skill_id,
            stamina: if gameplay_matches {
                nano.stamina() as i16
            } else {
                TUTORIAL_BUTTERCUP_INITIAL_STAMINA as i16
            },
            active: gameplay_matches && nano.is_active(),
        };
    }
    if !gameplay_matches {
        slot.active = false;
        slot.stamina = TUTORIAL_BUTTERCUP_INITIAL_STAMINA as i16;
        queue.equip(
            expected.nano_id,
            expected.skill_id,
            TUTORIAL_BUTTERCUP_INITIAL_STAMINA,
        );
    } else {
        slot.active = nano.is_active();
        slot.stamina = nano.stamina() as i16;
    }
}

pub(in super::super) fn reconcile_tutorial_nano_activation(
    tutorial: &mut TutorialSession,
    nano: &TutorialNanoGameplayState,
) {
    let active = nano.is_active() && tutorial_gameplay_nano_matches(nano);
    match tutorial.progress.stage() {
        Some(TutorialStage::NanoPower(NanoPowerStage::SummonNano)) => {
            // A previous activation event can survive a dismissal; conversely
            // a Nano summoned before this stage need not emit another event.
            if active {
                tutorial
                    .progress
                    .receive_event(TutorialEvent::NanoActive, 1);
            } else if tutorial.progress.event_value(TutorialEvent::NanoActive) != 0 {
                tutorial.progress.clear_event(TutorialEvent::NanoActive);
            }
        }
        Some(TutorialStage::NanoPower(NanoPowerStage::UseNanoPower))
            if !active
                && tutorial.progress.event_value(TutorialEvent::UseSkill) == 0
                && tutorial.progress.event_value(TutorialEvent::DeadNpc) == 0 =>
        {
            tutorial.init_step(NanoPowerStage::SummonNano as i16);
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn drive_local_tutorial(
    mut commands: Commands,
    time: Res<Time>,
    input: (Res<ButtonInput<KeyCode>>, Option<Res<GamepadActionState>>),
    state: Res<State<ClientState>>,
    bridge: Res<NetworkBridge>,
    asset_server: Res<AssetServer>,
    native_audio_catalog: Res<NativeAudioCatalog>,
    language: Res<Language>,
    mut tutorial: ResMut<TutorialSession>,
    actor_runtime: TutorialActorDrive,
    mut runtime: ResMut<RuntimeStatus>,
    mut players: Query<
        (
            Entity,
            &mut Transform,
            &mut LegacyPlayerController,
            &LegacyAvatarTargetFeed,
            &LegacyAvatarActionContext,
        ),
        (
            With<LocalPlayer>,
            Without<LegacyOrbitCamera>,
            Without<TutorialDome>,
        ),
    >,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    tutorial_voices: Query<Entity, With<TutorialVoiceAudio>>,
    tutorial_scene_audio: Query<
        Entity,
        (
            With<TutorialSceneAudio>,
            Without<TutorialLoopAudio>,
            Without<TutorialVoiceAudio>,
        ),
    >,
    tutorial_loop_audio: Query<(Entity, &TutorialLoopAudio)>,
) {
    let (keyboard, pad) = input;
    let TutorialActorDrive {
        mut tutorial_logic,
        mut native_mechanics,
        mut next_state,
        mut actor_commands,
        mut gameplay_nano_commands,
        gameplay_nano,
        mut effect_runtime,
        npc_snapshot,
        mut choreography_player,
        mut choreography_presentation,
        mut choreography_issues,
        mut choreography_execution,
        mut mission_runtime,
        tutorial_content,
        audio_mix,
        mut gameplay_audio,
        localization,
        voice_language,
        mut voice_subtitles,
        mut auxiliary_presentation,
        mut ambient,
        mut tutorial_domes,
    } = actor_runtime;
    if *state.get() != ClientState::Tutorial {
        return;
    }
    if tutorial.completion_requested {
        return;
    }
    let Ok((player_entity, mut transform, mut controller, target_feed, action_context)) =
        players.single_mut()
    else {
        return;
    };
    if !controller.movement_enabled {
        return;
    }
    let previous_stage = tutorial.progress.stage();
    reconcile_tutorial_nano_activation(&mut tutorial, &gameplay_nano);
    if previous_stage == Some(TutorialStage::NanoPower(NanoPowerStage::UseNanoPower))
        && tutorial.progress.stage() == Some(TutorialStage::NanoPower(NanoPowerStage::SummonNano))
    {
        stop_tutorial_dialogue(&mut mission_runtime, TutorialDialogue::NanoPowerReminder);
        stop_tutorial_voice(&mut commands, &tutorial_voices);
        voice_subtitles.clear();
    }
    let Some(stage) = tutorial.progress.stage() else {
        runtime.message = format!(
            "Tutorial blocked on unknown legacy state ({}, {})",
            tutorial.progress.chapter(),
            tutorial.progress.step()
        );
        return;
    };
    ensure_tutorial_nano_loadout(
        stage,
        &mut runtime,
        &gameplay_nano,
        &mut gameplay_nano_commands,
    );
    native_mechanics.sync_stable_stage(stage);
    tutorial_logic.npcs = *npc_snapshot.get();
    let target_selection = select_legacy_targets(target_feed, action_context);
    tutorial_logic.ui.hostile_target_selected = target_selection
        .focused_npc
        .is_some_and(|target| matches!(target.kind, LegacyTargetKind::Npc { team } if team != 1));

    let observed_stage = tutorial.observed_stage;
    // Voice and subtitle duration never gate gameplay or choreography.
    let tutorial_delta = time.delta_secs();
    if observed_stage != Some(stage) {
        // Outgoing audio was cancelled at the transition below. Do not stop
        // again here: the new stage's auxiliary may already have spoken in
        // the previous frame after drive_local_tutorial committed that edge.
        cancel_tutorial_step_sequence(observed_stage, Some(stage), &mut mission_runtime);
        tutorial.observed_stage = Some(stage);
        tutorial.step_elapsed = 0.0;
        tutorial.step_origin_position = transform.translation;
        tutorial.step_origin_yaw = controller.yaw_degrees;
        tutorial.jump_seen = controller.jumping;
        tutorial.scene = stage.metadata().scene;
        tutorial.presentation_cursor = 0;
        apply_tutorial_movement_stage_effects(stage, &mut effect_runtime);
        apply_tutorial_movement_stage_camera(stage, &mut choreography_presentation);
        match stage {
            TutorialStage::Movement(MovementStage::PostIntro) => {
                for (_, mut dome_transform) in &mut tutorial_domes {
                    dome_transform.translation =
                        unity_to_native_vector(Vec3::new(550.0, -110.0, 650.0));
                    dome_transform.scale = Vec3::new(1.0, 2.0, 1.0);
                }
            }
            TutorialStage::Combat(CombatStage::PlanetFusionOutro) => {
                for (entity, _) in &mut tutorial_domes {
                    commands.entity(entity).despawn();
                }
            }
            _ => {}
        }
        if let Some(cue) = stage.metadata().entry_voice_cue
            && tutorial_auxiliary_for_stage(stage).is_none()
        {
            stop_tutorial_voice(&mut commands, &tutorial_voices);
            if let Err(error) = spawn_tutorial_voice(
                &mut commands,
                &asset_server,
                &native_audio_catalog,
                &voice_language.effective,
                cue,
            ) {
                runtime.message = error;
            }
            start_tutorial_voice_subtitle(
                &mut voice_subtitles,
                &tutorial_content,
                &mut mission_runtime,
                &mut runtime,
                &localization,
                &language,
                cue,
                time.elapsed_secs_f64(),
            );
        }
        if let Some(sequence) = tutorial_auxiliary_for_stage(stage)
            && mission_runtime.auxiliary.active() != Some(sequence)
        {
            mission_runtime.auxiliary.start(sequence);
            mission_runtime.basic_arrow_started =
                sequence == TutorialAuxiliarySequence::BasicArrowKey;
        }
        append_tutorial_stage_instruction(
            observed_stage,
            stage,
            &mut mission_runtime,
            &localization,
            &language,
        );
    } else {
        tutorial.step_elapsed += tutorial_delta;
    }
    tutorial_logic.tick(tutorial_delta);
    if controller.jumping {
        tutorial.jump_seen = true;
        tutorial.progress.receive_event(TutorialEvent::Jump, 1);
    }
    let timer_seconds = tutorial_logic.reminder_elapsed_seconds.max(0.0) as u32;
    tutorial.progress.set_timer_seconds(timer_seconds);

    if keyboard.just_pressed(KeyCode::Digit1)
        || pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Nano1)) {
        match tutorial_nano_shortcut_action(
            native_mechanics.is_locked(TutorialInputLock::NanoActive),
            runtime.nano_slots[0].nano_id.is_some(),
            runtime.nano_slots[0].active,
            tutorial_gameplay_nano_matches(&gameplay_nano) && gameplay_nano.entity().is_some(),
        ) {
            Some(TutorialNanoShortcutAction::Summon) => {
                gameplay_nano_commands.summon(player_entity);
            }
            Some(TutorialNanoShortcutAction::Dismiss) => gameplay_nano_commands.dismiss(),
            None => {}
        }
    }

    let active_camera = cameras
        .iter()
        .find(|(_, _, orbit)| orbit.target == player_entity);
    let elapsed = tutorial.step_elapsed;
    let origin = tutorial.step_origin_position;
    let origin_yaw = tutorial.step_origin_yaw;
    let skip_scene = keyboard.just_pressed(KeyCode::Space)
        || pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Jump));
    let mut choreography_events = if tutorial.scene == TutorialScene::None {
        choreography_player.stop();
        Vec::new()
    } else if choreography_player.active_scene() != Some(tutorial.scene) {
        choreography_execution.player_start = if tutorial.scene == TutorialScene::BasicMove {
            // `cntutorialscript.StartPos` is a serialized, fixed tutorial
            // coordinate. BasicMoveEvent uses that exact point for the
            // Samurai Jack/Oil Ogre -> player camera flight; substituting the
            // player's live position changes the entire spline and can send
            // it through the terrain.
            unity_to_native_vector(TUTORIAL_START_UNITY_POSITION)
        } else {
            transform.translation
        };
        choreography_presentation.begin_scene(tutorial.scene);
        let mut events = choreography_player.start(tutorial.scene);
        if skip_scene {
            events.extend(choreography_player.skip());
        }
        events
    } else {
        choreography_player.tick_with_wait_resolution(tutorial_delta, skip_scene, |wait| {
            tutorial_blocking_wait_is_resolved(wait, &effect_runtime)
        })
    };
    let mut scene_completion = SceneCompletion::Pending;
    for event in choreography_events.iter().copied() {
        if let ChoreographyPlaybackEvent::Finished { completion, .. } = event {
            scene_completion = match completion {
                ChoreographyCompletion::Natural => SceneCompletion::Natural,
                ChoreographyCompletion::Skipped => SceneCompletion::Skipped,
            };
        }
        choreography_presentation.apply(event);
    }
    choreography_execution
        .pending
        .append(&mut choreography_events);
    choreography_issues.extend(choreography_player.take_issues());
    if scene_completion == SceneCompletion::Skipped {
        stop_tutorial_voice(&mut commands, &tutorial_voices);
        voice_subtitles.clear();
    }
    let presentation_elapsed = match scene_completion {
        SceneCompletion::Pending => Some(choreography_player.presentation_elapsed_seconds()),
        SceneCompletion::Natural => tutorial_scene_choreography(tutorial.scene)
            .map(|scene| scene.deterministic_duration_seconds),
        SceneCompletion::Skipped => None,
    };
    if let Some(presentation_elapsed) = presentation_elapsed {
        play_tutorial_scene_timeline(
            &mut commands,
            &asset_server,
            &native_audio_catalog,
            &audio_mix,
            &localization,
            &language,
            &voice_language.effective,
            tutorial.scene,
            presentation_elapsed,
            &mut tutorial.presentation_cursor,
            &mut runtime,
            &tutorial_content,
            &mut voice_subtitles,
            &mut mission_runtime,
            time.elapsed_secs_f64(),
            &tutorial_voices,
            &tutorial_loop_audio,
        );
    }

    match stage {
        TutorialStage::Movement(MovementStage::AwaitIntro) => {
            tutorial.scene = TutorialScene::BasicMove;
            tutorial.init_step(MovementStage::IntroCutscene as i16);
            runtime.message = "Tutorial: BasicMoveEvent".to_owned();
        }
        TutorialStage::Movement(MovementStage::IntroCutscene) if scene_completion.is_complete() => {
            tutorial.scene = TutorialScene::None;
            tutorial.init_step(MovementStage::PostIntro as i16);
            runtime.message = "Tutorial: Computress_Tut02".to_owned();
        }
        TutorialStage::Movement(MovementStage::IntroCutscene) => {}
        TutorialStage::Movement(MovementStage::PostIntro) if elapsed >= 1.5 => {
            tutorial.init_step(MovementStage::LookRight as i16);
        }
        TutorialStage::Movement(MovementStage::LookRight) => {
            let target = TUTORIAL_START_UNITY_POSITION + Vec3::new(0.0, 1.0, -8.0);
            if active_camera.is_some_and(|(camera, camera_transform, _)| {
                tutorial_target_in_center(camera, camera_transform, target, 300.0, f32::INFINITY)
            }) {
                tutorial.init_step(MovementStage::LookLeft as i16);
            } else if tutorial_movement_timeout_restarts(MovementStage::LookRight, elapsed) {
                tutorial.observed_stage = None;
            }
        }
        TutorialStage::Movement(MovementStage::LookLeft) => {
            let target = TUTORIAL_START_UNITY_POSITION + Vec3::new(8.0, 1.0, 0.0);
            if active_camera.is_some_and(|(camera, camera_transform, _)| {
                tutorial_target_in_center(camera, camera_transform, target, 300.0, f32::INFINITY)
            }) {
                tutorial.init_step(MovementStage::LookUp as i16);
            } else if tutorial_movement_timeout_restarts(MovementStage::LookLeft, elapsed) {
                tutorial.observed_stage = None;
            }
        }
        TutorialStage::Movement(MovementStage::LookUp) => {
            let target = TUTORIAL_START_UNITY_POSITION + Vec3::new(7.0, 5.0, 0.0);
            if active_camera.is_some_and(|(camera, camera_transform, _)| {
                let half_width = camera
                    .logical_viewport_size()
                    .map_or(0.0, |viewport| viewport.x * 0.5);
                tutorial_target_in_center(camera, camera_transform, target, half_width, 150.0)
            }) {
                tutorial.init_step(MovementStage::LookDown as i16);
            } else if tutorial_movement_timeout_restarts(MovementStage::LookUp, elapsed) {
                tutorial.observed_stage = None;
            }
        }
        TutorialStage::Movement(MovementStage::LookDown) => {
            let target = TUTORIAL_START_UNITY_POSITION + Vec3::new(4.0, 0.0, 0.0);
            if active_camera.is_some_and(|(camera, camera_transform, _)| {
                let half_width = camera
                    .logical_viewport_size()
                    .map_or(0.0, |viewport| viewport.x * 0.5);
                tutorial_target_in_center(camera, camera_transform, target, half_width, 50.0)
            }) {
                tutorial.init_step(MovementStage::MoveForward as i16);
            } else if tutorial_movement_timeout_restarts(MovementStage::LookDown, elapsed) {
                tutorial.observed_stage = None;
            }
        }
        TutorialStage::Movement(MovementStage::MoveForward) => {
            let target =
                unity_to_native_vector(TUTORIAL_START_UNITY_POSITION + Vec3::new(9.0, 1.0, 0.0));
            if transform.translation.distance(target) < 3.0 {
                tutorial.init_step(MovementStage::MoveBackward as i16);
            }
        }
        TutorialStage::Movement(MovementStage::MoveBackward) => {
            if horizontal_distance(transform.translation, origin) > 5.0 {
                tutorial.init_step(MovementStage::MoveAndSteer as i16);
            }
        }
        TutorialStage::Movement(MovementStage::MoveAndSteer) => {
            if horizontal_distance(transform.translation, origin) > 5.0
                && shortest_angle_delta(controller.yaw_degrees, origin_yaw) > 40.0
            {
                tutorial.init_step(MovementStage::ReachLedge as i16);
            }
        }
        TutorialStage::Movement(MovementStage::ReachLedge) => {
            let target = unity_to_native_vector(Vec3::new(555.82, -105.0, 654.2));
            if transform.translation.distance(target) < 3.0 {
                tutorial.init_step(MovementStage::JumpAndLand as i16);
            }
        }
        TutorialStage::Movement(MovementStage::JumpAndLand) => {
            let target = unity_to_native_vector(Vec3::new(566.0, -99.0, 665.0));
            if transform.translation.distance(target) < 3.0
                && tutorial.jump_seen
                && !controller.jumping
            {
                let _ = tutorial.init_chapter(1);
                effect_runtime
                    .enqueue(TutorialEffectRuntimeCommand::ClearTracked { source_line: 2991 });
                runtime.message = "Tutorial: BasicCombatEvent_A".to_owned();
            } else if tutorial_movement_timeout_restarts(MovementStage::JumpAndLand, elapsed) {
                tutorial.observed_stage = None;
            }
        }
        TutorialStage::Movement(MovementStage::PostIntro) => {}
        TutorialStage::Combat(_)
        | TutorialStage::Minimap(_)
        | TutorialStage::Mission(_)
        | TutorialStage::Infection(_)
        | TutorialStage::NanoPower(_) => {
            let observation = TutorialObservation {
                npcs: tutorial_logic.npcs,
                ui: tutorial_logic.ui,
                locale: if language.effective == "en" {
                    TutorialLocaleBranch::OriginalEnglish
                } else {
                    TutorialLocaleBranch::Localized
                },
                delay_completed: tutorial_logic.delay_completed(),
                scene_completion,
                ..default()
            }
            .with_progress(&tutorial.progress);
            match evaluate_progress(&tutorial.progress, &observation) {
                Ok(decision) => apply_tutorial_decision(
                    decision.reset,
                    decision.next_stage,
                    &decision.intents,
                    &mut commands,
                    &bridge,
                    &mut tutorial,
                    &mut tutorial_logic,
                    &mut native_mechanics,
                    &mut auxiliary_presentation,
                    &mut actor_commands,
                    &mut next_state,
                    &mut gameplay_nano_commands,
                    &mut mission_runtime,
                    gameplay_audio.as_deref_mut(),
                    &mut voice_subtitles,
                    &tutorial_content,
                    &mut ambient,
                    &mut choreography_player,
                    &mut runtime,
                    &mut transform,
                    &mut controller,
                    &tutorial_voices,
                    &tutorial_scene_audio,
                    &tutorial_loop_audio,
                ),
                Err(error) => {
                    runtime.message = format!("Tutorial transition error: {error:?}");
                }
            }
        }
    }
    // Cancel before the auxiliary driver runs later in this same Update.
    // A delayed movement hint must never emit into the next stage/cutscene.
    if tutorial.progress.stage() != Some(stage) {
        stop_tutorial_voice(&mut commands, &tutorial_voices);
        voice_subtitles.clear();
        cancel_tutorial_step_sequence(Some(stage), tutorial.progress.stage(), &mut mission_runtime);
    }
}

pub(in super::super) fn cancel_tutorial_step_sequence(
    previous: Option<TutorialStage>,
    current: Option<TutorialStage>,
    mission: &mut TutorialMissionRuntime,
) {
    if previous != current
        && let Some(sequence) = previous.and_then(tutorial_auxiliary_for_stage)
        && mission.auxiliary.active() == Some(sequence)
    {
        mission.auxiliary.stop();
    }
}
