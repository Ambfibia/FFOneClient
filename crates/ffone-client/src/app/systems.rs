use super::*;

/// Startup already installs the matching scale before render targets exist.
/// At runtime, update only when the option itself toggles. Recomputing from a
/// one-pixel OS clamp can resize color and depth targets on different frames.
pub(super) fn native_ui_scale_factor_update(
    previous_enabled: Option<bool>,
    physical_size: UVec2,
    enabled: bool,
) -> Option<Option<f32>> {
    match previous_enabled {
        None if enabled => None,
        None => Some(None),
        Some(previous) if previous == enabled => None,
        Some(_) => Some(native_ui_window_scale_factor(physical_size, enabled)),
    }
}

pub(super) fn sync_native_ui_window_scale(
    runtime: Res<OptionProductionRuntime>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut last_enabled: Local<Option<bool>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let enabled = runtime.options.display.scale_ui;
    let update =
        native_ui_scale_factor_update(*last_enabled, window.resolution.physical_size(), enabled);
    *last_enabled = Some(enabled);
    if let Some(target) = update {
        window.resolution.set_scale_factor_override(target);
    }
}
