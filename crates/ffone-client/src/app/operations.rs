use super::*;

/// Clean action 22 default. WorldMap intentionally has no configurable-input
/// indirection until an independently proven owner for that binding exists.

#[cfg(windows)]
pub(super) fn centered_outer_window_position(
    area_position: (i32, i32),
    area_size: (u32, u32),
    outer_size: (u32, u32),
) -> (i32, i32) {
    let center_axis = |origin: i32, monitor: u32, outer: u32| {
        (i64::from(origin) + (i64::from(monitor) - i64::from(outer)) / 2)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    };
    (
        center_axis(area_position.0, area_size.0, outer_size.0),
        center_axis(area_position.1, area_size.1, outer_size.1),
    )
}

#[cfg(windows)]
pub(super) fn primary_monitor_work_area(
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
) -> Option<((i32, i32), (u32, u32))> {
    let center_axis = |origin: i32, extent: u32| {
        (i64::from(origin) + i64::from(extent) / 2).clamp(i64::from(i32::MIN), i64::from(i32::MAX))
            as i32
    };
    let point = POINT {
        x: center_axis(monitor_position.x, monitor_size.width),
        y: center_axis(monitor_position.y, monitor_size.height),
    };
    let monitor = HMONITOR::MonitorFromPoint(point, co::MONITOR::DEFAULTTONEAREST);
    let info = monitor.GetMonitorInfo().ok()?;
    let width = u32::try_from(info.rcWork.right - info.rcWork.left).ok()?;
    let height = u32::try_from(info.rcWork.bottom - info.rcWork.top).ok()?;
    (width > 0 && height > 0).then_some(((info.rcWork.left, info.rcWork.top), (width, height)))
}

#[cfg(windows)]
pub(super) fn set_primary_window_icon(
    mut primary_window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    settings: Res<UserSettingsPersistence>,
    _non_send_marker: NonSendMarker,
) {
    let (entity, window) = &mut *primary_window;
    let icon_image = image::load_from_memory_with_format(
        include_bytes!("../../assets/fusionfall.ico"),
        image::ImageFormat::Ico,
    )
    .expect("embedded FusionFall window icon must be a valid ICO file")
    .into_rgba8();
    let (width, height) = icon_image.dimensions();
    let icon = Icon::from_rgba(icon_image.into_raw(), width, height)
        .expect("embedded FusionFall window icon must have valid RGBA dimensions");

    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(winit_window) = windows.get_window(*entity) {
            winit_window.set_window_icon(Some(icon));
            let inner_size = winit_window.inner_size();
            window
                .resolution
                .set_physical_resolution(inner_size.width, inner_size.height);

            if matches!(window.mode, WindowMode::Windowed)
                && let Some(monitor) = winit_window
                    .primary_monitor()
                    .or_else(|| winit_window.current_monitor())
            {
                let monitor_position = monitor.position();
                let monitor_size = monitor.size();
                let outer_size = winit_window.outer_size();
                // `MonitorHandle::size` includes the taskbar. Centering inside
                // Win32 `rcWork` keeps the decorated window visually centered
                // in the actually usable desktop area.
                let (area_position, area_size) =
                    primary_monitor_work_area(monitor_position, monitor_size).unwrap_or((
                        (monitor_position.x, monitor_position.y),
                        (monitor_size.width, monitor_size.height),
                    ));
                let (x, y) = centered_outer_window_position(
                    area_position,
                    area_size,
                    (outer_size.width, outer_size.height),
                );
                winit_window.set_outer_position(PhysicalPosition::new(x, y));
            }
            if matches!(window.mode, WindowMode::Windowed)
                && settings.last_observed().window_maximized
            {
                window.set_maximized(true);
            }
        }
    });
}

/// Releases window swap-chain views retained by cameras that stopped being
/// extracted.
///
/// `extract_cameras` strips `ExtractedCamera`/`ExtractedView` from a render
/// entity the moment `Camera::is_active` turns false, but it leaves
/// `ViewTarget` behind. That component still owns the `out_texture` view of
/// the swap-chain image acquired on the camera's last rendered frame, and
/// Bevy's own `cleanup_view_targets_for_resize` only inspects entities that
/// still carry an `ExtractedCamera`, so the stale view outlives the next
/// surface reconfiguration. D3D12 then fails `IDXGISwapChain::ResizeBuffers`
/// with `DXGI_ERROR_INVALID_CALL` because a back buffer is still referenced,
/// wgpu reports `window is in use` and invalidates the surface, and Bevy's
/// default render error handler quits the client.
///
/// The client hands the window-target UI cameras over on every
/// `NativeUiStartupPhase` transition, so the startup maximize and every later
/// resize or present-mode change ran into exactly that. `prepare_view_targets`
/// re-inserts `ViewTarget` in the same frame for every camera that is still
/// extracted, so dropping it here is not observable in a rendered frame.
pub(super) fn release_unextracted_view_targets(
    mut commands: Commands,
    stale: Query<Entity, (With<ViewTarget>, Without<ExtractedCamera>)>,
) {
    for entity in &stale {
        commands.entity(entity).remove::<ViewTarget>();
    }
}

pub(super) fn enforce_primary_good_msaa(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera>, Added<Camera>)>,
) {
    for camera in &cameras {
        commands.entity(camera).insert(Msaa::Off);
    }
}

pub(super) fn native_ui_window_scale_factor(physical_size: UVec2, enabled: bool) -> Option<f32> {
    if !enabled || physical_size.x == 0 || physical_size.y == 0 {
        return None;
    }
    let clean_scale = gameplay_ui_scale(physical_size.y as f32);
    let fit_scale = (physical_size.x as f32 / NATIVE_UI_REFERENCE_WIDTH)
        .min(physical_size.y as f32 / NATIVE_UI_REFERENCE_HEIGHT);
    // Preserve the clean height-driven scale wherever the reference surface
    // fits. Narrow or undersized windows cap it instead of cropping the UI.
    let scale = clean_scale.min(fit_scale);
    scale
        .is_finite()
        .then_some(scale)
        .filter(|scale| *scale > 0.0)
}

#[cfg(windows)]
pub(super) fn open_external_url(url: &str) -> Result<(), String> {
    Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
pub(super) fn open_external_url(url: &str) -> Result<(), String> {
    Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
pub(super) fn open_external_url(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(super) fn shared_gameplay_world_active(state: Res<State<ClientState>>) -> bool {
    client_state_sends_movement_intents(*state.get())
}

pub(super) fn world_nano_authority_active(state: Res<State<ClientState>>) -> bool {
    client_state_uses_world_nano_authority(*state.get())
}
