use super::*;
#[test]
fn eruption_material_curves_bind_actual_native_renderer_names() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root).unwrap();
    let plans = open(&locator).unwrap();
    let plan = &plans[&766];
    let bytes = locator.read(plan.mesh_scene.unwrap()).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let model: serde_json::Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let names: BTreeSet<_> = model["meshes"].as_array().unwrap().iter()
        .map(|mesh| mesh["name"].as_str().unwrap()).collect();
    let animation = plan.material_animation.as_ref().unwrap();
    assert_eq!(animation.curves.len(), 162);
    let mut uv_curves = 0;
    for curve in &animation.curves {
        assert!(names.contains(curve.node_name.as_str()),
            "unbound material curve {}: glTF renderer names differ from node names", curve.node_name);
        if matches!(curve.property, MaterialAnimatedProperty::UvOffsetX | MaterialAnimatedProperty::UvOffsetY | MaterialAnimatedProperty::UvRotationDegrees) {
            uv_curves += 1;
        }
    }
    assert_eq!(uv_curves, 18);
    assert!(mesh_effect_repeats_standard_animation(766));
}

#[test]
fn production_skill_mesh_materials_match_native_render_contracts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut count = 0;
    for path in SKILL_MESHES {
        let bytes = std::fs::read(root.join(path)).unwrap();
        let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let json: serde_json::Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
        for material in json["materials"].as_array().unwrap() {
            let parsed =
                crate::legacy_model_material::PendingLegacyModelMaterial::from_gltf_extras(
                    material["name"].as_str(),
                    &material["extras"].to_string(),
                )
                .unwrap_or_else(|error| panic!("{path}: {error}"));
            assert_eq!(
                parsed.source_pass_count,
                parsed.params.render_plan().passes.len()
            );
            count += 1;
        }
        for image in json["images"].as_array().unwrap() {
            let uri = image["uri"].as_str().unwrap();
            assert!(
                root.join(path).parent().unwrap().join(uri).is_file(),
                "{path}: {uri}"
            );
        }
    }
    assert!(count >= SKILL_MESHES.len());
}

#[test]
fn production_coco_effects_retain_all_emitters_and_pickup_mesh() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root).unwrap();
    let plans = open(&locator).unwrap();
    assert_eq!(
        plans.keys().copied().collect::<Vec<_>>(),
        vec![
            54, 368, 370, 397, 398, 401, 407, 411, 412, 429, 431, 434, 435, 436, 438, 441, 447, 500, 502, 530, 531, 532, 592, 714, 766, 805, 827, 828
        ]
    );
    for (id, count) in [(397, 5), (398, 2), (401, 2), (530, 3), (531, 3), (532, 3), (592, 4), (766, 3)] {
        let plan = &plans[&id];
        assert_eq!(plan.emitters.len(), count);
        assert_eq!(plan.maximum_timer, 1.0);
        assert_eq!(plan.longest_lifetime, 1.0);
    }
    assert_eq!(plans[&397].mesh_scene, Some(PICKUP_MESH));
    assert!(!plans[&397].emitters[1].script_keys.last().unwrap().emit);
    assert_eq!(plans[&435].maximum_timer, -8.0);
    assert_eq!(plans[&441].maximum_timer, -2.0);
    assert!(plans[&441].disable_update);
    assert!(plans[&714].mesh_scene.is_some());
}
