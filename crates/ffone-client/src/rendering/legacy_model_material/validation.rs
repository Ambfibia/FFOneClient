//! Runtime texture-binding, exact-mip metadata and asset-URI validation.

use super::shader_kind::LegacyMaterialExtrasError;
use bevy::prelude::*;
use ffone_skinned_model::{
    MaterialTextureBinding, NativeTextureMipLevel, PublishedMipPolicy, TextureColorSpace,
    TextureMipProvenance, minimal_windows_png_filename,
    windows_png_filename_preserving_legacy_extension,
};
use std::{
    ffi::OsStr,
    path::{Component, Path},
};

pub(super) fn validate_runtime_texture_binding(
    binding: &MaterialTextureBinding,
) -> Result<(), LegacyMaterialExtrasError> {
    if binding.unassigned_stale_null {
        // A named unassigned property is as legitimate as a nameless stale-null
        // one: the source PPtr named no object, so the shader property exists and
        // binds nothing. What it may never do is carry a texture payload.
        if binding.dynamic_texture.is_some()
            || binding.texture.is_some()
            || binding.source_name.is_some()
            || binding.uri.is_some()
            || binding.sampler.is_some()
            || binding.mip_provenance.is_some()
            || binding.mip_levels.is_some()
        {
            return Err(LegacyMaterialExtrasError(
                "unassigned texture property is contradictory".into(),
            ));
        }
        return Ok(());
    }
    if binding.slot.is_empty() {
        return Err(LegacyMaterialExtrasError(
            "runtime texture binding slot is empty".into(),
        ));
    }
    if binding.dynamic_texture.is_some() {
        if binding.texture.is_some()
            || binding.source_name.is_some()
            || binding.uri.is_some()
            || binding.sampler.is_some()
            || binding.mip_provenance.is_some()
            || binding.mip_levels.is_some()
        {
            return Err(LegacyMaterialExtrasError(format!(
                "dynamic texture binding {} also carries static Texture2D metadata",
                binding.slot
            )));
        }
        // Native GLB validation has already checked the exact embedded Ogg
        // bytes and provenance. Until a gameplay video-texture owner binds
        // them, resolve the ShaderLab slot through its declared fallback.
        return Ok(());
    }
    match (
        binding.texture,
        binding.source_name.as_deref(),
        binding.uri.as_deref(),
        binding.sampler.as_ref(),
        binding.mip_provenance.as_ref(),
        binding.mip_levels.as_deref(),
    ) {
        (None, None, None, None, None, None) => Ok(()),
        (
            Some(_),
            Some(source_name),
            Some(uri),
            Some(sampler),
            Some(mip_provenance),
            Some(mip_levels),
        ) => {
            let expected_filename = minimal_windows_png_filename(source_name).map_err(|error| {
                LegacyMaterialExtrasError(format!(
                    "texture binding {} sourceName cannot form a safe PNG URI: {error}",
                    binding.slot
                ))
            })?;
            let collision_filename =
                windows_png_filename_preserving_legacy_extension(source_name).map_err(|error| {
                    LegacyMaterialExtrasError(format!(
                        "texture binding {} sourceName cannot form a collision-safe PNG URI: {error}",
                        binding.slot
                    ))
                })?;
            let path = Path::new(uri);
            let actual_filename = path.file_name().and_then(|name| name.to_str());
            let filename_is_semantic = actual_filename.is_some_and(|actual| {
                actual.eq_ignore_ascii_case(&expected_filename)
                    || actual.eq_ignore_ascii_case(&collision_filename)
                    || is_shared_player_resource_set_png_uri(uri)
                    || is_shared_domain_texture_uri(uri)
                    || (is_shared_player_rendering_png_uri(uri)
                        && actual
                            .eq_ignore_ascii_case(&player_rendering_texture_filename(source_name)))
            });
            if !is_safe_relative_png_uri(uri) || !filename_is_semantic {
                return Err(LegacyMaterialExtrasError(format!(
                    "texture binding {} has unsafe or non-semantic external URI {uri:?}",
                    binding.slot
                )));
            }
            if sampler.descriptor.name != source_name {
                return Err(LegacyMaterialExtrasError(format!(
                    "texture binding {} sampler name contradicts sourceName",
                    binding.slot
                )));
            }
            validate_exact_mip_metadata(&binding.slot, uri, mip_provenance, mip_levels)?;
            match binding.slot.as_str() {
                "_MainTex" if binding.color_space != TextureColorSpace::Srgb => Err(
                    LegacyMaterialExtrasError("_MainTex must be typed as sRGB".into()),
                ),
                "_MaskTex" if binding.color_space != TextureColorSpace::Linear => Err(
                    LegacyMaterialExtrasError("_MaskTex must be typed as linear data".into()),
                ),
                "_BumpMap" | "_ShaderMap" if binding.color_space != TextureColorSpace::Linear => {
                    Err(LegacyMaterialExtrasError(format!(
                        "{} must be typed as linear",
                        binding.slot
                    )))
                }
                _ => Ok(()),
            }
        }
        _ => Err(LegacyMaterialExtrasError(format!(
            "texture binding {} must carry texture/sourceName/uri/sampler all together or all null",
            binding.slot
        ))),
    }
}

pub(super) fn validate_exact_mip_metadata(
    slot: &str,
    base_uri: &str,
    provenance: &TextureMipProvenance,
    levels: &[NativeTextureMipLevel],
) -> Result<(), LegacyMaterialExtrasError> {
    let invalid = |detail: &str| {
        LegacyMaterialExtrasError(format!(
            "texture binding {slot} has contradictory exact mip metadata: {detail}"
        ))
    };
    if !provenance.source_chain_complete
        || provenance.source_texture_format_name.is_empty()
        || !is_lower_hex_sha256(&provenance.source_chain_sha256)
        || levels.is_empty()
        || usize::try_from(provenance.source_mip_count).ok() != Some(levels.len())
        || levels.first().map(|level| level.uri.as_str()) != Some(base_uri)
    {
        return Err(invalid("incomplete provenance or level table"));
    }
    match provenance.published_policy {
        PublishedMipPolicy::BaseLevelOnly if levels.len() != 1 => {
            return Err(invalid("baseLevelOnly must contain exactly level zero"));
        }
        PublishedMipPolicy::ExactSourceLevels if levels.len() < 2 => {
            return Err(invalid(
                "exactSourceLevels must contain lower source levels",
            ));
        }
        _ => {}
    }

    let base_path = Path::new(base_uri);
    let base_stem = base_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| invalid("base URI has no UTF-8 stem"))?;
    let parent = base_path
        .parent()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty());
    let mut expected_offset = 0_u64;
    let mut previous_dimensions: Option<(u32, u32)> = None;
    for (index, level) in levels.iter().enumerate() {
        if level.level as usize != index
            || !is_safe_relative_png_uri(&level.uri)
            || level.width == 0
            || level.height == 0
            || level.source_byte_length == 0
            || level.png_byte_length == 0
            || level.source_byte_offset != expected_offset
            || !is_lower_hex_sha256(&level.source_byte_sha256)
            || !is_lower_hex_sha256(&level.decoded_rgba8_sha256)
            || !is_lower_hex_sha256(&level.png_sha256)
        {
            return Err(invalid(&format!("level {index} fields are invalid")));
        }
        let decoded_length = u64::from(level.width)
            .checked_mul(u64::from(level.height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| invalid(&format!("level {index} dimensions overflow")))?;
        if level.decoded_rgba8_byte_length != decoded_length {
            return Err(invalid(&format!(
                "level {index} decoded RGBA8 byte length is not width*height*4"
            )));
        }
        if index > 0 {
            let expected_uri = if let Some(parent) = parent {
                format!("{parent}/{base_stem}.mips/mip-{index:02}.png")
            } else {
                format!("{base_stem}.mips/mip-{index:02}.png")
            };
            if level.uri != expected_uri {
                return Err(invalid(&format!(
                    "level {index} URI {:?} is not exact published URI {expected_uri:?}",
                    level.uri
                )));
            }
        }
        if let Some((previous_width, previous_height)) = previous_dimensions {
            let expected = ((previous_width / 2).max(1), (previous_height / 2).max(1));
            if (level.width, level.height) != expected {
                return Err(invalid(&format!(
                    "level {index} dimensions {}x{} do not follow {}x{}",
                    level.width, level.height, expected.0, expected.1
                )));
            }
        }
        previous_dimensions = Some((level.width, level.height));
        expected_offset = expected_offset
            .checked_add(level.source_byte_length)
            .ok_or_else(|| invalid("source byte offsets overflow"))?;
    }
    if expected_offset != provenance.source_chain_byte_length {
        return Err(invalid(
            "source level byte ranges do not cover sourceChainByteLength",
        ));
    }
    Ok(())
}

pub(super) fn is_safe_relative_png_uri(uri: &str) -> bool {
    let path = Path::new(uri);
    if uri.is_empty()
        || uri.contains(['\\', ':', '?', '#', '%'])
        || path.is_absolute()
        || path.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return false;
    }

    let mut parent_count = 0_usize;
    let mut normal = Vec::new();
    let mut saw_normal = false;
    for component in path.components() {
        match component {
            Component::ParentDir if !saw_normal => parent_count += 1,
            Component::Normal(value) => {
                saw_normal = true;
                normal.push(value);
            }
            _ => return false,
        }
    }
    if normal.is_empty() {
        return false;
    }
    if parent_count == 0 {
        return true;
    }

    // Published model GLBs live below `models/<true-name>/model.glb`. Their
    // exact Texture2D payloads are shared either by the owning set's
    // `textures` directory or by `characters/player/rendering/textures` for
    // ToonRamps, so both canonical routes require a leading `../` chain. No
    // Domain-owned shared chains also cross model directories after relocation.
    // Other parent traversal is rejected, and AssetPath::resolve_embed
    // later proves that the resolved path remains inside the asset root.
    (normal.len() >= 2 && normal[0] == OsStr::new("textures"))
        || (normal.len() >= 3
            && normal[0] == OsStr::new("rendering")
            && normal[1] == OsStr::new("textures"))
        || is_shared_domain_texture_uri(uri)
}

pub(super) fn is_shared_domain_texture_uri(uri: &str) -> bool {
    let normal = leading_parent_uri_normal_components(uri);
    let Some(parts) = normal
        .iter()
        .map(|part| part.to_str())
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let semantic_owner = |owner: &str| {
        !owner.is_empty()
            && owner
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    };
    // Item packages retain accepted locations while sharing verified atlases
    // and their complete mip chains. AssetPath resolves them inside the root.
    let item_parts = match parts.as_slice() {
        ["back" | "glasses" | "hat" | "shirt" | "pants" | "shoes" | "vehicle" | "weapon", rest @ ..] => rest,
        _ => parts.as_slice(),
    };
    if let [owner, "models", model, textures, _, ..] = item_parts {
        if owner == model && semantic_owner(owner) && *textures == format!("{model}.textures") {
            return true;
        }
    }
    if let [owner, "textures", _, ..] = item_parts {
        if semantic_owner(owner) && ["back_", "face_", "hat_", "helmet_", "m_", "f_", "shirt_", "pants_", "shoes_", "vehicle_", "weapon_"]
            .iter().any(|prefix| owner.starts_with(prefix)) {
            return true;
        }
    }
    match parts.as_slice() {
        ["effects", "shared", "textures", _, ..]
        | ["characters", "shared", "textures", _, ..]
        | ["shared", "textures", _, ..] => true,
        // The Mandroid M-110 variant reuses the authored chef hat texture
        // rather than publishing another copy alongside the NPC model.
        ["player", "items", "hat", "hat_chefhat", "textures", _, ..] => true,
        [
            "characters",
            "npcs" | "mobs" | "nanos" | "fusions",
            owner,
            "textures",
            "shared",
            _,
            ..,
        ]
        | [owner, "textures", "shared", _, ..] => semantic_owner(owner),
        // A separate character variant may reuse its sibling's authored
        // texture/mip package, without duplicating those payloads.
        [owner, texture_dir, _, ..]
            if semantic_owner(owner)
                && ["npc_", "mob_", "fusion_", "nano_"]
                    .iter()
                    .any(|prefix| owner.starts_with(prefix))
                && *texture_dir == format!("{owner}.textures") =>
        {
            true
        }
        _ => false,
    }
}

pub(super) fn is_shared_player_rendering_png_uri(uri: &str) -> bool {
    let normal = leading_parent_uri_normal_components(uri);
    normal.len() >= 3 && normal[0] == OsStr::new("rendering") && normal[1] == OsStr::new("textures")
}

pub(super) fn is_shared_player_resource_set_png_uri(uri: &str) -> bool {
    let normal = leading_parent_uri_normal_components(uri);
    normal.len() >= 2 && normal[0] == OsStr::new("textures")
}

pub(super) fn leading_parent_uri_normal_components(uri: &str) -> Vec<&OsStr> {
    let mut saw_normal = false;
    let mut saw_parent = false;
    let mut normal = Vec::new();
    for component in Path::new(uri).components() {
        match component {
            Component::ParentDir if !saw_normal => saw_parent = true,
            Component::Normal(value) => {
                saw_normal = true;
                normal.push(value);
            }
            _ => return Vec::new(),
        }
    }
    if saw_parent { normal } else { Vec::new() }
}

pub(super) fn player_rendering_texture_filename(source_name: &str) -> String {
    let mut output = String::new();
    let mut underscore = false;
    for character in source_name.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            underscore = false;
        } else if !underscore && !output.is_empty() {
            output.push('_');
            underscore = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    output.truncate(112);
    format!("{output}.png")
}

pub(super) fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn exact_vec2_f32(
    values: [f64; 2],
    slot: &str,
    field: &str,
) -> Result<Vec2, LegacyMaterialExtrasError> {
    Ok(Vec2::new(
        exact_f32(values[0], slot, field)?,
        exact_f32(values[1], slot, field)?,
    ))
}

pub(super) fn exact_f32(value: f64, slot: &str, field: &str) -> Result<f32, LegacyMaterialExtrasError> {
    let converted = value as f32;
    if !value.is_finite() || !converted.is_finite() {
        return Err(LegacyMaterialExtrasError(format!(
            "texture binding {slot} {field} is not a finite f32"
        )));
    }
    Ok(converted)
}
