use super::*;

/// Returns the exact 6877 `AvatarUtil.GetDirNum` value for local input.
/// `local_axis.x` is right/left and `local_axis.y` is forward/backward.
#[must_use]
pub fn legacy_direction_key(local_axis: Vec2) -> u8 {
    let x = ((local_axis.x * 100.0) as i32).clamp(-1, 1);
    let forward = ((local_axis.y * 100.0) as i32).clamp(-1, 1);
    LEGACY_DIRECTION_MATRIX[(forward + 1) as usize][(x + 1) as usize]
}

/// Packs the legacy jump launch velocity and random landing key into
/// `sP_CL2FE_REQ_PC_JUMP.iCliTime`.
#[must_use]
pub fn pack_legacy_jump_client_time(jump_launch_velocity: f32, jump_key: u32) -> u64 {
    let velocity = (jump_launch_velocity.max(0.0) * 100.0) as u64;
    (velocity << 32) | u64::from(jump_key)
}

pub(super) fn paired_axis(positive: bool, negative: bool) -> f32 {
    if positive {
        1.0
    } else if negative {
        -1.0
    } else {
        0.0
    }
}

pub(super) fn movement_axes_from_keys(keys: &ButtonInput<KeyCode>, gate: &LegacyInputGate) -> Vec2 {
    // Horizontal has no alternate default. Axis.Vertical is primary W/S plus
    // alternate Up/Down, then clamped. ConfigurableInput stores Down as the
    // pair's `up` entry and flips the finished axis, so S/Down wins when both
    // directions in one mapping are held. Horizontal keeps D-over-A priority.
    let horizontal = if gate.allow_strafe {
        paired_axis(keys.pressed(KeyCode::KeyD), keys.pressed(KeyCode::KeyA))
    } else {
        0.0
    };
    let vertical_primary = -paired_axis(
        gate.allow_backward && keys.pressed(KeyCode::KeyS),
        gate.allow_forward && keys.pressed(KeyCode::KeyW),
    );
    let vertical_alternate = -paired_axis(
        gate.allow_backward && keys.pressed(KeyCode::ArrowDown),
        gate.allow_forward && keys.pressed(KeyCode::ArrowUp),
    );
    Vec2::new(
        horizontal,
        (vertical_primary + vertical_alternate).clamp(-1.0, 1.0),
    )
}

pub(super) fn free_camera_from_keys(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
}

pub(super) fn camera_turn_keys(keys: &ButtonInput<KeyCode>) -> (bool, bool) {
    (
        keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::ArrowLeft),
        keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::ArrowRight),
    )
}

pub(super) fn arrow_camera_recenter(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ArrowUp)
        || keys.pressed(KeyCode::ArrowDown)
        || keys.pressed(KeyCode::ArrowLeft)
        || keys.pressed(KeyCode::ArrowRight)
}

pub(super) fn clamp_legacy_angle(mut angle: f32, minimum: f32, maximum: f32) -> f32 {
    if angle < -360.0 {
        angle += 360.0;
    }
    if angle > 360.0 {
        angle -= 360.0;
    }
    angle.clamp(minimum, maximum)
}

pub(super) fn legacy_mouse_camera_axes(delta: Vec2, axis_per_pixel: f32) -> Vec2 {
    // ConfigurableInput.GetAxisOnly sums the configured input sources and
    // clamps the finished CameraX/CameraY axis before cnPlayerCamera applies
    // its per-axis multiplier and the user's CameraSensitivity. Without this
    // saturation, a fast mouse event can rotate the native camera far beyond
    // what the primary client permits in one frame.
    (bevy_mouse_delta_to_unity(delta) * axis_per_pixel).clamp(Vec2::splat(-1.0), Vec2::ONE)
}

pub fn simulate_legacy_players(
    time: Res<Time>,
    input: Res<LegacyInputState>,
    cameras: Query<&LegacyOrbitCamera>,
    mut players: Query<(Entity, &mut Transform, &mut LegacyPlayerController)>,
    mut intents: ResMut<MovementIntentQueue>,
) {
    let delta_seconds = time.delta_secs();
    if delta_seconds >= LEGACY_MAX_FRAME_DELTA {
        for (_, _, mut controller) in &mut players {
            controller.packet_position_sampled_this_frame = false;
            controller.movement_intent_emitted_this_frame = false;
            // GameCondition uses absolute Time.time, so its cooldown still
            // elapses across a frame that ForceUpdate itself discards.
            controller.advance_normal_jump_cooldown(delta_seconds);
            controller.last_move_displacement = Vec3::ZERO;
        }
        return;
    }

    for (entity, mut transform, mut controller) in &mut players {
        controller.packet_position_sampled_this_frame = false;
        controller.movement_intent_emitted_this_frame = false;
        if !controller.movement_enabled {
            controller.nano_dash_remaining = 0.0;
            controller.advance_normal_jump_cooldown(delta_seconds);
            controller.velocity = Vec3::ZERO;
            controller.last_move_displacement = Vec3::ZERO;
            controller.current_direction_key = 0;
            continue;
        }

        let camera = cameras.iter().find(|camera| camera.target == entity);
        let camera_yaw = camera.map_or(controller.yaw_degrees, |camera| camera.yaw_degrees);
        let force_camera_yaw = camera.is_some_and(|camera| camera.force_player_angle_this_frame);
        if controller.incapacitated {
            controller.nano_dash_remaining = 0.0;
        }
        let dashing = controller.nano_dash_remaining > 0.0;
        let scripted_horizontal = if dashing {
            let forward = LegacyUnityHeadingDegrees::new(camera_yaw).native_root_rotation()
                * Vec3::NEG_Z * controller.nano_dash_speed;
            controller.yaw_degrees = camera_yaw;
            controller.nano_dash_remaining -= delta_seconds;
            controller.nano_dash_speed = (controller.nano_dash_speed - 60.0 * delta_seconds).max(0.0);
            Some(Vec2::new(forward.x, forward.z))
        } else {
            controller.scripted_horizontal_velocity
        };
        let local_axis = if scripted_horizontal.is_some() || controller.incapacitated {
            Vec2::ZERO
        } else {
            controller.resolve_local_axis(input.local_axis)
        };
        let direction_key = if scripted_horizontal.is_some() {
            0
        } else {
            legacy_direction_key(local_axis)
        };
        controller.current_direction_key = direction_key;
        let has_horizontal_input = local_axis != Vec2::ZERO;

        if dashing {
            controller.vertical_velocity = 0.0;
            controller.advance_normal_jump_cooldown(delta_seconds);
        } else if controller.flight_enabled {
            controller.advance_normal_jump_cooldown(delta_seconds);
            controller.grounded = false;
            controller.jumping = false;
            controller.vertical_velocity =
                paired_axis(input.flight_ascend_held, input.flight_descend_held)
                    * controller.run_speed_server_units as f32
                    * SERVER_TO_CLIENT_SCALE;
        } else {
            if controller.grounded {
                controller.advance_jump_key();
            }
            let jump_just_pressed = input.jump_just_pressed && !controller.incapacitated
                && !controller.launcher_active();
            controller.step_normal_vertical(jump_just_pressed, delta_seconds);
        }

        // `cnPlayerCamera.PositionUpdate` runs at the end of ForceUpdate, after
        // CharacterController.Move and MovePacket. Preserve the current player
        // yaw for this frame's displacement/packet, then apply the newly read
        // camera yaw for the next frame. Jump() sets bMoveFlag even with no
        // horizontal input; arrow alt-bindings call SetForceAngle.
        let movement_yaw_degrees = controller.yaw_degrees;
        let apply_camera_yaw_after_move = !controller.launcher_active() && should_apply_camera_yaw(
            has_horizontal_input,
            controller.jumping,
            input.free_camera_held,
            force_camera_yaw,
        );

        let mut speed = controller.run_speed_server_units as f32 * SERVER_TO_CLIENT_SCALE;
        if local_axis.y < 0.0 {
            speed *= 0.5;
        }
        // Legacy input is +X right / +Z forward. After H, the Bevy gameplay
        // root uses local +X right / -Z forward and its reflected root yaw.
        let local_velocity =
            Vec3::new(local_axis.x, 0.0, -local_axis.y).normalize_or_zero() * speed;
        let mut horizontal_velocity = scripted_horizontal.map_or_else(
            || {
                LegacyUnityHeadingDegrees::new(movement_yaw_degrees).native_root_rotation()
                    * local_velocity
            },
            |scripted| Vec3::new(scripted.x, 0.0, scripted.y),
        );
        if controller.vehicle_speed.is_some() && scripted_horizontal.is_none() && !controller.flight_enabled && !controller.incapacitated {
            horizontal_velocity = controller.step_vehicle_horizontal(local_axis, movement_yaw_degrees, delta_seconds);
        }

        let requested_velocity = Vec3::new(
            horizontal_velocity.x,
            controller.vertical_velocity,
            horizontal_velocity.z,
        );
        controller.velocity = if controller.flight_enabled || controller.launcher_active() {
            requested_velocity
        } else {
            controller.apply_surface_sliding(requested_velocity)
        };
        let frame_velocity = controller.velocity;
        // Exact primary call:
        // controller.Move(kMovement * Time.deltaTime - Vector3.one * 1E-06f)
        // Flight is a native extension and deliberately remains unbiased so a
        // zero-input hover cannot acquire source-only horizontal drift.
        controller.last_move_displacement = if controller.flight_enabled {
            frame_velocity * delta_seconds
        } else {
            frame_velocity * delta_seconds - Vec3::splat(LEGACY_CHARACTER_MOVE_BIAS)
        };
        transform.translation += controller.last_move_displacement;

        // The packet is produced before the legacy controller consumes its
        // post-Move `Below` collision flag. Keep `jumping` true through packet
        // construction on the landing frame for the same ordering.
        let mut landed_on_placeholder = false;
        if let LegacyCollisionMode::PlaceholderGroundPlane { height } = controller.collision
            && transform.translation.y <= height
            && controller.vertical_velocity <= 0.0
        {
            transform.translation.y = height;
            landed_on_placeholder = controller.jumping || !controller.grounded;
        }
        if matches!(
            controller.collision,
            LegacyCollisionMode::PlaceholderGroundPlane { .. }
        ) && controller.vertical_velocity > 0.0
        {
            // The external authored-world collision pass performs the same
            // post-Move transition for real colliders.
            controller.grounded = false;
        }

        controller.packet_elapsed += delta_seconds;
        if !controller.launcher_active() && packet_is_due(&controller, direction_key) {
            controller.packet_position_sampled_this_frame = true;
            let speed_server_units = (if controller.vehicle_speed.is_some() { frame_velocity.length() } else { speed } * 100.0) as i32;
            let intent = if controller.jumping {
                let request = make_pc_jump_request(
                    transform.translation,
                    frame_velocity,
                    movement_yaw_degrees,
                    direction_key,
                    controller.jump_height_server_units,
                    controller.jump_packet_velocity,
                    controller.jump_key,
                );
                // This odd decrement is in the original MovePacket method.
                controller.jump_packet_velocity =
                    (controller.jump_packet_velocity - 100.0).max(0.0);
                Some(MovementIntent::Jump(request))
            } else if (controller.flight_enabled && frame_velocity != Vec3::ZERO)
                || (controller.vehicle_speed.is_some() && horizontal_velocity.length_squared() > 0.0) {
                Some(MovementIntent::Move(make_pc_move_request(
                    transform.translation,
                    frame_velocity,
                    movement_yaw_degrees,
                    direction_key,
                    speed_server_units,
                )))
            } else if direction_key == 0 {
                if controller.last_direction_key != 0
                    || controller.vehicle_stop_pending
                    || (transform.translation.y - controller.last_packet_position.y).abs() > 1.0
                {
                    Some(MovementIntent::Stop(make_pc_stop_request(
                        transform.translation,
                    )))
                } else {
                    None
                }
            } else {
                Some(MovementIntent::Move(make_pc_move_request(
                    transform.translation,
                    frame_velocity,
                    movement_yaw_degrees,
                    direction_key,
                    speed_server_units,
                )))
            };

            if let Some(intent) = intent {
                if matches!(intent, MovementIntent::Stop(_)) { controller.vehicle_stop_pending = false; }
                controller.movement_intent_emitted_this_frame = true;
                intents.push(entity, intent);
            }
            // MovePacket updates these even when an idle STOP was unnecessary.
            controller.packet_elapsed = 0.0;
            controller.last_direction_key = direction_key;
            controller.last_packet_yaw = movement_yaw_degrees;
            controller.last_packet_position = transform.translation;
        }

        if landed_on_placeholder {
            controller.land_on_external_collider();
        }

        // Home is tested near the end of cnAvatarThirdPersonMove.ForceUpdate,
        // after the current frame has already resolved/cancelled auto-run.
        controller.finish_input_frame(input.auto_run_just_pressed);

        if apply_camera_yaw_after_move {
            controller.yaw_degrees = camera_yaw;
            transform.rotation =
                LegacyUnityHeadingDegrees::new(controller.yaw_degrees).native_root_rotation();
        }
    }
}

pub(super) fn legacy_camera_rotation_to_native(yaw_degrees: f32, pitch_degrees: f32) -> Quat {
    Transform::IDENTITY
        .looking_to(
            legacy_camera_forward_to_native(yaw_degrees, pitch_degrees),
            Vec3::Y,
        )
        .rotation
}

/// `H * (Unity Quaternion.Euler(pitch, yaw, 0) * Vector3.forward)`.
///
/// Camera yaw remains a legacy Unity heading; only its world result is
/// reflected. This preserves the original mouse/key input signs.
pub(super) fn legacy_camera_forward_to_native(yaw_degrees: f32, pitch_degrees: f32) -> Vec3 {
    let yaw = yaw_degrees.to_radians();
    let pitch = pitch_degrees.to_radians();
    unity_to_native_vector(Vec3::new(
        yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ))
}
