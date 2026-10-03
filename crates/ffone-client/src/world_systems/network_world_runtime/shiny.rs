//! Presentation of server-owned eggs. The shiny table owns the mesh selection;
//! each placement remains a child of its network lifecycle entity.
use std::collections::BTreeMap;

use bevy::{
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
};
use serde_json::Value;

use super::{NetworkNpcVisualCatalogState0104, load_network_npc_visual_catalog_0104};
use crate::{
    character_scene::{NativeSceneRole, spawn_legacy_scene},
    coordinates::LegacyCharacterRootPolicy,
    entity_lifecycle::{NetworkShiny0104, consume_network_entity_lifecycle_0104},
};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShinyVisualDefinition {
    name: String,
    glb: String,
    pickup_effect: i32,
}

pub(super) fn definitions(
    document: &Value,
    registry: &Value,
) -> BTreeMap<i32, Result<ShinyVisualDefinition, String>> {
    let Some(table) = document["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|table| table["name"] == super::CONSOLIDATED_TABLE)
        })
        .and_then(|table| table.pointer("/value/m_pShinyTable"))
    else {
        return BTreeMap::new();
    };
    let Some(rows) = table["m_pShinyData"].as_array() else {
        return BTreeMap::new();
    };
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            // Protocol shiny_type is the table row index, as in TableContainer.
            let definition = (|| {
                let mesh_index =
                    row["m_iMesh"].as_u64().ok_or("missing shiny mesh index")? as usize;
                let mesh = table["m_pShinyMeshData"]
                    .get(mesh_index)
                    .ok_or("invalid shiny mesh index")?;
                let name = mesh["m_pstrMMeshModelString"]
                    .as_str()
                    .filter(|s| !s.trim().is_empty() && *s != "null")
                    .ok_or("empty shiny model")?;
                // Both published egg rows use their own authored atlas. Reject new
                // overrides until the runtime has an explicit binding contract.
                for key in ["m_pstrMTextureString", "m_pstrMTextureString2"] {
                    if mesh[key]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty() && s != "null")
                    {
                        return Err("shiny texture override requires a native binding".to_owned());
                    }
                }
                let models = registry["models"]
                    .as_array()
                    .ok_or("missing native model routes")?;
                let mut matches = models
                    .iter()
                    .filter(|m| m["logicalName"].as_str() == Some(name));
                let model = matches
                    .next()
                    .ok_or_else(|| format!("no native shiny model route for {name}"))?;
                if matches.next().is_some() {
                    return Err(format!("ambiguous native shiny model {name}"));
                }
                let glb = model["glb"].as_str().ok_or("missing shiny GLB path")?;
                Ok(ShinyVisualDefinition {
                    name: name.to_owned(),
                    glb: glb.to_owned(),
                    pickup_effect: row["m_iGetEffect"]
                        .as_i64()
                        .filter(|id| *id > 0)
                        .unwrap_or(397) as i32,
                })
            })();
            (index as i32, definition)
        })
        .collect()
}

#[derive(Component)]
struct ShinyVisual {
    shiny_type: i32,
    container: Entity,
    gltf: Handle<Gltf>,
    name: String,
}

#[derive(Component)]
struct ShinyVisualIssue {
    shiny_type: i32,
}

#[derive(Component)]
struct PendingShinyAnimation;

pub(super) fn register(app: &mut App) {
    app.add_systems(
        Update,
        (
            resolve
                .after(consume_network_entity_lifecycle_0104)
                .after(load_network_npc_visual_catalog_0104),
            animate.after(resolve),
            emit_idle_events
                .after(animate)
                .before(crate::tutorial_effects_runtime::process_tutorial_effect_runtime),
        ),
    );
}

fn resolve(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    catalog: Res<NetworkNpcVisualCatalogState0104>,
    shinies: Query<(
        Entity,
        &NetworkShiny0104,
        Option<&ShinyVisual>,
        Option<&ShinyVisualIssue>,
    )>,
) {
    let (Some(server), Some(catalog)) = (asset_server, catalog.catalog.as_ref()) else {
        return;
    };
    let mut remaining = 8;
    for (entity, shiny, current, issue) in &shinies {
        if current.is_some_and(|visual| visual.shiny_type == shiny.shiny_type)
            || issue.is_some_and(|issue| issue.shiny_type == shiny.shiny_type)
        {
            continue;
        }
        if remaining == 0 {
            break;
        }
        remaining -= 1;
        if let Some(current) = current {
            commands.entity(current.container).despawn();
            commands
                .entity(entity)
                .remove::<(ShinyVisual, ShinyPickupEffect)>();
        }
        let definition = match catalog.shinies.get(&shiny.shiny_type) {
            Some(Ok(definition)) => definition,
            problem => {
                warn!(
                    "cannot render shiny {} type {}: {:?}",
                    shiny.shiny_id, shiny.shiny_type, problem
                );
                commands.entity(entity).insert(ShinyVisualIssue {
                    shiny_type: shiny.shiny_type,
                });
                continue;
            }
        };
        let spawned = spawn_legacy_scene(
            &mut commands,
            entity,
            server.load(GltfAssetLabel::Scene(0).from_asset(definition.glb.clone())),
            definition.name.clone(),
            LegacyCharacterRootPolicy::Shiny,
            NativeSceneRole::WorldAuthored,
        );
        commands
            .entity(entity)
            .insert(ShinyVisual {
                shiny_type: shiny.shiny_type,
                container: spawned.visual_container,
                gltf: server.load(definition.glb.clone()),
                name: definition.name.clone(),
            })
            .insert(ShinyPickupEffect(definition.pickup_effect))
            .remove::<ShinyVisualIssue>();
    }
}

fn animate(
    mut commands: Commands,
    gltfs: Option<Res<Assets<Gltf>>>,
    mut graphs: Option<ResMut<Assets<AnimationGraph>>>,
    parents: Query<&ChildOf>,
    roots: Query<&ShinyVisual>,
    mut players: Query<
        (Entity, &mut AnimationPlayer),
        Or<(Added<AnimationPlayer>, With<PendingShinyAnimation>)>,
    >,
) {
    for (entity, mut player) in &mut players {
        let mut ancestor = entity;
        let visual = loop {
            if let Ok(visual) = roots.get(ancestor) {
                break Some(visual);
            }
            let Ok(parent) = parents.get(ancestor) else {
                break None;
            };
            ancestor = parent.parent();
        };
        let Some(visual) = visual else {
            continue;
        };
        let ready = gltfs.as_deref().and_then(|assets| assets.get(&visual.gltf));
        let (Some(gltf), Some(graphs)) = (ready, graphs.as_deref_mut()) else {
            commands.entity(entity).insert(PendingShinyAnimation);
            continue;
        };
        if let Some(clip) = gltf.named_animations.get("stand1") {
            let (graph, node) = AnimationGraph::from_clip(clip.clone());
            player.play(node).repeat();
            commands.entity(entity).insert((
                AnimationGraphHandle(graphs.add(graph)),
                ShinyEventCursor {
                    root: ancestor,
                    node,
                    name: visual.name.clone(),
                    seek: 0.0,
                    completions: 0,
                },
            ));
        } else {
            warn!("shiny type {} has no stand1 animation", visual.shiny_type);
        }
        commands.entity(entity).remove::<PendingShinyAnimation>();
    }
}

/// Resolved by the authoritative shiny table; pickup prediction waits for a
/// valid native visual definition.
#[derive(Component, Clone, Copy)]
pub struct ShinyPickupEffect(pub i32);

#[derive(Component)]
struct ShinyEventCursor {
    root: Entity,
    node: bevy::animation::graph::AnimationNodeIndex,
    name: String,
    seek: f32,
    completions: u32,
}

fn emit_idle_events(
    mut players: Query<(&AnimationPlayer, &mut ShinyEventCursor)>,
    mut runtime: Option<ResMut<crate::tutorial_effects_runtime::TutorialEffectRuntime>>,
) {
    use crate::tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntimeCommand, animation_event_crossings,
    };
    for (player, mut cursor) in &mut players {
        let Some(active) = player.animation(cursor.node) else {
            continue;
        };
        let effect_id = match cursor.name.as_str() {
            "shineni_Item" => 398,
            "shineni_buff" => 401,
            _ => continue,
        };
        let crossings = animation_event_crossings(
            cursor.seek,
            cursor.completions,
            active.seek_time(),
            active.completions(),
            active.repeat_mode(),
            0.1,
        );
        if cursor.seek != active.seek_time() || cursor.completions != active.completions() {
            cursor.seek = active.seek_time();
            cursor.completions = active.completions();
        }
        let Some(runtime) = runtime.as_deref_mut() else {
            continue;
        };
        for _ in 0..crossings {
            runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id,
                placement: TutorialEffectPlacement::ExactEntityBone {
                    root_entity: cursor.root,
                    node_name: cursor.name.clone(),
                    spawn_world_rotation: Quat::IDENTITY,
                    local_translation_after_parenting: Vec3::ZERO,
                    local_rotation_after_parenting: Quat::IDENTITY,
                },
                scale: 1.0,
                tracked: false,
                name: Some(format!("shiny-idle-{:?}-{effect_id}", cursor.root)),
                destroy_after_seconds: None,
                source_line: line!(),
            });
        }
    }
}

#[cfg(test)]
mod tests;
