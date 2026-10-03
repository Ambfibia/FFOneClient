use super::*;

#[test]
fn emits_deterministic_glb_with_skin_and_animation() {
    let model = rex();
    let first = encode_glb(&model).unwrap();
    let second = encode_glb(&model).unwrap();
    assert_eq!(first, second);
    let document = glb_json(&first);
    assert_eq!(document["extras"]["logicalModelName"], "Rex");
    assert_eq!(document["scenes"][0]["nodes"], json!([0]));
    assert_eq!(document["nodes"][0]["name"], "Rex");
    assert_eq!(document["skins"][0]["joints"], json!([1]));
    assert!(document["skins"][0]["inverseBindMatrices"].is_u64());
    assert!(document["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"].is_u64());
    assert!(document["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"].is_u64());
    assert_eq!(document["animations"][0]["name"], "run");
    assert_eq!(document["animations"][0]["extras"]["sampleRate"], 30.0);
    assert_eq!(document["animations"][0]["extras"]["wrapMode"], 2);
    assert_eq!(document["animations"][0]["extras"]["loop"], true);
    assert_eq!(
        document["animations"][0]["extras"]["nonTrs"]["events"][0]["functionName"],
        "Footstep"
    );
    assert_eq!(
        document["animations"][0]["extras"]["nonTrs"]["floatCurves"][0]["property"],
        "m_Enabled"
    );
    assert_eq!(
        document["animations"][0]["channels"][0]["target"]["node"],
        1
    );
}

#[test]
fn rejects_cycles_and_bad_animation_keys_but_allows_duplicate_node_names() {
    let mut model = rex();
    model.nodes[0].parent = Some(1);
    model.roots.clear();
    assert!(validate(&model).unwrap_err().to_string().contains("cycle"));
    let mut model = rex();
    model.nodes[1].name = "Rex".into();
    model.animations[0].metadata.float_curves.clear();
    validate(&model).unwrap();
    let mut model = rex();
    model.animations[0].channels[0].times = vec![1.0, 0.0];
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("times are invalid")
    );
}

#[test]
fn metadata_only_clip_is_preserved_without_an_empty_gltf_animation() {
    let mut model = rex();
    let clip = &mut model.animations[0];
    clip.name = "nif-default".into();
    clip.channels.clear();
    clip.metadata.float_curves.clear();
    clip.metadata.object_curves.clear();
    clip.metadata.events.clear();
    clip.keyed_duration = None;
    clip.event_duration = None;

    let first = encode_glb(&model).unwrap();
    let second = encode_glb(&model).unwrap();
    assert_eq!(
        first, second,
        "metadata-only GLB output must be deterministic"
    );
    let document = glb_json(&first);
    assert!(
        document.get("animations").is_none(),
        "a zero-channel glTF animation is schema-invalid"
    );
    let preserved = &document["extras"]["ffone"]["metadataOnlyAnimations"][0];
    assert_eq!(preserved["name"], "nif-default");
    assert_eq!(preserved["duration"], 1.0);
    assert_eq!(preserved["declaredDuration"], 1.0);
    assert!(preserved["keyedDuration"].is_null());
    assert!(preserved["eventDuration"].is_null());
    assert_eq!(preserved["sampleRate"], 30.0);
    assert_eq!(preserved["wrapMode"], 2);
    assert_eq!(preserved["loop"], true);
    assert_eq!(preserved["metadata"]["floatCurves"], json!([]));
    assert_eq!(preserved["metadata"]["objectCurves"], json!([]));
    assert_eq!(preserved["metadata"]["events"], json!([]));
}

#[test]
fn rejects_generated_node_mesh_skin_and_animation_names() {
    let mut model = materialized_rex();
    model.nodes[1].name = "Bip01--deadbeef".into();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("node name must be an exact source m_Name")
    );

    let mut model = materialized_rex();
    model.meshes[0].name = "Mesh#123".into();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("mesh name must be an exact source m_Name")
    );

    let mut model = materialized_rex();
    model.skins[0].name = "PathID_456".into();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("skin name must be an exact source m_Name")
    );

    let mut model = materialized_rex();
    model.animations[0].name = "run--0123456789abcdef".into();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("animation name must be an exact source m_Name")
    );

    let mut model = materialized_rex();
    model.nodes[1].name = "Editable Poly@#2".into();
    model.animations[0].metadata.float_curves[0].target_path = "Editable Poly@#2".into();
    validate(&model).expect("an unusual but exact source m_Name must remain valid");
}
