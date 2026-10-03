use super::*;

#[test]
fn native_cel_style_preserves_every_surface_pass() {
    for shader in [LegacyShaderKind::OpaqueNormal,LegacyShaderKind::AlphaBlendNormal,LegacyShaderKind::TransparentNormal] {
        let mut params=LegacyModelMaterialParams::for_shader(shader);
        let source=params.render_plan(); params.native_cel_shading=true;
        let plan=params.render_plan(); assert_eq!(plan,source);
        let material=params.material_for_pass(plan.passes[0],&LegacyModelTextures::default()).unwrap();
        assert_eq!(material.uniform.alpha_effect.w,1.0);
    }
}

#[test]
fn cached_static_material_companion_inherits_adaptive_range_immediately() {
    let mut world = World::new();
    let source = world.spawn_empty().id();
    let mesh = Mesh3d(Handle::<Mesh>::default());
    let passes = [
        (
            LegacyPassKind::CutoutDepth,
            Handle::<LegacyModelMaterial>::default(),
        ),
        (
            LegacyPassKind::TransparentColor,
            Handle::<LegacyModelMaterial>::default(),
        ),
    ];
    let tag = MeshTag(0x5aa5_1234);
    let range = VisibilityRange {
        start_margin: 0.0..0.0,
        end_margin: 340.0..341.0,
        use_aabb: true,
    };

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        apply_cached_legacy_static_material(
            &mut commands,
            source,
            &mesh,
            &passes,
            ExactMipChainApplied::default(),
            Some(&tag),
            Some(&range),
            Some(&Visibility::Hidden),
        );
    }
    queue.apply(&mut world);

    let children = world
        .get::<Children>(source)
        .expect("the second material pass must be a source child");
    assert_eq!(children.len(), 1);
    let companion = children[0];
    assert_eq!(world.get::<MeshTag>(companion), Some(&tag));
    let inherited_range = world
        .get::<VisibilityRange>(companion)
        .expect("companion must receive the range in its spawn command");
    assert_eq!(inherited_range.start_margin, range.start_margin);
    assert_eq!(inherited_range.end_margin, range.end_margin);
    assert_eq!(inherited_range.use_aabb, range.use_aabb);
    assert_eq!(
        world.get::<Visibility>(companion),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn foster_interior_retrolit_preserves_opaque_ambient_emissive_material() {
    let pending = production_static_world_pending(
        "../../assets/game/objects/nature/foster_none_forster/models/knd_book_five_heros_streetlight/visual.glb",
    );
    assert_eq!(pending.render_passes.len(), 1);
    let pass = pending.render_passes[0];
    assert_eq!(pass.render_mode.blend, LegacyBlendMode::Replace);
    assert_eq!(pass.render_mode.cull, LegacyCullMode::Back);
    assert_eq!(pass.render_mode.source_queue, 2000);
    assert!(pass.render_mode.depth_write);
    assert_eq!(pass.alpha_test, LegacyAlphaTest::Disabled);
    assert_eq!(
        pending.params.base_color,
        LinearRgba::new(0.35, 0.35, 0.35, 1.0)
    );
    assert_eq!(pending.params.ambient_color, pending.params.base_color);
    assert_eq!(pending.params.emission, LinearRgba::new(0.3, 0.3, 0.3, 1.0));
    assert!(pending.has_exact_ambient_color && pending.has_exact_emission);
    let material = pending
        .params
        .material_for_pass(pass, &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(material.uniform.light_direction_family.w, 6.0);
    assert_eq!(material.uniform.legacy_effect.w, 1.0);
    assert_eq!(material.render_mode.blend, LegacyBlendMode::Replace);
}

#[test]
fn production_quest_symbol_keeps_exact_packed_mask_and_two_pass_shader() {
    let glb_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/shared/effects/models/es866/marker_acquire.glb");
    let glb = std::fs::read(&glb_path).unwrap();
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let document: Value = serde_json::from_slice(&glb[20..20 + json_length]).unwrap();
    let nodes = document["nodes"].as_array().unwrap();
    let marker_index = nodes
        .iter()
        .position(|node| node["name"].as_str() == Some("Marker"))
        .expect("ES866 must retain its animated Marker bone");
    let renderer = nodes
        .iter()
        .find(|node| node["name"].as_str() == Some("marker_acquire_model"))
        .expect("ES866 must retain its separate renderer transform");
    assert!(nodes[marker_index].get("mesh").is_none());
    assert!(nodes[marker_index].get("skin").is_none());
    assert_eq!(renderer["mesh"].as_u64(), Some(0));
    assert_eq!(renderer["skin"].as_u64(), Some(0));
    let channels = document["animations"][0]["channels"].as_array().unwrap();
    for path in ["translation", "rotation", "scale"] {
        assert!(
            channels.iter().any(|channel| {
                channel["target"]["node"].as_u64() == Some(marker_index as u64)
                    && channel["target"]["path"].as_str() == Some(path)
            }),
            "ES866 Marker bone lost its {path} animation channel"
        );
    }

    let pending = production_model_pending(
        "../../assets/game/map/shared/effects/models/es866/marker_acquire.glb",
        "marker_acquire-initialshadinggroup",
    );
    assert_eq!(
        pending.params.shader,
        LegacyShaderKind::HologramSolidAdditive
    );
    assert_eq!(
        pending.params.base_color,
        LinearRgba::new(1.0, 0.859_000_027, 0.067_000_002, 1.0)
    );
    assert_eq!(pending.params.emission_strength, 1.0);
    assert_eq!(pending.params.transparency, 0.850_000_024);
    assert_eq!(pending.params.rim_power, 1.0);
    assert_eq!(pending.params.rim_intensity, 0.25);
    assert_eq!(pending.params.shadow_strength, 0.600_000_024);
    assert_eq!(pending.params.alpha_cutoff, 0.649_999_976);

    let mask = pending
        .texture_bindings
        .iter()
        .find(|binding| binding.slot == "_MaskTex")
        .expect("ES866 must bind its packed RGBA mask");
    assert_eq!(mask.color_space, TextureColorSpace::Linear);
    assert_eq!(mask.source_name.as_deref(), Some("marker_mask"));
    let [level] = mask.mip_levels.as_deref().unwrap() else {
        panic!("ES866 packed mask must preserve its exact base level");
    };
    assert_eq!((level.width, level.height), (64, 64));
    assert_eq!(
        level.png_sha256,
        "ec8bdddf37a1099b2d8ae45dc40e1c3fbb945b2b3fffcd6dda4b5feb4b0a3e72"
    );

    let plan = pending.params.render_plan();
    let [base, overlays] = plan.passes.as_slice() else {
        panic!("HologramSolid_Additive must retain BASE and OVERLAYS passes");
    };
    assert_eq!(
        base.render_mode.blend,
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
    );
    assert_eq!(overlays.render_mode.blend, LegacyBlendMode::SrcAlphaOne);
    assert!(base.render_mode.depth_write);
    assert!(overlays.render_mode.depth_write);
    assert_eq!(base.render_mode.cull, LegacyCullMode::Off);
    assert_eq!(overlays.render_mode.cull, LegacyCullMode::Off);
    assert_eq!(base.render_mode.source_queue, 2900);
    assert_eq!(overlays.render_mode.source_queue, 2900);
}

#[test]
fn legacy_static_world_v1_material_extras_use_native_rendering() {
    let extras = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:1713",
        "shaderName": "normal_blendSrcalphaInvsrcalpha_vertexColorAD",
        "exactTextureSlots": [{
            "slot": 0,
            "name": "_MainTex",
            "textureId": "CustomAssetBundle-example:324",
            "scale": {"x": 2.0, "y": 3.0},
            "offset": {"x": 0.25, "y": 0.5},
            "pivot": {"x": 0.5, "y": 0.5},
            "rotation": 37.0
        }],
        "legacyRenderState": {}
    })
    .to_string();

    let pending =
        PendingLegacyStaticWorldMaterial::from_gltf_extras(Some("garden wall"), &extras)
            .unwrap();
    assert_eq!(pending.true_name, "garden wall");
    assert_eq!(pending.params.shader, LegacyShaderKind::AlphaBlendNormal);
    assert_eq!(pending.params.uv_scale, Vec2::new(2.0, 3.0));
    assert_eq!(pending.params.uv_offset, Vec2::new(0.25, 0.5));
    assert_eq!(pending.params.uv_pivot, Vec2::splat(0.5));
    assert_eq!(pending.params.uv_rotation_degrees, 37.0);
    assert_eq!(pending.render_passes.len(), 1);
    assert!(!pending.has_exact_base_color);
    assert!(!pending.has_exact_ambient_color);
    assert!(!pending.has_exact_emission);
}

#[test]
fn repeated_material_extras_are_parsed_once_without_losing_name_identity() {
    let extras = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:1713",
        "shaderName": "normal_blendSrcalphaInvsrcalpha_vertexColorAD",
        "legacyRenderState": {}
    })
    .to_string();
    let mut cache = LegacyMaterialExtrasParseCache::default();

    let first = cache.parse(Some("garden wall"), &extras);
    let second = cache.parse(Some("garden wall"), &extras);

    assert_eq!(first, second);
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(
        cache.misses, 1,
        "the duplicate placement must be a cache hit"
    );

    let renamed = cache.parse(Some("garden wall variant"), &extras);
    assert_ne!(first, renamed);
    assert_eq!(cache.entries.len(), 2);
    assert_eq!(cache.misses, 2, "material name is part of exact identity");
}

#[test]
fn infected_water_keeps_its_exact_material_identity_and_source_values() {
    let extras = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:1726",
        "shaderName": "ffWater",
        "colors": [
            {"name": "_Color", "value": {"r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0}},
            {"name": "WaveSpeed", "value": {"r": -5.0, "g": -19.0, "b": 16.0, "a": -7.0}},
            {"name": "_RefrColor", "value": {"r": 0.689781, "g": 0.85, "b": 0.0, "a": 1.0}},
            {"name": "_HorizonColor", "value": {"r": 0.4489051, "g": 0.6386861, "b": 0.0, "a": 1.0}},
            {"name": "_FoamColor", "value": {"r": 0.8430762, "g": 0.8430762, "b": 0.8430762, "a": 0.0}}
        ],
        "floats": [
            {"name": "_WaveScale", "value": 0.013650944},
            {"name": "_ReflDistort", "value": 0.44},
            {"name": "_RefrDistort", "value": 0.4}
        ],
        "exactTextureSlots": [
            {"name": "_BumpMap", "textureId": "bundle:93"},
            {"name": "_Fresnel", "textureId": "bundle:350"},
            {"name": "_ReflectiveColor", "textureId": "bundle:306"}
        ],
        "runtimeTextureContract": {
            "schema": "ffone.legacy-water-texture-bindings.v1",
            "baseColorTexture": "_ReflectiveColor",
            "normalTexture": "_BumpMap",
            "emissiveTexture": "_Fresnel"
        }
    })
    .to_string();

    let pending =
        PendingLegacyWaterMaterial::from_gltf_extras(Some("ffPoison"), &extras).unwrap();
    assert_eq!(pending.true_name, "ffPoison");
    assert_eq!(pending.source_material_id, "CustomAssetBundle-example:1726");
    assert_eq!(
        pending.uniform.wave_speed,
        Vec4::new(-5.0, -19.0, 16.0, -7.0)
    );
    assert!((pending.uniform.water_params.x - 0.013650944).abs() <= f32::EPSILON);
    assert_eq!(pending.uniform.water_params.w, 1.0);
    assert_eq!(pending.uniform.horizon_color.blue, 0.0);
    assert_eq!(
        LEGACY_WATER_RENDER_MODE.cull,
        LegacyCullMode::Back,
        "asset-level winding repair restores authored Back culling for static water"
    );
}

#[test]
fn every_exported_capture_static_normal_shader_has_a_compatibility_plan() {
    let shader_names = [
        "normal",
        "normal_blendSrcalphaInvsrcalpha",
        "normal_blendSrcalphaInvsrcalpha_vertexColorAD",
        "normal_blendSrcalphaInvsrcalpha_cullOff",
        "normal_blendSrcalphaInvsrcalpha_cullOff_vertexColorAD",
        "normal_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD",
        "normal_blendOneOne_zwriteOff_cullOff",
        "normal_blendOneOne_zwriteOff_cullOff_vertexColorAD",
        "normal_blendSrcalphaInvsrcalphaTest_cullOff",
        "normal_blendSrcalphaOne_zwriteOff_cullOff",
        "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD",
        "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff",
        "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff_vertexColorAD",
        "normal_blendSrcalphaInvsrcalpha_zwriteOff",
        "normal_blendSrcalphaInvsrcalphaTest",
        "normal_blendOneOneTest",
        "normal_blendOneOneTest_cullOff",
        "normal_blendOneOne_zwriteOff",
        "normal_blendOneOne_zwriteOff_vertexColorAD",
        "normal_blendSrcalphaOne_zwriteOff",
        "normal_blendSrcalphaOne_zwriteOff_vertexColorAD",
        "normal_blendSrcalphaInvsrccolor",
        "normal_blendSrcalphaOne",
        "normal_blendSrcalphaOne_vertexColorAD",
        "normal_blendOneOne",
        "normal_blendOneOne_vertexColorAD",
        "normal_blendOneOne_cullOff",
        "normal_blendSrcalphaInvsrcalphaTest_zwriteOff_cullOff",
        "normal_blendSrcalphaZero",
        "normal_transparentcutoutinfrontofWater",
        "normal_glow_blendSrcalphaInvsrcalpha",
        "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff",
        "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD",
        "normal_glow_blendOneOne_zwriteOff_vertexColorAD",
    ];
    for shader_name in shader_names {
        let (shader, passes) = legacy_static_shader_plan(shader_name)
            .unwrap_or_else(|error| panic!("{shader_name}: {error}"));
        assert!(!passes.is_empty(), "{shader_name}");
        assert_ne!(shader, LegacyShaderKind::FusionEffect);
        assert!(
            passes
                .iter()
                .all(|pass| pass.kind != LegacyPassKind::Outline),
            "{shader_name}"
        );
    }
    assert!(legacy_static_shader_plan("ffWater").is_err());
}

#[test]
fn source_alpha_zero_static_blend_reaches_webgpu_exactly() {
    let (shader, passes) = legacy_static_shader_plan("normal_blendSrcalphaZero").unwrap();
    assert_eq!(shader, LegacyShaderKind::SrcAlphaZeroBackfaceDepthWrite);
    let [pass] = passes.as_slice() else {
        panic!("normal_blendSrcalphaZero must remain a single-pass program");
    };
    assert_eq!(pass.render_mode.blend, LegacyBlendMode::SrcAlphaZero);
    assert_eq!(pass.render_mode.cull, LegacyCullMode::Back);
    assert!(pass.render_mode.depth_write);
    assert_eq!(pass.render_mode.color_write, LegacyColorWriteMask::Rgba);
    assert_eq!(pass.render_mode.source_queue, 3000);

    let blend = bevy_blend_state(pass.render_mode.blend).unwrap();
    assert_eq!(blend.color.src_factor, BlendFactor::SrcAlpha);
    assert_eq!(blend.color.dst_factor, BlendFactor::Zero);
    assert_eq!(blend.alpha.src_factor, BlendFactor::SrcAlpha);
    assert_eq!(blend.alpha.dst_factor, BlendFactor::Zero);
}

#[test]
fn inverse_source_color_static_blend_reaches_webgpu_exactly() {
    let (_, passes) = legacy_static_shader_plan("normal_blendSrcalphaInvsrccolor").unwrap();
    let [pass] = passes.as_slice() else {
        panic!("inverse-source-color shader must have one pass");
    };
    let blend = bevy_blend_state(pass.render_mode.blend).unwrap();
    assert_eq!(blend.color.src_factor, BlendFactor::SrcAlpha);
    assert_eq!(blend.color.dst_factor, BlendFactor::OneMinusSrc);
    assert_eq!(blend.alpha.src_factor, BlendFactor::SrcAlpha);
    assert_eq!(blend.alpha.dst_factor, BlendFactor::OneMinusSrc);
}

#[test]
fn enriched_static_world_material_extras_preserve_exact_lighting_colors() {
    let extras = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:1631",
        "shaderName": "normal_blendSrcalphaInvsrcalpha_cullOff",
        "colors": [
            {"name": "_Color", "value": {"r": 0.294, "g": 0.294, "b": 0.294, "a": 1.0}},
            {"name": "_Emission", "value": {"r": 0.235, "g": 0.235, "b": 0.235, "a": 1.0}},
            {"name": "_AmbColor", "value": {"r": 0.294, "g": 0.294, "b": 0.294, "a": 1.0}}
        ],
        "floats": [{"name": "_Cutoff", "value": 0.4}],
        "exactTextureSlots": []
    })
    .to_string();

    let pending = PendingLegacyStaticWorldMaterial::from_gltf_extras(None, &extras).unwrap();
    assert_eq!(
        pending.params.shader,
        LegacyShaderKind::AlphaBlendNormalCullOff
    );
    assert!(pending.has_exact_base_color);
    assert!(pending.has_exact_ambient_color);
    assert!(pending.has_exact_emission);
    assert_eq!(pending.render_passes.len(), 1);
    assert!((pending.params.ambient_color.red - 0.294).abs() < 1.0e-6);
    assert!((pending.params.emission.red - 0.235).abs() < 1.0e-6);
    assert!((pending.params.alpha_cutoff - 0.4).abs() < 1.0e-6);
}

#[test]
fn ship_vortex_accepts_exact_source_color_blend_without_depth_writes() {
    let pending = production_static_world_pending(
        "../../assets/game/objects/effects/etc_blackhole_standard_1_blackhole2/models/etc_blackhole_standard_1_blackhole2/visual.glb",
    );
    assert_eq!(pending.render_passes.len(), 1);
    let pass = pending.render_passes[0];
    assert_eq!(
        pass.render_mode.blend,
        LegacyBlendMode::SrcColorOneMinusSrcAlpha
    );
    assert_eq!(pass.render_mode.cull, LegacyCullMode::Off);
    assert_eq!(pass.render_mode.color_write, LegacyColorWriteMask::Rgb);
    assert!(!pass.render_mode.depth_write);
    assert_eq!(pass.render_mode.source_queue, 3010);
    assert!(pending.params.fixed_function_fog);
    let gpu = bevy_blend_state(pass.render_mode.blend).unwrap();
    assert_eq!(gpu.color.src_factor, BlendFactor::Src);
    assert_eq!(gpu.color.dst_factor, BlendFactor::OneMinusSrcAlpha);
    // Every camera direction within the supported view range preserves
    // queue 3010 before 3011, even at opposite ends of the view volume.
    for depth in [-1000.0, 0.0, 1000.0] {
        assert!(
            legacy_render_queue_sort_bias(3010) + depth
                < legacy_render_queue_sort_bias(3011) - depth
        );
    }
}

#[test]
fn exact_normal_glow_keeps_its_rgb_only_fixed_function_pass() {
    let params =
        LegacyModelMaterialParams::from_shader_name("normal_glow_blendSrcalphaInvsrcalpha")
            .unwrap();
    assert_eq!(params.shader, LegacyShaderKind::AlphaBlendNormalGlow);
    assert!(params.glow_mask);
    let render_plan = params.render_plan();
    assert_eq!(render_plan.passes.len(), 1);
    assert_eq!(
        render_plan.passes[0].render_mode.color_write,
        LegacyColorWriteMask::Rgb
    );
    let canonical = canonical_expected_passes(&params);
    assert_eq!(canonical.len(), 1);
    assert_eq!(canonical[0].color_mask, 0b0111);
}

#[test]
fn exact_shader_names_classify_without_fuzzy_suffixes() {
    let exact = [
        (
            "SkinnedToonShading_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::SkinnedToon,
        ),
        (
            "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff",
            LegacyShaderKind::SkinnedToonCullOffFallback,
        ),
        (
            "ToonShading_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::Toon,
        ),
        (
            "Skin_DirLight_AmbLight_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::SkinDirectionalAlphaBlend,
        ),
        ("normal", LegacyShaderKind::OpaqueNormal),
        (
            "normal_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::AlphaBlendNormal,
        ),
        (
            "normal_glow_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::AlphaBlendNormalGlow,
        ),
        (
            "normal_blendSrcalphaInvsrcalpha_vertexColorAD",
            LegacyShaderKind::AlphaBlendNormalVertexColorAd,
        ),
        (
            "normal_blendSrcalphaInvsrcalpha_cullOff",
            LegacyShaderKind::AlphaBlendNormalCullOff,
        ),
        (
            "normal_blendSrcalphaInvsrcalpha_zwriteOff",
            LegacyShaderKind::TransparentNormal,
        ),
        (
            "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff",
            LegacyShaderKind::TransparentNormalCullOff,
        ),
        (
            "normal_blendSrcalphaZero",
            LegacyShaderKind::SrcAlphaZeroBackfaceDepthWrite,
        ),
        (
            "normal_blendSrcalphaOne_zwriteOff",
            LegacyShaderKind::SrcAlphaAdditiveBackface,
        ),
        (
            "normal_blendSrcalphaOne",
            LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite,
        ),
        (
            "normal_blendSrcalphaOne_zwriteOff_cullOff",
            LegacyShaderKind::SrcAlphaAdditiveTwoSided,
        ),
        (
            "normal_blendOneOneTest_cullOff",
            LegacyShaderKind::AdditiveTwoSided,
        ),
        (
            "normal_blendOneOneTest_zwriteOff_cullOff",
            LegacyShaderKind::AdditiveTestTwoSidedQueue3011,
        ),
        (
            "normal_blendOneOne_zwriteOff_cullOff",
            LegacyShaderKind::AdditiveOneOneTwoSided,
        ),
        (
            "normal_blendOneOne_zwriteOff_cullOff_vertexColorAD",
            LegacyShaderKind::AdditiveOneOneTwoSidedVertexColorAd,
        ),
        (
            "normal_blendOneOne_cullOff",
            LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite,
        ),
        (
            "normal_blendOneOne_zwriteOff",
            LegacyShaderKind::AdditiveOneOneBackface,
        ),
        (
            "normal_blendOneOne",
            LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite,
        ),
        (
            "normal_blendSrcalphaInvsrcalphaTest",
            LegacyShaderKind::TransparentCutoutDefaultCulling,
        ),
        (
            "normal_blendSrcalphaInvsrcalphaTest_cullOff",
            LegacyShaderKind::TransparentCutoutTwoSided,
        ),
        (
            "Skin_FusionEffect_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::FusionEffect,
        ),
        (
            "SkinnedFusionMatterLightDir_blendSrcalphaInvsrcalpha",
            LegacyShaderKind::FusionMatterLightDir,
        ),
        (
            "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_Rim_transparent",
            LegacyShaderKind::SkinnedToonRimTransparent,
        ),
        ("retro_diffuseFade", LegacyShaderKind::DiffuseFade),
    ];
    for (name, kind) in exact {
        assert_eq!(LegacyShaderKind::classify_exact(name), Ok(kind));
        assert_eq!(kind.exact_name(), name);
    }
    assert_eq!(
        LegacyShaderKind::classify_exact("SkinnedToonShading_blendSrcalphaInvsrcalpha e1"),
        Ok(LegacyShaderKind::SkinnedToon)
    );
    assert!(
        LegacyShaderKind::classify_exact("normal_blendOneOneTest_cullOff_vertexColorAD")
            .is_err()
    );
    assert_eq!(
        LegacyShaderKind::classify_exact(
            "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD"
        ),
        Ok(LegacyShaderKind::SrcAlphaAdditiveTwoSidedVertexColorAd)
    );
    assert!(
        LegacyShaderKind::classify_exact(
            "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD_unverified"
        )
        .is_err()
    );
    assert!(
        LegacyShaderKind::classify_exact(
            "Skin_DirLight_AmbLight_blendSrcalphaInvsrcalpha_cullOff"
        )
        .is_err()
    );
}

#[test]
fn production_hippie_hop_retains_tutorial_shader_black_defaults() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/characters/npcs/npc_hippiehop/npc_hippiehop.glb");
    let bytes = std::fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let document: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    for material in document["materials"].as_array().unwrap() {
        let metadata = &material["extras"]["ffone"];
        let name = metadata["declaredShaderName"].as_str().unwrap();
        if metadata["name"] == "npc_hippiehop-sub_link_a.dds" {
            assert_eq!(name, "normal_blendOneOne_zwriteOff_cullOff");
            continue;
        }
        assert_eq!(name, "Skin_FusionEffect_blendSrcalphaInvsrcalpha_tutorial");
        assert_eq!(
            LegacyShaderKind::classify_exact(name),
            Ok(LegacyShaderKind::FusionEffect)
        );
        for slot in ["_MainTex", "_BumpMap", "_ShaderMap"] {
            let defaults = metadata["shaderTextureDefaults"].as_array().unwrap();
            assert!(
                defaults
                    .iter()
                    .any(|p| p["slot"] == slot && p["value"] == "builtinBlack")
            );
        }
    }
}

#[test]
fn diffuse_fade_uses_primary_time_pod_render_state() {
    let plan = LegacyModelRenderPlan::for_shader(LegacyShaderKind::DiffuseFade);
    assert_eq!(plan.passes.len(), 1);
    let pass = plan.passes[0];
    assert_eq!(pass.kind, LegacyPassKind::Surface);
    assert_eq!(
        pass.render_mode.blend,
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
    );
    assert_eq!(pass.render_mode.cull, LegacyCullMode::Back);
    assert!(pass.render_mode.depth_write);
    assert_eq!(pass.render_mode.color_write, LegacyColorWriteMask::Rgb);
    assert_eq!(pass.render_mode.source_queue, 2900);
}

#[test]
fn toon_plan_keeps_outline_as_a_real_pass() {
    for kind in [
        LegacyShaderKind::SkinnedToon,
        LegacyShaderKind::SkinnedToonCullOffFallback,
        LegacyShaderKind::SkinnedToonCullOffCategory,
        LegacyShaderKind::FusionMatterLightDir,
        LegacyShaderKind::Toon,
    ] {
        let plan = LegacyModelRenderPlan::for_shader(kind);
        assert_eq!(plan.passes.len(), 2);
        assert_eq!(
            plan.passes[0].render_mode.cull,
            if kind == LegacyShaderKind::SkinnedToonCullOffCategory {
                LegacyCullMode::Off
            } else {
                LegacyCullMode::Back
            }
        );
        assert!(plan.passes[0].render_mode.depth_write);
        let outline = plan.outline().unwrap();
        assert_eq!(outline.render_mode.cull, LegacyCullMode::Front);
        assert!(outline.render_mode.depth_write);
    }
}

#[test]
fn recovered_tutorial_shader_plans_keep_their_exact_source_state() {
    let opaque = LegacyModelRenderPlan::for_shader(LegacyShaderKind::OpaqueNormal).passes[0];
    assert_eq!(opaque.render_mode.blend, LegacyBlendMode::Replace);
    assert_eq!(opaque.render_mode.cull, LegacyCullMode::Back);
    assert!(opaque.render_mode.depth_write);
    assert_eq!(opaque.render_mode.source_queue, 2000);

    let alpha_two_sided =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AlphaBlendNormalCullOff).passes[0];
    assert_eq!(
        alpha_two_sided.render_mode.blend,
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
    );
    assert_eq!(alpha_two_sided.render_mode.cull, LegacyCullMode::Off);
    assert!(alpha_two_sided.render_mode.depth_write);
    assert_eq!(alpha_two_sided.render_mode.source_queue, 3000);

    let cutout =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::TransparentCutoutDefaultCulling);
    assert_eq!(cutout.passes.len(), 2);
    assert_eq!(cutout.passes[0].render_mode.cull, LegacyCullMode::Back);
    assert_eq!(
        cutout.passes[0].alpha_test,
        LegacyAlphaTest::GreaterEqualLiteralNineTenths
    );
    assert!(cutout.passes[0].render_mode.depth_write);
    assert_eq!(
        cutout.passes[1].render_mode.blend,
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
    );
    assert!(!cutout.passes[1].render_mode.depth_write);
    assert_eq!(cutout.passes[1].render_mode.cull, LegacyCullMode::Back);
}

#[test]
fn actor_skin_renderer_order_nests_shader_pass_order() {
    let skin_surface = legacy_material_sort_bias(2_900, 0, 0).unwrap();
    let skin_outline = legacy_material_sort_bias(2_900, 0, 1).unwrap();
    let face_overlay = legacy_material_sort_bias(2_900, 1, 0).unwrap();
    let fusion_eye = legacy_material_sort_bias(3_000, 2, 0).unwrap();
    assert_eq!(skin_surface, 0.0);
    assert!(skin_surface < skin_outline);
    assert!(
        skin_outline < face_overlay,
        "M_Face:0 surface/outline must both precede M_Face:1 overlay"
    );
    assert!(
        face_overlay < fusion_eye,
        "the typed 3000 Fusion eye queue must follow the 2900 face overlay"
    );
    assert!(
        fusion_eye - face_overlay > 12.167,
        "the queue transition must dominate the audited character renderer-origin drift"
    );
    assert_eq!(
        legacy_material_sort_bias(2_900, 0, LEGACY_MATERIAL_PASS_STRIDE as usize,),
        None,
        "an unaudited shader pass count must fail closed"
    );
}

#[test]
fn malformed_flapjack_fusion_material_uses_texture2_semantic_fallback() {
    assert_eq!(
        legacy_npc_texture_role(
            "back_fusiontentacles-main-link_a.dds",
            LegacyShaderKind::FusionEffect,
            true,
            true,
        ),
        Some(LegacyNpcTextureRole::Sub)
    );
    assert_eq!(
        legacy_npc_texture_role(
            "fusion_mac-main-link_a.dds",
            LegacyShaderKind::SkinnedToon,
            true,
            true,
        ),
        Some(LegacyNpcTextureRole::Main)
    );
    assert_eq!(
        legacy_npc_texture_role(
            "fusion_mac-sub-link_b.dds",
            LegacyShaderKind::FusionEffect,
            true,
            true,
        ),
        Some(LegacyNpcTextureRole::Sub)
    );
}

#[test]
fn transparent_cutout_is_not_collapsed_to_one_pass() {
    let plan = LegacyModelRenderPlan::for_shader(LegacyShaderKind::TransparentCutoutTwoSided);
    assert_eq!(plan.passes.len(), 2);
    let depth = plan.passes[0];
    assert_eq!(depth.kind, LegacyPassKind::CutoutDepth);
    assert_eq!(depth.render_mode.blend, LegacyBlendMode::Replace);
    assert!(depth.render_mode.alpha_cutout && depth.render_mode.depth_write);
    assert_eq!(
        depth.alpha_test,
        LegacyAlphaTest::GreaterEqualLiteralNineTenths
    );
    let color = plan.passes[1];
    assert_eq!(color.kind, LegacyPassKind::TransparentColor);
    assert!(!color.render_mode.depth_write);
    assert_eq!(color.render_mode.color_write, LegacyColorWriteMask::Rgb);
}
