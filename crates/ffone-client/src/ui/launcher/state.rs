use super::*;

pub const LAUNCHER_UI_GAME_MODE_SLOT: usize = 13;

pub const LAUNCHER_UI_PARITY_STATUS: &str = "partial";

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct LauncherUiExternalState {
    pub current_hp: i32,
    pub system_popup_active: bool,
    pub aim_vertical_axis: f32,
    pub aim_horizontal_axis: f32,
}

impl Default for LauncherUiExternalState {
    fn default() -> Self {
        Self {
            current_hp: 1,
            system_popup_active: false,
            aim_vertical_axis: 0.0,
            aim_horizontal_axis: 0.0,
        }
    }
}
