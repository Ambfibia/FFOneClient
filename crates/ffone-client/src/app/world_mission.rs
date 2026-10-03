//! World mission Nanocom, reward chat lines, network frames, mission UI sync and task delete confirmation.

use super::LocalPlayer;
use super::local_inventory::LocalInventoryRuntime;
use super::mission_indicators::{
    WORLD_MISSION_BARKER_REFRESH_COUNT, WORLD_MISSION_BARKER_REFRESH_SECONDS,
    WorldMissionBarkerRequestRuntime, clean_world_mission_barker_request,
};
use super::npc_warp::NormalNpcWarpRuntime;
use super::runtime_status::{RuntimeStatus, legacy_avatar_max_fusion_matter};
use super::social_ingress::*;
use bevy::prelude::*;
use ffone_client::{
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_audio::GameplayAudioRuntime,
    gameplay_ui::ChatLineUi,
    guide_runtime::GuideRuntime,
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    localization::{Language, Localization, LocalizedText, localized_tutorial_mission_text},
    mission_ui::{MissionUiModel, PendingMissionUiRequest},
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    nanocom_message_ui::NanocomMessageUiModel,
    network::{NetworkBridge, NetworkCommand},
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::{TutorialMissionContent, TutorialMissionType},
    world_mission_runtime::{
        WorldMissionNearbyNpc, WorldMissionRuntime, WorldMissionServerEvent0104,
        WorldMissionUiResolution, decode_world_mission_event_0104,
    },
};
use ffone_protocol::{DecodedFrame, PcTaskStopRequest0104, WirePayload, packet};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const MISSION_DELETE_SYSTEM_MESSAGE_ID_BASE: u64 = 0x4D49_5353_4445_4C00;

#[derive(Debug, Resource)]
pub(super) struct MissionDeleteConfirmationRuntime {
    pub(super) next_request_id: u64,
    pub(super) pending: BTreeMap<u64, i32>,
}

impl Default for MissionDeleteConfirmationRuntime {
    fn default() -> Self {
        Self {
            next_request_id: MISSION_DELETE_SYSTEM_MESSAGE_ID_BASE,
            pending: BTreeMap::new(),
        }
    }
}

impl MissionDeleteConfirmationRuntime {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn queue(
        &mut self,
        messages: &mut SystemMessageUiModel,
        task_id: i32,
        text: impl Into<String>,
    ) -> Result<u64, String> {
        if !self.pending.is_empty() {
            return Err("a mission-delete confirmation is already pending".to_owned());
        }
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id < MISSION_DELETE_SYSTEM_MESSAGE_ID_BASE {
            self.next_request_id = MISSION_DELETE_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending.insert(request_id, task_id);
        messages.push(SystemMessageRequest::new(
            request_id,
            text,
            SystemMessageButtonType::DeleteMission,
        ));
        Ok(request_id)
    }
}

pub(super) fn localized_mission_delete_confirmation(
    content: &TutorialMissionContent,
    localization: &Localization,
    language: &Language,
    task_id: i32,
) -> Result<String, String> {
    let mission = content
        .mission(task_id)
        .map_err(|error| format!("cannot confirm deletion of task {task_id}: {error}"))?;
    let kind = localization.text(
        language,
        &match mission.mission_type {
            TutorialMissionType::Guide => LocalizedText::new("ui.mission.type.guide", "Guide"),
            TutorialMissionType::Nano => LocalizedText::new("ui.mission.type.nano", "Nano"),
            TutorialMissionType::World => LocalizedText::new("ui.mission.type.quest", "Quest"),
        },
    );
    let title = localization.text(
        language,
        &localized_tutorial_mission_text(task_id, "title", &mission.title),
    );
    let mission_name = localization.text(
        language,
        &LocalizedText::new("ui.mission.journal.mission_name", "{kind} : {title}")
            .with_arg("kind", kind)
            .with_arg("title", title),
    );
    Ok(localization.text(
        language,
        &LocalizedText::new(
            "ui.mission.delete.confirm",
            "Delete\n {mission}\nThis mission will no longer appear in your mission journal,\nbut you can get it again later by visiting the mission giver.",
        )
        .with_arg("mission", mission_name),
    ))
}

#[derive(Clone, Copy)]
pub(super) enum MissionNanocomEdge {
    Start,
    Success,
    Failure,
}

pub(super) fn enqueue_mission_nanocom(
    task_id: i32,
    edge: MissionNanocomEdge,
    content: &TutorialMissionContent,
    messages: &mut NanocomMessageUiModel,
) -> bool {
    let Ok(mission) = content.mission(task_id) else {
        return false;
    };
    let message = match edge {
        MissionNanocomEdge::Start => mission.start_nanocom_message.as_ref(),
        MissionNanocomEdge::Success => mission.success_nanocom_message.as_ref(),
        MissionNanocomEdge::Failure => mission.failure_nanocom_message.as_ref(),
    };
    let Some(message) = message else {
        return false;
    };
    let Some(npc) = content.gameplay_npc(message.npc_type) else {
        return false;
    };
    let localized_body = LocalizedText::new(
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            message.string_id
        ),
        message.text.clone(),
    );
    if matches!(edge, MissionNanocomEdge::Start)
        && mission.mission_type == TutorialMissionType::Nano
        && content
            .is_first_serialized_task(task_id)
            .is_ok_and(|is_first| is_first)
    {
        let icon_path = i16::try_from(mission.provenance.nano_id)
            .ok()
            .and_then(|nano_id| content.gameplay_nano(nano_id))
            .and_then(|nano| nano.icon_path.clone());
        messages.enqueue_nano_mission_localized(
            localized_body,
            icon_path,
            Some(npc.move_voice_owner.as_str()),
        );
        return true;
    }
    let icon_path = content
        .gameplay_npc_portrait_icon_path(message.npc_type)
        .map(str::to_owned);
    messages.enqueue_type_9_localized_optional_icon(
        LocalizedText::new(
            format!("content.npc.{}.name", message.npc_type),
            npc.name.clone(),
        ),
        localized_body,
        icon_path,
        Some(npc.move_voice_owner.as_str()),
    );
    true
}

pub(super) fn enqueue_world_mission_nanocom(
    event: &WorldMissionServerEvent0104,
    content: &TutorialMissionContent,
    messages: &mut NanocomMessageUiModel,
) -> bool {
    let (task_id, edge) = match event {
        WorldMissionServerEvent0104::TaskStartSuccess(reply) => {
            (reply.task_id, MissionNanocomEdge::Start)
        }
        WorldMissionServerEvent0104::TaskEndSuccess(reply) => {
            (reply.task_id, MissionNanocomEdge::Success)
        }
        WorldMissionServerEvent0104::TaskEndFailure(reply)
            if matches!(reply.error_code, 1 | 11 | 12) =>
        {
            (reply.task_id, MissionNanocomEdge::Failure)
        }
        _ => return false,
    };
    enqueue_mission_nanocom(task_id, edge, content, messages)
}

/// Clean `cnDisplayMapName.RewardFM`: outside the tutorial, every currency the
/// reward raised adds an `AddEventString(5, ...)` line, Taros first.
pub(super) fn reward_currency_chat_lines(
    previous_taros: i32,
    previous_fusion_matter: i32,
    reply: &ffone_protocol::RewardItemReply0104,
) -> Vec<ChatLineUi> {
    let mut lines = Vec::new();
    for (previous, total, key, fallback) in [
        (
            previous_taros,
            reply.candy,
            "ui.hud.chat.reward.taros",
            "You earned {amount} Taros for a total {total} taros.",
        ),
        (
            previous_fusion_matter,
            reply.fusion_matter,
            "ui.hud.chat.reward.fusion_matter",
            "You collected {amount} Fusion Matter for a total {total} Fusion Matter.",
        ),
    ] {
        if total <= previous {
            continue;
        }
        let amount = total.saturating_sub(previous).to_string();
        let total = total.to_string();
        let text = fallback
            .replace("{amount}", &amount)
            .replace("{total}", &total);
        let localized = LocalizedText::new(key, fallback)
            .with_arg("amount", amount)
            .with_arg("total", total);
        lines.push(ChatLineUi::receive(text).with_localized(localized));
    }
    lines
}

/// Clean `cnDisplayMapName.RewardItem`: outside the tutorial, every bag
/// C.R.A.T.E (`m_iChestCheck == 0`) or E.G.G (`1`) adds its own
/// `AddEventString(5, ...)` line. Quest items never reach this branch.
pub(super) fn reward_chest_chat_lines(
    content: &TutorialMissionContent,
    reply: &ffone_protocol::RewardItemReply0104,
) -> Vec<ChatLineUi> {
    reply
        .items
        .iter()
        .filter(|reward| reward.inventory_location == 1 && reward.item.item_type == 9)
        .filter_map(|reward| {
            let (key, fallback) = match content.reward_chest_kind(reward.item.item_id)? {
                0 => ("ui.hud.chat.reward.crate", "You receive 1 C.R.A.T.E"),
                1 => ("ui.hud.chat.reward.egg", "You receive 1 E.G.G"),
                _ => return None,
            };
            Some(ChatLineUi::receive(fallback).with_localized(LocalizedText::new(key, fallback)))
        })
        .collect()
}

pub(super) fn apply_world_mission_network_frame(
    frame: &DecodedFrame,
    bridge: &NetworkBridge,
    content: &TutorialMissionContent,
    npcs: &Query<&NetworkNpcAppearance0104>,
    inventory: &mut LocalInventoryRuntime,
    reward_notices: Option<&mut ffone_client::gameplay_ui::rewards::RewardNotices>,
    event_messages_in_chat: bool,
    mission: &mut WorldMissionRuntime,
    mission_ui: &mut MissionUiModel,
    nanocom_messages: &mut NanocomMessageUiModel,
    gameplay_audio: &mut GameplayAudioRuntime,
    runtime: &mut RuntimeStatus,
) -> Result<bool, String> {
    let Some(event) = decode_world_mission_event_0104(frame.packet_type, &frame.payload)? else {
        return Ok(false);
    };

    // OpenFusion emits REWARD_ITEM before a proactive/explicit END_SUCC. Apply
    // its complete post-state first so the next UI projection observes the
    // authoritative quest-item count and currencies.
    if let WorldMissionServerEvent0104::RewardItem(reply) = &event {
        inventory.apply_reward_item_post_state(reply)?;
        if let Some(notices) = reward_notices {
            // Reward notices exist only outside the tutorial, matching the
            // source's `bTutorial` gate; `bEventMessage1` gates the chat lines.
            if event_messages_in_chat {
                // `RewardItem` runs inside the packet's item loop, before the
                // currency event that produces the Taros/FM summary lines.
                let lines = reward_chest_chat_lines(content, reply).into_iter().chain(
                    reward_currency_chat_lines(runtime.candy, runtime.fusion_matter, reply),
                );
                for line in lines {
                    push_world_chat_line(runtime, line);
                }
            }
            notices.receive_currencies(
                runtime.candy,
                runtime.fusion_matter,
                reply.candy,
                reply.fusion_matter,
            );
            notices.receive(
                reply,
                content,
                mission,
                runtime.nano_battery,
                runtime.weapon_battery,
            );
        }
        runtime.candy = reply.candy;
        runtime.fusion_matter = reply.fusion_matter;
        runtime.max_fusion_matter =
            legacy_avatar_max_fusion_matter(runtime.player_level, reply.fusion_matter);
        runtime.nano_battery = reply.nano_battery;
        runtime.weapon_battery = reply.weapon_battery;
    }

    let retiring_task = match &event {
        WorldMissionServerEvent0104::TaskEndSuccess(reply) => Some(reply.task_id),
        WorldMissionServerEvent0104::TaskStopSuccess(reply) => Some(reply.task_id),
        WorldMissionServerEvent0104::TaskEndFailure(reply)
            if matches!(reply.error_code, 1 | 11 | 12) =>
        {
            Some(reply.task_id)
        }
        _ => None,
    }
    .filter(|task_id| {
        mission
            .active_tasks()
            .iter()
            .any(|task| task.task_id == *task_id)
    });
    let outcome = mission.apply_event(&event, content)?;
    if let Some(completion_kind) = outcome.completion_kind {
        gameplay_audio.queue_gameplay_ui_sound(completion_kind.legacy_audio_true_name());
    }
    enqueue_world_mission_nanocom(&event, content, nanocom_messages);
    match outcome.ui_resolution {
        Some(WorldMissionUiResolution::TaskStartAccepted(task_id)) => {
            mission_ui.confirm_task_start(task_id);
        }
        Some(WorldMissionUiResolution::QuestEndAccepted(task_id)) => {
            mission_ui.confirm_quest_end(task_id);
        }
        Some(WorldMissionUiResolution::TaskStopAccepted(task_id)) => {
            let remaining_active_task_ids = mission.active_task_ids();
            mission_ui.confirm_task_stop(task_id, &remaining_active_task_ids);
        }
        Some(WorldMissionUiResolution::RequestRejected(task_id)) => {
            mission_ui.reject_pending(task_id);
        }
        None => {}
    }
    // Release the old escort before starting the outgoing task. A later
    // polling system must not kick an NPC already assigned to the next task.
    if let Some(task_id) = retiring_task
        && !mission
            .active_tasks()
            .iter()
            .any(|task| task.task_id == task_id)
    {
        let npc_id = ffone_client::world_mission_runtime::mission_escort_npc_id(
            task_id,
            content,
            npcs.iter().map(|npc| (npc.0.npc_type, npc.0.npc_id)),
        );
        if npc_id != 0 {
            bridge
                .send(NetworkCommand::KickEscortNpc(npc_id))
                .map_err(|error| format!("escort release send failed: {error}"))?;
        }
    }
    if let Some(mut request) = outcome.follow_up_start {
        request.escort_npc_id = ffone_client::world_mission_runtime::mission_escort_npc_id(
            request.task_id,
            content,
            npcs.iter().map(|npc| (npc.0.npc_type, npc.0.npc_id)),
        );
        let pending = PendingMissionUiRequest::TaskStart {
            task_id: request.task_id,
            npc_id: 0,
        };
        let registered = ffone_protocol::RegisteredGameplayRequest0104::new(
            packet::P_CL2FE_REQ_PC_TASK_START,
            request.encode(),
        )
        .map_err(|error| format!("outgoing mission task-start codec rejected: {error}"))?;
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered)) {
            mission.cancel_request(pending);
            return Err(format!(
                "outgoing mission task {} could not be started: {error}",
                request.task_id
            ));
        }
    }
    Ok(true)
}

pub(super) fn sync_world_mission_ui(
    time: Res<Time>,
    bridge: Res<NetworkBridge>,
    content: Res<TutorialMissionContent>,
    inventory: Res<LocalInventoryRuntime>,
    guide: Res<GuideRuntime>,
    nano_bank: Res<NanoFreeTuningBank0104>,
    npc_appearances: Query<(&NetworkNpcAppearance0104, &GlobalTransform)>,
    local_players: Query<&GlobalTransform, With<LocalPlayer>>,
    mut mission: ResMut<WorldMissionRuntime>,
    mut barker_requests: ResMut<WorldMissionBarkerRequestRuntime>,
    mut random: ResMut<LegacyNanoStandRandomStream>,
    mut model: ResMut<MissionUiModel>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    mission.tick(time.delta_secs());
    let Some(quest_inventory) = inventory.quest_inventory.as_ref() else {
        return;
    };
    let owned_nanos = nano_bank
        .entries()
        .iter()
        .filter_map(|nano| (nano.id > 0).then_some(i32::from(nano.id)))
        .collect::<BTreeSet<_>>();
    let guide = guide
        .authoritative()
        .map_or(0, |state| i32::from(state.raw_mentor()));

    let nearby_npcs = npc_appearances
        .iter()
        .filter_map(|(appearance, transform)| {
            let definition = content.gameplay_npc(appearance.0.npc_type)?;
            let position = transform.translation();
            Some(WorldMissionNearbyNpc {
                npc_type: appearance.0.npc_type,
                npc_id: appearance.0.npc_id,
                position: [position.x, position.y, position.z],
                sight_range_server_units: definition.sight_range_server_units,
            })
        })
        .collect::<Vec<_>>();
    let player_position = local_players.single().ok().map(|transform| {
        let position = transform.translation();
        [position.x, position.y, position.z]
    });
    let now = time.elapsed_secs_f64();
    if now - barker_requests.last_refresh_time >= WORLD_MISSION_BARKER_REFRESH_SECONDS {
        barker_requests.last_refresh_time = now;
        barker_requests.loop_count = barker_requests.loop_count.saturating_add(1);
        if barker_requests.loop_count >= WORLD_MISSION_BARKER_REFRESH_COUNT {
            barker_requests.loop_count = 0;
            if let Some(request) = player_position.and_then(|position| {
                clean_world_mission_barker_request(
                    &mission,
                    &content,
                    position,
                    &nearby_npcs,
                    &mut random,
                )
            }) {
                match ffone_protocol::RegisteredGameplayRequest0104::new(
                    packet::P_CL2FE_REQ_BARKER,
                    request.encode(),
                ) {
                    Ok(registered) => {
                        if let Err(error) =
                            bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered))
                        {
                            runtime.message = format!(
                                "Completed-mission Barker request transport failed: {error}"
                            );
                        }
                    }
                    Err(error) => {
                        runtime.message =
                            format!("Completed-mission Barker request codec rejected: {error}");
                    }
                }
            }
        }
    }
    match mission.collect_automatic_end_requests(
        quest_inventory,
        player_position,
        &nearby_npcs,
        &content,
    ) {
        Ok(actions) => {
            for request in actions.end_requests {
                let pending = PendingMissionUiRequest::QuestEnd {
                    task_id: request.task_id,
                    npc_id: request.npc_id,
                    box1_choice: 0,
                    box2_choice: 0,
                };
                let registered = match ffone_protocol::RegisteredGameplayRequest0104::new(
                    packet::P_CL2FE_REQ_PC_TASK_END,
                    request.encode(),
                ) {
                    Ok(request) => request,
                    Err(error) => {
                        mission.cancel_request(pending);
                        runtime.message = format!("Automatic mission-end codec rejected: {error}");
                        continue;
                    }
                };
                if let Err(error) =
                    bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered))
                {
                    mission.cancel_request(pending);
                    runtime.message = format!(
                        "Automatic mission-end send failed for task {}: {error}",
                        request.task_id
                    );
                }
            }
            if let Some(reward) = actions.reward {
                match content.automatic_reward_entry(reward.task_id, reward.npc_id) {
                    Ok(entry) => {
                        if model.open_automatic_reward(entry) {
                            mission.acknowledge_automatic_reward(reward);
                        }
                    }
                    Err(error) => {
                        runtime.message = format!(
                            "Automatic mission reward ignored for task {}: {error}",
                            reward.task_id
                        );
                    }
                }
            }
        }
        Err(error) => runtime.message = format!("Automatic mission completion ignored {error}"),
    }

    match mission.journal_entries(&content) {
        Ok((active, completed)) => {
            model.enabled = true;
            if model.nanocom_journal.title.is_empty() {
                model.nanocom_journal.title = "MISSION JOURNAL".to_owned();
            }
            model.set_active_journal_missions(active);
            model.set_completed_journal_missions(completed);
        }
        Err(error) => runtime.message = format!("Mission journal ignored {error}"),
    }

    if let Some(npc_id) = model
        .npc_interaction
        .as_ref()
        .map(|interaction| interaction.npc_id)
        && let Some((appearance, _)) = npc_appearances
            .iter()
            .find(|(appearance, _)| appearance.0.npc_id == npc_id)
    {
        match mission.npc_entries(
            appearance.0.npc_type,
            npc_id,
            runtime.map_name.clone(),
            i32::from(runtime.player_level),
            guide,
            &owned_nanos,
            quest_inventory,
            &content,
        ) {
            Ok((available, completed)) => {
                if let Some(interaction) = &mut model.npc_interaction {
                    interaction.available_missions = available;
                    interaction.completed_missions = completed;
                }
            }
            Err(error) => runtime.message = format!("NpcIconMode mission refresh ignored {error}"),
        }
    }

    let runtime_selected = mission.selected_task_id(&content);
    let desired = model
        .selected_journal_task_id
        .filter(|task_id| {
            mission
                .active_tasks()
                .iter()
                .any(|active| active.task_id == *task_id)
        })
        .or(runtime_selected);
    model.selected_journal_task_id = desired;
    let Some(task_id) = desired else {
        return;
    };
    match mission.begin_select_task(task_id, &content) {
        Ok(Some(request)) => {
            let mission_id = request.mission_id;
            match ffone_protocol::RegisteredGameplayRequest0104::new(
                packet::P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID,
                request.encode(),
            ) {
                Ok(request) => {
                    if let Err(error) =
                        bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
                    {
                        mission.cancel_current_request(mission_id);
                        runtime.message = format!("Set-current-mission send failed: {error}");
                    }
                }
                Err(error) => {
                    mission.cancel_current_request(mission_id);
                    runtime.message = format!("Set-current-mission codec rejected: {error}");
                }
            }
        }
        Ok(None) => {}
        Err(error) => runtime.message = format!("Set-current-mission ignored {error}"),
    }
}

pub(super) fn begin_confirmed_world_task_stop(
    mission_ui: &mut MissionUiModel,
    mission: &WorldMissionRuntime,
    task_id: i32,
) -> Option<PendingMissionUiRequest> {
    if !mission
        .active_tasks()
        .iter()
        .any(|active| active.task_id == task_id)
        || !mission_ui.begin_task_stop(task_id)
    {
        return None;
    }
    Some(PendingMissionUiRequest::TaskStop { task_id })
}

pub(super) fn consume_mission_delete_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut confirmation: ResMut<MissionDeleteConfirmationRuntime>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut mission: ResMut<WorldMissionRuntime>,
    bridge: Res<NetworkBridge>,
    mut status: ResMut<RuntimeStatus>,
) {
    if confirmation.pending.is_empty() {
        return;
    }
    let owned_request_ids = confirmation
        .pending
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    for action in system_outbox.drain_matching(|candidate| {
        matches!(
            candidate,
            SystemMessageUiAction::Chosen { request_id, .. }
                if owned_request_ids.contains(request_id)
        )
    }) {
        let SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        } = action;
        let Some(task_id) = confirmation.pending.remove(&request_id) else {
            continue;
        };
        if button_type != SystemMessageButtonType::DeleteMission
            || choice != SystemMessageChoice::Primary
        {
            continue;
        }
        let Some(request) = begin_confirmed_world_task_stop(&mut mission_ui, &mission, task_id)
        else {
            continue;
        };
        if let Err(error) = mission.record_request(request) {
            mission_ui.reject_pending(task_id);
            status.message = format!("Mission stop request rejected: {error}");
            continue;
        }
        if let Err(error) = bridge.send(NetworkCommand::StopTask(PcTaskStopRequest0104 { task_id }))
        {
            mission.cancel_request(request);
            mission_ui.reject_pending(task_id);
            status.message = format!("Mission stop send failed: {error}");
        }
    }
}

pub(super) fn reset_world_mission_session(
    mut runtime: ResMut<WorldMissionRuntime>,
    mut model: ResMut<MissionUiModel>,
    mut normal_npc_warp: ResMut<NormalNpcWarpRuntime>,
    mut mission_delete_confirmation: ResMut<MissionDeleteConfirmationRuntime>,
) {
    runtime.clear();
    *model = MissionUiModel::default();
    normal_npc_warp.reset();
    mission_delete_confirmation.reset();
}
