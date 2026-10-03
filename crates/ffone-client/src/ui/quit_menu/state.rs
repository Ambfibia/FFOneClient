use super::*;

pub const QUIT_MENU_PARITY_STATUS: &str = "partial";

#[derive(Clone, Copy, Debug, Default, Resource)]
pub(super) struct QuitMenuPresentationState {
    pub(super) visible: bool,
}
