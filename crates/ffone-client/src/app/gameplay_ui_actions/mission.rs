use crate::app::gameplay_ui_actions::*;

pub(super) fn handle_mission(
    action: MissionAction,
    gameplay_modal: bool,
    context: &mut WorldGameplayActionContext<'_, '_>,
) {
    let WorldGameplayActionContext {
        mission_ui,
        content,
        modal_owners,
        bridge,
        runtime,
        npc_appearances,
        normal_warp,
        ..
    } = context;
    // A dialog can outlive movement or an NPC despawn. Revalidate before
    // opening a quest page and before recording/sending a transaction.
    let target = match &action {
        MissionAction::TaskStart { task_id, npc_id }
        | MissionAction::QuestEnd {
            task_id, npc_id, ..
        }
        | MissionAction::OpenMissionAllow { task_id, npc_id }
        | MissionAction::OpenMissionReward { task_id, npc_id }
            if *npc_id > 0 =>
        {
            Some((*task_id, *npc_id))
        }
        _ => None,
    };
    if let Some((task_id, npc_id)) = target {
        let mut matches = normal_warp
            .npc_sources
            .iter()
            .filter(|(_, npc, _)| npc.0.npc_id == npc_id);
        let target = matches.next();
        let valid = target.is_some_and(|(_, npc, transform)| {
            npc.0.hp > 0
                && matches.next().is_none()
                && modal_owners
                    .shell
                    .players
                    .single()
                    .is_ok_and(|(player, _, pending)| {
                        pending.is_none()
                            && ffone_client::world_targeting::world_npc_in_interaction_range(
                                player.translation,
                                transform.translation(),
                            )
                    })
        });
        if !valid {
            mission_ui.reject_pending(task_id);
            mission_ui.clear_npc_interaction_locally();
            runtime.message = format!("Mission action ignored stale or out-of-range NPC {npc_id}");
            return;
        }
    }
    for action in std::iter::once(action) {
        match action {
            MissionAction::RequestTaskStopConfirmation { task_id } => {
                let active = modal_owners
                    .mission
                    .world_mission
                    .active_tasks()
                    .iter()
                    .any(|task| task.task_id == task_id);
                if gameplay_modal
                    || modal_owners.shell.system_messages.is_popup()
                    || !modal_owners.mission.delete_confirmation.pending.is_empty()
                    || !active
                    || !mission_ui.can_request_task_stop_confirmation(task_id)
                {
                    continue;
                }
                let text = match localized_mission_delete_confirmation(
                    &content,
                    &modal_owners.shell.localization,
                    &modal_owners.shell.language,
                    task_id,
                ) {
                    Ok(text) => text,
                    Err(error) => {
                        runtime.message = error;
                        continue;
                    }
                };
                if let Err(error) = modal_owners.mission.delete_confirmation.queue(
                    &mut modal_owners.shell.system_messages,
                    task_id,
                    text,
                ) {
                    runtime.message = format!("Mission delete confirmation rejected: {error}");
                }
            }
            MissionAction::TaskStart { task_id, npc_id } => {
                if let Err(rejection) = modal_owners
                    .mission
                    .world_mission
                    .check_accept_task(task_id, &content)
                {
                    mission_ui.reject_pending(task_id);
                    let message = match rejection {
                        WorldMissionAcceptRejection::UnknownTask { task_id } => LocalizedText::new(
                            "ui.mission.accept.unknown_task",
                            "Mission task {task} is missing from TableData.",
                        )
                        .with_arg("task", task_id.to_string()),
                        WorldMissionAcceptRejection::CategoryFull {
                            mission_type,
                            capacity,
                        } => {
                            let category = modal_owners.shell.localization.text(
                                &modal_owners.shell.language,
                                &match mission_type {
                                    TutorialMissionType::Guide => LocalizedText::new(
                                        "ui.mission.type.guide",
                                        "Guide",
                                    ),
                                    TutorialMissionType::Nano => {
                                        LocalizedText::new("ui.mission.type.nano", "Nano")
                                    }
                                    TutorialMissionType::World => {
                                        LocalizedText::new("ui.mission.type.quest", "Quest")
                                    }
                                },
                            );
                            LocalizedText::new(
                                "ui.mission.accept.category_full",
                                "You cannot accept another {category} mission (limit {capacity}).",
                            )
                            .with_arg("category", category)
                            .with_arg("capacity", capacity.to_string())
                        }
                        WorldMissionAcceptRejection::UnsupportedLocation {
                            task_id,
                            task_type,
                            required_instance_id,
                        } => LocalizedText::new(
                            "ui.mission.accept.location_unsupported",
                            "Mission task {task} needs an unavailable location/instance trigger (type {task_type}, instance {instance}).",
                        )
                        .with_arg("task", task_id.to_string())
                        .with_arg("task_type", task_type.to_string())
                        .with_arg("instance", required_instance_id.to_string()),
                        WorldMissionAcceptRejection::UnsupportedEscort {
                            task_id,
                            escort_npc_type,
                        } => LocalizedText::new(
                            "ui.mission.accept.escort_unsupported",
                            "Mission task {task} needs an unavailable escort owner (NPC type {escort}).",
                        )
                        .with_arg("task", task_id.to_string())
                        .with_arg("escort", escort_npc_type.to_string()),
                    };
                    runtime.message = modal_owners
                        .shell
                        .localization
                        .text(&modal_owners.shell.language, &message);
                    continue;
                }
                let request = PendingMissionUiRequest::TaskStart { task_id, npc_id };
                let escort_npc_id = ffone_client::world_mission_runtime::mission_escort_npc_id(
                    task_id,
                    content,
                    npc_appearances
                        .iter()
                        .map(|(_, npc)| (npc.0.npc_type, npc.0.npc_id)),
                );
                let registered = match request.encode_registered_with_escort(escort_npc_id) {
                    Ok(request) => request,
                    Err(error) => {
                        mission_ui.reject_pending(task_id);
                        runtime.message = format!("Mission start request rejected: {error}");
                        continue;
                    }
                };
                if let Err(error) = modal_owners.mission.world_mission.record_request(request) {
                    mission_ui.reject_pending(task_id);
                    runtime.message = format!("Mission start request rejected: {error}");
                    continue;
                }
                if let Err(error) =
                    bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered))
                {
                    modal_owners.mission.world_mission.cancel_request(request);
                    mission_ui.reject_pending(task_id);
                    runtime.message = format!("Mission start send failed: {error}");
                }
            }
            MissionAction::QuestEnd {
                task_id,
                npc_id,
                box1_choice,
                box2_choice,
            } => {
                // An authoritative proactive END_SUCC may have retired this
                // task between NpcIconMode projection and button input.
                if !modal_owners
                    .mission
                    .world_mission
                    .active_tasks()
                    .iter()
                    .any(|active| active.task_id == task_id)
                {
                    mission_ui.reject_pending(task_id);
                    continue;
                }
                let request = PendingMissionUiRequest::QuestEnd {
                    task_id,
                    npc_id,
                    box1_choice,
                    box2_choice,
                };
                let escort_npc_id = ffone_client::world_mission_runtime::mission_escort_npc_id(
                    task_id,
                    content,
                    npc_appearances
                        .iter()
                        .map(|(_, npc)| (npc.0.npc_type, npc.0.npc_id)),
                );
                let registered = match request.encode_registered_with_escort(escort_npc_id) {
                    Ok(request) => request,
                    Err(error) => {
                        mission_ui.reject_pending(task_id);
                        runtime.message = format!("Mission completion request rejected: {error}");
                        continue;
                    }
                };
                if let Err(error) = modal_owners.mission.world_mission.record_request(request) {
                    mission_ui.reject_pending(task_id);
                    runtime.message = format!("Mission completion request rejected: {error}");
                    continue;
                }
                if let Err(error) =
                    bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered))
                {
                    modal_owners.mission.world_mission.cancel_request(request);
                    mission_ui.reject_pending(task_id);
                    runtime.message = format!("Mission completion send failed: {error}");
                }
            }
            // Returning from the journal to its NPC is not NpcIconMode.EndMode.
            MissionAction::OpenMissionJournal | MissionAction::CloseMissionJournal => {}
            MissionAction::OpenMissionAllow { task_id, npc_id }
            | MissionAction::OpenMissionReward { task_id, npc_id } => {
                let _ = (task_id, npc_id);
            }
            MissionAction::ConfirmTutorialExit {
                message_id,
                button_type,
            } => {
                let _ = (message_id, button_type);
            }
        }
    }
}
