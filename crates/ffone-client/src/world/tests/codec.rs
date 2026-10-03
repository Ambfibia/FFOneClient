use super::*;

#[test]
fn streamed_scene_admission_never_exceeds_per_frame_budgets() {
    let mut pending = PendingNativeWorldSceneSpawn::new(None);
    pending.terrain_spawned = true;
    let mut visual_indices = Vec::new();
    let mut collider_indices = Vec::new();
    let mut frames = 0;

    while !pending.is_complete(130, 70) {
        let (visuals, colliders) = pending.next_batch(
            130,
            70,
            NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME,
            NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME,
        );
        assert!(visuals.len() <= NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME);
        assert!(colliders.len() <= NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME);
        visual_indices.extend(visuals);
        collider_indices.extend(colliders);
        frames += 1;
    }

    let expected_frames = 130_usize
        .div_ceil(NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME)
        .max(70_usize.div_ceil(NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME));
    assert_eq!(frames, expected_frames);
    assert_eq!(visual_indices, (0..130).collect::<Vec<_>>());
    assert_eq!(collider_indices, (0..70).collect::<Vec<_>>());

    let mut paused = PendingNativeWorldSceneSpawn::new(None);
    let (visuals, colliders) = paused.next_batch(10, 10, 0, 0);
    assert!(visuals.is_empty());
    assert!(colliders.is_empty());
    assert_eq!(paused.next_visual, 0);
    assert_eq!(paused.next_collider, 0);
}

#[test]
fn residency_visibility_update_tolerates_same_frame_streamed_mesh_despawn() {
    let mut app = App::new();
    app.init_resource::<NativeWorldObjectRangeCameraCache>()
        .add_systems(
            Update,
            (
                despawn_before_residency,
                update_native_world_object_residency,
            )
                // Reproduce the production race deterministically: queue
                // the unload first, queue residency second, then apply
                // both command buffers at the schedule boundary.
                .chain_ignore_deferred(),
        );
    let center = Vec3::new(-3500.0, -50.0, 4500.0);
    app.world_mut().spawn((
        Camera3d::default(),
        LegacyOrbitCamera::new(Entity::PLACEHOLDER),
        Transform::from_translation(center),
    ));
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "same-frame-unload".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldVisualPresentationStatus::Ready,
            DespawnBeforeResidency,
        ))
        .id();
    let tag = pack_native_world_object_range(center, 100.0).unwrap();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldObjectRangeContract {
                tag,
                meshes: Vec::new(),
                visible: false,
            },
            Mesh3d::default(),
            Visibility::Hidden,
        ))
        .id();
    app.world_mut()
        .get_mut::<NativeWorldObjectRangeContract>(visual)
        .unwrap()
        .meshes
        .push(visual);

    app.update();

    assert!(
        app.world().get_entity(visual).is_err(),
        "the renderer was unloaded; its deferred residency update must be a harmless no-op"
    );
    assert!(app.world().get_entity(root).is_err());
}

#[test]
fn v2_catalog_opens_linked_metadata_without_reading_terrain_payload() {
    let source_path = asset_root()
        .join("../../work/native-terrain-contract-smoke-v3/maps/map_02_05/terrain/scene-instance.json");
    let source_bytes = fs::read(&source_path).unwrap();
    let source: serde_json::Value = serde_json::from_slice(&source_bytes).unwrap();
    let descriptor_blake3 = "1".repeat(64);
    let scene_instance_blake3 = blake3::hash(&source_bytes).to_hex().to_string();
    let terrain_data = source.get("terrainData").unwrap();
    let linkage = source.get("linkage").unwrap();
    let map_scene = source.get("mapScene").unwrap();
    let root_chain = source.get("rootChain").unwrap();
    let scene = serde_json::json!({
        "schema": NATIVE_WORLD_SCENE_SCHEMA,
        "name": "02_05",
        "scope": "worldMap",
        "tile": [2, 5],
        "coverage": "native-heightmap",
        "coordinateContract": {
            "schema": NATIVE_COORDINATE_CONTRACT_SCHEMA,
            "space": "native",
            "basis": WORLD_BASIS,
            "unitScale": WORLD_UNIT_SCALE,
            "originPolicy": WORLD_ORIGIN_POLICY,
            "gameplayFacingRotationApplied": false
        },
        "provenance": {
            "sourceBuild": "retrobution-20260613",
            "sourceArchive": map_scene.get("path").unwrap(),
            "sourceArchiveBlake3": map_scene.get("blake3").unwrap().as_str().unwrap().strip_prefix("blake3:").unwrap(),
            "sourceAsset": terrain_data.get("sourceAssetName").unwrap(),
            "rootTransformPathId": source.get("mapTileRootTransformPathId").unwrap()
        },
        "root": {
            "translation": [0.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        },
        "models": [],
        "visuals": [],
        "colliders": [],
        "nativeTerrain": {
            "name": terrain_data.get("trueName").unwrap(),
            "path": "world/maps/map_02_05/terrain/terrain.json",
            "blake3": descriptor_blake3,
            "trueName": terrain_data.get("trueName").unwrap(),
            "sourceGameObjectTrueName": linkage.pointer("/gameObject/trueName").unwrap(),
            "terrainDataPathId": terrain_data.get("sourcePathId").unwrap(),
            "terrainColliderPathId": linkage.pointer("/terrainCollider/pathId").unwrap(),
            "terrainGameObjectPathId": linkage.pointer("/gameObject/pathId").unwrap(),
            "terrainTransformPathId": linkage.pointer("/transform/pathId").unwrap(),
            "parentTransformPathId": linkage.pointer("/transform/parentTransformPathId").unwrap(),
            "terrainComponentPathId": linkage.pointer("/terrainComponent/pathId").unwrap(),
            "terrainRenderContract": linkage.pointer("/terrainComponent/renderContract").unwrap(),
            "sceneInstancePath": "world/maps/map_02_05/terrain/scene-instance.json",
            "sceneInstanceBlake3": scene_instance_blake3,
            "transform": source.get("localTransform").unwrap(),
            "rootChain": root_chain
        }
    });
    let mut scene_bytes = serde_json::to_vec_pretty(&scene).unwrap();
    scene_bytes.push(b'\n');
    let scene_blake3 = blake3::hash(&scene_bytes).to_hex().to_string();
    let catalog = serde_json::json!({
        "schema": NATIVE_WORLD_CATALOG_SCHEMA,
        "sourceBuild": "retrobution-20260613",
        "status": "native-heightmap-batch-with-typed-placement-blockers",
        "terrainGlbAllowed": false,
        "entries": [{
            "scope": "worldMap",
            "instanceId": "02_05",
            "tile": [2, 5],
            "placementStatus": "linked",
            "scene": "world/maps/map_02_05/scene.json",
            "sceneBlake3": scene_blake3,
            "terrainDescriptor": "world/maps/map_02_05/terrain/terrain.json",
            "terrainDescriptorBlake3": descriptor_blake3,
            "provenance": "world/maps/map_02_05/provenance.json"
        }],
        "blocked": []
    });
    let temp = TempDir::new().unwrap();
    let scene_instance_path = temp
        .path()
        .join("world/maps/map_02_05/terrain/scene-instance.json");
    fs::create_dir_all(scene_instance_path.parent().unwrap()).unwrap();
    fs::write(&scene_instance_path, source_bytes).unwrap();
    fs::write(
        temp.path().join("world/maps/map_02_05/scene.json"),
        scene_bytes,
    )
    .unwrap();
    fs::create_dir_all(temp.path().join("world")).unwrap();
    fs::write(
        temp.path().join("world/catalog.json"),
        serde_json::to_vec_pretty(&catalog).unwrap(),
    )
    .unwrap();

    assert!(
        !temp
            .path()
            .join("world/maps/map_02_05/terrain/terrain.json")
            .exists()
    );
    let loaded = NativeWorldCatalog::open(temp.path()).unwrap();
    assert_eq!(loaded.scenes().len(), 1);
    assert_eq!(loaded.cached_terrain_count(), 0);
    let selected = loaded.select(Vec3::new(-1024.0, -300.0, 2560.0)).unwrap();
    assert_eq!(selected.tile, [2, 5]);
    assert!(selected.native_terrain.as_ref().unwrap().loaded.is_none());
}

#[test]
fn authored_wall_motion_cannot_skip_a_finite_edge_between_frame_endpoints() {
    // The capsule travels past the end of a very narrow wall. Its support
    // ray misses both triangle faces, and neither frame endpoint overlaps
    // the mesh, but the continuous path intersects the rounded edge
    // contact volume. This was the remaining pass-through case in play.
    let narrow_wall = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/narrow-wall.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, -0.02),
            Vec3::new(0.0, 3.0, -0.02),
            Vec3::new(0.0, 0.0, 0.02),
            Vec3::new(0.0, 3.0, 0.02),
        ]
        .into(),
        indices: vec![0, 1, 2, 2, 1, 3].into(),
        local_min: Vec3::new(0.0, 0.0, -0.02),
        local_max: Vec3::new(0.0, 3.0, 0.02),
        is_trigger: false,
    };
    let previous = Vec3::new(1.0, 0.0, 0.2);
    let displacement = Vec3::new(-2.0, 0.0, 0.0);
    assert!(
        collider_wall_hit(
            &narrow_wall,
            Mat4::IDENTITY,
            previous,
            previous + displacement,
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
        )
        .is_none(),
        "the regression must exercise finite-edge recovery, not a face hit"
    );
    assert!(
        collider_wall_penetration(&narrow_wall, Mat4::IDENTITY, previous + displacement)
            .is_none(),
        "the final endpoint alone must be outside the edge contact volume"
    );

    let colliders = [(Mat4::IDENTITY, &narrow_wall)];
    let resolved = resolve_authored_wall_motion(previous, displacement, &colliders);
    assert!(
        (resolved.z - (previous.z + displacement.z)).abs() > 0.01,
        "continuous motion must observe and slide around the finite edge: {resolved:?}"
    );
}
