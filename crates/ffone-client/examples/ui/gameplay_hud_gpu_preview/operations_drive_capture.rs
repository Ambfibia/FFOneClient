use super::*;

pub(super) fn drive_capture(
    mut commands: Commands,
    portraits: Res<CharacterSelectionPortraitsModel>,
    mut state: ResMut<PreviewState>,
    mut transition: ResMut<GameplayMenuTransition>,
    roots: Query<&ComputedNode, With<GameplayHud>>,
    mut exit: MessageWriter<AppExit>,
    assets: Res<AssetServer>,
    images: Query<(&ImageNode, &ComputedNode, &UiGlobalTransform)>,
    scrolls: Query<(&ScrollPosition, &ComputedNode, &UiGlobalTransform)>,
    mut windows: Query<(Entity, &mut Window), With<bevy::window::PrimaryWindow>>,
    mut mouse: MessageWriter<bevy::input::mouse::MouseButtonInput>,
    mut wheel: MessageWriter<bevy::input::mouse::MouseWheel>,
    mut probe: Local<ScrollPointerProbe>,
    output: Res<PreviewOutput>,
    mut model: ResMut<GameplayUiModel>,
    mut notice: ResMut<ffone_client::gameplay_ui::CombatModeNotice>,
) {
    if env::var("FFONE_MISSION_UI_PREVIEW").as_deref() == Ok("enter-mid") {
        transition.open = true;
        transition.remaining = 0.5;
    }
    if let CharacterSelectionPortraitStatus::Blocked(error) = &portraits.slots[0].status {
        eprintln!("gameplay player portrait blocked: {error}");
        exit.write(AppExit::error());
        return;
    }
    let portrait_ready = matches!(
        portraits.slots[0].status,
        CharacterSelectionPortraitStatus::ReadyAnimated { .. }
    );
    if portrait_ready && state.portrait_ready_at.is_none() {
        state.portrait_ready_at = Some(Instant::now());
    }
    let laid_out = roots
        .single()
        .is_ok_and(|computed| computed.size().x >= 1264.0 && computed.size().y >= 681.0);
    let portrait_warm = state
        .portrait_ready_at
        .is_some_and(|ready| ready.elapsed() >= PORTRAIT_WARMUP);
    if !state.capture_issued && laid_out && portrait_warm {
        if let Ok(location) = env::var("FFONE_LOCATION_NOTICE_PREVIEW") {
            let Some(started) = state.damage_at else {
                let mut random =
                    ffone_client::legacy_npc_nano_animation::LegacyNanoStandRandomStream::with_seed(
                        7,
                    );
                notice.show_location(&location, &mut random);
                state.damage_at = Some(Instant::now());
                return;
            };
            if started.elapsed() < Duration::from_millis(1000) {
                return;
            }
        }
        if env::var_os("FFONE_REVIEW_DAMAGE").is_some() {
            let Some(started) = state.damage_at else {
                model.player.hp /= 2;
                for nano in &mut model.nanos {
                    nano.stamina_fraction *= 0.5;
                }
                model.player.allow_player_interaction = false;
                notice.show(true);
                state.damage_at = Some(Instant::now());
                return;
            };
            if started.elapsed() < Duration::from_millis(180) {
                return;
            }
        }

        if env::var_os("FFONE_CHAT_SCROLL_POINTER_PREVIEW").is_some() && probe.phase < 6 {
            let (window_entity, mut window) = windows.single_mut().unwrap();
            window.focused = true;
            let image_geometry = |suffix: &str| {
                images
                    .iter()
                    .find_map(|(image, node, transform)| {
                        assets
                            .get_path(image.image.id())
                            .filter(|path| path.path().ends_with(suffix))
                            .map(|_| {
                                (
                                    bevy::math::Affine2::from(transform).translation,
                                    node.size(),
                                )
                            })
                    })
                    .unwrap()
            };
            let offset = || {
                scrolls
                    .iter()
                    .filter_map(|(scroll, node, _)| {
                        let max =
                            (node.content_size().y - node.size().y) * node.inverse_scale_factor();
                        (max > 0.0 && node.size().y > 0.0).then(|| scroll.y.clamp(0.0, max))
                    })
                    .max_by(f32::total_cmp)
                    .expect("visible overflowing chat viewport")
            };
            let wheel_probe =
                env::var("FFONE_CHAT_SCROLL_POINTER_PREVIEW").as_deref() == Ok("wheel");
            match probe.phase {
                0 if wheel_probe => {
                    probe.before = offset();
                    let (_, node, transform) = scrolls
                        .iter()
                        .filter(|(_, node, _)| {
                            node.size().y > 0.0 && node.content_size().y > node.size().y
                        })
                        .max_by(|(left, _, _), (right, _, _)| left.y.total_cmp(&right.y))
                        .unwrap();
                    let point = bevy::math::Affine2::from(transform).translation
                        - Vec2::new(node.size().x * 0.3, 0.0);
                    window.set_physical_cursor_position(Some(point.as_dvec2()));
                }
                2 if wheel_probe => {
                    wheel.write(bevy::input::mouse::MouseWheel {
                        phase: bevy::input::touch::TouchPhase::Moved,
                        unit: bevy::input::mouse::MouseScrollUnit::Line,
                        x: 0.0,
                        y: 3.0,
                        window: window_entity,
                    });
                }
                0 => {
                    let (thumb, _) = image_geometry("chat/scrollbar/thumb.png");
                    let (track, size) = image_geometry("chat/scrollbar/track.png");
                    probe.before = offset();
                    probe.destination = Vec2::new(thumb.x, track.y - size.y * 0.35);
                    window.set_physical_cursor_position(Some(thumb.as_dvec2()));
                    mouse.write(bevy::input::mouse::MouseButtonInput {
                        button: MouseButton::Left,
                        state: bevy::input::ButtonState::Pressed,
                        window: window_entity,
                    });
                }
                2 => window.set_physical_cursor_position(Some(probe.destination.as_dvec2())),
                4 => {
                    mouse.write(bevy::input::mouse::MouseButtonInput {
                        button: MouseButton::Left,
                        state: bevy::input::ButtonState::Released,
                        window: window_entity,
                    });
                }
                5 => {
                    let after = offset();
                    assert!(
                        after < probe.before - 10.0,
                        "production pointer drag did not scroll: {} -> {after}",
                        probe.before
                    );
                    fs::write(
                        output.0.with_extension("pointer.json"),
                        format!("{{\"before\":{},\"after\":{after}}}", probe.before),
                    )
                    .unwrap();
                }
                _ => {}
            }
            probe.phase += 1;
            return;
        }
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("gameplay HUD capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}
