//! Admission of immutable static-world materials and shared render assets.
use super::{
    CachedLegacyStaticMaterial, ExactMipChainApplied, ExactMipChainCache, ExactMipPngBytes,
    LegacyBlendMode, LegacyMaterialApplied, LegacyMaterialMetadataError, LegacyModelMaterial,
    LegacyModelTextures, LegacyStaticMaterialCache, LegacyStaticMaterialCacheKey,
    PendingLegacyStaticWorldMaterial, StaticAssetSharing, apply_cached_legacy_static_material,
    apply_static_world_base_color_fallback, image_is_fully_opaque, legacy_static_world_sort_bias,
    resolve_exact_mip_chain, static_exact_mips::LegacyStaticExactMipManifest,
};
use bevy::{camera::visibility::VisibilityRange, mesh::MeshTag, prelude::*};

pub(super) fn apply_legacy_static_world_materials(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    png_bytes: Option<Res<Assets<ExactMipPngBytes>>>,
    mut mip_cache: Option<ResMut<ExactMipChainCache>>,
    static_exact_mips: Option<Res<LegacyStaticExactMipManifest>>,
    mut material_cache: Option<ResMut<LegacyStaticMaterialCache>>,
    meshes: Option<Res<Assets<Mesh>>>,
    sharing: Option<Res<StaticAssetSharing>>,
    mut surface_materials: ResMut<Assets<LegacyModelMaterial>>,
    query: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &PendingLegacyStaticWorldMaterial,
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
    let share_resources = sharing.is_none_or(|sharing| sharing.enabled);
    for (entity, mesh, standard_handle, pending, range_tag, range, visibility) in &query {
        let cache_key = mesh.0.path().map(|path| LegacyStaticMaterialCacheKey {
            model_source: path.without_label().clone_owned(),
            standard_material: standard_handle.0.id(),
        });
        // Resolve the source/path key before replacing the render mesh. Its
        // placement, bounds, metadata, colliders and entity remain independent.
        let shared_mesh = material_cache
            .as_deref_mut()
            .zip(meshes.as_deref())
            .filter(|_| share_resources)
            .map(|(cache, meshes)| Mesh3d(cache.shared.meshes.intern(&mesh.0, meshes)))
            .unwrap_or_else(|| mesh.clone());
        let mesh = &shared_mesh;
        if let Some(cached) = cache_key.as_ref().and_then(|key| {
            material_cache
                .as_deref()
                .and_then(|cache| cache.entries.get(key))
        }) {
            apply_cached_legacy_static_material(
                &mut commands,
                entity,
                mesh,
                &cached.passes,
                cached.mip_proof,
                range_tag,
                range,
                visibility,
            );
            continue;
        }
        let Some(standard_material) = standard_materials.get(&standard_handle.0) else {
            continue;
        };
        let mut base_texture = standard_material.base_color_texture.clone();
        let mut mip_proof = ExactMipChainApplied::default();
        if let (Some(texture_id), Some(manifest)) = (
            pending.base_texture_id.as_deref(),
            static_exact_mips.as_deref(),
        ) && let Some(binding) = manifest.bindings.get(texture_id)
        {
            let (Some(asset_server), Some(png_bytes), Some(mip_cache)) = (
                asset_server.as_deref(),
                png_bytes.as_deref(),
                mip_cache.as_deref_mut(),
            ) else {
                commands
                    .entity(entity)
                    .insert(LegacyMaterialMetadataError(format!(
                        "legacy static material {} ({}) cannot load its published exact mip chain: runtime mip services are unavailable",
                        pending.true_name, pending.source_material_id
                    )));
                continue;
            };
            match resolve_exact_mip_chain(
                asset_server,
                &mut images,
                png_bytes,
                mip_cache,
                &manifest.source,
                binding,
                standard_material.base_color_texture.as_ref(),
                standard_handle.0.id(),
            ) {
                Ok(Some(handle)) => {
                    base_texture = Some(handle);
                    mip_proof = ExactMipChainApplied {
                        assigned_texture_bindings: 1,
                        mip_chains_applied: 1,
                        mip_levels: u32::try_from(
                            binding
                                .mip_levels
                                .as_ref()
                                .expect("published binding is validated")
                                .len(),
                        )
                        .expect("published mip count fits u32"),
                    };
                }
                Ok(None) => continue,
                Err(error) => {
                    commands
                        .entity(entity)
                        .insert(LegacyMaterialMetadataError(format!(
                            "legacy static material {} ({}) exact mip chain failed validation: {error}",
                            pending.true_name, pending.source_material_id
                        )));
                    continue;
                }
            }
        }
        if share_resources
            && let (Some(handle), Some(cache)) =
                (base_texture.as_ref(), material_cache.as_deref_mut())
        {
            base_texture = Some(cache.shared.images.intern(handle, &images));
        }
        let texture_is_fully_opaque = match base_texture.as_ref() {
            Some(handle) => {
                let Some(image) = images.get(handle) else {
                    continue;
                };
                material_cache
                    .as_deref_mut()
                    .map(|cache| cache.shared.opaque(handle, image))
                    .unwrap_or_else(|| image_is_fully_opaque(image).unwrap_or(false))
            }
            None => true,
        };

        let mut params = pending.params.clone();
        let standard_base_color = standard_material.base_color.to_linear();
        apply_static_world_base_color_fallback(
            &mut params,
            standard_base_color,
            pending.has_exact_base_color,
        );
        let base_color = params.base_color;
        if !pending.has_exact_ambient_color {
            params.ambient_color = base_color;
        }
        if !pending.has_exact_emission {
            // In the audited tutorial static set, 141/209 records use
            // `_AmbColor == _Color` and `_Emission == _Color * 0.8` exactly;
            // this is the source-build convention omitted by old GLB extras.
            params.emission = LinearRgba::new(
                base_color.red * 0.8,
                base_color.green * 0.8,
                base_color.blue * 0.8,
                1.0,
            );
        }

        let mut glow_mask = pending
            .has_glow_mask
            .then(|| standard_material.emissive_texture.clone())
            .flatten();
        if share_resources
            && let (Some(handle), Some(cache)) = (glow_mask.as_ref(), material_cache.as_deref_mut())
        {
            glow_mask = Some(cache.shared.images.intern(handle, &images));
        }
        if pending.has_glow_mask && glow_mask.is_none() {
            commands
                .entity(entity)
                .insert(LegacyMaterialMetadataError(format!(
                    "normal_glow material {} ({}) did not load its _BumpMap glow-mask handle",
                    pending.true_name, pending.source_material_id
                )));
            continue;
        }
        let textures = LegacyModelTextures {
            base: base_texture.clone(),
            bump: glow_mask,
            base_builtin_white: base_texture.is_none(),
            ..default()
        };
        let Some(sort_biases) = pending
            .render_passes
            .iter()
            .enumerate()
            .map(|(pass_ordinal, pass)| {
                legacy_static_world_sort_bias(pass.render_mode.source_queue, pass_ordinal)
            })
            .collect::<Option<Vec<_>>>()
        else {
            commands
                .entity(entity)
                .insert(LegacyMaterialMetadataError(format!(
                    "legacy static material {} ({}) has an unsupported render queue/pass ordering",
                    pending.true_name, pending.source_material_id
                )));
            continue;
        };
        let mut native_passes = Vec::with_capacity(pending.render_passes.len());
        for (mut pass, sort_bias) in pending
            .render_passes
            .iter()
            .copied()
            .zip(sort_biases.into_iter())
        {
            // SrcAlpha/OneMinusSrcAlpha with alpha=1 produces the same color
            // as replace. Moving these opaque props out of Bevy's transparent
            // phase removes depth-order flicker and accidental see-through,
            // while real alpha textures retain their authored blend pass.
            if pass.render_mode.blend == LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
                && pass.render_mode.depth_write
                && base_color.alpha >= 0.999
                && texture_is_fully_opaque
            {
                pass.render_mode.blend = LegacyBlendMode::Replace;
                pass.render_mode.source_queue = 2_000;
            }
            let Some(mut material) = params.material_for_pass(pass, &textures) else {
                continue;
            };
            material.sort_bias = sort_bias;
            let handle = if share_resources && let Some(cache) = material_cache.as_deref_mut() {
                cache.admit_immutable(material, &mut surface_materials)
            } else {
                surface_materials.add(material)
            };
            native_passes.push((pass.kind, handle));
        }
        if !native_passes.is_empty() {
            apply_cached_legacy_static_material(
                &mut commands,
                entity,
                mesh,
                &native_passes,
                mip_proof,
                range_tag,
                range,
                visibility,
            );
            if let (Some(key), Some(cache)) = (cache_key, material_cache.as_deref_mut()) {
                cache.entries.insert(
                    key,
                    CachedLegacyStaticMaterial {
                        passes: native_passes,
                        mip_proof,
                    },
                );
            }
        }
    }
}
