//! Tutorial mission interaction, world indicators, stun and Nano condition effects.

use super::mission_indicators::{
    TutorialMissionIndicatorRuntime, TutorialNanoConditionEffectRuntime,
    tutorial_actor_smart_indicator_scale, tutorial_npc_mission_symbol, tutorial_quest_symbol_name,
    tutorial_selected_waypoint_npc_types, tutorial_smart_indicator_name,
};
use super::runtime_status::RuntimeStatus;
use super::tutorial_mission_flow::tutorial_warp_is_available;
use super::tutorial_session::{
    PendingTutorialActorEffects, TutorialLogicRuntime, TutorialMissionRuntime, TutorialSession,
};
use bevy::prelude::*;
use ffone_client::{
    character_scene::LegacyCharacterSceneStatus,
    mission_ui::{JournalOtherUi, MissionJournalUi, MissionUiModel, NpcInteractionUi, WarpUiEntry},
    network_world_runtime::NetworkNpcVisual0104,
    tutorial::{InfectionStage, TutorialStage},
    tutorial_actors::{TutorialActor, tutorial_actor_root_name},
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
    },
    tutorial_logic::{BUTTERCUP_ID, LAIR_DEXTER_ID},
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::{
        TUTORIAL_BUTTERCUP_SKILL_CONDITION, TUTORIAL_BUTTERCUP_STUN_EFFECT_ID,
        TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE, TutorialNanoSkillCondition,
    },
};
use std::collections::BTreeMap;

pub(super) fn sync_tutorial_mission_interaction(
    actors: Query<&TutorialActor>,
    content: Res<TutorialMissionContent>,
    mut runtime: ResMut<RuntimeStatus>,
    mission_runtime: Res<TutorialMissionRuntime>,
    tutorial: Res<TutorialSession>,
    mut model: ResMut<MissionUiModel>,
    mut logic: ResMut<TutorialLogicRuntime>,
) {
    if tutorial.completion_requested {
        *model = MissionUiModel::default();
        logic.ui = model.tutorial_observation();
        return;
    }
    model.enabled = true;
    let journal_entries = content
        .active_journal_entries(&mission_runtime.active_tasks, runtime.map_name.clone())
        .and_then(|active_missions| {
            content
                .completed_journal_entries(
                    &mission_runtime.completed_tasks,
                    runtime.map_name.clone(),
                )
                .map(|completed_missions| (active_missions, completed_missions))
        });
    match journal_entries {
        Ok((active_missions, completed_missions)) => {
            let journal = JournalOtherUi {
                title: "MISSION JOURNAL".to_owned(),
                active_missions,
                completed_missions,
            };
            if matches!(model.journal, MissionJournalUi::Other(_)) {
                model.journal = MissionJournalUi::Other(journal.clone());
            }
            model.nanocom_journal = journal;
        }
        Err(error) => {
            model.nanocom_journal = JournalOtherUi {
                title: "MISSION JOURNAL".to_owned(),
                ..default()
            };
            runtime.message = format!("Tutorial mission journal rejected: {error}");
        }
    }
    // Chapter 06 deliberately owns the mission-selection modal until its
    // TaskStart observation. Keep that modal bound to the scripted NPC even
    // if a transient input/UI edge clears TutorialActor::interacting. The
    // active task is the release condition, so a real row selection still
    // closes the list immediately after the mission chain advances.
    let scripted_selection_actor_id = match tutorial.progress.stage() {
        Some(TutorialStage::Infection(InfectionStage::SelectMission))
            if mission_runtime.task_is_active(2250) =>
        {
            Some(BUTTERCUP_ID)
        }
        Some(TutorialStage::Infection(InfectionStage::SelectDexterMission))
            if mission_runtime.task_is_active(2253) =>
        {
            Some(LAIR_DEXTER_ID)
        }
        _ => None,
    };
    let interacting = scripted_selection_actor_id
        .and_then(|id| {
            actors
                .iter()
                .find(|actor| actor.id == id && actor.is_alive())
        })
        .or_else(|| {
            actors
                .iter()
                .find(|actor| actor.interacting && actor.is_alive())
        })
        .copied();

    if let Some(actor) = interacting {
        let mut available_missions = Vec::new();
        let mut completed_missions = Vec::new();
        match actor.npc_type {
            2671 if mission_runtime.task_is_active(2249) => {
                if let Ok(mission) = content.mission_entry(2249, actor.id, runtime.map_name.clone())
                {
                    completed_missions.push(mission);
                }
            }
            2671 if !mission_runtime.task_is_active(2248)
                && !mission_runtime.completed_tasks.contains(&2248) =>
            {
                if let Ok(mission) = content.mission_entry(2248, actor.id, runtime.map_name.clone())
                {
                    available_missions.push(mission);
                }
            }
            2672 if mission_runtime.task_is_active(2250) => {
                if let Ok(mission) = content.mission_entry(2250, actor.id, runtime.map_name.clone())
                {
                    completed_missions.push(mission);
                }
            }
            2673 if mission_runtime.task_is_active(2253) => {
                if let Ok(mission) = content.mission_entry(2253, actor.id, runtime.map_name.clone())
                {
                    completed_missions.push(mission);
                }
            }
            _ => {}
        }
        let warp = content
            .warp(actor.npc_type)
            .ok()
            .filter(|definition| tutorial_warp_is_available(definition, &mission_runtime))
            .map(|definition| WarpUiEntry {
                npc_id: actor.id,
                npc_type: actor.npc_type,
                warp_id: definition.provenance.warp_id,
                required_task_id: definition.required_task_id,
                target: definition.target,
                label: definition.label.clone(),
            });
        // `NpcIconMode.InitMode` always appends its CLOSE action, even when
        // `CheckQuest` and every service/warp check are empty. Keeping the
        // interaction itself is what lets TaskStart/TaskEnd return to the
        // original 242x114 close-only `single_win`.
        let interaction = NpcInteractionUi {
            npc_id: actor.id,
            npc_type: actor.npc_type,
            name: content
                .npc_name(actor.npc_type)
                .unwrap_or_default()
                .to_owned(),
            available_missions,
            completed_missions,
            services: Vec::new(),
            warp,
        };
        let changed_actor = model
            .npc_interaction
            .as_ref()
            .is_none_or(|current| current.npc_id != actor.id);
        if changed_actor {
            model.show_npc_interaction(interaction);
        } else {
            if model.npc_interaction.as_ref() != Some(&interaction) {
                model.npc_interaction = Some(interaction);
            }
            // The TutorialActor interaction is the authoritative owner of
            // NpcIconMode. A presentation edge (for example a simultaneous
            // NanoCom/context transition) may have hidden the model after the
            // actor was first bound. Do not require another TalkNpc edge to
            // recover the mission list while that same actor is still live.
            if model.pending.is_none()
                && model.pending_warp.is_none()
                && matches!(model.journal, MissionJournalUi::Hidden)
            {
                model.npc_icon_mode_visible = true;
            }
        }
    } else if model.pending.is_none()
        && model.pending_warp.is_none()
        && matches!(model.journal, MissionJournalUi::Hidden)
    {
        model.npc_icon_mode_visible = false;
        model.npc_interaction = None;
    }

    let hostile_target_selected = logic.ui.hostile_target_selected;
    logic.ui = model.tutorial_observation();
    logic.ui.hostile_target_selected = hostile_target_selected;
}

pub(super) fn sync_tutorial_world_mission_indicators(
    mission: Res<TutorialMissionRuntime>,
    content: Res<TutorialMissionContent>,
    actors: Query<(Entity, &TutorialActor, &NetworkNpcVisual0104, &Transform)>,
    names: Query<&Name>,
    children: Query<&Children>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    mut presentation: ResMut<TutorialMissionIndicatorRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    const QUEST_SYMBOL_SOURCE_LINE: u32 = 3225;
    const SMART_INDICATOR_SOURCE_LINE: u32 = 2397;

    let selected_waypoint_npc_types = tutorial_selected_waypoint_npc_types(&mission, &content);
    let desired_smart_indicators = actors
        .iter()
        .filter_map(|(_, actor, _, transform)| {
            tutorial_actor_smart_indicator_scale(&content, actor, &selected_waypoint_npc_types).map(
                |scale| {
                    (
                        actor.id,
                        (
                            tutorial_actor_root_name(actor.id),
                            transform.rotation,
                            scale,
                        ),
                    )
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let stale_smart_indicators = presentation
        .smart_indicators
        .iter()
        .copied()
        .filter(|actor_id| !desired_smart_indicators.contains_key(actor_id))
        .collect::<Vec<_>>();
    for actor_id in stale_smart_indicators {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: tutorial_smart_indicator_name(actor_id),
            source_line: SMART_INDICATOR_SOURCE_LINE,
        });
        presentation.smart_indicators.remove(&actor_id);
    }

    for (&actor_id, (node_name, rotation, scale)) in &desired_smart_indicators {
        let effect_name = tutorial_smart_indicator_name(actor_id);
        if presentation.smart_indicators.contains(&actor_id)
            && effects.has_named_native_instance(&effect_name)
        {
            continue;
        }
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: 668,
            placement: TutorialEffectPlacement::ExactBone {
                actor_id,
                node_name: node_name.clone(),
                spawn_world_rotation: *rotation,
                local_translation_after_parenting: Vec3::ZERO,
                local_rotation_after_parenting: Quat::IDENTITY,
            },
            scale: *scale,
            tracked: false,
            name: Some(effect_name),
            destroy_after_seconds: None,
            source_line: SMART_INDICATOR_SOURCE_LINE,
        });
        presentation.smart_indicators.insert(actor_id);
    }

    let desired_symbols = actors
        .iter()
        .filter(|(_, actor, _, _)| actor.is_alive())
        .filter_map(|(entity, actor, _visual, transform)| {
            tutorial_npc_mission_symbol(actor, &mission).map(|symbol| {
                (
                    actor.id,
                    (symbol, actor.npc_type, transform.rotation, entity, true),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();

    let stale_symbols = presentation
        .mission_symbols
        .iter()
        .filter_map(|(&actor_id, &previous)| {
            desired_symbols
                .get(&actor_id)
                .is_none_or(|(next, _, _, _, _)| *next != previous)
                .then_some(actor_id)
        })
        .collect::<Vec<_>>();
    for actor_id in stale_symbols {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: tutorial_quest_symbol_name(actor_id),
            source_line: QUEST_SYMBOL_SOURCE_LINE,
        });
        presentation.mission_symbols.remove(&actor_id);
    }

    for (&actor_id, &(symbol, npc_type, rotation, actor_entity, expects_scene)) in &desired_symbols
    {
        let effect_name = tutorial_quest_symbol_name(actor_id);
        if presentation.mission_symbols.get(&actor_id) == Some(&symbol)
            && effects.has_named_native_instance(&effect_name)
        {
            continue;
        }
        let Some((node_name, local_translation, local_rotation)) = tutorial_quest_symbol_attachment(
            actor_entity,
            actor_id,
            expects_scene,
            content
                .gameplay_npc(npc_type)
                .map(|definition| definition.height()),
            &names,
            &children,
            &scene_statuses,
        ) else {
            // The source resolves GameIcon only after SetModel has produced
            // the hierarchy. Retry instead of losing the one-shot add.
            continue;
        };
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: symbol.effect_id(),
            placement: TutorialEffectPlacement::ExactBone {
                actor_id,
                node_name,
                spawn_world_rotation: rotation,
                local_translation_after_parenting: local_translation,
                local_rotation_after_parenting: local_rotation,
            },
            scale: 1.0,
            tracked: false,
            name: Some(effect_name),
            destroy_after_seconds: None,
            source_line: QUEST_SYMBOL_SOURCE_LINE,
        });
        presentation.mission_symbols.insert(actor_id, symbol);
    }
}

pub(super) fn tutorial_descendants_named(
    entity: Entity,
    target_name: &str,
    names: &Query<&Name>,
    children: &Query<&Children>,
    matches: &mut Vec<Entity>,
) {
    if names
        .get(entity)
        .is_ok_and(|name| name.as_str() == target_name)
    {
        matches.push(entity);
    }
    let Ok(entity_children) = children.get(entity) else {
        return;
    };
    for child in entity_children.iter() {
        tutorial_descendants_named(child, target_name, names, children, matches);
    }
}

pub(super) fn tutorial_actor_scene_is_terminal(
    entity: Entity,
    scene_statuses: &Query<&LegacyCharacterSceneStatus>,
    children: &Query<&Children>,
) -> bool {
    if scene_statuses.get(entity).is_ok_and(|status| {
        matches!(
            status,
            LegacyCharacterSceneStatus::Ready { .. } | LegacyCharacterSceneStatus::Blocked(_)
        )
    }) {
        return true;
    }
    children.get(entity).is_ok_and(|entity_children| {
        entity_children
            .iter()
            .any(|child| tutorial_actor_scene_is_terminal(child, scene_statuses, children))
    })
}

pub(super) fn tutorial_stun_effect_name(actor_id: i32) -> String {
    format!("tutorial NPC {actor_id} stun condition")
}

pub(super) fn tutorial_stun_effect_attachment(
    actor_entity: Entity,
    actor_id: i32,
    names: &Query<&Name>,
    children: &Query<&Children>,
    scene_statuses: &Query<&LegacyCharacterSceneStatus>,
) -> Option<(String, Quat)> {
    let mut matches = Vec::new();
    tutorial_descendants_named(
        actor_entity,
        TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE,
        names,
        children,
        &mut matches,
    );
    if matches.len() == 1 {
        return Some((
            TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE.to_owned(),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ));
    }

    // `Status.UpdateSkillBuff` falls back to the character root when a model
    // has no `state` node. Wait for asynchronous scene resolution first so a
    // temporarily incomplete hierarchy cannot choose that fallback early.
    tutorial_actor_scene_is_terminal(actor_entity, scene_statuses, children)
        .then(|| (tutorial_actor_root_name(actor_id), Quat::IDENTITY))
}

pub(super) fn tutorial_stun_effect_command(
    actor_entity: Entity,
    actor: &TutorialActor,
    actor_rotation: Quat,
    content: &TutorialMissionContent,
    node_name: String,
    local_rotation: Quat,
) -> Option<TutorialEffectRuntimeCommand> {
    let scale = content.gameplay_npc(actor.npc_type)?.radius() * 2.0;
    Some(TutorialEffectRuntimeCommand::Add {
        effect_id: TUTORIAL_BUTTERCUP_STUN_EFFECT_ID,
        placement: TutorialEffectPlacement::ExactEntityBone {
            root_entity: actor_entity,
            node_name,
            spawn_world_rotation: actor_rotation * local_rotation,
            local_translation_after_parenting: Vec3::ZERO,
            local_rotation_after_parenting: local_rotation,
        },
        scale,
        tracked: false,
        name: Some(tutorial_stun_effect_name(actor.id)),
        destroy_after_seconds: None,
        // Clean `Status.UpdateSkillBuff`: condition slot 10 owns ES366.
        source_line: 806,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_tutorial_nano_condition_effects(
    content: Res<TutorialMissionContent>,
    actors: Query<(
        Entity,
        &TutorialActor,
        &TutorialNanoSkillCondition,
        &Transform,
        &NetworkNpcVisual0104,
    )>,
    names: Query<&Name>,
    children: Query<&Children>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    mut presentation: ResMut<TutorialNanoConditionEffectRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    let desired = actors
        .iter()
        .filter(|(_, actor, condition, _, _)| {
            actor.is_alive() && condition.0 & TUTORIAL_BUTTERCUP_SKILL_CONDITION != 0
        })
        .map(|(entity, actor, _, transform, _)| (entity, (actor, transform.rotation)))
        .collect::<BTreeMap<_, _>>();

    let stale = presentation
        .stun_actor_ids
        .iter()
        .filter_map(|(&entity, &actor_id)| {
            desired
                .get(&entity)
                .is_none_or(|(actor, _)| actor.id != actor_id)
                .then_some((entity, actor_id))
        })
        .collect::<Vec<_>>();
    for (entity, actor_id) in stale {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: tutorial_stun_effect_name(actor_id),
            source_line: 806,
        });
        presentation.stun_actor_ids.remove(&entity);
    }

    for (&entity, &(actor, actor_rotation)) in &desired {
        let effect_name = tutorial_stun_effect_name(actor.id);
        if presentation.stun_actor_ids.get(&entity) == Some(&actor.id)
            && effects.has_named_native_instance(&effect_name)
        {
            continue;
        }
        let Some((node_name, local_rotation)) =
            tutorial_stun_effect_attachment(entity, actor.id, &names, &children, &scene_statuses)
        else {
            continue;
        };
        let Some(command) = tutorial_stun_effect_command(
            entity,
            actor,
            actor_rotation,
            &content,
            node_name,
            local_rotation,
        ) else {
            continue;
        };
        effects.enqueue(command);
        presentation.stun_actor_ids.insert(entity, actor.id);
    }
}

pub(super) fn cleanup_tutorial_nano_condition_effects(
    mut presentation: ResMut<TutorialNanoConditionEffectRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    for actor_id in presentation.stun_actor_ids.values().copied() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: tutorial_stun_effect_name(actor_id),
            source_line: 806,
        });
    }
    *presentation = TutorialNanoConditionEffectRuntime::default();
}

pub(super) fn tutorial_quest_symbol_attachment(
    actor_entity: Entity,
    actor_id: i32,
    expects_scene: bool,
    xdt_height: Option<f32>,
    names: &Query<&Name>,
    children: &Query<&Children>,
    scene_statuses: &Query<&LegacyCharacterSceneStatus>,
) -> Option<(String, Vec3, Quat)> {
    for node_name in ["GameIcon"] {
        let mut matches = Vec::new();
        tutorial_descendants_named(actor_entity, node_name, names, children, &mut matches);
        if matches.len() == 1 {
            return Some((node_name.to_owned(), Vec3::ZERO, Quat::IDENTITY));
        }
    }

    if expects_scene && !tutorial_actor_scene_is_terminal(actor_entity, scene_statuses, children) {
        return None;
    }

    let height = xdt_height?;
    Some((
        tutorial_actor_root_name(actor_id),
        Vec3::Y * (height * 0.85),
        Quat::IDENTITY,
    ))
}

pub(super) fn cleanup_tutorial_mission_indicators(
    mut presentation: ResMut<TutorialMissionIndicatorRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    for actor_id in presentation.smart_indicators.iter().copied() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: tutorial_smart_indicator_name(actor_id),
            source_line: 2397,
        });
    }
    for actor_id in presentation.mission_symbols.keys().copied() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: tutorial_quest_symbol_name(actor_id),
            source_line: 3225,
        });
    }
    *presentation = TutorialMissionIndicatorRuntime::default();
}

pub(super) fn cleanup_pending_tutorial_actor_effects(mut pending: ResMut<PendingTutorialActorEffects>) {
    *pending = PendingTutorialActorEffects::default();
}
