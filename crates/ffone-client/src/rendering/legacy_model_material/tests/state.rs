use super::*;

#[test]
fn production_retro_fusion_eye_keeps_new_atlas_and_additive_state() {
    let pending = production_model_pending(
        "../../assets/game/characters/fusions/fusion_dexter/fusion_dexter.glb",
        "spwaneye",
    );
    assert_eq!(pending.params.shader, LegacyShaderKind::AdditiveTwoSided);
    assert_eq!(pending.params.alpha_cutoff, 0.0);
    assert_eq!(
        pending.params.base_color,
        LinearRgba::new(0.5, 0.5, 0.5, 1.0)
    );
    assert_eq!(
        pending.params.emission,
        LinearRgba::new(0.347_494_57, 0.347_494_57, 0.347_494_57, 1.0)
    );
    let plan = pending.params.render_plan();
    let [pass] = plan.passes.as_slice() else {
        panic!("Fusion Eyes must remain one exact additive pass");
    };
    assert_eq!(pass.render_mode.blend, LegacyBlendMode::OneOne);
    assert_eq!(pass.render_mode.cull, LegacyCullMode::Off);
    assert!(!pass.render_mode.depth_write);
    assert_eq!(pass.render_mode.color_write, LegacyColorWriteMask::Rgb);
    assert_eq!(pass.render_mode.source_queue, 3000);
    assert_eq!(pass.alpha_test, LegacyAlphaTest::GreaterMaterialCutoff);
    let atlas = pending
        .texture_bindings
        .iter()
        .find(|binding| binding.slot == "_MainTex")
        .expect("Fusion Eyes must bind the published high-resolution atlas");
    assert_eq!(atlas.source_name.as_deref(), Some("spwaneye.dds"));
    assert_eq!(
        atlas.uri.as_deref(),
        Some("../../../effects/shared/textures/spwaneye.png")
    );
    let [level] = atlas.mip_levels.as_deref().unwrap() else {
        panic!("Fusion Eyes atlas must keep its exact base level");
    };
    assert_eq!((level.width, level.height), (128, 512));
    assert_eq!(
        level.png_sha256,
        "b895006b292f14784a902625761cf75b642c2103d3d845605f09bc47851de214"
    );
}

#[test]
fn production_forsaken_valley_materials_preserve_primary_fog_state() {
    let inherited = [
        (
            "../../assets/game/objects/unclassified/wd_dldb_dinosaur_bone_01/\
                 models/ep_dlfl_fusion_bone_01_wd_dldb_dinosaur_bone_01_wd_dldb/visual.glb",
            LegacyShaderKind::AlphaBlendNormal,
        ),
        (
            "../../assets/game/objects/nature/wd_dl_tree_01_standard_2/\
                 models/wd_dl_tree_01_standard_2/visual.glb",
            LegacyShaderKind::TransparentCutoutTwoSided,
        ),
    ];

    for (path, expected_shader) in inherited {
        let pending = production_static_world_pending(path);
        assert_eq!(pending.params.shader, expected_shader, "{path}");
        assert!(
            pending.params.fixed_function_fog,
            "primary shader inherits scene fog: {path}"
        );
        for pass in pending.render_passes {
            let material = pending
                .params
                .material_for_pass(pass, &LegacyModelTextures::default())
                .unwrap();
            assert_eq!(
                material.uniform.legacy_effect.w, 1.0,
                "non-additive inherited pass uses scene fog: {path}"
            );
        }
    }

    let explicit_off = production_static_world_pending(
        "../../assets/game/objects/nature/wd_dl_tree_01/\
             models/wd_dl_tree_01_standard_4/visual.glb",
    );
    assert_eq!(
        explicit_off.params.shader,
        LegacyShaderKind::AdditiveOneOneBackface
    );
    assert!(
        !explicit_off.params.fixed_function_fog,
        "primary normal_blendOneOne_zwriteOff has Fog Off"
    );
    let material = explicit_off
        .params
        .material_for_pass(
            explicit_off.render_passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    assert_eq!(material.uniform.legacy_effect.w, 0.0);
}

pub(super) fn runtime_binding_from_typed(binding: MaterialTextureBinding) -> LegacyGltfTextureBinding {
    LegacyGltfTextureBinding {
        slot: binding.slot,
        texture_index: binding.texture.map(|value| value as usize),
        source_name: binding.source_name,
        uri: binding.uri,
        sampler: binding.sampler,
        mip_provenance: binding.mip_provenance,
        mip_levels: binding.mip_levels,
        color_space: binding.color_space,
        scale: Vec2::new(binding.scale[0] as f32, binding.scale[1] as f32),
        offset: Vec2::new(binding.offset[0] as f32, binding.offset[1] as f32),
        pivot: binding
            .pivot
            .map(|pivot| Vec2::new(pivot[0] as f32, pivot[1] as f32)),
        rotation: binding.rotation.map(|rotation| rotation as f32),
    }
}

#[test]
fn outline_companion_preserves_depth_state_and_orders_after_surface() {
    let outline_mode = LegacyModelRenderPlan::for_shader(LegacyShaderKind::SkinnedToon)
        .outline()
        .unwrap()
        .render_mode;
    let material = LegacyOutlineMaterial {
        uniform: LegacyOutlineUniform {
            color: LinearRgba::BLACK,
            width_fat: Vec4::ZERO,
            fog_params: Vec4::ZERO,
        },
        render_mode: outline_mode,
        sort_bias: legacy_material_sort_bias(2_900, 0, 1).unwrap(),
    };
    let surface = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon)
        .material_for_pass(
            LegacyModelRenderPlan::for_shader(LegacyShaderKind::SkinnedToon).passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    assert_eq!(surface.depth_bias(), 0.0);
    assert_eq!(
        material.depth_bias(),
        legacy_material_sort_bias(2_900, 0, 1).unwrap()
    );
    // Transparent3d distance increases toward the camera, and ascending
    // back-to-front sorting submits the positive-biased companion later.
    assert!(material.depth_bias() > surface.depth_bias());
    assert!(material.render_mode.depth_write);
    assert_eq!(material.render_mode.cull, LegacyCullMode::Front);
}

#[test]
fn src_alpha_additive_two_sided_state_reaches_the_exact_bevy_pipeline() {
    let pass =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::SrcAlphaAdditiveTwoSided).passes[0];
    let mode = pass.render_mode;
    assert_eq!(pass.kind, LegacyPassKind::Surface);
    assert_eq!(pass.alpha_test, LegacyAlphaTest::Disabled);
    assert_eq!(mode.blend, LegacyBlendMode::SrcAlphaOne);
    assert_eq!(mode.cull, LegacyCullMode::Off);
    assert!(!mode.depth_write);
    assert_eq!(mode.color_write, LegacyColorWriteMask::Rgb);
    assert!(!mode.alpha_cutout);
    assert_eq!(mode.source_queue, 3010);

    let expected_component = BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    };
    assert_eq!(
        bevy_blend_state(mode.blend),
        Some(BlendState {
            color: expected_component,
            alpha: expected_component,
        })
    );
    assert_eq!(bevy_cull_mode(mode.cull), None);
    let write_mask = bevy_color_write_mask(mode.color_write);
    assert_eq!(
        write_mask,
        ColorWrites::RED | ColorWrites::GREEN | ColorWrites::BLUE
    );
    assert!(!write_mask.contains(ColorWrites::ALPHA));

    let material =
        LegacyModelMaterialParams::for_shader(LegacyShaderKind::SrcAlphaAdditiveTwoSided)
            .material_for_pass(pass, &LegacyModelTextures::default())
            .unwrap();
    assert!(matches!(material.alpha_mode(), AlphaMode::Blend));
}

#[test]
fn shared_image_interpretation_ignores_sampler_identity_but_not_gpu_state() {
    let typed = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
        "_ShaderMap",
        0,
        "ToonRamp9",
        "../../../../../rendering/textures/toonramp9.png",
        "linear",
    ))
    .unwrap();
    let left = runtime_binding_from_typed(typed.clone());
    let mut renamed = typed;
    renamed.sampler.as_mut().unwrap().index = 7;
    renamed.sampler.as_mut().unwrap().descriptor.name = "ToonRamp9.dds".into();
    let renamed = runtime_binding_from_typed(renamed);
    assert_eq!(
        exact_interpretation_digest(&left).unwrap(),
        exact_interpretation_digest(&renamed).unwrap()
    );

    let mut different_gpu_state = renamed;
    different_gpu_state
        .sampler
        .as_mut()
        .unwrap()
        .descriptor
        .wrap_t = SamplerWrapMode::ClampToEdge;
    assert_ne!(
        exact_interpretation_digest(&left).unwrap(),
        exact_interpretation_digest(&different_gpu_state).unwrap()
    );
}
