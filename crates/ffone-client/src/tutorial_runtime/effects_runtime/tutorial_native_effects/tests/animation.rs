use super::*;

#[test]
fn oni_projectile_pair_preserves_exact_native_mesh_animation_and_trail() {
    const ANIMATION_METADATA_SHA256: &str =
        "928f94f2218ead29db53e8b7c3d54ced92fe2ded831b265902d6490df101d058";
    const SHADER_SHA256: &str =
        "62560b7eaf853158c925a3c4e207878d7a2870140452a1abf64da830a6eceb22";
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&asset_root).unwrap();
    let catalog: JsonValue = serde_json::from_slice(
        &std::fs::read(asset_root.join("map/shared/projectiles/catalog.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        catalog.get("sourceBuild").and_then(JsonValue::as_str),
        Some(RETROBUTION_TUTORIAL_BUILD_ID)
    );
    for (
        effect_id,
        expected_scene,
        report_path,
        legacy_name,
        glb_blake3,
        container_route,
        root_path_id,
        closure_blake3,
    ) in [
        (
            15,
            "map/shared/projectiles/effects/models/es15/tail1.glb",
            "map/shared/projectiles/effects/models/es15/tail1.publish.json",
            "tail1",
            "2f7914e5b609d74341201ee612cc2e4cb57a171d6bbde0c9ea1ee03a52a84560",
            "prefabs/particle/effectscripts/es[15].prefab",
            9_853,
            "0644dd3ea157bf6d4223545b0ab7faa2b3b0fd0b1f8fe86a33bfb4ff2c5240b1",
        ),
        (
            391,
            "map/shared/projectiles/effects/models/es391/tail2.glb",
            "map/shared/projectiles/effects/models/es391/tail2.publish.json",
            "tail2",
            "78b56fc91ac0e0f9fe73d9f5972a3e78bccd134398b34f50a92f559679d4c02b",
            "prefabs/particle/effectscripts/es[391].prefab",
            9_282,
            "2938eaf91084541e5bc06b47d8eedf0777c27625fbf0a2d3a0785a2264fb4603",
        ),
    ] {
        let compiled = compile_projectile_plan(
            effect_id,
            library.projectile_effects.get(&effect_id).unwrap(),
        );
        let plan = compiled.plan.as_ref().unwrap_or_else(|| {
            panic!("projectile {effect_id} blockers: {:#?}", compiled.blockers)
        });
        assert_eq!(plan.rendered_nodes, 2);
        assert!(plan.trail.is_some(), "projectile {effect_id} lost TRAIL");
        assert_eq!(plan.mesh_scene, Some(expected_scene));
        let animation = plan
            .material_animation
            .as_ref()
            .unwrap_or_else(|| panic!("projectile {effect_id} lost nif-default"));
        assert!((animation.duration - 0.966_667_06).abs() < 1.0e-5);
        assert_eq!(animation.curves.len(), 1);
        let rotation = &animation.curves[0];
        assert_eq!(rotation.node_name, "Editable Poly");
        assert!(matches!(
            rotation.property,
            MaterialAnimatedProperty::UvRotationDegrees
        ));
        assert_eq!(rotation.curve.0.len(), 4);
        assert!(projectile_mesh_repeats_standard_animation(effect_id));
        assert!(
            compiled.blockers.is_empty(),
            "projectile {effect_id} retained blockers: {:#?}",
            compiled.blockers
        );

        let glb = std::fs::read(asset_root.join(expected_scene)).unwrap();
        assert_eq!(blake3::hash(&glb).to_hex().as_str(), glb_blake3);
        let report: JsonValue =
            serde_json::from_slice(&std::fs::read(asset_root.join(report_path)).unwrap())
                .unwrap();
        assert_eq!(
            report
                .pointer("/contract/legacy_name")
                .and_then(JsonValue::as_str),
            Some(legacy_name)
        );
        assert_eq!(
            report
                .pointer("/contract/glb_blake3")
                .and_then(JsonValue::as_str),
            Some(glb_blake3)
        );
        assert_eq!(
            report
                .pointer("/contract/source/animation_clips")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
        assert_eq!(
            report
                .pointer("/contract/source/animation_channels")
                .and_then(JsonValue::as_u64),
            Some(0)
        );
        assert_eq!(
            report
                .pointer("/contract/source/float_curves")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
        assert_eq!(
            report
                .pointer("/semanticProof/matched")
                .and_then(JsonValue::as_bool),
            Some(true)
        );
        for scope in ["source", "emitted"] {
            assert_eq!(
                report
                    .pointer(&format!("/semanticProof/{scope}/animationMetadataSha256"))
                    .and_then(JsonValue::as_str),
                Some(ANIMATION_METADATA_SHA256)
            );
        }
        assert_eq!(
            report
                .pointer("/materialPublish/materials/0/legacyShaderName")
                .and_then(JsonValue::as_str),
            Some("normal_blendOneOne_zwriteOff")
        );
        assert_eq!(
            report
                .pointer("/materialPublish/materials/0/shaderSha256")
                .and_then(JsonValue::as_str),
            Some(SHADER_SHA256)
        );

        let source = catalog
            .get("particleEffects")
            .and_then(JsonValue::as_array)
            .and_then(|effects| {
                effects.iter().find(|effect| {
                    effect.get("effectId").and_then(JsonValue::as_i64)
                        == Some(i64::from(effect_id))
                })
            })
            .unwrap_or_else(|| panic!("ES{effect_id} clean-source provenance"));
        assert_eq!(
            source.get("containerRoute").and_then(JsonValue::as_str),
            Some(container_route)
        );
        assert_eq!(
            source.get("rootAsset").and_then(JsonValue::as_str),
            Some("CustomAssetBundle-fa9dbcf4604f64024b06ff1d5e375918")
        );
        assert_eq!(
            source.get("rootPathId").and_then(JsonValue::as_i64),
            Some(root_path_id)
        );
        assert_eq!(
            source.get("closureBlake3").and_then(JsonValue::as_str),
            Some(closure_blake3)
        );
    }
}

#[test]
fn mesh_effects_preserve_the_exact_looping_legacy_animation_modes() {
    // EffectEmitterController.Start forces WrapMode.Loop on every
    // instantiated nifObject, including clips whose serialized Animation
    // component says Default. Keep this list aligned with the validated
    // native mesh publication map above.
    for effect_id in [
        425, 668, 705, 734, 736, 739, 740, 741, 742, 751, 766, 771, 833, 834, 865, 866,
    ] {
        assert!(mesh_effect_repeats_standard_animation(effect_id));
    }
    assert!(!mesh_effect_repeats_standard_animation(733));
}
