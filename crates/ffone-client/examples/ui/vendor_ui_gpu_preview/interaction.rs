use super::*;

pub(super) fn exercise_try_on_pointer(
    mut phase: Local<u32>,
    mut checked: ResMut<TryOnChecked>,
    model: Res<ffone_client::player_preview::NativePlayerPreviewModel>,
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    popup: Res<ffone_client::vendor_ui::VendorItemPopupState>,
    labels: Query<(&LocalizedText, &UiGlobalTransform, &ComputedNode)>,
    mut windows: Query<&mut Window>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
) {
    if !env::var("FFONE_VENDOR_POPUP").is_ok_and(|s| s.starts_with("try-on"))
        || preview.ready_frame.is_none()
    {
        return;
    }
    *frame += 1;
    if *frame > 19 && !checked.0 {
        if env::var("FFONE_VENDOR_POPUP").as_deref() == Ok("try-on-close") {
            if *phase > 0
                || matches!(
                    model.status,
                    ffone_client::player_preview::NativePlayerPreviewStatus::ReadyAnimated { .. }
                )
            {
                *phase += 1;
                let mut window = windows.single_mut().unwrap();
                let center = Vec2::new(
                    ((window.width() as i32 - 1020).max(0) / 2) as f32,
                    ((window.height() as i32 - 638).max(0) / 2) as f32,
                );
                match *phase {
                    1 => window.set_cursor_position(Some(center + Vec2::new(350.5, 409.))),
                    2 => mouse.press(MouseButton::Left),
                    6 => {
                        mouse.release(MouseButton::Left);
                        assert!(popup.try_on_yaw > 0.);
                    }
                    7 => window.set_cursor_position(Some(center + Vec2::new(508., 116.))),
                    8 => mouse.press(MouseButton::Left),
                    9 => mouse.release(MouseButton::Left),
                    11 => {
                        assert!(popup.is_open());
                        assert!(popup.try_on_item().is_none());
                        checked.0 = true;
                        eprintln!("Vendor real-pointer rotation and preview-only close passed");
                    }
                    _ => (),
                }
            }
        } else if matches!(
            model.status,
            ffone_client::player_preview::NativePlayerPreviewStatus::ReadyAnimated { .. }
        ) {
            checked.0 = true;
        }
    }
    if *frame == 15 {
        let (_, t, _) = labels
            .iter()
            .find(|(s, _, n)| s.key == "ui.vendor.popup.try_on" && n.size().x > 0.)
            .unwrap();
        let mut window = windows.single_mut().unwrap();
        let pos = t.translation / window.scale_factor();
        window.set_cursor_position(Some(pos));
    } else if *frame == 16 {
        mouse.press(MouseButton::Left);
    } else if *frame == 17 {
        mouse.release(MouseButton::Left);
    } else if *frame == 19 {
        assert!(popup.try_on_item().is_some(), "real TRY ON click");
        eprintln!("Vendor real-pointer TRY ON passed");
    }
}

pub(super) fn exercise_chest_pointer(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    state: Res<VendorUiState>,
    popup: Res<ffone_client::vendor_ui::VendorItemPopupState>,
    mut chest: ResMut<ffone_client::vendor_ui::VendorChestOpenState>,
    labels: Query<(&LocalizedText, &UiGlobalTransform, &ComputedNode)>,
    mut windows: Query<&mut Window>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
) {
    if env::var("FFONE_VENDOR_POPUP").as_deref() != Ok("chest-open")
        || state.phase != VendorLifecyclePhase::Visible
        || preview.ready_frame.is_none()
    {
        return;
    }
    *frame += 1;
    if *frame == 20 {
        let (_, transform, _) = labels
            .iter()
            .find(|(text, _, node)| text.key == "ui.inventory.action.open" && node.size().x > 0.)
            .expect("visible OPEN label");
        let mut window = windows.single_mut().unwrap();
        let cursor = transform.translation / window.scale_factor();
        window.set_cursor_position(Some(cursor));
    } else if *frame == 21 {
        mouse.press(MouseButton::Left);
    } else if *frame == 22 {
        mouse.release(MouseButton::Left);
    } else if *frame == 24 {
        assert!(!popup.is_open());
        let intent = chest
            .take_ready()
            .expect("real OPEN pointer queued one chest");
        assert_eq!(intent.slot, 11);
        assert_eq!(intent.item.item_id, 77);
        assert!(!chest.has_ready());
        eprintln!("Vendor chest real-pointer OPEN accepted once");
    }
}

pub(super) fn exercise_scroll_pointer(
    mut cursor: Local<Option<Vec2>>,
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<&mut Window>,
    elements: Query<(&VendorUiElement, &UiGlobalTransform)>,
    names: Query<(&Name, &UiGlobalTransform)>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    state: Res<VendorUiState>,
    projection: Res<VendorModeProjection0104>,
) {
    let Ok(mode) = env::var("FFONE_SERVICE_CONTROL") else {
        return;
    };
    if preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    let mut window = windows.single_mut().unwrap();
    window.focused = true;
    if let Some(p) = *cursor {
        window.set_physical_cursor_position(Some(p.as_dvec2()));
    }
    let point = |element| {
        elements
            .iter()
            .find(|(e, _)| **e == element)
            .unwrap()
            .1
            .translation
    };
    if *frame == 20 {
        let p = if mode == "inventory-scroll" {
            names
                .iter()
                .find(|(n, _)| n.as_str() == "VendorInventory scroll Down")
                .unwrap()
                .1
                .translation
        } else {
            point(match mode.as_str() {
                "scroll-up" => VendorUiElement::ScrollUp,
                "scroll-thumb" => VendorUiElement::ScrollThumb,
                "scroll-track" => VendorUiElement::ScrollTrack,
                _ => VendorUiElement::ScrollDown,
            })
        };
        *cursor = Some(p);
        window.set_physical_cursor_position(Some(p.as_dvec2()));
    }
    if *frame == 21 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 22 && mode == "scroll-thumb" {
        let p = point(VendorUiElement::ScrollDown) + Vec2::new(0., 30.);
        *cursor = Some(p);
        window.set_physical_cursor_position(Some(p.as_dvec2()));
    }
    if *frame == 70 {
        mouse.release(MouseButton::Left);
    }
    if *frame == 72 {
        if mode == "inventory-scroll" {
            assert!(state.inventory_scroll_y > 0.);
        } else if mode == "scroll-up" {
            assert_eq!(state.vendor_scroll_y, 0.);
        } else if mode == "scroll-thumb" {
            assert_eq!(
                state.vendor_scroll_y,
                ffone_client::vendor_ui::vendor_scroll_max(projection.rows_for_tab(state.tab))
            );
        } else {
            assert!(state.vendor_scroll_y > 0.);
            if mode == "scroll-hold" {
                assert!(state.vendor_scroll_y > 10.);
            }
        }
        println!("PASS vendor {mode} pointer scroll");
    }
}
