use super::*;

pub(super) fn update_orbit_camera(
    state: Res<EditorState>,
    mut motions: MessageReader<MouseMotion>,
    mut wheels: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut orbit: ResMut<OrbitCamera>,
    mut camera: Single<&mut Transform, With<PreviewCamera>>,
    icons: Res<icon_generator::IconGenerator>,
) {
    let over_viewport = !icons.active && !state.strings_open
        && !state.xdt_open
        && window.cursor_position().is_some_and(|cursor| {
            preview_viewport_logical_rect(Vec2::new(window.width(), window.height())).is_some_and(
                |(position, size)| {
                    cursor.x >= position.x
                        && cursor.x <= position.x + size.x
                        && cursor.y >= position.y
                        && cursor.y <= position.y + size.y
                },
            )
        });
    if over_viewport && (mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right)) {
        for motion in motions.read() {
            orbit.yaw -= motion.delta.x * 0.007;
            orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(-0.45, 1.05);
        }
    } else {
        motions.clear();
    }
    if over_viewport {
        for wheel in wheels.read() {
            let amount = match wheel.unit {
                MouseScrollUnit::Line => wheel.y,
                MouseScrollUnit::Pixel => wheel.y / 40.0,
            };
            orbit.distance = (orbit.distance * (-amount * 0.08).exp()).clamp(0.9, 120.0);
            orbit.fit_requested = false;
        }
    } else {
        wheels.clear();
    }
    let horizontal = orbit.pitch.cos();
    let target = Vec3::new(orbit.target_xz.x, orbit.target_y, orbit.target_xz.y);
    let offset = Vec3::new(
        orbit.yaw.sin() * horizontal,
        orbit.pitch.sin(),
        -orbit.yaw.cos() * horizontal,
    ) * orbit.distance;
    **camera = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
}

pub(super) fn sync_preview_camera_viewport(
    window: Single<&Window>,
    mut camera: Single<&mut Camera, With<PreviewCamera>>,
) {
    let Some((position, size)) =
        preview_viewport_logical_rect(Vec2::new(window.width(), window.height()))
    else {
        camera.viewport = None;
        return;
    };
    let scale_factor = window.resolution.scale_factor();
    let physical_window_size = UVec2::new(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    );
    let mut viewport = Viewport {
        physical_position: UVec2::new(
            (position.x * scale_factor).round().max(0.0) as u32,
            (position.y * scale_factor).round().max(0.0) as u32,
        ),
        physical_size: UVec2::new(
            (size.x * scale_factor).round().max(1.0) as u32,
            (size.y * scale_factor).round().max(1.0) as u32,
        ),
        ..default()
    };
    viewport.clamp_to_size(physical_window_size);
    camera.viewport = Some(viewport);
}
