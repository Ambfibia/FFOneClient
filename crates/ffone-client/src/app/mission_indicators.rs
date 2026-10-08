//! Tutorial and world mission symbols, waypoints, NPC game icons and indicator sync.

use super::LocalPlayer;
use super::local_inventory::LocalInventoryRuntime;
use super::race::RaceProductionRuntime;
use super::runtime_status::RuntimeStatus;
use super::tutorial_indicators::{tutorial_actor_scene_is_terminal, tutorial_descendants_named};
use super::tutorial_session::TutorialMissionRuntime;
use bevy::prelude::*;
use ffone_client::{
    character_scene::LegacyCharacterSceneStatus,
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_ui::{CurrentObjectiveProgressUi, CurrentObjectiveUi, MinimapMarkerIcon},
    guide_runtime::GuideRuntime,
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    mission_ui::MissionUiModel,
    movement::SERVER_TO_CLIENT_SCALE,
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    network_world_runtime::{
        NetworkHnpcVisual0104, NetworkNpcVisual0104, NetworkNpcVisualIssue0104,
    },
    tutorial_actors::TutorialActor,
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
    },
    tutorial_mission_content::TutorialMissionContent,
    world_mission_indicators::{
        ActiveMissionWaypointTask, ClientNpcWaypointCatalog, ClientNpcWaypointUpdate,
        WORLD_MISSION_INDICATOR_REFRESH_RANGE, WORLD_QUEST_SYMBOL_ROOT_HEIGHT_FACTOR,
        WorldMissionIndicatorEligibilityInput, WorldMissionIndicatorRefresh,
        WorldMissionIndicatorSymbol, project_selected_mission_waypoints,
        project_world_mission_indicator_refresh, resolve_client_npc_waypoint_update,
        world_npc_game_icon_effect,
    },
    world_mission_runtime::{WorldMissionNearbyNpc, WorldMissionRuntime},
};
use ffone_protocol::NpcBarkerRequest0104;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MinimapMissionSymbol {
    Advance,
    New,
}

impl MinimapMissionSymbol {
    pub(super) const fn effect_id(self) -> i32 {
        match self {
            Self::Advance => 865,
            Self::New => 866,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialMissionIndicatorRuntime {
    pub(super) mission_symbols: BTreeMap<i32, MinimapMissionSymbol>,
    pub(super) smart_indicators: BTreeSet<i32>,
}

#[derive(Debug, Default, Resource)]
pub(super) struct TutorialNanoConditionEffectRuntime {
    pub(super) stun_actor_ids: BTreeMap<Entity, i32>,
}

#[derive(Debug, Default, Resource)]
pub(super) struct WorldMissionWaypointRuntime {
    pub(super) native_target: Option<Vec3>,
    pub(super) npc_type: Option<i32>,
    pub(super) source_row_index: Option<u32>,
}

impl WorldMissionWaypointRuntime {
    pub(super) fn apply(&mut self, update: ClientNpcWaypointUpdate) {
        match update {
            ClientNpcWaypointUpdate::Clear => *self = Self::default(),
            ClientNpcWaypointUpdate::PreservePrevious { .. } => {}
            ClientNpcWaypointUpdate::Set {
                npc_type,
                source_row_index,
                native_position,
            } => {
                self.native_target = Some(native_position);
                self.npc_type = Some(npc_type);
                self.source_row_index = Some(source_row_index);
            }
        }
    }
}

#[derive(Debug, Resource)]
pub(super) struct WorldMissionIndicatorRuntime {
    pub(super) refresh_elapsed_seconds: f32,
    pub(super) mission_symbols: BTreeMap<i32, WorldMissionIndicatorSymbol>,
    pub(super) smart_indicators: BTreeSet<i32>,
    pub(super) game_icons: BTreeMap<i32, i32>,
}

/// Clean `cnMissionManager.Update` cadence for completed-mission ambient
/// barks: one refresh per second and one request scan every 20 refreshes.
#[derive(Debug, Default, Resource)]
pub(super) struct WorldMissionBarkerRequestRuntime {
    pub(super) last_refresh_time: f64,
    pub(super) loop_count: u8,
}

pub(super) const WORLD_MISSION_BARKER_REFRESH_SECONDS: f64 = 1.0;
pub(super) const WORLD_MISSION_BARKER_REFRESH_COUNT: u8 = 20;
pub(super) const WORLD_MISSION_BARKER_NEAR_DISTANCE: f32 = 5_000.0;

impl Default for WorldMissionIndicatorRuntime {
    fn default() -> Self {
        Self {
            // Clean also refreshes when mission state is initialized. Start
            // ready so the first World frame does not wait for the periodic
            // one-second reconciliation pass.
            refresh_elapsed_seconds: 1.0,
            mission_symbols: BTreeMap::new(),
            smart_indicators: BTreeSet::new(),
            game_icons: BTreeMap::new(),
        }
    }
}

pub(super) fn tutorial_npc_mission_symbol(
    actor: &TutorialActor,
    mission: &TutorialMissionRuntime,
    content:&TutorialMissionContent,
    surface:ffone_client::tutorial_mission_content::MissionMarkerSurface,
) -> Option<MinimapMissionSymbol> {
    let (task,symbol)=match actor.npc_type {
        2671 if mission.task_is_active(2249) => (2249,MinimapMissionSymbol::Advance),
        2671 if !mission.task_is_active(2248) && !mission.completed_tasks.contains(&2248) => {
            (2248,MinimapMissionSymbol::New)
        }
        2672 if mission.task_is_active(2250) => (2250,MinimapMissionSymbol::Advance),
        2673 if mission.task_is_active(2253) => (2253,MinimapMissionSymbol::Advance),
        _ => return None,
    };
    content.mission(task).ok().filter(|definition|definition.provenance.marker_visibility.visible_on(surface)).map(|_|symbol)
}

pub(super) const fn tutorial_minimap_marker_icon(
    npc_type: i32,
    mission_symbol: Option<MinimapMissionSymbol>,
) -> Option<MinimapMarkerIcon> {
    match mission_symbol {
        Some(MinimapMissionSymbol::New) => Some(MinimapMarkerIcon::New),
        Some(MinimapMissionSymbol::Advance) => Some(MinimapMarkerIcon::Advance),
        None => match npc_type {
            2671..=2673 => Some(MinimapMarkerIcon::ShowNpc),
            2674..=2677 => Some(MinimapMarkerIcon::Mob),
            2678 => Some(MinimapMarkerIcon::Fusion),
            _ => None,
        },
    }
}

pub(super) fn tutorial_quest_symbol_name(actor_id: i32) -> String {
    format!("Tutorial quest symbol for actor {actor_id}")
}

pub(super) fn tutorial_smart_indicator_name(actor_id: i32) -> String {
    format!("Tutorial selected-mission smart indicator for actor {actor_id}")
}

pub(super) fn tutorial_selected_waypoint_npc_types(
    mission: &TutorialMissionRuntime,
    content: &TutorialMissionContent,
) -> BTreeSet<i32> {
    let Some(selected_mission_id) = mission.selected_mission_id else {
        return BTreeSet::new();
    };
    mission
        .active_tasks
        .iter()
        .filter_map(|task_id| content.mission(*task_id).ok())
        .filter(|definition| definition.provenance.mission_id == selected_mission_id)
        .filter(|definition|definition.provenance.marker_visibility.visible_on(ffone_client::tutorial_mission_content::MissionMarkerSurface::Overhead))
        .map(|definition| definition.provenance.grant_waypoint_npc_type)
        .filter(|npc_type| *npc_type > 0)
        .collect()
}

pub(super) fn tutorial_current_objective_ui(
    mission: &TutorialMissionRuntime,
    content: &TutorialMissionContent,
    display_enabled: bool,
) -> CurrentObjectiveUi {
    if !display_enabled {
        return CurrentObjectiveUi::default();
    }
    let Some(selected_mission_id) = mission.selected_mission_id else {
        return CurrentObjectiveUi::default();
    };
    mission
        .active_tasks
        .iter()
        .filter_map(|task_id| content.mission(*task_id).ok())
        .find(|definition| definition.provenance.mission_id == selected_mission_id)
        .map_or_else(CurrentObjectiveUi::default, |definition| {
            let enemies = (0..3)
                .filter_map(|index| {
                    let npc_type = definition.provenance.completion_enemy_ids[index];
                    let needed = definition.provenance.completion_enemy_counts[index];
                    (npc_type > 0 && needed > 0).then(|| CurrentObjectiveProgressUi {
                        content_id: npc_type,
                        name: content
                            .gameplay_npc(npc_type)
                            .expect("validated tutorial mission enemy")
                            .name
                            .clone(),
                        complete: 0,
                        needed,
                    })
                })
                .collect();
            let quest_items = (0..3)
                .filter_map(|index| {
                    let item_id = definition.provenance.completion_item_ids[index];
                    let needed = definition.provenance.completion_item_counts[index];
                    (item_id > 0 && needed > 0).then(|| CurrentObjectiveProgressUi {
                        content_id: item_id,
                        name: content
                            .quest_item_name(item_id)
                            .expect("validated tutorial mission quest item")
                            .to_owned(),
                        complete: 0,
                        needed,
                    })
                })
                .collect();
            CurrentObjectiveUi {
                visible: true,
                task_id: Some(definition.provenance.task_id),
                title: definition.title.clone(),
                body: definition.objective.clone(),
                remaining_time_seconds: None,
                enemies,
                quest_items,
            }
        })
}

pub(super) fn tutorial_smart_indicator_scale(content: &TutorialMissionContent, npc_type: i32) -> Option<f32> {
    content
        .gameplay_npc(npc_type)
        .map(|definition| definition.radius() * 2.0)
}

pub(super) fn tutorial_actor_smart_indicator_scale(
    content: &TutorialMissionContent,
    actor: &TutorialActor,
    selected_waypoint_npc_types: &BTreeSet<i32>,
) -> Option<f32> {
    let definition = content.gameplay_npc(actor.npc_type)?;
    (actor.is_alive()
        && !actor.interacting
        && definition.npc_class != 0
        && definition.npc_class != 25
        && selected_waypoint_npc_types.contains(&actor.npc_type))
    .then(|| tutorial_smart_indicator_scale(content, actor.npc_type))
    .flatten()
}

pub(super) fn active_world_mission_waypoint_tasks(
    mission: &WorldMissionRuntime,
    content: &TutorialMissionContent,
    surface:ffone_client::tutorial_mission_content::MissionMarkerSurface,
) -> Vec<ActiveMissionWaypointTask> {
    mission
        .active_tasks()
        .iter()
        .filter_map(|active| {
            let definition = content.mission(active.task_id).ok()?;
            if !definition.provenance.marker_visibility.visible_on(surface){return None;}
            Some(ActiveMissionWaypointTask {
                task_id: active.task_id,
                mission_id: definition.provenance.mission_id,
                grant_waypoint_npc_type: definition.provenance.grant_waypoint_npc_type,
            })
        })
        .collect()
}

pub(super) fn sync_world_mission_waypoint(
    mission: Res<WorldMissionRuntime>,
    content: Res<TutorialMissionContent>,
    catalog: Res<ClientNpcWaypointCatalog>,
    mut presentation: ResMut<WorldMissionWaypointRuntime>,
) {
    let active_tasks = active_world_mission_waypoint_tasks(&mission, &content,ffone_client::tutorial_mission_content::MissionMarkerSurface::Minimap);
    let projection =
        project_selected_mission_waypoints(mission.selected_mission_id(), &active_tasks);
    presentation.apply(resolve_client_npc_waypoint_update(
        projection.current_grant_waypoint_npc_type,
        &catalog,
    ));
}

pub(super) fn world_smart_indicator_name(npc_id: i32) -> String {
    format!("world NPC {npc_id} smart mission indicator")
}

pub(super) fn world_quest_symbol_name(npc_id: i32) -> String {
    format!("world NPC {npc_id} quest symbol")
}

pub(super) fn world_npc_game_icon_name(npc_id: i32) -> String {
    format!("world NPC {npc_id} game icon")
}

pub(super) fn world_network_npc_root_name(npc_id: i32, npc_type: i32) -> String {
    format!("network NPC {npc_id} type {npc_type}")
}

pub(super) fn world_network_hnpc_root_name(npc_id: i32, npc_type: i32) -> String {
    format!("network HNPC {npc_id} type {npc_type}")
}

pub(super) fn world_network_npc_visual_root_name(npc_id: i32, npc_type: i32, has_hnpc_visual: bool) -> String {
    if has_hnpc_visual {
        world_network_hnpc_root_name(npc_id, npc_type)
    } else {
        world_network_npc_root_name(npc_id, npc_type)
    }
}

pub(super) fn world_npc_overhead_effect_attachment(
    root_entity: Entity,
    npc_id: i32,
    npc_type: i32,
    height_server_units: i32,
    has_direct_visual: bool,
    has_hnpc_visual: bool,
    visual_issue: bool,
    names: &Query<&Name>,
    children: &Query<&Children>,
    scene_statuses: &Query<&LegacyCharacterSceneStatus>,
) -> Option<(String, Vec3, Quat)> {
    for node_name in ["GameIcon"] {
        let mut matches = Vec::new();
        tutorial_descendants_named(root_entity, node_name, names, children, &mut matches);
        if matches.len() == 1 {
            return Some((node_name.to_owned(), Vec3::ZERO, Quat::IDENTITY));
        }
    }

    // Network visuals are classified asynchronously as either an ordinary
    // scene or an avatar-composed HNPC rig. Do not commit a named attachment
    // against the temporary lifecycle root before that classification lands:
    // the HNPC resolver renames the root from `network NPC` to `network HNPC`.
    if !has_direct_visual && !has_hnpc_visual && !visual_issue {
        return None;
    }

    if has_direct_visual
        && !visual_issue
        && !tutorial_actor_scene_is_terminal(root_entity, scene_statuses, children)
    {
        return None;
    }

    Some((
        world_network_npc_visual_root_name(npc_id, npc_type, has_hnpc_visual),
        Vec3::Y
            * (height_server_units as f32
                * SERVER_TO_CLIENT_SCALE
                * WORLD_QUEST_SYMBOL_ROOT_HEIGHT_FACTOR),
        Quat::IDENTITY,
    ))
}

/// `NpcMoveController.SetupNPC` calls `MakeGameIcon` when each NPC finishes
/// loading. Keep this owner independent from the one-second mission refresh:
/// service icons must materialize even while there is no unique LocalPlayer,
/// and a late/rebuilt character hierarchy must be retried on the next frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn sync_world_npc_game_icons(
    content: Res<TutorialMissionContent>,
    runtime: Res<RuntimeStatus>,
    race_production: Res<RaceProductionRuntime>,
    npcs: Query<(
        Entity,
        &NetworkNpcAppearance0104,
        &GlobalTransform,
        Option<&NetworkNpcVisual0104>,
        Option<&NetworkHnpcVisual0104>,
        Option<&NetworkNpcVisualIssue0104>,
    )>,
    names: Query<&Name>,
    children: Query<&Children>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    mut presentation: ResMut<WorldMissionIndicatorRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    const GAME_ICON_SOURCE_LINE: u32 = 187;

    let live_npc_ids = npcs
        .iter()
        .filter_map(|(_, appearance, _, _, _, _)| {
            (appearance.0.hp > 0).then_some(appearance.0.npc_id)
        })
        .collect::<BTreeSet<_>>();
    let stale_game_icons = presentation
        .game_icons
        .keys()
        .copied()
        .filter(|npc_id| !live_npc_ids.contains(npc_id))
        .collect::<Vec<_>>();
    for npc_id in stale_game_icons {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: world_npc_game_icon_name(npc_id),
            source_line: GAME_ICON_SOURCE_LINE,
        });
        presentation.game_icons.remove(&npc_id);
    }

    for (entity, appearance, transform, visual, hnpc_visual, visual_issue) in &npcs {
        let npc_id = appearance.0.npc_id;
        if appearance.0.hp <= 0 {
            continue;
        }
        let Some(definition) = content.gameplay_npc(appearance.0.npc_type) else {
            continue;
        };
        let base_game_icon = world_npc_game_icon_effect(
            appearance.0.npc_type,
            definition.npc_class,
            definition.attack_effect,
            race_production.player.ring_race_active,
            runtime.nano_recall.registered_here(npc_id),
        );
        // `SetQuestSymbol` destroys `pEffectGameIcon`; when the quest symbol
        // clears it calls `MakeGameIcon` to restore the class-specific icon.
        let desired_game_icon = (!presentation.mission_symbols.contains_key(&npc_id))
            .then_some(base_game_icon)
            .flatten();
        let game_icon_name = world_npc_game_icon_name(npc_id);
        if presentation.game_icons.get(&npc_id).copied() != desired_game_icon
            && presentation.game_icons.remove(&npc_id).is_some()
        {
            effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
                name: game_icon_name.clone(),
                source_line: GAME_ICON_SOURCE_LINE,
            });
        }
        let Some(effect_id) = desired_game_icon else {
            continue;
        };
        if presentation.game_icons.get(&npc_id) == Some(&effect_id)
            && effects.has_named_native_instance(&game_icon_name)
        {
            continue;
        }
        let Some((node_name, local_translation, local_rotation)) =
            world_npc_overhead_effect_attachment(
                entity,
                npc_id,
                appearance.0.npc_type,
                definition.height_server_units,
                visual.is_some(),
                hnpc_visual.is_some(),
                visual_issue.is_some(),
                &names,
                &children,
                &scene_statuses,
            )
        else {
            continue;
        };
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id,
            placement: TutorialEffectPlacement::ExactEntityBone {
                root_entity: entity,
                node_name,
                spawn_world_rotation: transform.rotation(),
                local_translation_after_parenting: local_translation,
                local_rotation_after_parenting: local_rotation,
            },
            scale: 1.0,
            tracked: false,
            name: Some(game_icon_name),
            destroy_after_seconds: None,
            source_line: GAME_ICON_SOURCE_LINE,
        });
        presentation.game_icons.insert(npc_id, effect_id);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_world_mission_indicators(
    time: Res<Time>,
    runtime: Res<RuntimeStatus>,
    mission: Res<WorldMissionRuntime>,
    content: Res<TutorialMissionContent>,
    inventory: Res<LocalInventoryRuntime>,
    guide: Res<GuideRuntime>,
    nano_bank: Res<NanoFreeTuningBank0104>,
    mission_ui: Res<MissionUiModel>,
    local_players: Query<&GlobalTransform, With<LocalPlayer>>,
    npcs: Query<(
        Entity,
        &NetworkNpcAppearance0104,
        &GlobalTransform,
        Option<&NetworkNpcVisual0104>,
        Option<&NetworkHnpcVisual0104>,
        Option<&NetworkNpcVisualIssue0104>,
    )>,
    names: Query<&Name>,
    children: Query<&Children>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    mut presentation: ResMut<WorldMissionIndicatorRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    const SMART_INDICATOR_SOURCE_LINE: u32 = 2397;
    const QUEST_SYMBOL_SOURCE_LINE: u32 = 3225;

    let live_npc_ids = npcs
        .iter()
        .filter_map(|(_, appearance, _, _, _, _)| {
            (appearance.0.hp > 0).then_some(appearance.0.npc_id)
        })
        .collect::<BTreeSet<_>>();
    let stale_smart = presentation
        .smart_indicators
        .iter()
        .copied()
        .filter(|npc_id| !live_npc_ids.contains(npc_id))
        .collect::<Vec<_>>();
    for npc_id in stale_smart {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: world_smart_indicator_name(npc_id),
            source_line: SMART_INDICATOR_SOURCE_LINE,
        });
        presentation.smart_indicators.remove(&npc_id);
    }
    let stale_symbols = presentation
        .mission_symbols
        .keys()
        .copied()
        .filter(|npc_id| !live_npc_ids.contains(npc_id))
        .collect::<Vec<_>>();
    for npc_id in stale_symbols {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: world_quest_symbol_name(npc_id),
            source_line: QUEST_SYMBOL_SOURCE_LINE,
        });
        presentation.mission_symbols.remove(&npc_id);
    }
    presentation.refresh_elapsed_seconds += time.delta_secs();
    if presentation.refresh_elapsed_seconds < 1.0 {
        return;
    }
    presentation.refresh_elapsed_seconds %= 1.0;

    let Ok(player) = local_players.single() else {
        return;
    };
    let player_position = player.translation();
    let active_tasks = active_world_mission_waypoint_tasks(&mission, &content,ffone_client::tutorial_mission_content::MissionMarkerSurface::Overhead);
    let selected_projection =
        project_selected_mission_waypoints(mission.selected_mission_id(), &active_tasks);
    let owned_nanos = nano_bank
        .entries()
        .iter()
        .filter_map(|nano| (nano.id > 0).then_some(i32::from(nano.id)))
        .collect::<BTreeSet<_>>();
    let guide = guide
        .authoritative()
        .map_or(0, |state| i32::from(state.raw_mentor()));
    let quest_inventory = inventory
        .quest_inventory
        .as_ref()
        .map(|inventory| inventory.as_slice())
        .unwrap_or(&[]);

    for (entity, appearance, transform, visual, hnpc_visual, visual_issue) in &npcs {
        let npc_id = appearance.0.npc_id;
        if appearance.0.hp <= 0 {
            continue;
        }
        let Some(definition) = content.gameplay_npc(appearance.0.npc_type) else {
            continue;
        };
        let in_refresh_near_list = player_position.distance(transform.translation())
            < WORLD_MISSION_INDICATOR_REFRESH_RANGE;
        let (has_new_mission_available, has_active_terminating_task) = mission
            .npc_mission_markers_on(
                appearance.0.npc_type,
                ffone_client::tutorial_mission_content::MissionMarkerSurface::Overhead,
                i32::from(runtime.player_level),
                guide,
                &owned_nanos,
                quest_inventory,
                &content,
            );
        let refresh =
            project_world_mission_indicator_refresh(WorldMissionIndicatorEligibilityInput {
                in_refresh_near_list,
                npc_class: definition.npc_class,
                selected_waypoint_target: selected_projection
                    .smart_indicator_npc_types
                    .contains(&appearance.0.npc_type),
                smart_force_deleted: mission_ui.npc_icon_mode_visible
                    && mission_ui
                        .npc_interaction
                        .as_ref()
                        .is_some_and(|interaction| interaction.npc_id == npc_id),
                radius_server_units: definition.radius_server_units,
                has_active_terminating_task,
                has_new_mission_available,
            });
        let WorldMissionIndicatorRefresh::Reconcile(desired) = refresh else {
            continue;
        };

        let smart_name = world_smart_indicator_name(npc_id);
        if let Some(smart) = desired.smart {
            if !presentation.smart_indicators.contains(&npc_id)
                || !effects.has_named_native_instance(&smart_name)
            {
                effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: smart.effect_id,
                    placement: TutorialEffectPlacement::ExactEntityBone {
                        root_entity: entity,
                        node_name: world_network_npc_visual_root_name(
                            npc_id,
                            appearance.0.npc_type,
                            hnpc_visual.is_some(),
                        ),
                        spawn_world_rotation: transform.rotation(),
                        local_translation_after_parenting: Vec3::ZERO,
                        local_rotation_after_parenting: Quat::IDENTITY,
                    },
                    scale: smart.scale,
                    tracked: false,
                    name: Some(smart_name),
                    destroy_after_seconds: None,
                    source_line: SMART_INDICATOR_SOURCE_LINE,
                });
                presentation.smart_indicators.insert(npc_id);
            }
        } else if presentation.smart_indicators.remove(&npc_id) {
            effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
                name: smart_name,
                source_line: SMART_INDICATOR_SOURCE_LINE,
            });
        }

        let symbol_name = world_quest_symbol_name(npc_id);
        if presentation.mission_symbols.get(&npc_id) != desired.quest_symbol.as_ref() {
            if presentation.mission_symbols.remove(&npc_id).is_some() {
                effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
                    name: symbol_name.clone(),
                    source_line: QUEST_SYMBOL_SOURCE_LINE,
                });
            }
        }
        let Some(symbol) = desired.quest_symbol else {
            continue;
        };
        if presentation.mission_symbols.get(&npc_id) == Some(&symbol)
            && effects.has_named_native_instance(&symbol_name)
        {
            continue;
        }
        let Some((node_name, local_translation, local_rotation)) =
            world_npc_overhead_effect_attachment(
                entity,
                npc_id,
                appearance.0.npc_type,
                definition.height_server_units,
                visual.is_some(),
                hnpc_visual.is_some(),
                visual_issue.is_some(),
                &names,
                &children,
                &scene_statuses,
            )
        else {
            continue;
        };
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: symbol.effect_id(),
            placement: TutorialEffectPlacement::ExactEntityBone {
                root_entity: entity,
                node_name,
                spawn_world_rotation: transform.rotation(),
                local_translation_after_parenting: local_translation,
                local_rotation_after_parenting: local_rotation,
            },
            scale: 1.0,
            tracked: false,
            name: Some(symbol_name),
            destroy_after_seconds: None,
            source_line: QUEST_SYMBOL_SOURCE_LINE,
        });
        presentation.mission_symbols.insert(npc_id, symbol);
    }
}

pub(super) fn reset_world_mission_barker_request(mut runtime: ResMut<WorldMissionBarkerRequestRuntime>) {
    *runtime = WorldMissionBarkerRequestRuntime::default();
}

pub(super) fn clean_world_mission_barker_request(
    mission: &WorldMissionRuntime,
    content: &TutorialMissionContent,
    player_position: [f32; 3],
    nearby_npcs: &[WorldMissionNearbyNpc],
    random: &mut LegacyNanoStandRandomStream,
) -> Option<NpcBarkerRequest0104> {
    let completed = mission
        .completed_task_ids()
        .filter_map(|task_id| content.mission(task_id).ok())
        .collect::<Vec<_>>();
    if completed.is_empty() {
        return None;
    }
    // Clean selects a completed mission before it scans the 5000-unit near
    // list, so a failed NPC match still consumes this one global RNG draw.
    let selected = completed[random.next_index(completed.len())];
    let player_position = Vec3::from_array(player_position);
    nearby_npcs.iter().find_map(|npc| {
        let position = Vec3::from_array(npc.position);
        if position.distance(player_position) >= WORLD_MISSION_BARKER_NEAR_DISTANCE {
            return None;
        }
        let barker_type = content.gameplay_npc_barker_type(npc.npc_type)?;
        let index = usize::try_from(barker_type.checked_sub(1)?).ok()?;
        if index >= selected.provenance.barker_text_ids.len()
            || selected.provenance.barker_text_ids[index] <= 0
        {
            return None;
        }
        Some(NpcBarkerRequest0104 {
            mission_task_id: selected.provenance.task_id,
            npc_id: npc.npc_id,
        })
    })
}
