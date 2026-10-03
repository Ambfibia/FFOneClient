use super::*;

pub(super) fn default_output(language: &str) -> PathBuf {
    PathBuf::from(format!(
        "target/ui-parity/vendor-mode-{language}-1264x681.png"
    ))
}

pub(super) fn open_popup_preview(
    mut opened: Local<bool>,
    state: Res<VendorUiState>,
    mut projection: ResMut<VendorModeProjection0104>,
    content: Res<ffone_client::tutorial_mission_content::TutorialMissionContent>,
    mut popup: ResMut<ffone_client::vendor_ui::VendorItemPopupState>,
) {
    if *opened || state.phase != VendorLifecyclePhase::Visible {
        return;
    }
    let Ok(scenario) = env::var("FFONE_VENDOR_POPUP") else {
        return;
    };
    use ffone_client::vendor_ui::{VendorActivationOutcome0104, VendorTab0104};
    if scenario.starts_with("chest") {
        let value = item(9, 77, 1, 0);
        let metadata = content.resolve(value).expect("production chest metadata");
        let row = &mut projection.inventory[11].item;
        row.item = value;
        row.empty = false;
        row.icon = metadata.icon.clone().map_or(
            ffone_client::vendor_ui::VendorPresentationIcon0104::Empty,
            ffone_client::vendor_ui::VendorPresentationIcon0104::Resolved,
        );
        row.metadata = Some(metadata);
        row.count_label = None;
    }
    if scenario == "long-name"
        || scenario == "expires"
        || scenario == "rental"
        || scenario == "combined"
        || scenario == "vehicle"
        || scenario == "localized-weapon"
        || scenario == "localized-general"
        || scenario.starts_with("try-on")
    {
        let value = if scenario == "long-name" {
            item(1, 92, 0, 0)
        } else if scenario == "expires" {
            item(1, 100, 0, 1800000000)
        } else if scenario == "rental" {
            item(10, 1, 0, 90061)
        } else if scenario == "combined" {
            item(1, 100, 107 << 16, 0)
        } else if scenario == "vehicle" {
            item(10, 1, 0, 0)
        } else if scenario != "localized-general" {
            item(0, 328, 0, 0)
        } else {
            item(7, 7, 0, 0)
        };
        let metadata = content.resolve(value).expect("production item metadata");
        let row = &mut projection.catalog_rows[0];
        row.item = value;
        row.icon = metadata.icon.clone().map_or(
            ffone_client::vendor_ui::VendorPresentationIcon0104::Empty,
            ffone_client::vendor_ui::VendorPresentationIcon0104::Resolved,
        );
        row.price = Some(metadata.buy_price);
        row.metadata = Some(metadata);
    }
    let activation = match scenario.as_str() {
        "long-name" | "expires" | "rental" | "combined" | "vehicle" | "buy"
        | "localized-weapon" | "localized-general" | "try-on" | "try-on-close" => {
            projection.primary_row_activation(VendorTab0104::Buy, 0)
        }
        "quantity" => projection.primary_row_activation(VendorTab0104::Buy, 6),
        "buyback" => projection.primary_row_activation(VendorTab0104::Buyback, 0),
        "sell" => projection.primary_inventory_activation(11),
        "chest" | "chest-open" => projection.primary_inventory_activation(11),
        _ => panic!("unsupported popup capture scenario"),
    };
    let VendorActivationOutcome0104::Popup(contract) = activation else {
        panic!("preview must open popup")
    };
    popup.open(contract, &projection);
    popup.set_try_on_allowed(scenario.starts_with("try-on"));
    assert!(popup.is_open());
    *opened = true;
}

pub(super) fn drive_try_on_preview(
    data: Res<TryOnFixture>,
    popup: Res<ffone_client::vendor_ui::VendorItemPopupState>,
    mut model: ResMut<ffone_client::player_preview::NativePlayerPreviewModel>,
) {
    use ffone_client::{network::CharacterSummary, player_preview::NativePlayerPreviewStage};
    use ffone_protocol::{CharacterStyle0104, EquippedItem0104};
    let Some(item) = popup.try_on_item() else {
        model.visible = false;
        model.clear_look();
        return;
    };
    let mut equipment = [EquippedItem0104::default(); 9];
    for (kind, id) in [(1, 100), (2, 357), (3, 400)] {
        equipment[kind] = EquippedItem0104 {
            item_type: kind as i16,
            item_id: id,
            ..default()
        };
    }
    let character = CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 1,
        first_name: "Test".into(),
        last_name: "Hero".into(),
        position: [0; 3],
        equipment,
        style: CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 4,
            hair_style: 4,
            hair_color: 14,
            skin_color: 9,
            eye_color: 2,
            height: 2,
            body: 0,
            class: 0,
            appearance_flag: 1,
            tutorial_flag: 0,
            payzone_flag: 0,
        },
    };
    model.stage = NativePlayerPreviewStage::TryOn;
    model.yaw_degrees = popup.try_on_yaw;
    model
        .set_look(data.0.resolve_try_on_character(&character, item).unwrap())
        .unwrap();
    model.visible = true;
}

pub(super) fn preview_projection() -> VendorModeProjection0104 {
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
        // Clean item type 8 exercises its keyed `Quest {item_id}` copy while
        // preserving AvatarUtil's early-null checker behavior.
        (2, item(8, 300, 0, 0)),
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
    let runtime = InventoryRuntime0104::from_pc_load(4_242, &load);
    let vendor = [
        vendor_entry(0, item(0, 100, 0, 0)),
        vendor_entry(1, item(1, 101, 0, 0)),
        vendor_entry(2, item(2, 102, 0, 0)),
        vendor_entry(3, item(3, 103, 0, 0)),
        vendor_entry(4, item(10, 107, 0, 0)),
        vendor_entry(5, item(0, 108, 0, 0)),
        vendor_entry(6, item(7, 200, 0, 0)),
        vendor_entry(7, item(9, 300, 0, 0)),
    ];
    let recent = [
        VendorRecentBuyEntry0104 {
            source_slot_id: 0,
            item: item(7, 201, 3, 0),
        },
        VendorRecentBuyEntry0104 {
            source_slot_id: 1,
            item: item(0, 100, 0, 0),
        },
    ];
    VendorModeProjection0104::from_authoritative(
        4_242,
        ffone_client::vendor_ui::VendorSession0104 {
            requested_npc_id: 815,
            table_vendor_id: 27,
            accepted_npc_id: 815,
        },
        1_500,
        25,
        40,
        "Computress",
        "Rare gear, batteries, and supplies.",
        &vendor,
        &recent,
        &runtime,
        &PreviewCatalog,
        &PreviewEligibility,
    )
    .expect("preview authority must agree")
}

pub(super) const fn vendor_entry(source_slot_id: usize, item: ItemBase0104) -> VendorCatalogEntry0104 {
    VendorCatalogEntry0104 {
        source_slot_id,
        item,
    }
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
    mut state: ResMut<VendorUiState>,
    mut modal: ResMut<VendorModalState>,
    mut close_gate: ResMut<VendorCloseGate0104>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.insert_resource(preview_projection());
    *state = VendorUiState {
        phase: VendorLifecyclePhase::Visible,
        opening_elapsed_seconds: VENDOR_OPEN_SECONDS,
        ..default()
    };
    *modal = VendorModalState::default();
    *close_gate = VendorCloseGate0104 {
        mode_accepts_escape: true,
        target_action_idle: true,
    };
    commands.insert_resource(PreviewAssets {
        images: VENDOR_UI_DEFAULT_IMAGE_PATHS
            .iter()
            .copied()
            .chain(PREVIEW_ICON_PATHS)
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load("fonts/jeffe.otf"),
        service_font: asset_server.load(VENDOR_SERVICE_FONT_PATH),
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    asset_status: Res<VendorUiAssetStatus>,
    language: Res<PreviewLanguage>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<VendorUiRoot>>,
    elements: Query<(&VendorUiElement, &ComputedNode)>,
    texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &VendorUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
    )>,
    checked: Res<TryOnChecked>,
    mut try_on_warm: Local<Option<Instant>>,
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
        )
        || matches!(
            asset_server.load_state(preview_assets.service_font.id()),
            LoadState::Failed(_)
        )
        || asset_status.0 == VendorStaticAssetReadiness::Failed;
    if failed {
        eprintln!("VendorMode acceptance asset failed to load");
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
        )
        && matches!(
            asset_server.load_state(preview_assets.service_font.id()),
            LoadState::Loaded
        )
        && asset_status.0 == VendorStaticAssetReadiness::Ready;
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some()
        && fonts.get(&preview_assets.service_font).is_some();
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let panel_exact = element_has_size(
        &elements,
        VendorUiElement::VendorPanel,
        Vec2::new(498.0, 638.0),
    ) && element_has_size(
        &elements,
        VendorUiElement::PcStuffPanel,
        Vec2::new(380.0, 632.0),
    );
    let viewport_exact = element_has_size(
        &elements,
        VendorUiElement::ListViewport,
        Vec2::new(464.0, 400.0),
    );
    let npc_boundary_exact = element_has_size(
        &elements,
        VendorUiElement::NpcPreviewBoundary,
        Vec2::new(200.0, 150.0),
    );
    let first_row_exact =
        element_has_size(&elements, VendorUiElement::Row(0), Vec2::new(433.0, 75.0));
    let element_count_exact = elements.iter().count() == EXPECTED_ELEMENT_COUNT;
    let text_audit = audit_vendor_text(&texts, &preview_assets, &language.0);
    if assets_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("VendorMode text audit waiting: {error}");
            }
            Err(_) => {}
        }
    }
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && panel_exact
        && viewport_exact
        && npc_boundary_exact
        && first_row_exact
        && element_count_exact
        && (text_audit.is_ok()
            || (env::var_os("FFONE_SERVICE_CONTROL").is_some() && state.text_audited));
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }

    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if checked.0 && try_on_warm.is_none() {
        *try_on_warm = Some(Instant::now());
    }
    let avatar_ready = !env::var("FFONE_VENDOR_POPUP").is_ok_and(|s| s.starts_with("try-on"))
        || try_on_warm.is_some_and(|since| since.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && avatar_ready && !state.capture_issued {
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
            "VendorMode acceptance capture timed out: elements={} expected={EXPECTED_ELEMENT_COUNT}",
            elements.iter().count(),
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

pub(super) fn element_has_size(
    elements: &Query<(&VendorUiElement, &ComputedNode)>,
    wanted: VendorUiElement,
    size: Vec2,
) -> bool {
    elements
        .iter()
        .any(|(element, node)| *element == wanted && node.size() == size)
}

pub(super) fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}
