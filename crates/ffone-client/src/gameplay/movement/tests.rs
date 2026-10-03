use std::time::Duration;

use bevy::time::TimeUpdateStrategy;

use crate::movement::*;

#[test]
fn vehicle_acceleration_coasting_and_dismount_preserve_momentum_contract() {
    let mut controller = LegacyPlayerController::from_server_attributes(600, 568);
    controller.set_vehicle_speed(Some(1100));
    let first = controller.step_vehicle_horizontal(Vec2::Y, 0.0, 1.0 / 60.0);
    assert!(first.length() > 0.0 && first.length() < 1.0);
    for _ in 0..600 { controller.step_vehicle_horizontal(Vec2::Y, 0.0, 1.0 / 60.0); }
    let top = controller.vehicle_momentum.length();
    assert!((top - 14.85).abs() < 0.001);
    let coast = controller.step_vehicle_horizontal(Vec2::ZERO, 0.0, 1.0 / 60.0).length();
    assert!((coast - top * 0.98).abs() < 0.0001);
    controller.set_vehicle_speed(Some(1100));
    assert!((controller.vehicle_momentum.length() - coast).abs() < 0.0001);
    controller.set_vehicle_speed(None);
    assert_eq!(controller.vehicle_momentum, Vec3::ZERO);
}

#[test]
fn vehicle_direction_caps_and_air_acceleration_are_distinct() {
    for (axis, multiplier) in [(Vec2::NEG_Y, 0.54), (Vec2::ONE, 1.215), (Vec2::X, 0.9)] {
        let mut controller = LegacyPlayerController::from_baseline_table();
        controller.set_vehicle_speed(Some(1100));
        for _ in 0..1200 { controller.step_vehicle_horizontal(axis, 0.0, 1.0 / 60.0); }
        assert!((controller.vehicle_momentum.length() - 11.0 * multiplier).abs() < 0.001);
    }
    let mut ground = LegacyPlayerController::from_baseline_table();
    ground.set_vehicle_speed(Some(1100));
    let mut air = ground.clone();
    air.grounded = false;
    assert!(air.step_vehicle_horizontal(Vec2::Y, 0.0, 1.0 / 60.0).length()
        < ground.step_vehicle_horizontal(Vec2::Y, 0.0, 1.0 / 60.0).length());
}

#[test]
fn vehicle_coasting_sends_move_until_final_stop_and_dismount_restores_walking() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
        .insert_resource(LegacyInputState { local_axis: Vec2::Y, ..default() })
        .init_resource::<MovementIntentQueue>()
        .add_systems(Update, simulate_legacy_players);
    app.update();
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_vehicle_speed(Some(1100));
    let player = app.world_mut().spawn((Transform::default(), controller)).id();
    for _ in 0..120 { app.update(); }
    while app.world_mut().resource_mut::<MovementIntentQueue>().pop_front().is_some() {}
    app.world_mut().resource_mut::<LegacyInputState>().local_axis = Vec2::ZERO;
    let mut moves = 0;
    let mut stops = 0;
    for _ in 0..600 {
        app.update();
        while let Some(intent) = app.world_mut().resource_mut::<MovementIntentQueue>().pop_front() {
            match intent {
                MovementIntent::Move(request) => { assert_eq!(stops, 0); assert!(request.speed > 0); moves += 1; }
                MovementIntent::Stop(_) => stops += 1,
                MovementIntent::Jump(_) => panic!("grounded vehicle must not emit jump"),
            }
        }
    }
    assert!(moves > 2);
    assert_eq!(stops, 1);
    app.world_mut().get_mut::<LegacyPlayerController>(player).unwrap().set_vehicle_speed(None);
    app.world_mut().resource_mut::<LegacyInputState>().local_axis = Vec2::Y;
    app.update();
    let velocity = app.world().get::<LegacyPlayerController>(player).unwrap().velocity;
    assert!((Vec2::new(velocity.x, velocity.z).length() - 6.0).abs() < 0.001);
}

#[test]
fn movement_uses_the_shared_reflected_coordinate_contract() {
    let server = ProtocolPosition::new([125, 375, -250]);
    let native = server.to_native();
    assert_eq!(native, Vec3::new(-1.25, -2.5, 3.75));
    assert_eq!(ProtocolPosition::from_native(native), server);

    assert_eq!(
        ProtocolMoveVelocity::from_native(Vec3::new(-1.5, -2.0, 3.25)).raw(),
        [1.5, 3.25, -2.0]
    );
    assert_eq!(
        ProtocolScaledVelocity::from_native(Vec3::new(-1.5, -2.0, 3.25)).raw(),
        [150, 325, -200]
    );
}

#[test]
fn direction_matrix_preserves_legacy_key_values() {
    assert_eq!(legacy_direction_key(Vec2::ZERO), 0);
    assert_eq!(legacy_direction_key(Vec2::new(0.0, 1.0)), 1); // W / Up
    assert_eq!(legacy_direction_key(Vec2::new(1.0, 1.0)), 2); // D + W
    assert_eq!(legacy_direction_key(Vec2::new(1.0, 0.0)), 3); // D
    assert_eq!(legacy_direction_key(Vec2::new(1.0, -1.0)), 4); // D + S
    assert_eq!(legacy_direction_key(Vec2::new(0.0, -1.0)), 5); // S / Down
    assert_eq!(legacy_direction_key(Vec2::new(-1.0, -1.0)), 6); // A + S
    assert_eq!(legacy_direction_key(Vec2::new(-1.0, 0.0)), 7); // A
    assert_eq!(legacy_direction_key(Vec2::new(-1.0, 1.0)), 8); // A + W
}

#[test]
fn primary_and_arrow_axes_keep_configurable_input_precedence() {
    let gate = LegacyInputGate::default();
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);
    assert_eq!(movement_axes_from_keys(&keys, &gate), Vec2::new(0.0, -1.0));

    keys.release(KeyCode::KeyS);
    keys.press(KeyCode::ArrowDown);
    assert_eq!(movement_axes_from_keys(&keys, &gate), Vec2::ZERO);

    keys.release(KeyCode::KeyW);
    keys.release(KeyCode::KeyS);
    keys.release(KeyCode::ArrowDown);
    keys.press(KeyCode::KeyA);
    keys.press(KeyCode::ArrowLeft);
    // Left arrow turns/recenters the camera; only A contributes to strafe.
    assert_eq!(movement_axes_from_keys(&keys, &gate), Vec2::new(-1.0, 0.0));
}

#[test]
fn input_gate_is_permissive_by_default_and_filters_axes_independently() {
    let default_gate = LegacyInputGate::default();
    assert!(default_gate.allow_forward);
    assert!(default_gate.allow_backward);
    assert!(default_gate.allow_strafe);
    assert!(default_gate.allow_keyboard_turning);
    assert!(default_gate.allow_jump);
    assert!(default_gate.allow_mouse_camera);

    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);
    keys.press(KeyCode::KeyD);

    let no_forward = LegacyInputGate {
        allow_forward: false,
        ..default_gate
    };
    assert_eq!(
        movement_axes_from_keys(&keys, &no_forward),
        Vec2::new(1.0, -1.0)
    );

    let no_backward = LegacyInputGate {
        allow_backward: false,
        ..default_gate
    };
    assert_eq!(
        movement_axes_from_keys(&keys, &no_backward),
        Vec2::new(1.0, 1.0)
    );

    let no_strafe = LegacyInputGate {
        allow_strafe: false,
        ..default_gate
    };
    assert_eq!(
        movement_axes_from_keys(&keys, &no_strafe),
        Vec2::new(0.0, -1.0)
    );
}

#[test]
fn baseline_attributes_match_avatar_table_row_one() {
    let controller = LegacyPlayerController::from_baseline_table();
    assert_eq!(LEGACY_BASE_RUN_SPEED_SERVER_UNITS, 600);
    assert_eq!(LEGACY_BASE_JUMP_HEIGHT_SERVER_UNITS, 568);
    assert_eq!(controller.run_speed_server_units, 600);
    assert_eq!(controller.jump_height_server_units, 568);
    assert!(!controller.is_auto_running());
}

#[test]
fn orbit_camera_uses_serialized_main_camera_overrides() {
    let target = Entity::PLACEHOLDER;
    let camera = LegacyOrbitCamera::new(target);

    assert_eq!(camera.target, target);
    assert_eq!(camera.height, 1.4);
    assert_eq!(camera.minimum_distance, 4.0);
    assert_eq!(camera.maximum_distance, 12.0);
    assert_eq!(camera.distance, 5.0);
    assert_eq!(camera.default_distance, 7.0);
    assert_eq!(camera.pitch_degrees, 0.0);
    assert_eq!(camera.default_pitch_degrees, 11.5);
    assert_eq!(camera.minimum_pitch_degrees, -30.0);
    assert_eq!(camera.maximum_pitch_degrees, 70.0);
    assert_eq!(camera.mouse_axis_per_pixel, 0.03);
    assert_eq!(camera.scroll_axis_per_line, 0.1);
}

#[test]
fn orbit_camera_accepts_a_modal_subtarget_without_a_player_controller() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, update_legacy_camera_pose);
    let target = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .id();
    let camera = app
        .world_mut()
        .spawn((LegacyOrbitCamera::new(target), Transform::default()))
        .id();

    app.update();

    let camera_transform = app.world().get::<Transform>(camera).unwrap();
    let focus = Vec3::new(1.0, 3.4, 3.0);
    assert!((camera_transform.translation.distance(focus) - 4.0).abs() < 0.000_1);
    assert!(camera_transform.rotation.is_finite());
}

#[test]
fn nano_subtarget_keeps_exact_height_and_distance_without_orbit_correction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, update_legacy_camera_pose);
    let target = app
        .world_mut()
        .spawn(Transform::from_xyz(1.0, 3.0, 2.0))
        .id();
    let mut camera = LegacyOrbitCamera::new(target);
    camera.height = 0.6 * 0.6;
    camera.distance = 2.0;
    camera.pitch_degrees = -30.0;
    camera.sub_target_forward = Some(Vec3::NEG_Z);
    let entity = app.world_mut().spawn((camera, Transform::default())).id();
    app.update();
    let transform = app.world().get::<Transform>(entity).unwrap();
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(1.0, 3.36, 4.0), 0.0001)
    );
    assert!(
        transform
            .forward()
            .as_vec3()
            .abs_diff_eq(Vec3::NEG_Z, 0.0001)
    );
}

#[test]
fn disabled_mouse_camera_gate_blocks_all_keyboard_zoom_bindings() {
    for key in [KeyCode::Equal, KeyCode::Minus] {
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(key);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(keyboard)
            .insert_resource(LegacyInputGate {
                allow_mouse_camera: false,
                ..default()
            })
            .add_systems(Update, update_legacy_camera_input);
        let camera_entity = app
            .world_mut()
            .spawn(LegacyOrbitCamera::new(Entity::PLACEHOLDER))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<LegacyOrbitCamera>(camera_entity)
                .expect("test orbit camera")
                .distance,
            5.0,
            "{key:?} changed camera distance while input was disabled"
        );
    }
}

#[test]
fn configured_camera_turn_replaces_legacy_fixed_key() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyQ);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
        .insert_resource(keys)
        .init_resource::<LegacyInputGate>()
        .init_resource::<LegacyCameraKeyInput>()
        .add_systems(Update, update_legacy_camera_input);
    let camera = app.world_mut().spawn(LegacyOrbitCamera::new(Entity::PLACEHOLDER)).id();
    app.update();
    assert_eq!(app.world().get::<LegacyOrbitCamera>(camera).unwrap().yaw_degrees, 0.0);
    app.world_mut().resource_mut::<LegacyCameraKeyInput>().turn_left = true;
    app.update();
    assert!(app.world().get::<LegacyOrbitCamera>(camera).unwrap().yaw_degrees < 0.0);
}

#[test]
fn stand_force_reset_only_clears_the_current_direction() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.current_direction_key = 5;
    controller.last_direction_key = 5;

    controller.reset_current_move_direction();

    assert_eq!(controller.current_direction_key(), 0);
    assert_eq!(controller.last_direction_key, 5);
}

#[test]
fn scripted_direction_setter_accepts_only_the_legacy_eight_way_table() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    for direction in 0..=8 {
        assert!(controller.set_current_direction_key(direction));
        assert_eq!(controller.current_direction_key(), direction);
    }
    assert!(!controller.set_current_direction_key(9));
    assert_eq!(controller.current_direction_key(), 8);
}

#[test]
fn home_toggle_runs_after_manual_cancel_and_affects_the_next_frame() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_auto_run(true);

    assert_eq!(controller.resolve_local_axis(Vec2::ZERO), Vec2::Y);
    assert!(controller.is_auto_running());

    // Manual A cancels the already active run for this frame.
    assert_eq!(controller.resolve_local_axis(Vec2::NEG_X), Vec2::NEG_X);
    assert!(!controller.is_auto_running());
    // Home is handled near the end of ForceUpdate, re-enabling it only for
    // the following frame.
    controller.finish_input_frame(true);
    assert!(controller.is_auto_running());
    assert_eq!(controller.resolve_local_axis(Vec2::ZERO), Vec2::Y);

    // Toggling off does not retract the forward axis already selected for
    // the current frame.
    controller.finish_input_frame(true);
    assert!(!controller.is_auto_running());
}

#[test]
fn authoritative_teleport_clears_pre_regen_motion_without_echoing_old_position() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.velocity = Vec3::new(3.0, 4.0, 5.0);
    controller.grounded = true;
    controller.jumping = true;
    controller.auto_run = true;
    controller.vertical_velocity = 5.68;
    controller.jump_packet_velocity = 5.68;
    controller.packet_elapsed = 0.75;
    controller.current_direction_key = 1;
    controller.last_direction_key = 1;
    controller.last_packet_position = Vec3::splat(-10.0);
    let destination = Vec3::new(373.33, -5.7, 442.6);

    controller.apply_authoritative_teleport(destination);

    assert_eq!(controller.velocity, Vec3::ZERO);
    assert!(!controller.grounded);
    assert!(!controller.jumping);
    assert!(!controller.auto_run);
    assert_eq!(controller.vertical_velocity, 0.0);
    assert_eq!(controller.jump_packet_velocity, 0.0);
    assert_eq!(controller.packet_elapsed, 0.0);
    assert_eq!(controller.current_direction_key, 0);
    assert_eq!(controller.last_direction_key, 0);
    assert_eq!(controller.last_packet_position, destination);
}

#[test]
fn free_camera_and_alt_arrows_keep_distinct_legacy_semantics() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyQ);
    assert_eq!(camera_turn_keys(&keys), (true, false));
    assert!(!arrow_camera_recenter(&keys));

    keys.release(KeyCode::KeyQ);
    keys.press(KeyCode::KeyE);
    assert_eq!(camera_turn_keys(&keys), (false, true));
    assert!(!arrow_camera_recenter(&keys));

    keys.release(KeyCode::KeyE);
    keys.press(KeyCode::ArrowLeft);
    assert_eq!(camera_turn_keys(&keys), (true, false));
    assert!(arrow_camera_recenter(&keys));

    keys.press(KeyCode::ControlLeft);
    assert!(free_camera_from_keys(&keys));
    keys.release(KeyCode::ControlLeft);
    keys.press(KeyCode::ControlRight);
    assert!(free_camera_from_keys(&keys));

    assert!(!should_apply_camera_yaw(true, false, true, false));
    assert!(!should_apply_camera_yaw(false, true, true, false));
    // Arrow alt-mapping calls SetForceAngle before the FreeCamera guard.
    assert!(should_apply_camera_yaw(false, false, true, true));
    assert!(should_apply_camera_yaw(true, false, false, false));
}

#[test]
fn camera_and_player_heading_share_the_same_reflected_yaw_sign() {
    for yaw in [-180.0, -90.0, 0.0, 90.0, 180.0] {
        let heading = LegacyUnityHeadingDegrees::new(yaw);
        assert!(
            legacy_camera_forward_to_native(yaw, 0.0)
                .abs_diff_eq(heading.native_forward(), 0.000_01)
        );
    }

    let yaw = 90.0_f32;
    let pitch = 30.0_f32;
    let expected = unity_to_native_vector(Vec3::new(
        yaw.to_radians().sin() * pitch.to_radians().cos(),
        -pitch.to_radians().sin(),
        yaw.to_radians().cos() * pitch.to_radians().cos(),
    ));
    assert!(legacy_camera_forward_to_native(yaw, pitch).abs_diff_eq(expected, 0.000_01));
}

#[test]
fn camera_pose_uses_the_exact_legacy_slerp_factor_and_arrow_snap() {
    let from = legacy_camera_rotation_to_native(0.0, 0.0);
    let desired = legacy_camera_rotation_to_native(90.0, 0.0);
    let mut camera = LegacyOrbitCamera::new(Entity::PLACEHOLDER);
    camera.configurable_sensitivity = 0.0;

    let factor = (0.5 + camera.configurable_sensitivity * 0.1).clamp(0.0, 1.0);
    let smoothed = from.slerp(desired, factor);
    assert!(
        (from.angle_between(smoothed) - from.angle_between(desired) * 0.5).abs() < 0.000_01
    );

    camera.force_player_angle_this_frame = true;
    let snapped = if camera.force_player_angle_this_frame {
        desired
    } else {
        from.slerp(desired, factor)
    };
    assert!(snapped.abs_diff_eq(desired, f32::EPSILON));
}

#[test]
fn window_mouse_y_is_adapted_to_unitys_up_positive_axis() {
    assert_eq!(
        bevy_mouse_delta_to_unity(Vec2::new(3.0, -4.0)),
        Vec2::new(3.0, 4.0)
    );
    assert_eq!(
        bevy_mouse_delta_to_unity(Vec2::new(-2.0, 5.0)),
        Vec2::new(-2.0, -5.0)
    );
}

#[test]
fn external_transport_sends_idle_position_at_packet_interval() {
    let owner = Entity::from_bits(123);
    let mut controller = LegacyPlayerController::from_baseline_table();
    let mut intents = MovementIntentQueue::default();
    let position = Vec3::new(40.0, 5.0, -12.0);
    controller.record_external_transport_motion();
    controller.reconcile_external_collision_position(owner, position, Some(&mut intents));
    assert!(intents.is_empty(), "carry must respect the packet interval");
    // The carry must survive even if the platform stopped before this tick.
    controller.packet_position_sampled_this_frame = true;
    controller.reconcile_external_collision_position(owner, position, Some(&mut intents));
    let Some(MovementIntent::Stop(packet)) = intents.pop_front() else {
        panic!("idle rider must publish its carried position");
    };
    assert_eq!(
        packet.position,
        ProtocolPosition::from_native(position).raw()
    );
    controller.movement_intent_emitted_this_frame = false;
    controller.reconcile_external_collision_position(owner, position, Some(&mut intents));
    assert!(
        intents.is_empty(),
        "stationary support must not send another packet"
    );
    controller.record_external_transport_motion();
    controller.apply_authoritative_teleport(Vec3::ZERO);
    controller.packet_position_sampled_this_frame = true;
    controller.reconcile_external_collision_position(owner, Vec3::ZERO, Some(&mut intents));
    assert!(intents.is_empty(), "teleport must discard pending carry");
}

#[test]
fn nano_dash_moves_without_input_then_restores_gravity_and_stops() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0/60.0)))
        .init_resource::<LegacyInputState>().init_resource::<MovementIntentQueue>()
        .add_systems(Update, simulate_legacy_players);
    app.update();
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.grounded = false;
    controller.launch_nano_dash();
    let player = app.world_mut().spawn((Transform::from_xyz(0.0,20.0,0.0), controller)).id();
    for _ in 0..34 { app.update(); }
    let position = app.world().get::<Transform>(player).unwrap().translation;
    assert!(position.z > 9.5 && position.z < 11.0, "{position:?}");
    assert!((position.y-20.0).abs()<0.001);
    for _ in 0..15 { app.update(); }
    let later = app.world().get::<Transform>(player).unwrap().translation;
    assert!((later.z-position.z).abs()<0.001);
    assert!(later.y < position.y-0.1);
    assert!(!app.world().resource::<MovementIntentQueue>().is_empty());
}

#[test]
fn incapacitation_blocks_input_and_auto_run_but_preserves_falling() {
    let mut app=App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0/60.0)))
        .insert_resource(LegacyInputState { local_axis: Vec2::Y, jump_just_pressed: true, ..default() })
        .init_resource::<MovementIntentQueue>().add_systems(Update,simulate_legacy_players);
    app.update();
    let mut controller=LegacyPlayerController::from_baseline_table();
    controller.incapacitated=true; controller.auto_run=true; controller.grounded=false;
    let player=app.world_mut().spawn((Transform::from_xyz(0.0,20.0,0.0),controller)).id();
    for _ in 0..30 { app.update(); }
    let p=app.world().get::<Transform>(player).unwrap().translation;
    assert!(p.x.abs()<0.001 && p.z.abs()<0.001);
    assert!(p.y<19.0);
    app.world_mut().get_mut::<LegacyPlayerController>(player).unwrap().incapacitated=false;
    app.update();
    assert!(app.world().get::<Transform>(player).unwrap().translation.z>0.09);
}

#[test]
fn mouse_camera_axes_match_configurable_input_clamping() {
    assert!(
        legacy_mouse_camera_axes(Vec2::new(10.0, -20.0), 0.03)
            .abs_diff_eq(Vec2::new(0.3, 0.6), f32::EPSILON)
    );
    assert_eq!(
        legacy_mouse_camera_axes(Vec2::new(100.0, -100.0), 0.03),
        Vec2::ONE
    );
    assert_eq!(
        legacy_mouse_camera_axes(Vec2::new(-100.0, 100.0), 0.03),
        Vec2::splat(-1.0)
    );
}

#[test]
fn packet_builders_preserve_axis_angle_scale_and_jump_bits() {
    let position = Vec3::new(1.25, -2.5, 3.75);
    let velocity = Vec3::new(4.0, 6.0, -5.0);
    let movement = make_pc_move_request(position, velocity, 270.9, 2, 650);
    assert_eq!(movement.client_time, 0);
    assert_eq!(movement.position, [-125, 375, -250]);
    assert_eq!(movement.velocity, [-4.0, -5.0, 6.0]);
    assert_eq!(movement.angle, 90);
    assert_eq!(movement.key_value, 2);
    assert_eq!(movement.speed, 650);

    let stop = make_pc_stop_request(position);
    assert_eq!(stop.client_time, 0);
    assert_eq!(stop.position, [-125, 375, -250]);

    let jump = make_pc_jump_request(position, velocity, 90.5, 8, 700, 7.25, 0x4567_89ab);
    assert_eq!(jump.position, [-125, 375, -250]);
    assert_eq!(jump.velocity, [-400, -500, 600]);
    assert_eq!(jump.angle, -90);
    assert_eq!(jump.key_value, 8);
    assert_eq!(jump.speed, 700);
    assert_eq!(jump.client_time, (725_u64 << 32) | 0x4567_89ab);
}

#[test]
fn backward_speed_is_half_and_packet_cadence_is_strict() {
    let mut controller = LegacyPlayerController::from_server_attributes(651, 700);
    controller.packet_elapsed = LEGACY_PACKET_MIN_INTERVAL;
    assert!(!packet_is_due(&controller, 5));
    controller.packet_elapsed += 0.000_1;
    assert!(packet_is_due(&controller, 5));

    let backward_client_speed =
        controller.run_speed_server_units as f32 * SERVER_TO_CLIENT_SCALE * 0.5;
    assert_eq!((backward_client_speed * 100.0) as i32, 325);
}

#[test]
fn grounded_jump_keeps_the_full_launch_velocity_for_its_first_move() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.step_normal_vertical(true, 1.0 / 60.0);

    assert!(controller.grounded);
    assert!(controller.jumping);
    assert_eq!(controller.vertical_velocity, 5.68);
    assert_eq!(controller.jump_packet_velocity, 5.68);

    // The collision result clears ground only after the launch Move.
    controller.set_grounded(false);
    controller.step_normal_vertical(false, 1.0 / 60.0);
    assert!((controller.vertical_velocity - (5.68 - LEGACY_GRAVITY / 60.0)).abs() < 0.000_01);
}

#[test]
fn nano_rocket_replaces_fall_and_preserves_jump_packet_launch_velocity() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    assert!(controller.launch_nano_rocket(8.68));
    controller.step_normal_vertical(false, 1.0 / 60.0);
    assert_eq!(controller.vertical_velocity, 8.68);
    assert_eq!(controller.jump_packet_velocity, 8.68);
    controller.grounded = false;
    let mut rise = 0.0;
    for _ in 0..30 {
        controller.step_normal_vertical(false, 1.0 / 60.0);
        rise += controller.vertical_velocity / 60.0;
    }
    assert!(
        rise > 3.0 && rise < 3.1,
        "rocket ascent must enter ordinary gravity integration: {rise}"
    );
    controller.vertical_velocity = -10.0;
    assert!(controller.launch_nano_rocket(9.58));
    controller.step_normal_vertical(false, 1.0 / 60.0);
    assert!(
        (controller.vertical_velocity - (9.58 - LEGACY_GRAVITY / 60.0)).abs() < 0.0001
    );
    assert_eq!(controller.jump_packet_velocity, 9.58);
    assert!(!controller.launch_nano_rocket(f32::NAN));
    assert!(!controller.launch_nano_rocket(-1.0));
    assert_eq!(controller.jump_packet_velocity, 9.58);
}

#[test]
fn ordinary_steep_contact_replaces_xz_and_doubles_only_the_submitted_vertical_move() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller.vertical_velocity = 4.0;
    controller.set_external_collision_result(
        LEGACY_COLLISION_SIDES,
        Some(Vec3::new(-1.0, 0.0, 1.0).normalize()),
    );

    let submitted = controller.apply_surface_sliding(Vec3::new(6.0, 4.0, 0.0));

    assert!(submitted.abs_diff_eq(
        Vec3::new(-std::f32::consts::SQRT_2, 8.0, std::f32::consts::SQRT_2),
        0.000_01,
    ));
    assert_eq!(
        controller.vertical_velocity, 4.0,
        "Retrobution keeps fVelocityZ undoubled for the following gravity step"
    );
    assert!(controller.surface_sliding());

    controller.set_external_collision_result(LEGACY_COLLISION_BELOW, Some(Vec3::Y));
    let walkable = controller.apply_surface_sliding(Vec3::new(6.0, 4.0, 0.0));
    assert_eq!(walkable, Vec3::new(6.0, 4.0, 0.0));
    assert!(!controller.surface_sliding());
}

#[test]
fn terminal_fall_settles_only_after_two_identical_post_move_heights() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller.start_normal_jump(-LEGACY_GRAVITY);
    controller.surface_sliding = true;

    assert!(!controller.settle_terminal_blocked_fall(12.5));
    assert!(controller.jumping);
    assert!(controller.surface_sliding());

    assert!(controller.settle_terminal_blocked_fall(12.5));
    assert!(controller.grounded);
    assert!(!controller.jumping);
    assert!(!controller.surface_sliding());
    assert_eq!(controller.vertical_velocity, 0.0);
}

#[test]
fn sixty_hz_jump_trajectory_matches_retrobution_force_update_order() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(LegacyInputState {
            jump_just_pressed: true,
            ..default()
        })
        .init_resource::<MovementIntentQueue>()
        .add_systems(Update, simulate_legacy_players);
    // Bevy's first Time update establishes its clock with a zero delta.
    // Warm it before spawning so the player's first simulated frame is
    // the same controlled 1/60 s step used by the Retrobution oracle.
    app.update();
    let player = app
        .world_mut()
        .spawn((
            Transform::default(),
            LegacyPlayerController::from_baseline_table().with_placeholder_ground_plane(0.0),
        ))
        .id();

    app.update();
    {
        let transform = app
            .world()
            .get::<Transform>(player)
            .expect("test player transform");
        let controller = app
            .world()
            .get::<LegacyPlayerController>(player)
            .expect("test player controller");
        assert!(
            (transform.translation.y - (5.68 / 60.0 - LEGACY_CHARACTER_MOVE_BIAS)).abs()
                < 0.000_001
        );
        assert!(controller.last_move_displacement().abs_diff_eq(
            Vec3::new(
                -LEGACY_CHARACTER_MOVE_BIAS,
                5.68 / 60.0 - LEGACY_CHARACTER_MOVE_BIAS,
                -LEGACY_CHARACTER_MOVE_BIAS,
            ),
            f32::EPSILON,
        ));
        assert_eq!(controller.velocity.y, 5.68);
        assert!(controller.jumping);
        assert!(!controller.grounded);
        assert!(app.world().resource::<MovementIntentQueue>().is_empty());
    }

    app.world_mut()
        .resource_mut::<LegacyInputState>()
        .jump_just_pressed = false;
    for frame in 2..=70 {
        app.update();
        let transform = app
            .world()
            .get::<Transform>(player)
            .expect("test player transform");
        let controller = app
            .world()
            .get::<LegacyPlayerController>(player)
            .expect("test player controller");
        match frame {
            35 => {
                assert!((transform.translation.y - 1.660_555_6).abs() < 0.000_1);
                assert!((controller.velocity.y - 0.013_333_3).abs() < 0.000_1);
                assert!(controller.jumping);
                assert!(!controller.grounded);
            }
            36 => {
                assert!((transform.translation.y - 1.658).abs() < 0.000_1);
                assert!((controller.velocity.y + 0.153_333_3).abs() < 0.000_1);
            }
            69 => {
                assert!((transform.translation.y - 0.015_333_3).abs() < 0.000_1);
                assert!(controller.jumping);
                assert!(!controller.grounded);
            }
            70 => {
                assert_eq!(transform.translation.y, 0.0);
                assert_eq!(controller.velocity.y, 0.0);
                assert!(controller.grounded);
                assert!(!controller.jumping);
            }
            _ => {}
        }
    }
}

#[test]
fn unsupported_avatar_keeps_the_legacy_edge_jump_window() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller.vertical_velocity = -1.0;

    controller.step_normal_vertical(true, 1.0 / 60.0);

    assert!(controller.jumping);
    assert!((controller.vertical_velocity - (5.68 - LEGACY_GRAVITY / 60.0)).abs() < 0.000_01);
    assert_eq!(controller.jump_packet_velocity, 5.68);
}

#[test]
fn edge_jump_window_ends_only_after_the_strict_half_gravity_transition() {
    let mut last_valid_frame = LegacyPlayerController::from_baseline_table();
    last_valid_frame.set_grounded(false);
    last_valid_frame.vertical_velocity = -4.99;

    last_valid_frame.step_normal_vertical(false, 0.001);
    assert_eq!(
        last_valid_frame.vertical_velocity,
        LEGACY_FALL_JUMP_VELOCITY
    );
    assert!(
        !last_valid_frame.jumping,
        "equality at -gravity/2 must keep bJumpFlag clear"
    );
    last_valid_frame.step_normal_vertical(true, 1.0 / 60.0);
    assert!(last_valid_frame.jumping);
    assert!(last_valid_frame.vertical_velocity > 5.5);

    let mut one_frame_too_late = LegacyPlayerController::from_baseline_table();
    one_frame_too_late.set_grounded(false);
    one_frame_too_late.vertical_velocity = LEGACY_FALL_JUMP_VELOCITY;
    one_frame_too_late.step_normal_vertical(false, 0.001);
    assert!(one_frame_too_late.jumping);
    let falling_velocity = one_frame_too_late.vertical_velocity;
    one_frame_too_late.step_normal_vertical(true, 1.0 / 60.0);
    assert!(
        one_frame_too_late.vertical_velocity < falling_velocity,
        "once the automatic falling Jump() sets bJumpFlag, Space must not relaunch"
    );
}

#[test]
fn normal_jump_uses_the_legacy_half_second_cooldown_without_buffering_input() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.step_normal_vertical(true, 0.0);
    assert_eq!(
        controller.normal_jump_cooldown_remaining,
        LEGACY_NORMAL_JUMP_COOLDOWN_SECONDS
    );

    controller.land_on_external_collider();
    controller.step_normal_vertical(true, 0.49);
    assert!(
        !controller.jumping,
        "a new press inside GameCondition cooldown type 3 must be rejected"
    );

    controller.step_normal_vertical(false, 0.02);
    assert!(
        !controller.jumping,
        "Retrobution does not buffer the rejected press until the cooldown expires"
    );
    controller.step_normal_vertical(true, 0.0);
    assert!(controller.jumping);
    assert_eq!(controller.vertical_velocity, 5.68);
}

#[test]
fn unsupported_fall_enters_jump_state_only_below_half_gravity() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller.vertical_velocity = -4.0;

    controller.step_normal_vertical(false, 0.1);
    assert_eq!(controller.vertical_velocity, LEGACY_FALL_JUMP_VELOCITY);
    assert!(!controller.jumping, "the source comparison is strict");

    controller.step_normal_vertical(false, 0.001);
    assert!(controller.vertical_velocity < LEGACY_FALL_JUMP_VELOCITY);
    assert!(controller.jumping);
    assert_eq!(
        controller.jump_packet_velocity, 0.0,
        "negative Jump(force) must not replace jumpVelocity"
    );
}

#[test]
fn unsupported_rise_uses_the_exact_positive_jump_threshold() {
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    controller.vertical_velocity = LEGACY_RISE_JUMP_VELOCITY;

    controller.step_normal_vertical(false, 0.0);
    assert!(!controller.jumping, "the source comparison is strict");

    controller.vertical_velocity += 0.001;
    controller.step_normal_vertical(false, 0.0);
    assert!(controller.jumping);
    assert_eq!(
        controller.jump_packet_velocity,
        LEGACY_RISE_JUMP_VELOCITY + 0.001
    );
}

#[test]
fn landing_forces_a_packet_only_for_an_active_jump_state() {
    let mut short_fall = LegacyPlayerController::from_baseline_table();
    short_fall.set_grounded(false);
    short_fall.packet_elapsed = 0.125;
    short_fall.land_on_external_collider();
    assert_eq!(short_fall.packet_elapsed, 0.125);

    let mut jump = LegacyPlayerController::from_baseline_table();
    jump.start_normal_jump(5.68);
    jump.packet_elapsed = 0.125;
    jump.land_on_external_collider();
    assert!(jump.packet_elapsed.is_infinite());
    assert!(jump.grounded);
    assert!(!jump.jumping);
    assert_eq!(jump.vertical_velocity, 0.0);
}

#[test]
fn flight_uses_held_vertical_input_without_gravity_and_emits_move_packets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(LegacyInputState {
            flight_ascend_held: true,
            ..default()
        })
        .init_resource::<MovementIntentQueue>()
        .add_systems(Update, simulate_legacy_players);
    app.update();

    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_flight_enabled(true);
    let player = app
        .world_mut()
        .spawn((Transform::default(), controller))
        .id();
    app.update();

    let transform = app.world().get::<Transform>(player).unwrap();
    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!(controller.flight_enabled());
    assert!(!controller.grounded);
    assert!(!controller.jumping);
    assert!((controller.velocity.y - 6.0).abs() < f32::EPSILON);
    assert!((transform.translation.y - 0.1).abs() < 0.000_01);
    assert!(matches!(
        app.world_mut()
            .resource_mut::<MovementIntentQueue>()
            .pop_front(),
        Some(MovementIntent::Move(request)) if request.velocity[2] == 6.0
    ));

    app.world_mut()
        .resource_mut::<LegacyInputState>()
        .flight_ascend_held = false;
    app.update();
    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert_eq!(
        controller.velocity.y, 0.0,
        "hover must not accumulate gravity"
    );
}
