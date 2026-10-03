use super::*;

#[test]
fn terrain_motion_runtime_keeps_accelerated_players_above_ground() {
    for speed in [600, 900, 1800] {
        for delta in [1.0 / 60.0, 0.1, 0.2] {
            let heights = (0..9)
                .flat_map(|_| {
                    (0..65).map(|column| ((column as f32 - 2.0) * 0.9).clamp(0.0, 4.5))
                })
                .collect::<Vec<_>>();
            let terrain = NativeHeightmapCollider::test_heightfield(65, 9, &heights);
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .insert_resource(TimeUpdateStrategy::ManualDuration(
                    std::time::Duration::from_secs_f64(delta),
                ))
                .insert_resource(crate::movement::LegacyInputState {
                    local_axis: Vec2::NEG_X,
                    ..default()
                })
                .init_resource::<MovementIntentQueue>()
                .init_resource::<NativeTerrainSpatialRegistry>()
                .init_resource::<AuthoredColliderSpatialIndex>()
                .add_systems(
                    Update,
                    (
                        crate::movement::simulate_legacy_players,
                        sync_native_terrain_spatial_registry,
                        resolve_authored_world_ground,
                    )
                        .chain(),
                );
            app.world_mut()
                .spawn((GlobalTransform::IDENTITY, terrain.clone()));
            let player = app
                .world_mut()
                .spawn((
                    Transform::from_xyz(-1.0, 0.0, 4.0),
                    LegacyPlayerController::from_server_attributes(speed, 568).with_yaw(180.0),
                ))
                .id();
            for _ in 0..14 {
                app.update();
                let position = app.world().get::<Transform>(player).unwrap().translation;
                let floor = terrain
                    .ground_height(
                        &GlobalTransform::IDENTITY,
                        position.x,
                        position.z,
                        -10.0,
                        10.0,
                    )
                    .unwrap_or_else(|| panic!("left test terrain: speed={speed}, delta={delta}, position={position:?}"));
                assert!(
                    position.y >= floor - 0.04,
                    "speed={speed}, delta={delta}, position={position:?}, floor={floor}"
                );
            }
            assert!(app.world().get::<Transform>(player).unwrap().translation.x < -1.5);
        }
    }
}

#[test]
fn terrain_motion_sweeps_fast_uphill_and_airborne_crossings() {
    let heights = (0..9)
        .flat_map(|_| (0..9).map(|column| ((column as f32 - 2.0) * 0.9).max(0.0)))
        .collect::<Vec<_>>();
    let terrain = NativeHeightmapCollider::test_heightfield(9, 9, &heights);
    let global = GlobalTransform::IDENTITY;
    for (start, displacement, moving_up) in [
        (
            Vec3::new(-1.0, 0.0, 4.0),
            Vec3::new(-3.0, -0.000001, 0.0),
            false,
        ),
        (Vec3::new(-1.0, 1.0, 4.0), Vec3::new(-4.0, -0.8, 0.0), false),
        (Vec3::new(-1.0, 0.0, 4.0), Vec3::new(-4.0, 0.5, 0.0), true),
    ] {
        let (minimum, maximum) =
            authored_wall_spatial_query_bounds(start, start + displacement);
        let (patch, bounds) =
            authored_terrain_motion_patch([(&global, &terrain)], minimum, maximum).unwrap();
        let result = resolve_authored_controller_motion_with_bounds(
            start,
            displacement,
            displacement.y,
            &[(Mat4::IDENTITY, &patch, &bounds)],
            moving_up,
        );
        let floor = terrain
            .ground_height(&global, result.position.x, result.position.z, -10.0, 10.0)
            .unwrap();
        assert!(
            result.position.y >= floor - 0.04,
            "sweep tunneled: start={start:?}, displacement={displacement:?}, result={:?}, floor={floor}",
            result.position
        );
        assert!(result.position.x < start.x - 0.1, "motion was discarded");
    }
}

#[test]
fn terrain_motion_patch_keeps_tile_edges_and_excludes_interior_overhead() {
    let terrain = NativeHeightmapCollider::test_heightfield(3, 3, &[0.0; 9]);
    let global = GlobalTransform::from_translation(Vec3::new(10.0, 50.0, 20.0));
    assert!(
        authored_terrain_motion_patch(
            [(&global, &terrain)],
            Vec3::new(7.9, 49.0, 20.9),
            Vec3::new(8.1, 52.0, 21.1)
        )
        .is_some()
    );
    assert!(
        authored_terrain_motion_patch(
            [(&global, &terrain)],
            Vec3::new(8.0, -1.0, 20.0),
            Vec3::new(10.0, 2.0, 22.0)
        )
        .is_none()
    );
    assert!(
        authored_terrain_motion_patch(
            [(&global, &terrain)],
            Vec3::new(100.0, 49.0, 100.0),
            Vec3::new(101.0, 52.0, 101.0)
        )
        .is_none()
    );
}

#[test]
fn foster_warp_keeps_interior_floor_below_outdoor_terrain() {
    let assets = crate::assets::AssetLocator::open(asset_root()).unwrap();
    let content =
        crate::tutorial_mission_content::TutorialMissionContent::open(&assets).unwrap();
    let warp = content.normal_gameplay_warp_for_npc(1464).unwrap();
    assert_eq!(warp.target.map_id, 0);
    let destination =
        ProtocolPosition::new([warp.target.x, warp.target.y, warp.target.z]).to_native();
    let catalog = NativeWorldCatalog::open(asset_root()).unwrap();
    let scene = catalog.select(destination).unwrap();
    let instance = scene.native_terrain.as_ref().unwrap();
    let terrain = catalog.load_terrain(instance).unwrap();
    let terrain_global = GlobalTransform::from(scene.terrain_world_matrix(instance).unwrap());
    let heightmap = NativeHeightmapCollider::from_terrain(&terrain, Handle::default());
    let outdoor_height = heightmap
        .ground_height(
            &terrain_global,
            destination.x,
            destination.z,
            -1000.0,
            1000.0,
        )
        .unwrap();
    assert!(outdoor_height > destination.y + 50.0);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .init_resource::<crate::movement::LegacyInputState>()
        .init_resource::<MovementIntentQueue>()
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                crate::movement::simulate_legacy_players,
                sync_native_terrain_spatial_registry,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    app.world_mut().spawn((terrain_global, heightmap));
    let mut floor = None::<f32>;
    for placement in &scene.colliders {
        if placement.is_trigger || placement.transform.translation[1] > -100.0 {
            continue;
        }
        let model = scene
            .models
            .iter()
            .find(|model| model.id == placement.model)
            .unwrap();
        let collider = authored_collider_from_glb_mesh(&model.path, placement.mesh);
        let global = GlobalTransform::from(
            scene.root.try_to_bevy("scene").unwrap().to_matrix()
                * placement
                    .transform
                    .try_to_bevy("collider")
                    .unwrap()
                    .to_matrix(),
        );
        let bounds = AuthoredColliderWorldBounds::from_collider(&collider, &global).unwrap();
        if let Some(height) = collider_ground_height_with_bounds(
            &collider,
            global.to_matrix(),
            &bounds,
            destination.x,
            destination.z,
            destination.y - 5.0,
            destination.y + 0.1,
        ) {
            floor = Some(floor.map_or(height, |old| old.max(height)));
        }
        app.world_mut().spawn((global, collider, bounds));
    }
    let floor = floor.expect("published Foster interior must provide a floor at the warp");
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.apply_authoritative_teleport(destination);
    let player = app
        .world_mut()
        .spawn((Transform::from_translation(destination), controller))
        .id();
    for _ in 0..120 {
        app.update();
        let position = app.world().get::<Transform>(player).unwrap().translation;
        assert!(
            position.y < outdoor_height - 50.0,
            "interior warp ejected onto terrain: {position:?}"
        );
    }
    let position = app.world().get::<Transform>(player).unwrap().translation;
    assert!(
        (position.y - floor).abs() < 0.1,
        "interior floor={floor}, player={position:?}"
    );
    assert!(
        app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .grounded
    );

    // The same recovery still repairs a shallow penetration outdoors.
    let shallow = destination.with_y(outdoor_height - 0.15);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = shallow;
    app.world_mut()
        .get_mut::<LegacyPlayerController>(player)
        .unwrap()
        .apply_authoritative_teleport(shallow);
    app.update();
    assert!(
        (app.world().get::<Transform>(player).unwrap().translation.y - outdoor_height).abs()
            < 0.1
    );
}

pub(super) fn sanitize_runtime_terrain(value: &mut serde_json::Value) {
    let object = value.as_object_mut().unwrap();
    object.remove("source");
    object.remove("sceneInstance");
    if let Some(attributes) = object
        .get_mut("gameplayAttributes")
        .and_then(serde_json::Value::as_object_mut)
    {
        attributes.remove("rawParsedDocument");
        attributes.remove("source");
    }
    if let Some(details) = object
        .get_mut("detailAndTrees")
        .and_then(serde_json::Value::as_object_mut)
    {
        details.remove("assetClosure");
        details.remove("rawDocument");
        if let Some(trees) = details.get_mut("trees") {
            remove_key_recursively(trees, "rawDocument");
        }
    }
    remove_key_recursively(
        object
            .get_mut("splat")
            .expect("production terrain has splat content"),
        "sourceEncoded",
    );
    if let Some(splat) = object
        .get_mut("splat")
        .and_then(serde_json::Value::as_object_mut)
    {
        if let Some(weights) = splat
            .get_mut("weightMaps")
            .and_then(serde_json::Value::as_array_mut)
        {
            for weight in weights {
                weight.as_object_mut().unwrap().remove("source");
            }
        }
        if let Some(layers) = splat
            .get_mut("layers")
            .and_then(serde_json::Value::as_array_mut)
        {
            for layer in layers {
                let layer = layer.as_object_mut().unwrap();
                layer.remove("modeSource");
                layer.remove("modeEvidence");
                layer
                    .get_mut("albedo")
                    .and_then(serde_json::Value::as_object_mut)
                    .unwrap()
                    .remove("source");
            }
        }
    }
    if let Some(lightmap) = object
        .get_mut("lightmap")
        .and_then(serde_json::Value::as_object_mut)
    {
        lightmap.remove("sourcePointer");
        lightmap.remove("source");
        remove_key_recursively(
            object
                .get_mut("lightmap")
                .expect("lightmap was present immediately above"),
            "sourceEncoded",
        );
    }
}

#[test]
fn first_scene_is_bound_to_the_lossless_native_heightmap() {
    let scene = first_scene();
    let instance = scene.native_terrain.as_ref().unwrap();
    assert_eq!(instance.path, "map/tiles/map_08_06/terrain/terrain.json");
    assert_eq!(instance.true_name, "TerrainData_08_06");
    assert_eq!(instance.terrain_collider_path_id, 15_281);
    assert_eq!(instance.terrain_game_object_path_id, 15_279);
    assert_eq!(instance.terrain_transform_path_id, 742);
    let terrain = instance.loaded.as_ref().unwrap();
    assert_eq!(terrain.geometry().vertex_count(), 129 * 129);
    assert_eq!(terrain.geometry().index_count(), 128 * 128 * 6);
    assert_eq!(terrain.descriptor().splat.layers.len(), 15);
    assert_eq!(terrain.descriptor().splat.weight_maps.len(), 4);
}

#[test]
fn canonical_world_scenes_use_native_terrain_json_and_static_model_glbs() {
    for relative in NATIVE_WORLD_SCENE_PATHS {
        let bytes = fs::read(join_relative(&asset_root(), relative)).unwrap();
        let scene: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let terrain = scene
            .get("nativeTerrain")
            .and_then(|value| value.get("path"))
            .and_then(serde_json::Value::as_str)
            .expect("every canonical world scene must retain native terrain");
        assert!(
            terrain.ends_with("/terrain/terrain.json"),
            "{} uses an unexpected native terrain path: {}",
            relative,
            terrain
        );
        assert!(
            scene
                .get("models")
                .and_then(serde_json::Value::as_array)
                .expect("canonical world scene models must be an array")
                .iter()
                .all(|model| model
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|path| path.to_ascii_lowercase().ends_with(".glb"))),
            "{relative} contains a static model that is not a native GLB"
        );
    }
}

#[test]
fn world_policy_rejects_axis_conversion_invalid_terrain_trs_and_path() {
    let mut scene = first_scene();
    scene.validate_with_policy(false).unwrap();
    scene.coordinate_contract.gameplay_facing_rotation_applied = true;
    assert!(scene.validate_with_policy(false).is_err());

    let mut scene = first_scene();
    scene.root.rotation = [0.0, 0.0, 0.0, 2.0];
    assert!(scene.validate_with_policy(false).is_err());

    let mut scene = first_scene();
    scene.native_terrain.as_mut().unwrap().transform.scale = [1.0, 0.0, 1.0];
    assert!(scene.validate_with_policy(false).is_err());

    let mut scene = first_scene();
    scene.native_terrain.as_mut().unwrap().path = "../outside.json".to_owned();
    assert!(scene.validate_with_policy(false).is_err());
}

#[test]
fn native_terrain_registry_uses_legacy_dong_key_in_reflected_space() {
    let key_08_06 = ((8_i32 & 0xffff) << 16) | (6_i32 & 0xffff);
    assert_eq!(native_dong_key(-4096.0, 3072.0), Some(key_08_06));
    assert_eq!(native_dong_key(-4607.9, 3583.9), Some(key_08_06));
    assert_ne!(native_dong_key(-4608.0, 3072.0), Some(key_08_06));
    assert_eq!(native_dong_key(f32::NAN, 0.0), None);

    let first = Entity::from_raw_u32(1).unwrap();
    let second = Entity::from_raw_u32(2).unwrap();
    let mut registry = NativeTerrainSpatialRegistry::default();
    registry.insert_entity(first, key_08_06);
    assert_eq!(
        registry.lookup(-4200.0, 3200.0),
        NativeTerrainSpatialLookup::Found(first)
    );
    registry.insert_entity(second, key_08_06);
    assert_eq!(
        registry.lookup(-4200.0, 3200.0),
        NativeTerrainSpatialLookup::DuplicateKey {
            dong_key: key_08_06,
            entities: 2
        }
    );
    registry.remove_entity(second);
    assert_eq!(
        registry.lookup(-4200.0, 3200.0),
        NativeTerrainSpatialLookup::Found(first)
    );
}

#[test]
fn published_v2_smoke_lazily_verifies_frozen_native_terrain_v1() {
    let published_root =
        asset_root().join("../../work/native-terrain-publisher-smoke-v4-2/assets/game");
    let catalog = NativeWorldCatalog::open(&published_root).unwrap();
    assert_eq!(catalog.scenes().len(), 2);
    assert_eq!(catalog.cached_terrain_count(), 0);
    let map_08 = catalog.select(Vec3::new(-4096.0, -300.0, 3072.0)).unwrap();
    assert_eq!(map_08.scope, Some(NativeWorldScope::WorldMap));
    assert!(map_08.native_terrain.as_ref().unwrap().loaded.is_none());
    let terrain_08 = catalog
        .load_terrain(map_08.native_terrain.as_ref().unwrap())
        .unwrap();
    assert_eq!(terrain_08.geometry().vertex_count(), 129 * 129);
    assert_eq!(terrain_08.geometry().index_count(), 128 * 128 * 6);
    assert!(terrain_08.gameplay_attributes_available());
    let map_02 = catalog.select(Vec3::new(-1024.0, -300.0, 2560.0)).unwrap();
    let terrain_02 = catalog
        .load_terrain(map_02.native_terrain.as_ref().unwrap())
        .unwrap();
    assert_eq!(terrain_02.geometry().vertex_count(), 129 * 129);
    assert_eq!(catalog.cached_terrain_count(), 2);

    fn spawn_smoke(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        catalog: Res<NativeWorldCatalog>,
    ) {
        let scene = catalog.select(Vec3::new(-4096.0, -300.0, 3072.0)).unwrap();
        spawn_native_world_scene(&mut commands, &asset_server, &catalog, scene).unwrap();
    }
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .insert_resource(catalog)
        .add_systems(Startup, spawn_smoke);
    app.update();
    let mut scene_entities = app
        .world_mut()
        .query_filtered::<(&Name, &Transform), With<NativeWorldSceneEntity>>();
    let transforms = scene_entities
        .iter(app.world())
        .map(|(name, transform)| (name.as_str().to_owned(), *transform))
        .collect::<Vec<_>>();
    assert_eq!(
        transforms.len(),
        3,
        "outer scene, one exact rootChain node, and one terrain owner must be spawned"
    );
    let mut roots = app.world_mut().query_filtered::<
        (&Visibility, &NativeWorldPresentationStatus),
        With<NativeWorldSceneRoot>,
    >();
    let (visibility, presentation) = roots.single(app.world()).unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(*presentation, NativeWorldPresentationStatus::Loading);
    assert!(
        transforms
            .iter()
            .any(|(_, transform)| { transform.translation == Vec3::new(-4096.0, 0.0, 3072.0) })
    );
    assert!(
        transforms
            .iter()
            .any(|(_, transform)| { transform.translation == Vec3::new(0.0, -300.0, 0.0) })
    );
}

#[test]
fn grounded_search_follows_walkable_terrain_without_reaching_a_real_drop() {
    let previous = Vec3::new(0.0, 5.0, 0.0);
    let current = Vec3::new(0.1, 5.0, 0.0);
    let (minimum, maximum) = authored_player_ground_search_range(previous, current, true);
    let expected_descent =
        0.1 * AUTHORED_WALKABLE_MAX_TANGENT + AUTHORED_GROUND_SWEEP_TOLERANCE;
    assert!((minimum - (5.0 - expected_descent)).abs() <= f32::EPSILON);
    assert!((maximum - (5.0 + GROUNDED_STEP_UP)).abs() <= f32::EPSILON);

    let (capped_minimum, _) =
        authored_player_ground_search_range(previous, Vec3::new(1.0, 5.0, 0.0), true);
    assert!((capped_minimum - (5.0 - GROUNDED_STEP_UP)).abs() <= f32::EPSILON);
    assert!(
        capped_minimum > 4.6,
        "the grounded continuation must not reacquire the 0.6 m bench drop"
    );

    let (airborne_minimum, _) = authored_player_ground_search_range(previous, current, false);
    assert!(
        airborne_minimum > 4.99,
        "airborne motion must retain the exact swept segment instead of slope continuation"
    );
}
