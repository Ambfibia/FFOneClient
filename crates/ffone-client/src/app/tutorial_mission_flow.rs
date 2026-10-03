//! Tutorial dialogue, instructions, location tasks, warps and mission outbox.

use super::LocalPlayer;
use super::gameplay_ui_actions::*;
use super::npc_warp::NormalNpcWarpIdentity;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_ambience::TutorialVoiceAudio;
use super::tutorial_scene_audio::{request_tutorial_completion, stop_tutorial_voice};
use super::tutorial_session::{TutorialLogicRuntime, TutorialMissionRuntime, TutorialSession};
use super::tutorial_warp::*;
use bevy::prelude::*;
use ffone_client::{
    coordinates::ProtocolPosition,
    gameplay_audio::{GameplayAudioRuntime, LegacyNpcVoiceCue},
    gameplay_ui::{GameplayUiAction, GameplayUiOutbox},
    localization::{Language, Localization, localized_tutorial_instruction},
    mission_ui::{MissionUiModel, PendingMissionUiRequest},
    movement::LegacyPlayerController,
    network::NetworkBridge,
    tutorial::{
        InfectionStage, MinimapStage, MissionStage, MovementStage, NanoPowerStage, TutorialEvent,
        TutorialStage,
    },
    tutorial_actors::{TutorialActor, TutorialActorCommandQueue},
    tutorial_auxiliary_choreography::TutorialAuxiliarySequence,
    tutorial_logic::{TutorialDialogue, TutorialIntent},
    tutorial_mission_content::{
        TutorialMissionContent, TutorialWarpDefinition, TutorialWarpTarget,
    },
    tutorial_voice_subtitles::TutorialVoiceSubtitleState,
};

pub(super) fn tutorial_auxiliary_for_stage(stage: TutorialStage) -> Option<TutorialAuxiliarySequence> {
    match stage {
        TutorialStage::Movement(MovementStage::MoveForward) => {
            Some(TutorialAuxiliarySequence::BasicArrowKey)
        }
        TutorialStage::NanoPower(NanoPowerStage::UseNanoPower) => {
            Some(TutorialAuxiliarySequence::NanoPowerTalk1)
        }
        _ => None,
    }
}

pub(super) fn tutorial_instruction_is_emitted_by_auxiliary(stage: TutorialStage) -> bool {
    matches!(
        stage,
        TutorialStage::Movement(MovementStage::MoveForward)
            | TutorialStage::Minimap(MinimapStage::MinimapSequence)
            | TutorialStage::Mission(MissionStage::MissionMenu)
            | TutorialStage::Mission(MissionStage::RewardMenu)
            | TutorialStage::Infection(InfectionStage::ApproachButtercup)
            | TutorialStage::Infection(InfectionStage::OutsidePortal)
            | TutorialStage::Infection(InfectionStage::TalkDexter)
            | TutorialStage::Infection(InfectionStage::ExitPrompt)
            | TutorialStage::NanoPower(NanoPowerStage::UseNanoPower)
    )
}

pub(super) fn append_tutorial_stage_instruction(
    observed_stage: Option<TutorialStage>,
    stage: TutorialStage,
    mission_runtime: &mut TutorialMissionRuntime,
    localization: &Localization,
    language: &Language,
) {
    if observed_stage == Some(stage) || tutorial_instruction_is_emitted_by_auxiliary(stage) {
        return;
    }
    if let Some(instruction) = stage.metadata().instruction {
        mission_runtime.push_instruction_chat_line(
            localization.text(language, &localized_tutorial_instruction(instruction)),
        );
    }
}

pub(super) fn tutorial_auxiliary_for_dialogue(dialogue: TutorialDialogue) -> TutorialAuxiliarySequence {
    match dialogue {
        TutorialDialogue::Minimap => TutorialAuxiliarySequence::MinimapEvent,
        TutorialDialogue::TalkNumbuhTwoMission => TutorialAuxiliarySequence::TalkNumTwo1,
        TutorialDialogue::TalkNumbuhTwoReward => TutorialAuxiliarySequence::TalkNumTwo2,
        TutorialDialogue::TalkButtercup => TutorialAuxiliarySequence::TalkButtercup1,
        TutorialDialogue::TalkDexterOutside => TutorialAuxiliarySequence::TalkDexter1,
        TutorialDialogue::TalkDexterInside => TutorialAuxiliarySequence::TalkDexter2,
        TutorialDialogue::TalkDexterExit => TutorialAuxiliarySequence::TalkDexter3,
        TutorialDialogue::NanoPowerReminder => TutorialAuxiliarySequence::NanoPowerTalk1,
    }
}

pub(super) fn start_tutorial_dialogue(runtime: &mut TutorialMissionRuntime, dialogue: TutorialDialogue) {
    if !runtime.dialogues.contains(&dialogue) {
        runtime.dialogues.push(dialogue);
    }
    let sequence = tutorial_auxiliary_for_dialogue(dialogue);
    runtime.auxiliary_dialogue = Some(dialogue);
    if runtime.auxiliary.active() != Some(sequence) {
        runtime.auxiliary.start(sequence);
    }
}

pub(super) fn stop_tutorial_dialogue(runtime: &mut TutorialMissionRuntime, dialogue: TutorialDialogue) {
    runtime.dialogues.retain(|active| *active != dialogue);
    if runtime.auxiliary_dialogue == Some(dialogue) {
        runtime.auxiliary_dialogue = None;
        runtime.auxiliary.stop();
    }
}

pub(super) fn apply_tutorial_dialogue_intent(
    intent: TutorialIntent,
    runtime: &mut TutorialMissionRuntime,
    actor_commands: &mut TutorialActorCommandQueue,
    logic: &mut TutorialLogicRuntime,
) -> bool {
    match intent {
        TutorialIntent::StartDialogue(dialogue) => {
            start_tutorial_dialogue(runtime, dialogue);
        }
        TutorialIntent::StopDialogue(dialogue) => {
            // Retrobution stops only the matching coroutine here. The linked
            // NPC interaction and NpcIcon camera remain alive until CLOSE.
            stop_tutorial_dialogue(runtime, dialogue);
        }
        TutorialIntent::ExitUi => {
            runtime.dialogues.clear();
            runtime.auxiliary_dialogue = None;
            runtime.auxiliary.stop();
            actor_commands.clear_interactions();
            logic.ui.npc_icon_mode_visible = false;
        }
        _ => return false,
    }
    true
}

pub(super) fn tutorial_warp_is_available(
    definition: &TutorialWarpDefinition,
    mission_runtime: &TutorialMissionRuntime,
) -> bool {
    !mission_runtime
        .completed_warp_ids
        .contains(&definition.provenance.warp_id)
        && definition
            .required_task_id
            .is_none_or(|task_id| mission_runtime.task_is_active(task_id))
}

/// `cnMissionManager.Update` scans location tasks once per second.
pub(super) const TUTORIAL_LOCATION_TASK_SCAN_SECONDS: f32 = 1.0;

pub(super) fn tutorial_location_task_completion_candidate(
    content: &TutorialMissionContent,
    mission_runtime: &TutorialMissionRuntime,
    player_position: Vec3,
    actors: &[(i32, bool, Vec3)],
) -> Option<i32> {
    mission_runtime
        .active_tasks
        .iter()
        .copied()
        .find(|task_id| {
            let Ok(definition) = content.mission(*task_id) else {
                return false;
            };
            if definition.provenance.task_type != 2 {
                return false;
            }
            let terminator_npc_type = definition.provenance.terminator_npc_type;
            let Some(sight_range) = content
                .gameplay_npc(terminator_npc_type)
                .map(|definition| definition.sight_range())
            else {
                return false;
            };
            actors.iter().any(|(npc_type, alive, position)| {
                *alive
                    && *npc_type == terminator_npc_type
                    && position.distance(player_position) <= sight_range
            })
        })
}

pub(super) fn scan_tutorial_location_tasks(
    time: Res<Time>,
    content: Res<TutorialMissionContent>,
    mut tutorial: ResMut<TutorialSession>,
    mut mission_runtime: ResMut<TutorialMissionRuntime>,
    mut gameplay_audio: Option<ResMut<GameplayAudioRuntime>>,
    mut runtime: ResMut<RuntimeStatus>,
    players: Query<&Transform, With<LocalPlayer>>,
    actors: Query<(&TutorialActor, &Transform)>,
) {
    mission_runtime.location_scan_elapsed_seconds += time.delta_secs();
    if mission_runtime.location_scan_elapsed_seconds < TUTORIAL_LOCATION_TASK_SCAN_SECONDS {
        return;
    }
    mission_runtime.location_scan_elapsed_seconds = mission_runtime
        .location_scan_elapsed_seconds
        .rem_euclid(TUTORIAL_LOCATION_TASK_SCAN_SECONDS);

    let Ok(player) = players.single() else {
        return;
    };
    let actor_samples = actors
        .iter()
        .map(|(actor, transform)| (actor.npc_type, actor.is_alive(), transform.translation))
        .collect::<Vec<_>>();
    let Some(task_id) = tutorial_location_task_completion_candidate(
        &content,
        &mission_runtime,
        player.translation,
        &actor_samples,
    ) else {
        return;
    };

    match mission_runtime.complete_task(&content, task_id) {
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
            runtime.message = format!("Tutorial location task {task_id} rejected: {error}");
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn apply_tutorial_warp(
    content: &TutorialMissionContent,
    mission_runtime: &mut TutorialMissionRuntime,
    current_map: Option<i32>,
    npc_type: i32,
    warp_id: i32,
    required_task_id: Option<i32>,
    target: TutorialWarpTarget,
    player_transform: &mut Transform,
    player_controller: &mut LegacyPlayerController,
) -> Result<(), String> {
    validate_tutorial_warp(
        content,
        mission_runtime,
        current_map,
        npc_type,
        warp_id,
        required_task_id,
        target,
    )?;
    mission_runtime.completed_warp_ids.push(warp_id);
    let destination = ProtocolPosition::new([target.x, target.y, target.z]).to_native();
    player_transform.translation = destination;
    player_controller.apply_authoritative_teleport(destination);
    Ok(())
}

pub(super) fn validate_tutorial_warp(
    content: &TutorialMissionContent,
    mission_runtime: &TutorialMissionRuntime,
    current_map: Option<i32>,
    npc_type: i32,
    warp_id: i32,
    required_task_id: Option<i32>,
    target: TutorialWarpTarget,
) -> Result<(), String> {
    let definition = content.warp(npc_type).map_err(|error| error.to_string())?;
    if definition.provenance.warp_id != warp_id
        || definition.required_task_id != required_task_id
        || definition.target != target
    {
        return Err(format!(
            "warp payload for NPC type {npc_type} contradicts immutable content"
        ));
    }
    if current_map != Some(target.map_id) {
        return Err(format!(
            "warp {warp_id} requires map {}, current map is {current_map:?}",
            target.map_id
        ));
    }
    if mission_runtime.completed_warp_ids.contains(&warp_id) {
        return Err(format!("warp {warp_id} was already completed"));
    }
    if !tutorial_warp_is_available(definition, mission_runtime) {
        return Err(match required_task_id {
            Some(task_id) => format!("warp {warp_id} requires active task {task_id}"),
            None => format!("warp {warp_id} is unavailable"),
        });
    }

    Ok(())
}

pub(super) fn consume_tutorial_mission_outbox(
    mut commands: Commands,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut model: ResMut<MissionUiModel>,
    mut tutorial: ResMut<TutorialSession>,
    mut mission_runtime: ResMut<TutorialMissionRuntime>,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
    content: Res<TutorialMissionContent>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    actors: Query<(Entity, &TutorialActor)>,
    mut next_state: ResMut<NextState<ClientState>>,
    players: Query<(&Transform, &LegacyPlayerController), With<LocalPlayer>>,
    tutorial_voices: Query<Entity, With<TutorialVoiceAudio>>,
    mut voice_subtitles: Option<ResMut<TutorialVoiceSubtitleState>>,
    mut gameplay_audio: Option<ResMut<GameplayAudioRuntime>>,
) {
    let actions = outbox.drain_matching(tutorial_owns_gameplay_action);
    if tutorial.completion_requested {
        return;
    }
    for action in actions {
        if tutorial.completion_requested {
            break;
        }
        match action {
            GameplayUiAction::ConfirmTutorialExit {
                message_id,
                button_type,
            } => {
                if model.take_tutorial_exit_confirmation(message_id, button_type) {
                    request_tutorial_completion(
                        &bridge,
                        &mut tutorial,
                        &mut runtime,
                        &mut next_state,
                    );
                    *model = MissionUiModel::default();
                    break;
                } else {
                    runtime.message =
                        "Tutorial exit rejected: exact SystemMessage 47 confirmation was not pending"
                            .to_owned();
                }
            }
            GameplayUiAction::TaskStart { task_id, npc_id } => {
                if !matches!(
                    model.pending,
                    Some(PendingMissionUiRequest::TaskStart {
                        task_id: pending_task,
                        npc_id: pending_npc,
                    }) if pending_task == task_id && pending_npc == npc_id
                ) {
                    continue;
                }
                match mission_runtime.start_task(&content, task_id) {
                    Ok(started) => {
                        model.confirm_task_start(task_id);
                        if started {
                            tutorial.progress.receive_event(TutorialEvent::TaskStart, 1);
                        }
                    }
                    Err(error) => {
                        model.reject_pending(task_id);
                        runtime.message = format!("Tutorial TaskStart rejected: {error}");
                    }
                }
            }
            GameplayUiAction::QuestEnd {
                task_id,
                npc_id,
                box1_choice,
                box2_choice,
            } => {
                if !matches!(
                    model.pending,
                    Some(PendingMissionUiRequest::QuestEnd {
                        task_id: pending_task,
                        npc_id: pending_npc,
                        box1_choice: pending_box1,
                        box2_choice: pending_box2,
                    }) if pending_task == task_id
                        && pending_npc == npc_id
                        && pending_box1 == box1_choice
                        && pending_box2 == box2_choice
                ) {
                    continue;
                }
                match mission_runtime.complete_task(&content, task_id) {
                    Ok(mutation) => {
                        model.confirm_quest_end(task_id);
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
                        model.reject_pending(task_id);
                        runtime.message = format!("Tutorial QuestEnd rejected: {error}");
                    }
                }
            }
            GameplayUiAction::NpcWarp {
                npc_id,
                npc_type,
                warp_id,
                required_task_id,
                target,
            } => {
                if !model.pending_warp_matches(npc_id, npc_type, warp_id, required_task_id, target)
                {
                    runtime.message =
                        format!("Tutorial warp {warp_id} rejected: no exact pending UI request");
                    continue;
                }
                let live_actor_matches = actors.iter().any(|(_, actor)| {
                    actor.id == npc_id
                        && actor.npc_type == npc_type
                        && actor.interacting
                        && actor.is_alive()
                });
                if !live_actor_matches {
                    model.reject_warp(npc_id, npc_type, warp_id, required_task_id, target);
                    runtime.message = format!(
                        "Tutorial warp {warp_id} rejected: live NPC {npc_id}/{npc_type} is not interacting"
                    );
                    continue;
                }
                let Ok(_) = players.single() else {
                    model.reject_warp(npc_id, npc_type, warp_id, required_task_id, target);
                    runtime.message = format!(
                        "Tutorial warp {warp_id} rejected: expected exactly one LocalPlayer"
                    );
                    continue;
                };
                match validate_tutorial_warp(
                    &content,
                    &mission_runtime,
                    runtime.map_number,
                    npc_type,
                    warp_id,
                    required_task_id,
                    target,
                ) {
                    Ok(()) => {
                        if mission_runtime.pending_warp.is_none() {
                            mission_runtime.pending_warp =
                                Some(PendingTutorialNpcWarp::new(NormalNpcWarpIdentity {
                                    npc_id,
                                    npc_type,
                                    warp_id,
                                    required_task_id,
                                    target,
                                }));
                        }
                    }
                    Err(error) => {
                        model.reject_warp(npc_id, npc_type, warp_id, required_task_id, target);
                        runtime.message = format!("Tutorial warp {warp_id} rejected: {error}");
                    }
                }
            }
            GameplayUiAction::NpcIconClose { npc_id } => {
                stop_tutorial_voice(&mut commands, &tutorial_voices);
                if let Some(subtitles) = voice_subtitles.as_deref_mut() {
                    *subtitles = TutorialVoiceSubtitleState::default();
                }
                mission_runtime.auxiliary.stop();
                if let Some(audio) = gameplay_audio.as_deref_mut()
                    && let Some((entity, actor)) =
                        actors.iter().find(|(_, actor)| actor.id == npc_id)
                    && let Some(npc) = content.gameplay_npc(actor.npc_type)
                {
                    audio.queue_legacy_npc_voice(
                        entity,
                        &npc.move_voice_owner,
                        LegacyNpcVoiceCue::Farewell,
                    );
                }
                actor_commands.clear_interactions();
                tutorial
                    .progress
                    .receive_event(TutorialEvent::NpcIconClose, 1);
            }
            GameplayUiAction::NpcService { .. }
            | GameplayUiAction::SelectChatChannel(_)
            | GameplayUiAction::ToggleMenuChat
            | GameplayUiAction::ToggleEmotes
            | GameplayUiAction::SelectQuickChatItem(_)
            | GameplayUiAction::SendChat(_)
            | GameplayUiAction::WarpAwayStarted
            | GameplayUiAction::RequestWarpAway
            | GameplayUiAction::OpenNanocomMenu
            | GameplayUiAction::CloseNanocomMenu
            | GameplayUiAction::OpenUserEquipItemMode { .. }
            | GameplayUiAction::OpenOptionFromNanocomSettings
            | GameplayUiAction::OpenEmailFromNanocom
            | GameplayUiAction::OpenWorldMapFromNanocom
            | GameplayUiAction::OpenGameGuideFromNanocom
            | GameplayUiAction::OpenQuitFromNanocom
            | GameplayUiAction::OpenMissionJournal
            | GameplayUiAction::CloseMissionJournal
            | GameplayUiAction::RequestTaskStopConfirmation { .. }
            | GameplayUiAction::OpenMissionAllow { .. }
            | GameplayUiAction::OpenMissionReward { .. } => {}
        }
    }
}
