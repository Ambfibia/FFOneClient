use super::*;

#[test]
fn tutorial_world_fallback_root_preserves_both_scope_identities() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let scene = catalog
        .select_in_scope(NativeWorldScope::Tutorial, Vec3::new(-547.0, -105.4, 655.0))
        .unwrap();
    assert_eq!(scene.scope, Some(NativeWorldScope::WorldMap));

    let mut app = App::new();
    let mut queue = CommandQueue::default();
    let root = {
        let mut commands = Commands::new(&mut queue, app.world());
        spawn_native_world_scene_root(&mut commands, scene, NativeWorldScope::Tutorial).unwrap()
    };
    queue.apply(app.world_mut());

    let root = app.world().get::<NativeWorldSceneRoot>(root).unwrap();
    assert_eq!(root.scope, NativeWorldScope::WorldMap);
    assert_eq!(root.selection_scope, NativeWorldScope::Tutorial);
}

#[test]
fn tutorial_override_replaces_only_its_matching_world_fallback_tile() {
    let mut catalog = load_native_world_scenes(asset_root()).unwrap();
    let mut tutorial = catalog
        .scene_for_tile(NativeWorldScope::WorldMap, [0, 0])
        .unwrap()
        .clone();
    tutorial.name = "tutorial override 00_00".to_owned();
    tutorial.scope = Some(NativeWorldScope::Tutorial);
    catalog.scenes.push(tutorial);
    (catalog.world_by_dong_key, catalog.tutorial_by_dong_key) =
        build_catalog_indices(&catalog.scenes).unwrap();
    catalog.presentation_footprints =
        NativeWorldCatalog::presentation_footprints(&catalog.scenes);

    let selected = catalog
        .scene_for_tile(NativeWorldScope::Tutorial, [0, 0])
        .unwrap();
    assert_eq!(selected.scope, Some(NativeWorldScope::Tutorial));
    assert!(catalog.scene_is_selected_for_scope(NativeWorldScope::Tutorial, selected));
    let world = catalog
        .scene_for_tile(NativeWorldScope::WorldMap, [0, 0])
        .unwrap();
    assert!(!catalog.scene_is_selected_for_scope(NativeWorldScope::Tutorial, world));

    let neighbor = catalog
        .scene_for_tile(NativeWorldScope::Tutorial, [1, 0])
        .unwrap();
    assert_eq!(neighbor.scope, Some(NativeWorldScope::WorldMap));
    assert!(catalog.scene_is_selected_for_scope(NativeWorldScope::Tutorial, neighbor));
}

#[test]
fn retrobution_spawn_selects_map_12_03_by_native_position() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    assert_eq!(catalog.scenes().len(), 170);
    assert_eq!(
        catalog.blocked_count(),
        0,
        "the runtime registry contains playable scenes only; conversion blockers are not shipped"
    );
    assert_eq!(catalog.registered_ambience_cells(false), 170);
    assert_eq!(catalog.registered_ambience_cells(true), 0);
    assert_eq!(
        catalog.cached_terrain_count(),
        0,
        "catalog metadata loading must not decode any Gray16/PNG terrain payload"
    );
    let spawn = ProtocolPosition::new([632_032, 187_177, -5_500]).to_native();
    assert!(spawn.abs_diff_eq(Vec3::new(-6320.32, -55.0, 1871.77), 0.000_2));

    let selected = select_native_world_scene(&catalog, spawn).unwrap();
    assert_eq!(selected.tile, [12, 3]);
    assert_eq!(
        selected.root.translation,
        [0.0, 0.0, 0.0],
        "the scene root stays identity because the exact serialized map root is carried by rootChain"
    );
    assert!(
        selected
            .terrain_origin()
            .unwrap()
            .abs_diff_eq(Vec3::new(-6144.0, -300.0, 1536.0), f32::EPSILON),
        "map root translation and terrain owner Y must each be composed exactly once"
    );
    let terrain = selected.native_terrain.as_ref().unwrap();
    assert_eq!(terrain.path, "map/tiles/map_12_03/terrain/terrain.json");
    assert_eq!(terrain.terrain_collider_path_id, 13_865);
    assert_eq!(terrain.terrain_game_object_path_id, 13_863);
    assert_eq!(terrain.terrain_transform_path_id, 2_890);
    assert_eq!(terrain.transform.translation, [0.0, -300.0, 0.0]);
    assert!(terrain.loaded.is_none());
    let loaded = catalog.load_terrain(terrain).unwrap();
    assert_eq!(loaded.geometry().vertex_count(), 129 * 129);
    assert_eq!(loaded.geometry().index_count(), 128 * 128 * 6);
    assert_eq!(catalog.cached_terrain_count(), 1);

    let map_12_03_center = Vec3::new(-6400.0, 0.0, 1792.0);
    let ambience = catalog.sample_ambience(map_12_03_center, false);
    assert!((ambience.fog_depth - 0.44).abs() < 0.000_001);
    assert_eq!(
        ambience.fog_color,
        [0.019607844, 0.5803922, 0.3019608, 1.0,]
    );
    let applied = catalog.applied_ambience(map_12_03_center, false);
    assert!(applied.fog_enabled);
    assert!((applied.fog_density - 0.0022).abs() < 0.000_001);

    let map_08_06 = catalog
        .scenes()
        .iter()
        .find(|scene| scene.tile == [8, 6])
        .unwrap();
    assert!(!map_08_06.contains_native_xz(spawn));
}

#[test]
fn sector_v_uses_the_exact_retrobution_distance_fog_sample() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let sector_v_center = Vec3::new(-3_840.0, 0.0, 4_352.0);
    let sample = catalog.sample_ambience(sector_v_center, false);
    assert!((sample.fog_depth - 0.789_999_96).abs() < 0.000_001);
    assert_eq!(sample.fog_color, [0.176_470_6, 0.588_235_3, 1.0, 1.0]);
    let applied = catalog.applied_ambience(sector_v_center, false);
    assert!(applied.fog_enabled);
    assert!((applied.fog_density - 0.003_95).abs() < 0.000_001);
}

#[test]
fn frozen_v3_root_chain_is_composed_exactly_once() {
    let path = asset_root()
        .join("../../work/native-terrain-contract-smoke-v3/maps/map_02_05/terrain/scene-instance.json");
    let bytes = fs::read(path).unwrap();
    let scene: NativeTerrainSceneInstance = serde_json::from_slice(&bytes).unwrap();
    validate_root_chain(
        &scene.root_chain,
        scene.linkage.transform.parent_transform_path_id,
        scene.map_tile_root_transform_path_id,
    )
    .unwrap();
    let mut matrix = Mat4::IDENTITY;
    for node in scene.root_chain.nodes.iter().rev() {
        matrix *= node
            .native_local_transform
            .try_to_bevy("frozen root node")
            .unwrap()
            .to_matrix();
    }
    matrix *= scene
        .local_transform
        .try_to_bevy("frozen owner")
        .unwrap()
        .to_matrix();
    let origin = matrix.transform_point3(Vec3::ZERO);
    assert!(origin.abs_diff_eq(Vec3::new(-1024.0, -300.0, 2560.0), f32::EPSILON));

    let duplicated_root = scene.root_chain.nodes[0]
        .native_local_transform
        .try_to_bevy("duplicated root")
        .unwrap()
        .to_matrix()
        * matrix;
    assert_ne!(
        duplicated_root.transform_point3(Vec3::ZERO),
        origin,
        "rootChain must never be precomposed and spawned a second time"
    );
}

#[test]
fn repeated_npc_move_packets_preserve_ground_y_while_transport_keeps_authoritative_y() {
    let floor = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/network-npc-floor.glb".to_owned(),
        vertices: vec![
            Vec3::new(-4.0, 0.0, -4.0),
            Vec3::new(4.0, 0.0, -4.0),
            Vec3::new(-4.0, 0.0, 4.0),
            Vec3::new(4.0, 0.0, 4.0),
        ]
        .into(),
        indices: vec![0, 2, 1, 2, 3, 1].into(),
        local_min: Vec3::new(-4.0, 0.0, -4.0),
        local_max: Vec3::new(4.0, 0.0, 4.0),
        is_trigger: false,
    };
    let self_floor = floor.clone();
    let floor_global = GlobalTransform::IDENTITY;
    let floor_bounds =
        AuthoredColliderWorldBounds::from_collider(&floor, &floor_global).unwrap();

    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(Update, advance_network_npc_motion_0104)
        .add_systems(Update, advance_network_transportation_motion_0104)
        .add_systems(Update, sync_authored_collider_spatial_index)
        .add_systems(
            Update,
            resolve_authored_world_ground
                .after(advance_network_npc_motion_0104)
                .after(advance_network_transportation_motion_0104)
                .after(sync_authored_collider_spatial_index),
        );
    app.world_mut().spawn((floor_global, floor, floor_bounds));
    let scene_root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "network-npc-tile".to_owned(),
                tile: [0, 0],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldPresentationStatus::Loading,
        ))
        .id();
    let npc = app
        .world_mut()
        .spawn((
            NetworkNpc0104 {
                npc_id: 7,
                npc_type: 728,
            },
            NetworkNpcAppearance0104(NpcAppearance0104 {
                npc_id: 7,
                npc_type: 728,
                hp: 500,
                condition_bit_flag: 0,
                position: [0, 0, 100],
                angle: 0,
                barker_type: 0,
            }),
            NetworkNpcGrounding0104 {
                probe_height: 2.0,
                vertical_velocity: 0.0,
            },
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();
    let self_floor_global = GlobalTransform::from(Transform::from_xyz(0.0, 0.5, 0.0));
    let self_floor_bounds =
        AuthoredColliderWorldBounds::from_collider(&self_floor, &self_floor_global).unwrap();
    app.world_mut().spawn((
        ChildOf(npc),
        self_floor_global,
        self_floor,
        self_floor_bounds,
    ));

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.1));
    app.update();
    assert_eq!(
        app.world().get::<Transform>(npc).unwrap().translation.y,
        1.0,
        "an NPC must retain authoritative Y until every collider in its tile is ready"
    );
    *app.world_mut()
        .get_mut::<NativeWorldPresentationStatus>(scene_root)
        .unwrap() = NativeWorldPresentationStatus::Ready;

    for _ in 0..4 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(0.1));
        app.update();
    }
    assert!(
        app.world()
            .get::<Transform>(npc)
            .unwrap()
            .translation
            .abs_diff_eq(Vec3::ZERO, 0.000_01),
        "NPC did not settle onto the authored world floor"
    );

    let transportation = app
        .world_mut()
        .spawn((
            NetworkTransportation0104 {
                transportation_kind: 1,
                id: 9,
                transportation_type: 2,
            },
            NetworkTransportationMotion0104 {
                destination: Vec3::new(1.0, 4.0, 0.0),
                speed: 100.0,
                move_style: 1,
            },
            Transform::IDENTITY,
        ))
        .id();
    // OpenFusion combat movement publishes another NPC_MOVE every 400 ms.
    // Each packet may carry the player's server Z, but primary
    // NpcMoveController discards that vertical target for locomotion.
    for packet_index in 1..=3 {
        let expected_x = packet_index as f32;
        app.world_mut()
            .entity_mut(npc)
            .insert(NetworkNpcMotion0104 {
                destination: Vec3::new(expected_x, 1.0, 0.0),
                speed: 100.0,
                move_style: 1,
            });
        for tick in 0..4 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(0.1));
            app.update();
            assert!(
                app.world()
                    .get::<Transform>(npc)
                    .unwrap()
                    .translation
                    .abs_diff_eq(Vec3::new(expected_x, 0.0, 0.0), 0.000_01),
                "NPC_MOVE packet {packet_index} lifted the grounded NPC on cadence tick {tick}"
            );
        }
    }
    assert_eq!(
        app.world()
            .get::<Transform>(transportation)
            .unwrap()
            .translation,
        Vec3::new(1.0, 4.0, 0.0),
        "transportation retains its authoritative three-dimensional path"
    );
}

#[test]
fn capsule_bottom_accepts_only_walkable_platform_edge_support() {
    let platform = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/capsule-edge-support.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(2.0, 1.5, 0.0),
            Vec3::new(0.0, 1.5, 2.0),
            Vec3::new(2.0, 1.5, 2.0),
        ]
        .into(),
        indices: vec![0, 2, 1, 2, 3, 1].into(),
        local_min: Vec3::new(0.0, 1.5, 0.0),
        local_max: Vec3::new(2.0, 1.5, 2.0),
        is_trigger: false,
    };
    let global = GlobalTransform::IDENTITY;
    let bounds = AuthoredColliderWorldBounds::from_collider(&platform, &global).unwrap();

    let direct = collider_capsule_ground_contact_with_bounds(
        &platform,
        Mat4::IDENTITY,
        &bounds,
        1.0,
        1.0,
        1.5,
        1.3,
        1.6,
    )
    .expect("the platform center must retain the exact face-height contact");
    assert!((direct.0 - 1.5).abs() <= f32::EPSILON);
    assert!(direct.1.abs_diff_eq(Vec3::Y, f32::EPSILON));

    let edge = collider_capsule_ground_contact_with_bounds(
        &platform,
        Mat4::IDENTITY,
        &bounds,
        -0.15,
        1.0,
        1.5,
        1.3,
        1.6,
    )
    .expect("the lower capsule sphere must retain a walkable edge contact");
    assert!((1.4..1.5).contains(&edge.0));
    assert!(edge.1.y >= AUTHORED_WALKABLE_MIN_UP_DOT);

    assert!(
        collider_capsule_ground_contact_with_bounds(
            &platform,
            Mat4::IDENTITY,
            &bounds,
            -0.22,
            1.0,
            1.5,
            1.3,
            1.6,
        )
        .is_none(),
        "an edge normal steeper than slopeLimit must not become magnetic support"
    );
}

#[test]
fn step_sized_steep_dimple_keeps_the_capsule_radial_support() {
    let dimple = step_sized_dimple_collider(0.2, 0.1);
    let global = GlobalTransform::IDENTITY;
    let bounds = AuthoredColliderWorldBounds::from_collider(&dimple, &global).unwrap();
    let support = collider_capsule_ground_contact_with_bounds(
        &dimple,
        Mat4::IDENTITY,
        &bounds,
        0.0,
        0.0,
        0.0,
        -0.1,
        0.1,
    )
    .expect("a steep feature shorter than stepOffset must retain its upward capsule edge");
    assert!((support.0 + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH).abs() < 0.000_01);
    assert!(support.1.abs_diff_eq(Vec3::Y, 0.000_01));

    let tall_dimple = step_sized_dimple_collider(0.6, 0.3);
    let tall_bounds =
        AuthoredColliderWorldBounds::from_collider(&tall_dimple, &global).unwrap();
    assert!(
        collider_capsule_ground_contact_with_bounds(
            &tall_dimple,
            Mat4::IDENTITY,
            &tall_bounds,
            0.0,
            0.0,
            0.0,
            -0.1,
            0.1,
        )
        .is_none(),
        "a tall steep trough must remain non-walkable instead of becoming magnetic support"
    );
}

#[test]
fn forced_surface_slide_can_latch_and_jump_from_a_step_sized_dimple() {
    let dimple = step_sized_dimple_collider(0.2, 0.1);
    let global = GlobalTransform::IDENTITY;
    let bounds = AuthoredColliderWorldBounds::from_collider(&dimple, &global).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState::default())
        .init_resource::<crate::movement::MovementIntentQueue>()
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                crate::movement::simulate_legacy_players,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    app.world_mut().spawn((global, dimple, bounds));
    app.update();

    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller.set_external_collision_result(
        crate::movement::LEGACY_COLLISION_SIDES,
        Some(Vec3::new(2.0, 1.0, 0.0).normalize()),
    );
    let player = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(
                0.0,
                -AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH,
                0.0,
            )),
            controller,
        ))
        .id();

    let mut saw_forced_slide = false;
    let mut latched = None;
    for frame in 0..12 {
        app.update();
        let position = app.world().get::<Transform>(player).unwrap().translation;
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        saw_forced_slide |= controller.surface_sliding();
        if controller.grounded && !controller.surface_sliding() {
            latched = Some((frame, position));
            break;
        }
    }
    assert!(
        saw_forced_slide,
        "the fixture never entered the source slide state"
    );
    let (latch_frame, latched_position) =
        latched.expect("the small dimple must replace the steep callback and latch Below");
    assert!(
        latch_frame <= 3,
        "the dimple latched too late on frame {latch_frame}"
    );
    assert!(
        latched_position.x.abs() < AUTHORED_CHARACTER_CONTROLLER_RADIUS,
        "forced sliding escaped the dimple before it could become support: {latched_position:?}"
    );

    app.world_mut()
        .resource_mut::<crate::movement::LegacyInputState>()
        .jump_just_pressed = true;
    app.update();
    let jumped_position = app.world().get::<Transform>(player).unwrap().translation;
    let jumped = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!(
        jumped.jumping,
        "jump input was not accepted from the latched dimple"
    );
    assert!(
        jumped_position.y > latched_position.y + 0.01,
        "the capsule latched but could not push off: before={latched_position:?}, after={jumped_position:?}"
    );
}

#[test]
fn leaving_a_bench_starts_source_gravity_instead_of_snapping_to_the_floor() {
    fn horizontal_surface(
        source: &str,
        minimum: Vec2,
        maximum: Vec2,
        height: f32,
    ) -> (GlobalTransform, AuthoredTriMeshCollider) {
        (
            GlobalTransform::IDENTITY,
            AuthoredTriMeshCollider {
                source_mesh: Handle::default(),
                source_model_path: source.to_owned(),
                vertices: vec![
                    Vec3::new(minimum.x, height, minimum.y),
                    Vec3::new(maximum.x, height, minimum.y),
                    Vec3::new(minimum.x, height, maximum.y),
                    Vec3::new(maximum.x, height, maximum.y),
                ]
                .into(),
                indices: vec![0, 2, 1, 2, 3, 1].into(),
                local_min: Vec3::new(minimum.x, height, minimum.y),
                local_max: Vec3::new(maximum.x, height, maximum.y),
                is_trigger: false,
            },
        )
    }

    let (floor_global, floor) = horizontal_surface(
        "models/bench-departure-floor.glb",
        Vec2::splat(-4.0),
        Vec2::splat(4.0),
        0.0,
    );
    let floor_bounds =
        AuthoredColliderWorldBounds::from_collider(&floor, &floor_global).unwrap();
    let (bench_global, bench) = horizontal_surface(
        "models/bench-seat.glb",
        Vec2::new(-1.0, -1.0),
        Vec2::new(0.0, 1.0),
        0.6,
    );
    let bench_bounds =
        AuthoredColliderWorldBounds::from_collider(&bench, &bench_global).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.1),
        ))
        .insert_resource(crate::movement::LegacyInputState::default())
        .init_resource::<crate::movement::MovementIntentQueue>()
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                crate::movement::simulate_legacy_players,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    app.world_mut().spawn((floor_global, floor, floor_bounds));
    app.world_mut().spawn((bench_global, bench, bench_bounds));

    // Establish Bevy's clock before the avatar is placed just beyond the
    // seat's capsule-radius support. The source controller was grounded on
    // the preceding frame, exactly as it is immediately after walking off.
    app.update();
    let player = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.4, 0.6, 0.0),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();

    app.update();
    let first_position = app.world().get::<Transform>(player).unwrap().translation;
    let first_controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!(
        (first_position.y - (0.6 - crate::movement::LEGACY_CHARACTER_MOVE_BIAS)).abs()
            <= AUTHORED_GROUND_SWEEP_TOLERANCE,
        "leaving the 0.6 m seat snapped directly to the lower floor: {first_position:?}"
    );
    assert!(!first_controller.grounded);
    assert_eq!(first_controller.velocity.y, 0.0);

    app.update();
    let falling_position = app.world().get::<Transform>(player).unwrap().translation;
    let falling_controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!((falling_controller.velocity.y + 1.0).abs() <= f32::EPSILON);
    assert!(
        (falling_position.y
            - (first_position.y - 0.1 - crate::movement::LEGACY_CHARACTER_MOVE_BIAS))
            .abs()
            <= AUTHORED_GROUND_SWEEP_TOLERANCE,
        "the first unsupported frame did not integrate Retrobution gravity: \
         first={first_position:?}, falling={falling_position:?}"
    );
    assert!(!falling_controller.grounded);
}

#[test]
fn ordinary_wall_surface_sliding_carries_the_baseline_jump_onto_a_tall_ledge() {
    let mut floor = closed_vehicle_test_collider();
    floor.source_model_path = "models/jump-regression-floor.glb".to_owned();
    floor.vertices = vec![
        Vec3::new(-8.0, 0.0, -4.0),
        Vec3::new(8.0, 0.0, -4.0),
        Vec3::new(-8.0, 0.0, 4.0),
        Vec3::new(8.0, 0.0, 4.0),
    ]
    .into();
    floor.indices = vec![0, 2, 1, 2, 3, 1].into();
    floor.local_min = Vec3::new(-8.0, 0.0, -4.0);
    floor.local_max = Vec3::new(8.0, 0.0, 4.0);
    let floor_global = GlobalTransform::IDENTITY;
    let floor_bounds =
        AuthoredColliderWorldBounds::from_collider(&floor, &floor_global).unwrap();

    let mut ledge = closed_vehicle_test_collider();
    ledge.source_model_path = "models/jump-regression-ledge.glb".to_owned();
    // The source box spans local X -1..1 and Y 0..2. This produces a
    // broad x=0..8 landing surface whose authored top is exactly y=1.5.
    let ledge_global = GlobalTransform::from(
        Transform::from_xyz(4.0, 0.0, 0.0).with_scale(Vec3::new(4.0, 0.75, 1.0)),
    );
    let ledge_bounds =
        AuthoredColliderWorldBounds::from_collider(&ledge, &ledge_global).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState {
            // A zero-degree reflected gameplay root maps legacy left to
            // native +X; drive toward the ledge centered at x=4.
            local_axis: Vec2::NEG_X,
            jump_just_pressed: true,
            ..default()
        })
        .init_resource::<crate::movement::MovementIntentQueue>()
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                crate::movement::simulate_legacy_players,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    app.world_mut().spawn((floor_global, floor, floor_bounds));
    app.world_mut().spawn((ledge_global, ledge, ledge_bounds));
    app.update();

    let player = app
        .world_mut()
        .spawn((
            Transform::from_xyz(-1.5, 0.0, 0.0),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();
    let mut maximum_y = 0.0_f32;
    let mut landed_on_ledge = false;
    let mut observed_surface_sliding = false;
    for frame in 0..90 {
        app.update();
        if frame == 0 {
            app.world_mut()
                .resource_mut::<crate::movement::LegacyInputState>()
                .jump_just_pressed = false;
        }
        let position = app.world().get::<Transform>(player).unwrap().translation;
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        let grounded = controller.grounded;
        let jumping = controller.jumping;
        let surface_sliding = controller.surface_sliding();
        maximum_y = maximum_y.max(position.y);
        observed_surface_sliding |= surface_sliding;

        let expected_packet_position = ProtocolPosition::from_native(position).raw();
        while let Some(packet) = app
            .world_mut()
            .resource_mut::<crate::movement::MovementIntentQueue>()
            .pop_front()
        {
            let packet_position = match packet {
                crate::movement::MovementIntent::Move(request) => request.position,
                crate::movement::MovementIntent::Stop(request) => request.position,
                crate::movement::MovementIntent::Jump(request) => request.position,
            };
            assert_eq!(
                packet_position, expected_packet_position,
                "jump packet exposed a pre-sweep position inside the ledge on frame {frame}"
            );
        }

        if grounded && !jumping && (position.y - 1.5).abs() <= GROUND_EPSILON {
            landed_on_ledge = true;
            break;
        }
    }

    let final_position = app.world().get::<Transform>(player).unwrap().translation;
    assert!(
        observed_surface_sliding,
        "the ordinary steep contact never activated Retrobution HandleSurfaceSliding: \
         apex={maximum_y}, final={final_position:?}"
    );
    assert!(
        (1.81..=1.84).contains(&maximum_y),
        "the source surface-slide frame plus finite-lip capsule recovery left the accepted \
         range above the unobstructed 1.6606 m baseline apex: \
         apex={maximum_y}, final={final_position:?}"
    );
    assert!(
        landed_on_ledge,
        "a source-reachable 1.5 m ledge rejected the baseline jump: \
         apex={maximum_y}, final={final_position:?}"
    );
}
