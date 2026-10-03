//! Native sampler baselines, anisotropy and exact sampler descriptors.

use bevy::{
    image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor},
    prelude::*,
};
use ffone_skinned_model::{NativeSampler, SamplerMagFilter, SamplerMinFilter, SamplerWrapMode};
use std::sync::atomic::{AtomicU16, Ordering};

/// Clamp applied to every trilinear legacy texture while the
/// `ANISOTROPIC FILTERING` graphics option is enabled. WebGPU allows up to 16.
pub const LEGACY_TEXTURE_ANISOTROPY_ENABLED: u16 = 16;

/// Disabled state. Every published legacy sampler serializes
/// `anisotropyLevel: 1`, so this is also the exact source baseline.
pub const LEGACY_TEXTURE_ANISOTROPY_DISABLED: u16 = 1;

/// Live `ANISOTROPIC FILTERING` clamp for legacy model textures.
///
/// Unity kept `Texture2D.anisotropyLevel` (always 1 in the published set) and
/// multiplied `QualitySettings.anisotropicFiltering` on top of it at runtime.
/// Bevy instead bakes the sampler into the `Image` asset, so the loader and the
/// sampler validator both read the option from here rather than each growing
/// their own copy of the graphics settings.
pub(super) static LEGACY_TEXTURE_ANISOTROPY: AtomicU16 = AtomicU16::new(LEGACY_TEXTURE_ANISOTROPY_DISABLED);

/// WebGPU rejects `anisotropy_clamp > 1` unless all three filters are linear,
/// so a point-filtered legacy texture must keep its exact clamp.
pub(super) fn sampler_supports_anisotropy(descriptor: &ImageSamplerDescriptor) -> bool {
    descriptor.mag_filter == ImageFilterMode::Linear
        && descriptor.min_filter == ImageFilterMode::Linear
        && descriptor.mipmap_filter == ImageFilterMode::Linear
}

// Sampler labels retain only the immutable sampling baseline, never asset identity.
// They travel with asynchronously loaded/assembled Images and do not hold handles.
pub(super) fn native_sampler_baseline(descriptor: &ImageSamplerDescriptor) -> Option<(ImageFilterMode, u16)> {
    let (mip, clamp) = descriptor.label.as_deref()?.strip_prefix("ffone/native-sampler/")?.split_once('/')?;
    Some((match mip { "linear" => ImageFilterMode::Linear, "nearest" => ImageFilterMode::Nearest, _ => return None }, clamp.parse().ok()?))
}
pub(super) fn apply_native_anisotropy(descriptor: &mut ImageSamplerDescriptor, desired: u16) {
    let Some((mipmap_filter, legacy)) = native_sampler_baseline(descriptor) else { return; };
    descriptor.mipmap_filter = mipmap_filter;
    descriptor.anisotropy_clamp = legacy;
    if descriptor.mag_filter == ImageFilterMode::Linear && descriptor.min_filter == ImageFilterMode::Linear {
        descriptor.anisotropy_clamp = legacy.max(desired);
        if descriptor.anisotropy_clamp > 1 { descriptor.mipmap_filter = ImageFilterMode::Linear; }
    }
}

pub(super) fn validate_sampler_descriptor(
    slot: &str,
    expected: &NativeSampler,
    actual: &ImageSamplerDescriptor,
) -> Result<(), String> {
    let expected_descriptor = exact_sampler_descriptor(slot, expected)?;
    let legacy_anisotropy = u16::try_from(expected.anisotropy_level.max(1)).unwrap_or(u16::MAX);
    // Filters and wrap modes stay exact. Anisotropy is the one field the
    // graphics option may raise, and an image stamped on the other side of a
    // live toggle is still a valid legacy sampler, so accept the whole range
    // the option can produce instead of only the value committed right now.
    let filterable = expected_descriptor.mag_filter == ImageFilterMode::Linear
        && expected_descriptor.min_filter == ImageFilterMode::Linear;
    let baseline_mip = native_sampler_baseline(&expected_descriptor).unwrap().0;
    let mipmap_matches = actual.mipmap_filter == baseline_mip
        || (filterable && actual.anisotropy_clamp > 1 && actual.mipmap_filter == ImageFilterMode::Linear);
    let anisotropy_matches = if filterable {
        (legacy_anisotropy..=legacy_anisotropy.max(LEGACY_TEXTURE_ANISOTROPY_ENABLED))
            .contains(&actual.anisotropy_clamp)
    } else {
        actual.anisotropy_clamp == expected_descriptor.anisotropy_clamp
    };
    if actual.mag_filter != expected_descriptor.mag_filter
        || actual.min_filter != expected_descriptor.min_filter
        || !mipmap_matches
        || actual.address_mode_u != expected_descriptor.address_mode_u
        || actual.address_mode_v != expected_descriptor.address_mode_v
        || !anisotropy_matches
        || (actual.anisotropy_clamp > 1 && !sampler_supports_anisotropy(actual))
        || actual.compare.is_some()
        || actual.border_color.is_some()
    {
        return Err(format!(
            "texture binding {slot} loaded sampler {actual:?} contradicts exact descriptor {expected:?}"
        ));
    }
    Ok(())
}

pub(super) fn exact_sampler_descriptor(
    slot: &str,
    expected: &NativeSampler,
) -> Result<ImageSamplerDescriptor, String> {
    if expected.mip_map_bias != 0.0 {
        return Err(format!(
            "texture binding {slot} has nonzero legacy mipMapBias {}, which a WebGPU sampler cannot represent",
            expected.mip_map_bias
        ));
    }
    let mag_filter = match expected.mag_filter {
        SamplerMagFilter::Nearest => ImageFilterMode::Nearest,
        SamplerMagFilter::Linear => ImageFilterMode::Linear,
    };
    let (min_filter, mipmap_filter) = match expected.min_filter {
        SamplerMinFilter::Nearest | SamplerMinFilter::NearestMipmapNearest => {
            (ImageFilterMode::Nearest, ImageFilterMode::Nearest)
        }
        SamplerMinFilter::Linear | SamplerMinFilter::LinearMipmapNearest => {
            (ImageFilterMode::Linear, ImageFilterMode::Nearest)
        }
        SamplerMinFilter::NearestMipmapLinear => {
            (ImageFilterMode::Nearest, ImageFilterMode::Linear)
        }
        SamplerMinFilter::LinearMipmapLinear => (ImageFilterMode::Linear, ImageFilterMode::Linear),
    };
    let address_mode = |mode| match mode {
        SamplerWrapMode::ClampToEdge => ImageAddressMode::ClampToEdge,
        SamplerWrapMode::MirroredRepeat => ImageAddressMode::MirrorRepeat,
        SamplerWrapMode::Repeat => ImageAddressMode::Repeat,
    };
    let anisotropy = u16::try_from(expected.anisotropy_level.max(1)).map_err(|_| {
        format!(
            "texture binding {slot} anisotropy {} does not fit WebGPU",
            expected.anisotropy_level
        )
    })?;
    let mut descriptor = ImageSamplerDescriptor {
        address_mode_u: address_mode(expected.wrap_s),
        address_mode_v: address_mode(expected.wrap_t),
        mag_filter,
        min_filter,
        mipmap_filter,
        anisotropy_clamp: anisotropy,
        ..default()
    };
    // The published set serializes `anisotropyLevel: 1` everywhere, which left
    // every trilinear world texture point-sampled along its long axis. Raising
    // the option restores the authored baseline when disabled. Bilinear native
    // textures promote mip sampling to linear while anisotropy is enabled;
    // point textures, wrap modes and the complete mip chain remain unchanged.
    descriptor.label = Some(format!("ffone/native-sampler/{}/{}",
        if mipmap_filter == ImageFilterMode::Linear { "linear" } else { "nearest" }, anisotropy));
    apply_native_anisotropy(&mut descriptor, LEGACY_TEXTURE_ANISOTROPY.load(Ordering::Relaxed));
    Ok(descriptor)
}
