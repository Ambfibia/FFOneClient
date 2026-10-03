use super::*;

pub(super) fn production_static_world_pending(
    relative_glb_path: &str,
) -> PendingLegacyStaticWorldMaterial {
    let glb_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_glb_path);
    let bytes = std::fs::read(&glb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", glb_path.display()));
    assert_eq!(&bytes[0..4], b"glTF");
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    assert_eq!(&bytes[16..20], b"JSON");
    let document: Value = serde_json::from_slice(&bytes[20..20 + json_length]).unwrap();
    let materials = document["materials"].as_array().unwrap();
    assert_eq!(materials.len(), 1, "{}", glb_path.display());
    let source_material = &materials[0];
    let material_name = source_material["name"].as_str().unwrap();
    let extras = serde_json::to_string(&source_material["extras"]).unwrap();
    PendingLegacyStaticWorldMaterial::from_gltf_extras(Some(material_name), &extras).unwrap()
}

#[test]
fn foster_retrolit_admission_rejects_unimplemented_specular_and_alpha() {
    let mut extras = serde_json::json!({
        "ffoneSourceMaterialId": "test/interior",
        "shaderName": "RetroLit",
        "colors": [
            {"name": "_Color", "value": {"r": 0.35, "g": 0.35, "b": 0.35, "a": 1.0}},
            {"name": "_SpecColor", "value": {"r": 0.0, "g": 0.0, "b": 0.0, "a": 1.0}}
        ]
    });
    let parse = |extras: &Value| {
        PendingLegacyStaticWorldMaterial::from_gltf_extras(None, &extras.to_string())
    };
    let pending = parse(&extras).unwrap();
    assert_eq!(pending.params.emission, LinearRgba::new(0.0, 0.0, 0.0, 0.0));
    assert!(
        pending.has_exact_emission,
        "shader default must bypass compatibility emission"
    );
    extras["colors"][1]["value"]["r"] = serde_json::json!(0.1);
    assert!(parse(&extras).is_err());
    extras["colors"][1]["value"]["r"] = serde_json::json!(0.0);
    extras["colors"][0]["value"]["a"] = serde_json::json!(0.5);
    assert!(parse(&extras).is_err());
    extras["colors"] = serde_json::json!([]);
    assert!(parse(&extras).is_err(), "source specular defaults to white");
}

#[test]
fn production_retro_fusion_matter_ignores_stale_pre_replacement_colors() {
    let pending = production_model_pending(
        "../../assets/game/characters/mobs/mob_cerberus/mob_cerberus.glb",
        "mob_cerberus-sub-link_b.dds",
    );
    assert_eq!(
        pending.params.shader,
        LegacyShaderKind::FusionMatterLightDir
    );
    assert_eq!(
        pending.params.tint_color,
        LinearRgba::new(0.0, 1.0, 0.0, 1.0)
    );
    assert_eq!(
        pending.params.ambient_color,
        LinearRgba::new(1.0, 0.976, 0.208, 1.0)
    );
    assert_eq!(
        pending.params.emission,
        LinearRgba::new(0.529, 0.576, 1.0, 1.0)
    );
    assert_eq!(pending.params.shadow_strength, 0.65);
    assert_eq!(pending.params.outline_width, 0.005);
}

#[test]
fn skinned_meshes_disable_invalid_static_aabb_frustum_culling() {
    let mut app = App::new();
    app.add_systems(Update, disable_static_frustum_culling_for_skinned_meshes);
    let skinned = app.world_mut().spawn(SkinnedMesh::default()).id();
    let rigid = app.world_mut().spawn_empty().id();

    app.update();

    assert!(app.world().get::<NoFrustumCulling>(skinned).is_some());
    assert!(app.world().get::<NoFrustumCulling>(rigid).is_none());
}

#[test]
fn skinned_frustum_policy_is_idempotent() {
    let mut app = App::new();
    app.add_systems(Update, disable_static_frustum_culling_for_skinned_meshes);
    let skinned = app
        .world_mut()
        .spawn((SkinnedMesh::default(), NoFrustumCulling))
        .id();

    app.update();
    app.update();

    assert!(app.world().get::<NoFrustumCulling>(skinned).is_some());
}

#[test]
fn production_past_pokey_pool_trim_uses_exact_source_alpha_zero_contract() {
    let path = "../../assets/game/objects/structures/sb_sbpg_tx/models/\
                    sb_sbpg_ppg_house_01_02_default222_variant_0003/visual.glb";
    let pending = production_static_world_pending(path);
    assert_eq!(
        pending.source_material_id,
        "CustomAssetBundle-e5a53b272630f4113ac7dfa756eae499:199"
    );
    assert_eq!(
        pending.base_texture_id.as_deref(),
        Some("CustomAssetBundle-e5a53b272630f4113ac7dfa756eae499:3")
    );
    assert_eq!(
        pending.params.shader,
        LegacyShaderKind::SrcAlphaZeroBackfaceDepthWrite
    );
    assert!(pending.params.fixed_function_fog);
    assert_eq!(pending.render_passes.len(), 1);
    assert_eq!(
        pending.render_passes[0].render_mode.blend,
        LegacyBlendMode::SrcAlphaZero
    );
    assert!(pending.render_passes[0].render_mode.depth_write);
}

#[test]
fn toon_surfaces_and_outlines_share_primary_fog_contract() {
    for shader_name in [
        "ToonShading_blendSrcalphaInvsrcalpha",
        "SkinnedToonShading_blendSrcalphaInvsrcalpha",
        "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff",
    ] {
        let params = LegacyModelMaterialParams::from_shader_name(shader_name).unwrap();
        assert!(params.fixed_function_fog, "{shader_name}");

        let surface_pass = params
            .render_plan()
            .passes
            .into_iter()
            .find(|pass| pass.kind == LegacyPassKind::Surface)
            .unwrap();
        let surface = params
            .material_for_pass(surface_pass, &LegacyModelTextures::default())
            .unwrap();
        assert_eq!(surface.uniform.legacy_effect.w, 1.0, "{shader_name}");

        let outline = params.outline_material().unwrap();
        assert_eq!(outline.uniform.fog_params.x, 1.0, "{shader_name}");
    }
}

#[test]
fn production_epbarrier_generator_preserves_exact_encoded_base_color() {
    let glb_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../assets/game/objects/effects/epbarrier1_03_default_forcefieldgenerator/\
             models/epbarrier1_03_default_forcefieldgenerator/visual.glb",
    );
    let bytes = std::fs::read(&glb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", glb_path.display()));
    assert_eq!(&bytes[0..4], b"glTF");
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    assert_eq!(&bytes[16..20], b"JSON");
    let document: Value = serde_json::from_slice(&bytes[20..20 + json_length]).unwrap();
    let source_material = &document["materials"][0];
    let material_name = source_material["name"].as_str().unwrap();
    let extras = serde_json::to_string(&source_material["extras"]).unwrap();
    let pending =
        PendingLegacyStaticWorldMaterial::from_gltf_extras(Some(material_name), &extras)
            .unwrap();

    assert_eq!(
        pending.source_material_id,
        "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a:1791"
    );
    assert_eq!(pending.params.shader, LegacyShaderKind::AlphaBlendNormal);
    assert!(pending.params.fixed_function_fog);
    assert!(pending.has_exact_base_color);
    assert!((pending.params.base_color.red - 0.235_294_13).abs() < 1.0e-7);
    assert!(pending.has_exact_ambient_color);
    assert!(pending.has_exact_emission);
    assert!((pending.params.ambient_color.red - 0.235_294_13).abs() < 1.0e-7);
    assert!((pending.params.emission.red - 0.294_117_66).abs() < 1.0e-7);

    let pass = pending.render_passes[0];
    let material = pending
        .params
        .material_for_pass(pass, &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(pending.params.shader.shading_family(), 6.0);
    assert_eq!(material.uniform.light_direction_family.w, 6.0);
    assert_eq!(material.uniform.legacy_effect.w, 1.0);
    assert_eq!(material.uniform.ambient_color, pending.params.ambient_color);
    let ambient_primary = material.uniform.ambient_color.red * 0.2;
    assert!((ambient_primary - 0.047_058_83).abs() < 1.0e-7);
    let wgsl = include_str!("../legacy_model_base.wgsl");
    assert!(wgsl.contains("const LEGACY_GLOBAL_AMBIENT: f32 = 0.2;"));
    assert!(wgsl.contains(
        "material.light_direction_family.w > 5.5 && material.light_direction_family.w < 6.5"
    ));
    assert!(wgsl.contains("let view_position = view.view_from_world * world_position;"));
    assert!(wgsl.contains("1.0 - exp(-(density_depth * density_depth))"));
    assert!(wgsl.contains("linear_to_srgb(fog_color)"));
    assert!(wgsl.contains("let fog_color = fog.base_color.rgb;"));
    assert_eq!(LegacyShaderKind::OpaqueNormal.shading_family(), 0.0);
    assert_eq!(
        LegacyShaderKind::AlphaBlendNormalVertexColorAd.shading_family(),
        0.0
    );
    assert_eq!(LegacyShaderKind::RotatingFlipbook.shading_family(), 0.0);

    let pbr_factor = source_material["pbrMetallicRoughness"]["baseColorFactor"]
        .as_array()
        .unwrap();
    let standard_base_color = Color::srgba(
        pbr_factor[0].as_f64().unwrap() as f32,
        pbr_factor[1].as_f64().unwrap() as f32,
        pbr_factor[2].as_f64().unwrap() as f32,
        pbr_factor[3].as_f64().unwrap() as f32,
    )
    .to_linear();
    assert!(
        (standard_base_color.red - pending.params.base_color.red).abs() > 0.1,
        "the glTF fallback is in a different color domain and must not replace exact ShaderLab values"
    );

    let authored = pending.params.base_color;
    let mut resolved = pending.params.clone();
    apply_static_world_base_color_fallback(
        &mut resolved,
        standard_base_color,
        pending.has_exact_base_color,
    );
    assert_eq!(resolved.base_color, authored);
}

#[test]
fn production_epbarrier_static_adapter_preserves_source_queue_order() {
    let cases = [
        (
            "../../assets/game/objects/effects/epbarrier1_03_default_forcefieldgenerator/models/epbarrier1_03_default_forcefieldgenerator/visual.glb",
            "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a:1791",
            3_000,
            204_800.0,
        ),
        (
            "../../assets/game/objects/vehicles/motttt/models/plane01/visual.glb",
            "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a:1690",
            3_010,
            225_280.0,
        ),
        (
            "../../assets/game/objects/effects/epbarrier1_03_defaultsd_forcefieldgenerator1/models/epbarrier1_03_defaultsd_forcefieldgenerator1/visual.glb",
            "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a:1798",
            3_011,
            227_328.0,
        ),
    ];

    let mut app = App::new();
    app.insert_resource(Assets::<StandardMaterial>::default())
        .insert_resource(Assets::<Image>::default())
        .insert_resource(Assets::<LegacyModelMaterial>::default())
        .add_systems(Update, apply_legacy_static_world_materials);

    let ring = production_static_world_pending(
        "../../assets/game/objects/vehicles/motttt/models/plane01/visual.glb",
    );
    assert!(
        ring.params.fixed_function_fog,
        "the ring shader inherits primary fixed-function fog"
    );
    let ring_material = ring
        .params
        .material_for_pass(ring.render_passes[0], &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(
        ring_material.uniform.legacy_effect.w, 2.0,
        "additive EPbarrier fog must fade toward black, not inject the infected-zone fog color"
    );
    let center = production_static_world_pending(
        "../../assets/game/objects/effects/epbarrier1_03_defaultsd_forcefieldgenerator1/models/epbarrier1_03_defaultsd_forcefieldgenerator1/visual.glb",
    );
    assert!(
        !center.params.fixed_function_fog,
        "the center shader explicitly declares Fog Mode Off"
    );

    let mut expected = Vec::new();
    for (path, source_material_id, source_queue, expected_bias) in cases {
        let pending = production_static_world_pending(path);
        assert_eq!(pending.source_material_id, source_material_id);
        assert_eq!(pending.render_passes.len(), 1);
        assert_eq!(
            pending.render_passes[0].render_mode.source_queue,
            source_queue
        );
        assert_eq!(
            legacy_static_world_sort_bias(source_queue, 0),
            Some(expected_bias)
        );

        let standard_handle = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let entity = app
            .world_mut()
            .spawn((
                Mesh3d(Handle::<Mesh>::default()),
                MeshMaterial3d(standard_handle),
                pending,
            ))
            .id();
        expected.push((entity, source_queue, expected_bias));
    }

    app.update();

    let mut actual_biases = Vec::new();
    for (entity, source_queue, expected_bias) in expected {
        let handle = app
            .world()
            .get::<MeshMaterial3d<LegacyModelMaterial>>(entity)
            .unwrap()
            .0
            .clone();
        let materials = app.world().resource::<Assets<LegacyModelMaterial>>();
        let material = materials.get(&handle).unwrap();
        assert_eq!(material.sort_bias, expected_bias);
        if source_queue == 3_000 {
            // The opaque-equivalent optimization changes the pipeline
            // queue, but ordering still comes from authored q3000.
            assert_eq!(material.render_mode.source_queue, 2_000);
        } else {
            assert_eq!(material.render_mode.source_queue, source_queue);
        }
        actual_biases.push(material.sort_bias);
    }
    assert_eq!(actual_biases, vec![204_800.0, 225_280.0, 227_328.0]);
    assert!(actual_biases.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn static_glow_bump_map_is_typed_as_a_fixed_function_mask() {
    let extras = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:249",
        "shaderName": "normal_glow_blendSrcalphaInvsrcalpha",
        "colors": [
            {"name": "_Color", "value": {"r": 0.294, "g": 0.294, "b": 0.294, "a": 1.0}},
            {"name": "_Emission", "value": {"r": 0.235, "g": 0.235, "b": 0.235, "a": 1.0}}
        ],
        "floats": [],
        "exactTextureSlots": [{
            "name": "_BumpMap",
            "textureId": "CustomAssetBundle-example:15"
        }],
        "runtimeTextureContract": {
            "schema": "ffone.legacy-glow-texture-bindings.v1",
            "emissiveTexture": "_BumpMap",
            "semantic": "constantColor(0.5)-lerp-by-texture-alpha"
        }
    })
    .to_string();

    let pending =
        PendingLegacyStaticWorldMaterial::from_gltf_extras(Some("gate"), &extras).unwrap();
    assert!(pending.has_glow_mask);
    assert!(pending.params.glow_mask);
    assert_eq!(
        pending.params.shader,
        LegacyShaderKind::AlphaBlendNormalGlow
    );
    assert_eq!(
        pending.render_passes[0].render_mode.color_write,
        LegacyColorWriteMask::Rgb
    );
    let material = pending
        .params
        .material_for_pass(pending.render_passes[0], &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(material.uniform.legacy_effect.z, 1.0);

    let missing_contract = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:249",
        "shaderName": "normal_glow_blendSrcalphaInvsrcalpha",
        "exactTextureSlots": [{
            "name": "_BumpMap",
            "textureId": "CustomAssetBundle-example:15"
        }]
    })
    .to_string();
    assert!(
        PendingLegacyStaticWorldMaterial::from_gltf_extras(None, &missing_contract).is_err()
    );
}

pub(super) fn test_assigned_binding(
    slot: &str,
    texture: u32,
    source_name: &str,
    uri: &str,
    color_space: &str,
) -> Value {
    const SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    serde_json::json!({
        "slot": slot,
        "texture": texture,
        "sourceName": source_name,
        "uri": uri,
        "sampler": {
            "index": texture,
            "descriptor": {
                "name": source_name,
                "magFilter": "linear",
                "minFilter": "linear",
                "wrapS": "repeat",
                "wrapT": "repeat",
                "legacyFilterMode": 1,
                "legacyWrapMode": 0,
                "anisotropyLevel": 1,
                "mipMapBias": 0.0
            }
        },
        "mipProvenance": {
            "sourceTextureFormat": 4,
            "sourceTextureFormatName": "ARGB32",
            "sourceMipCount": 1,
            "sourceChainByteLength": 4,
            "sourceChainSha256": SHA256,
            "sourceChainComplete": true,
            "sourceLayout": "largestToSmallestContiguous",
            "publishedPixelTransform": "vertical-flip-only-for-png-top-left-origin",
            "publishedPolicy": "baseLevelOnly"
        },
        "mipLevels": [{
            "level": 0,
            "width": 1,
            "height": 1,
            "uri": uri,
            "sourceByteOffset": 0,
            "sourceByteLength": 4,
            "sourceByteSha256": SHA256,
            "decodedRgba8ByteLength": 4,
            "decodedRgba8Sha256": SHA256,
            "pngByteLength": 70,
            "pngSha256": SHA256
        }],
        "scale": [1.0, 1.0],
        "offset": [0.0, 0.0],
        "pivot": [0.5, 0.5],
        "rotation": 0.0,
        "colorSpace": color_space
    })
}

pub(super) fn test_null_binding(slot: &str, color_space: &str) -> Value {
    serde_json::json!({
        "slot": slot,
        "texture": null,
        "sourceName": null,
        "uri": null,
        "sampler": null,
        "mipProvenance": null,
        "mipLevels": null,
        "scale": [1.0, 1.0],
        "offset": [0.0, 0.0],
        "pivot": [0.5, 0.5],
        "rotation": 0.0,
        "colorSpace": color_space
    })
}

#[test]
fn actor_skin_base_level_accepts_audited_source_mips_with_exact_source_sampler() {
    let mut contract = test_runtime_texture_contract("alternate_donor", 10, &"a".repeat(64));
    contract.source.mip_map = true;
    contract.source.source_mip_count = 9;

    assert!(validate_character_runtime_texture_contract(&contract).is_ok());

    contract.sampler.min_filter = SamplerMinFilter::LinearMipmapNearest;
    assert!(validate_character_runtime_texture_contract(&contract).is_ok());
}

#[test]
fn production_dexter_keeps_authored_face_sword_and_glasses_slots() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/characters/npcs/npc_dexter2/npc_dexter2.glb");
    let bytes = std::fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let document: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let primitives = document["meshes"][0]["primitives"].as_array().unwrap();
    assert_eq!(primitives.len(), 4);
    for (slot, primitive) in primitives.iter().enumerate() {
        let extras = bevy::gltf::GltfExtras {
            value: primitive["extras"].to_string(),
        };
        assert_eq!(native_npc_table_texture_writable(Some(&extras)), slot == 0);
    }
    assert!(native_npc_table_texture_writable(None));
}

#[test]
fn retrobution_rim_and_fusion_matter_defaults_are_exact_and_distinct() {
    let toon = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon);
    assert_eq!(toon.shader.rim_mode(), 1.0);
    assert_eq!(toon.rim_power, 4.0);
    assert_eq!(toon.rim_intensity, 2.0);

    let weeper =
        LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToonRimTransparent);
    assert_eq!(weeper.shader.rim_mode(), 2.0);
    assert_eq!(weeper.rim_power, 10.0);
    assert_eq!(weeper.rim_intensity, 0.5);
    assert_eq!(weeper.transparency, 0.5);
    assert_eq!(
        LegacyModelRenderPlan::for_shader(weeper.shader).passes[0]
            .render_mode
            .source_queue,
        3003
    );

    let fusion = LegacyModelMaterialParams::for_shader(LegacyShaderKind::FusionMatterLightDir);
    assert_eq!(fusion.shader.rim_mode(), 3.0);
    assert_eq!(fusion.base_color, LinearRgba::new(0.6, 0.6, 0.6, 1.0));
    assert_eq!(fusion.tint_color, LinearRgba::new(0.0, 1.0, 0.0, 1.0));
    assert_eq!(
        fusion.ambient_color,
        LinearRgba::new(1.0, 0.976, 0.208, 1.0)
    );
    assert_eq!(fusion.emission, LinearRgba::new(0.529, 0.576, 1.0, 1.0));
    assert_eq!(fusion.rim_color, LinearRgba::new(1.0, 0.976, 0.208, 1.0));
    assert_eq!(fusion.shadow_strength, 0.65);
    assert_eq!(legacy_outline_visibility(true), Visibility::Inherited);
    assert_eq!(legacy_outline_visibility(false), Visibility::Hidden);
}

#[test]
fn duplicate_cull_off_toon_name_uses_typed_passes_as_its_discriminator() {
    let name = "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff";
    let classified = LegacyShaderKind::classify_exact(name).unwrap();

    let fallback = canonical_expected_passes(&LegacyModelMaterialParams::for_shader(
        LegacyShaderKind::SkinnedToonCullOffFallback,
    ));
    assert_eq!(
        refine_exact_shader_kind(name, classified, &fallback),
        LegacyShaderKind::SkinnedToonCullOffFallback
    );

    let category = canonical_expected_passes(&LegacyModelMaterialParams::for_shader(
        LegacyShaderKind::SkinnedToonCullOffCategory,
    ));
    assert_eq!(
        refine_exact_shader_kind(name, classified, &category),
        LegacyShaderKind::SkinnedToonCullOffCategory
    );

    let e1 = "SkinnedToonShading_blendSrcalphaInvsrcalpha e1";
    let e1_classified = LegacyShaderKind::classify_exact(e1).unwrap();
    assert_eq!(
        refine_exact_shader_kind(e1, e1_classified, &category),
        LegacyShaderKind::SkinnedToonCullOffCategory
    );
}

#[test]
fn embedded_wgsl_keeps_the_fixed_function_contract_visible() {
    let base = include_str!("../legacy_model_base.wgsl");
    let outline = include_str!("../legacy_model_outline.wgsl");
    let water = include_str!("../legacy_water.wgsl");
    for shader in [base, outline, water] {
        assert!(shader.contains("mesh_functions::get_tag(instance_index)"));
        assert!(shader.contains("(end - 32.0)) / 32.0"));
        assert!(shader.contains("pbr_functions::visibility_range_dither"));
        assert!(!shader.contains("get_visibility_range_dither_level"));
    }
    // A fogged surface and its ink line must reach the sampled area haze on
    // the same object-range curve; the ordered dither alone erodes an
    // object out of clear air and exposes the native draw distance.
    for shader in [base, outline] {
        assert!(shader.contains("range_fade = clamp(f32(mesh.visibility_range_dither) / 16.0"));
        assert!(
            shader.contains(
                "let haze_amount = max(fog_amount, select(0.0, range_fade, fogged));"
            )
        );
    }
    assert!(base.contains("mix(encoded_surface, encoded_fog, haze_amount)"));
    assert!(outline.contains("haze_amount,"));
    assert!(base.contains("expanded_position += vertex.normal * material.legacy_effect.x"));
    assert!(base.contains("let legacy_time_x = globals.time / 20.0"));
    assert!(base.contains("transformed_uv.y + scroll"));
    assert!(base.contains("transformed_uv.y - scroll"));
    assert!(base.contains("scroll + transformed_normal.x"));
    assert!(base.contains("linear_to_srgb(base_sample.rgb)"));
    assert!(base.contains("fusion_rgb + bump.rgb"));
    assert!(base.contains("fusion_rgb + shader_map.rgb"));
    assert!(base.contains("srgb_to_linear("));
    assert!(base.contains("linear_to_srgb(base_sample.rgb) * material.base_color.rgb * 2.0"));
    // Toon RGB lighting is the native comic style; it is intentionally
    // independent of the source ramp. Alpha and non-toon contracts remain.
    assert!(base.contains("main_uv + vec2<f32>(0.0, legacy_time_x * 0.25)"));
    assert!(base.contains("main_uv + vec2<f32>(0.0, legacy_time_x * -0.15)"));
    assert!(base.contains("+ vec2<f32>(0.0, legacy_time_x * -0.35)"));
    assert!(base.contains("linear_to_srgb(fusion_sample.rgb).xy * 0.65"));
    assert!(base.contains("view_normal.y * 2.0 - 0.6"));
    assert!(!base.contains("let highlight_direction"));
    assert!(base.contains("masks_electricity.b * material.emission.rgb"));
    assert!(base.contains("material.base_color.rgb * diffuse_light + material.emission.rgb"));
    assert!(base.contains("primary_rgb += material.ambient_color.rgb * LEGACY_GLOBAL_AMBIENT"));
    assert!(base.contains(
        "material.light_direction_family.w > 2.5 && material.light_direction_family.w < 3.5"
    ));
    assert!(base.contains(
        "material.light_direction_family.w > 3.5 && material.light_direction_family.w < 4.5"
    ));
    assert!(base.contains(
        "material.light_direction_family.w > 5.5 && material.light_direction_family.w < 6.5"
    ));
    assert_eq!(
        LegacyShaderKind::AdditiveOneOneTwoSided.shading_family(),
        4.0
    );
    assert_eq!(
        LegacyModelMaterialParams::for_shader(LegacyShaderKind::AdditiveOneOneTwoSided)
            .light_direction_world,
        LEGACY_MAIN_LIGHT_RAY_DIRECTION
    );
    assert!(base.contains("var fixed_function_rgb_scale = 2.0"));
    assert!(base.contains("fixed_function_rgb_scale = 1.0"));
    assert!(base.contains("fixed_function_alpha = base_sample.a * mesh.legacy_primary.a"));
    assert!(base.contains("* fixed_function_rgb_scale"));
    assert!(!base.contains("base_sample.rgb * mesh.legacy_primary.rgb * 2.0"));
    assert!(base.contains("let glow_mask = textureSample(bump_texture"));
    assert!(base.contains("material.legacy_effect.z > 0.5"));
    assert!(!base.contains("world_normal *= material.legacy_effect.w"));
    assert!(base.contains("* material.base_color.a"));
    assert!(base.contains("* material.custom_effect.y"));
    assert!(base.contains(
        "primary_alpha = material.base_color.r + diffuse_light * material.base_color.a"
    ));
    assert!(base.contains("surface.a <= material.alpha_effect.x"));
    assert!(base.contains("rotate_about_pivot"));
    assert!(base.contains("let angle = radians(degrees)"));
    assert!(base.contains("material.uv_pivot_rotation.z"));
    assert!(base.contains("#ifdef LEGACY_GPU_UV_ANIMATION"));
    assert!(base.contains("material.uv_animation.xy * animation_time"));
    assert!(!base.contains("let distortion ="));
    assert!(!base.contains("surface.rgb *= fusion_light"));
    assert!(outline.contains("skinning::skin_model"));
    assert!(outline.contains("outline.width_fat.x"));
    assert!(outline.contains("mesh_view_bindings::view"));
    assert!(outline.contains("#ifdef DISTANCE_FOG\n#import bevy_pbr::mesh_view_bindings::fog"));
    assert!(outline.contains("outline.fog_params.x > 0.5"));
    assert!(outline.contains("view.view_from_world * mesh.world_position"));
}

#[test]
fn exact_white_and_black_fallbacks_are_used_only_for_unassigned_slots() {
    let pending = |defaults: Vec<ShaderLabTextureDefaultProperty>,
                   texture_bindings: Vec<LegacyGltfTextureBinding>| {
        PendingLegacyModelMaterial {
            true_name: "toon".into(),
            serialized_shader_name: "skinntoone2".into(),
            declared_shader_name: "SkinnedToonShading_blendSrcalphaInvsrcalpha e1".into(),
            params: LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon),
            shader_texture_defaults: defaults,
            texture_bindings,
            source_render_queue: 2_900,
            source_passes: Vec::new(),
            source_pass_count: 0,
        }
    };
    let property = |slot: &str, value| ShaderLabTextureDefaultProperty {
        slot: slot.into(),
        value,
    };
    let mut images = Assets::<Image>::default();

    let exact_white = pending(
        vec![
            property("_MainTex", ShaderLabTextureDefault::BuiltinWhite),
            property("_SpecMap", ShaderLabTextureDefault::Blank),
            property("_ShaderMap", ShaderLabTextureDefault::BuiltinWhite),
        ],
        Vec::new(),
    );
    let mut textures = LegacyModelTextures::default();
    apply_unassigned_shader_texture_defaults(&exact_white, &mut textures, &mut images);
    assert!(textures.base_builtin_white);
    assert!(textures.toon_ramp_builtin_white);
    assert!(textures.is_complete_for(LegacyShaderKind::SkinnedToon));

    let blank_base = pending(
        vec![
            property("_MainTex", ShaderLabTextureDefault::Blank),
            property("_ShaderMap", ShaderLabTextureDefault::BuiltinWhite),
        ],
        Vec::new(),
    );
    let mut textures = LegacyModelTextures::default();
    apply_unassigned_shader_texture_defaults(&blank_base, &mut textures, &mut images);
    assert!(!textures.base_builtin_white);
    assert!(!textures.is_complete_for(LegacyShaderKind::SkinnedToon));

    let assigned_main = runtime_binding_from_typed(
        serde_json::from_value(test_assigned_binding(
            "_MainTex",
            0,
            "body.dds",
            "toon.textures/body.png",
            "srgb",
        ))
        .unwrap(),
    );
    let assigned_overrides_default = pending(
        vec![
            property("_MainTex", ShaderLabTextureDefault::BuiltinWhite),
            property("_ShaderMap", ShaderLabTextureDefault::BuiltinWhite),
        ],
        vec![assigned_main],
    );
    let mut textures = LegacyModelTextures::default();
    apply_unassigned_shader_texture_defaults(
        &assigned_overrides_default,
        &mut textures,
        &mut images,
    );
    assert!(!textures.base_builtin_white);
    assert!(textures.toon_ramp_builtin_white);

    let mut fusion_black = pending(
        vec![
            property("_MainTex", ShaderLabTextureDefault::BuiltinBlack),
            property("_BumpMap", ShaderLabTextureDefault::BuiltinBlack),
            property("_ShaderMap", ShaderLabTextureDefault::BuiltinBlack),
        ],
        Vec::new(),
    );
    fusion_black.params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::FusionEffect);
    let mut textures = LegacyModelTextures::default();
    apply_unassigned_shader_texture_defaults(&fusion_black, &mut textures, &mut images);
    assert!(textures.is_complete_for(LegacyShaderKind::FusionEffect));
    for handle in [
        textures.base.as_ref().unwrap(),
        textures.bump.as_ref().unwrap(),
        textures.effect_map.as_ref().unwrap(),
    ] {
        assert_eq!(
            images.get(handle).unwrap().data.as_deref(),
            Some([0, 0, 0, 255].as_slice())
        );
    }
    assert_eq!(
        images
            .get(textures.base.as_ref().unwrap())
            .unwrap()
            .texture_descriptor
            .format,
        TextureFormat::Rgba8UnormSrgb
    );
}
