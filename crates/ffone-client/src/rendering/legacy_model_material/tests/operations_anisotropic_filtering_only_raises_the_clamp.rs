use super::*;

#[test]
fn loaded_sampler_must_match_exact_filter_wrap_and_anisotropy() {
    let expected = NativeSampler {
        name: "npc_dexter.dds".into(),
        mag_filter: SamplerMagFilter::Linear,
        min_filter: SamplerMinFilter::Linear,
        wrap_s: SamplerWrapMode::Repeat,
        wrap_t: SamplerWrapMode::Repeat,
        legacy_filter_mode: 1,
        legacy_wrap_mode: 0,
        anisotropy_level: 1,
        mip_map_bias: 0.0,
    };
    let mut actual = bevy::image::ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        anisotropy_clamp: 1,
        ..default()
    };
    validate_sampler_descriptor("_MainTex", &expected, &actual).unwrap();
    actual.address_mode_v = ImageAddressMode::ClampToEdge;
    assert!(
        validate_sampler_descriptor("_MainTex", &expected, &actual)
            .unwrap_err()
            .contains("contradicts exact descriptor")
    );
}

#[test]
fn bilinear_native_anisotropy_is_reversible_and_does_not_admit_ui_samplers() {
    let baseline = ImageSamplerDescriptor {
        label: Some("ffone/native-sampler/nearest/1".into()),
        mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest, ..default()
    };
    let mut image = baseline.clone();
    for _ in 0..3 {
        apply_native_anisotropy(&mut image, 16);
        assert_eq!(image.mipmap_filter, ImageFilterMode::Linear);
        assert_eq!(image.anisotropy_clamp, 16);
        apply_native_anisotropy(&mut image, 1);
        assert_eq!(image, baseline);
    }
    let mut ui = ImageSamplerDescriptor {label: None, ..baseline};
    let before = ui.clone();
    apply_native_anisotropy(&mut ui, 16);
    assert_eq!(ui, before);
}

#[test]
fn anisotropic_filtering_only_raises_the_clamp_of_a_trilinear_legacy_sampler() {
    let trilinear = NativeSampler {
        name: "npc_dexter.dds".into(),
        mag_filter: SamplerMagFilter::Linear,
        min_filter: SamplerMinFilter::LinearMipmapLinear,
        wrap_s: SamplerWrapMode::Repeat,
        wrap_t: SamplerWrapMode::Repeat,
        legacy_filter_mode: 2,
        legacy_wrap_mode: 0,
        anisotropy_level: 1,
        mip_map_bias: 0.0,
    };
    let point = NativeSampler {
        min_filter: SamplerMinFilter::Nearest,
        mag_filter: SamplerMagFilter::Nearest,
        legacy_filter_mode: 0,
        ..trilinear.clone()
    };

    let exact = exact_sampler_descriptor("_MainTex", &trilinear).unwrap();
    assert_eq!(exact.anisotropy_clamp, LEGACY_TEXTURE_ANISOTROPY_DISABLED);

    // The published set serializes `anisotropyLevel: 1` everywhere, so the
    // enabled option is the only thing that can raise this field.
    let restore =
        LEGACY_TEXTURE_ANISOTROPY.swap(LEGACY_TEXTURE_ANISOTROPY_ENABLED, Ordering::Relaxed);
    let filtered = exact_sampler_descriptor("_MainTex", &trilinear).unwrap();
    let unfiltered = exact_sampler_descriptor("_MainTex", &point).unwrap();
    LEGACY_TEXTURE_ANISOTROPY.store(restore, Ordering::Relaxed);

    assert_eq!(filtered.anisotropy_clamp, LEGACY_TEXTURE_ANISOTROPY_ENABLED);
    assert_eq!(filtered.mag_filter, exact.mag_filter);
    assert_eq!(filtered.min_filter, exact.min_filter);
    assert_eq!(filtered.mipmap_filter, exact.mipmap_filter);
    assert_eq!(filtered.address_mode_u, exact.address_mode_u);
    assert_eq!(filtered.address_mode_v, exact.address_mode_v);
    // WebGPU rejects an anisotropic clamp on a point-filtered sampler.
    assert_eq!(
        unfiltered.anisotropy_clamp,
        LEGACY_TEXTURE_ANISOTROPY_DISABLED
    );

    // An image stamped on either side of a live toggle stays valid.
    for clamp in [
        LEGACY_TEXTURE_ANISOTROPY_DISABLED,
        LEGACY_TEXTURE_ANISOTROPY_ENABLED,
    ] {
        let actual = ImageSamplerDescriptor {
            anisotropy_clamp: clamp,
            ..filtered.clone()
        };
        validate_sampler_descriptor("_MainTex", &trilinear, &actual).unwrap();
    }
    let wrong_wrap = ImageSamplerDescriptor {
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..filtered.clone()
    };
    assert!(validate_sampler_descriptor("_MainTex", &trilinear, &wrong_wrap).is_err());
}
