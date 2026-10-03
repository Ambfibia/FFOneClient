//! Standalone clean-Retrobution BankMode UI/style/localization acceptance.
//!
//! The production module keeps the typed authority and UI tree; this harness
//! independently opens the four shipped bundles and audits every spawned Text.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use bevy::{asset::AssetPlugin, prelude::*, text::LineHeight};
use ffone_client::{
    bank_ui::{
        BANK_BACKGROUND_ONLY_FONT_PATH_ID, BANK_BUTTON_FONT_SIZE, BANK_BUTTON_LINE_HEIGHT,
        BANK_BUTTON_PADDING_BOTTOM, BANK_BUTTON_PADDING_LEFT, BANK_BUTTON_PADDING_RIGHT,
        BANK_BUTTON_PADDING_TOP, BANK_BUTTON_SOURCE_FONT_PATH_ID,
        BANK_CONTROLLER_COMPONENT_PATH_ID, BANK_CONTROLLER_SCRIPT_PATH_ID, BANK_COUNT_LABEL_LEFT,
        BANK_COUNT_LABEL_TOP, BANK_DEXLABS_SOURCE_PATH_ID, BANK_EQUIP_COMPONENT_PATH_ID,
        BANK_EQUIP_FONT_SIZE, BANK_EQUIP_LINE_HEIGHT, BANK_EQUIP_SCRIPT_PATH_ID,
        BANK_GAME_OBJECT_PATH_ID, BANK_GUI_DEPTH_0104, BANK_INVENTORY_MANAGER_COMPONENT_PATH_ID,
        BANK_INVENTORY_SKIN_PATH_ID, BANK_LABEL_FONT_SIZE, BANK_LABEL_LINE_HEIGHT,
        BANK_LABEL_PADDING_BOTTOM, BANK_LABEL_PADDING_TOP, BANK_PANEL_COMPONENT_PATH_ID,
        BANK_PANEL_SCRIPT_PATH_ID, BANK_PC_STUFF_COMPONENT_PATH_ID, BANK_PC_STUFF_REDEEM_CODE_RECT,
        BANK_PC_STUFF_SCRIPT_PATH_ID, BANK_PC_STUFF_TAROS_DIGIT_RECTS, BANK_SLOT_BUTTON_BORDER,
        BANK_SLOT_BUTTON_SOURCE_PATH_ID, BANK_TAROS_COUNTER_SOURCE_PATH_ID, BankUiElement,
        BankUiPlugin, BankUiTextStyle,
    },
    localization::LocalizedText,
    user_equip_ui::{USER_EQUIP_COUNT_LABEL_LEFT, USER_EQUIP_COUNT_LABEL_TOP},
};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Deserialize)]
struct TextBundle {
    schema: String,
    locale: String,
    entries: BTreeMap<String, String>,
}

const BANK_KEYS: [(&str, &str, &str); 16] = [
    (
        "ui.bank.search.placeholder",
        "Search by item name...",
        "Поиск по названию...",
    ),
    ("ui.bank.title", "Morbucks Savings and Loan", "Банк Морбакс"),
    ("ui.bank.tab.vault", "BANK", "БАНК"),
    ("ui.bank.taros.digit", "{digit}", "{digit}"),
    ("ui.inventory.tab.equipment", "EQUIPMENT", "ЭКИПИРОВКА"),
    ("ui.inventory.equipped", " EQUIPPED", " НАДЕТО"),
    ("ui.inventory.item.count", "{count}", "{count}"),
    ("ui.inventory.slot.head", "HEAD", "ГОЛОВА"),
    ("ui.inventory.slot.face", "FACE", "ЛИЦО"),
    ("ui.inventory.slot.back", "BACK", "СПИНА"),
    ("ui.inventory.slot.chest", "CHEST", "ТОРС"),
    ("ui.inventory.slot.legs", "LEGS", "НОГИ"),
    ("ui.inventory.slot.feet", "FEET", "СТОПЫ"),
    (
        "ui.inventory.slot.weapon",
        "WEAPON {ordinal}",
        "ОРУЖИЕ {ordinal}",
    ),
    ("ui.inventory.slot.vehicle", "VEHICLE", "ТРАНСПОРТ"),
    ("ui.enchant.redeem_code", "REDEEM CODE", "ВВЕСТИ КОД"),
];

fn placeholders(value: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut remainder = value;
    while let Some(open) = remainder.find('{') {
        remainder = &remainder[open + 1..];
        let Some(close) = remainder.find('}') else {
            break;
        };
        result.insert(remainder[..close].to_owned());
        remainder = &remainder[close + 1..];
    }
    result
}

fn open_bundle(path: &Path, locale: &str) -> TextBundle {
    let bundle: TextBundle = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(bundle.schema, "ffone.text-bundle.v1");
    assert_eq!(bundle.locale, locale);
    bundle
}

#[test]
fn production_bank_bundles_have_key_parity_and_safe_placeholders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let runtime = root.join("assets/game/localization");

    let en = open_bundle(&runtime.join("en.json"), "en");
    let ru = open_bundle(&runtime.join("ru.json"), "ru");
    assert_eq!(
        en.entries.keys().collect::<BTreeSet<_>>(),
        ru.entries.keys().collect::<BTreeSet<_>>()
    );
    for (key, en_value) in &en.entries {
        assert_eq!(
            placeholders(en_value),
            placeholders(&ru.entries[key]),
            "placeholder mismatch for {key}"
        );
    }
    for (key, expected_en, expected_ru) in BANK_KEYS {
        assert_eq!(en.entries.get(key).map(String::as_str), Some(expected_en));
        assert_eq!(ru.entries.get(key).map(String::as_str), Some(expected_ru));
    }
}

#[test]
fn every_bank_text_is_key_first_style_typed_and_uses_args_for_dynamic_copy() {
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
        &UiTransform,
    )>();
    let rows = texts.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 274);
    let expected_keys = BANK_KEYS
        .iter()
        .map(|(key, _, _)| *key)
        .collect::<BTreeSet<_>>();
    let actual_keys = rows
        .iter()
        .map(|(_, localized, _, _, _)| localized.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_keys, expected_keys);
    assert!(rows.iter().all(|(_, localized, style, font, transform)| {
        !localized.key.trim().is_empty()
            && font.0.font_size == style.font_size().into()
            && *font.1 == LineHeight::Px(style.line_height())
            && transform.translation == Val2::px(0.0, style.replacement_y_offset())
    }));

    let counts = rows
        .iter()
        .filter(|(_, localized, _, _, _)| localized.key == "ui.inventory.item.count")
        .collect::<Vec<_>>();
    assert_eq!(counts.len(), 250);
    assert!(counts.iter().all(|(_, localized, style, _, _)| {
        **style == BankUiTextStyle::LabelUpperLeft
            && localized.fallback == "{count}"
            && localized.args.keys().map(String::as_str).eq(["count"])
    }));
    let weapons = rows
        .iter()
        .filter(|(_, localized, _, _, _)| localized.key == "ui.inventory.slot.weapon")
        .collect::<Vec<_>>();
    assert_eq!(weapons.len(), 2);
    assert!(weapons.iter().all(|(_, localized, _, _, _)| {
        localized.fallback == "WEAPON {ordinal}"
            && localized.args.keys().map(String::as_str).eq(["ordinal"])
    }));
    let taros_digits = rows
        .iter()
        .filter(|(_, localized, _, _, _)| localized.key == "ui.bank.taros.digit")
        .collect::<Vec<_>>();
    assert_eq!(taros_digits.len(), 9);
    assert!(taros_digits.iter().all(|(_, localized, style, _, _)| {
        **style == BankUiTextStyle::BlankBoxMiddleRight
            && localized.fallback == "{digit}"
            && localized.args.keys().map(String::as_str).eq(["digit"])
    }));
    let redeem = rows
        .iter()
        .find(|(_, localized, _, _, _)| localized.key == "ui.enchant.redeem_code")
        .unwrap();
    assert_eq!(*redeem.2, BankUiTextStyle::ButtonMiddleCenter);
    assert_eq!(redeem.3.0.font_size, BANK_BUTTON_FONT_SIZE.into());
    assert_eq!(*redeem.3.1, LineHeight::Px(BANK_BUTTON_LINE_HEIGHT));

    let mut nodes = world.query::<(&BankUiElement, &Node)>();
    let (_, bank_count) = nodes
        .iter(world)
        .find(|(element, _)| **element == BankUiElement::BankSlotCount(0))
        .unwrap();
    assert_eq!(bank_count.left, px(BANK_COUNT_LABEL_LEFT));
    assert_eq!(bank_count.top, px(BANK_COUNT_LABEL_TOP));
    assert_eq!(bank_count.padding.top, px(BANK_LABEL_PADDING_TOP));
    assert_eq!(bank_count.padding.bottom, px(BANK_LABEL_PADDING_BOTTOM));
    let (_, inventory_count) = nodes
        .iter(world)
        .find(|(element, _)| **element == BankUiElement::InventorySlotCount(0))
        .unwrap();
    assert_eq!(inventory_count.left, px(USER_EQUIP_COUNT_LABEL_LEFT));
    assert_eq!(inventory_count.top, px(USER_EQUIP_COUNT_LABEL_TOP));
    let (_, first_taros_digit) = nodes
        .iter(world)
        .find(|(element, _)| **element == BankUiElement::TarosDigit(0))
        .unwrap();
    assert_eq!(
        [
            first_taros_digit.left,
            first_taros_digit.top,
            first_taros_digit.width,
            first_taros_digit.height,
        ],
        [px(22), px(564), px(12), px(20)]
    );
    let (_, redeem_node) = nodes
        .iter(world)
        .find(|(element, _)| **element == BankUiElement::RedeemCode)
        .unwrap();
    assert_eq!(redeem_node.left, px(BANK_PC_STUFF_REDEEM_CODE_RECT.left));
    assert_eq!(redeem_node.top, px(BANK_PC_STUFF_REDEEM_CODE_RECT.top));
    assert_eq!(redeem_node.padding.left, px(BANK_BUTTON_PADDING_LEFT));
    assert_eq!(redeem_node.padding.right, px(BANK_BUTTON_PADDING_RIGHT));
    assert_eq!(redeem_node.padding.top, px(BANK_BUTTON_PADDING_TOP));
    assert_eq!(redeem_node.padding.bottom, px(BANK_BUTTON_PADDING_BOTTOM));
    assert_eq!(BANK_PC_STUFF_TAROS_DIGIT_RECTS.len(), 9);
}

#[test]
fn clean_primary_owner_styles_fonts_and_draw_depth_are_frozen() {
    assert_eq!(BANK_GAME_OBJECT_PATH_ID, 1_279);
    assert_eq!(
        [
            (
                BANK_CONTROLLER_COMPONENT_PATH_ID,
                BANK_CONTROLLER_SCRIPT_PATH_ID
            ),
            (BANK_PANEL_COMPONENT_PATH_ID, BANK_PANEL_SCRIPT_PATH_ID),
            (
                BANK_PC_STUFF_COMPONENT_PATH_ID,
                BANK_PC_STUFF_SCRIPT_PATH_ID
            ),
            (BANK_EQUIP_COMPONENT_PATH_ID, BANK_EQUIP_SCRIPT_PATH_ID),
        ],
        [(1_586, 919), (1_587, 1_162), (1_588, 1_045), (1_589, 1_027)]
    );
    assert_eq!(BANK_INVENTORY_MANAGER_COMPONENT_PATH_ID, 1_419);
    assert_eq!(BANK_INVENTORY_SKIN_PATH_ID, 1_366);
    assert_eq!(BANK_GUI_DEPTH_0104, 9);
    assert_eq!(BANK_SLOT_BUTTON_SOURCE_PATH_ID, 640);
    assert_eq!(BANK_DEXLABS_SOURCE_PATH_ID, 451);
    assert_eq!(BANK_TAROS_COUNTER_SOURCE_PATH_ID, 326);
    assert_eq!(
        [
            BANK_SLOT_BUTTON_BORDER.min_inset.x,
            BANK_SLOT_BUTTON_BORDER.max_inset.x,
            BANK_SLOT_BUTTON_BORDER.min_inset.y,
            BANK_SLOT_BUTTON_BORDER.max_inset.y,
        ],
        [6.0, 6.0, 6.0, 4.0]
    );
    assert_eq!(BANK_BACKGROUND_ONLY_FONT_PATH_ID, 949);
    assert_eq!(BANK_BUTTON_SOURCE_FONT_PATH_ID, 933);
    assert_eq!(BANK_LABEL_FONT_SIZE, 12.0);
    assert_eq!(BANK_LABEL_LINE_HEIGHT, 13.560_000_42);
    assert_eq!(BANK_EQUIP_FONT_SIZE, 8.0);
    assert_eq!(BANK_EQUIP_LINE_HEIGHT, 9.039_999_96);
    assert_eq!(BANK_BUTTON_FONT_SIZE, 11.0);
    assert_eq!(BANK_BUTTON_LINE_HEIGHT, 11.300_000_19);
    assert_eq!(BankUiTextStyle::LabelUpperLeft.legacy_alignment(), 0);
    assert_eq!(BankUiTextStyle::BlankBoxMiddleRight.legacy_alignment(), 5);
    assert_eq!(BankUiTextStyle::ButtonMiddleCenter.legacy_alignment(), 4);
    assert_eq!(BankUiTextStyle::EquipFontMiddleRight.legacy_alignment(), 5);
    assert_eq!(
        BankUiTextStyle::LabelUpperLeft.padding(),
        [0.0, 0.0, 3.0, 3.0]
    );
    assert_eq!(
        BankUiTextStyle::ButtonMiddleCenter.padding(),
        [6.0, 6.0, 3.0, 3.0]
    );
    assert_eq!(
        BankUiTextStyle::BlankBoxMiddleRight.content_offset(),
        [0.0, 0.0]
    );
    assert!(BankUiTextStyle::LabelUpperLeft.word_wrap());
    assert!(!BankUiTextStyle::ButtonMiddleCenter.word_wrap());
}

#[test]
fn production_source_has_no_raw_dynamic_text_write_escape_hatch() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let direct = manifest_dir.join("src/bank_ui.rs");
    let probe_relative = manifest_dir.join("../../crates/ffone-client/src/ui/bank/mod.rs");
    let source_path = if direct.is_file() {
        direct
    } else {
        probe_relative
    };
    let source = fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", source_path.display()));
    for forbidden in [
        "Option<&mut Text>",
        "text.0.clear()",
        "text.0.push_str",
        "text.0 = value",
    ] {
        assert!(
            !source.contains(forbidden),
            "raw Text write reintroduced: {forbidden}"
        );
    }
    assert!(source.contains("Option<&mut LocalizedText>"));
    assert!(!source.contains("\"NANOS\""));
    assert!(!source.contains("USER_EQUIP_NANO_TAB"));
}
