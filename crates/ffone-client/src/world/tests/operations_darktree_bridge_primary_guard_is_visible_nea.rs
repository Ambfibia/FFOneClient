use super::*;

#[test]
fn future_genius_grove_preloads_exact_infected_neighbor_water() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let position = Vec3::new(-5_888.0, 0.0, 768.0);
    let waters = neighboring_water_scenes(&catalog, NativeWorldScope::WorldMap, position);
    let tiles = waters
        .iter()
        .map(|(scene, indices)| {
            assert_eq!(indices.len(), 1, "one infected surface per Future dong");
            let visual = &scene.visuals[indices[0]];
            let model = scene.model(&visual.model).expect("infected water model");
            assert_eq!(model.root_name, "ffPoison");
            assert!(is_canonical_native_world_water_model_path(
                &model.path,
                "ffpoison"
            ));
            scene.tile
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        tiles,
        HashSet::from([
            [10, 0],
            [10, 1],
            [10, 2],
            [11, 0],
            [11, 1],
            [11, 2],
            [12, 0],
            [12, 1],
            [12, 2],
        ])
    );
}

#[test]
fn genius_grove_preloads_only_neighbor_water_from_outside_full_scene_radius() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let position = Vec3::new(-3_328.0, 0.0, 3_328.0);
    let ordinary_tiles = catalog
        .legacy_stream_target_tiles(NativeWorldScope::WorldMap, position)
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(
        ordinary_tiles,
        HashSet::from([[5, 6], [6, 5], [6, 6], [7, 6]])
    );

    let mut water_visuals = 0_usize;
    let water_tiles = neighboring_water_scenes(&catalog, NativeWorldScope::WorldMap, position)
        .into_iter()
        .map(|(scene, indices)| {
            water_visuals += indices.len();
            scene.tile
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        water_tiles,
        HashSet::from([[5, 5], [5, 6], [5, 7], [6, 6], [7, 5], [7, 6], [7, 7]])
    );
    assert_eq!(water_visuals, 9, "all exact ffWater and ffPoison visuals");
    assert!(water_tiles.contains(&[5, 5]) && !ordinary_tiles.contains(&[5, 5]));
    assert!(water_tiles.contains(&[7, 5]) && !ordinary_tiles.contains(&[7, 5]));
    assert!(water_tiles.contains(&[7, 7]) && !ordinary_tiles.contains(&[7, 7]));
    assert_eq!(NATIVE_WORLD_MAX_RESIDENT_TILES, 9);
}

#[test]
fn neighbor_water_preload_spawns_only_water_and_yields_to_ready_full_tile() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<GltfMesh>()
        .init_resource::<NativeWorldStreamingStatus>()
        .insert_resource(catalog)
        .add_systems(Update, stream_neighboring_water_surfaces);
    let target = app
        .world_mut()
        .spawn(Transform::from_xyz(-3_328.0, 0.0, 3_328.0))
        .id();
    app.world_mut()
        .spawn((LegacyOrbitCamera::new(target), Transform::default()));

    app.update();

    let water_roots = app
        .world_mut()
        .query::<&NativeWorldNeighborWaterRoot>()
        .iter(app.world())
        .map(|root| root.tile)
        .collect::<HashSet<_>>();
    assert_eq!(
        water_roots,
        HashSet::from([[5, 5], [5, 6], [5, 7], [6, 6], [7, 5], [7, 6], [7, 7]])
    );
    let water_visuals = app
        .world_mut()
        .query_filtered::<Entity, With<NativeWorldNeighborWaterVisual>>()
        .iter(app.world())
        .count();
    assert_eq!(water_visuals, 9);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<NativeWorldSceneRoot>>()
            .iter(app.world())
            .count(),
        0,
        "water-only preload must not consume a full resident tile slot"
    );

    app.world_mut().spawn((
        NativeWorldSceneRoot {
            name: "map_05_05".to_owned(),
            tile: [5, 5],
            scope: NativeWorldScope::WorldMap,
            selection_scope: NativeWorldScope::WorldMap,
        },
        NativeWorldVisualPresentationStatus::Ready,
    ));
    app.update();

    let water_roots = app
        .world_mut()
        .query::<&NativeWorldNeighborWaterRoot>()
        .iter(app.world())
        .map(|root| root.tile)
        .collect::<HashSet<_>>();
    assert_eq!(water_roots.len(), 6);
    assert!(!water_roots.contains(&[5, 5]));
}

#[test]
fn legacy_stream_target_never_exceeds_the_documented_resident_budget() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    for z in (0..=8_192).step_by(64) {
        for unity_x in (0..=8_192).step_by(64) {
            let targets = catalog.legacy_stream_target_tiles(
                NativeWorldScope::WorldMap,
                Vec3::new(-(unity_x as f32), 0.0, z as f32),
            );
            assert!(
                targets.len() <= NATIVE_WORLD_MAX_RESIDENT_TILES,
                "position ({unity_x}, {z}) targets {} tiles",
                targets.len()
            );
        }
    }
}

#[test]
fn stream_cap_pressure_evicts_one_retained_non_target_and_then_progresses() {
    let catalog = load_native_world_scenes(asset_root()).unwrap();
    let position = Vec3::new(-5888.0, 0.0, 5824.0);
    let targets = catalog
        .legacy_stream_target_tiles(NativeWorldScope::WorldMap, position)
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(targets.len(), 7);

    let retained = catalog
        .scenes()
        .iter()
        .filter(|scene| scene.scope == Some(NativeWorldScope::WorldMap))
        .filter_map(|scene| {
            let distance = catalog.presentation_squared_distance(
                NativeWorldScope::WorldMap,
                position,
                scene,
            )?;
            (distance <= EXTENDED_DONG_UNLOAD_DISTANCE_NATIVE.powi(2))
                .then_some((scene.tile, distance))
        })
        .collect::<Vec<_>>();
    assert_eq!(retained.len(), 10, "target/retention transition union");

    // Nine physical roots retain all three hysteresis-only tiles while one
    // current target is missing. Admission is capped until exactly one
    // stale root begins its incremental unload.
    let missing_target = [11, 12];
    assert!(targets.contains(&missing_target));
    let loaded = retained
        .into_iter()
        .filter(|(tile, _)| *tile != missing_target)
        .collect::<Vec<_>>();
    assert_eq!(loaded.len(), NATIVE_WORLD_MAX_RESIDENT_TILES);
    assert!(
        !native_world_has_physical_admission_capacity(loaded.len()),
        "the missing target cannot be admitted before an unload"
    );
    let eviction = native_world_stream_eviction_tile(loaded, &targets)
        .expect("one retained non-target must make room");
    assert_eq!(eviction, [12, 12]);
    assert!(!targets.contains(&eviction));
    assert_ne!(eviction, missing_target);
    assert!(native_world_has_physical_admission_capacity(
        NATIVE_WORLD_MAX_RESIDENT_TILES - 1
    ));
}

#[test]
fn unloading_roots_still_consume_physical_admission_capacity() {
    assert!(native_world_has_physical_admission_capacity(0));
    assert!(native_world_has_physical_admission_capacity(
        NATIVE_WORLD_MAX_RESIDENT_TILES - 1
    ));
    assert!(!native_world_has_physical_admission_capacity(
        NATIVE_WORLD_MAX_RESIDENT_TILES
    ));
    assert!(!native_world_has_physical_admission_capacity(
        NATIVE_WORLD_MAX_RESIDENT_TILES + 1
    ));
}

#[test]
fn streamed_scene_unload_is_amortized_across_frames() {
    let mut app = App::new();
    app.add_systems(Update, unload_native_world_scenes_incrementally);
    let root = app
        .world_mut()
        .spawn((PendingNativeWorldSceneUnload::default(), Visibility::Hidden))
        .id();
    for _ in 0..(NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME * 2 + 6) {
        app.world_mut().spawn((ChildOf(root), Transform::default()));
    }

    app.update();
    assert!(app.world().get_entity(root).is_ok());
    while app.world().get_entity(root).is_ok() {
        let previous = game_entity_count(app.world());
        app.update();
        let removed = previous - game_entity_count(app.world());
        assert!(removed > 0);
        assert!(removed <= NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME as u32);
    }
}

pub(super) fn count_residency_visibility_changes(
    changed: Query<(), Changed<Visibility>>,
    mut changes: ResMut<ResidencyVisibilityChanges>,
) {
    changes.0 += changed.iter().count();
}

#[test]
fn darktree_bridge_primary_guard_is_visible_near_and_hidden_at_the_screenshot_position() {
    let mut app = App::new();
    app.init_resource::<NativeWorldObjectRangeCameraCache>()
        .add_systems(Update, update_native_world_object_residency);

    // The map_12_02 bridge renderers share this source transform. Their
    // ordinary adaptive contract reaches 340, while the clean Balanced
    // SmallStuff presentation boundary is exactly 130.
    let source_center = Vec3::new(-6491.805, -58.971, 1511.814);
    let tag = pack_native_world_object_range(source_center, EXTENDED_WORLD_CAMERA_FAR_NATIVE)
        .expect("the production bridge center must fit the packed world contract");
    let (decoded_center, adaptive_end) = unpack_native_world_object_range(tag);
    assert_eq!(adaptive_end, EXTENDED_WORLD_CAMERA_FAR_NATIVE);

    let near_position = decoded_center + Vec3::X * 129.0;
    let screenshot_position = Vec3::new(-6440.0, decoded_center.y, 1216.0);
    let screenshot_distance = screenshot_position.distance(decoded_center);
    assert!(screenshot_distance > PRIMARY_AUDITED_SMALLSTUFF_PRESENTATION_END_NATIVE);
    assert!(screenshot_distance < adaptive_end);

    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            LegacyOrbitCamera::new(Entity::PLACEHOLDER),
            Transform::from_translation(near_position),
        ))
        .id();
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "map_12_02-darktree-bridge".to_owned(),
                tile: [12, 2],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldVisualPresentationStatus::Ready,
        ))
        .id();
    let bridge = app
        .world_mut()
        .spawn((
            ChildOf(root),
            NativeWorldObjectRangeMember {
                scene_root: root,
                group: NativeWorldObjectRangeGroupKey::Visual(9439),
            },
            NativeWorldPrimaryFarPresentationGuard,
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
        .get_mut::<NativeWorldObjectRangeContract>(bridge)
        .unwrap()
        .meshes
        .push(bridge);

    app.update();
    assert_eq!(
        app.world().get::<Visibility>(bridge),
        Some(&Visibility::Inherited),
        "the guarded bridge remains an ordinary visible object inside 130 units"
    );
    assert!(
        app.world()
            .get::<NativeWorldObjectRangeContract>(bridge)
            .unwrap()
            .visible
    );

    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation = screenshot_position;
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(bridge),
        Some(&Visibility::Hidden),
        "the screenshot position is inside native 340 but outside primary SmallStuff 130"
    );
    assert!(
        !app.world()
            .get::<NativeWorldObjectRangeContract>(bridge)
            .unwrap()
            .visible
    );

    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation = near_position;
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(bridge),
        Some(&Visibility::Inherited),
        "returning near the bridge must restore it rather than suppressing its asset"
    );
}

#[test]
fn unchanged_residency_does_not_dirty_visibility_at_the_next_camera_step() {
    let mut app = App::new();
    app.init_resource::<NativeWorldObjectRangeCameraCache>()
        .init_resource::<ResidencyVisibilityChanges>()
        .add_systems(
            Update,
            (
                update_native_world_object_residency,
                count_residency_visibility_changes.after(update_native_world_object_residency),
            ),
        );
    let center = Vec3::new(-3500.0, -50.0, 4500.0);
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            LegacyOrbitCamera::new(Entity::PLACEHOLDER),
            Transform::from_translation(center),
        ))
        .id();
    let root = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "unchanged-residency".to_owned(),
                tile: [7, 8],
                scope: NativeWorldScope::WorldMap,
                selection_scope: NativeWorldScope::WorldMap,
            },
            NativeWorldVisualPresentationStatus::Ready,
        ))
        .id();
    let tag = pack_native_world_object_range(center, 1_000.0).unwrap();
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
                visible: true,
            },
            Mesh3d::default(),
            Visibility::Inherited,
        ))
        .id();
    app.world_mut()
        .get_mut::<NativeWorldObjectRangeContract>(visual)
        .unwrap()
        .meshes
        .push(visual);

    // The first run establishes both the camera residency cache and the
    // change-detection baseline for the probe system.
    app.update();
    app.world_mut()
        .resource_mut::<ResidencyVisibilityChanges>()
        .0 = 0;

    // Cross the exact residency sampling threshold while staying well
    // inside the same object's range. The logical and physical state are
    // unchanged, so no downstream visibility work should be scheduled.
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation
        .x += NATIVE_WORLD_RANGE_CAMERA_STEP;
    app.update();

    assert_eq!(
        app.world().resource::<ResidencyVisibilityChanges>().0,
        0,
        "an unchanged residency sample must not dirty Visibility"
    );
    assert_eq!(
        app.world().get::<Visibility>(visual),
        Some(&Visibility::Inherited)
    );
}

#[test]
fn streamed_scene_unload_counts_root_entities_against_the_global_budget() {
    let mut app = App::new();
    app.add_systems(Update, unload_native_world_scenes_incrementally);
    for _ in 0..(NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME + 8) {
        app.world_mut()
            .spawn(PendingNativeWorldSceneUnload::default());
    }

    app.update();
    assert!(game_entity_count(app.world()) >= 8);
    while game_entity_count(app.world()) != 0 {
        let previous = game_entity_count(app.world());
        app.update();
        let removed = previous - game_entity_count(app.world());
        assert!(removed > 0 && removed <= NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME as u32);
    }
}

#[test]
fn translating_closed_vehicle_pushes_a_stationary_capsule_outside_then_allows_escape() {
    let vehicle = closed_vehicle_test_collider();
    let previous = Mat4::from_translation(Vec3::new(-1.5, 0.0, 0.0));
    let current = Mat4::from_translation(Vec3::new(-0.5, 0.0, 0.0));
    let current_global = GlobalTransform::from(Transform::from_xyz(-0.5, 0.0, 0.0));
    let bounds = AuthoredColliderWorldBounds::from_collider(&vehicle, &current_global).unwrap();
    let motion = RuntimeAuthoredColliderMotion {
        previous_world_from_local: previous,
        current_world_from_local: current,
    };
    let stationary = Vec3::new(0.4, 0.0, 0.0);

    let pushed = resolve_runtime_authored_collider_motion(
        stationary, stationary, &vehicle, &bounds, motion,
    );
    let contact_x =
        0.5 + AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    assert!(
        pushed.x >= contact_x - AUTHORED_COLLISION_CONTACT_TOLERANCE,
        "the kinematic body crossed the stationary capsule instead of pushing it: {pushed:?}"
    );

    let escaped =
        resolve_authored_wall_motion(pushed, Vec3::new(0.25, 0.0, 0.0), &[(current, &vehicle)]);
    assert!(
        escaped.x > pushed.x + 0.2,
        "the outward-wound body trapped a capsule after the kinematic push: \
         pushed={pushed:?}, escaped={escaped:?}"
    );
}

#[test]
fn published_traffic_car_colliders_have_walkable_roofs_and_kinematic_escape() {
    for (
        label,
        visual_glb,
        collider_glb,
        expected_vertices,
        expected_indices,
        expected_roof_y,
        stationary_x,
    ) in [
        (
            "passenger",
            "characters/npcs/dt_etc_passengercar_a/DT_ETC_PASSENGERCAR_A.glb",
            "characters/npcs/dt_etc_passengercar_a/DT_ETC_PASSENGERCAR_A.collision.glb",
            226,
            1_200,
            1.931_329_8,
            0.9,
        ),
        (
            "pickup",
            "characters/npcs/dt_etc_pickupcar_a/DT_ETC_PICKUPCAR_A.glb",
            "characters/npcs/dt_etc_pickupcar_a/DT_ETC_PICKUPCAR_A.collision.glb",
            194,
            942,
            1.773_020_7,
            1.4,
        ),
    ] {
        let collider = published_gltf_collider_from_glb(collider_glb);
        assert_eq!(collider.vertex_count(), expected_vertices, "{label}");
        assert_eq!(collider.index_count(), expected_indices, "{label}");

        // Follow the actual retained KFM hierarchy down to `collision`,
        // then apply the exact XDT scale shared by NPC types 2917..=2926.
        let current = published_character_node_world_transform(visual_glb, "collision", 0.95);
        let roof = collider_ground_height(&collider, current, 0.0, 0.0, 0.0, 3.0)
            .unwrap_or_else(|| panic!("{label} roof winding is not walkable"));
        assert!(
            (roof - expected_roof_y).abs() <= 0.000_1,
            "{label} roof height changed: expected {expected_roof_y}, got {roof}"
        );

        let previous = Mat4::from_translation(Vec3::NEG_X * 0.6) * current;
        let (minimum, maximum) = collider_world_bounds(&collider, current).unwrap();
        let bounds = AuthoredColliderWorldBounds { minimum, maximum };
        let stationary = Vec3::new(stationary_x, 0.0, 0.0);
        let pushed = resolve_runtime_authored_collider_motion(
            stationary,
            stationary,
            &collider,
            &bounds,
            RuntimeAuthoredColliderMotion {
                previous_world_from_local: previous,
                current_world_from_local: current,
            },
        );
        assert!(
            pushed.x > stationary.x + 0.15,
            "{label} car crossed a stationary capsule instead of pushing it: \
             stationary={stationary:?}, pushed={pushed:?}"
        );

        let current_pose = [(current, &collider)];
        let nearby = collect_authored_motion_triangles(pushed, Vec3::ZERO, &current_pose);
        let residual = deepest_authored_capsule_penetration(pushed, pushed.y, &nearby, true);
        assert!(
            residual.is_none_or(|push| push.length() <= AUTHORED_COLLISION_CONTACT_TOLERANCE),
            "{label} car left the pushed capsule penetrating its real triangles: {residual:?}"
        );

        let escaped = resolve_authored_wall_motion(pushed, Vec3::X * 0.25, &current_pose);
        assert!(
            escaped.x > pushed.x + 0.2,
            "{label} collider trapped the capsule after its kinematic push: \
             pushed={pushed:?}, escaped={escaped:?}"
        );
    }
}

#[test]
fn moving_vehicle_no_contact_mapping_is_identity() {
    let vehicle = closed_vehicle_test_collider();
    let previous = Mat4::from_scale_rotation_translation(
        Vec3::splat(0.95),
        Quat::from_rotation_y(-0.35),
        Vec3::new(-0.3, 0.0, 0.1),
    );
    let current = Mat4::from_scale_rotation_translation(
        Vec3::splat(0.95),
        Quat::from_rotation_y(0.4),
        Vec3::new(0.6, 0.0, -0.2),
    );
    let current_global = GlobalTransform::from(
        Transform::from_xyz(0.6, 0.0, -0.2)
            .with_rotation(Quat::from_rotation_y(0.4))
            .with_scale(Vec3::splat(0.95)),
    );
    let bounds = AuthoredColliderWorldBounds::from_collider(&vehicle, &current_global).unwrap();
    // The padded broad phase overlaps, but the capsule itself is just
    // above the roof. This exercises the inverse/forward mapping path and
    // proves that no-contact motion does not carry nearby players.
    let feet = Vec3::new(0.0, 2.25, 0.0);
    let resolved = resolve_runtime_authored_collider_motion(
        feet,
        feet,
        &vehicle,
        &bounds,
        RuntimeAuthoredColliderMotion {
            previous_world_from_local: previous,
            current_world_from_local: current,
        },
    );
    assert!(
        resolved.abs_diff_eq(feet, 0.000_01),
        "no-contact mapping carried the player: {resolved:?}"
    );
}

#[test]
fn supported_moving_vehicle_is_carried_exactly_once() {
    let vehicle = closed_vehicle_test_collider();
    let previous = Mat4::IDENTITY;
    let current_transform = Transform::from_xyz(1.0, 0.0, 0.0);
    let current_global = GlobalTransform::from(current_transform);
    let bounds = AuthoredColliderWorldBounds::from_collider(&vehicle, &current_global).unwrap();

    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    let collider = app
        .world_mut()
        .spawn((
            current_global,
            vehicle,
            bounds,
            RuntimeAuthoredColliderMotion {
                previous_world_from_local: previous,
                current_world_from_local: current_transform.to_matrix(),
            },
        ))
        .id();
    let player = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 2.0, 0.0),
            LegacyPlayerController::from_baseline_table(),
            NativeWorldGroundSupport {
                collider,
                last_world_from_local: previous,
                surface_normal: Vec3::Y,
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));

    app.update();

    let feet = app.world().get::<Transform>(player).unwrap().translation;
    assert!(
        feet.abs_diff_eq(Vec3::new(1.0, 2.0, 0.0), 0.000_01),
        "support carry and relative sweep applied the vehicle delta twice: {feet:?}"
    );
}

#[test]
fn translating_support_does_not_accumulate_lateral_drift() {
    let origin = Vec3::new(1843.75, 317.125, -2671.5);
    let start = origin + Vec3::new(1.375, 2.0, -0.625);
    for direction in [Vec3::X, Vec3::Y, Vec3::Z] {
        let pose = |distance| {
            Mat4::from_scale_rotation_translation(
                Vec3::new(1.3, 0.7, 2.1),
                Quat::from_euler(EulerRot::YXZ, 0.731, 0.123, -0.217),
                origin + direction * distance,
            )
        };
        let mut previous = pose(0.0);
        let mut point = start;
        // Ten minutes at 60 Hz, repeatedly travelling out and back.
        for frame in 1..=36_000 {
            let phase = frame % 1200;
            let distance = phase.min(1200 - phase) as f32 * 0.03125;
            let current = pose(distance);
            point = carried_support_point(previous, current, point).unwrap();
            assert_eq!(point, start + direction * distance, "frame {frame}");
            previous = current;
        }
        assert_eq!(point, start);
    }
}

#[test]
fn moving_support_carries_the_same_platform_local_point() {
    let previous = Mat4::from_translation(Vec3::new(10.0, 2.0, -4.0));
    let current = Mat4::from_scale_rotation_translation(
        Vec3::ONE,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Vec3::new(12.0, 5.0, -4.0),
    );
    let point = previous.transform_point3(Vec3::new(2.0, 1.0, 0.0));
    let carried = carried_support_point(previous, current, point).unwrap();
    assert!(carried.abs_diff_eq(current.transform_point3(Vec3::new(2.0, 1.0, 0.0)), 1e-5));
}

#[test]
fn nanomachine_top_stops_a_falling_capsule() {
    let collider = published_gltf_collider_from_glb(
        "characters/npcs/nanomachine/Nanomachine.collision.glb",
    );
    let pose = published_character_node_world_transform(
        "characters/npcs/nanomachine/Nanomachine.glb",
        "collision",
        1.0,
    );
    let bounds =
        AuthoredColliderWorldBounds::from_collider(&collider, &GlobalTransform::from(pose))
            .unwrap();
    // Negative control: the former sidecar had these same vertices with
    // every triangle reversed. It must reproduce the fall through the lid.
    let mut inward = collider.clone();
    let mut indices = inward.indices.to_vec();
    for triangle in indices.chunks_exact_mut(3) {
        triangle.swap(1, 2);
    }
    inward.indices = indices.into();
    let missed_lid = resolve_authored_controller_motion_with_bounds(
        Vec3::new(0.0, 4.0, 0.0),
        Vec3::new(0.0, -4.0, 0.0),
        -4.0,
        &[(pose, &inward, &bounds)],
        false,
    );
    assert!(
        missed_lid.position.y < 2.0,
        "negative control must miss the lid"
    );
    for x in [-0.2, 0.0, 0.2] {
        for z in [-0.2, 0.0, 0.2] {
            let result = resolve_authored_controller_motion_with_bounds(
                Vec3::new(x, 4.0, z),
                Vec3::new(0.0, -4.0, 0.0),
                -4.0,
                &[(pose, &collider, &bounds)],
                false,
            );
            assert!(
                result.position.y > 2.0,
                "capsule entered the nanomachine from above at {x}, {z}: {:?}",
                result.position
            );
            assert_ne!(result.collision_flags & LEGACY_COLLISION_BELOW, 0);
        }
    }
    for angle in 0..32 {
        let radians = angle as f32 * std::f32::consts::TAU / 32.0;
        let direction = Vec3::new(radians.cos(), 0.0, radians.sin());
        let result = resolve_authored_controller_motion_with_bounds(
            direction * 3.0,
            -direction * 3.0,
            0.0,
            &[(pose, &collider, &bounds)],
            false,
        );
        assert!(
            result.position.dot(direction) > 0.4,
            "capsule entered the nanomachine from side {angle}: {:?}",
            result.position
        );
    }
}
