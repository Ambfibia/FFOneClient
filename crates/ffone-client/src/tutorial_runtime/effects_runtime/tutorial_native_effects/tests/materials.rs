use super::*;

#[test]
fn particle_material_sample_changes_only_at_visible_animation_boundaries() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let compiled = compile_effect_plan(767, library.effects.get(&767).unwrap());
    let mut emitter = compiled
        .plan
        .unwrap_or_else(|| panic!("ES767 native blockers: {:#?}", compiled.blockers))
        .emitters
        .into_iter()
        .next()
        .expect("ES767 particle emitter");
    emitter.animate_color = true;

    assert_eq!(
        particle_material_animation_sample(&emitter, 0.0),
        particle_material_animation_sample(&emitter, 0.01),
        "sub-frame age changes must reuse the same shared material"
    );
    assert_ne!(
        particle_material_animation_sample(&emitter, 0.01),
        particle_material_animation_sample(&emitter, 0.1),
        "a quantized colour boundary must still update the material"
    );
    assert_ne!(
        particle_material_animation_sample(&emitter, 0.12),
        particle_material_animation_sample(&emitter, 0.13),
        "an atlas-cell boundary must still update the material"
    );
}

#[test]
fn material_curve_node_matches_the_published_node_dot_material_surface_name() {
    assert!(material_surface_matches_node(
        "Editable Poly.bubble_win_01",
        "Editable Poly"
    ));
    assert!(material_surface_matches_node(
        "PolyMesh.projectile_03",
        "PolyMesh"
    ));
    assert!(!material_surface_matches_node(
        "Editable Poly1.bubble_win_01",
        "Editable Poly"
    ));
}

#[test]
fn exact_shader_name_accepts_only_a_unique_leading_source_token() {
    let make = |script: &str| TutorialUnityObjectProof {
        asset: "asset".to_owned(),
        path_id: 1,
        type_id: 48,
        class_id: 48,
        object_type: "Shader".to_owned(),
        name: String::new(),
        canonical_blake3: "hash".to_owned(),
        value: serde_json::json!({ "m_Script": script }),
    };
    assert_eq!(
        exact_shader_name(&make(
            "Shader \"particle_blendOneOne_zwriteOff_cullOff\" {\n}\n"
        )),
        Ok("particle_blendOneOne_zwriteOff_cullOff".to_owned())
    );
    assert!(exact_shader_name(&make("// prefix\nShader \"bad\" {}")).is_err());
    assert!(exact_shader_name(&make("Shader \"one\" {} Shader \"two\" {}")).is_err());
    assert!(exact_shader_name(&make("Shader \"\" {}")).is_err());
}

#[test]
fn every_published_particle_shader_has_an_exact_blend_mode() {
    for (shader, expected) in [
        (
            "particle_blendZeroInvsrcalpha_zwriteOff_cullOff",
            ParticleBlendMode::ZeroOneMinusSrcAlpha,
        ),
        (
            "particle_blendOneOne_zwriteOff_cullOff",
            ParticleBlendMode::OneOne,
        ),
        (
            "particle_blendOneInvsrccolor_zwriteOff_cullOff",
            ParticleBlendMode::OneOneMinusSrcColor,
        ),
        (
            "particle_blendInvdestcolorOne_zwriteOff_cullOff",
            ParticleBlendMode::OneMinusDstColorOne,
        ),
        (
            "particle_blendInvsrccolorDestalpha_zwriteOff_cullOff",
            ParticleBlendMode::OneMinusSrcColorDstAlpha,
        ),
        (
            "particle_blendInvsrccolorInvsrccolor_zwriteOff_cullOff",
            ParticleBlendMode::OneMinusSrcColorOneMinusSrcColor,
        ),
        (
            "particle_blendSrccolorOne_zwriteOff_cullOff",
            ParticleBlendMode::SrcColorOne,
        ),
        (
            "particle_blendInvsrcalphaOne_zwriteOff_cullOff",
            ParticleBlendMode::OneMinusSrcAlphaOne,
        ),
        (
            "particle_blendSrcalphaOne_zwriteOff_cullOff",
            ParticleBlendMode::SrcAlphaOne,
        ),
        (
            "particle_blendSrcalphaInvsrccolor_zwriteOff_cullOff",
            ParticleBlendMode::SrcAlphaOneMinusSrcColor,
        ),
        (
            "particle_blendDestalphaInvsrccolor_zwriteOff_cullOff",
            ParticleBlendMode::DstAlphaOneMinusSrcColor,
        ),
    ] {
        assert_eq!(particle_blend_mode(shader), Some(expected), "{shader}");
    }
    assert_eq!(particle_blend_mode("particle_unknown"), None);
}

#[test]
fn native_particle_shader_reconstructs_primary_black_exp2_fog() {
    let shader = include_str!("../tutorial_native_effects.wgsl");
    assert!(shader.contains("mesh_view_bindings::view"));
    assert!(shader.contains("#ifdef DISTANCE_FOG\n#import bevy_pbr::mesh_view_bindings::fog"));
    assert!(shader.contains("view.view_from_world * mesh.world_position"));
    assert!(shader.contains("1.0 - exp(-(density_depth * density_depth))"));
    assert!(shader.contains("mix(encoded_rgb, vec3<f32>(0.0), fog_amount)"));
}

#[test]
fn es769_uses_its_exact_zero_inverse_source_alpha_blend() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let compiled = compile_effect_plan(769, library.effects.get(&769).unwrap());
    assert!(
        compiled.blockers.is_empty(),
        "ES769 native blockers: {:#?}",
        compiled.blockers
    );
    let plan = compiled.plan.expect("ES769 native particle plan");
    assert!(
        plan.emitters
            .iter()
            .any(|emitter| { emitter.blend_mode == ParticleBlendMode::ZeroOneMinusSrcAlpha })
    );
}

#[test]
fn buttercup_bloo_and_mac_attack_effects_have_exact_mesh_and_material_animation_plans() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    for (effect_id, expected_curves) in [
        (547, 126),
        (548, 40),
        (601, 4),
        (602, 18),
        (626, 54),
        (627, 45),
    ] {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        assert!(
            compiled.blockers.is_empty(),
            "Fusion effect {effect_id} blockers: {:#?}",
            compiled.blockers
        );
        let plan = compiled.plan.unwrap();
        assert_eq!(plan.mesh_scene, exact_effect_mesh_scene(effect_id));
        let animation = plan.material_animation.unwrap();
        assert_eq!(animation.curves.len(), expected_curves);
        assert!(animation.duration.is_finite() && animation.duration > 0.0);
    }
}
