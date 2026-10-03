use super::*;

#[test]
fn adaptive_world_range_is_object_sized_and_quantized_consistently() {
    assert_eq!(LEGACY_WORLD_CAMERA_FAR_NATIVE, 300.0);
    assert_eq!(EXTENDED_WORLD_CAMERA_FAR_NATIVE, 340.0);
    assert_eq!(native_world_object_range(1.0), 75.0);
    assert_eq!(native_world_object_range(10.0), 340.0);
    assert_eq!(NATIVE_WORLD_OBJECT_FADE_BAND, 32.0);
    for shader in [
        include_str!("../../rendering/legacy_model_material/legacy_model_base.wgsl"),
        include_str!("../../rendering/legacy_model_material/legacy_model_outline.wgsl"),
        include_str!("../../rendering/legacy_model_material/legacy_water.wgsl"),
    ] {
        assert!(shader.contains("(end - 32.0)) / 32.0"));
        assert!(shader.contains("mesh_functions::get_tag(instance_index)"));
        assert!(!shader.contains("get_visibility_range_dither_level"));
    }
    let center = Vec3::new(-3656.7, -50.6, 4563.2);
    let tag = pack_native_world_object_range(center, 271.0).unwrap();
    let (decoded_center, decoded_end) = unpack_native_world_object_range(tag);
    assert!(center.distance(decoded_center) <= 15.1);
    assert!((decoded_end - 271.0).abs() <= 265.0 / 30.0);
    let sentinel = native_world_pipeline_sentinel_range();
    assert_eq!(sentinel.end_margin, 340.0..341.0);
    assert!(sentinel.use_aabb);
    assert!(!sentinel.is_abrupt());
    assert!(pack_native_world_object_range(Vec3::new(1.0, 0.0, 1.0), 340.0).is_none());
    assert!(
        pack_native_world_object_range(Vec3::new(-1000.0, -18001.0, 1000.0), 340.0).is_none()
    );
}

#[test]
fn prefab_renderers_share_a_group_but_direct_renderers_remain_singletons() {
    let first = "Plane26 [prefab:BuildPlayer-Map_07_08#12706:bundle#7467 visual]";
    let second = "blackhole [prefab:BuildPlayer-Map_07_08#12706:bundle#7479 visual]";
    assert_eq!(
        native_world_object_range_group_key(1, first),
        native_world_object_range_group_key(2, second)
    );
    assert_ne!(
        native_world_object_range_group_key(1, "tree [BuildPlayer-Map_07_08#1 visual]"),
        native_world_object_range_group_key(2, "tree [BuildPlayer-Map_07_08#1 visual]")
    );
}

#[test]
fn published_prefab_owner_groups_are_retained_as_complete_objects() {
    let tiles = asset_root().join("map/tiles");
    let mut prefab_visuals = 0_usize;
    let mut groups = HashMap::<(String, String), usize>::new();
    for entry in std::fs::read_dir(tiles).unwrap() {
        let entry = entry.unwrap();
        let scene_path = entry.path().join("scene.json");
        if !scene_path.is_file() {
            continue;
        }
        let scene: NativeWorldScene =
            serde_json::from_slice(&std::fs::read(scene_path).unwrap()).unwrap();
        for visual in &scene.visuals {
            let Some(owner) = native_world_prefab_owner(&visual.name) else {
                continue;
            };
            prefab_visuals += 1;
            *groups
                .entry((scene.name.clone(), owner.to_owned()))
                .or_default() += 1;
        }
    }
    let multipart = groups.values().filter(|&&members| members > 1).count();
    let fifteen_part = groups.values().filter(|&&members| members == 15).count();
    assert_eq!(prefab_visuals, 12_548);
    assert_eq!(groups.len(), 940);
    assert_eq!(multipart, 920);
    assert_eq!(fifteen_part, 799);
    assert_eq!(groups.values().copied().max(), Some(15));
}

#[test]
fn world_presentation_waits_for_object_complete_range_contract() {
    let mut app = App::new();
    init_world_presentation_assets(&mut app);
    app.add_systems(Update, reveal_native_world_scenes);
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "range-gated".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldBehaviourStatus::Ready,
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            Visibility::Hidden,
        ))
        .id();
    let visual = app
        .world_mut()
        .spawn((
            ChildOf(root),
            SpawnedNativeWorldVisual {
                model_path: "models/range-gated.glb".to_owned(),
                source_model_path: "models/range-gated.glb".to_owned(),
                scene: 0,
            },
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(0),
            },
            NativeWorldVisualSceneReady,
            Visibility::Hidden,
        ))
        .id();

    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Loading)
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden)
    );

    app.world_mut().entity_mut(visual).insert((
        NativeWorldObjectRangeReady,
        NativeWorldObjectRangeContract {
            tag: 1,
            meshes: vec![visual],
            visible: false,
        },
    ));
    app.update();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Hidden),
        "revealing the tile must preserve direct-mesh range culling"
    );
}

#[test]
fn grounded_object_edge_neither_ejects_the_capsule_nor_sends_its_unresolved_position() {
    let mut floor = closed_vehicle_test_collider();
    floor.source_model_path = "models/grounded-edge-floor.glb".to_owned();
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

    let mut obstacle = closed_vehicle_test_collider();
    obstacle.source_model_path = "models/grounded-edge-obstacle.glb".to_owned();
    let obstacle_global = GlobalTransform::IDENTITY;
    let obstacle_bounds =
        AuthoredColliderWorldBounds::from_collider(&obstacle, &obstacle_global).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState {
            // Zero-yaw legacy left maps to native +X, toward the box's
            // x=-1 face from the player's x=-2 start.
            local_axis: Vec2::NEG_X,
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
    app.world_mut()
        .spawn((obstacle_global, obstacle, obstacle_bounds));
    app.update();

    let player = app
        .world_mut()
        .spawn((
            Transform::from_xyz(-2.0, 0.0, 0.0),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();
    for frame in 0..30 {
        app.update();
        let transform = app.world().get::<Transform>(player).unwrap();
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        assert!(
            transform.translation.y.abs() <= GROUND_EPSILON,
            "grounded wall approach sank before recovery on frame {frame}: {:?}",
            transform.translation,
        );
        assert!(
            !controller.surface_sliding(),
            "support contact must overwrite the side callback instead of ejecting a grounded \
             capsule on frame {frame}"
        );
    }

    let final_position = app.world().get::<Transform>(player).unwrap().translation;
    let contact_x = -1.0
        - (AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH);
    assert!(
        (final_position.x - contact_x).abs() <= AUTHORED_COLLISION_CONTACT_TOLERANCE,
        "capsule did not settle on the box sweep plane: {final_position:?}"
    );
    let expected_packet_position = ProtocolPosition::from_native(final_position).raw();
    let packets = app
        .world_mut()
        .resource_mut::<crate::movement::MovementIntentQueue>()
        .take_all();
    assert!(!packets.is_empty());
    for packet in packets {
        let packet_position = match packet {
            crate::movement::MovementIntent::Move(request) => request.position,
            crate::movement::MovementIntent::Stop(request) => request.position,
            crate::movement::MovementIntent::Jump(request) => request.position,
        };
        assert_eq!(
            packet_position, expected_packet_position,
            "packet exposed the pre-sweep position inside the object"
        );
    }
}
