use std::{fs, path::Path};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::rule_ui::*;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn clean_mode_root_component_skin_and_raw_archives_are_explicit() {
    assert_eq!(RULE_UI_GAME_MODE_VALUE, 30);
    assert_eq!(RULE_UI_GAME_OBJECT_PATH_ID, 1_278);
    assert_eq!(RULE_UI_COMPONENT_PATH_ID, 1_616);
    assert_eq!(RULE_UI_SCRIPT_PATH_ID, 1_019);
    assert_eq!(RULE_UI_SKIN_PATH_ID, 1_381);
    assert!(!RULE_UI_ROOT_INITIALLY_ACTIVE);
    assert_eq!(RULE_UI_ROOT_FONT_EXTERNAL_FILE_ID, 1);
    assert_eq!(RULE_UI_ROOT_FONT_EXTERNAL_PATH_ID, 10_102);
    assert_eq!(RULE_UI_JEFFE_12_FONT_PATH_ID, 953);
    assert_eq!(RULE_UI_JEFFE_16_FONT_PATH_ID, 1_012);
    assert_eq!(RULE_UI_CHALET_SMALL_FONT_PATH_ID, 1_018);
    assert_eq!(RULE_UI_INVENTORY_SKIN_PATH_ID, 1_366);
    assert_eq!(RULE_UI_CLOSE_NORMAL_TEXTURE_PATH_ID, 105);
    assert_eq!(RULE_UI_CLOSE_HOVER_TEXTURE_PATH_ID, 157);
    for hash in [
        RULE_UI_MAIN_SHA256,
        RULE_UI_TUTORIAL_SHA256,
        RULE_UI_CHARACTER_CREATION_SHA256,
        RULE_UI_ICONS_SHA256,
    ] {
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    assert_eq!(RULE_UI_PARITY_STATUS, "partial");
    assert!(RULE_UI_PARITY_CAVEAT.contains("normalized clean-primary golden"));
}

#[test]
fn clean_rules_table_has_only_the_two_reachable_unlinked_pages() {
    assert_eq!(
        RulePageId::CLEAN_ORDER,
        [RulePageId::Vehicle, RulePageId::Combining]
    );
    assert_eq!(RULE_UI_PAGES.len(), 2);

    let vehicle = RulePageId::Vehicle.spec();
    assert_eq!((vehicle.string_start, vehicle.image_start), (1, 8));
    assert_eq!((vehicle.previous, vehicle.next), (None, None));
    assert_eq!(vehicle.strings, RULE_UI_VEHICLE_STRINGS);
    assert_eq!(vehicle.images, RULE_UI_VEHICLE_IMAGE_PATHS);
    assert_eq!(vehicle.strings[0], "WHAT ARE VEHICLES?");
    assert!(vehicle.strings[5].contains("hit the \"V\" key."));
    assert!(vehicle.strings[1].ends_with(' '));

    let combining = RulePageId::Combining.spec();
    assert_eq!((combining.string_start, combining.image_start), (12, 19));
    assert_eq!((combining.previous, combining.next), (None, None));
    assert_eq!(combining.strings, RULE_UI_COMBINING_STRINGS);
    assert_eq!(combining.images, RULE_UI_COMBINE_IMAGE_PATHS);
    assert_eq!(combining.strings[0], "WHAT IS COMBINING?");
    assert!(combining.strings[5].starts_with('\n'));
    assert!(combining.strings[5].contains("not 100% guaranteed.  If"));
    assert!(combining.strings[3].ends_with(' '));

    assert_eq!(RulePageId::try_from(1), Ok(RulePageId::Vehicle));
    assert_eq!(RulePageId::try_from(2), Ok(RulePageId::Combining));
    assert_eq!(RulePageId::try_from(0), Err(RulePageIndexError(0)));
    assert_eq!(RulePageId::try_from(3), Err(RulePageIndexError(3)));
}

#[test]
fn serialized_geometry_and_1264x681_projection_are_exact() {
    assert_eq!(
        RULE_UI_WINDOW_RECT,
        RuleUiRect::new(0.0, 0.0, 1_020.0, 638.0)
    );
    assert_eq!(
        RULE_UI_FRAME_RECT,
        RuleUiRect::new(10.0, 50.0, 980.0, 540.0)
    );
    assert_eq!(RULE_UI_CLOSE_RECT, RuleUiRect::new(987.0, 0.0, 30.0, 30.0));
    assert_eq!(RULE_UI_BACK_RECT, RuleUiRect::new(500.0, 605.0, 80.0, 25.0));
    assert_eq!(
        RULE_UI_PREVIOUS_RECT,
        RuleUiRect::new(560.0, 320.0, 320.0, 320.0)
    );
    assert_eq!(
        RULE_UI_NEXT_RECT,
        RuleUiRect::new(0.0, 505.0, 1_000.0, 50.0)
    );
    assert_eq!(
        RULE_UI_IMAGE_RECTS,
        [
            RuleUiRect::new(15.0, 40.0, 104.0, 91.0),
            RuleUiRect::new(0.0, 150.0, 542.0, 361.0),
            RuleUiRect::new(550.0, 0.0, 452.0, 282.0),
            RuleUiRect::new(895.0, 305.0, 103.0, 201.0),
        ]
    );

    assert_eq!(clean_rule_ui_scale(681.0), 1.0);
    let layout = rule_ui_layout(Vec2::new(1_264.0, 681.0), 1.0);
    assert_eq!(layout.viewport, Vec2::new(1_264.0, 681.0));
    assert_eq!(layout.legacy_pivot, Vec2::new(632.0, 340.0));
    assert_eq!(layout.fit_pivot, Vec2::new(632.0, 340.5));
    assert_eq!(layout.fit_scale, Vec2::ONE);
    assert_eq!(layout.combined_scale, Vec2::ONE);
    assert_eq!(
        layout.background.source,
        RuleUiRect::new(-328.0, -379.0, 1_920.0, 1_440.0)
    );
    assert_eq!(layout.background.painted, layout.background.source);
    assert_eq!(
        layout.window.source,
        RuleUiRect::new(122.0, 21.5, 1_020.0, 638.0)
    );
    assert_eq!(layout.window.painted, layout.window.source);

    // Frame-local serialized positions retain their exact final screen
    // positions once the centered window/group origins are applied.
    assert_eq!(
        RULE_UI_CONTENT_1_RECT.translated(
            layout.window.source.x + RULE_UI_FRAME_RECT.x,
            layout.window.source.y + RULE_UI_FRAME_RECT.y,
        ),
        RuleUiRect::new(262.0, 131.5, 380.0, 300.0)
    );
    assert_eq!(
        RULE_UI_IMAGE_RECTS[3].translated(
            layout.window.source.x + RULE_UI_FRAME_RECT.x,
            layout.window.source.y + RULE_UI_FRAME_RECT.y,
        ),
        RuleUiRect::new(1_027.0, 376.5, 103.0, 201.0)
    );
}

#[test]
fn low_viewports_apply_the_clean_anisotropic_fit_about_float_screen_center() {
    let layout = rule_ui_layout(Vec2::new(800.0, 500.0), 1.0);
    assert_close(layout.fit_scale.x, 800.0 / 1_020.0);
    assert_close(layout.fit_scale.y, 500.0 / 638.0);
    assert_close(layout.window.painted.x, 0.0);
    assert_close(layout.window.painted.y, 0.0);
    assert_close(layout.window.painted.width, 800.0);
    assert_close(layout.window.painted.height, 500.0);

    let high_scale = clean_rule_ui_scale(1_080.0);
    assert_close(high_scale, 1.476_562_5);
    let high = rule_ui_layout(Vec2::new(1_920.0, 1_080.0), high_scale);
    assert_eq!(high.fit_scale, Vec2::ONE);
    assert_eq!(high.window.painted.center(), Vec2::new(960.0, 540.0));
    assert_close(high.window.painted.width, 1_020.0 * high_scale);
    assert_close(high.window.painted.height, 638.0 * high_scale);

    assert_eq!(clean_rule_ui_scale(f32::NAN), 1.0);
    assert_eq!(clean_rule_fit_scale(Vec2::new(f32::NAN, -1.0)), Vec2::ZERO);
    let invalid = rule_ui_layout(Vec2::new(f32::INFINITY, f32::NAN), f32::NAN);
    assert_eq!(invalid.viewport, Vec2::ZERO);
    assert_eq!(invalid.combined_scale, Vec2::ZERO);
}

#[test]
fn entry_is_transactional_and_canonical_navigation_stays_hidden() {
    let mut model = RuleUiModel::default();
    assert_eq!(model.open_table_index(1), Ok(()));
    assert!(model.visible);
    assert_eq!(model.current_page, Some(RulePageId::Vehicle));
    let before = model.clone();
    assert_eq!(model.open_table_index(0), Err(RulePageIndexError(0)));
    assert_eq!(model, before);

    let mut outbox = RuleUiOutbox::default();
    let mut audio = RuleUiAudioOutbox::default();
    assert!(!activate_rule_ui_button(
        RuleUiButtonKind::Previous,
        &mut model,
        &mut outbox,
        &mut audio,
    ));
    assert!(!activate_rule_ui_button(
        RuleUiButtonKind::Next,
        &mut model,
        &mut outbox,
        &mut audio,
    ));
    assert_eq!(model, before);
    assert!(outbox.is_empty());
    assert!(audio.is_empty());
}

#[test]
fn close_and_back_exit_immediately_with_click_but_escape_uses_the_gate() {
    for (button, source) in [
        (RuleUiButtonKind::Close, RuleUiDismissalSource::CloseButton),
        (RuleUiButtonKind::Back, RuleUiDismissalSource::BackButton),
    ] {
        let mut model = RuleUiModel::default();
        model.open(RulePageId::Vehicle);
        let mut outbox = RuleUiOutbox::default();
        let mut audio = RuleUiAudioOutbox::default();
        assert!(activate_rule_ui_button(
            button,
            &mut model,
            &mut outbox,
            &mut audio,
        ));
        assert!(!model.visible);
        assert_eq!(
            outbox.pop_front(),
            Some(RuleUiAction::ExitMode {
                source,
                event_group: 2,
                event_function: 1,
                cursor_locked: false,
            })
        );
        assert_eq!(audio.pop_front(), Some(RuleUiAudioCue::ButtonSound));
        assert_eq!(RuleUiAudioCue::ButtonSound.gain(), 0.7);
        assert_eq!(
            RuleUiAudioCue::ButtonSound.candidate_paths(),
            &RULE_UI_BUTTON_SOUND_PATHS
        );
    }

    let mut model = RuleUiModel::default();
    model.open(RulePageId::Combining);
    let mut outbox = RuleUiOutbox::default();
    assert!(request_rule_ui_escape_close(&mut model, &mut outbox));
    assert!(model.visible);
    assert!(model.escape_close_pending);
    assert_eq!(
        outbox.pop_front(),
        Some(RuleUiAction::RequestEscapeCloseGate {
            event_group: 2,
            event_function: 24,
        })
    );
    assert!(!request_rule_ui_escape_close(&mut model, &mut outbox));
    assert!(resolve_rule_ui_escape_close_gate(
        &mut model,
        &mut outbox,
        false,
    ));
    assert!(model.visible);
    assert!(!model.escape_close_pending);
    assert!(outbox.is_empty());

    assert!(request_rule_ui_escape_close(&mut model, &mut outbox));
    assert_eq!(
        outbox.pop_front(),
        Some(RuleUiAction::RequestEscapeCloseGate {
            event_group: 2,
            event_function: 24,
        })
    );
    assert!(resolve_rule_ui_escape_close_gate(
        &mut model,
        &mut outbox,
        true,
    ));
    assert!(!model.visible);
    assert_eq!(
        outbox.pop_front(),
        Some(RuleUiAction::ExitMode {
            source: RuleUiDismissalSource::EscapeCloseGate,
            event_group: 2,
            event_function: 1,
            cursor_locked: false,
        })
    );
}

#[test]
fn help_and_system_popup_reproduce_the_asymmetric_clean_input_gates() {
    let mut model = RuleUiModel::default();
    model.open(RulePageId::Vehicle);
    let boundary = model.input_boundary();
    assert!(boundary.blocks_lower_ui);
    assert!(boundary.blocks_gameplay_input);
    assert!(boundary.requires_pointer);
    assert!(!boundary.cursor_locked_while_visible);
    assert!(!boundary.cursor_locked_after_exit);
    assert!(boundary.mouse_controls_enabled);
    assert!(boundary.escape_close_gate_enabled);

    model.set_help_active(true);
    let help = model.input_boundary();
    assert!(!help.mouse_controls_enabled);
    assert!(help.escape_close_gate_enabled);
    let mut outbox = RuleUiOutbox::default();
    assert!(request_rule_ui_escape_close(&mut model, &mut outbox));

    model.escape_close_pending = false;
    outbox.clear();
    model.set_system_popup_active(true);
    let popup = model.input_boundary();
    assert!(!popup.mouse_controls_enabled);
    assert!(!popup.escape_close_gate_enabled);
    assert!(!request_rule_ui_escape_close(&mut model, &mut outbox));
    assert!(outbox.is_empty());

    let mut audio = RuleUiAudioOutbox::default();
    assert!(!activate_rule_ui_button(
        RuleUiButtonKind::Back,
        &mut model,
        &mut outbox,
        &mut audio,
    ));
    assert!(model.visible);
    assert!(audio.is_empty());
}

#[test]
fn rule_tree_localizes_every_text_and_keeps_style_specific_font_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(RuleUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut texts =
        world.query::<(&RuleUiTextElement, (&TextFont, &LineHeight), &LocalizedText)>();
    let texts = texts.iter(world).collect::<Vec<_>>();
    assert_eq!(texts.len(), 10);
    assert!(
        texts
            .iter()
            .all(|(_, _, localized)| !localized.key.is_empty())
    );
    let (_, content_font, content_text) = texts
        .iter()
        .find(|(element, ..)| element.role == RuleUiTextRole::Content1)
        .copied()
        .unwrap();
    assert_eq!(
        content_font.0.font_size.eval(Vec2::ZERO, 16.0),
        RULE_UI_CHALET_SMALL_FONT_SIZE
    );
    assert_eq!(
        (*content_font.1),
        LineHeight::Px(RULE_UI_CHALET_SMALL_LINE_HEIGHT)
    );
    assert_eq!(content_text.key, "ui.rule.vehicle.description");
    let (_, title_font, title_text) = texts
        .iter()
        .find(|(element, ..)| element.role == RuleUiTextRole::Title)
        .copied()
        .unwrap();
    assert_eq!(
        title_font.0.font_size.eval(Vec2::ZERO, 16.0),
        RULE_UI_JEFFE_16_FONT_SIZE
    );
    assert_eq!(
        (*title_font.1),
        LineHeight::Px(RULE_UI_JEFFE_16_LINE_HEIGHT)
    );
    assert_eq!(title_text.key, "ui.rule.vehicle.title");

    world
        .resource_mut::<RuleUiModel>()
        .open(RulePageId::Combining);
    app.update();
    let world = app.world_mut();
    let mut titles = world.query::<(&RuleUiTextElement, &LocalizedText)>();
    let (_, title) = titles
        .iter(world)
        .find(|(element, _)| element.role == RuleUiTextRole::Title)
        .unwrap();
    assert_eq!(title.key, "ui.rule.combining.title");
    assert_eq!(title.fallback, RULE_UI_COMBINING_STRINGS[0]);
}

#[test]
fn semantic_assets_contract_dimensions_and_hashes_are_exact() {
    let expected = [
        (
            RULE_UI_PANEL_BACK_PATH,
            1_249_122,
            1_920,
            1_440,
            "fb9c6c8a4b8313766364ff3398070b35d8a4137f511accc4ddaaea91bd0162f1",
        ),
        (
            RULE_UI_RULE_BACK_PATH,
            14_902,
            1_024,
            545,
            "47571fcd38f256139ffbdd416323bd4fc46564adc9d2658bad19ffc4c9fb85da",
        ),
        (
            RULE_UI_VEHICLE_IMAGE_PATHS[0],
            11_596,
            104,
            91,
            "7cd769be30aa58bf7baabf173f2e218c0417c9b4c017d9e227f91497d5139246",
        ),
        (
            RULE_UI_VEHICLE_IMAGE_PATHS[1],
            130_305,
            542,
            361,
            "aaeeb47c36e436198afd34306c2141cd725e55d63a5464178e185be5a5bdb18e",
        ),
        (
            RULE_UI_VEHICLE_IMAGE_PATHS[2],
            159_686,
            452,
            282,
            "be6fa21c0729e8829954d862613a8ae154bc238d90ce0862cd11a0cc0210ec10",
        ),
        (
            RULE_UI_VEHICLE_IMAGE_PATHS[3],
            16_581,
            103,
            201,
            "9e48951cd888b7f586f4d5eb40e115cb8394255d8738579534d80d41aa09b0c1",
        ),
        (
            RULE_UI_COMBINE_IMAGE_PATHS[0],
            13_409,
            103,
            90,
            "47461a02639258853c1cd57a2741311a420e001054facc40beb6880c08c8ac62",
        ),
        (
            RULE_UI_COMBINE_IMAGE_PATHS[1],
            108_784,
            542,
            361,
            "48201a0611bb9130e05d1b8528c7d9f8266c15e0e9031679d060eef56c43a18e",
        ),
        (
            RULE_UI_COMBINE_IMAGE_PATHS[2],
            172_797,
            452,
            282,
            "22a7e2488114be9036f462ebbac67bc37a9f56203a8efeff9bf0a3a46d24a150",
        ),
        (
            RULE_UI_COMBINE_IMAGE_PATHS[3],
            13_144,
            103,
            201,
            "a28d63e64e3cfff0e1d45836b9fc7cbc3f94f4919220320e888a3597a100f0b9",
        ),
        (
            RULE_UI_BACK_NORMAL_PATH,
            540,
            112,
            26,
            "3284505c50a008f764460133e323935c4e9e70866a92063105757a801ac9f68c",
        ),
        (
            RULE_UI_BACK_HOVER_PATH,
            587,
            20,
            25,
            "6a841912fb35eb3c6eeceaf24176ecd3e1258157f96ed59b481b83f7c022bef3",
        ),
        (
            RULE_UI_NAV_NORMAL_PATH,
            587,
            20,
            25,
            "6a841912fb35eb3c6eeceaf24176ecd3e1258157f96ed59b481b83f7c022bef3",
        ),
        (
            RULE_UI_NAV_HOVER_PATH,
            452,
            20,
            25,
            "e23a8bb2dbb1a90e785a21f776693e94beb6ae542d55996231172f4ef3dd8c73",
        ),
        (
            RULE_UI_CLOSE_NORMAL_PATH,
            1_638,
            32,
            33,
            "d927fb6cd16d7980a6b2cd1bc147b0352a36b28a6a1f1feb26ba31b4eb3a642f",
        ),
        (
            RULE_UI_CLOSE_HOVER_PATH,
            1_518,
            32,
            33,
            "fab03d8ec8ce1c29d0eec6fed01be85133af36158cd95ae3ae948a51681d8e1b",
        ),
    ];
    assert_eq!(RULE_UI_IMAGE_PATHS.len(), expected.len());

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let asset_root = workspace_root.join("assets/game");
    for (index, (relative, bytes, width, height, sha256)) in expected.into_iter().enumerate() {
        assert_eq!(RULE_UI_IMAGE_PATHS[index], relative);
        let data = fs::read(asset_root.join(relative)).expect("published Rule PNG must exist");
        assert_eq!(data.len(), bytes);
        assert_eq!(&data[1..4], b"PNG");
        assert_eq!(u32::from_be_bytes(data[16..20].try_into().unwrap()), width);
        assert_eq!(u32::from_be_bytes(data[20..24].try_into().unwrap()), height);
        assert_eq!(format!("{:x}", Sha256::digest(&data)), sha256);
    }

    for (relative, bytes, sha256) in [
        (
            RULE_UI_JEFFE_FONT_PATH,
            32_776,
            "f8d41844ad2092d9998e51b8cbef5b65b3ce6db276c93949ececae227674c3e1",
        ),
        (
            RULE_UI_CHALET_FONT_PATH,
            96_832,
            "6383bd9f81e56d61139884d8e42cb7b2146a11dde4efde55c8bff1e4c2c0bbe8",
        ),
    ] {
        let data = fs::read(asset_root.join(relative)).expect("shared Rule font must exist");
        assert_eq!(data.len(), bytes);
        assert_eq!(format!("{:x}", Sha256::digest(&data)), sha256);
    }
    for (index, (bytes, sha256)) in [
        (
            3_997,
            "e2b000255b7b31c8e87756ccab6003d0f77631cfe292bbb466b79a68bf67588e",
        ),
        (
            4_150,
            "d4fc4e38c9153907a9e49d8115d35c032a11834883e46aa8ef97bca4c3220ca2",
        ),
        (
            4_175,
            "1df348d55d5ac6628e8a6658d3539d51fb6ab8508cfaf61cc945f830fde364c2",
        ),
        (
            4_041,
            "fdd50f98423b81454e5c23125cf836f494f51ad7101773c385129a3d611e9db3",
        ),
        (
            4_508,
            "bf3c9f15c5ac3540eb10a84e10182a04e2c478f32a0ecdc85c0a0cc24f5d80cb",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let data = fs::read(asset_root.join(RULE_UI_BUTTON_SOUND_PATHS[index]))
            .expect("shared Rule click candidate must exist");
        assert_eq!(data.len(), bytes);
        assert_eq!(format!("{:x}", Sha256::digest(&data)), sha256);
    }
}
