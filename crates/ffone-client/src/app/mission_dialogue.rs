use super::*;
use ffone_client::world_mission_runtime::WorldMissionDialogueEdge;

#[cfg(test)]
mod tests;

/// Present accepted tutorial task edges. World edges arrive as packets
/// (`enqueue_world_mission_nanocom`); the local tutorial records them in
/// `start_task`/`complete_task`, including an outgoing task started by a
/// completion, so its STMessage/SUMessage NanoCom rows (Numbuh Two after the
/// Oil Ogre, both of Dexter's calls) appear with the lines the script voices.
/// The table owns each bubble's speaker; see [`tutorial_mission_bubble`].
pub(super) fn present_tutorial_mission_dialogue(
    mut mission: ResMut<TutorialMissionRuntime>,
    content: Res<TutorialMissionContent>,
    tutorial: Res<TutorialSession>,
    npcs: Query<(Entity, &TutorialActor, &GlobalTransform)>,
    players: Query<&GlobalTransform, With<LocalPlayer>>,
    mut messages: ResMut<NanocomMessageUiModel>,
    mut bubbles: ResMut<NpcBarkerBubbleRuntime>,
) {
    if mission.dialogue_edges.is_empty() {
        return;
    }
    let player = players.single().ok();
    for (task_id, edge) in std::mem::take(&mut mission.dialogue_edges) {
        let nanocom_edge = match edge {
            WorldMissionDialogueEdge::Start => MissionNanocomEdge::Start,
            WorldMissionDialogueEdge::Success | WorldMissionDialogueEdge::Complete => {
                MissionNanocomEdge::Success
            }
            WorldMissionDialogueEdge::Failure => MissionNanocomEdge::Failure,
        };
        enqueue_mission_nanocom(task_id, nanocom_edge, &content, &mut messages);
        let (Some(player), Ok(task)) = (player, content.mission(task_id)) else {
            continue;
        };
        let Some(dialogue) = tutorial_mission_bubble(task, edge, tutorial.scene) else {
            continue;
        };
        let Some(definition) = content.gameplay_npc(dialogue.npc_type) else {
            continue;
        };
        let range = 5000.0
            + (definition.radius_server_units as f32 * 0.01)
                .max(definition.height_server_units as f32 * 0.005);
        let owner = npcs
            .iter()
            .filter(|(_, actor, transform)| {
                actor.is_alive()
                    && actor.npc_type == dialogue.npc_type
                    && transform
                        .translation()
                        .distance_squared(player.translation())
                        < range * range
            })
            .min_by_key(|(_, actor, _)| actor.id)
            .map(|(entity, _, _)| entity);
        if let Some(owner) = owner {
            bubbles.request_quest_dialogue(
                owner,
                dialogue.npc_type,
                LocalizedText::new(
                    format!(
                        "content.tabledata.mission.mission_string.{}.str_name_string",
                        dialogue.string_id
                    ),
                    &dialogue.text,
                ),
            );
        }
    }
}

/// Table bubble for an accepted tutorial edge. In an interactive stage the
/// bubble is the only text of the voiced line (Buttercup's 2250 line during
/// `ButtercupDelay`). An edge that opens a cutscene is already subtitled
/// there: Dexter's 2253 starts InfectionB in the same frame, so its bubble
/// would repeat the subtitle over his head. NPC bubbles are not hidden by
/// cutscenes, hence the gate here.
fn tutorial_mission_bubble(
    task: &ffone_client::tutorial_mission_content::TutorialMissionDefinition,
    edge: WorldMissionDialogueEdge,
    scene: TutorialScene,
) -> Option<&ffone_client::tutorial_mission_content::TutorialMissionDialogue> {
    if scene != TutorialScene::None {
        return None;
    }
    match edge {
        WorldMissionDialogueEdge::Start => task.start_dialogue.as_ref(),
        WorldMissionDialogueEdge::Success | WorldMissionDialogueEdge::Complete => {
            task.success_dialogue.as_ref()
        }
        WorldMissionDialogueEdge::Failure => task.failure_dialogue.as_ref(),
    }
}

pub(super) fn present_world_mission_dialogue(
    mut mission: ResMut<WorldMissionRuntime>,
    content: Res<TutorialMissionContent>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform)>,
    players: Query<&GlobalTransform, With<LocalPlayer>>,
    mut bubbles: ResMut<NpcBarkerBubbleRuntime>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut messages: ResMut<SystemMessageUiModel>,
) {
    for task_id in mission.take_inventory_full_notices() {
        if let Some(definition) = content.system_message_definition(12) {
            let request_id = 0x4D49_5353_0000_0000 | task_id as u32 as u64;
            if !messages
                .stack()
                .iter()
                .any(|message| message.request_id == request_id)
            {
                messages.push(SystemMessageRequest::new_localized(
                    request_id,
                    LocalizedText::new(
                        "content.tabledata.message.message.12.sz_string",
                        &definition.exact_text,
                    ),
                    definition.runtime_button_type,
                ));
            }
        }
    }
    let edges = mission.take_dialogue_edges();
    let Ok(player) = players.single() else {
        return;
    };
    let nearby = |npc_type| {
        npcs.iter()
            .filter(|(_, appearance, transform)| {
                if appearance.0.npc_type != npc_type {
                    return false;
                }
                let Some(npc) = content.gameplay_npc(npc_type) else {
                    return false;
                };
                // NpcContainer receives 5000 in client coordinates, then
                // adds the converted actor radius/half-height. Do not scale
                // the near-list argument as if it were a network position.
                let range = 5000.0
                    + (npc.radius_server_units as f32 * 0.01)
                        .max(npc.height_server_units as f32 * 0.005);
                transform
                    .translation()
                    .distance_squared(player.translation())
                    < range * range
            })
            .min_by_key(|(_, appearance, _)| appearance.0.npc_id)
            .map(|(entity, _, _)| entity)
    };
    for (task_id, edge) in edges {
        let Ok(task) = content.mission(task_id) else {
            continue;
        };
        let dialogue = match edge {
            WorldMissionDialogueEdge::Start => task.start_dialogue.as_ref(),
            WorldMissionDialogueEdge::Success | WorldMissionDialogueEdge::Complete => {
                task.success_dialogue.as_ref()
            }
            WorldMissionDialogueEdge::Failure => task.failure_dialogue.as_ref(),
        };
        if let Some(dialogue) = dialogue {
            // SetBubbleChat returns before generic voice if its explicit
            // dialogue speaker is absent from the source near list.
            let Some(owner) = nearby(dialogue.npc_type) else {
                continue;
            };
            bubbles.request_quest_dialogue(
                owner,
                dialogue.npc_type,
                LocalizedText::new(
                    format!(
                        "content.tabledata.mission.mission_string.{}.str_name_string",
                        dialogue.string_id
                    ),
                    dialogue.text.clone(),
                ),
            );
        }
        let Some((npc_type, cue)) = mission_voice(task, edge) else {
            continue;
        };
        if let Some(entity) = nearby(npc_type)
            && let Some(npc) = content.gameplay_npc(npc_type)
        {
            audio.queue_legacy_npc_voice(entity, &npc.move_voice_owner, cue);
        }
    }
}

fn mission_voice(
    task: &ffone_client::tutorial_mission_content::TutorialMissionDefinition,
    edge: WorldMissionDialogueEdge,
) -> Option<(i32, LegacyNpcVoiceCue)> {
    Some(match edge {
        WorldMissionDialogueEdge::Start => (
            task.provenance.start_npc_type,
            if task.provenance.task_type == 6 {
                LegacyNpcVoiceCue::QuestStartEscort
            } else {
                LegacyNpcVoiceCue::QuestAccepted
            },
        ),
        WorldMissionDialogueEdge::Complete => (
            task.provenance.terminator_npc_type,
            LegacyNpcVoiceCue::QuestCompleted,
        ),
        WorldMissionDialogueEdge::Success | WorldMissionDialogueEdge::Failure => return None,
    })
}
