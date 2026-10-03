use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LauncherUiInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub fire_enabled: bool,
    pub escape_enabled: bool,
}

pub(super) fn handle_launcher_keyboard(
    keyboard: Option<MessageReader<KeyboardInput>>,
    time: Res<Time>,
    external: Res<LauncherUiExternalState>,
    mut model: ResMut<LauncherUiModel>,
    mut outbox: ResMut<LauncherUiOutbox>,
) {
    let mut fire_pressed = false;
    let mut fire_released = false;
    let mut escape_pressed = false;
    if let Some(mut keyboard) = keyboard {
        for key in keyboard.read() {
            match (key.key_code, key.state) {
                (KeyCode::Space, ButtonState::Pressed) if !key.repeat => fire_pressed = true,
                (KeyCode::Space, ButtonState::Released) => fire_released = true,
                (KeyCode::Escape, ButtonState::Pressed) if !key.repeat => escape_pressed = true,
                _ => {}
            }
        }
    }
    model.update(
        LauncherUiFrameInput {
            delta_seconds: time.delta_secs(),
            fire_pressed,
            fire_released,
            escape_pressed,
            current_hp: external.current_hp,
            system_popup_active: external.system_popup_active,
        },
        &mut outbox,
    );
}
