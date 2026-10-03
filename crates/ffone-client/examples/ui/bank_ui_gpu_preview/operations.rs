use super::*;

pub(super) fn default_output(language: &str) -> PathBuf {
    PathBuf::from(format!(
        "target/ui-parity/bank-mode-{language}-1264x681.png"
    ))
}

pub(super) fn preview_projection() -> BankModeProjection0104 {
    let mut load = PcLoadData0104::zeroed();
    let equipment = [
        (0, item(0, 100, 0, 0)),
        (1, item(1, 101, 0, 0)),
        (3, item(3, 103, 0, 0)),
        (4, item(4, 104, 0x0012_0000, 0)),
        (5, item(5, 105, 0, 0)),
        (6, item(6, 106, 0, 0)),
        (7, item(0, 108, 0, 0)),
        (8, item(10, 107, 0, 0)),
    ];
    let inventory = [
        (0, item(0, 100, 0x0011_0002, 0)),
        (1, item(7, 200, 25, 0)),
        (2, item(9, 300, 0, 0)),
        (4, item(1, 101, 0, 0)),
        (5, item(2, 102, 0, 0)),
        (6, item(3, 103, 0, 0)),
        (7, item(4, 104, 0, 0)),
        (8, item(5, 105, 0, 0)),
        (9, item(6, 106, 0, 0)),
        (10, item(10, 107, 0, 0)),
        (11, item(7, 201, 3, 0)),
    ];
    for (slot, value) in equipment {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    for (slot, value) in inventory {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    let popup_fixture =
        std::env::var("FFONE_BANK_POINTER").is_ok_and(|mode| mode.starts_with("popup-"));
    if popup_fixture {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET,
            item(0, 328, 0, 0),
        );
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + ItemBase0104::SIZE,
            item(7, 7, 25, 0),
        );
    }
    let runtime = InventoryRuntime0104::from_pc_load(4_242, &load);

    let mut bank_items = [item(0, 0, 0, 0); BANK_SLOT_COUNT_0104];
    bank_items[0] = item(7, 200, 99, 0);
    bank_items[1] = item(0, 100, 0x0011_0002, 0);
    bank_items[2] = item(9, 300, 0, 0);
    bank_items[3] = item(1, 101, 0, 0);
    bank_items[4] = item(2, 102, 0, 0);
    bank_items[5] = item(3, 103, 0, 0);
    bank_items[6] = item(4, 104, 0, 0);
    bank_items[7] = item(5, 105, 0, 0);
    bank_items[8] = item(6, 106, 0, 0);
    bank_items[9] = item(10, 107, 0, 0);
    bank_items[12] = item(7, 201, 7, 0);
    if popup_fixture {
        bank_items[0] = item(7, 7, 99, 0);
        bank_items[1] = item(0, 328, 0, 0);
        match std::env::var("FFONE_BANK_POINTER").as_deref() {
            Ok("popup-long-name") => bank_items[1] = item(1, 92, 0, 0),
            Ok("popup-expires") => bank_items[1] = item(1, 100, 0, 1800000000),
            Ok("popup-rental") => bank_items[1] = item(10, 1, 0, 1800000000),
            _ => {}
        }

        if std::env::var("FFONE_BANK_POINTER").as_deref() == Ok("popup-combined") {
            bank_items[1] = item(1, 100, 107 << 16, 0);
        }
        if std::env::var("FFONE_BANK_POINTER").as_deref() == Ok("popup-vehicle") {
            bank_items[1] = item(10, 1, 0, 0);
        }
        bank_items[2] = item(9, 77, 0, 0);
    }

    BankModeProjection0104::from_authoritative_open(
        4_242,
        815,
        &PcBankOpenSuccess0104 {
            bank_items,
            extra_bank: 1,
        },
        &runtime,
        &PreviewCatalog,
        &PreviewEligibility,
    )
    .expect("preview authority must agree")
}

pub(super) const fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

pub(super) fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut pc_stuff: ResMut<BankPcStuffAuthority0104>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.insert_resource(preview_projection());
    *state = BankUiState {
        phase: BankLifecyclePhase::Visible,
        opening_elapsed_seconds: BANK_OPEN_SECONDS,
        ..default()
    };
    *modal = BankModalState::default();
    *pc_stuff = BankPcStuffAuthority0104::from_authoritative_taros(12_345_678);
    commands.insert_resource(PreviewAssets {
        images: BANK_UI_DEFAULT_IMAGE_PATHS
            .iter()
            .copied()
            .chain(PREVIEW_ICON_PATHS)
            .chain([
                "icons/items/weapons/wpnicon_385.png",
                "icons/items/general/generalitemicon_16.png",
            ])
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load("fonts/jeffe.otf"),
        search_font: asset_server.load(ffone_client::bank_ui::BANK_SEARCH_FONT_PATH),
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    language: Res<PreviewLanguage>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<BankUiRoot>>,
    elements: Query<(&BankUiElement, &ComputedNode)>,
    texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &BankUiTextStyle,
        (&TextFont, &LineHeight, &ComputedTextBlock),
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
        &UiGlobalTransform,
        Option<&ChildOf>,
    )>,
    computed_nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        );
    if failed {
        eprintln!("BankMode acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Loaded
        );
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some()
        && fonts.get(&preview_assets.search_font).is_some();
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let element_count_exact = elements.iter().count() == EXPECTED_ELEMENT_COUNT;
    let viewport_exact = elements.iter().any(|(element, node)| {
        *element == BankUiElement::BankViewport && node.size() == Vec2::new(450.0, 445.0)
    });
    let panel_geometry_exact = elements.iter().any(|(element, node)| {
        *element == BankUiElement::BankPanel && node.size() == Vec2::new(495.0, 638.0)
    }) && elements.iter().any(|(element, node)| {
        *element == BankUiElement::PcStuffPanel && node.size() == Vec2::new(380.0, 632.0)
    }) && elements.iter().any(|(element, node)| {
        *element == BankUiElement::EquipmentPanel && node.size() == Vec2::new(66.0, 639.0)
    });
    let text_audit = audit_bank_text(
        &texts,
        &computed_nodes,
        &preview_assets.font,
        &preview_assets.search_font,
        &language.0,
    );
    if assets_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("BankMode text audit waiting: {error}");
            }
            Err(_) => {}
        }
    }
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && element_count_exact
        && viewport_exact
        && panel_geometry_exact
        && (text_audit.is_ok()
            || (env::var_os("FFONE_SERVICE_CONTROL").is_some() && state.text_audited));
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }

    let warmed = state.ready_frame.is_some_and(|frame| {
        state.frames.saturating_sub(frame)
            >= if std::env::var("FFONE_BANK_POINTER")
                .is_ok_and(|mode| mode.starts_with("popup-delete"))
            {
                40
            } else {
                WARMUP_FRAMES_AFTER_LOAD
            }
    }) && state
        .ready_at
        .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!(
            "BankMode acceptance capture timed out: elements={} expected={EXPECTED_ELEMENT_COUNT}",
            elements.iter().count()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

pub(super) fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

pub(super) fn exercise_service_controls(
    mut cursor: Local<Option<Vec2>>,
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<&mut Window>,
    elements: Query<(&BankUiElement, &UiGlobalTransform)>,
    names: Query<(&Name, &UiGlobalTransform, &Node)>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut outbox: ResMut<ffone_client::bank_ui::BankUiOutbox0104>,
    state: Res<BankUiState>,
    mut modal: ResMut<BankModalState>,
    projection: Res<BankModeProjection0104>,
    mut before: Local<Option<BankModeProjection0104>>,
) {
    let Ok(mode) = env::var("FFONE_SERVICE_CONTROL") else {
        return;
    };
    if preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    if *frame < 20 {
        return;
    }
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
    let mut move_to = |p: Vec2| {
        *cursor = Some(p);
        window.set_physical_cursor_position(Some(p.as_dvec2()));
    };
    if mode.starts_with("drag") {
        if *frame == 20 {
            *before = Some(projection.clone());
            move_to(point(BankUiElement::BankSlotFrame(0)));
        }
        if *frame == 21 {
            mouse.press(MouseButton::Left);
        }
        if *frame == 22 {
            move_to(Vec2::new(650., 300.));
        }
        if *frame == 24 {
            assert!(
                names
                    .iter()
                    .any(|(n, _, node)| n.as_str() == "Bank dragged item"
                        && node.display != Display::None)
            );
            assert!(outbox.is_empty());
            println!("PASS bank ghost visible before authority changes");
        }
        if mode == "drag-hold" {
            return;
        }
        if *frame == 29 && mode == "drag-modal" {
            modal.generic_popup = true;
        }
        if *frame == 30 && mode == "drag-drop" {
            move_to(point(BankUiElement::InventorySlotFrame(4)));
        }
        if *frame == 31 {
            mouse.release(MouseButton::Left);
        }
        if *frame == 32 {
            if mode == "drag-drop" {
                assert!(
                    matches!(outbox.pop_front(),Some(ffone_client::bank_ui::BankUiCommand0104::ItemMove(p)) if p.from_location==3 && p.from_slot_num==0 && p.to_location==1 && p.to_slot_num==4)
                );
            } else {
                assert!(outbox.is_empty());
            }
            if mode == "drag-modal" {
                assert!(
                    names
                        .iter()
                        .any(|(n, _, node)| n.as_str() == "Bank dragged item"
                            && node.display == Display::None)
                );
            }
            assert_eq!(before.as_ref().unwrap(), &*projection);
            println!("PASS bank {mode}: exact request/cancel; authoritative inventory unchanged");
        }
        return;
    }
    let role = if mode == "scroll-up" {
        BankUiElement::BankScrollUp
    } else if mode == "scroll-thumb" {
        BankUiElement::BankScrollThumb
    } else if mode == "scroll-track" {
        BankUiElement::BankScrollTrack
    } else {
        BankUiElement::BankScrollDown
    };
    if *frame == 20 {
        let p = if mode == "inventory-scroll" {
            names
                .iter()
                .find(|(n, _, _)| n.as_str() == "BankInventory scroll Down")
                .unwrap()
                .1
                .translation
        } else {
            point(role)
        };
        move_to(p);
    }
    if *frame == 21 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 22 && mode == "scroll-thumb" {
        move_to(point(BankUiElement::BankScrollDown) + Vec2::new(0., 30.));
    }
    if *frame == 70 {
        mouse.release(MouseButton::Left);
    }
    if *frame == 72 {
        if mode == "inventory-scroll" {
            assert!(state.inventory_scroll_y > 0.);
        } else if mode == "scroll-up" {
            assert_eq!(state.bank_scroll_y, 0.);
        } else if mode == "scroll-thumb" {
            assert_eq!(
                state.bank_scroll_y,
                ffone_client::bank_ui::bank_scroll_max()
            );
        } else {
            assert!(state.bank_scroll_y > 0.);
            if mode == "scroll-hold" {
                assert!(state.bank_scroll_y > 10.);
            }
        }
        assert!(outbox.is_empty());
        println!("PASS bank {mode} pointer scroll");
    }
}

// Exercise the real field hit target and Bevy keyboard delivery, then restore the
// complete bank before the geometry capture. Filtered items retain their slot IDs.
pub(super) fn exercise_bank_search(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<(Entity, &mut Window)>,
    labels: Query<(&LocalizedText, &UiGlobalTransform)>,
    slots: Query<(&BankUiElement, &Node)>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut keyboard: MessageWriter<bevy::input::keyboard::KeyboardInput>,
) {
    if env::var_os("FFONE_BANK_SEARCH").is_none() || preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    let (window_entity, mut window) = windows.single_mut().unwrap();
    if *frame <= 4 {
        let (_, transform) = labels
            .iter()
            .find(|(text, _)| text.key.starts_with("ui.bank.search."))
            .unwrap();
        window.focused = true;
        window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
    }
    if *frame == 2 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 3 {
        mouse.release(MouseButton::Left);
    }
    let key = match *frame {
        4 => Some((KeyCode::KeyZ, Some("zzzzzzzzzz"))),
        6..=15 => Some((KeyCode::Backspace, None)),
        17 => Some((KeyCode::Escape, None)),
        _ => None,
    };
    if let Some((key_code, text)) = key {
        keyboard.write(bevy::input::keyboard::KeyboardInput {
            key_code,
            logical_key: bevy::input::keyboard::Key::Character(text.unwrap_or("").into()),
            state: bevy::input::ButtonState::Pressed,
            text: text.map(Into::into),
            repeat: false,
            window: window_entity,
        });
    }
    if *frame == 5 || *frame == 16 {
        let visible = slots
            .iter()
            .filter(|(e, node)| {
                matches!(e, BankUiElement::BankSlotFrame(_)) && node.display != Display::None
            })
            .count();
        assert_eq!(
            visible,
            if *frame == 5 { 0 } else { 200 },
            "real keyboard search visibility"
        );
    }
    if *frame == 18 {
        assert!(
            labels
                .iter()
                .any(|(text, _)| text.key == "ui.bank.search.placeholder")
        );
        println!(
            "PASS bank search: label hit, keyboard text, filtering, backspace, Escape focus release"
        );
    }
}
