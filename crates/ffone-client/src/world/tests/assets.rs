use super::*;

pub(super) fn asset_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .to_path_buf()
}

#[test]
fn pokey_oaks_frond_guard_keeps_the_asset_and_matches_the_reported_position() {
    let scene_path = asset_root().join("map/tiles/map_12_02/scene.json");
    let scene: NativeWorldScene =
        serde_json::from_slice(&fs::read(scene_path).unwrap()).unwrap();
    let reported_tree = [
        ("#8086", PRIMARY_ETC_TREE_07_FROND_MODEL_PATH),
        ("#8089", PRIMARY_ETC_TREE_07_TRUNK_MODEL_PATH),
    ]
    .map(|(source_node, expected_model)| {
        let (visual_index, visual) = scene
            .visuals
            .iter()
            .enumerate()
            .find(|(_, visual)| visual.name.contains(source_node))
            .expect("the reported Pokey Oaks etc_tree_07 renderer");
        let model = scene.model(&visual.model).unwrap();
        assert_eq!(visual.legacy_layer, Some(12));
        assert_eq!(model.path, expected_model);
        assert!(!is_legacy_collision_helper_visual(&scene, visual));
        assert!(!is_legacy_zero_contribution_visual(&scene, visual));
        (visual_index, visual)
    });

    let source_center = Vec3::from_array(reported_tree[0].1.transform.translation);
    assert_eq!(
        source_center,
        Vec3::from_array(reported_tree[1].1.transform.translation),
        "the frond and trunk are the two renderer parts of one source tree"
    );
    let screenshot_position = Vec3::new(-6440.0, source_center.y, 1216.0);
    let screenshot_distance = screenshot_position.distance(source_center);
    assert!(screenshot_distance > PRIMARY_AUDITED_SMALLSTUFF_PRESENTATION_END_NATIVE);
    assert!(screenshot_distance < EXTENDED_WORLD_CAMERA_FAR_NATIVE);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Gltf>().init_asset::<GltfMesh>();
    let root = app.world_mut().spawn_empty().id();
    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut queue = CommandQueue::default();
    let renderers = {
        let mut commands = Commands::new(&mut queue, app.world());
        reported_tree.map(|(visual_index, visual)| {
            spawn_native_world_visual(
                &mut commands,
                &asset_server,
                &scene,
                root,
                visual_index,
                visual,
            )
            .unwrap()
        })
    };
    queue.apply(app.world_mut());

    for renderer in renderers {
        let entity = app.world().entity(renderer);
        assert!(entity.contains::<PendingNativeWorldVisualAsset>());
        assert!(entity.contains::<NativeWorldObjectRangeMember>());
        assert!(entity.contains::<NativeWorldPrimaryFarPresentationGuard>());
        assert!(!entity.contains::<NativeWorldObjectRangeUnbounded>());
        assert_eq!(entity.get::<Visibility>(), Some(&Visibility::Hidden));
    }
}

pub(super) fn rewrite_registry(root: &Path, mutate: impl FnOnce(&mut serde_json::Value)) {
    let registry_path = join_relative(root, RUNTIME_WORLD_REGISTRY_PATH);
    let mut registry: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
    mutate(&mut registry);
    write_pretty_json(&registry_path, &registry);
}

#[test]
fn runtime_registry_opens_after_all_2405_conversion_metadata_files_are_absent() {
    let fixture = write_runtime_world_fixture();
    let technical_counts = [
        ("world/catalog.json", 1_usize),
        ("provenance.json", 173),
        ("terrain/manifest.json", 173),
        ("terrain/scene-instance.json", 170),
        ("terrain/components/*.parsed.json", 850),
        ("terrain/environment/source/*.parsed.json", 692),
        ("terrain/details/detail-database.raw.json", 173),
        ("terrain/details/trees.raw.json", 173),
    ];
    assert_eq!(
        technical_counts
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>(),
        2_405
    );
    for relative in [
        NATIVE_WORLD_CATALOG_PATH,
        "world/maps/map_08_06/provenance.json",
        "world/maps/map_08_06/terrain/manifest.json",
        "world/maps/map_08_06/terrain/scene-instance.json",
        "world/maps/map_08_06/terrain/components/transform.parsed.json",
        "world/maps/map_08_06/terrain/environment/source/dong-color-setup.parsed.json",
        "world/maps/map_08_06/terrain/details/detail-database.raw.json",
        "world/maps/map_08_06/terrain/details/trees.raw.json",
    ] {
        assert!(
            !join_relative(fixture.path(), relative).exists(),
            "conversion-only file unexpectedly survived: {relative}"
        );
    }

    let catalog = NativeWorldCatalog::open(fixture.path()).unwrap();
    assert_eq!(catalog.scenes().len(), 1);
    assert_eq!(catalog.blocked_count(), 0);
    assert_eq!(catalog.scenes()[0].name, "map_08_06");
    assert!(catalog.scenes()[0].provenance.is_none());
    let terrain = catalog.scenes()[0].native_terrain.as_ref().unwrap();
    assert!(terrain.scene_instance_path.is_none());
    assert!(terrain.scene_instance_blake3.is_none());
}

#[test]
fn runtime_registry_retains_scripted_asset_hashes_for_streaming() {
    let fixture = write_runtime_world_fixture();
    let behaviour_path = "map/tiles/map_08_06/behaviour.json";
    let objects_path = "map/tiles/map_08_06/objects.json";
    fs::create_dir_all(fixture.path().join("map/tiles/map_08_06")).unwrap();
    fs::write(fixture.path().join(behaviour_path), b"behaviour").unwrap();
    fs::write(fixture.path().join(objects_path), b"objects").unwrap();
    rewrite_registry(fixture.path(), |registry| {
        registry["entries"][0]["behaviour"] = serde_json::json!({
            "path": behaviour_path,
            "blake3": blake3::hash(b"behaviour").to_hex().to_string()
        });
        registry["entries"][0]["objects"] = serde_json::json!({
            "path": objects_path,
            "blake3": blake3::hash(b"objects").to_hex().to_string()
        });
    });

    let catalog = NativeWorldCatalog::open(fixture.path()).unwrap();
    let (path, hash) = catalog
        .behaviour_asset(NativeWorldScope::WorldMap, "map_08_06")
        .unwrap();
    assert_eq!(path, fixture.path().join(behaviour_path));
    assert_eq!(hash, blake3::hash(b"behaviour").to_hex().as_str());
    assert!(
        catalog
            .object_asset(NativeWorldScope::WorldMap, "map_08_06")
            .is_some()
    );
}

#[test]
fn runtime_registry_rejects_path_hash_and_schema_drift() {
    let fixture = write_runtime_world_fixture();
    rewrite_registry(fixture.path(), |registry| {
        registry["entries"][0]["scene"]["blake3"] = "0".repeat(64).into();
    });
    let error = NativeWorldCatalog::open(fixture.path()).unwrap_err();
    assert!(error.message().contains("native metadata hash mismatch"));

    let fixture = write_runtime_world_fixture();
    rewrite_registry(fixture.path(), |registry| {
        registry["entries"][0]["scene"]["path"] = "map/tiles/map_08_06/not-scene.json".into();
    });
    let error = NativeWorldCatalog::open(fixture.path()).unwrap_err();
    assert!(
        error
            .message()
            .contains("scope/tile/path identity mismatch")
    );

    let fixture = write_runtime_world_fixture();
    rewrite_registry(fixture.path(), |registry| {
        registry["sourceBuild"] = "Retrobution-20260613".into();
    });
    let error = NativeWorldCatalog::open(fixture.path()).unwrap_err();
    assert!(error.message().contains("unknown field"));
}

#[test]
fn slider_idle_rider_keeps_support_and_publishes_route_positions() {
    let collider = published_gltf_collider_from_glb(
        "characters/transportation/downtown_bus/DT_ETC_Downtownbus_A_00.collision.glb",
    );
    let base = published_character_node_world_transform(
        "characters/transportation/downtown_bus/DT_ETC_Downtownbus_A_00.glb",
        "collision",
        1.0,
    );
    let bounds =
        AuthoredColliderWorldBounds::from_collider(&collider, &GlobalTransform::from(base))
            .unwrap();
    let (height, _) = collider_capsule_ground_contact_with_bounds(
        &collider, base, &bounds, 3.0, 0.0, 5.0, -5.0, 5.0,
    )
    .expect("authored Slider deck beneath rider");
    let initial = Vec3::new(3.0, height, 0.0);
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .init_resource::<crate::movement::LegacyInputState>()
        .init_resource::<crate::movement::MovementIntentQueue>()
        .add_systems(
            Update,
            (
                crate::movement::simulate_legacy_players,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    let support = app
        .world_mut()
        .spawn((GlobalTransform::from(base), collider, bounds))
        .id();
    let rider = app
        .world_mut()
        .spawn((
            Transform::from_translation(initial),
            LegacyPlayerController::from_baseline_table(),
            NativeWorldGroundSupport {
                collider: support,
                last_world_from_local: base,
                surface_normal: Vec3::Y,
            },
        ))
        .id();
    let mut previous = base;
    let mut packets = 0;
    for frame in 1..=600 {
        let t = frame as f32 / 60.0;
        let offset = Vec3::new(t * 12.0, (t * 2.0).sin() * 0.2, t * 3.0);
        let pose = Mat4::from_translation(offset) * base;
        let collider = app.world().get::<AuthoredTriMeshCollider>(support).unwrap();
        let bounds =
            AuthoredColliderWorldBounds::from_collider(collider, &GlobalTransform::from(pose))
                .unwrap();
        app.world_mut().entity_mut(support).insert((
            GlobalTransform::from(pose),
            bounds,
            RuntimeAuthoredColliderMotion {
                previous_world_from_local: previous,
                current_world_from_local: pose,
            },
        ));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
        app.update();
        let position = app.world().get::<Transform>(rider).unwrap().translation;
        assert!(
            position.abs_diff_eq(initial + offset, 0.02),
            "rider lost the moving deck at frame {frame}: {position:?}"
        );
        assert!(
            app.world().get::<NativeWorldGroundSupport>(rider).is_some(),
            "rider lost support at frame {frame}"
        );
        let mut queue = app
            .world_mut()
            .resource_mut::<crate::movement::MovementIntentQueue>();
        for packet in queue.take_all() {
            let crate::movement::MovementIntent::Stop(packet) = packet else {
                panic!("idle rider should send position without input movement");
            };
            assert_eq!(
                packet.position,
                ProtocolPosition::from_native(position).raw()
            );
            packets += 1;
        }
        previous = pose;
    }
    assert!(
        packets >= 10,
        "server must receive positions throughout the ride: {packets}"
    );
}

#[test]
fn world_and_tutorial_catalog_scopes_allow_the_same_dong_key_but_not_duplicates() {
    let mut world = first_scene();
    world.scope = Some(NativeWorldScope::WorldMap);
    let mut tutorial = world.clone();
    tutorial.name = "Tutorial same spatial key".to_owned();
    tutorial.scope = Some(NativeWorldScope::Tutorial);
    let (world_index, tutorial_index) =
        build_catalog_indices(&[world.clone(), tutorial]).unwrap();
    assert_eq!(world_index.len(), 1);
    assert_eq!(tutorial_index.len(), 1);

    let mut duplicate = world.clone();
    duplicate.name = "Duplicate world key".to_owned();
    assert!(build_catalog_indices(&[world, duplicate]).is_err());
}
