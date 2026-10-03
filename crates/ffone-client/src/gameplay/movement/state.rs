use super::*;

/// Current digital input after applying the original primary/alternate mapping
/// precedence. Arrow up/down are alternate W/S; arrow left/right turn and
/// recenter the camera rather than strafing.
#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct LegacyInputState {
    pub local_axis: Vec2,
    pub jump_just_pressed: bool,
    /// Native flight extension: Space raises the player while flight is on.
    pub flight_ascend_held: bool,
    /// Native flight extension: either Shift key lowers the player.
    pub flight_descend_held: bool,
    /// `FreeCamera` is Left Control with Right Control as its alternate map.
    pub free_camera_held: bool,
    /// `AutoRun` is Home and toggles at the end of the legacy movement update.
    pub auto_run_just_pressed: bool,
}

/// Committed keyboard camera actions supplied by the production options owner.
/// Standalone movement users can omit this resource and retain legacy keys.
#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct LegacyCameraKeyInput {
    pub turn_left: bool,
    pub turn_right: bool,
    pub recenter: bool,
    pub pad_axis: Vec2,
    pub pad_zoom: f32,
}
