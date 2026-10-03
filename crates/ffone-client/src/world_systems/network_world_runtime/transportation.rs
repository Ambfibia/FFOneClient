//! Exact visual, animation and collision presentation for server-owned Slider transport.
//!
//! Route sampling stays server-owned: this module only presents protocol-0104
//! appearance/move packets, matching the clean `cnBusContainer`/`cnBusMoveController` split.

use bevy::{
    animation::RepeatAnimation,
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use ffone_skinned_model::{NativeSampler, SamplerMagFilter, SamplerMinFilter, SamplerWrapMode};

use super::{NetworkTransportation0104, spawn_pending_authored_model_collider};
use crate::{
    character_scene::spawn_legacy_transportation_scene,
    legacy_model_material::{
        LegacyMaterialApplied, LegacyModelMaterial, LegacyNpcTextureRole,
        PendingLegacyModelMaterial, load_legacy_main_texture_replacement_with_sampler,
    },
};

const SLIDER_TRANSPORTATION_TYPE: i32 = 1;
const SLIDER_LOGICAL_NAME: &str = "DT_ETC_Downtownbus_A_00";
const SLIDER_GLTF: &str = "characters/transportation/downtown_bus/DT_ETC_Downtownbus_A_00.glb";
const SLIDER_COLLISION_GLTF: &str =
    "characters/transportation/downtown_bus/DT_ETC_Downtownbus_A_00.collision.glb";
const SLIDER_COLLISION_NODE: &str = "collision";
const SLIDER_STAND_ANIMATION: &str = "stand1";
const SLIDER_MAIN_TEXTURE: &str = "characters/transportation/downtown_bus/DT_ETC_DowntownBus01.png";
const SLIDER_SUB_TEXTURE: &str = "characters/transportation/downtown_bus/DT_ETC_DowntownBus05.png";
#[cfg(test)]
const SLIDER_MAIN_MATERIAL: &str =
    "dt_etc_downtownbus_a_00-01 - default-dt_etc_downtownbus_a_01.dds";
#[cfg(test)]
const SLIDER_SUB_MATERIAL: &str =
    "dt_etc_downtownbus_a_00-02 - default-dt_etc_downtownbus_a_02.dds";
const SLIDER_COLLISION_VERTEX_COUNT: usize = 134;
const SLIDER_COLLISION_INDEX_COUNT: usize = 531;
const MAX_TRANSPORTATION_VISUAL_SPAWNS_PER_FRAME: usize = 2;

pub(super) fn register_network_transportation_visual_runtime_0104(app: &mut App) {
    app.add_systems(
        Update,
        (
            resolve_network_transportation_visuals_0104,
            bind_slider_texture_variants_0104.after(resolve_network_transportation_visuals_0104),
            play_network_transportation_animation_0104
                .after(resolve_network_transportation_visuals_0104),
        ),
    );
}

#[derive(Clone, Debug, Component)]
pub struct NetworkTransportationVisual0104 {
    pub transportation_type: i32,
    pub visual_container: Entity,
    gltf: Handle<Gltf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub struct NetworkTransportationVisualIssue0104 {
    pub transportation_type: i32,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Component)]
struct SliderTextureVariantBound0104;

/// A Slider animation player discovered before its glTF animation table has
/// entered `Assets<Gltf>`. Only these transport players are retried; unrelated
/// world/character animation players are still visited once through `Added`.
#[derive(Clone, Copy, Debug, Component)]
struct PendingSliderAnimation0104;

fn slider_texture_sampler(true_name: &str) -> NativeSampler {
    NativeSampler {
        name: true_name.to_owned(),
        mag_filter: SamplerMagFilter::Linear,
        min_filter: SamplerMinFilter::Linear,
        wrap_s: SamplerWrapMode::Repeat,
        wrap_t: SamplerWrapMode::Repeat,
        legacy_filter_mode: 1,
        legacy_wrap_mode: 0,
        anisotropy_level: 1,
        mip_map_bias: 0.0,
    }
}

/// Exact `cnBusMoveController.SetupBus` table-texture selection.
///
/// Loading the table textures does not imply assignment: Unity checks the
/// case-sensitive material name for `main`/`sub`. The shipped bus has neither,
/// so its authored atlases (and their complete mip chains) must remain bound.
/// Mapping its `-01`/`-02` materials to table slots substitutes incompatible UV
/// atlases. `main` wins when both substrings occur, matching the source writes.
fn slider_texture_role(true_name: &str) -> Option<LegacyNpcTextureRole> {
    if true_name.contains("main") {
        Some(LegacyNpcTextureRole::Main)
    } else if true_name.contains("sub") {
        Some(LegacyNpcTextureRole::Sub)
    } else {
        None
    }
}

fn bind_slider_texture_variants_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
    parents: Query<&ChildOf>,
    roots: Query<(Entity, &NetworkTransportationVisual0104)>,
    surfaces: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            &PendingLegacyModelMaterial,
        ),
        (
            Added<LegacyMaterialApplied>,
            Without<SliderTextureVariantBound0104>,
        ),
    >,
) {
    let (Some(asset_server), Some(mut materials)) = (asset_server, materials) else {
        return;
    };
    for (entity, material_handle, pending) in &surfaces {
        let Some((root, visual)) = transportation_visual_ancestor(entity, &parents, &roots) else {
            continue;
        };
        let selected = match slider_texture_role(&pending.true_name) {
            Some(LegacyNpcTextureRole::Main) => Some(("DT_ETC_DowntownBus01", SLIDER_MAIN_TEXTURE)),
            Some(LegacyNpcTextureRole::Sub) => Some(("DT_ETC_DowntownBus05", SLIDER_SUB_TEXTURE)),
            None => None,
        };
        let Some((true_name, path)) = selected else {
            commands
                .entity(entity)
                .insert(SliderTextureVariantBound0104);
            continue;
        };
        let replacement = match load_legacy_main_texture_replacement_with_sampler(
            &asset_server,
            pending,
            path,
            &slider_texture_sampler(true_name),
        ) {
            Ok(replacement) => replacement,
            Err(error) => {
                commands
                    .entity(root)
                    .insert(NetworkTransportationVisualIssue0104 {
                        transportation_type: visual.transportation_type,
                        detail: format!(
                            "cannot bind primary Slider texture {true_name:?} to material {:?}: {error}",
                            pending.true_name
                        ),
                    });
                commands
                    .entity(entity)
                    .insert(SliderTextureVariantBound0104);
                continue;
            }
        };
        let Some(mut material) = materials.get(&material_handle.0).cloned() else {
            continue;
        };
        material.base_texture = Some(replacement);
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(material)),
            SliderTextureVariantBound0104,
        ));
    }
}

#[derive(Clone, Copy, Debug, Component)]
struct PendingSliderCollision0104 {
    gameplay_root: Entity,
    transportation_type: i32,
}

fn resolve_network_transportation_visuals_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    transports: Query<(
        Entity,
        &NetworkTransportation0104,
        Option<&NetworkTransportationVisual0104>,
        Option<&NetworkTransportationVisualIssue0104>,
    )>,
) {
    let Some(asset_server) = asset_server else {
        return;
    };
    let mut remaining = MAX_TRANSPORTATION_VISUAL_SPAWNS_PER_FRAME;
    for (root, transportation, current, issue) in &transports {
        if transportation.transportation_type != SLIDER_TRANSPORTATION_TYPE {
            if let Some(current) = current {
                commands.entity(current.visual_container).despawn();
                commands
                    .entity(root)
                    .remove::<NetworkTransportationVisual0104>();
            }
            if issue
                .is_none_or(|issue| issue.transportation_type != transportation.transportation_type)
            {
                commands
                    .entity(root)
                    .insert(NetworkTransportationVisualIssue0104 {
                        transportation_type: transportation.transportation_type,
                        detail: format!(
                            "no primary visual contract for transportation type {}",
                            transportation.transportation_type
                        ),
                    });
            }
            continue;
        }
        if current
            .is_some_and(|visual| visual.transportation_type == transportation.transportation_type)
        {
            continue;
        }
        if remaining == 0 {
            break;
        }
        remaining -= 1;
        if let Some(current) = current {
            commands.entity(current.visual_container).despawn();
        }

        let gltf: Handle<Gltf> = asset_server.load(SLIDER_GLTF);
        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(SLIDER_GLTF));
        let spawned =
            spawn_legacy_transportation_scene(&mut commands, root, scene, SLIDER_LOGICAL_NAME);
        commands
            .entity(spawned.visual_container)
            .insert(Name::new(format!(
                "network Slider {} visual",
                transportation.id
            )));
        commands
            .entity(spawned.scene)
            .insert((
                Name::new(format!(
                    "network Slider {} normalized Scene0",
                    transportation.id
                )),
                PendingSliderCollision0104 {
                    gameplay_root: root,
                    transportation_type: transportation.transportation_type,
                },
            ))
            .observe(materialize_slider_collision_0104);
        commands
            .entity(root)
            .insert(NetworkTransportationVisual0104 {
                transportation_type: transportation.transportation_type,
                visual_container: spawned.visual_container,
                gltf,
            });
        commands
            .entity(root)
            .remove::<NetworkTransportationVisualIssue0104>();
    }
}

fn materialize_slider_collision_0104(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pending: Query<&PendingSliderCollision0104>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    let scene = event.event().entity;
    let Ok(pending) = pending.get(scene) else {
        return;
    };
    let mut stack = children
        .get(scene)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut matches = Vec::new();
    while let Some(entity) = stack.pop() {
        if names
            .get(entity)
            .is_ok_and(|name| name.as_str() == SLIDER_COLLISION_NODE)
        {
            matches.push(entity);
        }
        if let Ok(descendants) = children.get(entity) {
            stack.extend(descendants.iter());
        }
    }
    let [collision_node] = matches.as_slice() else {
        commands
            .entity(pending.gameplay_root)
            .insert(NetworkTransportationVisualIssue0104 {
                transportation_type: pending.transportation_type,
                detail: format!(
                    "primary Slider expected exactly one collision node, found {}",
                    matches.len()
                ),
            });
        return;
    };
    spawn_pending_authored_model_collider(
        &mut commands,
        &asset_server,
        *collision_node,
        "primary Slider MeshCollider",
        SLIDER_COLLISION_GLTF,
        0,
        0,
        Transform::IDENTITY,
        SLIDER_COLLISION_VERTEX_COUNT,
        SLIDER_COLLISION_INDEX_COUNT,
    );
    commands
        .entity(scene)
        .remove::<PendingSliderCollision0104>();
}

fn play_network_transportation_animation_0104(
    mut commands: Commands,
    gltfs: Option<Res<Assets<Gltf>>>,
    graphs: Option<ResMut<Assets<AnimationGraph>>>,
    parents: Query<&ChildOf>,
    roots: Query<(Entity, &NetworkTransportationVisual0104)>,
    mut players: Query<
        (
            Entity,
            &mut AnimationPlayer,
            Option<&PendingSliderAnimation0104>,
        ),
        Or<(Added<AnimationPlayer>, With<PendingSliderAnimation0104>)>,
    >,
) {
    let mut graphs = graphs;
    for (entity, mut player, pending) in &mut players {
        let Some((root, visual)) = transportation_visual_ancestor(entity, &parents, &roots) else {
            continue;
        };
        if pending.is_none() {
            commands.entity(entity).insert(PendingSliderAnimation0104);
        }
        let (Some(gltfs), Some(graphs)) = (gltfs.as_deref(), graphs.as_deref_mut()) else {
            continue;
        };
        let Some(gltf) = gltfs.get(&visual.gltf) else {
            continue;
        };
        let Some(clip) = gltf.named_animations.get(SLIDER_STAND_ANIMATION).cloned() else {
            commands
                .entity(root)
                .insert(NetworkTransportationVisualIssue0104 {
                    transportation_type: visual.transportation_type,
                    detail: format!(
                        "primary Slider GLB has no {:?} animation",
                        SLIDER_STAND_ANIMATION
                    ),
                });
            commands
                .entity(entity)
                .remove::<PendingSliderAnimation0104>();
            continue;
        };
        let (graph, node) = AnimationGraph::from_clip(clip);
        player.stop_all();
        player.start(node).set_repeat(RepeatAnimation::Forever);
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(graphs.add(graph)))
            .remove::<PendingSliderAnimation0104>();
    }
}

fn transportation_visual_ancestor<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    roots: &'a Query<(Entity, &NetworkTransportationVisual0104)>,
) -> Option<(Entity, &'a NetworkTransportationVisual0104)> {
    loop {
        if let Ok((root, visual)) = roots.get(entity) {
            return Some((root, visual));
        }
        entity = parents.get(entity).ok()?.parent();
    }
}

#[cfg(test)]
mod tests;
