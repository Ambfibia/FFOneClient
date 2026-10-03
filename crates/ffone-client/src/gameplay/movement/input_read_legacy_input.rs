use super::*;

pub fn read_legacy_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    gate: Res<LegacyInputGate>,
    mut input: ResMut<LegacyInputState>,
) {
    let Some(keyboard) = keyboard else {
        *input = LegacyInputState::default();
        return;
    };
    input.local_axis = movement_axes_from_keys(&keyboard, &gate);
    input.jump_just_pressed = gate.allow_jump && keyboard.just_pressed(KeyCode::Space);
    input.flight_ascend_held = gate.allow_jump && keyboard.pressed(KeyCode::Space);
    input.flight_descend_held = gate.allow_jump
        && (keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight));
    input.free_camera_held = free_camera_from_keys(&keyboard);
    // Auto-run resolves to the forward axis, so it must not be possible to
    // enable it while that direction is locked.
    input.auto_run_just_pressed = gate.allow_forward && keyboard.just_pressed(KeyCode::Home);
}
