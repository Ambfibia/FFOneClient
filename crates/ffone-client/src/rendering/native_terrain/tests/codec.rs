use super::*;

#[test]
fn sanitized_runtime_metadata_keeps_only_payload_contract_and_still_validates() {
    let descriptor_path = "map/tiles/map_08_06/terrain/terrain.json";
    let root = asset_root();
    let terrain_root = root.join("map/tiles/map_08_06/terrain");
    let mut descriptor_json: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(descriptor_path)).unwrap()).unwrap();
    sanitize_runtime_terrain(&mut descriptor_json);
    let descriptor: NativeTerrainDescriptor =
        serde_json::from_value(descriptor_json.clone()).unwrap();
    validate_descriptor(&descriptor).unwrap();
    assert!(descriptor.source.is_none());
    assert!(descriptor.scene_instance.is_none());
    let attributes = descriptor.gameplay_attributes.as_ref().unwrap();
    assert!(attributes.raw_parsed_document.is_none());
    assert!(attributes.source.is_none());
    assert!(
        descriptor
            .splat
            .weight_maps
            .iter()
            .all(|weight| weight.source.is_none()
                && weight.mips.iter().all(|mip| mip.source_encoded.is_none()))
    );
    assert!(descriptor.splat.layers.iter().all(|layer| {
        layer.mode_source.is_none()
            && layer.mode_evidence.is_none()
            && layer.albedo.source.is_none()
            && layer
                .albedo
                .mips
                .iter()
                .all(|mip| mip.source_encoded.is_none())
    }));
    let serialized_descriptor = serde_json::to_string(&descriptor_json).unwrap();
    for forbidden in [
        "retrobution",
        "rawParsedDocument",
        "rawDocument",
        "sceneInstance",
        "sourceEncoded",
    ] {
        assert!(
            !serialized_descriptor
                .to_ascii_lowercase()
                .contains(&forbidden.to_ascii_lowercase())
        );
    }

    load_heightmap(&terrain_root, &descriptor).unwrap();
    load_gameplay_attributes(&terrain_root, attributes).unwrap();
    for weight in &descriptor.splat.weight_maps {
        verify_texture_mip_files(&terrain_root, &weight.mips, "sanitized weight").unwrap();
    }
    for layer in &descriptor.splat.layers {
        let layer_root = if layer.albedo.path.starts_with("map/shared/") {
            &root
        } else {
            &terrain_root
        };
        verify_texture_mip_files(layer_root, &layer.albedo.mips, "sanitized albedo").unwrap();
    }

    let environment_path = terrain_root.join("environment/environment.json");
    let mut environment_json: serde_json::Value =
        serde_json::from_slice(&fs::read(environment_path).unwrap()).unwrap();
    sanitize_runtime_environment(&mut environment_json);
    let environment: NativeTerrainEnvironment =
        serde_json::from_value(environment_json.clone()).unwrap();
    validate_environment(&environment, descriptor.environment.as_ref().unwrap()).unwrap();
    assert!(environment.map_scene.is_none());
    assert!(environment.placement_audit.is_none());
    assert!(environment.source_code_evidence.is_empty());
    assert!(environment.ambience.source_object.is_none());
    assert!(environment.terrain_detail.source_object.is_none());
    assert!(
        environment
            .terrain_detail
            .terrain_renderer_source_object
            .is_none()
    );
    let serialized = serde_json::to_string(&environment_json).unwrap();
    for forbidden in [
        "Retrobution",
        "parsedData",
        "sourceObject",
        "placementAudit",
        "sourceCodeEvidence",
    ] {
        assert!(!serialized.contains(forbidden));
    }
}

#[test]
fn grass_chunk_admission_never_exceeds_the_per_frame_vertex_budget() {
    let mut budget = NativeTerrainGrassAdmissionBudget::default();
    assert!(budget.admit(40_000));
    assert!(!budget.admit(40_000));
    assert!(budget.admit(20_000));
    assert!(budget.admit(5_536));
    assert!(!budget.admit(1));

    let mut oversized = NativeTerrainGrassAdmissionBudget::default();
    assert!(!oversized.admit(NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME + 1));
    assert_eq!(
        oversized.remaining_vertices,
        NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME
    );
}
