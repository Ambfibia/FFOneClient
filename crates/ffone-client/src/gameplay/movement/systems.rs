use super::*;

/// Advances the dependency-free xorshift32 stream used when the original
/// Unity RNG state is unavailable. Callers must supply a non-zero state; this
/// preserves a continuous native stream but does not claim Unity bit parity.
#[must_use]
pub fn advance_native_xorshift32(state: &mut u32) -> u32 {
    debug_assert_ne!(*state, 0, "xorshift32 requires a non-zero state");
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 17;
    value ^= value << 5;
    *state = value;
    value
}

pub(super) fn should_apply_camera_yaw(
    has_horizontal_input: bool,
    jumping: bool,
    free_camera_held: bool,
    force_camera_yaw: bool,
) -> bool {
    // SetForceAngle (the alternate arrow path) runs before the old camera's
    // FreeCamera check and therefore remains authoritative while Ctrl is held.
    force_camera_yaw || (!free_camera_held && (has_horizontal_input || jumping))
}

pub fn update_legacy_camera_input(
    time: Res<Time>,
    gate: Res<LegacyInputGate>,
    configured_keys: Option<Res<LegacyCameraKeyInput>>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse_motion: Option<Res<AccumulatedMouseMotion>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
) {
    let delta_seconds = time.delta_secs();
    let mouse_delta = if gate.allow_mouse_camera {
        mouse_motion.map_or(Vec2::ZERO, |motion| motion.delta)
    } else {
        Vec2::ZERO
    };
    let scroll_delta = if gate.allow_mouse_camera {
        mouse_scroll.map_or(0.0, |scroll| scroll.delta.y)
    } else {
        0.0
    };

    for mut camera in &mut cameras {
        camera.force_player_angle_this_frame = false;

        // cnPlayerCamera: Mouse X * 0.4 * CameraSensitivity * fMouseSensitivity
        // and Mouse Y * 0.2 with the opposite pitch sign.
        // Window cursor Y grows downwards; Unity's legacy Mouse Y grows
        // upwards. Adapt that screen axis before applying the original
        // cnPlayerCamera subtraction formula below.
        let unity_mouse = legacy_mouse_camera_axes(mouse_delta, camera.mouse_axis_per_pixel);
        camera.yaw_degrees +=
            unity_mouse.x * 0.4 * camera.configurable_sensitivity * camera.mouse_sensitivity;
        camera.pitch_degrees -=
            unity_mouse.y * 0.2 * camera.configurable_sensitivity * camera.mouse_sensitivity;
        if gate.allow_mouse_camera && let Some(configured) = configured_keys.as_ref() {
            camera.yaw_degrees += configured.pad_axis.x * delta_seconds * 90.0;
            camera.pitch_degrees -= configured.pad_axis.y * delta_seconds * 60.0;
            camera.distance += configured.pad_zoom * delta_seconds * camera.mouse_sensitivity;
        }

        if let Some(keys) = keyboard.as_ref() {
            let (turn_left, turn_right) = if gate.allow_keyboard_turning {
                configured_keys.as_ref().map_or_else(|| camera_turn_keys(keys),
                    |configured| (configured.turn_left, configured.turn_right))
            } else {
                (false, false)
            };
            if turn_left {
                camera.yaw_degrees -= delta_seconds * 20.0 * camera.configurable_sensitivity;
            }
            if turn_right {
                camera.yaw_degrees += delta_seconds * 20.0 * camera.configurable_sensitivity;
            }

            let zoom_in = gate.allow_mouse_camera && keys.pressed(KeyCode::Equal);
            let zoom_out = gate.allow_mouse_camera && keys.pressed(KeyCode::Minus);
            let key_zoom = (if zoom_in { -1.0 } else { 0.0 } + if zoom_out { 1.0 } else { 0.0 })
                * camera.mouse_sensitivity
                * 0.01;
            if key_zoom != 0.0 {
                camera.distance += key_zoom;
            } else {
                let axis = scroll_delta * camera.scroll_axis_per_line;
                camera.distance += if (-0.9..=0.9).contains(&axis) {
                    axis * camera.mouse_sensitivity
                } else {
                    axis * camera.mouse_sensitivity * 0.01
                };
            }

            // The arrow bindings are `altMapping`, which triggers the old
            // camera's half-step recenter and SetForceAngle behavior.
            if gate.allow_keyboard_turning && configured_keys.as_ref().map_or_else(
                || arrow_camera_recenter(keys), |configured| configured.recenter,
            ) {
                camera.distance = (camera.default_distance + camera.distance) * 0.5;
                camera.pitch_degrees = (camera.default_pitch_degrees + camera.pitch_degrees) * 0.5;
                camera.force_player_angle_this_frame = true;
            }
        } else {
            let axis = scroll_delta * camera.scroll_axis_per_line;
            camera.distance += axis * camera.mouse_sensitivity;
        }

        camera.pitch_degrees = clamp_legacy_angle(
            camera.pitch_degrees,
            camera.minimum_pitch_degrees,
            camera.maximum_pitch_degrees,
        );
        camera.yaw_degrees = clamp_legacy_angle(camera.yaw_degrees, -360.0, 360.0);
        camera.distance = camera
            .distance
            .clamp(camera.minimum_distance, camera.maximum_distance);
    }
}
