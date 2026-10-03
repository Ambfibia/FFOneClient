use super::*;

#[cfg(test)]
pub(super) const fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

#[cfg(test)]
pub(super) fn one_row_projection(item_type: i16) -> CashmallModeProjection0104 {
    CashmallModeProjection0104 {
        player_inventory: vec![inventory_item(
            CASHMALL_SLOT_TYPE,
            0,
            item(item_type, 77, 5),
            "Retained item",
            14,
            250,
            Some("icons/items/weapons/wpnicon_01.png"),
            Some(true),
        )],
        taros: 1_000,
        ..Default::default()
    }
}

#[cfg(test)]
pub(super) fn spawned_cashmall_app() -> (TempDir, App) {
    let asset_root = tempdir().expect("temporary Cash Mall asset root");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(CashmallUiPlugin0104);
    app.update();
    (asset_root, app)
}

#[test]
fn five_tabs_keep_clean_visual_and_label_asymmetries() {
    for tab in CashmallTab0104::ALL {
        let selected = cashmall_tab_view_0104(tab, tab, true, true);
        assert_eq!(selected.visual, CashmallTabVisual0104::Selected);
        assert_eq!(selected.label, tab.legacy_name());
        assert_eq!(
            selected.label_source,
            CashmallTabLabelSource0104::RawEnumName
        );
        assert!(!selected.pressed_visual_is_distinct);

        let inactive = cashmall_tab_view_0104(CashmallTab0104::New, tab, false, false);
        if tab != CashmallTab0104::New {
            assert_eq!(inactive.visual, CashmallTabVisual0104::Normal);
            assert_eq!(
                inactive.label_source,
                CashmallTabLabelSource0104::LocalizedKey
            );
            assert_eq!(
                cashmall_tab_view_0104(CashmallTab0104::New, tab, true, true).visual,
                CashmallTabVisual0104::Hover
            );
        }
    }
    assert_eq!(
        cashmall_tab_asset_role_0104(CashmallTab0104::New, CashmallTabVisual0104::Selected),
        CashmallStaticAssetRole0104::FirstSelected
    );
    assert_eq!(
        cashmall_tab_asset_role_0104(CashmallTab0104::Etc, CashmallTabVisual0104::Hover),
        CashmallStaticAssetRole0104::SecondHover
    );
}

#[test]
fn every_tab_projects_the_same_fixed_slot9_scan() {
    let projection = CashmallModeProjection0104 {
        player_inventory: vec![
            inventory_item(9, 0, item(7, 10, 4), "Potion", 1, 25, None, None),
            inventory_item(9, 19, item(0, 11, 0), "Weapon", 3, 50, None, Some(true)),
        ],
        taros: 100,
        ..Default::default()
    };
    let baseline = projection.rows_for_tab(CashmallTab0104::New);
    for tab in CashmallTab0104::ALL {
        assert_eq!(projection.rows_for_tab(tab), baseline);
    }
    assert_eq!(baseline.len(), 2);
    assert!(!CASHMALL_TAB_FILTERING_REACHABLE);
}

#[test]
fn slot9_scan_is_ordered_limited_and_preserves_clean_row_quirks() {
    let entries = vec![
        inventory_item(9, 3, item(7, 103, 7), "Third", 9, 500, None, None),
        inventory_item(8, 1, item(0, 999, 0), "Wrong type", 99, 1, None, Some(true)),
        inventory_item(
            9,
            1,
            item(0, 101, 0),
            "First duplicate wins",
            4,
            80,
            Some("icons/items/weapons/wpnicon_01.png"),
            Some(false),
        ),
        inventory_item(
            9,
            1,
            item(0, 102, 0),
            "Ignored duplicate",
            5,
            80,
            None,
            Some(true),
        ),
        inventory_item(
            9,
            20,
            item(0, 120, 0),
            "Outside scan",
            6,
            80,
            None,
            Some(true),
        ),
    ];
    let rows = cashmall_scan_slot9_0104(&entries, 100);
    assert_eq!(
        rows.iter().map(|row| row.scan_slot_id).collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert_eq!(rows[0].source.name, "First duplicate wins");
    assert_eq!(
        rows[0].frame_visual,
        CashmallSlotFrameVisual0104::Restricted
    );
    assert_eq!(rows[0].frame_alpha_percent, 100);
    assert_eq!(rows[0].icon_alpha_percent, 100);
    assert_eq!(rows[0].level_label, "LEVEL 4");
    assert!(!rows[0].price_text_visible);
    assert_eq!(rows[1].frame_visual, CashmallSlotFrameVisual0104::Normal);
    assert_eq!(rows[1].frame_alpha_percent, 40);
    assert_eq!(rows[1].icon, CashmallPresentationIcon0104::MissingChecker);
}

#[test]
fn go_to_my_stuff_boundary_is_exact_and_hides_without_a_close_effect() {
    let (mut state, mut outbox) = visible_state(false);
    state
        .request_go_to_stuff(Default::default(), &mut outbox)
        .unwrap();
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Hidden);
    assert!(!state.ui_input_active());
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![CashmallLocalEffect0104::GoToMyStuff(
            CashmallGoToStuffBoundary0104 {
                close_inventory_event: [2, 3, 5],
                request_game_mode_event: [2, 0],
                next_game_mode: 6,
                receive_init_event: [2, 3, 0],
                next_mode_init_argument: 2,
                first_use_condition: 3,
                final_refresh_event: [11, 18],
            }
        )]
    );
}

#[test]
fn close_and_modal_gates_follow_the_two_legacy_panels() {
    let (mut state, mut outbox) = visible_state(false);
    let modal = CashmallModalState0104 {
        generic_popup_active: true,
        ..Default::default()
    };
    let controls = state.input_capabilities(modal);
    assert!(!controls.cashmall_controls);
    assert!(controls.update_scroll);
    assert_eq!(
        state.select_tab(modal, CashmallTab0104::Scroll, &mut outbox),
        Err(CashmallActionBlocked0104::ControlsDisabled)
    );
    assert_eq!(
        state.request_close(
            CashmallCloseSource0104::ConfigurableKey4,
            Default::default(),
            CashmallCloseGate0104 {
                mode_accepts_escape: false,
                exit_arbitration_clear: true,
            },
            &mut outbox,
        ),
        Err(CashmallActionBlocked0104::ModeRejectedEscape)
    );
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Visible);
    assert_eq!(
        state.request_close(
            CashmallCloseSource0104::ConfigurableKey4,
            Default::default(),
            CashmallCloseGate0104 {
                mode_accepts_escape: true,
                exit_arbitration_clear: false,
            },
            &mut outbox,
        ),
        Err(CashmallActionBlocked0104::ExitArbitrationRejected)
    );
}

#[test]
fn dead_help_receiver_and_retained_clean_bugs_are_explicit() {
    let (state, mut outbox) = visible_state(false);
    state.request_help(Default::default(), &mut outbox).unwrap();
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            CashmallLocalEffect0104::PlayAudio(CashmallAudioCue0104::ButtonSound),
            CashmallLocalEffect0104::DeadHelpSendMessage,
        ]
    );
    assert!(!CASHMALL_HELP_RECEIVER_PRESENT);
    assert!(!CASHMALL_SET_VENDOR_ITEM_REACHABLE);
    assert_eq!(CASHMALL_CACHED_ITEM_COUNT_0104, 0);
    assert_eq!(CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT, 0.0);
    assert!(!CASHMALL_ITEM_BAR_ASSIGNED);
    assert!(!CASHMALL_INVENTORY_MODE_INITIALIZATION_REACHABLE);
    assert!(!CASHMALL_NPC_ASSIGNMENT_FROM_CHAT_ENTRY_REACHABLE);
    assert!(!CASHMALL_FREE_ASSETS_CLEARS_LOADED_TEXTURES);
}

#[test]
fn all_native_static_assets_exist_and_cashmall_exports_match_declared_hashes() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let contract = CashmallUiAssetContract0104::default();
    for path in contract.paths() {
        assert!(
            asset_root.join(path).is_file(),
            "missing native asset {path}"
        );
    }
    for evidence in CASHMALL_SOURCE_TEXTURES_0104 {
        let bytes = fs::read(asset_root.join(evidence.runtime_path)).unwrap();
        assert_eq!(bytes.len(), evidence.png_bytes, "{}", evidence.runtime_path);
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            evidence.width,
            "{} width",
            evidence.runtime_path
        );
        assert_eq!(
            u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
            evidence.height,
            "{} height",
            evidence.runtime_path
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)).to_uppercase(),
            evidence.png_sha256,
            "{}",
            evidence.runtime_path
        );
    }
}

#[test]
fn exact_text_styles_preserve_source_alignment_padding_and_replacement_metrics() {
    let cases = [
        (
            CashmallTextStyle0104::LabelUpperLeft,
            977,
            12.0,
            13.560_000_42,
            0,
        ),
        (
            CashmallTextStyle0104::BlankBoxUpperLeft,
            977,
            12.0,
            13.560_000_42,
            0,
        ),
        (
            CashmallTextStyle0104::BlankBoxMiddleRight,
            977,
            12.0,
            13.560_000_42,
            5,
        ),
        (
            CashmallTextStyle0104::ButtonMiddleCenter,
            933,
            11.0,
            11.300_000_19,
            4,
        ),
        (
            CashmallTextStyle0104::EquipBarMiddleCenter,
            970,
            7.0,
            6.780_000_21,
            4,
        ),
        (
            CashmallTextStyle0104::EquipFontMiddleRight,
            970,
            7.0,
            6.780_000_21,
            5,
        ),
    ];
    for (style, source_font, size, line_height, alignment) in cases {
        assert_eq!(style.source_font_path_id(), source_font);
        assert_eq!(style.font_size(), size);
        assert_eq!(style.line_height(), line_height);
        assert_eq!(style.legacy_alignment(), alignment);
        assert_eq!(style.content_offset(), [0.0, 0.0]);
    }
    assert_eq!(
        CashmallTextStyle0104::LabelUpperLeft.padding(),
        [0.0, 0.0, 3.0, 3.0]
    );
    assert_eq!(
        CashmallTextStyle0104::ButtonMiddleCenter.padding(),
        [6.0, 6.0, 3.0, 3.0]
    );
    assert_eq!(
        CashmallTextStyle0104::BlankBoxMiddleRight.padding(),
        [0.0; 4]
    );
    assert_eq!(CASHMALL_REPLACEMENT_FONT_Y_OFFSET, 0.0);
}

#[test]
fn taros_counter_keeps_nine_individual_middle_right_digits() {
    let digits = (0..9)
        .map(|index| {
            cashmall_taros_counter_digit_0104(1_500, index)
                .unwrap()
                .to_string()
        })
        .collect::<String>();
    assert_eq!(digits, "000001500");
    assert_eq!(cashmall_taros_counter_digit_0104(1_500, 9), None);
    for (index, rect) in CASHMALL_PC_STUFF_TAROS_DIGIT_RECTS.into_iter().enumerate() {
        assert_eq!(
            rect,
            CashmallUiRect0104::new(22.0 + index as f32 * 12.0, 564.0, 12.0, 20.0)
        );
    }
}

#[test]
fn ecs_every_text_is_key_first_and_carries_an_exact_source_style() {
    let (_asset_root, mut app) = spawned_cashmall_app();
    let world = app.world_mut();
    let total = world.query::<&Text>().iter(world).count();
    let mut query = world.query::<(
        &Text,
        &LocalizedText,
        &CashmallTextStyle0104,
        (&TextFont, &LineHeight),
    )>();
    let rows = query
        .iter(world)
        .map(|(text, localized, style, font)| {
            (
                text.clone(),
                localized.clone(),
                *style,
                (font.0.clone(), *font.1),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), total);
    assert!(total > 100);
    let expected_font = rows[0].3.0.font.clone();
    for (_text, localized, style, font) in rows {
        assert!(!localized.key.is_empty());
        assert_eq!(font.0.font, expected_font);
        assert_eq!(font.0.font_size, style.font_size().into());
        assert_eq!(font.1, LineHeight::Px(style.line_height()));
    }
}

#[test]
fn ecs_representative_text_nodes_keep_upper_left_middle_right_and_padding() {
    let (_asset_root, mut app) = spawned_cashmall_app();
    let world = app.world_mut();
    let mut query = world.query::<(&CashmallUiElement0104, &CashmallTextStyle0104, &Node)>();
    let rows = query.iter(world).collect::<Vec<_>>();
    let title = rows
        .iter()
        .find(|(element, _, _)| **element == CashmallUiElement0104::Title)
        .unwrap();
    assert_eq!(*title.1, CashmallTextStyle0104::LabelUpperLeft);
    assert_eq!(title.2.justify_content, JustifyContent::FlexStart);
    assert_eq!(title.2.align_items, AlignItems::FlexStart);
    assert_eq!(title.2.padding.top, px(3));
    let cash = rows
        .iter()
        .find(|(element, _, _)| **element == CashmallUiElement0104::CashDigit(0))
        .unwrap();
    assert_eq!(*cash.1, CashmallTextStyle0104::BlankBoxMiddleRight);
    assert_eq!(cash.2.justify_content, JustifyContent::FlexEnd);
    assert_eq!(cash.2.align_items, AlignItems::Center);
    let equip = rows
        .iter()
        .find(|(element, _, _)| **element == CashmallUiElement0104::EquipmentSlotLabel(0))
        .unwrap();
    assert_eq!(*equip.1, CashmallTextStyle0104::EquipFontMiddleRight);
    assert_eq!(equip.2.justify_content, JustifyContent::FlexEnd);
}

#[test]
fn ecs_tab_images_are_sliced_and_shared_panels_draw_above_cashmall_depth10() {
    let (_asset_root, mut app) = spawned_cashmall_app();
    let world = app.world_mut();
    let mut image_query = world.query::<(&CashmallUiElement0104, &ImageNode)>();
    for (element, image) in image_query.iter(world) {
        let CashmallUiElement0104::TabVisual(tab) = *element else {
            continue;
        };
        let NodeImageMode::Sliced(slicer) = &image.image_mode else {
            panic!("tab {tab:?} was stretched")
        };
        assert_eq!(slicer.border, cashmall_tab_border_0104(tab));
    }
    let mut z_query = world.query::<(&CashmallUiElement0104, &ZIndex)>();
    let z = z_query
        .iter(world)
        .map(|(element, z)| (*element, z.0))
        .collect::<Vec<_>>();
    assert_eq!(
        z.iter()
            .find(|(element, _)| *element == CashmallUiElement0104::CashmallPanel)
            .unwrap()
            .1,
        0
    );
    assert_eq!(
        z.iter()
            .find(|(element, _)| *element == CashmallUiElement0104::PcStuffPanel)
            .unwrap()
            .1,
        1
    );
    assert_eq!(
        z.iter()
            .find(|(element, _)| *element == CashmallUiElement0104::EquipmentPanel)
            .unwrap()
            .1,
        1
    );
}
