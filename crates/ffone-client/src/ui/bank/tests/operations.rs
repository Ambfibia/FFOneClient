use super::*;

pub(super) const fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

pub(super) const fn empty() -> ItemBase0104 {
    item(0, 0, 0, 0)
}

pub(super) fn open_with(extra_bank: i32, items: &[(usize, ItemBase0104)]) -> PcBankOpenSuccess0104 {
    let mut bank_items = [empty(); BANK_SLOT_COUNT_0104];
    for &(slot, value) in items {
        bank_items[slot] = value;
    }
    PcBankOpenSuccess0104 {
        bank_items,
        extra_bank,
    }
}

pub(super) fn projection(
    extra_bank: i32,
    bank: &[(usize, ItemBase0104)],
    inventory: &[(usize, ItemBase0104)],
) -> BankModeProjection0104 {
    BankModeProjection0104::from_authoritative_open(
        OWNER_PC_ID,
        NPC_ID,
        &open_with(extra_bank, bank),
        &runtime_with(inventory),
        &AllCatalog,
        &AllowEquip,
    )
    .unwrap()
}

#[test]
fn clean_half_flag_locks_60_through_199_without_splitting_stacks() {
    let half = projection(
        0,
        &[
            (59, item(7, 40, 99, 0)),
            (60, item(7, 41, 123, 0)),
            (199, item(7, 42, 456, 0)),
        ],
        &[],
    );
    assert!(!half.has_full_access());
    assert_eq!(half.accessible_bank_slots(), 60);
    assert!(!half.bank[59].locked);
    assert!(half.bank[60].locked);
    assert!(half.bank[199].locked);
    assert_eq!(half.bank[60].item.option, 123);
    assert_eq!(half.bank[199].item.option, 456);
    assert_eq!(half.bank[60].count_label.as_deref(), Some("123"));

    let full = projection(1, &[(199, item(7, 42, 456, 0))], &[]);
    assert!(full.has_full_access());
    assert_eq!(full.accessible_bank_slots(), 200);
    assert!(!full.bank[199].locked);
}

#[test]
fn transfers_emit_exact_requests_without_speculative_projection_mutation() {
    let projection = projection(1, &[(5, item(7, 77, 25, 0))], &[(0, item(7, 88, 3, 0))]);
    let before = projection.clone();
    let from = BankSlotRef0104::new(BankSlotLocation0104::Bank, 5).unwrap();
    let to = BankSlotRef0104::new(BankSlotLocation0104::Inventory, 1).unwrap();
    let intent = projection.transfer_intent(from, to).unwrap();
    let request = intent.wire_request();
    assert_eq!(
        request,
        ItemMoveRequest0104 {
            from_location: 3,
            from_slot_num: 5,
            to_location: 1,
            to_slot_num: 1,
        }
    );
    assert_eq!(
        request.encode(),
        [3i32, 5, 1, 1]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(projection, before);

    let one_click = projection.one_click_intent(from).unwrap();
    assert_eq!(
        one_click.to, to,
        "first empty inventory slot is authoritative"
    );
    assert_eq!(
        projection.transfer_intent(from, from),
        Err(BankTransferError0104::SameSlotNoOp { slot: from })
    );
}

#[test]
fn authoritative_move_success_applies_post_values_atomically_then_requires_rebuild() {
    let moved = item(7, 77, 25, 0);
    let runtime = runtime_with(&[(0, item(7, 88, 3, 0))]);
    let mut snapshot = BankAuthoritativeSnapshot0104::from_open_success(
        OWNER_PC_ID,
        NPC_ID,
        &open_with(1, &[(5, moved)]),
        &runtime,
    )
    .unwrap();
    let mut projection = BankModeProjection0104::from_authoritative_snapshot(
        &snapshot,
        &AllCatalog,
        &AllowEquip,
    );
    let projection_before = projection.clone();

    let receipt = snapshot
        .apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 3,
            from_slot_num: 5,
            from_slot_item: empty(),
            to_location: 1,
            to_slot_num: 1,
            to_slot_item: moved,
        })
        .unwrap();
    assert_eq!(
        receipt,
        BankAuthorityMutationReceipt0104 {
            primary: BankSlotRef0104::new(BankSlotLocation0104::Bank, 5).unwrap(),
            secondary: Some(BankSlotRef0104::new(BankSlotLocation0104::Inventory, 1).unwrap()),
        }
    );
    assert_eq!(snapshot.bank()[5], empty());
    assert_eq!(snapshot.inventory()[1], moved);
    assert_eq!(
        projection, projection_before,
        "authoritative store changes do not silently mutate a rendered projection"
    );
    projection.rebuild_from_authoritative_snapshot(&snapshot, &AllCatalog, &AllowEquip);
    assert!(projection.bank[5].empty);
    assert_eq!(projection.item_mode.inventory[1].item.item, moved);

    let before_conflict = snapshot.clone();
    let same_slot = BankSlotRef0104::new(BankSlotLocation0104::Bank, 5).unwrap();
    assert_eq!(
        snapshot.apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 3,
            from_slot_num: 5,
            from_slot_item: empty(),
            to_location: 3,
            to_slot_num: 5,
            to_slot_item: item(7, 999, 1, 0),
        }),
        Err(BankAuthorityMutationError0104::ConflictingSameSlotValues {
            slot: same_slot,
            first: empty(),
            second: item(7, 999, 1, 0),
        })
    );
    assert_eq!(snapshot, before_conflict, "failed receipt is atomic");
}

#[test]
fn hidden_or_missing_static_assets_never_draw() {
    let projection = BankModeProjection0104::default();
    assert!(
        bank_mode_view(
            1_264,
            681,
            BankUiState::default(),
            BankModalState::default(),
            &projection,
            true,
        )
        .is_none()
    );
    let mut visible = BankUiState {
        phase: BankLifecyclePhase::Visible,
        opening_elapsed_seconds: BANK_OPEN_SECONDS,
        ..default()
    };
    assert!(
        bank_mode_view(
            1_264,
            681,
            visible,
            BankModalState::default(),
            &projection,
            false,
        )
        .is_none()
    );
    visible.send_pending = true;
    let view = bank_mode_view(
        1_264,
        681,
        visible,
        BankModalState::default(),
        &projection,
        true,
    )
    .unwrap();
    assert!(!view.controls_enabled);
}

#[test]
fn bank_tree_localizes_every_text_and_preserves_source_font_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(BankUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut texts = world.query::<(
        &Text,
        &LocalizedText,
        &BankUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &UiTransform,
        &ChildOf,
    )>();
    let texts = texts
        .iter(world)
        .map(
            |(text, localized, style, font, layout, transform, parent)| {
                (
                    text,
                    localized,
                    style,
                    font,
                    layout,
                    transform,
                    world.get::<Node>(parent.parent()).unwrap(),
                )
            },
        )
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 274);
    assert!(
        texts
            .iter()
            .all(|(_, localized, _, _, _, _, _)| !localized.key.is_empty())
    );
    assert!(texts.iter().all(|(_, _, style, font, _, transform, _)| {
        font.0.font_size.eval(Vec2::ZERO, 16.0) == style.font_size()
            && (*font.1) == LineHeight::Px(style.line_height())
            && transform.translation == Val2::px(0.0, style.replacement_y_offset())
    }));

    let (_, title, title_style, title_font, title_layout, _, title_node) = texts
        .iter()
        .find(|(_, localized, _, _, _, _, _)| localized.key == "ui.bank.title")
        .copied()
        .unwrap();
    assert_eq!(title.fallback, "Morbucks Savings and Loan");
    assert_eq!(*title_style, BankUiTextStyle::LabelUpperLeft);
    assert_eq!(title_style.source_style_name(), "label");
    assert_eq!(title_style.source_font_path_id(), 977);
    assert_eq!(
        title_font.0.font_size.eval(Vec2::ZERO, 16.0),
        BANK_LABEL_FONT_SIZE
    );
    assert_eq!((*title_font.1), LineHeight::Px(BANK_LABEL_LINE_HEIGHT));
    assert_eq!(title_layout.linebreak, LineBreak::WordBoundary);
    assert_eq!(title_node.padding.left, px(0));
    assert_eq!(title_node.padding.right, px(0));
    assert_eq!(title_node.padding.top, px(BANK_LABEL_PADDING_TOP));
    assert_eq!(title_node.padding.bottom, px(BANK_LABEL_PADDING_BOTTOM));

    let (_, _, slot_style, slot_font, slot_layout, _, _) = texts
        .iter()
        .find(|(_, localized, _, _, _, _, _)| localized.key == "ui.inventory.slot.head")
        .copied()
        .unwrap();
    assert_eq!(*slot_style, BankUiTextStyle::EquipFontMiddleRight);
    assert_eq!(slot_style.source_style_name(), "equipfont");
    assert_eq!(slot_style.source_font_path_id(), 970);
    assert_eq!(
        slot_font.0.font_size.eval(Vec2::ZERO, 16.0),
        BANK_EQUIP_FONT_SIZE
    );
    assert_eq!((*slot_font.1), LineHeight::Px(BANK_EQUIP_LINE_HEIGHT));
    assert_eq!(slot_layout.justify, Justify::Right);
    assert_eq!(slot_layout.linebreak, LineBreak::NoWrap);

    let count_rows = texts
        .iter()
        .filter(|(_, localized, _, _, _, _, _)| localized.key == "ui.inventory.item.count")
        .collect::<Vec<_>>();
    assert_eq!(
        count_rows.len(),
        BANK_SLOT_COUNT_0104 + INVENTORY_SLOT_COUNT_0104
    );
    assert!(count_rows.iter().all(|(_, localized, style, _, _, _, _)| {
        **style == BankUiTextStyle::LabelUpperLeft
            && localized.fallback == "{count}"
            && localized.args.keys().map(String::as_str).eq(["count"])
    }));

    let taros_rows = texts
        .iter()
        .filter(|(_, localized, _, _, _, _, _)| localized.key == "ui.bank.taros.digit")
        .collect::<Vec<_>>();
    assert_eq!(taros_rows.len(), BANK_PC_STUFF_TAROS_DIGIT_RECTS.len());
    assert!(
        taros_rows
            .iter()
            .all(|(_, localized, style, font, layout, transform, node)| {
                **style == BankUiTextStyle::BlankBoxMiddleRight
                    && localized.fallback == "{digit}"
                    && localized.args.keys().map(String::as_str).eq(["digit"])
                    && font.0.font_size.eval(Vec2::ZERO, 16.0) == BANK_LABEL_FONT_SIZE
                    && (*font.1) == LineHeight::Px(BANK_LABEL_LINE_HEIGHT)
                    && layout.justify == Justify::Right
                    && layout.linebreak == LineBreak::NoWrap
                    && transform.translation == Val2::px(0.0, BANK_TEXT_REPLACEMENT_Y_OFFSET)
                    && node.padding == UiRect::ZERO
            })
    );

    let (_, redeem, redeem_style, redeem_font, redeem_layout, _, _) = texts
        .iter()
        .find(|(_, localized, _, _, _, _, _)| localized.key == "ui.enchant.redeem_code")
        .copied()
        .unwrap();
    assert_eq!(redeem.fallback, "REDEEM CODE");
    assert_eq!(*redeem_style, BankUiTextStyle::ButtonMiddleCenter);
    assert_eq!(redeem_style.source_style_name(), "button");
    assert_eq!(redeem_style.source_font_path_id(), 933);
    assert_eq!(
        redeem_font.0.font_size.eval(Vec2::ZERO, 16.0),
        BANK_BUTTON_FONT_SIZE
    );
    assert_eq!((*redeem_font.1), LineHeight::Px(BANK_BUTTON_LINE_HEIGHT));
    assert_eq!(redeem_layout.justify, Justify::Center);
    assert_eq!(redeem_layout.linebreak, LineBreak::NoWrap);

    let mut element_nodes = world.query::<(&BankUiElement, &Node)>();
    let (_, bank_count) = element_nodes
        .iter(world)
        .find(|(element, _)| **element == BankUiElement::BankSlotCount(0))
        .unwrap();
    assert_eq!(bank_count.left, px(BANK_COUNT_LABEL_LEFT));
    assert_eq!(bank_count.top, px(BANK_COUNT_LABEL_TOP));
    let (_, inventory_count) = element_nodes
        .iter(world)
        .find(|(element, _)| **element == BankUiElement::InventorySlotCount(0))
        .unwrap();
    assert_eq!(inventory_count.left, px(USER_EQUIP_COUNT_LABEL_LEFT));
    assert_eq!(inventory_count.top, px(USER_EQUIP_COUNT_LABEL_TOP));
    assert!(element_nodes.iter(world).all(
        |(element, _)| !matches!(element, BankUiElement::TarosDigit(index) if *index >= 9)
    ));
    assert_eq!(
        element_nodes
            .iter(world)
            .filter(|(element, _)| matches!(element, BankUiElement::TarosDigit(_)))
            .count(),
        9
    );
}

#[test]
fn clean_owner_draw_order_and_called_style_contract_are_explicit() {
    assert_eq!(BANK_GAME_OBJECT_PATH_ID, 1_279);
    assert_eq!(BANK_CONTROLLER_COMPONENT_PATH_ID, 1_586);
    assert_eq!(BANK_CONTROLLER_SCRIPT_PATH_ID, 919);
    assert_eq!(BANK_PANEL_COMPONENT_PATH_ID, 1_587);
    assert_eq!(BANK_PANEL_SCRIPT_PATH_ID, 1_162);
    assert_eq!(BANK_PC_STUFF_COMPONENT_PATH_ID, 1_588);
    assert_eq!(BANK_PC_STUFF_SCRIPT_PATH_ID, 1_045);
    assert_eq!(BANK_EQUIP_COMPONENT_PATH_ID, 1_589);
    assert_eq!(BANK_EQUIP_SCRIPT_PATH_ID, 1_027);
    assert_eq!(BANK_INVENTORY_MANAGER_COMPONENT_PATH_ID, 1_419);
    assert_eq!(BANK_INVENTORY_SKIN_PATH_ID, 1_366);
    assert_eq!(BANK_GUI_DEPTH_0104, 9);
    assert_eq!(BANK_UI_Z_INDEX, 19);

    assert_eq!(BANK_SLOT_BUTTON_SOURCE_PATH_ID, 640);
    assert_eq!(BANK_DEXLABS_SOURCE_PATH_ID, 451);
    assert_eq!(BANK_TAROS_COUNTER_SOURCE_PATH_ID, 326);
    assert_eq!(BANK_SLOT_BUTTON_BORDER.min_inset.x, 6.0);
    assert_eq!(BANK_SLOT_BUTTON_BORDER.max_inset.x, 6.0);
    assert_eq!(BANK_SLOT_BUTTON_BORDER.min_inset.y, 6.0);
    assert_eq!(BANK_SLOT_BUTTON_BORDER.max_inset.y, 4.0);
    assert_eq!(BANK_LABEL_SOURCE_FONT_PATH_ID, 977);
    assert_eq!(BANK_BUTTON_SOURCE_FONT_PATH_ID, 933);
    assert_eq!(BANK_EQUIP_SOURCE_FONT_PATH_ID, 970);
    assert_eq!(BANK_BACKGROUND_ONLY_FONT_PATH_ID, 949);
    assert_eq!(BankUiTextStyle::LabelUpperLeft.source_style_name(), "label");
    assert_eq!(
        BankUiTextStyle::BlankBoxUpperLeft.source_style_name(),
        "blankbox"
    );
    assert_eq!(
        BankUiTextStyle::BlankBoxMiddleRight.source_style_name(),
        "blankbox"
    );
    assert_eq!(
        BankUiTextStyle::ButtonMiddleCenter.source_style_name(),
        "button"
    );
    assert_eq!(
        BankUiTextStyle::EquipBarMiddleCenter.source_style_name(),
        "equipbar"
    );
    assert_eq!(
        BankUiTextStyle::EquipFontMiddleRight.source_style_name(),
        "equipfont"
    );
    assert_eq!(BankUiTextStyle::LabelUpperLeft.legacy_alignment(), 0);
    assert_eq!(BankUiTextStyle::BlankBoxUpperLeft.legacy_alignment(), 0);
    assert_eq!(BankUiTextStyle::BlankBoxMiddleRight.legacy_alignment(), 5);
    assert_eq!(BankUiTextStyle::ButtonMiddleCenter.legacy_alignment(), 4);
    assert_eq!(BankUiTextStyle::EquipBarMiddleCenter.legacy_alignment(), 4);
    assert_eq!(BankUiTextStyle::EquipFontMiddleRight.legacy_alignment(), 5);
    assert_eq!(
        BankUiTextStyle::LabelUpperLeft.padding(),
        [0.0, 0.0, 3.0, 3.0]
    );
    assert_eq!(
        BankUiTextStyle::ButtonMiddleCenter.padding(),
        [6.0, 6.0, 3.0, 3.0]
    );
    for style in [
        BankUiTextStyle::LabelUpperLeft,
        BankUiTextStyle::BlankBoxUpperLeft,
        BankUiTextStyle::BlankBoxMiddleRight,
        BankUiTextStyle::ButtonMiddleCenter,
        BankUiTextStyle::EquipBarMiddleCenter,
        BankUiTextStyle::EquipFontMiddleRight,
    ] {
        assert_eq!(style.content_offset(), [0.0, 0.0]);
        assert_eq!(style.word_wrap(), style == BankUiTextStyle::LabelUpperLeft);
        assert_eq!(style.replacement_y_offset(), 0.0);
    }
}

#[test]
fn published_bank_assets_match_expected_clean_hashes() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let asset_root = workspace_root.join("assets/game");
    let expected: BTreeMap<&str, &str> = [
        (
            BANK_TRADE_BACK_PATH,
            "DA8131CC458AED9D97E57C193B5B298134E7A967CC17EDF08769FFFE85A5D455",
        ),
        (
            BANK_PANEL_PATH,
            "9D3CFBF808DAE2DC51E11E003FD7F08039A618F08F5277E4CE6D355D9C9F09DE",
        ),
        (
            BANK_INFO_PATH,
            "343709111A7B3717F5D2B774BEE4385AD2AE26B8697070C4934AE0CBCE00996F",
        ),
        (
            BANK_LOCKED_SLOT_PATH,
            "4756B230D53760956B6CF81A88B32A5526E1A720AA0091FC2D965F3F829D42B3",
        ),
        (
            BANK_SCROLL_TRACK_PATH,
            "94CC317466DE205D6336038B99626A3596EA5592298ECDC409F4E93C7FB83355",
        ),
        (
            BANK_SCROLL_THUMB_PATH,
            "A7AFFF2D82DDC872CC99BC2695039530CE1E1A554DCEB95F33E344804EF589AD",
        ),
        (
            BANK_SCROLL_UP_PATH,
            "4B1A86AD7CD3D5A885F1E0137769D0DE9F3CA2226572573BF541E14278F660EE",
        ),
        (
            BANK_SCROLL_DOWN_PATH,
            "DE13BE5D83A1E28EBD312C1C38F293DC28126514F90E92124084E431B5A149FD",
        ),
        (
            BANK_SCROLL_SHADOW_PATH,
            "13DCD40705113B8DB6962A2CAE4F0EA69671A4D94A22D559A9768EB3ACFBB294",
        ),
    ]
    .into_iter()
    .collect();

    for (runtime_path, declared) in expected {
        let actual = Sha256::digest(fs::read(asset_root.join(runtime_path)).unwrap());
        assert_eq!(format!("{actual:X}"), declared);
    }
    for path in BANK_UI_DEFAULT_IMAGE_PATHS {
        assert!(asset_root.join(path).is_file(), "missing {path}");
    }
    assert!(asset_root.join(USER_EQUIP_FONT_PATH).is_file());

    let bank_button = fs::read(asset_root.join(BANK_SLOT_BUTTON_PATH)).unwrap();
    let raw = Sha256::digest(&bank_button);
    assert_eq!(format!("{raw:X}"), BANK_SLOT_BUTTON_SHA256);
    let rgba = image::load_from_memory(&bank_button).unwrap().to_rgba8();
    assert_eq!(rgba.dimensions(), (20, 25));
    let pixels = Sha256::digest(rgba.as_raw());
    assert_eq!(format!("{pixels:X}"), BANK_SLOT_BUTTON_RGBA_SHA256);

    for (path, expected_hash, expected_size) in [
        (BANK_DEXLABS_PATH, BANK_DEXLABS_SHA256, 11_819),
        (BANK_TAROS_COUNTER_PATH, BANK_TAROS_COUNTER_SHA256, 3_201),
    ] {
        let bytes = fs::read(asset_root.join(path)).unwrap();
        assert_eq!(bytes.len(), expected_size);
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)), expected_hash);
    }
}

#[test]
fn pc_stuff_taros_comes_only_from_authority_and_uses_the_clean_nine_digit_loop() {
    let mut load = PcLoadData0104::zeroed();
    load.as_bytes_mut()[PcLoadData0104::CANDY_OFFSET..PcLoadData0104::CANDY_OFFSET + 4]
        .copy_from_slice(&12_345_678i32.to_le_bytes());
    let mut authority = BankPcStuffAuthority0104::from_pc_load(&load);
    assert_eq!(authority.taros(), 12_345_678);
    assert_eq!(
        bank_taros_counter_digits_0104(authority.taros()),
        ["0", "1", "2", "3", "4", "5", "6", "7", "8"].map(str::to_owned)
    );
    assert_eq!(bank_taros_counter_digit_0104(12_345_678, 9), None);
    authority.replace_authoritative_taros(987_654_321);
    assert_eq!(
        bank_taros_counter_digits_0104(authority.taros()),
        ["9", "8", "7", "6", "5", "4", "3", "2", "1"].map(str::to_owned)
    );
}
