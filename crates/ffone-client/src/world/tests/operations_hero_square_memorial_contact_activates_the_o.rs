use super::*;

#[test]
fn source_reachable_ledge_does_not_depend_on_the_jump_contact_phase() {
    let mut floor = closed_vehicle_test_collider();
    floor.source_model_path = "models/jump-phase-floor.glb".to_owned();
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
    ledge.source_model_path = "models/jump-phase-ledge.glb".to_owned();
    let ledge_global = GlobalTransform::from(
        Transform::from_xyz(4.0, 0.0, 0.0).with_scale(Vec3::new(4.0, 0.9, 1.0)),
    );
    let ledge_bounds =
        AuthoredColliderWorldBounds::from_collider(&ledge, &ledge_global).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState {
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

    // These offsets change the wall-contact frame without changing the
    // jump, speed, ledge or input. Retrobution's rounded lower capsule
    // clears the same source-reachable lip for every phase.
    let starts = [
        (-1.35, 0.0),
        (-1.45, 0.0),
        (-1.55, 0.0),
        (-1.65, 0.0),
        (-1.75, 0.0),
        (-1.85, 0.0),
        (-1.35, 0.8),
        (-1.55, 0.8),
        (-1.75, 0.8),
        (-1.35, 0.95),
        (-1.55, 0.95),
        (-1.75, 0.95),
        // Contact after the unobstructed apex. PhysX still performs its
        // artificial stepOffset UP before SIDE, then restores it in DOWN;
        // treating descent as one diagonal sweep makes these valid late
        // contacts catch below the same lip.
        (-3.65, 0.0),
        (-3.75, 0.0),
        (-3.85, 0.0),
    ];
    let players = starts
        .into_iter()
        .map(|(x, z)| {
            app.world_mut()
                .spawn((
                    Transform::from_xyz(x, 0.0, z),
                    LegacyPlayerController::from_baseline_table(),
                ))
                .id()
        })
        .collect::<Vec<_>>();
    let mut landed = vec![false; players.len()];
    for frame in 0..90 {
        app.update();
        if frame == 0 {
            app.world_mut()
                .resource_mut::<crate::movement::LegacyInputState>()
                .jump_just_pressed = false;
        }
        for (index, &player) in players.iter().enumerate() {
            let position = app.world().get::<Transform>(player).unwrap().translation;
            let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
            landed[index] |= controller.grounded
                && !controller.jumping
                && (position.y - 1.8).abs() <= GROUND_EPSILON;
        }
    }

    for (((start_x, start_z), landed), player) in starts.into_iter().zip(landed).zip(players) {
        let final_position = app.world().get::<Transform>(player).unwrap().translation;
        assert!(
            landed,
            "source-reachable ledge depended on contact phase from \
             x={start_x}, z={start_z}: \
             final={final_position:?}"
        );
    }
}

#[test]
fn sector_v_treehouse_walkable_roof_holds_a_stationary_capsule() {
    const TREEHOUSE_COLLIDER: &str = "objects/unclassified/unnamed_static_object/models/unnamed_static_object_variant_0824/collision.glb";
    let treehouse_bytes =
        fs::read(join_relative(&asset_root(), TREEHOUSE_COLLIDER)).expect("treehouse collider");
    assert_eq!(treehouse_bytes.len(), 122_720);
    assert_eq!(
        format!("{:X}", Sha256::digest(&treehouse_bytes)),
        "B239850ABF9EDAEA07A5BD09CC69BDA044DD4AA6818918860BBC448AF89AF130",
    );
    let treehouse = authored_collider_from_glb(TREEHOUSE_COLLIDER);
    assert_eq!(treehouse.vertex_count(), 8_177);
    assert_eq!(treehouse.index_count(), 11_487);

    // Primary Map_12_03 owns this as GameObject #12442 / MeshCollider
    // #12444 / external mesh fileId 1 pathId 185. Triangle 306 is one of
    // the broad upper treehouse faces: its 41.24-degree normal is below
    // the serialized 50-degree CharacterController slopeLimit, while a
    // neighbouring decorative face is steeper than the limit.
    let treehouse_transform =
        Transform::from_translation(Vec3::new(-6_430.437_5, -59.631_423_95, 1_772.723_4))
            .with_rotation(Quat::from_xyzw(
                0.712_750_73,
                -0.209_203_84,
                -0.238_334_92,
                -0.625_632_94,
            ))
            .with_scale(Vec3::new(1.0, 1.000_000_1, 1.000_000_1));
    let world_from_treehouse = treehouse_transform.to_matrix();
    let triangle = treehouse
        .indices
        .chunks_exact(3)
        .nth(306)
        .expect("accepted treehouse mesh must retain roof triangle 306");
    let roof = AuthoredCollisionTriangle::new(
        world_from_treehouse.transform_point3(treehouse.vertices[triangle[0] as usize]),
        world_from_treehouse.transform_point3(treehouse.vertices[triangle[1] as usize]),
        world_from_treehouse.transform_point3(treehouse.vertices[triangle[2] as usize]),
    )
    .expect("roof triangle");
    let roof_angle = roof
        .normal
        .dot(Vec3::Y)
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();
    assert!((roof_angle - 41.235_45).abs() < 0.001, "{roof_angle}");
    let raw_face_point = (roof.a + roof.b + roof.c) / 3.0;
    let treehouse_global = GlobalTransform::from(treehouse_transform);
    let treehouse_bounds =
        AuthoredColliderWorldBounds::from_collider(&treehouse, &treehouse_global).unwrap();
    let (start_y, support_normal) = collider_capsule_ground_contact_with_bounds(
        &treehouse,
        world_from_treehouse,
        &treehouse_bounds,
        raw_face_point.x,
        raw_face_point.z,
        raw_face_point.y,
        raw_face_point.y - GROUND_EPSILON,
        raw_face_point.y + GROUNDED_STEP_UP,
    )
    .expect("walkable roof must supply lower-sphere support");
    assert!(
        start_y > raw_face_point.y + 0.05,
        "a sloped capsule face must include its radius/skin support offset"
    );
    assert!(support_normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT);
    let start = Vec3::new(raw_face_point.x, start_y, raw_face_point.z);

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
    app.world_mut()
        .spawn((treehouse_global, treehouse, treehouse_bounds));
    app.update();
    let player = app
        .world_mut()
        .spawn((
            Transform::from_translation(start),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();

    for frame in 0..120 {
        app.update();
        let position = app.world().get::<Transform>(player).unwrap().translation;
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        assert!(
            controller.grounded,
            "walkable treehouse roof lost Below on frame {frame}: start={start:?}, current={position:?}"
        );
        assert!(
            !controller.surface_sliding(),
            "walkable treehouse roof activated surface sliding on frame {frame}: start={start:?}, current={position:?}"
        );
        assert!(
            Vec2::new(position.x - start.x, position.z - start.z).length()
                <= AUTHORED_COLLISION_CONTACT_TOLERANCE,
            "stationary capsule drifted across the walkable treehouse roof on frame {frame}: start={start:?}, current={position:?}"
        );
    }
}

#[test]
fn sector_v_treehouse_retains_its_step_sized_concave_edge_supports() {
    const TREEHOUSE_COLLIDER: &str = "objects/unclassified/unnamed_static_object/models/unnamed_static_object_variant_0824/collision.glb";
    let treehouse = authored_collider_from_glb(TREEHOUSE_COLLIDER);
    let transform =
        Transform::from_translation(Vec3::new(-6_430.437_5, -59.631_423_95, 1_772.723_4))
            .with_rotation(Quat::from_xyzw(
                0.712_750_73,
                -0.209_203_84,
                -0.238_334_92,
                -0.625_632_94,
            ))
            .with_scale(Vec3::new(1.0, 1.000_000_1, 1.000_000_1));
    let matrix = transform.to_matrix();
    let global = GlobalTransform::from(transform);
    let bounds = AuthoredColliderWorldBounds::from_collider(&treehouse, &global).unwrap();
    type EdgeKey = ([i32; 3], [i32; 3]);
    let quantize = |point: Vec3| {
        [
            (point.x * 10_000.0).round() as i32,
            (point.y * 10_000.0).round() as i32,
            (point.z * 10_000.0).round() as i32,
        ]
    };
    let mut edges = std::collections::HashMap::<EdgeKey, Vec<(usize, Vec3, Vec3)>>::new();
    for (triangle_index, triangle) in treehouse.indices.chunks_exact(3).enumerate() {
        let points = [
            treehouse.vertices[triangle[0] as usize],
            treehouse.vertices[triangle[1] as usize],
            treehouse.vertices[triangle[2] as usize],
        ];
        for (start, end) in [(0, 1), (1, 2), (2, 0)] {
            let left = quantize(points[start]);
            let right = quantize(points[end]);
            let key = if left <= right {
                (left, right)
            } else {
                (right, left)
            };
            edges
                .entry(key)
                .or_default()
                .push((triangle_index, points[start], points[end]));
        }
    }
    let mut candidates = Vec::new();
    for owners in edges.values().filter(|owners| owners.len() == 2) {
        let (left_index, local_a, local_b) = owners[0];
        let (right_index, _, _) = owners[1];
        let triangle = |index: usize| {
            let indices = treehouse.indices.chunks_exact(3).nth(index).unwrap();
            let a = matrix.transform_point3(treehouse.vertices[indices[0] as usize]);
            let b = matrix.transform_point3(treehouse.vertices[indices[1] as usize]);
            let c = matrix.transform_point3(treehouse.vertices[indices[2] as usize]);
            (a, b, c, (b - a).cross(c - a).normalize_or_zero())
        };
        let (left_a, left_b, left_c, left_normal) = triangle(left_index);
        let (right_a, right_b, right_c, right_normal) = triangle(right_index);
        if !(left_normal.y > 0.0
            && right_normal.y > 0.0
            && left_normal.y < AUTHORED_WALKABLE_MIN_UP_DOT
            && right_normal.y < AUTHORED_WALKABLE_MIN_UP_DOT)
        {
            continue;
        }
        let left_horizontal = Vec2::new(left_normal.x, left_normal.z).normalize_or_zero();
        let right_horizontal = Vec2::new(right_normal.x, right_normal.z).normalize_or_zero();
        if left_horizontal.dot(right_horizontal) > -0.25 {
            continue;
        }
        let midpoint = matrix.transform_point3((local_a + local_b) * 0.5);
        let maximum_y = left_a
            .y
            .max(left_b.y)
            .max(left_c.y)
            .max(right_a.y)
            .max(right_b.y)
            .max(right_c.y);
        let rise = maximum_y - midpoint.y;
        if rise > GROUNDED_STEP_UP + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH {
            continue;
        }
        let support = collider_capsule_ground_contact_with_bounds(
            &treehouse,
            matrix,
            &bounds,
            midpoint.x,
            midpoint.z,
            midpoint.y - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH,
            midpoint.y - 0.2,
            midpoint.y + 0.2,
        );
        candidates.push((
            midpoint.y,
            midpoint,
            left_index,
            right_index,
            rise,
            left_normal,
            right_normal,
            support,
        ));
    }
    candidates.sort_by(|left, right| right.0.total_cmp(&left.0));
    assert_eq!(candidates.len(), 6);
    assert!(candidates.iter().all(|candidate| {
        candidate
            .7
            .is_some_and(|(_, normal)| normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT)
    }));
}

#[test]
fn sector_v_treehouse_concave_entries_settle_without_freefall() {
    const TREEHOUSE_COLLIDER: &str = "objects/unclassified/unnamed_static_object/models/unnamed_static_object_variant_0824/collision.glb";
    let treehouse = authored_collider_from_glb(TREEHOUSE_COLLIDER);
    let transform =
        Transform::from_translation(Vec3::new(-6_430.437_5, -59.631_423_95, 1_772.723_4))
            .with_rotation(Quat::from_xyzw(
                0.712_750_73,
                -0.209_203_84,
                -0.238_334_92,
                -0.625_632_94,
            ))
            .with_scale(Vec3::new(1.0, 1.000_000_1, 1.000_000_1));
    let global = GlobalTransform::from(transform);
    let bounds = AuthoredColliderWorldBounds::from_collider(&treehouse, &global).unwrap();
    let matrix = global.to_matrix();
    let radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let mut starts = Vec::new();
    for (candidate_index, (edge_xz, triangle_index)) in [
        (Vec2::new(-6_452.375_5, 1_737.564_8), 2_241_usize),
        (Vec2::new(-6_428.552_2, 1_736.656_5), 460_usize),
        (Vec2::new(-6_414.331_5, 1_758.251_3), 248_usize),
    ]
    .into_iter()
    .enumerate()
    {
        let triangle = treehouse
            .indices
            .chunks_exact(3)
            .nth(triangle_index)
            .unwrap();
        let a = matrix.transform_point3(treehouse.vertices[triangle[0] as usize]);
        let b = matrix.transform_point3(treehouse.vertices[triangle[1] as usize]);
        let c = matrix.transform_point3(treehouse.vertices[triangle[2] as usize]);
        let normal = (b - a).cross(c - a).normalize();
        let horizontal = Vec3::new(normal.x, 0.0, normal.z).normalize_or_zero();
        for (offset_index, offset) in [0.05, 0.10, 0.20, 0.30, 0.40].into_iter().enumerate() {
            // Begin tangent to the steep face just uphill from the shared
            // edge, then let the original forced-slide velocity carry the
            // lower capsule sphere into the concavity.
            let x = edge_xz.x - horizontal.x * offset;
            let z = edge_xz.y - horizontal.z * offset;
            let expected_plane_y =
                a.y - (normal.x * (x - a.x) + normal.z * (z - a.z)) / normal.y;
            let Some((plane_y, normal)) = treehouse
                .indices
                .chunks_exact(3)
                .filter_map(|indices| {
                    let a = matrix.transform_point3(treehouse.vertices[indices[0] as usize]);
                    let b = matrix.transform_point3(treehouse.vertices[indices[1] as usize]);
                    let c = matrix.transform_point3(treehouse.vertices[indices[2] as usize]);
                    let normal = (b - a).cross(c - a).normalize_or_zero();
                    if normal.y <= 0.0 || normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT {
                        return None;
                    }
                    triangle_height_at_xz(a, b, c, x, z)
                        .map(|height| (height, normal, (height - expected_plane_y).abs()))
                })
                .min_by(|left, right| left.2.total_cmp(&right.2))
                .filter(|candidate| candidate.2 <= 0.5)
                .map(|(height, normal, _)| (height, normal))
            else {
                continue;
            };
            let y = plane_y
                + ((radius + AUTHORED_COLLISION_CONTACT_TOLERANCE) / normal.y
                    - AUTHORED_CHARACTER_CONTROLLER_RADIUS)
                    .max(0.0);
            starts.push((
                candidate_index,
                offset_index,
                offset,
                Vec3::new(x, y, z),
                normal,
            ));
        }
    }

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
    app.world_mut().spawn((global, treehouse, bounds));
    app.update();

    let mut players = Vec::new();
    for (candidate_index, offset_index, offset, start, normal) in starts {
        let mut controller = LegacyPlayerController::from_baseline_table();
        controller.set_grounded(false);
        controller.set_external_collision_result(
            crate::movement::LEGACY_COLLISION_SIDES,
            Some(normal),
        );
        let entity = app
            .world_mut()
            .spawn((Transform::from_translation(start), controller))
            .id();
        players.push((candidate_index, offset_index, offset, start, entity));
    }
    assert_eq!(players.len(), 4);
    for _ in 0..90 {
        app.update();
    }
    for (candidate_index, offset_index, offset, start, entity) in players {
        let position = app.world().get::<Transform>(entity).unwrap().translation;
        let controller = app.world().get::<LegacyPlayerController>(entity).unwrap();
        assert!(
            controller.grounded && !controller.jumping && !controller.surface_sliding(),
            "Sector V concavity did not settle candidate={candidate_index} sample={offset_index} offset={offset}: start={start:?}, final={position:?}"
        );
        assert!(
            position.y >= start.y - 0.5,
            "Sector V concavity allowed a freefall instead of catching candidate={candidate_index} sample={offset_index} offset={offset}: start={start:?}, final={position:?}"
        );
    }
}

#[test]
fn hero_square_memorial_contact_activates_the_original_surface_slide_step() {
    const MEMORIAL_COLLIDER: &str = "objects/unclassified/unnamed_static_object/models/unnamed_static_object_variant_0314/collision.glb";
    let memorial_bytes =
        fs::read(join_relative(&asset_root(), MEMORIAL_COLLIDER)).expect("memorial collider");
    assert_eq!(memorial_bytes.len(), 35_004);
    assert_eq!(
        format!("{:X}", Sha256::digest(&memorial_bytes)),
        "A26A7B3A0051D3A1F206C7C98902F87CC09AD515C646E23A9945EABCC0FF960C",
    );
    let memorial = authored_collider_from_glb(MEMORIAL_COLLIDER);
    assert_eq!(memorial.vertex_count(), 2_126);
    assert_eq!(memorial.index_count(), 3_936);

    // Primary `builds/retrobution-20260613/Map_12_03.unity3d` is 114,906
    // bytes, SHA-256 E974490A30E1A31DABE677423583D0127788FBAC48F7F84D251ABBA7A9A5A631.
    // It owns BuildPlayer-Map_12_03 GameObject #12210 / MeshCollider
    // #12212. This is the large central Hero Square memorial surrounded
    // by the five SB_SBKN_tower_hologram placements; it is layer 0/tag 0,
    // non-trigger and non-convex, so ResetCollisionVariables selects the
    // ordinary non-priority surface-sliding branch in the clean client.
    let memorial_transform = Transform::from_translation(Vec3::new(
        -6_287.617_187_5,
        -39.829_170_2,
        1_902.172_851_6,
    ))
    .with_rotation(Quat::from_xyzw(
        -0.323_683_9,
        0.628_672_2,
        0.628_672_24,
        0.323_683_86,
    ))
    .with_scale(Vec3::new(0.890_000_16, 0.890_000_2, 0.890_000_2));
    let world_from_memorial = memorial_transform.to_matrix();
    let memorial_global = GlobalTransform::from(memorial_transform);
    let bounds =
        AuthoredColliderWorldBounds::from_collider(&memorial, &memorial_global).unwrap();
    let mut lowest_outer_ledge: Option<(AuthoredCollisionTriangle, Vec3, f32)> = None;
    for triangle in memorial.indices.chunks_exact(3) {
        let a = world_from_memorial.transform_point3(memorial.vertices[triangle[0] as usize]);
        let b = world_from_memorial.transform_point3(memorial.vertices[triangle[1] as usize]);
        let c = world_from_memorial.transform_point3(memorial.vertices[triangle[2] as usize]);
        let Some(triangle) = AuthoredCollisionTriangle::new(a, b, c) else {
            continue;
        };
        if triangle.normal.y < AUTHORED_WALKABLE_MIN_UP_DOT {
            continue;
        }
        let centroid = (a + b + c) / 3.0;
        let radius = Vec2::new(
            centroid.x - memorial_transform.translation.x,
            centroid.z - memorial_transform.translation.z,
        )
        .length();
        if (-54.55..=-54.25).contains(&centroid.y)
            && lowest_outer_ledge.is_none_or(|(_, _, current)| radius > current)
        {
            lowest_outer_ledge = Some((triangle, centroid, radius));
        }
    }
    let (lowest_outer_ledge, ledge_centroid, _) =
        lowest_outer_ledge.expect("Hero Square memorial lowest raised walkable tier");
    assert!(
        bounds
            .minimum
            .abs_diff_eq(Vec3::new(-6_320.112, -57.113_22, 1_870.252), 0.001,),
        "unexpected memorial minimum: {:?}",
        bounds.minimum,
    );
    assert!(
        bounds
            .maximum
            .abs_diff_eq(Vec3::new(-6_253.194_3, -31.359_455, 1_936.641_2), 0.001,),
        "unexpected memorial maximum: {:?}",
        bounds.maximum,
    );

    let memorial_center = memorial_transform.translation;
    let outer_face = memorial
        .indices
        .chunks_exact(3)
        .filter_map(|triangle| {
            let a =
                world_from_memorial.transform_point3(memorial.vertices[triangle[0] as usize]);
            let b =
                world_from_memorial.transform_point3(memorial.vertices[triangle[1] as usize]);
            let c =
                world_from_memorial.transform_point3(memorial.vertices[triangle[2] as usize]);
            AuthoredCollisionTriangle::new(a, b, c)
        })
        .filter_map(|triangle| {
            let centroid = (triangle.a + triangle.b + triangle.c) / 3.0;
            let radial = Vec3::new(
                centroid.x - memorial_center.x,
                0.0,
                centroid.z - memorial_center.z,
            );
            let outward = radial.normalize_or_zero();
            (triangle.normal.y.abs() < AUTHORED_WALKABLE_MIN_UP_DOT
                && centroid.y <= bounds.minimum.y + 1.0
                && triangle.normal.dot(outward) > 0.5)
                .then_some((triangle, centroid, outward, radial.length_squared()))
        })
        .max_by(|left, right| left.3.total_cmp(&right.3))
        .expect("Hero Square memorial must retain its outer steep collision face");
    let (_, centroid, outward, _) = outer_face;
    let radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let previous = Vec3::new(
        centroid.x + outward.x * (radius + 0.2),
        bounds.minimum.y,
        centroid.z + outward.z * (radius + 0.2),
    );
    let colliders = [(world_from_memorial, &memorial, &bounds)];
    let contact = resolve_authored_wall_motion_detailed_with_bounds(
        previous,
        -outward * 0.5,
        &colliders,
        false,
    );
    let contact_normal = contact
        .contact_normal
        .expect("the real memorial face must report ControllerColliderHit contact");
    assert!(contact_normal.y < AUTHORED_WALKABLE_MIN_UP_DOT);
    assert!(
        Vec3::new(contact_normal.x, 0.0, contact_normal.z)
            .normalize_or_zero()
            .dot(outward)
            > 0.8,
        "memorial callback normal points through its authored outer face: {contact_normal:?}"
    );

    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    assert!(controller.launch_scripted_ballistic(Vec3::new(0.0, 4.0, 0.0)));
    controller.set_external_collision_result(
        legacy_collision_flag_for_normal(contact_normal),
        Some(contact_normal),
    );
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState::default())
        .init_resource::<crate::movement::MovementIntentQueue>()
        .add_systems(Update, crate::movement::simulate_legacy_players);
    app.update();
    let player = app
        .world_mut()
        .spawn((Transform::from_translation(previous), controller))
        .id();
    app.update();

    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!(controller.surface_sliding());
    assert!(
        (controller.velocity.y - (4.0 - crate::movement::LEGACY_GRAVITY / 60.0) * 2.0).abs()
            < 0.000_01
    );

    // Exercise the user's actual route as well as the isolated callback:
    // jump from the broad bottom slab onto the first raised memorial tier.
    // The tier is 1.6 m above the surrounding slab, just below the
    // baseline jump apex, and its outer lip is a finite convex mesh edge.
    let outward = Vec3::new(
        ledge_centroid.x - memorial_center.x,
        0.0,
        ledge_centroid.z - memorial_center.z,
    )
    .normalize_or_zero();
    let outer_projection = [
        lowest_outer_ledge.a,
        lowest_outer_ledge.b,
        lowest_outer_ledge.c,
    ]
    .into_iter()
    .map(|point| {
        Vec3::new(
            point.x - memorial_center.x,
            0.0,
            point.z - memorial_center.z,
        )
        .dot(outward)
    })
    .fold(f32::NEG_INFINITY, f32::max);
    let approach_xz = memorial_center + outward * (outer_projection + radius + 0.20);
    let approach_y = collider_capsule_ground_contact_with_bounds(
        &memorial,
        world_from_memorial,
        &bounds,
        approach_xz.x,
        approach_xz.z,
        bounds.minimum.y,
        bounds.minimum.y - 1.0,
        ledge_centroid.y - 0.25,
    )
    .expect("the memorial bottom slab must support the approach")
    .0;
    let ledge_y = collider_capsule_ground_contact_with_bounds(
        &memorial,
        world_from_memorial,
        &bounds,
        ledge_centroid.x,
        ledge_centroid.z,
        ledge_centroid.y,
        ledge_centroid.y - 0.5,
        ledge_centroid.y + 0.5,
    )
    .expect("the first memorial tier must support the capsule")
    .0;
    assert!(
        (1.4..1.8).contains(&(ledge_y - approach_y)),
        "unexpected first memorial tier height: approach={approach_y}, ledge={ledge_y}"
    );

    let mut jump_app = App::new();
    jump_app
        .add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState {
            local_axis: Vec2::Y,
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
    jump_app
        .world_mut()
        .spawn((memorial_global, memorial, bounds));
    jump_app.update();
    let mut jump_controller = LegacyPlayerController::from_baseline_table();
    let inward = -outward;
    jump_controller.yaw_degrees = (-inward.x).atan2(inward.z).to_degrees();
    let baseline_jump_force = jump_controller.jump_height_server_units as f32
        * crate::movement::SERVER_TO_CLIENT_SCALE;
    let jumper = jump_app
        .world_mut()
        .spawn((
            Transform::from_xyz(approach_xz.x, approach_y, approach_xz.z),
            jump_controller,
        ))
        .id();
    let mut mounted = false;
    let mut saw_original_edge_boost = false;
    let mut crossed_upper_lip_while_rising = false;
    for frame in 0..120 {
        jump_app.update();
        if frame == 0 {
            jump_app
                .world_mut()
                .resource_mut::<crate::movement::LegacyInputState>()
                .jump_just_pressed = false;
        }
        let position = jump_app
            .world()
            .get::<Transform>(jumper)
            .unwrap()
            .translation;
        let controller = jump_app
            .world()
            .get::<LegacyPlayerController>(jumper)
            .unwrap();
        saw_original_edge_boost |= controller.surface_sliding()
            && controller.velocity.y > baseline_jump_force + AUTHORED_COLLISION_EPSILON;
        let radial_projection = Vec3::new(
            position.x - memorial_center.x,
            0.0,
            position.z - memorial_center.z,
        )
        .dot(outward);
        crossed_upper_lip_while_rising |= controller.jumping
            && controller.velocity.y > 0.0
            && position.y >= ledge_y - 0.05
            && radial_projection < outer_projection + radius;
        mounted |= controller.grounded && !controller.jumping && position.y >= ledge_y - 0.05;
    }
    let final_position = jump_app
        .world()
        .get::<Transform>(jumper)
        .unwrap()
        .translation;
    assert!(
        mounted,
        "baseline jump could not mount the real Hero Square upper edge: \
         approach={approach_xz:?}/{approach_y}, ledge={ledge_centroid:?}/{ledge_y}, \
        final={final_position:?}"
    );
    assert!(
        saw_original_edge_boost,
        "the real Hero Square side contact never doubled the submitted upward Move"
    );
    assert!(
        crossed_upper_lip_while_rising,
        "the boosted capsule never crossed the real upper lip while it was still rising"
    );
    assert!(
        (Vec2::new(controller.velocity.x, controller.velocity.z).length()
            - crate::movement::LEGACY_SURFACE_SLIDE_SPEED)
            .abs()
            < 0.000_01
    );
}
