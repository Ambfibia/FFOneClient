use super::*;
use tempfile::TempDir;

#[test]
fn runtime_world_registry_blocks_every_legacy_world_command_without_mutation() {
    let temp = TempDir::new().unwrap();
    let asset_root = temp.path().join("assets/game");
    let registry_path = asset_root.join("_runtime/world.json");
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    let registry = br#"{"schema":"ffone.runtime-world.v1","entries":[]}"#;
    fs::write(&registry_path, registry).unwrap();

    for command in [
        "plan-world",
        "publish-world",
        "publish-native-terrain-batch",
    ] {
        let error = ensure_legacy_world_publication_allowed(&asset_root, command)
            .unwrap_err()
            .to_string();
        assert!(error.contains(command), "unexpected error: {error}");
        assert!(
            error.contains(RUNTIME_WORLD_REGISTRY_SCHEMA),
            "unexpected error: {error}"
        );
        assert!(
            error.contains("registry-aware world workflow in FusionForge"),
            "unexpected error: {error}"
        );
    }

    assert_eq!(fs::read(&registry_path).unwrap(), registry);
    assert_eq!(fs::read_dir(&asset_root).unwrap().count(), 1);
    assert_eq!(
        fs::read_dir(registry_path.parent().unwrap())
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn legacy_world_guard_allows_an_asset_tree_without_a_runtime_registry() {
    let temp = TempDir::new().unwrap();
    ensure_legacy_world_publication_allowed(temp.path(), "plan-world").unwrap();
}

#[test]
fn legacy_world_guard_fails_closed_for_unusable_registry() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("_runtime/world.json");
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();

    fs::write(&registry_path, b"not-json").unwrap();
    assert!(matches!(
        ensure_legacy_world_publication_allowed(temp.path(), "publish-world"),
        Err(SemanticAssetError::Json { .. })
    ));

    fs::write(&registry_path, br#"{"schema":"unexpected.v1"}"#).unwrap();
    let error = ensure_legacy_world_publication_allowed(temp.path(), "publish-world")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unsupported schema"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("FusionForge"),
        "unexpected error: {error}"
    );
}

fn manifest_entry(path: &str, kind: AssetKind, bytes: &[u8]) -> AssetManifestEntry {
    AssetManifestEntry {
        source_path: path.to_owned(),
        path: path.to_owned(),
        kind,
        bytes: bytes.len() as u64,
        blake3: blake3::hash(bytes).to_hex().to_string(),
    }
}

fn write_manifest(root: &Path, entries: Vec<AssetManifestEntry>) -> PathBuf {
    let path = root.join("asset-manifest.json");
    write_json(
        &path,
        &AssetManifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            protocol: 104,
            locale: "ru-RU".to_owned(),
            source_pack: serde_json::json!({"schema":"test","manifest_blake3":"test"}),
            files: entries,
        },
    )
    .unwrap();
    path
}

fn proof(name: &str) -> OwnershipProof {
    OwnershipProof {
        true_legacy_name: name.to_owned(),
        authority: "serialized object path id 1".to_owned(),
        source_build: "retrobution-20260613".to_owned(),
        source_archive: "Map_00_00".to_owned(),
        source_asset: "BuildPlayer-Map_00_00".to_owned(),
        source_object_ids: BTreeMap::from([("pathId".to_owned(), 1)]),
    }
}

fn make_glb(json: &Value) -> Vec<u8> {
    let mut json_bytes = serde_json::to_vec(json).unwrap();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    let length = 12 + 8 + json_bytes.len();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"glTF");
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&(length as u32).to_le_bytes());
    bytes.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&0x4E4F_534Au32.to_le_bytes());
    bytes.extend_from_slice(&json_bytes);
    bytes
}

#[test]
fn audit_counts_hashes_and_duplicate_readable_names_exactly() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let first = b"first";
    let second = b"second";
    fs::create_dir_all(root.join("textures")).unwrap();
    fs::write(root.join("textures/a--0123456789abcdef.png"), first).unwrap();
    fs::write(root.join("textures/a--fedcba9876543210.png"), second).unwrap();
    let manifest = write_manifest(
        root,
        vec![
            manifest_entry(
                "textures/a--0123456789abcdef.png",
                AssetKind::Texture,
                first,
            ),
            manifest_entry(
                "textures/a--fedcba9876543210.png",
                AssetKind::Texture,
                second,
            ),
        ],
    );

    let audit = audit_asset_tree(root, &manifest).unwrap();
    assert_eq!(audit.counts.manifest_entries, 2);
    assert_eq!(audit.counts.verified_entries, 2);
    assert_eq!(audit.counts.hashed_filename_entries, 2);
    assert_eq!(audit.counts.duplicate_name_candidate_groups, 1);
    assert_eq!(audit.counts.duplicate_name_candidate_entries, 2);
}

#[test]
fn plan_rejects_hashes_and_path_traversal() {
    assert!(validate_relative_path("../bad.glb").is_err());
    assert!(has_content_hash_filename(
        "characters/npc/dexter/dexter--0123456789abcdef.glb"
    ));
}

#[test]
fn duplicate_ownership_group_requires_explicit_variants() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("data")).unwrap();
    fs::write(root.join("data/a.json"), b"{}").unwrap();
    fs::write(root.join("data/b.json"), b"{}").unwrap();
    let manifest = write_manifest(
        root,
        vec![
            manifest_entry("data/a.json", AssetKind::Data, b"{}"),
            manifest_entry("data/b.json", AssetKind::Data, b"{}"),
        ],
    );
    let route = |source: &str, destination: &str| SemanticRoute {
        source_path: source.to_owned(),
        destination_path: destination.to_owned(),
        kind: AssetKind::Data,
        category: SemanticCategory::Data,
        ownership: proof("SameName"),
        duplicate_group: Some("same-name".to_owned()),
        variant_reason: None,
        content: RouteContent::CopyExact,
    };
    let plan = RoutePlan {
        schema: ROUTE_PLAN_SCHEMA.to_owned(),
        source_build: "retrobution-20260613".to_owned(),
        policy: "test".to_owned(),
        routes: vec![
            route("data/a.json", "data/named/a.json"),
            route("data/b.json", "data/named/b.json"),
        ],
    };
    assert!(validate_route_plan(root, &manifest, &plan).is_err());
}

#[test]
fn native_terrain_batch_publishes_blocked_placement_data_closure() {
    let temp = TempDir::new().unwrap();
    let asset_root = temp.path().join("assets/game");
    fs::create_dir_all(&asset_root).unwrap();
    let stale_scene_path = "world/maps/map_01_03/scene.json";
    let stale_scene = b"{\"schema\":\"stale-linked-scene\"}\n";
    fs::create_dir_all(asset_root.join("world/maps/map_01_03")).unwrap();
    fs::write(asset_root.join(stale_scene_path), stale_scene).unwrap();
    let manifest_path = write_manifest(
        &asset_root,
        vec![manifest_entry(
            stale_scene_path,
            AssetKind::Data,
            stale_scene,
        )],
    );
    let batch_root = temp.path().join("batch");
    let source_root = batch_root.join("maps/map_01_03/terrain");
    fs::create_dir_all(source_root.join("weights")).unwrap();
    fs::create_dir_all(source_root.join("layers/grass")).unwrap();
    fs::create_dir_all(source_root.join("gameplay")).unwrap();
    let heightmap = b"gray16-png";
    let weights = b"linear-rgba";
    let albedo = b"srgb-rgba";
    let attributes = b"gray8-attributes";
    let attributes_parsed = b"{\"schema\":\"test.attributes\"}\n";
    fs::write(source_root.join("heightmap.png"), heightmap).unwrap();
    fs::write(source_root.join("weights/weights_00.png"), weights).unwrap();
    fs::write(source_root.join("layers/grass/albedo.png"), albedo).unwrap();
    fs::write(source_root.join("gameplay/attributes.png"), attributes).unwrap();
    fs::write(source_root.join("gameplay/attributes.bin"), attributes).unwrap();
    fs::write(
        source_root.join("gameplay/attributes.raw.json"),
        attributes_parsed,
    )
    .unwrap();
    let prefixed = |bytes: &[u8]| format!("blake3:{}", blake3::hash(bytes).to_hex());
    let terrain = serde_json::json!({
        "schema": "ffone.native-terrain.v1",
        "trueName": "TerrainData_01_03",
        "source": {
            "assetName": "TerrainDataFixture",
            "terrainDataPathId": 7
        },
        "heightmap": {
            "path": "heightmap.png",
            "pngBlake3": prefixed(heightmap)
        },
        "splat": {
            "weightMaps": [{
                "path": "weights/weights_00.png",
                "pngBlake3": prefixed(weights)
            }],
            "layers": [{
                "albedo": {
                    "path": "layers/grass/albedo.png",
                    "pngBlake3": prefixed(albedo)
                }
            }]
        },
        "gameplayAttributes": {
            "path": "gameplay/attributes.png",
            "pngBlake3": prefixed(attributes),
            "rawPath": "gameplay/attributes.bin",
            "rawBlake3": prefixed(attributes),
            "rawParsedDocument": {
                "path": "gameplay/attributes.raw.json",
                "blake3": prefixed(attributes_parsed)
            }
        }
    });
    let mut terrain_bytes = serde_json::to_vec_pretty(&terrain).unwrap();
    terrain_bytes.push(b'\n');
    fs::write(source_root.join("terrain.json"), &terrain_bytes).unwrap();
    let plan = NativeTerrainPublicationPlan {
        schema: NATIVE_TERRAIN_PUBLICATION_PLAN_SCHEMA.to_owned(),
        status: "complete-with-placement-blockers".to_owned(),
        source_output_root: slash_path(&batch_root),
        entries: vec![NativeTerrainPublicationEntry {
            scope: "worldMap".to_owned(),
            instance_id: "map_01_03".to_owned(),
            source_root: "maps/map_01_03/terrain".to_owned(),
            destination_root: "assets/game/world/maps/map_01_03/terrain".to_owned(),
            terrain_document: "terrain.json".to_owned(),
            terrain_document_blake3: blake3::hash(&terrain_bytes).to_hex().to_string(),
            heightmap_path: "heightmap.png".to_owned(),
            heightmap_blake3: blake3::hash(heightmap).to_hex().to_string(),
            scene_instance_document: None,
            scene_instance_document_blake3: None,
            environment_document: None,
            environment_document_blake3: None,
            placement_status: "blocked".to_owned(),
            source_bundle: "Map_01_03".to_owned(),
            payloads: Vec::new(),
        }],
        blocked: vec![NativeTerrainPublicationBlocker {
            scope: "worldMap".to_owned(),
            instance_id: "map_01_03".to_owned(),
            stage: "scene-owner-link".to_owned(),
            code: "map_root_trs_missing".to_owned(),
            message: "TerrainData is complete; world root placement is not proven".to_owned(),
            placement_status: "blocked".to_owned(),
        }],
    };
    let plan_path = batch_root.join("publication-plan.json");
    write_json(&plan_path, &plan).unwrap();

    let report = publish_native_terrain_batch(&asset_root, &manifest_path, &plan_path).unwrap();
    assert_eq!(report.entries_published, 1);
    assert_eq!(report.blocked_placements_retained_as_data, 1);
    assert_eq!(report.published_files.len(), 9);
    assert_eq!(
        report.removed_orphan_files,
        vec![stale_scene_path.to_owned()]
    );
    assert!(!asset_root.join(stale_scene_path).exists());
    assert!(
        asset_root
            .join("world/maps/map_01_03/terrain/gameplay/attributes.png")
            .is_file()
    );
    let manifest = read_manifest(&manifest_path).unwrap();
    assert_eq!(manifest.files.len(), 9);
    assert!(
        manifest
            .files
            .iter()
            .all(|entry| !entry.path.to_ascii_lowercase().ends_with(".glb"))
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|entry| entry.path == "world/catalog.json")
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|entry| entry.path == "world/maps/map_01_03/provenance.json")
    );

    let second = publish_native_terrain_batch(&asset_root, &manifest_path, &plan_path).unwrap();
    assert!(second.replaced_files.is_empty());
    assert!(second.removed_orphan_files.is_empty());
}

#[test]
fn frozen_v3_linked_batch_builds_v2_catalog_and_exact_root_chain_scenes() {
    let plan_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../work/native-terrain-contract-smoke-v3/publication-plan.json");
    let plan_bytes = fs::read(&plan_path).unwrap();
    let plan: NativeTerrainPublicationPlan = serde_json::from_slice(&plan_bytes).unwrap();
    assert_eq!(plan.entries.len(), 2);
    assert!(plan.entries.iter().all(|entry| {
        entry.scope == "worldMap"
            && entry.placement_status == "linked"
            && entry.scene_instance_document.as_deref() == Some("scene-instance.json")
    }));
    let (files, roots) =
        collect_native_terrain_batch_files(plan_path.parent().unwrap(), &plan).unwrap();
    assert_eq!(roots.len(), 2);
    let catalog = match &files.get("world/catalog.json").unwrap().source {
        NativeTerrainBatchFileSource::Generated(bytes) => {
            serde_json::from_slice::<Value>(bytes).unwrap()
        }
        NativeTerrainBatchFileSource::File(_) => panic!("catalog must be generated"),
    };
    assert_eq!(
        catalog.get("schema").and_then(Value::as_str),
        Some(WORLD_CATALOG_SCHEMA)
    );
    let entries = catalog.get("entries").and_then(Value::as_array).unwrap();
    assert_eq!(entries.len(), 2);
    for entry in entries {
        let scene_path = entry.get("scene").and_then(Value::as_str).unwrap();
        let scene_hash = entry.get("sceneBlake3").and_then(Value::as_str).unwrap();
        let scene_bytes = match &files.get(scene_path).unwrap().source {
            NativeTerrainBatchFileSource::Generated(bytes) => bytes,
            NativeTerrainBatchFileSource::File(_) => panic!("scene must be generated"),
        };
        assert_eq!(blake3::hash(scene_bytes).to_hex().as_str(), scene_hash);
        let scene: Value = serde_json::from_slice(scene_bytes).unwrap();
        assert_eq!(
            scene.get("schema").and_then(Value::as_str),
            Some(NATIVE_WORLD_SCENE_SCHEMA)
        );
        assert_eq!(
            scene
                .pointer("/nativeTerrain/rootChain/includesOwnerTransform")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            scene.get("root").and_then(|root| root.get("translation")),
            Some(&serde_json::json!([0.0, 0.0, 0.0]))
        );
    }
    assert!(
        files
            .keys()
            .all(|path| !path.to_ascii_lowercase().ends_with("terrain.glb"))
    );
}

#[test]
fn native_terrain_name_falls_back_only_to_proven_terrain_data_true_name() {
    assert_eq!(
        native_terrain_display_name("", "TerrainData_08_06"),
        "TerrainData_08_06"
    );
    assert_eq!(
        native_terrain_display_name("Map_08_06", "TerrainData_08_06"),
        "Map_08_06"
    );
}

#[test]
fn external_glb_uri_must_keep_relative_dependency_route() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let glb = make_glb(&serde_json::json!({
        "asset":{"version":"2.0"},
        "buffers":[{"uri":"mesh.bin","byteLength":4}],
        "nodes":[], "meshes":[], "skins":[], "animations":[]
    }));
    fs::create_dir_all(root.join("models/source")).unwrap();
    fs::write(root.join("models/source/model.glb"), &glb).unwrap();
    fs::write(root.join("models/source/mesh.bin"), b"1234").unwrap();
    let manifest = write_manifest(
        root,
        vec![
            manifest_entry("models/source/model.glb", AssetKind::Model, &glb),
            manifest_entry("models/source/mesh.bin", AssetKind::Data, b"1234"),
        ],
    );
    let model_route = SemanticRoute {
        source_path: "models/source/model.glb".to_owned(),
        destination_path: "world/maps/test/model/model.glb".to_owned(),
        kind: AssetKind::Model,
        category: SemanticCategory::World,
        ownership: proof("TrueRoot"),
        duplicate_group: None,
        variant_reason: None,
        content: RouteContent::CopyExact,
    };
    let mut plan = RoutePlan {
        schema: ROUTE_PLAN_SCHEMA.to_owned(),
        source_build: "retrobution-20260613".to_owned(),
        policy: "test".to_owned(),
        routes: vec![model_route],
    };
    assert!(validate_route_plan(root, &manifest, &plan).is_err());
    plan.routes.push(SemanticRoute {
        source_path: "models/source/mesh.bin".to_owned(),
        destination_path: "world/maps/test/model/mesh.bin".to_owned(),
        kind: AssetKind::Data,
        category: SemanticCategory::World,
        ownership: proof("TrueRoot buffer"),
        duplicate_group: None,
        variant_reason: None,
        content: RouteContent::CopyExact,
    });
    assert!(validate_route_plan(root, &manifest, &plan).is_ok());
}

#[test]
fn publication_is_copy_only_manifested_and_idempotent() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let glb = make_glb(&serde_json::json!({
        "asset":{"version":"2.0"},
        "nodes":[{}], "meshes":[{}], "skins":[{"joints":[0]}], "animations":[{}]
    }));
    fs::create_dir_all(root.join("models")).unwrap();
    fs::write(root.join("models/source.glb"), &glb).unwrap();
    let manifest = write_manifest(
        root,
        vec![manifest_entry("models/source.glb", AssetKind::Model, &glb)],
    );
    let plan = RoutePlan {
        schema: ROUTE_PLAN_SCHEMA.to_owned(),
        source_build: "retrobution-20260613".to_owned(),
        policy: "test".to_owned(),
        routes: vec![SemanticRoute {
            source_path: "models/source.glb".to_owned(),
            destination_path: "world/maps/test/model/source.glb".to_owned(),
            kind: AssetKind::Model,
            category: SemanticCategory::World,
            ownership: proof("TrueRoot"),
            duplicate_group: None,
            variant_reason: None,
            content: RouteContent::CopyExact,
        }],
    };

    let first = publish_route_plan(root, &manifest, &plan).unwrap();
    assert_eq!(first.committed_files.len(), 1);
    assert_eq!(fs::read(root.join("models/source.glb")).unwrap(), glb);
    assert_eq!(
        fs::read(root.join("world/maps/test/model/source.glb")).unwrap(),
        glb
    );
    let second = publish_route_plan(root, &manifest, &plan).unwrap();
    assert_eq!(second.reused_identical_files.len(), 1);
    assert_eq!(read_manifest(&manifest).unwrap().files.len(), 2);
}

#[test]
fn rewritten_scene_changes_only_exact_string_values() {
    let mut value = serde_json::json!({
        "models":[{"path":"models/world/Map/scene_model.glb"}],
        "note":"prefix models/world/Map/scene_model.glb suffix"
    });
    rewrite_json_strings(
        &mut value,
        &BTreeMap::from([(
            "models/world/Map/scene_model.glb".to_owned(),
            "world/maps/map/models/scene_model.glb".to_owned(),
        )]),
    );
    assert_eq!(
        value["models"][0]["path"],
        "world/maps/map/models/scene_model.glb"
    );
    assert_eq!(
        value["note"],
        "prefix models/world/Map/scene_model.glb suffix"
    );
}
