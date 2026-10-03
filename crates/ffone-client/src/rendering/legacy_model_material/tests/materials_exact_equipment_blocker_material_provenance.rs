use super::*;

#[test]
fn equipment_blocker_shader_modes_reach_exact_bevy_fixed_function_state() {
    let toon = LegacyModelRenderPlan::for_shader(LegacyShaderKind::SkinnedToonCullOffFallback);
    assert_eq!(toon.passes.len(), 2);
    assert_eq!(toon.passes[0].kind, LegacyPassKind::Surface);
    assert_eq!(toon.passes[0].render_mode.cull, LegacyCullMode::Back);
    assert!(toon.passes[0].render_mode.depth_write);
    assert_eq!(toon.passes[0].render_mode.source_queue, 2900);
    assert_eq!(toon.passes[1].kind, LegacyPassKind::Outline);
    assert_eq!(toon.passes[1].render_mode.cull, LegacyCullMode::Front);

    let alpha = LegacyModelRenderPlan::for_shader(LegacyShaderKind::AlphaBlendNormal).passes[0];
    assert_eq!(alpha.alpha_test, LegacyAlphaTest::Disabled);
    assert_eq!(
        alpha.render_mode.blend,
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
    );
    assert_eq!(alpha.render_mode.cull, LegacyCullMode::Back);
    assert!(alpha.render_mode.depth_write);
    assert_eq!(alpha.render_mode.color_write, LegacyColorWriteMask::Rgba);
    assert_eq!(alpha.render_mode.source_queue, 3000);

    let additive =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AdditiveOneOneTwoSided).passes[0];
    assert_eq!(additive.alpha_test, LegacyAlphaTest::Disabled);
    assert_eq!(additive.render_mode.blend, LegacyBlendMode::OneOne);
    assert_eq!(additive.render_mode.cull, LegacyCullMode::Off);
    assert!(!additive.render_mode.depth_write);
    assert!(!additive.render_mode.alpha_cutout);
    assert_eq!(additive.render_mode.color_write, LegacyColorWriteMask::Rgb);
    assert_eq!(additive.render_mode.source_queue, 3011);

    let alpha_component = BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    };
    assert_eq!(
        bevy_blend_state(alpha.render_mode.blend),
        Some(BlendState {
            color: alpha_component,
            alpha: alpha_component,
        })
    );
    let additive_component = BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    };
    assert_eq!(
        bevy_blend_state(additive.render_mode.blend),
        Some(BlendState {
            color: additive_component,
            alpha: additive_component,
        })
    );
    assert_eq!(bevy_cull_mode(additive.render_mode.cull), None);
    assert_eq!(
        bevy_color_write_mask(additive.render_mode.color_write),
        ColorWrites::RED | ColorWrites::GREEN | ColorWrites::BLUE
    );

    let face = LegacyModelRenderPlan::for_shader(LegacyShaderKind::SkinDirectionalAlphaBlend);
    assert_eq!(face.passes.len(), 1);
    let face = face.passes[0];
    assert_eq!(face.kind, LegacyPassKind::Surface);
    assert_eq!(
        face.render_mode.blend,
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
    );
    assert_eq!(face.render_mode.cull, LegacyCullMode::Back);
    assert!(face.render_mode.depth_write);
    assert_eq!(face.render_mode.color_write, LegacyColorWriteMask::Rgba);
    assert_eq!(face.render_mode.source_queue, 2_900);
    assert_eq!(face.alpha_test, LegacyAlphaTest::Disabled);
}

#[test]
fn exact_equipment_blocker_material_provenance_parses_without_fallback() {
    let alpha_blend = serde_json::json!({
        "enabled":true,
        "sourceColor":"sourceAlpha",
        "destinationColor":"oneMinusSourceAlpha",
        "colorOperation":"add",
        "sourceAlpha":"sourceAlpha",
        "destinationAlpha":"oneMinusSourceAlpha",
        "alphaOperation":"add"
    });
    let disabled_alpha = serde_json::json!({"mode":"disabled"});
    let disabled_outline = serde_json::json!({"mode":"disabled"});
    let pass = |name: Option<&str>,
                blend: Value,
                cull: &str,
                z_write: bool,
                color_mask: u8,
                outline: Value| {
        serde_json::json!({
            "name":name,
            "blend":blend,
            "cull":cull,
            "zWrite":z_write,
            "zTest":"lessEqual",
            "alphaTest":disabled_alpha.clone(),
            "colorMask":color_mask,
            "outline":outline
        })
    };
    let extras =
        |name: &str, shader: &str, render_queue: i32, defaults: Value, passes: Value| {
            serde_json::json!({
                "ffone": {
                    "name":name,
                    "serializedShaderName":shader,
                    "declaredShaderName":shader,
                    "legacyShaderName":shader,
                    "renderQueue":render_queue,
                    "standardTextureRefsAreLoaderHints":true,
                    "shaderTextureDefaults":defaults,
                    "textureBindings":[],
                    "passes":passes
                }
            })
            .to_string()
        };

    // These are the single pass signatures present in all 77 failed
    // full-v1 equipment previews, copied from their GLB extras.
    let toon_shader = "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff";
    let toon = extras(
        "f_back_b10galacticenforcer",
        toon_shader,
        2900,
        serde_json::json!([
            {"slot":"_MainTex","value":"builtinWhite"},
            {"slot":"_SpecMap","value":"blank"},
            {"slot":"_ShaderMap","value":"builtinWhite"}
        ]),
        serde_json::json!([
            pass(
                Some("BASE"),
                alpha_blend.clone(),
                "back",
                true,
                15,
                disabled_outline.clone()
            ),
            pass(
                Some("OUTLINE"),
                alpha_blend.clone(),
                "front",
                true,
                15,
                serde_json::json!({
                    "mode":"worldSpace",
                    "width":0.004999999888241291,
                    "color":[0.0,0.0,0.0,1.0]
                })
            )
        ]),
    );
    let toon = PendingLegacyModelMaterial::from_gltf_extras(None, &toon).unwrap();
    assert_eq!(
        toon.params.shader,
        LegacyShaderKind::SkinnedToonCullOffFallback
    );
    assert_eq!(toon.declared_shader_name, toon_shader);
    assert_eq!(toon.source_pass_count, 2);

    let face_shader = "Skin_DirLight_AmbLight_blendSrcalphaInvsrcalpha";
    let face = serde_json::json!({
        "ffone": {
            "name":"m_face_001_type01-main-link_a.dds",
            "serializedShaderName":face_shader,
            "declaredShaderName":face_shader,
            "legacyShaderName":face_shader,
            "renderQueue":2900,
            "standardTextureRefsAreLoaderHints":true,
            "colors":[
                {"name":"_Color","value":[0.0,0.0,0.0,1.0]},
                {"name":"_Emission","value":[0.5,0.5,0.5,1.0]},
                {"name":"_AmbColor","value":[0.0,0.0,0.0,1.0]}
            ],
            "floats":[{"name":"_FatFactor","value":0.0}],
            "shaderTextureDefaults":[
                {"slot":"_MainTex","value":"builtinWhite"},
                {"slot":"_SpecMap","value":"blank"}
            ],
            "textureBindings":[],
            "passes":[pass(
                Some("BASE"),
                alpha_blend.clone(),
                "back",
                true,
                15,
                disabled_outline.clone()
            )]
        }
    })
    .to_string();
    let face = PendingLegacyModelMaterial::from_gltf_extras(None, &face).unwrap();
    assert_eq!(
        face.params.shader,
        LegacyShaderKind::SkinDirectionalAlphaBlend
    );
    assert_eq!(face.params.ambient_color, LinearRgba::BLACK);
    assert_eq!(face.params.emission, LinearRgba::new(0.5, 0.5, 0.5, 1.0));
    assert_eq!(face.params.fat_factor, 0.0);
    assert_eq!(face.source_render_queue, 2_900);
    assert_eq!(face.source_pass_count, 1);
    assert_eq!(face.source_passes[0].name.as_deref(), Some("BASE"));

    let alpha_shader = "normal_blendSrcalphaInvsrcalpha";
    let alpha = extras(
        "vehicle_kimchi",
        alpha_shader,
        3000,
        serde_json::json!([{"slot":"_MainTex","value":"builtinWhite"}]),
        serde_json::json!([pass(
            None,
            alpha_blend,
            "back",
            true,
            15,
            disabled_outline.clone()
        )]),
    );
    let alpha = PendingLegacyModelMaterial::from_gltf_extras(None, &alpha).unwrap();
    assert_eq!(alpha.params.shader, LegacyShaderKind::AlphaBlendNormal);
    assert_eq!(alpha.source_pass_count, 1);

    let additive_shader = "normal_blendOneOne_zwriteOff_cullOff";
    let additive = extras(
        "vehicle_timesquad",
        additive_shader,
        3011,
        serde_json::json!([{"slot":"_MainTex","value":"builtinWhite"}]),
        serde_json::json!([pass(
            None,
            serde_json::json!({
                "enabled":true,
                "sourceColor":"one",
                "destinationColor":"one",
                "colorOperation":"add",
                "sourceAlpha":"one",
                "destinationAlpha":"one",
                "alphaOperation":"add"
            }),
            "off",
            false,
            7,
            disabled_outline
        )]),
    );
    let additive = PendingLegacyModelMaterial::from_gltf_extras(None, &additive).unwrap();
    assert_eq!(
        additive.params.shader,
        LegacyShaderKind::AdditiveOneOneTwoSided
    );
    assert_eq!(additive.source_pass_count, 1);
}

#[test]
fn material_builder_uses_literal_cutout_threshold_and_preserves_uvs() {
    let mut params = LegacyModelMaterialParams::from_shader_name(
        "normal_blendSrcalphaInvsrcalphaTest_cullOff",
    )
    .unwrap();
    params.alpha_cutoff = 0.75;
    params.uv_scale = Vec2::new(2.0, 3.0);
    params.uv_offset = Vec2::new(0.25, -0.5);
    params.uv_pivot = Vec2::splat(0.5);
    params.uv_rotation_degrees = 37.0;
    let pass = params.render_plan().passes[0];
    let material = params
        .material_for_pass(pass, &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(
        material.uniform.alpha_effect.xyz(),
        Vec3::new(0.9, 1.0, 0.0)
    );
    assert_eq!(
        material.uniform.uv_scale_offset,
        Vec4::new(2.0, 3.0, 0.25, -0.5)
    );
    assert_eq!(
        material.uniform.uv_pivot_rotation,
        Vec4::new(0.5, 0.5, 37.0, 0.0)
    );
    assert_eq!(material.uniform.uv_animation, Vec4::ZERO);
    assert!(!material.gpu_uv_animation);
}

#[test]
fn additive_greater_uses_material_cutoff_and_rejects_equality() {
    let mut params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::AdditiveTwoSided);
    params.alpha_cutoff = 0.0;
    let pass = params.render_plan().passes[0];
    let material = params
        .material_for_pass(pass, &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(
        material.uniform.alpha_effect.xyz(),
        Vec3::new(0.0, 1.0, 1.0)
    );
}

#[test]
fn exact_src_alpha_additive_extras_validate_without_shader_name_guessing() {
    let extras = r#"{
            "ffone": {
                "name":"blackhole-21 - default-fusion_mac_corruptak.dds",
                "serializedShaderName":"normal_blendSrcalphaOne_zwriteOff_cullOff",
                "declaredShaderName":"normal_blendSrcalphaOne_zwriteOff_cullOff",
                "legacyShaderName":"normal_blendSrcalphaOne_zwriteOff_cullOff",
                "renderQueue":3010,
                "standardTextureRefsAreLoaderHints":true,
                "colors":[
                    {"name":"_Color","value":[0.25,0.5,0.75,0.8]},
                    {"name":"_Emission","value":[0.1,0.2,0.3,0.0]}
                ],
                "shaderTextureDefaults":[],
                "textureBindings":[],
                "passes":[{
                    "name":null,
                    "blend":{"enabled":true,"sourceColor":"sourceAlpha","destinationColor":"one","colorOperation":"add","sourceAlpha":"sourceAlpha","destinationAlpha":"one","alphaOperation":"add"},
                    "cull":"off","zWrite":false,"zTest":"lessEqual",
                    "alphaTest":{"mode":"disabled"},
                    "colorMask":7,
                    "outline":{"mode":"disabled"}
                }]
            }
        }"#;
    let pending = PendingLegacyModelMaterial::from_gltf_extras(None, extras).unwrap();
    assert_eq!(
        pending.params.shader,
        LegacyShaderKind::SrcAlphaAdditiveTwoSided
    );
    assert_eq!(pending.source_render_queue, 3010);
    assert_eq!(pending.source_pass_count, 1);
    assert_eq!(
        pending.source_passes[0].blend,
        canonical_blend(
            true,
            MaterialBlendFactor::SourceAlpha,
            MaterialBlendFactor::One,
        )
    );
    assert_eq!(pending.source_passes[0].color_mask, 0b0111);
}

#[test]
fn typed_pass_contradiction_is_rejected_instead_of_counted() {
    let extras = r#"{
            "ffone": {
                "name":"fusion",
                "serializedShaderName":"Skin_FusionEffect_blendSrcalphaInvsrcalpha",
                "declaredShaderName":"Skin_FusionEffect_blendSrcalphaInvsrcalpha",
                "legacyShaderName":"Skin_FusionEffect_blendSrcalphaInvsrcalpha",
                "renderQueue":2900,
                "standardTextureRefsAreLoaderHints":true,
                "shaderTextureDefaults":[],
                "textureBindings":[],
                "passes":[{
                    "name":"BASE",
                    "blend":{"enabled":true,"sourceColor":"sourceAlpha","destinationColor":"oneMinusSourceAlpha","colorOperation":"add","sourceAlpha":"sourceAlpha","destinationAlpha":"oneMinusSourceAlpha","alphaOperation":"add"},
                    "cull":"back","zWrite":true,"zTest":"lessEqual",
                    "alphaTest":{"mode":"disabled"},
                    "colorMask":15,
                    "outline":{"mode":"disabled"}
                }]
            }
        }"#;
    let error = PendingLegacyModelMaterial::from_gltf_extras(None, extras).unwrap_err();
    assert!(error.0.contains("contradicts the canonical pass"));
}

#[test]
fn typed_additive_pass_cannot_replace_greater_property_cutoff_with_literal() {
    let extras = r#"{
            "ffone": {
                "name":"spwaneye",
                "serializedShaderName":"normal_blendOneOneTest_cullOff",
                "declaredShaderName":"normal_blendOneOneTest_cullOff",
                "legacyShaderName":"normal_blendOneOneTest_cullOff",
                "renderQueue":3000,
                "standardTextureRefsAreLoaderHints":true,
                "floats":[{"name":"_Cutoff","value":0.0}],
                "shaderTextureDefaults":[],
                "textureBindings":[],
                "passes":[{
                    "name":null,
                    "blend":{"enabled":true,"sourceColor":"one","destinationColor":"one","colorOperation":"add","sourceAlpha":"one","destinationAlpha":"one","alphaOperation":"add"},
                    "cull":"off","zWrite":false,"zTest":"lessEqual",
                    "alphaTest":{"mode":"enabled","compare":"greater","reference":{"source":"literal","value":0.0}},
                    "colorMask":7,
                    "outline":{"mode":"disabled"}
                }]
            }
        }"#;
    let error = PendingLegacyModelMaterial::from_gltf_extras(None, extras).unwrap_err();
    assert!(error.0.contains("contradicts the canonical pass"));
    assert!(error.0.contains("FloatProperty"));
}

#[test]
fn typed_additive_pass_supplies_exact_shader_default_when_cutoff_is_not_saved() {
    let extras = r#"{
            "ffone": {
                "name":"fusion-eye",
                "serializedShaderName":"normal_blendOneOneTest_cullOff",
                "declaredShaderName":"normal_blendOneOneTest_cullOff",
                "legacyShaderName":"normal_blendOneOneTest_cullOff",
                "renderQueue":3000,
                "standardTextureRefsAreLoaderHints":true,
                "shaderTextureDefaults":[],
                "textureBindings":[],
                "passes":[{
                    "name":null,
                    "blend":{"enabled":true,"sourceColor":"one","destinationColor":"one","colorOperation":"add","sourceAlpha":"one","destinationAlpha":"one","alphaOperation":"add"},
                    "cull":"off","zWrite":false,"zTest":"lessEqual",
                    "alphaTest":{"mode":"enabled","compare":"greater","reference":{"source":"floatProperty","name":"_Cutoff","resolved_value":0.0}},
                    "colorMask":7,
                    "outline":{"mode":"disabled"}
                }]
            }
        }"#;

    let pending = PendingLegacyModelMaterial::from_gltf_extras(None, extras).unwrap();
    assert_eq!(pending.params.alpha_cutoff, 0.0);
    assert_eq!(pending.source_pass_count, 1);
}

#[test]
fn extras_parser_rejects_unknown_shader_instead_of_guessing() {
    let extras = r#"{"ffone":{"name":"glass","serializedShaderName":"normal_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD","declaredShaderName":"normal_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD","legacyShaderName":"normal_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD"}}"#;
    assert!(PendingLegacyModelMaterial::from_gltf_extras(None, extras).is_err());
}
