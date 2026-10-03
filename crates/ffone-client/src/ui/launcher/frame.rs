
pub const LAUNCHER_REQUEST_PACKET_ID: u32 = 318_767_166;

pub const LAUNCHER_REQUEST_PACKET_SIZE: usize = 40;

pub const LAUNCHER_BROADCAST_PACKET_ID: u32 = 822_083_702;

pub const LAUNCHER_BROADCAST_PACKET_SIZE: usize = 52;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LauncherUiFrameInput {
    pub delta_seconds: f32,
    pub fire_pressed: bool,
    pub fire_released: bool,
    pub escape_pressed: bool,
    pub current_hp: i32,
    pub system_popup_active: bool,
}

impl Default for LauncherUiFrameInput {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            fire_pressed: false,
            fire_released: false,
            escape_pressed: false,
            current_hp: 1,
            system_popup_active: false,
        }
    }
}
