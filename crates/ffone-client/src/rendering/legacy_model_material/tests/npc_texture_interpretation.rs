use super::super::exact_mip_chain::exact_staging_image;
use super::*;

#[test]
fn portal_shared_png_keeps_linear_mips_when_gltf_loaded_srgb_first() {
    use bevy::image::{CompressedImageFormats, ImageType};
    let model_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/characters/npcs/npc_fusiongate/npc_fusiongate.glb");
    let pending = production_model_pending(
        "../../assets/game/characters/npcs/npc_fusiongate/npc_fusiongate.glb",
        "spawn11_green",
    );
    let binding = pending
        .texture_bindings
        .iter()
        .find(|b| b.slot == "_BumpMap")
        .unwrap();
    assert_eq!(binding.color_space, TextureColorSpace::Linear);
    let sampler = ImageSampler::Descriptor(
        exact_sampler_descriptor(&binding.slot, &binding.sampler.as_ref().unwrap().descriptor)
            .unwrap(),
    );
    let mut levels = Vec::new();
    for level in binding.mip_levels.as_ref().unwrap() {
        let bytes = std::fs::read(model_path.parent().unwrap().join(&level.uri)).unwrap();
        let shared = Image::from_buffer(
            &bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            sampler.clone(),
            RenderAssetUsages::default(),
        )
        .unwrap();
        assert!(
            validate_loaded_image(binding, &shared)
                .unwrap_err()
                .contains("color space")
        );
        let private = exact_staging_image(binding, &shared).unwrap();
        validate_loaded_mip_level(
            binding,
            level,
            &private,
            &ExactMipPngBytes {
                bytes: bytes.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            shared.texture_descriptor.format,
            TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(
            shared.data, private.data,
            "reinterpretation must preserve every published byte"
        );
        let mut damaged = private.clone();
        damaged.data.as_mut().unwrap()[0] ^= 1;
        assert!(
            validate_loaded_mip_level(binding, level, &damaged, &ExactMipPngBytes { bytes })
                .unwrap_err()
                .contains("hash mismatch")
        );
        levels.push(private);
    }
    let linear = assemble_exact_mip_image(binding, &levels).unwrap();
    validate_loaded_image(binding, &linear).unwrap();
    assert_eq!(linear.texture_descriptor.format, TextureFormat::Rgba8Unorm);
    assert_eq!(
        linear.texture_descriptor.mip_level_count as usize,
        levels.len()
    );
    let mut srgb_binding = binding.clone();
    srgb_binding.color_space = TextureColorSpace::Srgb;
    let srgb_levels: Vec<_> = levels
        .iter()
        .map(|image| exact_staging_image(&srgb_binding, image).unwrap())
        .collect();
    let srgb = assemble_exact_mip_image(&srgb_binding, &srgb_levels).unwrap();
    let mut assets = Assets::<Image>::default();
    let mut shared = super::super::shared_assets::SharedAssets::<Image>::default();
    let linear_handle = shared.insert(linear, &mut assets);
    let srgb_handle = shared.insert(srgb, &mut assets);
    assert_ne!(
        linear_handle.id(),
        srgb_handle.id(),
        "GPU color interpretations must stay independent"
    );
    assert_eq!(
        assets.get(&linear_handle).unwrap().data,
        assets.get(&srgb_handle).unwrap().data
    );
}
