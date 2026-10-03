use super::*;

/// Reproduces the clean `OnGUI` ordering: the current frame is drawn from the
/// value already stored in `bgTexture2`, then the ready CharacterCreation
/// texture is assigned for the following GUI frame. The serialized LoginMode
/// object is inactive outside this UI, so a hidden native root must not consume
/// that first fallback frame in advance.
pub(super) fn login_background_frame(
    surface: LoginSurface,
    visible: bool,
    assigned_before_draw: bool,
    background_asset_loaded: bool,
) -> (LoginBackgroundMode0104, bool) {
    let mode = login_background_mode(surface, assigned_before_draw);
    let assigned_after_draw = assigned_before_draw
        || (visible && background_asset_loaded && login_can_assign_loaded_background(surface));
    (mode, assigned_after_draw)
}
