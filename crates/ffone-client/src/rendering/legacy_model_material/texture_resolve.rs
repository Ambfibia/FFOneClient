//! Resolution of glTF textures and Unity default textures for material slots.

use super::ExactMipPngBytes;
use super::exact_mip_cache::{ExactMipChainApplied, ExactMipChainCache};
use super::exact_mip_chain::resolve_exact_mip_chain;
use super::metadata::PendingLegacyModelMaterial;
use super::params::LegacyModelTextures;
use super::shader_kind::LegacyShaderKind;
use bevy::{
    asset::{AssetPath, LoadState, RenderAssetUsages},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use ffone_skinned_model::{PublishedMipPolicy, ShaderLabTextureDefault};

#[derive(Debug)]
pub(super) struct ResolvedLegacyModelTextures {
    pub(super) textures: LegacyModelTextures,
    pub(super) mip_marker: ExactMipChainApplied,
}

pub(super) fn resolve_gltf_textures(
    asset_server: &AssetServer,
    standard_materials: &Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    png_bytes: &Assets<ExactMipPngBytes>,
    mip_cache: &mut ExactMipChainCache,
    mesh: &Mesh3d,
    standard_handle: &MeshMaterial3d<StandardMaterial>,
    pending: &PendingLegacyModelMaterial,
) -> Result<Option<ResolvedLegacyModelTextures>, String> {
    let source = mesh
        .0
        .path()
        .ok_or_else(|| "glTF primitive mesh has no source asset path".to_owned())?
        .without_label()
        .clone_owned();
    let Some(standard_material) = standard_materials.get(&standard_handle.0) else {
        return match asset_server.load_state(standard_handle.0.id()) {
            LoadState::Failed(error) => Err(format!(
                "standard glTF material for {} failed to load: {error}",
                pending.true_name
            )),
            _ => Ok(None),
        };
    };
    let mut textures = LegacyModelTextures::default();
    let mut mip_marker = ExactMipChainApplied::default();
    apply_unassigned_shader_texture_defaults(pending, &mut textures, images);
    for binding in &pending.texture_bindings {
        if binding.texture_index.is_none() {
            continue;
        }
        mip_marker.assigned_texture_bindings = mip_marker
            .assigned_texture_bindings
            .checked_add(1)
            .ok_or_else(|| "assigned texture-binding count overflows u32".to_owned())?;
        let uses_loader_hint = matches!(
            binding.slot.as_str(),
            "_MainTex" | "_BumpMap" | "_ShaderMap"
        );
        let standard_image = match binding.slot.as_str() {
            "_MainTex" | "_MaskTex" => standard_material.base_color_texture.as_ref(),
            "_BumpMap" => standard_material.normal_map_texture.as_ref(),
            "_ShaderMap" => standard_material.occlusion_texture.as_ref(),
            // Other assigned slots are still loaded, hashed and assembled;
            // they simply are not sampled by the closed native shader set.
            _ => None,
        };
        if uses_loader_hint && standard_image.is_none() {
            return Err(format!(
                "material {} slot {} has no required StandardMaterial loader-hint handle",
                pending.true_name, binding.slot
            ));
        }
        let expected_uri = binding.uri.as_deref().ok_or_else(|| {
            format!(
                "material {} slot {} has no typed external URI",
                pending.true_name, binding.slot
            )
        })?;
        let expected_path = source
            .resolve_embed(&AssetPath::try_parse(expected_uri).map_err(|error| error.to_string())?);
        if let Some(standard_image) = standard_image {
            let actual_path = standard_image
                .path()
                .ok_or_else(|| {
                    format!(
                        "material {} slot {} StandardMaterial image has no asset path",
                        pending.true_name, binding.slot
                    )
                })?
                .without_label()
                .clone_owned();
            if actual_path != expected_path {
                return Err(format!(
                    "material {} slot {} StandardMaterial image path {:?} contradicts exact URI {:?}",
                    pending.true_name, binding.slot, actual_path, expected_path
                ));
            }
            let Some(_image) = images.get(standard_image) else {
                return match asset_server.load_state(standard_image.id()) {
                    LoadState::Failed(error) => Err(format!(
                        "material {} slot {} external PNG failed to load: {error}",
                        pending.true_name, binding.slot
                    )),
                    _ => Ok(None),
                };
            };
            // glTF loader hints prove the URI and asynchronous load, but their
            // cached sampler/color space can belong to another material slot.
            // Exact-chain admission validates the bytes and installs this
            // binding's interpretation on the final GPU image.
        }

        let Some(handle) = resolve_exact_mip_chain(
            asset_server,
            images,
            png_bytes,
            mip_cache,
            &source,
            binding,
            standard_image,
            standard_handle.0.id(),
        )?
        else {
            return Ok(None);
        };
        let leases = mip_cache
            .pending_materials
            .entry(standard_handle.0.id())
            .or_default();
        if !leases.iter().any(|lease| lease.id() == handle.id()) {
            leases.push(handle.clone());
        }
        let provenance = binding
            .mip_provenance
            .as_ref()
            .ok_or_else(|| format!("texture binding {} has no mip provenance", binding.slot))?;
        if provenance.published_policy == PublishedMipPolicy::ExactSourceLevels {
            let level_count = u32::try_from(
                binding
                    .mip_levels
                    .as_ref()
                    .expect("assigned binding was typed before runtime")
                    .len(),
            )
            .map_err(|_| format!("texture binding {} level count overflows u32", binding.slot))?;
            mip_marker.mip_chains_applied = mip_marker
                .mip_chains_applied
                .checked_add(1)
                .ok_or_else(|| "exact mip-chain count overflows u32".to_owned())?;
            mip_marker.mip_levels = mip_marker
                .mip_levels
                .checked_add(level_count)
                .ok_or_else(|| "exact mip-level count overflows u32".to_owned())?;
        }
        match binding.slot.as_str() {
            "_MainTex" | "_MaskTex" => textures.base = Some(handle),
            "_ShaderMap" if pending.params.shader == LegacyShaderKind::FusionEffect => {
                textures.effect_map = Some(handle);
            }
            "_ShaderMap" => textures.toon_ramp = Some(handle),
            "_BumpMap" => textures.bump = Some(handle),
            // Null/unused spec maps and custom maps remain visible in the
            // pending typed metadata; they are never guessed into another slot.
            _ => {}
        }
    }
    Ok(Some(ResolvedLegacyModelTextures {
        textures,
        mip_marker,
    }))
}

pub(super) fn apply_unassigned_shader_texture_defaults(
    pending: &PendingLegacyModelMaterial,
    textures: &mut LegacyModelTextures,
    images: &mut Assets<Image>,
) {
    // A saved assignment always wins. Only an absent/null saved slot may use
    // the exact ShaderLab fallback. Bevy's absent optional texture is its
    // implicit white image, while Unity's exact `black` built-in needs an
    // explicit one-pixel image. Other named built-ins and `blank` remain
    // provenance and therefore cannot make a required sampled slot complete.
    for property in &pending.shader_texture_defaults {
        let has_saved_assignment = pending
            .texture_bindings
            .iter()
            .any(|binding| binding.slot == property.slot && binding.texture_index.is_some());
        if has_saved_assignment {
            continue;
        }
        match (property.slot.as_str(), property.value) {
            ("_MainTex", ShaderLabTextureDefault::BuiltinWhite) => {
                textures.base_builtin_white = true;
            }
            ("_MaskTex", ShaderLabTextureDefault::BuiltinWhite)
                if pending.params.shader == LegacyShaderKind::HologramSolidAdditive =>
            {
                textures.base_builtin_white = true;
            }
            ("_ShaderMap", ShaderLabTextureDefault::BuiltinWhite)
                if pending.params.shader == LegacyShaderKind::FusionEffect =>
            {
                textures.effect_map_builtin_white = true;
            }
            ("_ShaderMap", ShaderLabTextureDefault::BuiltinWhite) => {
                textures.toon_ramp_builtin_white = true;
            }
            ("_BumpMap", ShaderLabTextureDefault::BuiltinWhite) => {
                textures.bump_builtin_white = true;
            }
            ("_MainTex", ShaderLabTextureDefault::BuiltinBlack) => {
                textures.base = Some(images.add(unity_builtin_black_image(true)));
            }
            ("_MaskTex", ShaderLabTextureDefault::BuiltinBlack)
                if pending.params.shader == LegacyShaderKind::HologramSolidAdditive =>
            {
                textures.base = Some(images.add(unity_builtin_black_image(true)));
            }
            ("_ShaderMap", ShaderLabTextureDefault::BuiltinBlack)
                if pending.params.shader == LegacyShaderKind::FusionEffect =>
            {
                textures.effect_map = Some(images.add(unity_builtin_black_image(false)));
            }
            ("_ShaderMap", ShaderLabTextureDefault::BuiltinBlack) => {
                textures.toon_ramp = Some(images.add(unity_builtin_black_image(false)));
            }
            ("_BumpMap", ShaderLabTextureDefault::BuiltinBlack) => {
                textures.bump = Some(images.add(unity_builtin_black_image(false)));
            }
            _ => {}
        }
    }
}

pub(super) fn unity_builtin_black_image(srgb: bool) -> Image {
    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0, 0, 0, 255],
        if srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::default(),
    )
}
