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
    if state.world_open.is_some() {
        motions.clear();
        wheels.clear();
        return;
    }
    let over_viewport = !icons.active
        && !state.strings_open
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
    if over_viewport
        && (mouse.pressed(MouseButton::Left)
            || mouse.pressed(MouseButton::Right)
            || mouse.pressed(MouseButton::Middle))
    {
        for motion in motions.read() {
            if mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle) {
                let factor = orbit.distance * 0.0015;
                let rotation = Quat::from_rotation_y(-orbit.yaw);
                let right = rotation * -Vec3::X;
                let up = rotation * Vec3::new(0., orbit.pitch.cos(), orbit.pitch.sin());
                let offset = (-right * motion.delta.x + up * motion.delta.y) * factor;
                orbit.target_xz += Vec2::new(offset.x, offset.z);
                orbit.target_y += offset.y;
                orbit.fit_requested = false;
            } else {
                orbit.yaw -= motion.delta.x * 0.007;
                orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(-0.45, 1.05);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn right_drag_pans_with_camera_angle_and_left_drag_orbits() {
        let state = EditorState {
            reveal_selection: false,
            kind: CatalogKind::Npc,
            selected: 0,
            search: String::new(),
            search_focused: false,
            strings_open: false,
            xdt_open: false,
            missions_open: false,
            world_open: None,
            viewer_tabs: BTreeMap::new(),
            details_open: false,
            npc_inspector: NpcInspectorTab::Details,
            equipment_female: false,
            equipment_category: None,
            animation_page: 0,
            clip_index: 0,
            pose_mode: EditorPoseMode::Default,
            paused: true,
            looping: true,
            speed: 1.,
            turntable: false,
            playback_revision: 0,
        };
        let mut app = App::new();
        app.insert_resource(state)
            .init_resource::<OrbitCamera>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(icon_generator::IconGenerator::new(PathBuf::from("unused")))
            .add_message::<MouseMotion>()
            .add_message::<MouseWheel>()
            .add_systems(Update, update_orbit_camera);
        let mut window = Window::default();
        window.resolution.set(1280., 720.);
        let (position, size) = preview_viewport_logical_rect(Vec2::new(1280., 720.)).unwrap();
        window.set_cursor_position(Some(position + size * 0.5));
        app.world_mut().spawn(window);
        app.world_mut().spawn((PreviewCamera, Transform::default()));
        let delta = Vec2::new(20., 10.);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.world_mut().write_message(MouseMotion { delta });
        app.update();
        let orbit = app.world().resource::<OrbitCamera>();
        assert_eq!(orbit.yaw, 0.);
        assert_eq!(orbit.pitch, 0.12);
        assert!(orbit.target_xz.x > 0.);
        assert!(orbit.target_y > 1.);
        let target = (orbit.target_xz, orbit.target_y);
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.release(MouseButton::Right);
        mouse.press(MouseButton::Left);
        app.world_mut().write_message(MouseMotion { delta });
        app.update();
        let orbit = app.world().resource::<OrbitCamera>();
        assert_ne!(orbit.yaw, 0.);
        assert_eq!((orbit.target_xz, orbit.target_y), target);
        app.world_mut().resource_mut::<OrbitCamera>().yaw = std::f32::consts::FRAC_PI_2;
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.release(MouseButton::Left);
        mouse.press(MouseButton::Right);
        app.world_mut().write_message(MouseMotion {
            delta: Vec2::new(20., 0.),
        });
        app.update();
        let orbit = app.world().resource::<OrbitCamera>();
        assert!((orbit.target_xz.x - target.0.x).abs() < 0.0001);
        assert!(orbit.target_xz.y > target.0.y);
    }
}
