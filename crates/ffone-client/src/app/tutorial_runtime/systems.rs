use super::*;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_tutorial_auxiliary_emission(
    emission: TutorialAuxiliaryEmission,
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    language: &Language,
    voice_locale: &str,
    localization: &Localization,
    tutorial_content: &TutorialMissionContent,
    voice_subtitles: &mut TutorialVoiceSubtitleState,
    now_seconds: f64,
    mission_runtime: &mut TutorialMissionRuntime,
    presentation: &mut TutorialAuxiliaryPresentation,
    actor_commands: &mut TutorialActorCommandQueue,
    effect_runtime: &mut TutorialEffectRuntime,
    pending_actor_effects: &mut PendingTutorialActorEffects,
    runtime: &mut RuntimeStatus,
    tutorial_voices: &Query<Entity, With<TutorialVoiceAudio>>,
    nanocom_messages: &mut NanocomMessageUiModel,
) {
    if presentation.sequence != Some(emission.sequence) {
        *presentation = TutorialAuxiliaryPresentation {
            sequence: Some(emission.sequence),
            ..default()
        };
    }
    let source_line = tutorial_auxiliary_definition(emission.sequence)
        .source
        .start_line;
    match emission.action {
        TutorialAuxiliaryAction::PreloadEffect { effect_id } => {
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
                effect_id,
                source_line,
            });
        }
        TutorialAuxiliaryAction::SendMessageBox {
            npc_type,
            message,
            message_type,
        } => {
            presentation.message_box = Some((npc_type, message, message_type));
            let title_fallback = tutorial_content.npc_name(npc_type).unwrap_or("UNKNOWN NPC");
            let localized_title =
                LocalizedText::new(format!("content.npc.{npc_type}.name"), title_fallback);
            let localized_body = localized_tutorial_auxiliary_text(message, tutorial_content)
                .unwrap_or_else(|| {
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "")
                });
            let title = localization.text(language, &localized_title);
            let body = localization.text(language, &localized_body);
            if npc_type == TUTORIAL_NANOCOM_NPC_TYPE
                && message_type == TUTORIAL_NANOCOM_MESSAGE_TYPE
            {
                let voice_owner = tutorial_content
                    .gameplay_npc(npc_type)
                    .map(|npc| npc.move_voice_owner.as_str());
                let request_id = nanocom_messages.enqueue_type_9_localized(
                    localized_title,
                    localized_body,
                    NANOCOM_NUMBUH_TWO_ICON_PATH,
                    voice_owner,
                );
                // The voice chat line below is this message's only chat copy.
                nanocom_messages.discard_chat_echo(request_id);
            }
            mission_runtime.push_voice_chat_line(format!("{title}: {body}"));
        }
        TutorialAuxiliaryAction::Voice { clip, subtitle, .. } => {
            stop_tutorial_voice(commands, tutorial_voices);
            if let Err(error) =
                spawn_tutorial_voice(commands, asset_server, catalog, voice_locale, clip)
            {
                runtime.message = error;
            }
            start_tutorial_voice_subtitle(
                voice_subtitles,
                tutorial_content,
                mission_runtime,
                runtime,
                localization,
                language,
                clip,
                now_seconds,
            );
            // `subtitle` is the exact VoiceOut chat payload. It is rendered by
            // TutorialVoiceSubtitleState, not by the independent StringList
            // used by SubText/SubText2.
            let _ = subtitle;
        }
        TutorialAuxiliaryAction::ShowPicture {
            position,
            resource,
            pivot,
        } => {
            presentation.picture = Some((resource, position, pivot));
            presentation.picture_touched = true;
        }
        TutorialAuxiliaryAction::ShowCursor {
            position,
            resource,
            pivot,
            ..
        } => {
            presentation.cursor = Some((resource, position, pivot));
            presentation.cursor_touched = true;
        }
        TutorialAuxiliaryAction::HidePicture => {
            presentation.picture = None;
            presentation.cursor = None;
            presentation.picture_touched = true;
            presentation.cursor_touched = true;
        }
        TutorialAuxiliaryAction::Subtitle { channel, text } => {
            if let Some(text) =
                resolve_tutorial_auxiliary_text(text, tutorial_content, localization, language)
            {
                mission_runtime.push_instruction_chat_line(text);
            }
            match channel {
                TutorialSubtitleChannel::Primary => {
                    presentation.primary_subtitle = Some(text);
                    presentation.primary_subtitle_touched = true;
                }
                TutorialSubtitleChannel::Secondary => {
                    presentation.secondary_subtitle = Some(text);
                    presentation.secondary_subtitle_touched = true;
                }
            }
        }
        TutorialAuxiliaryAction::ClearSubtitle => {
            presentation.primary_subtitle = None;
            presentation.secondary_subtitle = None;
            presentation.primary_subtitle_touched = true;
            presentation.secondary_subtitle_touched = true;
        }
        TutorialAuxiliaryAction::ClearEffects => {
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::ClearTracked { source_line });
        }
        TutorialAuxiliaryAction::SpawnNpc {
            legacy_server_position,
            npc_type,
            runtime_id,
        } => {
            actor_commands.spawn(TutorialNpcSpawn::new(
                runtime_id,
                npc_type,
                LegacySpawnPosition::centiunits(
                    (legacy_server_position[0] * 100.0) as i32,
                    (legacy_server_position[1] * 100.0) as i32,
                    (legacy_server_position[2] * 100.0) as i32,
                ),
                None,
            ));
        }
        TutorialAuxiliaryAction::SpawnEffectRelativeToActor {
            runtime_id,
            offset,
            effect_id,
        } => {
            pending_actor_effects
                .spawns
                .push_back(PendingTutorialActorEffect {
                    sequence: emission.sequence,
                    runtime_id,
                    offset,
                    effect_id,
                    source_line,
                });
        }
        TutorialAuxiliaryAction::SetNpcAngle {
            runtime_id,
            legacy_degrees,
        } => {
            actor_commands.set_native_rotation(
                runtime_id,
                ProtocolYawDegrees::new(legacy_degrees).native_root_rotation(),
            );
        }
        TutorialAuxiliaryAction::SetNpcAnimation {
            runtime_id,
            animation,
        } => {
            actor_commands.play_pose(runtime_id, animation, false);
        }
        TutorialAuxiliaryAction::SetFlag { flag, value } => match flag {
            TutorialBooleanFlag::MyPointEvent => mission_runtime.my_point_event = value,
            TutorialBooleanFlag::WayPointEvent => mission_runtime.waypoint_event = value,
        },
        TutorialAuxiliaryAction::SetWaypoint { target, enabled } => {
            mission_runtime.waypoint_actor_id = if enabled {
                match target {
                    TutorialWaypointTarget::DisabledAtZero => None,
                    TutorialWaypointTarget::ActorPosition { runtime_id } => Some(runtime_id),
                }
            } else {
                None
            };
        }
        TutorialAuxiliaryAction::SetFusionMatter { value } => {
            mission_runtime.fusion_matter = value;
        }
    }
}

pub(in super::super) fn apply_tutorial_movement_stage_effects(
    stage: TutorialStage,
    effect_runtime: &mut TutorialEffectRuntime,
) {
    for effect in tutorial_movement_stage_effects(stage) {
        match effect {
            TutorialMovementStageEffect::Preload { source_line } => {
                effect_runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
                    effect_id: TUTORIAL_MOVEMENT_MARKER_EFFECT_ID,
                    source_line,
                });
            }
            TutorialMovementStageEffect::Clear { source_line } => {
                effect_runtime.enqueue(TutorialEffectRuntimeCommand::ClearTracked { source_line });
            }
            TutorialMovementStageEffect::Marker {
                unity_position,
                source_line,
            } => {
                effect_runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: TUTORIAL_MOVEMENT_MARKER_EFFECT_ID,
                    placement: TutorialEffectPlacement::World {
                        position: unity_to_native_vector(unity_position),
                        rotation: Quat::IDENTITY,
                    },
                    scale: 2.0,
                    tracked: true,
                    name: None,
                    destroy_after_seconds: None,
                    source_line,
                });
            }
        }
    }
}

pub(in super::super) fn apply_tutorial_movement_stage_camera(
    stage: TutorialStage,
    presentation: &mut TutorialChoreographyPresentation,
) {
    let Some(unity_target) = tutorial_movement_stage_camera_target(stage) else {
        return;
    };
    presentation.camera.look_at = None;
    presentation.camera.resolved_look_at = Some(unity_to_native_vector(unity_target));
    presentation.camera.look_at_revision = presentation.camera.look_at_revision.wrapping_add(1);
}

pub(in super::super) fn apply_tutorial_input_stage_transition(
    native: &mut TutorialNativeMechanics,
    next_stage: TutorialStage,
    intents: &[TutorialIntent],
) {
    for &intent in intents {
        if matches!(
            intent,
            TutorialIntent::PushInputFilter | TutorialIntent::PopInputFilter
        ) {
            let application = native.apply_intent(intent);
            debug_assert_ne!(application, TutorialNativeIntentApplication::Unhandled);
        }
    }
    native.sync_stable_stage(next_stage);
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_tutorial_decision(
    reset: Option<TutorialProgressReset>,
    next_stage: TutorialStage,
    intents: &[TutorialIntent],
    commands: &mut Commands,
    bridge: &NetworkBridge,
    tutorial: &mut TutorialSession,
    logic: &mut TutorialLogicRuntime,
    native_mechanics: &mut TutorialNativeMechanics,
    auxiliary_presentation: &mut TutorialAuxiliaryPresentation,
    actor_commands: &mut TutorialActorCommandQueue,
    next_state: &mut NextState<ClientState>,
    gameplay_nano_commands: &mut TutorialNanoGameplayCommandQueue,
    mission_runtime: &mut TutorialMissionRuntime,
    mut gameplay_audio: Option<&mut GameplayAudioRuntime>,
    voice_subtitles: &mut TutorialVoiceSubtitleState,
    tutorial_content: &TutorialMissionContent,
    ambient: &mut TutorialAmbientRuntime,
    choreography_player: &mut TutorialChoreographyPlayer,
    runtime: &mut RuntimeStatus,
    player_transform: &mut Transform,
    player_controller: &mut LegacyPlayerController,
    tutorial_voices: &Query<Entity, With<TutorialVoiceAudio>>,
    tutorial_scene_audio: &Query<
        Entity,
        (
            With<TutorialSceneAudio>,
            Without<TutorialLoopAudio>,
            Without<TutorialVoiceAudio>,
        ),
    >,
    tutorial_loop_audio: &Query<(Entity, &TutorialLoopAudio)>,
) {
    if let Some(reset) = reset {
        let (next_chapter, next_step) = next_stage.legacy();
        match reset {
            TutorialProgressReset::InitStep => {
                tutorial.init_step(next_step);
                logic.restart_reminder();
                logic.delay_remaining_seconds = None;
            }
            TutorialProgressReset::DirectStepWrite => {
                tutorial.write_step_preserving_state(next_step);
            }
            TutorialProgressReset::InitChapter => {
                if next_step != 0 {
                    runtime.message = format!(
                        "Tutorial transition rejected: InitChapter target {next_stage:?} is not step 0"
                    );
                    return;
                }
                if let Err(chapter) = tutorial.init_chapter(next_chapter) {
                    runtime.message =
                        format!("Tutorial transition rejected: invalid chapter {chapter}");
                    return;
                }
                if let Some(cue) = tutorial_ambient_after_chapter_init(next_stage) {
                    ambient.request(Some(cue));
                }
                logic.restart_reminder();
                logic.delay_remaining_seconds = None;
            }
        }
    }

    // `SetTutorialInputPush/Pop` belongs to the stage being exited. Applying
    // it after binding `next_stage` either snapshots the new restricted mask
    // or, for Pop, overwrites the new stage's permissions with the old mask.
    // The latter left Attack locked on Mission/ObjectiveCombat, so the Oil
    // Ogre could spawn but could never be defeated.
    apply_tutorial_input_stage_transition(native_mechanics, next_stage, intents);
    for &intent in intents {
        if matches!(
            intent,
            TutorialIntent::PushInputFilter | TutorialIntent::PopInputFilter
        ) {
            logic
                .deferred_intents
                .retain(|deferred| *deferred != intent);
        }
    }
    let mut loop_audio_stopped = false;
    for &intent in intents {
        if matches!(
            intent,
            TutorialIntent::PushInputFilter | TutorialIntent::PopInputFilter
        ) {
            continue;
        }
        if apply_tutorial_native_intent(intent, native_mechanics, auxiliary_presentation) {
            logic
                .deferred_intents
                .retain(|deferred| *deferred != intent);
            continue;
        }
        let previous_sequence = mission_runtime.auxiliary.active();
        if apply_tutorial_dialogue_intent(intent, mission_runtime, actor_commands, logic) {
            if previous_sequence != mission_runtime.auxiliary.active() {
                stop_tutorial_voice(commands, tutorial_voices);
                voice_subtitles.clear();
            }
            continue;
        }
        match intent {
            TutorialIntent::SpawnNpc(spawn) => {
                actor_commands.spawn(spawn);
            }
            TutorialIntent::DeleteNpc(id) => {
                actor_commands.delete(id);
            }
            TutorialIntent::FaceNpc { id, target } => {
                actor_commands.face_actor(id, target);
            }
            TutorialIntent::PlayNpcAnimation { id, clip, once } => {
                actor_commands.play_pose(id, clip, once);
            }
            TutorialIntent::AttackNpc(id) => {
                actor_commands.attempt_player_attack(id, player_transform.translation);
            }
            TutorialIntent::ConfigureDemoMonster { dont_kill } => {
                actor_commands.configure_demo_monster(dont_kill);
            }
            TutorialIntent::StartTask(task_id) => {
                match mission_runtime.start_task(tutorial_content, task_id) {
                    Ok(true) => {
                        // `present_tutorial_mission_dialogue` presents the start
                        // edge, like an outgoing task's start after `EndTask`.
                        tutorial.progress.receive_event(TutorialEvent::TaskStart, 1);
                    }
                    Ok(false) => {}
                    Err(error) => {
                        runtime.message = format!("Tutorial StartTask({task_id}) rejected: {error}")
                    }
                }
            }
            TutorialIntent::ClearEventFlags => {
                tutorial.progress.clear_event_flags();
            }
            TutorialIntent::EndTask(task_id) => {
                match mission_runtime.complete_task(tutorial_content, task_id) {
                    Ok(mutation) => {
                        if let Some(audio) = gameplay_audio.as_deref_mut() {
                            mutation.queue_completion_audio(audio);
                        }
                        if mutation.state_changed {
                            tutorial.progress.receive_event(TutorialEvent::QuestEnd, 1);
                        }
                        if mutation.outgoing_started {
                            tutorial.progress.receive_event(TutorialEvent::TaskStart, 1);
                        }
                    }
                    Err(error) => {
                        runtime.message = format!("Tutorial EndTask({task_id}) rejected: {error}")
                    }
                }
            }
            TutorialIntent::SetFusionMatter(value) => {
                mission_runtime.fusion_matter = value;
            }
            TutorialIntent::SetWaypointToNpc(id) => {
                mission_runtime.waypoint_actor_id = Some(id);
            }
            TutorialIntent::ClearWaypoint => {
                mission_runtime.waypoint_actor_id = None;
            }
            TutorialIntent::StartScene(scene) => {
                mission_runtime.auxiliary.stop();
                mission_runtime.auxiliary_dialogue = None;
                voice_subtitles.clear();
                for entity in tutorial_scene_audio {
                    commands.entity(entity).despawn();
                }
                stop_tutorial_voice(commands, tutorial_voices);
                tutorial.scene = scene;
                tutorial.step_elapsed = 0.0;
                tutorial.presentation_cursor = 0;
            }
            TutorialIntent::FinishScene { scene, completion } => {
                voice_subtitles.clear();
                for entity in tutorial_scene_audio {
                    commands.entity(entity).despawn();
                }
                stop_tutorial_voice(commands, tutorial_voices);
                tutorial.scene = TutorialScene::None;
                tutorial.presentation_cursor = 0;
                runtime.message = format!("Tutorial scene {scene:?} finished ({completion:?})");
            }
            TutorialIntent::StartReminderTimer => {
                logic.restart_reminder();
                tutorial.progress.set_timer_seconds(0);
            }
            TutorialIntent::StartDelayMillis(milliseconds) => {
                logic.delay_remaining_seconds = Some(milliseconds as f32 / 1_000.0);
            }
            TutorialIntent::WarpPlayer(position) => {
                let destination = tutorial_client_position_to_native(position);
                player_transform.translation = destination;
                player_controller.apply_authoritative_teleport(destination);
            }
            TutorialIntent::EquipNano {
                slot,
                nano_id,
                skill_id,
            } => {
                let Ok(index) = usize::try_from(slot) else {
                    logic.defer(intent);
                    continue;
                };
                let (Ok(nano_id), Ok(skill_id)) = (i16::try_from(nano_id), i16::try_from(skill_id))
                else {
                    logic.defer(intent);
                    continue;
                };
                let Ok(stamina) = i16::try_from(TUTORIAL_BUTTERCUP_INITIAL_STAMINA) else {
                    logic.defer(intent);
                    runtime.message =
                        "Tutorial Nano equip rejected: initial stamina does not fit protocol i16"
                            .into();
                    continue;
                };
                if !equip_tutorial_nano(
                    runtime,
                    gameplay_nano_commands,
                    index,
                    nano_id,
                    skill_id,
                    stamina,
                ) {
                    logic.defer(intent);
                }
            }
            TutorialIntent::StopLoopSound => {
                if !loop_audio_stopped {
                    for (entity, _) in tutorial_loop_audio {
                        commands.entity(entity).despawn();
                    }
                    loop_audio_stopped = true;
                }
                choreography_player.stop_all_loops();
            }
            TutorialIntent::ExitTutorial => {
                request_tutorial_completion(bridge, tutorial, runtime, next_state);
            }
            unsupported => {
                logic.defer(unsupported);
                runtime.message = format!("Tutorial waiting for native mechanic: {unsupported:?}");
            }
        }
    }
}

pub(in super::super) fn apply_pending_tutorial_exit(mut commands: Commands, drive: TutorialExitDrive) {
    let TutorialExitDrive {
        mut tutorial,
        mut tutorial_overlay,
        mut logic,
        mut native,
        mut choreography_player,
        mut choreography_execution,
        mut mission_runtime,
        mut mission_model,
        mut auxiliary_presentation,
        mut actor_commands,
        mut nano_commands,
        mut gameplay_nano_commands,
        mut nano_state,
        mut player_commands,
        mut pending_actor_effects,
        mut ambient,
        mut actors,
        domes,
        ambient_audio,
        music_audio,
        loop_audio,
        mut players,
    } = drive;
    if !tutorial.completion_requested || tutorial.exit_teardown_applied {
        return;
    }
    tutorial.exit_teardown_applied = true;

    // Exact `cntutorialscript.ExitTutorial`: the script is disabled before
    // SAVE_CHAR_TUTOR/CHAR_SELECT completes, so this state must survive the
    // login-server acknowledgement and freeze every subsequent tutorial tick.
    for entity in &domes {
        commands.entity(entity).despawn();
    }
    for entity in &ambient_audio {
        commands.entity(entity).despawn();
    }
    for entity in &music_audio {
        commands.entity(entity).despawn();
    }
    for entity in &loop_audio {
        commands.entity(entity).despawn();
    }

    choreography_player.stop();
    choreography_player.stop_all_loops();
    choreography_execution.pending.clear();
    mission_runtime.auxiliary.stop();
    actor_commands.take_all();
    for mut actor in &mut actors {
        actor.interacting = false;
    }
    nano_commands.clear();
    if let Some(entity) = nano_state.entity() {
        commands.entity(entity).despawn();
    }
    nano_state.mark_absent();
    gameplay_nano_commands.clear();
    player_commands.clear();
    *mission_model = MissionUiModel::default();
    *pending_actor_effects = PendingTutorialActorEffects::default();
    *auxiliary_presentation = TutorialAuxiliaryPresentation::default();
    tutorial_overlay.hide_and_clear();
    *ambient = TutorialAmbientRuntime::default();
    logic.delay_remaining_seconds = None;
    logic.deferred_intents.clear();
    tutorial.scene = TutorialScene::None;
    tutorial.presentation_cursor = 0;
    let _ = native.apply_intent(TutorialIntent::SetInstanceMap(false));
    native.fail_closed_input();

    for mut controller in &mut players {
        controller.velocity = Vec3::ZERO;
        controller.movement_enabled = false;
    }
}
