use std::{collections::BTreeSet, fs, path::Path};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::server_selection_ui::*;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {expected}, got {actual}"
    );
}

fn placeholders(template: &str) -> BTreeSet<&str> {
    let mut values = BTreeSet::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else {
            break;
        };
        values.insert(&rest[..close]);
        rest = &rest[close + 1..];
    }
    values
}

fn custom_style<'a>(skin: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    skin["customStyles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|style| style["name"] == name)
        .unwrap_or_else(|| panic!("missing FusionFallSkin style {name}"))
}

fn background_path(style: &serde_json::Value, state: &str) -> i64 {
    style["states"][state]["background"]["pathId"]
        .as_i64()
        .unwrap()
}

fn assert_edges(value: &serde_json::Value, left: i64, right: i64, top: i64, bottom: i64) {
    assert_eq!(value["left"], left);
    assert_eq!(value["right"], right);
    assert_eq!(value["top"], top);
    assert_eq!(value["bottom"], bottom);
}

fn init(character_count: i8) -> ServerSelectionInit {
    ServerSelectionInit {
        login: ServerSelectionLoginSnapshot {
            character_count,
            account_id: "DexLab".into(),
            ..default()
        },
        auto_login: true,
        warp_shard: true,
    }
}

fn open(character_count: i8) -> (ServerSelectionUiModel, ServerSelectionUiOutbox) {
    let mut model = ServerSelectionUiModel::default();
    let mut outbox = ServerSelectionUiOutbox::default();
    model.open(init(character_count), &mut outbox);
    (model, outbox)
}

#[test]
fn raw_root_skin_and_packet_boundaries_are_exact() {
    assert_eq!(SERVER_SELECTION_GAME_MODE_SLOT, 26);
    assert_eq!(SERVER_SELECTION_US_LOCALE_VALUE, 0);
    assert_eq!(SERVER_SELECTION_KOREA_LOCALE_VALUE, 2);
    assert!(!SERVER_SELECTION_DEFAULT_US_REACHABLE);
    assert!(SERVER_SELECTION_PARITY_CAVEAT.contains("bypasses GameMode 26"));
    assert_eq!(SERVER_SELECTION_ROOT_NAME, "ServerSelectionMode");
    assert_eq!(SERVER_SELECTION_ROOT_PATH_ID, 1_295);
    assert_eq!(SERVER_SELECTION_TRANSFORM_PATH_ID, 1_249);
    assert_eq!(SERVER_SELECTION_MODE_COMPONENT_PATH_ID, 1_536);
    assert_eq!(SERVER_SELECTION_GUI_COMPONENT_PATH_ID, 1_537);
    assert_eq!(SERVER_SELECTION_MODE_SCRIPT_PATH_ID, 1_040);
    assert_eq!(SERVER_SELECTION_GUI_SCRIPT_PATH_ID, 1_033);
    assert_eq!(SERVER_SELECTION_SKIN_PATH_ID, 1_374);
    assert!(!SERVER_SELECTION_ROOT_INITIALLY_ACTIVE);
    assert_eq!(SERVER_SELECTION_REQ_SHARD_LIST_PACKET_ID, 301_989_896);
    assert_eq!(SERVER_SELECTION_REP_SHARD_LIST_PACKET_ID, 553_648_153);
    assert_eq!(SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_ID, 301_989_902);
    assert_eq!(SERVER_SELECTION_REQ_SHARD_LIST_PACKET_SIZE, 1);
    assert_eq!(SERVER_SELECTION_REP_SHARD_LIST_PACKET_SIZE, 26);
    assert_eq!(SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_SIZE, 1);
    assert_eq!(SERVER_SELECTION_JEFFE_16_SOURCE_PATH_ID, 1_012);
    assert_eq!(SERVER_SELECTION_JEFFE_14_SOURCE_PATH_ID, 903);
    assert_eq!(SERVER_SELECTION_JEFFE_16_FONT_SIZE, 14.0);
    assert_eq!(SERVER_SELECTION_JEFFE_16_LINE_HEIGHT, 16.451_999_66);
    assert_eq!(SERVER_SELECTION_JEFFE_14_FONT_SIZE, 12.0);
    assert_eq!(SERVER_SELECTION_JEFFE_14_LINE_HEIGHT, 13.710_000_04);
    assert_eq!(SERVER_SELECTION_LABEL_TOP_PADDING, 3.0);
}

#[test]
fn clean_keyboard_focus_tab_and_external_call_contract_is_exact() {
    assert_eq!(
        SERVER_SELECTION_KEYBOARD_CONTRACT,
        ServerSelectionKeyboardContract {
            named_control_count: 0,
            accepts_text_input: false,
            tab_navigation: false,
            escape_handler: false,
        }
    );
    assert_eq!(SERVER_SELECTION_EXTERNAL_CALL_FUNCTION, "HomePage");
    assert_eq!(SERVER_SELECTION_EXTERNAL_CALL_ARGUMENT, "gameObject");
    assert_eq!(SERVER_SELECTION_EXTERNAL_CALL_ARGUMENT_COUNT, 1);
    assert!(
        !ServerSelectionUiModel::default()
            .input_boundary()
            .escape_enabled
    );
}

#[test]
fn localization_contract_is_unique_and_keeps_en_ru_placeholders_identical() {
    let keys = SERVER_SELECTION_LOCALIZATION_ENTRIES
        .iter()
        .map(|entry| entry.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(keys.len(), SERVER_SELECTION_LOCALIZATION_ENTRIES.len());
    assert!(
        keys.iter()
            .all(|key| key.starts_with("ui.server_selection."))
    );
    for entry in SERVER_SELECTION_LOCALIZATION_ENTRIES {
        assert_eq!(
            placeholders(entry.en),
            placeholders(entry.ru),
            "placeholder mismatch for {}",
            entry.key
        );
    }
}

#[test]
fn dynamic_server_and_channel_copy_uses_keyed_templates_and_arguments() {
    let collapsed = server_selection_server_heading_localized(false, 1);
    assert_eq!(collapsed.key, SERVER_SELECTION_SERVER_COLLAPSED_KEY);
    assert_eq!(collapsed.fallback, SERVER_SELECTION_COPY_SERVER_COLLAPSED);
    assert_eq!(collapsed.args.get("server").map(String::as_str), Some("1"));
    assert_eq!(
        localized_fallback(&collapsed),
        "+ Server 1 ----------------"
    );

    let expanded = server_selection_server_heading_localized(true, 1);
    assert_eq!(expanded.key, SERVER_SELECTION_SERVER_EXPANDED_KEY);
    assert_eq!(localized_fallback(&expanded), "- Server 1 ----------------");

    let channel = server_selection_channel_localized(25);
    assert_eq!(channel.key, SERVER_SELECTION_CHANNEL_ROW_KEY);
    assert_eq!(channel.fallback, SERVER_SELECTION_COPY_CHANNEL_ROW);
    assert_eq!(channel.args.get("channel").map(String::as_str), Some("25"));
    assert_eq!(localized_fallback(&channel), "- Channel 25  -------");

    assert_eq!(
        ServerSelectionPopulation::Busy.localized().key,
        SERVER_SELECTION_STATUS_BUSY_KEY
    );
}

#[test]
fn presentation_ecs_is_key_first_and_preserves_replacement_font_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(ServerSelectionUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut all_text =
        world.query_filtered::<(&Text, (&TextFont, &LineHeight), Option<&LocalizedText>), With<Text>>();
    let rows = all_text.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 58);
    assert!(rows.iter().all(|(_, _, localized)| localized.is_some()));
    let large = rows
        .iter()
        .filter(|(_, font, _)| {
            font.0.font_size.eval(Vec2::ZERO, 16.0) == SERVER_SELECTION_JEFFE_16_FONT_SIZE
        })
        .collect::<Vec<_>>();
    let small = rows
        .iter()
        .filter(|(_, font, _)| {
            font.0.font_size.eval(Vec2::ZERO, 16.0) == SERVER_SELECTION_JEFFE_14_FONT_SIZE
        })
        .collect::<Vec<_>>();
    assert_eq!(large.len(), 8);
    assert_eq!(small.len(), 50);
    assert!(large.iter().all(|(_, font, _)| {
        (*font.1) == LineHeight::Px(SERVER_SELECTION_JEFFE_16_LINE_HEIGHT)
    }));
    assert!(small.iter().all(|(_, font, _)| {
        (*font.1) == LineHeight::Px(SERVER_SELECTION_JEFFE_14_LINE_HEIGHT)
    }));
    assert!(rows.iter().any(|(text, _, localized)| {
        let localized = localized.unwrap();
        text.0 == "- Channel 25  -------"
            && localized.key == SERVER_SELECTION_CHANNEL_ROW_KEY
            && localized.args.get("channel").map(String::as_str) == Some("25")
    }));
}

#[test]
fn source_audit_rejects_raw_text_literals_at_spawn_sites() {
    let source = concat!(
        include_str!("state.rs"),
        "\n",
        include_str!("localization_server_selection_localization_en.rs"),
        "\n",
        include_str!("assets.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("frame.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("output.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("audio.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("mod.rs")
    );
    let production = source.split("#[cfg(test)]").next().unwrap();
    let raw_literal_spawn = ["Text::new(", "\""].concat();
    assert!(!production.contains(&raw_literal_spawn));
    assert!(production.contains("LocalizedText::new"));
    assert!(production.contains("before(LocalizationSet::Apply)"));
}

#[test]
fn exact_1264x681_geometry_uses_integer_halves_and_scale_to_fit() {
    let layout = ServerSelectionUiLayout::from_viewport(Vec2::new(1_264.0, 681.0), 0.0, true);
    assert_eq!(layout.viewport, Vec2::new(1_264.0, 681.0));
    assert_eq!(
        layout.panel,
        ServerSelectionUiRect::new(705.0, 180.0, 375.0, 330.0)
    );
    assert_eq!(
        layout.account,
        ServerSelectionUiRect::new(792.0, 520.0, 200.0, 27.0)
    );
    assert_eq!(
        layout.homepage,
        ServerSelectionUiRect::new(792.0, 552.0, 200.0, 27.0)
    );
    assert_eq!(
        layout.quit,
        ServerSelectionUiRect::new(830.0, 584.0, 125.0, 27.0)
    );
    assert_close(layout.background.top, 0.0);
    assert_close(layout.background.height, 681.0);
    assert_close(layout.background.width, 1_020.0 * 681.0 / 638.0);
    assert_close(
        layout.background.left,
        (1_264.0 - layout.background.width) * 0.5,
    );
    assert_eq!(
        SERVER_SELECTION_VIEWPORT_RECT,
        ServerSelectionUiRect::new(37.0, 50.0, 293.0, 227.0)
    );
    assert_eq!(
        SERVER_SELECTION_CONNECT_RECT,
        ServerSelectionUiRect::new(113.0, 290.0, 135.0, 28.0)
    );
    assert_eq!(
        SERVER_SELECTION_CHANNEL_STATUS_RECT,
        ServerSelectionUiRect::new(168.0, 0.0, 45.0, 16.0)
    );
}

#[test]
fn open_requests_list_immediately_and_refresh_is_strictly_greater_than_sixty() {
    let (mut model, mut outbox) = open(1);
    assert_eq!(
        outbox.pop_front(),
        Some(ServerSelectionUiEffect::RequestShardList {
            packet_id: SERVER_SELECTION_REQ_SHARD_LIST_PACKET_ID,
            packet_size: 1,
        })
    );
    model.tick(60.0, &mut outbox);
    assert!(outbox.is_empty());
    model.tick(0.000_1, &mut outbox);
    assert_eq!(outbox.len(), 1);
    assert_eq!(model.refresh_timer, 0.0);
    outbox.clear();
    model.tick(180.0, &mut outbox);
    assert_eq!(outbox.len(), 1, "clean code does not catch up intervals");
}

#[test]
fn shard_reply_ignores_index_zero_and_accepts_only_exact_packet_id() {
    let (mut model, mut outbox) = open(1);
    outbox.clear();
    model.shard_statuses[0] = 3;
    let mut payload = [0_u8; 26];
    payload[0] = 1;
    for (index, value) in payload.iter_mut().enumerate().skip(1) {
        *value = (index % 4) as u8;
    }
    assert!(!model.receive_shard_list(0xDEAD_BEEF, payload));
    assert_eq!(model.shard_statuses[1], 0);
    assert!(model.receive_shard_list(SERVER_SELECTION_REP_SHARD_LIST_PACKET_ID, payload));
    assert_eq!(model.shard_statuses[0], 3);
    assert_eq!(model.shard_statuses[1], 1);
    assert_eq!(model.shard_statuses[25], 1);
}

#[test]
fn awake_and_gui_pass_scroll_height_quirks_are_retained() {
    let (mut model, mut outbox) = open(1);
    outbox.clear();
    assert_eq!(model.scroll_view_height, 402.0);
    model.complete_gui_pass();
    assert_eq!(model.scroll_view_height, 30.0);
    model.toggle_server();
    assert_eq!(model.scroll_view_height, 492.0);
    assert_eq!(model.maximum_scroll(), 265.0);
    model.scroll_by(9_999.0);
    assert_eq!(model.scroll_y, 265.0);
    model.toggle_server();
    assert_eq!(model.scroll_y, 0.0);
}

#[test]
fn no_selection_opens_message_250_and_closed_channel_remains_selectable() {
    let (mut model, mut outbox) = open(1);
    outbox.clear();
    assert!(!model.connect(&mut outbox));
    assert_eq!(
        outbox.pop_front(),
        Some(ServerSelectionUiEffect::SystemMessage { message_id: 250 })
    );
    model.shard_statuses[7] = ServerSelectionPopulation::Closed as u8;
    assert!(model.select_shard(1, 7));
    assert_eq!(model.selected_shard, 7);
}

#[test]
fn existing_character_connect_order_and_shard_side_channel_are_exact() {
    let (mut model, mut outbox) = open(1);
    outbox.clear();
    model.select_shard(1, 12);
    assert!(model.connect(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            ServerSelectionUiEffect::SetGameMode(2),
            ServerSelectionUiEffect::InitCharacterSelection {
                login: init(1).login,
                account_id: "DexLab".into(),
                shard: 12,
                zero: 0,
                auto_login: true,
                warp_shard: true,
            },
            ServerSelectionUiEffect::SendServerSelect {
                server_number: 1,
                packet_id: SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_ID,
                packet_size: 1,
            },
        ]
    );
    // Wire request contains only server number; selected shard belongs to
    // the CharacterSelection init event.
    assert_eq!(SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_SIZE, 1);
}

#[test]
fn zero_character_branch_inverts_auto_login_and_preserves_effect_order() {
    let (mut model, mut outbox) = open(0);
    outbox.clear();
    model.select_shard(1, 3);
    assert!(model.connect(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            ServerSelectionUiEffect::InitCharacterCreation {
                mode: 2,
                login: init(0).login,
                account_id: "DexLab".into(),
                shard: 3,
                zero: 0,
                inverted_auto_login: false,
                warp_shard: false,
            },
            ServerSelectionUiEffect::ApplySoundOptions,
            ServerSelectionUiEffect::SetGameMode(21),
            ServerSelectionUiEffect::InitDexterNameCreate {
                character_id: 0,
                event_scene_name: "NameCreate",
                slot_number: 1,
            },
            ServerSelectionUiEffect::LegacyEvent {
                manager: 9,
                function: 6,
                element_function: None,
            },
            ServerSelectionUiEffect::SendServerSelect {
                server_number: 1,
                packet_id: SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_ID,
                packet_size: 1,
            },
        ]
    );
}

#[test]
fn account_homepage_escape_and_audio_contract_preserve_clean_silence() {
    let (model, mut outbox) = open(1);
    outbox.clear();
    model.open_web(ServerSelectionWebSource::MyAccount, &mut outbox);
    model.open_web(ServerSelectionWebSource::Homepage, &mut outbox);
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            ServerSelectionUiEffect::OpenUrl {
                source: ServerSelectionWebSource::MyAccount,
                url: SERVER_SELECTION_URL,
            },
            ServerSelectionUiEffect::OpenUrl {
                source: ServerSelectionWebSource::Homepage,
                url: SERVER_SELECTION_URL,
            },
        ]
    );
    assert!(!model.press_escape());
    assert!(!model.input_boundary().escape_enabled);
    assert!(SERVER_SELECTION_AUDIO_ROUTES.is_empty());
}

#[test]
fn quit_gate_rejection_and_non_editor_acceptance_are_exact() {
    let (mut model, mut outbox) = open(1);
    outbox.clear();
    assert!(model.request_quit(&mut outbox));
    assert_eq!(
        outbox.pop_front(),
        Some(ServerSelectionUiEffect::RequestQuitGate {
            manager: 3,
            function: 1,
        })
    );
    assert!(!model.resolve_quit_gate(1, false, &mut outbox));
    assert_eq!(model.phase, ServerSelectionPhase::Visible);
    model.request_quit(&mut outbox);
    outbox.clear();
    assert!(model.resolve_quit_gate(0, false, &mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            ServerSelectionUiEffect::SetGameMode(1),
            ServerSelectionUiEffect::LegacyEvent {
                manager: 2,
                function: 3,
                element_function: Some(8),
            },
            ServerSelectionUiEffect::SetSystemFocusOut(true),
            ServerSelectionUiEffect::LegacyEvent {
                manager: 9,
                function: 6,
                element_function: None,
            },
            ServerSelectionUiEffect::SkipWorldUpdate(true),
            ServerSelectionUiEffect::SetSaveResolution(false),
            ServerSelectionUiEffect::ExternalCallHomePage,
        ]
    );
}

#[test]
fn fusion_fall_skin_used_style_metrics_are_exact() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ledger = workspace
        .join("assets/game")
        .join(SERVER_SELECTION_SKIN_LEDGER_PATH);
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&ledger).unwrap()).unwrap();
    let skin = value["skins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|skin| skin["pathId"] == SERVER_SELECTION_SKIN_PATH_ID)
        .unwrap();
    assert_eq!(skin["name"], "FusionFallSkin");
    assert_eq!(
        skin["font"]["pathId"],
        SERVER_SELECTION_JEFFE_16_SOURCE_PATH_ID
    );

    let label = &skin["builtInStyles"]["label"];
    assert_eq!(label["font"]["pathId"], 1_012);
    assert_eq!(background_path(label, "normal"), 0);
    assert_edges(&label["margin"], 4, 4, 4, 4);
    assert_edges(&label["padding"], 0, 0, 3, 3);
    assert_eq!(label["alignment"], 0);
    assert_eq!(label["wordWrap"], 1);
    assert_eq!(label["textClipping"], 1);

    let button = &skin["builtInStyles"]["button"];
    assert_eq!(button["font"]["pathId"], 0);
    assert_eq!(background_path(button, "normal"), 411);
    assert_eq!(background_path(button, "hover"), 320);
    assert_eq!(background_path(button, "active"), 567);
    assert_edges(&button["border"], 5, 5, 5, 5);
    assert_edges(&button["margin"], 4, 4, 4, 4);
    assert_edges(&button["padding"], 10, 6, 3, 6);
    assert_eq!(button["alignment"], 4);
    assert_eq!(button["wordWrap"], 0);
    assert_eq!(
        button["states"]["normal"]["textColor"]["r"]
            .as_f64()
            .unwrap(),
        0.899_999_976_158_142_1
    );

    let ss_box = custom_style(skin, "SSBox");
    assert_eq!(ss_box["font"]["pathId"], 1_066);
    assert_eq!(background_path(ss_box, "normal"), 434);
    assert_edges(&ss_box["border"], 0, 0, 0, 0);
    assert_eq!(ss_box["imagePosition"], 3);
    assert_eq!(ss_box["alignment"], 3);

    let ss_inside = custom_style(skin, "SSBoxInside");
    assert_eq!(ss_inside["font"]["pathId"], 1_066);
    assert_eq!(background_path(ss_inside, "normal"), 328);
    assert_edges(&ss_inside["padding"], 0, 0, 0, 0);

    let server_button = custom_style(skin, "SSButtonServer");
    assert_eq!(server_button["font"]["pathId"], 903);
    assert_eq!(background_path(server_button, "normal"), 0);
    assert_eq!(background_path(server_button, "hover"), 367);
    assert_eq!(background_path(server_button, "active"), 367);
    assert_edges(&server_button["padding"], 0, 0, 0, 0);
    assert_eq!(server_button["alignment"], 3);

    let font_16 = custom_style(skin, "Font16");
    assert_eq!(font_16["font"]["pathId"], 1_012);
    assert_eq!(background_path(font_16, "normal"), 0);
    assert_eq!(background_path(font_16, "hover"), 367);
    assert_eq!(background_path(font_16, "active"), 543);
    assert_edges(&font_16["padding"], 0, 0, 0, 0);
    assert_eq!(font_16["alignment"], 3);

    let font_14 = custom_style(skin, "Font14");
    assert_eq!(font_14["font"]["pathId"], 903);
    assert_eq!(background_path(font_14, "normal"), 0);
    assert_eq!(background_path(font_14, "hover"), 367);
    assert_eq!(background_path(font_14, "active"), 543);
    assert_edges(&font_14["padding"], 0, 0, 0, 0);
    assert_eq!(font_14["alignment"], 3);

    let selected = custom_style(skin, "SSButtonSelected");
    assert_eq!(selected["font"]["pathId"], 903);
    assert_eq!(background_path(selected, "normal"), 543);

    let red = custom_style(skin, "RedButton");
    assert_eq!(red["font"]["pathId"], 0);
    assert_eq!(background_path(red, "normal"), 282);
    assert_eq!(background_path(red, "hover"), 190);
    assert_eq!(background_path(red, "active"), 282);
    assert_edges(&red["border"], 5, 5, 5, 5);
    assert_edges(&red["margin"], 0, 0, 0, 0);
    assert_edges(&red["padding"], 0, 0, 0, 0);
    assert_eq!(red["alignment"], 4);
    assert_eq!(red["wordWrap"], 1);
    assert_eq!(red["states"]["active"]["textColor"]["r"], 0.0);
    assert_eq!(
        legacy_button_text_color(LegacyButtonKind::Red, Interaction::Pressed),
        Color::BLACK
    );

    let scrollbar = &skin["builtInStyles"]["verticalScrollbar"];
    assert_eq!(background_path(scrollbar, "normal"), 374);
    assert_edges(&scrollbar["border"], 2, 2, 4, 4);
    assert_edges(&scrollbar["overflow"], 0, 1, 0, 0);
    assert_eq!(scrollbar["fixedWidth"], 16.0);

    let thumb = &skin["builtInStyles"]["verticalScrollbarThumb"];
    assert_eq!(background_path(thumb, "normal"), 324);
    assert_edges(&thumb["border"], 6, 6, 6, 6);
    assert_edges(&thumb["padding"], 0, 0, 6, 6);
    assert_edges(&thumb["overflow"], -1, -1, 0, 0);
    assert_eq!(thumb["fixedWidth"], 15.0);

    let up = &skin["builtInStyles"]["verticalScrollbarUpButton"];
    assert_eq!(background_path(up, "normal"), 63);
    assert_eq!(up["fixedHeight"], 12.0);
    let down = &skin["builtInStyles"]["verticalScrollbarDownButton"];
    assert_eq!(background_path(down, "normal"), 415);
    assert_eq!(down["fixedHeight"], 12.0);
}

#[test]
fn exact_pngs_are_present() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for asset in SERVER_SELECTION_ASSET_CONTRACTS {
        let path = workspace.join("assets/game").join(asset.path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert_eq!(bytes.len() as u64, asset.png_size, "{}", path.display());
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)), asset.png_sha256);
        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .unwrap_or_else(|error| panic!("failed to decode {}: {error}", path.display()));
        assert_eq!(image.width(), asset.source_width);
        assert_eq!(image.height(), asset.source_height);
        assert_eq!(asset.source_payload_sha256.len(), 64);
    }
}
