//! Exact mip-chain loading, digest checks, pixel transforms and image assembly.

use super::ExactMipPngBytes;
use super::exact_mip_cache::{
    ExactMipChainCache, ExactMipChainCacheEntry, ExactMipChainKey, ExactMipLevelAssets,
};
use super::params::LegacyGltfTextureBinding;
use super::samplers::{exact_sampler_descriptor, validate_sampler_descriptor};
use bevy::{
    asset::{AssetPath, LoadState, RenderAssetUsages},
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat},
};
use ffone_skinned_model::{
    NativeTextureMipLevel, PublishedMipPolicy, PublishedPixelTransform, TextureColorSpace,
};
use sha2::{Digest, Sha256};

pub(super) fn resolve_exact_mip_chain(
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    png_bytes: &Assets<ExactMipPngBytes>,
    cache: &mut ExactMipChainCache,
    model_source: &AssetPath<'static>,
    binding: &LegacyGltfTextureBinding,
    standard_base: Option<&Handle<Image>>,
    source_material: AssetId<StandardMaterial>,
) -> Result<Option<Handle<Image>>, String> {
    let levels = binding
        .mip_levels
        .as_deref()
        .ok_or_else(|| format!("texture binding {} has no exact mip levels", binding.slot))?;
    let provenance = binding.mip_provenance.as_ref().ok_or_else(|| {
        format!(
            "texture binding {} has no exact mip provenance",
            binding.slot
        )
    })?;
    let key = ExactMipChainKey {
        model_source: model_source.clone(),
        binding_digest: exact_binding_digest(binding)?,
    };
    if let Some(weak) = cache
        .entries
        .get(&key)
        .and_then(|entry| entry.assembled.as_ref())
    {
        if let Some(strong) = weak.upgrade() {
            let handle = Handle::Strong(strong);
            if images.get(&handle).is_none() {
                return Err(format!(
                    "texture binding {} cached assembled Image was unexpectedly removed",
                    binding.slot
                ));
            }
            return Ok(Some(handle));
        }
        // No live material owns this chain anymore. Reload on the next admission;
        // the cache must not keep an unloaded character's GPU/CPU images alive.
        cache.entries.remove(&key);
    }
    if !cache.entries.contains_key(&key) {
        let sampler = binding
            .sampler
            .as_ref()
            .ok_or_else(|| format!("texture binding {} has no exact sampler", binding.slot))?;
        let image_sampler = ImageSampler::Descriptor(exact_sampler_descriptor(
            &binding.slot,
            &sampler.descriptor,
        )?);
        let is_srgb = binding.color_space == TextureColorSpace::Srgb;
        let mut level_assets = Vec::with_capacity(levels.len());
        for level in levels {
            let path = model_source.resolve_embed(
                &AssetPath::try_parse(&level.uri).map_err(|error| error.to_string())?,
            );
            // URI-keyed decoded PNGs may be shared across Linear/sRGB slots.
            // The binding digest owns the final GPU interpretation, not this
            // staging handle or the settings of whichever glTF loaded first.
            let settings_sampler = image_sampler.clone();
            let image = asset_server
                .load_builder()
                .with_settings::<ImageLoaderSettings>(move |settings| {
                    settings.is_srgb = is_srgb;
                    settings.sampler = settings_sampler.clone();
                    // Individual published levels are CPU staging assets. Only
                    // the assembled multi-mip Image belongs in the render world.
                    settings.asset_usage = RenderAssetUsages::MAIN_WORLD;
                })
                .load::<Image>(path.clone());
            let raw = asset_server.load::<ExactMipPngBytes>(path.clone());
            level_assets.push(ExactMipLevelAssets {
                path,
                image,
                png_bytes: raw,
            });
        }
        cache.entries.insert(
            key.clone(),
            ExactMipChainCacheEntry {
                levels: level_assets,
                assembled: None,
                source_material,
            },
        );
    }

    let entry = cache
        .entries
        .get(&key)
        .expect("exact mip entry was inserted above")
        .clone();
    let mut loaded_levels = Vec::with_capacity(entry.levels.len());
    for (metadata, assets) in levels.iter().zip(&entry.levels) {
        let Some(image) = images.get(&assets.image) else {
            return match asset_server.load_state(assets.image.id()) {
                LoadState::Failed(error) => Err(format!(
                    "texture binding {} mip {} image {:?} failed to load: {error}",
                    binding.slot, metadata.level, assets.path
                )),
                _ => Ok(None),
            };
        };
        let Some(raw) = png_bytes.get(&assets.png_bytes) else {
            return match asset_server.load_state(assets.png_bytes.id()) {
                LoadState::Failed(error) => Err(format!(
                    "texture binding {} mip {} PNG bytes {:?} failed to load: {error}",
                    binding.slot, metadata.level, assets.path
                )),
                _ => Ok(None),
            };
        };
        let staging = exact_staging_image(binding, image)?;
        validate_loaded_mip_level(binding, metadata, &staging, raw)?;
        loaded_levels.push(staging);
    }

    if provenance.published_policy == PublishedMipPolicy::BaseLevelOnly {
        if let Some(standard_base) = standard_base {
            if images.get(standard_base).is_some_and(|image| {
                image.asset_usage.contains(RenderAssetUsages::RENDER_WORLD)
                    && validate_loaded_image(binding, image).is_ok()
            }) {
                complete_exact_mip_entry(cache, &key, standard_base);
                return Ok(Some(standard_base.clone()));
            }
        }
        // Missing loader hints, CPU-only staging images and hints owned by a
        // different color/sampler interpretation all need a distinct GPU image,
        // even when the published chain contains only one level.
        let assembled = assemble_exact_mip_image(binding, &loaded_levels)?;
        let handle = cache.shared_images.insert(assembled, images);
        complete_exact_mip_entry(cache, &key, &handle);
        return Ok(Some(handle));
    }
    let assembled = assemble_exact_mip_image(binding, &loaded_levels)?;
    let handle = cache.shared_images.insert(assembled, images);
    complete_exact_mip_entry(cache, &key, &handle);
    Ok(Some(handle))
}

pub(super) fn complete_exact_mip_entry(
    cache: &mut ExactMipChainCache,
    key: &ExactMipChainKey,
    image: &Handle<Image>,
) {
    if let Handle::Strong(handle) = image {
        let entry = cache
            .entries
            .get_mut(key)
            .expect("admitted exact mip chain");
        entry.assembled = Some(std::sync::Arc::downgrade(handle));
        // Validation has completed. Encoded PNGs and individual decoded levels
        // are staging data, not permanent copies beside the assembled GPU image.
        entry.levels.clear();
    }
}

pub(super) fn exact_binding_digest(binding: &LegacyGltfTextureBinding) -> Result<[u8; 32], String> {
    let bytes = serde_json::to_vec(&(
        &binding.slot,
        binding.texture_index,
        &binding.source_name,
        &binding.uri,
        &binding.sampler,
        &binding.mip_provenance,
        &binding.mip_levels,
        binding.color_space,
    ))
    .map_err(|error| {
        format!(
            "cannot hash exact texture binding {}: {error}",
            binding.slot
        )
    })?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

#[cfg(test)]
pub(super) fn exact_interpretation_digest(binding: &LegacyGltfTextureBinding) -> Result<[u8; 32], String> {
    let sampler = binding.sampler.as_ref().map(|sampler| &sampler.descriptor);
    let bytes = serde_json::to_vec(&(
        binding.color_space,
        sampler.map(|sampler| sampler.mag_filter),
        sampler.map(|sampler| sampler.min_filter),
        sampler.map(|sampler| sampler.wrap_s),
        sampler.map(|sampler| sampler.wrap_t),
        sampler.map(|sampler| sampler.anisotropy_level),
        sampler.map(|sampler| sampler.mip_map_bias),
    ))
    .map_err(|error| {
        format!(
            "cannot hash texture interpretation for {}: {error}",
            binding.slot
        )
    })?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

/// PNG decoding preserves encoded RGBA8 bytes in both color spaces. Preserve
/// the shared glTF image and reinterpret only a private, hash-validated staging
/// copy; assembly installs the binding's sampler and GPU format on its output.
pub(super) fn exact_staging_image(
    binding: &LegacyGltfTextureBinding,
    image: &Image,
) -> Result<Image, String> {
    if !matches!(image.texture_descriptor.format,
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb)
    {
        return Err(format!("texture binding {} has unsupported PNG staging format {:?}",
            binding.slot, image.texture_descriptor.format));
    }
    let mut staging = image.clone();
    staging.texture_descriptor.format = match binding.color_space {
        TextureColorSpace::Linear => TextureFormat::Rgba8Unorm,
        TextureColorSpace::Srgb => TextureFormat::Rgba8UnormSrgb,
    };
    Ok(staging)
}

pub(super) fn validate_loaded_mip_level(
    binding: &LegacyGltfTextureBinding,
    level: &NativeTextureMipLevel,
    image: &Image,
    png: &ExactMipPngBytes,
) -> Result<(), String> {
    // A mip staging URI can already have been loaded by glTF before this
    // exact-chain request runs. Bevy keys Images by URI, so the first load's
    // sampler settings win even though the staging Image is never sampled by
    // the legacy material. Validate its pixels and color interpretation here;
    // `assemble_exact_mip_image` installs the audited sampler on the distinct
    // render-world Image that is actually bound.
    let expected_srgb = binding.color_space == TextureColorSpace::Srgb;
    if image.texture_descriptor.format.is_srgb() != expected_srgb {
        return Err(format!(
            "texture binding {} mip {} loaded color space contradicts typed {:?}",
            binding.slot, level.level, binding.color_space
        ));
    }
    let descriptor = &image.texture_descriptor;
    let expected_format = match binding.color_space {
        TextureColorSpace::Srgb => TextureFormat::Rgba8UnormSrgb,
        TextureColorSpace::Linear => TextureFormat::Rgba8Unorm,
    };
    if descriptor.dimension != TextureDimension::D2
        || descriptor.size.width != level.width
        || descriptor.size.height != level.height
        || descriptor.size.depth_or_array_layers != 1
        || descriptor.mip_level_count != 1
        || descriptor.sample_count != 1
        || descriptor.format != expected_format
        || image.texture_view_descriptor.is_some()
    {
        return Err(format!(
            "texture binding {} mip {} loaded image layout {:?} is not exact {}x{} RGBA8 {:?}",
            binding.slot, level.level, descriptor, level.width, level.height, binding.color_space
        ));
    }
    let data = image.data.as_deref().ok_or_else(|| {
        format!(
            "texture binding {} mip {} has no CPU RGBA8 data",
            binding.slot, level.level
        )
    })?;
    let decoded_length = usize::try_from(level.decoded_rgba8_byte_length).map_err(|_| {
        format!(
            "texture binding {} mip {} decoded byte length does not fit this platform",
            binding.slot, level.level
        )
    })?;
    if data.len() != decoded_length {
        return Err(format!(
            "texture binding {} mip {} decoded RGBA8 length mismatch: got {} bytes, expected {} bytes",
            binding.slot,
            level.level,
            data.len(),
            decoded_length
        ));
    }
    let provenance = binding.mip_provenance.as_ref().ok_or_else(|| {
        format!(
            "texture binding {} mip {} has no published pixel-transform provenance",
            binding.slot, level.level
        )
    })?;
    let source_decoded = inverse_published_pixel_transform_rgba8(
        data,
        level.width,
        level.height,
        provenance.published_pixel_transform,
    )
    .map_err(|detail| {
        format!(
            "texture binding {} mip {} cannot reverse published pixel transform: {detail}",
            binding.slot, level.level
        )
    })?;
    let decoded_hash = sha256_hex(&source_decoded);
    if decoded_hash != level.decoded_rgba8_sha256 {
        return Err(format!(
            "texture binding {} mip {} source-decoded RGBA8 hash mismatch after inverse {:?}: got {decoded_hash}, expected {}",
            binding.slot,
            level.level,
            provenance.published_pixel_transform,
            level.decoded_rgba8_sha256
        ));
    }
    let png_length = usize::try_from(level.png_byte_length).map_err(|_| {
        format!(
            "texture binding {} mip {} PNG byte length does not fit this platform",
            binding.slot, level.level
        )
    })?;
    if png.bytes.len() != png_length || sha256_hex(&png.bytes) != level.png_sha256 {
        return Err(format!(
            "texture binding {} mip {} published PNG length/hash mismatch",
            binding.slot, level.level
        ));
    }
    Ok(())
}

pub(super) fn inverse_published_pixel_transform_rgba8(
    published: &[u8],
    width: u32,
    height: u32,
    transform: PublishedPixelTransform,
) -> Result<Vec<u8>, String> {
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or_else(|| "RGBA8 row byte length overflows this platform".to_owned())?;
    let height = usize::try_from(height)
        .map_err(|_| "RGBA8 height does not fit this platform".to_owned())?;
    let expected = row_bytes
        .checked_mul(height)
        .ok_or_else(|| "RGBA8 image byte length overflows this platform".to_owned())?;
    if row_bytes == 0 || height == 0 || published.len() != expected {
        return Err(format!(
            "published RGBA8 layout is {} bytes, expected {row_bytes}x{height}={expected}",
            published.len()
        ));
    }

    match transform {
        PublishedPixelTransform::VerticalFlipOnlyForPngTopLeftOrigin => {
            let mut source_decoded = Vec::with_capacity(expected);
            for row in published.chunks_exact(row_bytes).rev() {
                source_decoded.extend_from_slice(row);
            }
            Ok(source_decoded)
        }
    }
}

pub(super) fn assemble_exact_mip_image(
    binding: &LegacyGltfTextureBinding,
    levels: &[Image],
) -> Result<Image, String> {
    let Some(base) = levels.first() else {
        return Err(format!(
            "texture binding {} has no mip images",
            binding.slot
        ));
    };
    let mip_level_count = u32::try_from(levels.len()).map_err(|_| {
        format!(
            "texture binding {} mip-level count does not fit WebGPU",
            binding.slot
        )
    })?;
    let total_bytes = levels.iter().try_fold(0_usize, |total, image| {
        image
            .data
            .as_ref()
            .and_then(|data| total.checked_add(data.len()))
            .ok_or_else(|| {
                format!(
                    "texture binding {} mip-chain byte length overflows",
                    binding.slot
                )
            })
    })?;
    let mut data = Vec::with_capacity(total_bytes);
    for image in levels {
        data.extend_from_slice(
            image
                .data
                .as_deref()
                .ok_or_else(|| format!("texture binding {} mip has no CPU data", binding.slot))?,
        );
    }
    let mut assembled = base.clone();
    assembled.data = Some(data);
    assembled.texture_descriptor.mip_level_count = mip_level_count;
    let sampler = binding.sampler.as_ref().ok_or_else(|| {
        format!(
            "texture binding {} has no typed sampler descriptor",
            binding.slot
        )
    })?;
    assembled.sampler = ImageSampler::Descriptor(exact_sampler_descriptor(
        &binding.slot,
        &sampler.descriptor,
    )?);
    assembled.asset_usage = RenderAssetUsages::default();
    assembled.copy_on_resize = false;
    validate_loaded_image(binding, &assembled)?;
    Ok(assembled)
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn validate_loaded_image(binding: &LegacyGltfTextureBinding, image: &Image) -> Result<(), String> {
    let expected_sampler = binding.sampler.as_ref().ok_or_else(|| {
        format!(
            "texture binding {} has no typed sampler descriptor",
            binding.slot
        )
    })?;
    if expected_sampler.descriptor.mip_map_bias != 0.0 {
        return Err(format!(
            "texture binding {} has nonzero legacy mipMapBias {}, which a WebGPU sampler cannot represent",
            binding.slot, expected_sampler.descriptor.mip_map_bias
        ));
    }
    let ImageSampler::Descriptor(actual) = &image.sampler else {
        return Err(format!(
            "texture binding {} external PNG did not retain its glTF sampler descriptor",
            binding.slot
        ));
    };
    validate_sampler_descriptor(&binding.slot, &expected_sampler.descriptor, actual)?;
    let expected_srgb = binding.color_space == TextureColorSpace::Srgb;
    if image.texture_descriptor.format.is_srgb() != expected_srgb {
        return Err(format!(
            "texture binding {} loaded color space contradicts typed {:?}",
            binding.slot, binding.color_space
        ));
    }
    Ok(())
}
