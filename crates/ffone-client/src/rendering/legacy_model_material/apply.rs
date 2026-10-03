//! Discovery and application of native materials, water, static caches and renderer order.

use super::exact_mip_cache::{ExactMipChainApplied, ExactMipChainCache};
use super::expected_passes::LegacyMaterialMetadataError;
use super::extras_parse::{LegacyMaterialExtrasParseCache, ParsedLegacyMaterialExtras};
use super::material_uniforms::{LegacyOutlineCompanion, LegacyOutlineSource};
use super::metadata::{PendingLegacyModelMaterial, PendingLegacyWaterMaterial};
use super::params::LegacyModelMaterialParams;
use super::render_plan::LegacyPassKind;
use super::shared_assets::ModelMaterialSharing;
use super::shared_assets::StaticAssetSharing;
use super::sort_order::{
    LegacyMaterialPassCompanion, LegacyMaterialRendererOrder, LegacyMaterialSortOrderApplied,
    legacy_material_sort_bias,
};
use super::static_material_cache::LegacyStaticMaterialCache;
use super::sync::try_insert_streamed_entity;
use super::texture_resolve::resolve_gltf_textures;
use super::{ExactMipPngBytes, LegacyModelMaterial, LegacyOutlineMaterial, LegacyWaterMaterial};
use crate::legacy_environment::LegacyWaterSurface;
use bevy::{
    camera::visibility::{NoFrustumCulling, VisibilityRange},
    gltf::{GltfMaterialExtras, GltfMaterialName, GltfMeshExtras},
    mesh::{MeshTag, skinning::SkinnedMesh},
    prelude::*,
    render::render_resource::TextureFormat,
};
use serde_json::Value;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyMaterialApplied;

pub(super) fn discover_legacy_material_extras(
    mut commands: Commands,
    mut cache: ResMut<LegacyMaterialExtrasParseCache>,
    query: Query<
        (Entity, &GltfMaterialExtras, Option<&GltfMaterialName>),
        Added<GltfMaterialExtras>,
    >,
) {
    for (entity, extras, material_name) in &query {
        let material_name = material_name.map(|name| name.0.as_str());
        match cache.parse(material_name, &extras.value) {
            ParsedLegacyMaterialExtras::Model(pending) => {
                try_insert_streamed_entity(&mut commands, entity, pending);
            }
            ParsedLegacyMaterialExtras::Water(pending) => {
                try_insert_streamed_entity(&mut commands, entity, pending);
            }
            ParsedLegacyMaterialExtras::StaticWorld(pending) => {
                try_insert_streamed_entity(&mut commands, entity, pending);
            }
            ParsedLegacyMaterialExtras::Error(error) => {
                try_insert_streamed_entity(
                    &mut commands,
                    entity,
                    LegacyMaterialMetadataError(error),
                );
            }
        }
    }
}

pub(super) fn discover_legacy_material_renderer_order(
    mut commands: Commands,
    query: Query<(Entity, &GltfMeshExtras), Added<GltfMeshExtras>>,
) {
    for (entity, extras) in &query {
        match renderer_index_from_mesh_extras(&extras.value) {
            Ok(Some(renderer_index)) => {
                try_insert_streamed_entity(
                    &mut commands,
                    entity,
                    LegacyMaterialRendererOrder { renderer_index },
                );
            }
            Ok(None) => {}
            Err(error) => {
                try_insert_streamed_entity(
                    &mut commands,
                    entity,
                    LegacyMaterialMetadataError(error),
                );
            }
        }
    }
}

pub(super) fn renderer_index_from_mesh_extras(extras: &str) -> Result<Option<u16>, String> {
    let root: Value = serde_json::from_str(extras)
        .map_err(|error| format!("invalid glTF mesh extras: {error}"))?;
    let Some(ffone) = root.get("ffone") else {
        return Ok(None);
    };
    let ffone = ffone
        .as_object()
        .ok_or_else(|| "glTF mesh extras.ffone is not an object".to_owned())?;
    let renderer_index = ffone
        .get("rendererIndex")
        .and_then(Value::as_u64)
        .ok_or_else(|| "glTF mesh extras.ffone.rendererIndex is missing or invalid".to_owned())?;
    let renderer_index = u16::try_from(renderer_index).map_err(|_| {
        format!("glTF mesh renderer index {renderer_index} does not fit the native order contract")
    })?;
    Ok(Some(renderer_index))
}

pub(super) fn apply_legacy_water_materials(
    mut commands: Commands,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<LegacyWaterMaterial>>,
    query: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            &PendingLegacyWaterMaterial,
        ),
        (
            Without<LegacyMaterialApplied>,
            Without<LegacyMaterialMetadataError>,
        ),
    >,
) {
    for (entity, standard_handle, pending) in &query {
        let Some(standard_material) = standard_materials.get(&standard_handle.0) else {
            continue;
        };
        let (Some(reflective_gradient), Some(bump_map), Some(fresnel_map)) = (
            standard_material.base_color_texture.clone(),
            standard_material.normal_map_texture.clone(),
            standard_material.emissive_texture.clone(),
        ) else {
            commands.entity(entity).insert(LegacyMaterialMetadataError(
                format!(
                    "ffWater material {} ({}) did not load its reflective, bump and Fresnel handles",
                    pending.true_name, pending.source_material_id
                ),
            ));
            continue;
        };
        let handle = water_materials.add(LegacyWaterMaterial {
            uniform: pending.uniform,
            reflective_gradient: Some(reflective_gradient),
            bump_map: Some(bump_map),
            fresnel_map: Some(fresnel_map),
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                MeshMaterial3d(handle),
                LegacyWaterSurface {
                    infected: pending.true_name == "ffPoison",
                },
                LegacyMaterialApplied,
                ExactMipChainApplied::default(),
            ));
    }
}

pub(super) fn apply_cached_legacy_static_material(
    commands: &mut Commands,
    entity: Entity,
    mesh: &Mesh3d,
    passes: &[(LegacyPassKind, Handle<LegacyModelMaterial>)],
    mip_proof: ExactMipChainApplied,
    range_tag: Option<&MeshTag>,
    range: Option<&VisibilityRange>,
    visibility: Option<&Visibility>,
) {
    let Some(((_, source_material), companion_passes)) = passes.split_first() else {
        return;
    };
    commands
        .entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert((
            mesh.clone(),
            MeshMaterial3d(source_material.clone()),
            LegacyMaterialApplied,
            mip_proof,
        ));
    for (pass, material) in companion_passes {
        let mut companion = commands.spawn((
            Mesh3d(mesh.0.clone()),
            MeshMaterial3d(material.clone()),
            Transform::IDENTITY,
            Visibility::Inherited,
            LegacyMaterialPassCompanion {
                source_mesh_entity: entity,
                pass: *pass,
            },
        ));
        if let Some(tag) = range_tag {
            companion.insert(tag.clone());
        }
        if let Some(range) = range {
            companion.insert(range.clone());
        }
        if let Some(visibility) = visibility {
            companion.insert(*visibility);
        }
        let companion = companion.id();
        commands.entity(entity).add_child(companion);
    }
}

pub(super) fn apply_static_world_base_color_fallback(
    params: &mut LegacyModelMaterialParams,
    standard_base_color: LinearRgba,
    has_exact_base_color: bool,
) {
    if !has_exact_base_color {
        params.base_color = standard_base_color;
    }
}

pub(super) fn image_is_fully_opaque(image: &Image) -> Option<bool> {
    let data = image.data.as_deref()?;
    match image.texture_descriptor.format {
        TextureFormat::Rgba8Unorm
        | TextureFormat::Rgba8UnormSrgb
        | TextureFormat::Bgra8Unorm
        | TextureFormat::Bgra8UnormSrgb => {
            Some(data.chunks_exact(4).all(|pixel| pixel[3] == u8::MAX))
        }
        // These formats have no alpha channel.
        TextureFormat::R8Unorm
        | TextureFormat::R8Snorm
        | TextureFormat::R8Uint
        | TextureFormat::R8Sint
        | TextureFormat::Rg8Unorm
        | TextureFormat::Rg8Snorm
        | TextureFormat::Rg8Uint
        | TextureFormat::Rg8Sint => Some(true),
        _ => None,
    }
}

pub(super) fn apply_legacy_model_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    png_bytes: Res<Assets<ExactMipPngBytes>>,
    mut mip_cache: ResMut<ExactMipChainCache>,
    mut surface_materials: ResMut<Assets<LegacyModelMaterial>>,
    mut outline_materials: ResMut<Assets<LegacyOutlineMaterial>>,
    query: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &PendingLegacyModelMaterial,
            Option<&SkinnedMesh>,
            Option<&MeshTag>,
            Option<&VisibilityRange>,
            Option<&Visibility>,
        ),
        (
            Without<LegacyMaterialApplied>,
            Without<LegacyMaterialMetadataError>,
        ),
    >,
) {
    for (entity, mesh, standard_handle, pending, skin, range_tag, range, visibility) in &query {
        let result = resolve_gltf_textures(
            &asset_server,
            &standard_materials,
            &mut images,
            &png_bytes,
            &mut mip_cache,
            mesh,
            standard_handle,
            pending,
        );
        let resolved = match result {
            Ok(Some(resolved)) if resolved.textures.is_complete_for(pending.params.shader) => {
                resolved
            }
            Ok(Some(_)) => {
                mip_cache.pending_materials.remove(&standard_handle.0.id());
                commands
                    .entity(entity)
                    .insert(LegacyMaterialMetadataError(format!(
                        "material {} lacks required exact texture bindings for {}",
                        pending.true_name,
                        pending.params.shader.exact_name()
                    )));
                continue;
            }
            Err(error) => {
                mip_cache.pending_materials.remove(&standard_handle.0.id());
                commands
                    .entity(entity)
                    .insert(LegacyMaterialMetadataError(error));
                continue;
            }
            Ok(None) => continue,
        };

        let mut assigned_source = false;
        for pass in pending.params.render_plan().passes {
            if pass.kind == LegacyPassKind::Outline {
                let Some(material) = pending.params.outline_material() else {
                    continue;
                };
                let handle = outline_materials.add(material);
                let mut companion = commands.spawn((
                    Mesh3d(mesh.0.clone()),
                    MeshMaterial3d(handle),
                    Transform::IDENTITY,
                    Visibility::Inherited,
                    LegacyOutlineCompanion {
                        source_mesh_entity: entity,
                    },
                    LegacyMaterialPassCompanion {
                        source_mesh_entity: entity,
                        pass: pass.kind,
                    },
                ));
                if let Some(skin) = skin {
                    companion.insert((skin.clone(), NoFrustumCulling));
                }
                let companion = companion.id();
                // Static-world adaptive distance is owned by the complete
                // logical object, not by an individual material pass. A pass
                // can be spawned after the world's propagation system has
                // already run in this Update, so inherit the source contract
                // immediately instead of flashing an unbounded outline for a
                // frame. If aggregation is still pending, the ordinary
                // Added<Mesh3d> retry path will bind this companion later.
                let mut companion_commands = commands.entity(companion);
                if let Some(tag) = range_tag {
                    companion_commands.insert(tag.clone());
                }
                if let Some(range) = range {
                    companion_commands.insert(range.clone());
                }
                if let Some(visibility) = visibility {
                    companion_commands.insert(*visibility);
                }
                commands
                    .entity(entity)
                    .add_child(companion)
                    .insert(LegacyOutlineSource {
                        width: pending.params.outline_width,
                        color: pending.params.outline_color,
                    });
                continue;
            }

            let Some(material) = pending.params.material_for_pass(pass, &resolved.textures) else {
                continue;
            };
            let handle = surface_materials.add(material);
            if !assigned_source {
                commands
                    .entity(entity)
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .insert(MeshMaterial3d(handle));
                assigned_source = true;
            } else {
                let mut companion = commands.spawn((
                    Mesh3d(mesh.0.clone()),
                    MeshMaterial3d(handle),
                    Transform::IDENTITY,
                    Visibility::Inherited,
                    LegacyMaterialPassCompanion {
                        source_mesh_entity: entity,
                        pass: pass.kind,
                    },
                ));
                if let Some(skin) = skin {
                    companion.insert((skin.clone(), NoFrustumCulling));
                }
                let companion = companion.id();
                let mut companion_commands = commands.entity(companion);
                if let Some(tag) = range_tag {
                    companion_commands.insert(tag.clone());
                }
                if let Some(range) = range {
                    companion_commands.insert(range.clone());
                }
                if let Some(visibility) = visibility {
                    companion_commands.insert(*visibility);
                }
                commands.entity(entity).add_child(companion);
            }
        }
        if assigned_source {
            commands
                .entity(entity)
                .insert((LegacyMaterialApplied, resolved.mip_marker));
        }
        mip_cache.pending_materials.remove(&standard_handle.0.id());
    }
}

pub(super) fn apply_legacy_material_renderer_order(
    mut commands: Commands,
    mut cache: Option<ResMut<LegacyStaticMaterialCache>>,
    sharing: Option<Res<StaticAssetSharing>>,
    model_sharing: Option<Res<ModelMaterialSharing>>,
    mut surface_materials: ResMut<Assets<LegacyModelMaterial>>,
    mut outline_materials: ResMut<Assets<LegacyOutlineMaterial>>,
    mut material_handles: Query<&mut MeshMaterial3d<LegacyModelMaterial>>,
    sources: Query<
        (
            Entity,
            &LegacyMaterialRendererOrder,
            &PendingLegacyModelMaterial,
            Option<&LegacyMaterialSortOrderApplied>,
        ),
        Or<(
            Without<LegacyMaterialSortOrderApplied>,
            Changed<LegacyMaterialRendererOrder>,
        )>,
    >,
    surface_companions: Query<(Entity, &LegacyMaterialPassCompanion)>,
    outline_companions: Query<(
        &LegacyMaterialPassCompanion,
        &MeshMaterial3d<LegacyOutlineMaterial>,
    )>,
) {
    for (source_entity, renderer_order, pending, applied) in &sources {
        if let Some(applied) = applied {
            if applied.renderer_index == renderer_order.renderer_index {
                continue;
            }
            // A modular rebind can replace the local glTF ordinal with the
            // flattened actor ordinal. The old proof and all pass biases are
            // then stale. Invalidate even if a late companion is not ready:
            // Without<...> must keep retrying after Changed<...> expires.
            commands
                .entity(source_entity)
                .queue_silenced(|mut entity: EntityWorldMut| {
                    entity.remove::<LegacyMaterialSortOrderApplied>();
                });
        }
        let Ok(source_handle) = material_handles.get(source_entity) else {
            continue;
        };
        let source_handle = source_handle.0.clone();
        let plan = pending.params.render_plan();
        let Some(source_pass_ordinal) = plan
            .passes
            .iter()
            .position(|pass| pass.kind != LegacyPassKind::Outline)
        else {
            continue;
        };
        let Some(source_bias) = legacy_material_sort_bias(
            pending.source_render_queue,
            renderer_order.renderer_index,
            source_pass_ordinal,
        ) else {
            continue;
        };
        if surface_materials.get(&source_handle).is_none() {
            continue;
        }

        let mut ordered_surface_companions = Vec::new();
        let mut ordered_outline_companions = Vec::new();
        let mut complete = true;
        for (pass_ordinal, pass) in plan.passes.iter().enumerate() {
            if pass_ordinal == source_pass_ordinal {
                continue;
            }
            let Some(sort_bias) = legacy_material_sort_bias(
                pending.source_render_queue,
                renderer_order.renderer_index,
                pass_ordinal,
            ) else {
                complete = false;
                break;
            };
            match pass.kind {
                LegacyPassKind::Outline => {
                    let matches = outline_companions
                        .iter()
                        .filter(|(companion, _)| {
                            companion.source_mesh_entity == source_entity
                                && companion.pass == pass.kind
                        })
                        .map(|(_, handle)| handle.0.clone())
                        .collect::<Vec<_>>();
                    let [handle] = matches.as_slice() else {
                        complete = false;
                        break;
                    };
                    if outline_materials.get(handle).is_none() {
                        complete = false;
                        break;
                    }
                    ordered_outline_companions.push((handle.clone(), sort_bias));
                }
                _ => {
                    let matches = surface_companions
                        .iter()
                        .filter(|(_, companion)| {
                            companion.source_mesh_entity == source_entity
                                && companion.pass == pass.kind
                        })
                        .filter_map(|(entity, _)| {
                            material_handles
                                .get(entity)
                                .ok()
                                .map(|handle| (entity, handle.0.clone()))
                        })
                        .collect::<Vec<_>>();
                    let [(entity, handle)] = matches.as_slice() else {
                        complete = false;
                        break;
                    };
                    if surface_materials.get(handle).is_none() {
                        complete = false;
                        break;
                    }
                    ordered_surface_companions.push((*entity, handle.clone(), sort_bias));
                }
            }
        }
        if !complete {
            continue;
        }

        let share = sharing.as_ref().is_none_or(|sharing| sharing.enabled)
            && model_sharing.as_ref().is_none_or(|sharing| sharing.0);
        ordered_surface_companions.insert(0, (source_entity, source_handle.clone(), source_bias));
        for (entity, handle, sort_bias) in ordered_surface_companions {
            if share
                && surface_materials
                    .get(&handle)
                    .is_some_and(|material| !material.is_instance_private())
                && let Some(cache) = cache.as_deref_mut()
            {
                if let Some(mut material) = surface_materials.get(&handle).cloned() {
                    material.sort_bias = sort_bias;
                    let shared = cache.admit_immutable(material, &mut surface_materials);
                    if let Ok(mut bound) = material_handles.get_mut(entity) {
                        if bound.0 != shared {
                            bound.0 = shared;
                        }
                    }
                }
            } else if let Some(mut material) = surface_materials.get_mut(&handle) {
                material.sort_bias = sort_bias;
            }
        }
        for (handle, sort_bias) in ordered_outline_companions {
            if let Some(mut material) = outline_materials.get_mut(&handle) {
                material.sort_bias = sort_bias;
            }
        }
        commands
            .entity(source_entity)
            .insert(LegacyMaterialSortOrderApplied {
                renderer_index: renderer_order.renderer_index,
                pass_count: u8::try_from(plan.passes.len()).unwrap_or(u8::MAX),
            });
    }
}
