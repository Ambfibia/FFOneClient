use super::*;

#[test]
fn authored_ground_rejects_slopes_over_the_exact_fifty_degree_limit() {
    let collider = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/slope.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 3.464_101_6, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
        ]
        .into(),
        indices: vec![0, 2, 1].into(),
        local_min: Vec3::ZERO,
        local_max: Vec3::new(2.0, 3.464_101_6, 2.0),
        is_trigger: false,
    };
    assert!(
        collider_ground_height(&collider, Mat4::IDENTITY, 0.5, 0.5, -1.0, 4.0).is_none(),
        "a 60-degree face is a wall, not ground"
    );

    let walkable = AuthoredTriMeshCollider {
        vertices: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
        ]
        .into(),
        local_max: Vec3::new(2.0, 2.0, 2.0),
        ..collider
    };
    assert!(
        collider_ground_height(&walkable, Mat4::IDENTITY, 0.5, 0.5, -1.0, 4.0).is_some(),
        "a 45-degree face remains walkable"
    );
}

#[test]
fn authored_segment_hit_is_two_sided_and_returns_the_first_crossing() {
    let a = Vec3::new(0.0, 0.0, -1.0);
    let b = Vec3::new(0.0, 2.0, -1.0);
    let c = Vec3::new(0.0, 0.0, 1.0);
    let forward =
        segment_triangle_hit(Vec3::new(-2.0, 0.5, 0.0), Vec3::new(2.0, 0.5, 0.0), a, b, c)
            .unwrap();
    let backward =
        segment_triangle_hit(Vec3::new(2.0, 0.5, 0.0), Vec3::new(-2.0, 0.5, 0.0), a, b, c)
            .unwrap();
    assert!((forward.fraction - 0.5).abs() <= f32::EPSILON);
    assert!((backward.fraction - 0.5).abs() <= f32::EPSILON);
    assert!(
        forward
            .point
            .abs_diff_eq(Vec3::new(0.0, 0.5, 0.0), f32::EPSILON)
    );
    assert!(segment_triangle_hit(Vec3::Y, Vec3::Y * 2.0, a, b, c).is_none());
}

#[test]
fn authored_wall_probe_blocks_vertical_faces_but_not_walkable_ground() {
    let wall = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/wall.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, 3.0, -2.0),
            Vec3::new(0.0, 0.0, 2.0),
        ]
        .into(),
        indices: vec![0, 1, 2].into(),
        local_min: Vec3::new(0.0, 0.0, -2.0),
        local_max: Vec3::new(0.0, 3.0, 2.0),
        is_trigger: false,
    };
    let hit = collider_wall_hit(
        &wall,
        Mat4::IDENTITY,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
    )
    .unwrap();
    let expected_fraction = (1.0
        - (AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH))
        * 0.5;
    assert!((hit.fraction - expected_fraction).abs() <= f32::EPSILON);
    let overlap = collider_wall_penetration(&wall, Mat4::IDENTITY, Vec3::new(0.1, 0.0, 0.0))
        .expect("capsule overlap must recover from a wall face");
    assert!(overlap.x > 0.0);
    let edge_overlap =
        collider_wall_penetration(&wall, Mat4::IDENTITY, Vec3::new(0.05, 0.0, -2.1))
            .expect("capsule overlap must include a finite triangle edge");
    assert!(edge_overlap.length() > 0.0);

    let angular_band = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/angular-band.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.5, -1.0),
            Vec3::new(0.0, 0.6, -1.0),
            Vec3::new(0.0, 0.5, 1.0),
            Vec3::new(0.0, 0.6, 1.0),
        ]
        .into(),
        indices: vec![0, 1, 2, 2, 1, 3].into(),
        local_min: Vec3::new(0.0, 0.5, -1.0),
        local_max: Vec3::new(0.0, 0.6, 1.0),
        is_trigger: false,
    };
    let angular_overlap =
        collider_wall_penetration(&angular_band, Mat4::IDENTITY, Vec3::new(0.2, 0.0, 0.0))
            .expect("the complete capsule segment must block between old height probes");
    assert!(angular_overlap.x > 0.0);

    let low_step = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/low-step.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, GROUNDED_STEP_UP, -2.0),
            Vec3::new(0.0, 0.0, 2.0),
        ]
        .into(),
        indices: vec![0, 1, 2].into(),
        local_min: Vec3::new(0.0, 0.0, -2.0),
        local_max: Vec3::new(0.0, GROUNDED_STEP_UP, 2.0),
        is_trigger: false,
    };
    assert!(
        collider_wall_penetration(&low_step, Mat4::IDENTITY, Vec3::new(0.1, 0.0, 0.0))
            .is_none(),
        "the exact capsule must step over a riser at CharacterController.stepOffset"
    );

    let floor = AuthoredTriMeshCollider {
        source_model_path: "models/floor.glb".to_owned(),
        vertices: vec![
            Vec3::new(-2.0, 0.0, -2.0),
            Vec3::new(2.0, 0.0, -2.0),
            Vec3::new(-2.0, 0.0, 2.0),
        ]
        .into(),
        indices: vec![0, 2, 1].into(),
        local_min: Vec3::new(-2.0, 0.0, -2.0),
        local_max: Vec3::new(2.0, 0.0, 2.0),
        ..wall
    };
    assert!(
        collider_wall_hit(
            &floor,
            Mat4::IDENTITY,
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
        )
        .is_none()
    );
    assert!(
        collider_wall_penetration(&floor, Mat4::IDENTITY, Vec3::ZERO).is_none(),
        "walkable ground is handled only by the ground-height solver"
    );
}

#[test]
fn rising_capsule_keeps_sliding_around_a_sharp_finite_edge() {
    let narrow_wall = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/rising-sharp-edge.glb".to_owned(),
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
    let colliders = [(Mat4::IDENTITY, &narrow_wall)];
    let mut feet = Vec3::new(0.35, 0.0, 0.30);
    for frame in 0..30 {
        let previous = feet;
        feet = resolve_authored_wall_motion(feet, Vec3::new(-0.08, 0.04, -0.04), &colliders);
        assert!(
            feet.y >= previous.y + 0.039,
            "sharp-edge contact stalled the rising jump on frame {frame}: \
             previous={previous:?}, resolved={feet:?}"
        );
    }
    assert!(
        feet.z < -0.5,
        "the capsule accumulated obsolete edge normals instead of sliding around the lip: \
         {feet:?}"
    );
}

#[test]
fn lower_capsule_sphere_keeps_the_upward_normal_at_a_finite_ledge_lip() {
    let triangle = AuthoredCollisionTriangle::new(
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.5, 1.0),
        Vec3::new(0.0, 1.5, -1.0),
    )
    .expect("vertical ledge triangle");
    assert!(triangle.normal.x < -0.99);

    // The lower sphere center is (-0.2, 1.6, 0): it overlaps the finite
    // top edge at (0, 1.5, 0). A plane-only wall correction has Y=0 and
    // pins the controller until its feet nearly reach the top. Unity CCT
    // instead returns the radial edge normal and starts lifting now.
    let feet = Vec3::new(-0.2, 1.3, 0.0);
    let push = deepest_authored_capsule_penetration(feet, feet.y, &[triangle], false)
        .expect("lower sphere must overlap the finite upper edge");
    assert!(
        push.x < 0.0,
        "ledge still has to reject inward motion: {push:?}"
    );
    assert!(
        push.y > AUTHORED_COLLISION_EPSILON,
        "finite upper edge discarded the source upward recovery: {push:?}"
    );
}

#[test]
fn rising_capsule_rolls_over_a_reachable_convex_box_corner() {
    let ledge = closed_vehicle_test_collider();
    let world_from_ledge = Mat4::from_scale_rotation_translation(
        Vec3::new(4.0, 0.9, 1.0),
        Quat::IDENTITY,
        Vec3::new(4.0, 0.0, 0.0),
    );
    let colliders = [(world_from_ledge, &ledge)];

    let callback_triangles =
        collect_authored_motion_triangles(Vec3::new(-0.19, 1.3, 1.19), Vec3::ZERO, &colliders);
    let callback_normal = authored_capsule_callback_normal(
        Vec3::new(-0.19, 1.3, 1.19),
        1.3,
        &callback_triangles,
        false,
    )
    .expect("convex corner must report a ControllerColliderHit face");
    assert!(
        callback_normal.y.abs() <= AUTHORED_COLLISION_EPSILON
            && callback_normal.x < -0.6
            && callback_normal.z > 0.6,
        "ControllerColliderHit must retain the rounded capsule normal at a finite convex \
         vertex: {callback_normal:?}"
    );

    // Once the same lower sphere has moved above an upper lip, the finite
    // feature normal rotates upward through slopeLimit. That is the actual
    // ControllerColliderHit normal, but PhysX still reports `Sides` when
    // it was produced by the SIDE pass; collision flags are not derived
    // from this normal.
    let upper_lip_feet = Vec3::new(-0.16, 1.70, 0.0);
    let upper_lip_triangles =
        collect_authored_motion_triangles(upper_lip_feet, Vec3::ZERO, &colliders);
    let upper_lip_normal = authored_capsule_callback_normal(
        upper_lip_feet,
        upper_lip_feet.y,
        &upper_lip_triangles,
        false,
    )
    .expect("upper lip must report its rounded ControllerColliderHit normal");
    assert!(
        upper_lip_normal.x < 0.0
            && upper_lip_normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT
            && upper_lip_normal.z.abs() <= AUTHORED_COLLISION_EPSILON,
        "upper lip did not produce its rounded upward callback normal: {upper_lip_normal:?}"
    );
    let mut final_callback_normal = None;
    retain_source_contact_normal(&mut final_callback_normal, Some(callback_normal));
    retain_source_contact_normal(&mut final_callback_normal, Some(upper_lip_normal));
    assert!(
        final_callback_normal
            .unwrap()
            .abs_diff_eq(upper_lip_normal, AUTHORED_COLLISION_EPSILON),
        "the later upper-lip callback did not replace the earlier side callback"
    );

    // This approaches the exposed (-X,+Z) top corner diagonally.  A
    // plane-only response sees two walls and pins the capsule; the source
    // CCT instead rolls its lower sphere around the single finite corner.
    let mut feet = Vec3::new(-0.34, 1.20, 1.34);
    for frame in 0..24 {
        let previous = feet;
        feet = resolve_authored_wall_motion(feet, Vec3::new(0.04, 0.04, -0.04), &colliders);
        assert!(
            feet.y >= previous.y + 0.02,
            "convex top corner stalled the rising capsule on frame {frame}: \
             previous={previous:?}, resolved={feet:?}"
        );
    }
    assert!(
        feet.x > 0.1 && feet.z < 0.9,
        "the rising capsule never rolled onto the top past the convex corner: {feet:?}"
    );
}

#[test]
fn rising_controller_uses_up_then_side_and_does_not_ground_on_upper_lip_normal() {
    let ledge = closed_vehicle_test_collider();
    let ledge_global = GlobalTransform::from(
        Transform::from_xyz(4.0, 0.0, 0.0).with_scale(Vec3::new(4.0, 0.9, 1.0)),
    );
    let ledge_bounds =
        AuthoredColliderWorldBounds::from_collider(&ledge, &ledge_global).unwrap();
    let colliders = [(ledge_global.to_matrix(), &ledge, &ledge_bounds)];
    let previous = Vec3::new(-0.18, 1.66, 0.0);
    let submitted = Vec3::new(0.10, 0.04, 0.0);
    let resolved = resolve_authored_controller_motion_with_bounds(
        previous,
        submitted,
        submitted.y,
        &colliders,
        true,
    );

    assert!(
        resolved.collision_flags & crate::movement::LEGACY_COLLISION_SIDES != 0,
        "the raised horizontal sweep did not report its upper-lip SIDE contact: \
         {resolved:?}"
    );
    assert!(
        resolved.collision_flags & LEGACY_COLLISION_BELOW == 0,
        "an upward-facing SIDE callback was incorrectly converted to Below: {resolved:?}"
    );
    assert!(
        resolved.position.y >= previous.y + submitted.y - AUTHORED_COLLISION_EPSILON,
        "the SIDE pass lost height already gained by the preceding UP pass: {resolved:?}"
    );
}

#[test]
fn descending_controller_keeps_the_step_up_when_down_hits_a_walkable_upper_lip() {
    let ledge = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/steep-sided-upper-lip.glb".to_owned(),
        vertices: vec![
            Vec3::new(0.0, 2.0, -1.0),
            Vec3::new(-0.4, 0.0, -1.0),
            Vec3::new(-0.4, 0.0, 1.0),
            Vec3::new(0.0, 2.0, 1.0),
            Vec3::new(4.0, 2.0, -1.0),
            Vec3::new(4.0, 2.0, 1.0),
        ]
        .into(),
        indices: vec![0, 1, 2, 0, 2, 3, 0, 3, 5, 0, 5, 4].into(),
        local_min: Vec3::new(-0.4, 0.0, -1.0),
        local_max: Vec3::new(4.0, 2.0, 1.0),
        is_trigger: false,
    };
    let ledge_global = GlobalTransform::IDENTITY;
    let ledge_bounds =
        AuthoredColliderWorldBounds::from_collider(&ledge, &ledge_global).unwrap();
    let colliders = [(ledge_global.to_matrix(), &ledge, &ledge_bounds)];

    // The descending controller is already high enough for the artificial
    // stepOffset UP to put its lower sphere above the lip. SIDE must keep
    // that progress, then DOWN lands on the walkable top. The adjacent
    // steep face is close to the capsule, but it is not the triangle
    // touched by DOWN and must not start WALK_EXPERIMENT.
    let previous = Vec3::new(-0.28, 1.65, 0.0);
    let submitted = Vec3::new(0.18, -0.14, 0.0);
    assert!(
        !authored_down_contact_is_non_walkable(
            Vec3::new(-0.10, 1.95, 0.0),
            previous.y,
            Vec3::Y,
            &colliders,
        ),
        "a walkable upper-lip DOWN contact inherited the neighbouring steep face"
    );
    assert!(
        authored_down_contact_is_non_walkable(
            Vec3::new(-0.10, 1.95, 0.0),
            previous.y,
            Vec3::new(-5.0, 1.0, 0.0).normalize(),
            &colliders,
        ),
        "the same authored feature must remain non-walkable when DOWN actually hits its steep face"
    );
    let resolved = resolve_authored_controller_motion_with_bounds(
        previous,
        submitted,
        submitted.y,
        &colliders,
        false,
    );

    assert!(
        resolved.collision_flags & LEGACY_COLLISION_BELOW != 0,
        "DOWN discarded the reachable upper-lip contact: {resolved:?}"
    );
    assert!(
        resolved.position.x > -0.20,
        "a neighbouring side face falsely rolled back the reachable upper edge: {resolved:?}"
    );
    assert!(
        resolved.position.y >= previous.y,
        "the finite upper edge discarded its rounded upward recovery: {resolved:?}"
    );
}

#[test]
fn baseline_jump_can_mount_a_reachable_box_top_through_its_convex_corner() {
    let mut floor = closed_vehicle_test_collider();
    floor.source_model_path = "models/convex-corner-floor.glb".to_owned();
    floor.vertices = vec![
        Vec3::new(-8.0, 0.0, -8.0),
        Vec3::new(8.0, 0.0, -8.0),
        Vec3::new(-8.0, 0.0, 8.0),
        Vec3::new(8.0, 0.0, 8.0),
    ]
    .into();
    floor.indices = vec![0, 2, 1, 2, 3, 1].into();
    floor.local_min = Vec3::new(-8.0, 0.0, -8.0);
    floor.local_max = Vec3::new(8.0, 0.0, 8.0);
    let floor_global = GlobalTransform::IDENTITY;
    let floor_bounds =
        AuthoredColliderWorldBounds::from_collider(&floor, &floor_global).unwrap();

    let mut ledge = closed_vehicle_test_collider();
    ledge.source_model_path = "models/convex-corner-ledge.glb".to_owned();
    let ledge_global = GlobalTransform::from(
        Transform::from_xyz(4.0, 0.0, 0.0).with_scale(Vec3::new(4.0, 1.0, 1.0)),
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

    let starts = [
        ("straight-close", Vec3::new(-0.50, 0.0, 0.0), 0.0),
        ("straight-early", Vec3::new(-0.75, 0.0, 0.0), 0.0),
        ("straight-mid", Vec3::new(-1.00, 0.0, 0.0), 0.0),
        ("diagonal-near", Vec3::new(-1.25, 0.0, -2.25), 45.0),
        ("diagonal-mid", Vec3::new(-1.35, 0.0, -2.35), 45.0),
        ("diagonal-far", Vec3::new(-1.45, 0.0, -2.45), 45.0),
        ("diagonal-limit", Vec3::new(-1.55, 0.0, -2.55), 45.0),
        // The real gameplay complaint is normally a top *edge*, not the
        // intersection of two side faces. Exercise a straight approach
        // to the middle of the -X face as well as the convex box corner.
        ("straight-upper-edge", Vec3::new(-1.25, 0.0, 0.0), 0.0),
        ("straight-late", Vec3::new(-1.75, 0.0, 0.0), 0.0),
        ("straight-apex", Vec3::new(-2.25, 0.0, 0.0), 0.0),
    ];
    let players = starts
        .into_iter()
        .map(|(_, position, yaw_degrees)| {
            let mut controller = LegacyPlayerController::from_baseline_table();
            controller.yaw_degrees = yaw_degrees;
            app.world_mut()
                .spawn((Transform::from_translation(position), controller))
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
                && (position.y - 2.0).abs() <= GROUND_EPSILON;
        }
    }

    for (((label, start, _), landed), player) in starts.into_iter().zip(landed).zip(players) {
        let final_position = app.world().get::<Transform>(player).unwrap().translation;
        assert!(
            landed,
            "baseline diagonal jump could not mount the reachable convex corner from \
             {label} start={start:?}: final={final_position:?}"
        );
    }
}

#[test]
fn walkable_slope_constraint_preserves_three_dimensional_run_speed() {
    let angle = 30.0_f32.to_radians();
    let normal = Vec3::new(-angle.sin(), angle.cos(), 0.0);
    let requested = Vec3::new(0.1, 0.0, 0.0);
    let constrained = constrain_displacement_to_walkable_surface(requested, normal);
    assert!(constrained.dot(normal).abs() <= AUTHORED_COLLISION_EPSILON);
    assert!((constrained.length() - requested.length()).abs() <= f32::EPSILON);
    assert!(constrained.x < requested.x && constrained.y > 0.0);

    assert!(
        constrain_displacement_to_walkable_surface(requested, Vec3::Y)
            .abs_diff_eq(requested, f32::EPSILON)
    );
}

#[test]
fn grounded_controller_keeps_baseline_speed_across_a_walkable_ramp() {
    let tangent = 30.0_f32.to_radians().tan();
    let ramp = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/constant-speed-ramp.glb".to_owned(),
        vertices: vec![
            Vec3::new(-5.0, -5.0 * tangent, -2.0),
            Vec3::new(5.0, 5.0 * tangent, -2.0),
            Vec3::new(-5.0, -5.0 * tangent, 2.0),
            Vec3::new(5.0, 5.0 * tangent, 2.0),
        ]
        .into(),
        indices: vec![0, 2, 1, 2, 3, 1].into(),
        local_min: Vec3::new(-5.0, -5.0 * tangent, -2.0),
        local_max: Vec3::new(5.0, 5.0 * tangent, 2.0),
        is_trigger: false,
    };
    let ramp_global = GlobalTransform::IDENTITY;
    let ramp_bounds = AuthoredColliderWorldBounds::from_collider(&ramp, &ramp_global).unwrap();
    let raw_start = Vec3::new(-3.0, -3.0 * tangent, 0.0);
    let (start_y, normal) = collider_capsule_ground_contact_with_bounds(
        &ramp,
        Mat4::IDENTITY,
        &ramp_bounds,
        raw_start.x,
        raw_start.z,
        raw_start.y,
        raw_start.y - GROUND_EPSILON,
        raw_start.y + GROUNDED_STEP_UP,
    )
    .expect("walkable ramp support");
    assert!((normal.y - 30.0_f32.to_radians().cos()).abs() < 0.000_01);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .insert_resource(crate::movement::LegacyInputState {
            // Zero-yaw legacy left maps to native +X, directly uphill.
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
    app.world_mut().spawn((ramp_global, ramp, ramp_bounds));
    app.update();
    let player = app
        .world_mut()
        .spawn((
            Transform::from_xyz(raw_start.x, start_y, raw_start.z),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();

    // The first Move establishes the preceding CCT Below/normal state.
    app.update();
    let measured_start = app.world().get::<Transform>(player).unwrap().translation;
    for _ in 0..30 {
        app.update();
    }
    let measured_end = app.world().get::<Transform>(player).unwrap().translation;
    let displacement = measured_end - measured_start;
    let expected_distance = 30.0
        * crate::movement::LEGACY_BASE_RUN_SPEED_SERVER_UNITS as f32
        * crate::movement::SERVER_TO_CLIENT_SCALE
        / 60.0;
    assert!(
        (displacement.length() - expected_distance).abs() <= 0.002,
        "walkable ramp changed baseline surface speed: displacement={displacement:?}, \
         distance={}, expected={expected_distance}",
        displacement.length()
    );
    assert!(
        displacement.normalize().dot(normal).abs() <= 0.001,
        "grounded movement left the supporting ramp plane: {displacement:?}"
    );
}

#[test]
fn authored_capsule_motion_lands_on_a_platform_during_diagonal_fall() {
    let platform = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/platform.glb".to_owned(),
        vertices: vec![
            Vec3::new(-1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, -1.0),
            Vec3::new(-1.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ]
        .into(),
        indices: vec![0, 2, 1, 1, 2, 3].into(),
        local_min: Vec3::new(-1.0, 0.0, -1.0),
        local_max: Vec3::new(1.0, 0.0, 1.0),
        is_trigger: false,
    };
    let previous = Vec3::new(-0.6, 1.0, 0.0);
    let displacement = Vec3::new(1.2, -2.0, 0.0);
    let colliders = [(Mat4::IDENTITY, &platform)];
    let resolved = resolve_authored_wall_motion(previous, displacement, &colliders);
    assert!(
        resolved.y >= -AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH - AUTHORED_COLLISION_EPSILON,
        "the unified 3D capsule sweep must not cross the platform top: {resolved:?}"
    );
    assert!(
        resolved.x > 0.5,
        "landing must preserve the horizontal part of the diagonal motion"
    );
    let snapped_height = collider_ground_height(
        &platform,
        Mat4::IDENTITY,
        resolved.x,
        resolved.z,
        resolved.y - GROUND_EPSILON,
        resolved.y - displacement.y + GROUND_EPSILON,
    )
    .expect("the post-sweep ground stage must observe the crossed platform top");
    assert!(snapped_height.abs() <= f32::EPSILON);
}

#[test]
fn authored_motion_narrow_phase_keeps_only_nearby_triangles() {
    let mut vertices = vec![
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 2.0, -1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    let mut indices = vec![0, 1, 2];
    for index in 0..256_u32 {
        let base = vertices.len() as u32;
        let x = 100.0 + index as f32;
        vertices.extend([
            Vec3::new(x, 0.0, -1.0),
            Vec3::new(x, 2.0, -1.0),
            Vec3::new(x, 0.0, 1.0),
        ]);
        indices.extend([base, base + 1, base + 2]);
    }
    let collider = AuthoredTriMeshCollider {
        source_mesh: Handle::default(),
        source_model_path: "models/long-collider-strip.glb".to_owned(),
        vertices: vertices.into(),
        indices: indices.into(),
        local_min: Vec3::new(0.0, 0.0, -1.0),
        local_max: Vec3::new(355.0, 2.0, 1.0),
        is_trigger: false,
    };
    let colliders = [(Mat4::IDENTITY, &collider)];
    let candidates = collect_authored_motion_triangles(
        Vec3::new(-0.5, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        &colliders,
    );
    assert_eq!(
        candidates.len(),
        1,
        "repeated capsule iterations must not rescan distant mesh triangles"
    );
}

#[test]
fn oblique_corner_clipping_never_reintroduces_an_earlier_wall_component() {
    let contacts = [Vec3::X, Vec3::new(-1.0, 0.0, 1.0).normalize()];
    let requested = Vec3::new(-1.0, 0.0, -1.0);
    let clipped = clip_displacement_against_contacts(requested, &contacts);
    assert!(
        contacts
            .iter()
            .all(|normal| clipped.dot(*normal) >= -AUTHORED_COLLISION_EPSILON),
        "the contact cone must satisfy every wall, got {clipped:?}"
    );
}
