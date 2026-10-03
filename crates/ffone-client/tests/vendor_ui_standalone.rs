//! Standalone clean-Retrobution VendorMode UI/style/localization acceptance.
//!
//! The production module owns the typed projection and exact source UI tree;
//! this harness independently opens the four shipped bundles and audits every
//! spawned `Text` without requiring the full client binary.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use bevy::{asset::AssetPlugin, prelude::*, text::LineHeight};
use ffone_client::{
    localization::LocalizedText,
    vendor_ui::{
        VENDOR_BATTERY_COUNT_RECTS, VENDOR_BATTERY_FRAME_RECTS, VENDOR_BATTERY_ICON_RECTS,
        VENDOR_BATTERY_LABEL_RECTS, VENDOR_BOOST_ICON_PATH, VENDOR_BOOST_ICON_SHA256,
        VENDOR_BOOST_ICON_SOURCE_PATH_ID, VENDOR_BUTTON_FONT_SIZE, VENDOR_BUTTON_LINE_HEIGHT,
        VENDOR_BUTTON_PADDING_BOTTOM, VENDOR_BUTTON_PADDING_LEFT, VENDOR_BUTTON_PADDING_RIGHT,
        VENDOR_BUTTON_PADDING_TOP, VENDOR_BUTTON_SOURCE_FONT_PATH_ID,
        VENDOR_CN_VENDOR_COMPONENT_PATH_ID, VENDOR_CN_VENDOR_SCRIPT_PATH_ID, VENDOR_DEXLABS_PATH,
        VENDOR_DEXLABS_SHA256, VENDOR_DEXLABS_SOURCE_PATH_ID, VENDOR_EQUIP_COMPONENT_PATH_ID,
        VENDOR_EQUIP_FONT_SIZE, VENDOR_EQUIP_LINE_HEIGHT, VENDOR_EQUIP_SCRIPT_PATH_ID,
        VENDOR_EQUIP_SOURCE_FONT_PATH_ID, VENDOR_GAME_OBJECT_PATH_ID, VENDOR_GO_TO_STUFF_RECT,
        VENDOR_INVENTORY_MANAGER_COMPONENT_PATH_ID, VENDOR_INVENTORY_SKIN_PATH_ID,
        VENDOR_LABEL_FONT_SIZE, VENDOR_LABEL_LINE_HEIGHT, VENDOR_LABEL_PADDING_BOTTOM,
        VENDOR_LABEL_PADDING_TOP, VENDOR_LABEL_SOURCE_FONT_PATH_ID, VENDOR_PANEL_COMPONENT_PATH_ID,
        VENDOR_PANEL_GUI_DEPTH_0104, VENDOR_PANEL_SCRIPT_PATH_ID,
        VENDOR_PC_STUFF_COMPONENT_PATH_ID, VENDOR_PC_STUFF_DEXLABS_RECT,
        VENDOR_PC_STUFF_INVENTORY_SHADOW_RECT, VENDOR_PC_STUFF_REDEEM_CODE_RECT,
        VENDOR_PC_STUFF_SCRIPT_PATH_ID, VENDOR_PC_STUFF_TAROS_COUNTER_RECT,
        VENDOR_PC_STUFF_TAROS_DIGIT_RECTS, VENDOR_POTION_ICON_PATH, VENDOR_POTION_ICON_SHA256,
        VENDOR_POTION_ICON_SOURCE_PATH_ID, VENDOR_PRIMARY_FIRST_PASS_DLL_SHA256,
        VENDOR_ROW_ITEM_BOX_RECT, VENDOR_ROW_LEVEL_RECT, VENDOR_ROW_NAME_RECT,
        VENDOR_ROW_PRICE_ICON_RECT, VENDOR_ROW_PRICE_RECT, VENDOR_ROW_VEHICLE_SPEED_RECT,
        VENDOR_SERVICE_FONT_SIZE, VENDOR_SERVICE_LINE_HEIGHT, VENDOR_SERVICE_SOURCE_FONT_PATH_ID,
        VENDOR_SHARED_GUI_DEPTH_0104, VENDOR_SOURCE_MAIN_ARCHIVE_SHA256,
        VENDOR_SOURCE_TUTORIAL_ARCHIVE_SHA256, VENDOR_TABLE_SHADOW_RECT, VENDOR_TAROS_COUNTER_PATH,
        VENDOR_TAROS_COUNTER_SHA256, VENDOR_TAROS_COUNTER_SOURCE_PATH_ID,
        VENDOR_TEXT_REPLACEMENT_Y_OFFSET, VendorUiElement, VendorUiPlugin, VendorUiTextStyle,
    },
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

#[derive(Deserialize)]
struct TextBundle {
    schema: String,
    locale: String,
    entries: BTreeMap<String, String>,
}

const VENDOR_KEYS: [(&str, &str); 26] = [
    ("ui.vendor.shopkeeper", "SHOPKEEPER - {name}"),
    ("ui.vendor.service", "{service}"),
    ("ui.vendor.tab.buy", "BUY"),
    ("ui.vendor.tab.buyback", "BUY BACK"),
    ("ui.vendor.item.name", "{name}"),
    ("ui.vendor.item.level", "LEVEL {level}"),
    ("ui.vendor.item.vehicle_speed", "Speed: {speed} Class"),
    ("ui.vendor.item.price", "{price} TAROS"),
    ("ui.inventory.go_to_my_stuff", "GO TO MY STUFF"),
    ("ui.vendor.taros.digit", "{digit}"),
    ("ui.inventory.tab.equipment", "EQUIPMENT"),
    ("ui.inventory.equipped", " EQUIPPED"),
    ("ui.inventory.item.count", "{count}"),
    ("ui.inventory.item.quest", "Quest {item_id}"),
    ("ui.inventory.slot.head", "HEAD"),
    ("ui.inventory.slot.face", "FACE"),
    ("ui.inventory.slot.back", "BACK"),
    ("ui.inventory.slot.chest", "CHEST"),
    ("ui.inventory.slot.legs", "LEGS"),
    ("ui.inventory.slot.feet", "FEET"),
    ("ui.inventory.slot.weapon", "WEAPON {ordinal}"),
    ("ui.inventory.slot.vehicle", "VEHICLE"),
    ("ui.enchant.redeem_code", "REDEEM CODE"),
    ("ui.enchant.battery.boosts", "BOOSTS"),
    ("ui.enchant.battery.potions", "POTIONS"),
    ("ui.enchant.battery.count", "{count}"),
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
fn production_vendor_bundles_have_key_parity_and_safe_placeholders() {
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
    for (key, expected_en) in VENDOR_KEYS {
        assert_eq!(en.entries.get(key).map(String::as_str), Some(expected_en));
        assert!(ru.entries.contains_key(key), "RU is missing {key}");
    }
    assert_eq!(ru.entries["ui.vendor.service"], "{service}");
    assert_eq!(ru.entries["ui.vendor.item.name"], "{name}");
    assert_eq!(
        ru.entries["ui.vendor.item.vehicle_speed"],
        "Скорость: {speed} класс"
    );
    assert_eq!(ru.entries["ui.vendor.taros.digit"], "{digit}");
    assert_eq!(ru.entries["ui.inventory.item.quest"], "Задание {item_id}");
}

#[test]
fn every_vendor_text_is_key_first_style_typed_and_dynamic_copy_uses_args() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(VendorUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut decorations = world.query::<(&Pickable, Option<&bevy::ui::FocusPolicy>)>();
    for (pickable, focus) in decorations.iter(world) {
        if *pickable == Pickable::IGNORE {
            assert_eq!(focus, Some(&bevy::ui::FocusPolicy::Pass));
        }
    }
    let mut texts = world.query::<(
        &Text,
        &LocalizedText,
        &VendorUiTextStyle,
        (&TextFont, &LineHeight),
        &UiTransform,
    )>();
    let rows = texts.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 330);
    let expected_keys = VENDOR_KEYS
        .iter()
        .map(|(key, _)| *key)
        .filter(|key| *key != "ui.inventory.item.quest")
        .collect::<BTreeSet<_>>();
    let actual_keys = rows
        .iter()
        .map(|(_, localized, _, _, _)| localized.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_keys, expected_keys);
    assert!(!actual_keys.contains("ui.content.passthrough"));
    assert!(rows.iter().all(|(_, localized, style, font, transform)| {
        !localized.key.trim().is_empty()
            && font.0.font_size == style.font_size().into()
            && *font.1 == LineHeight::Px(style.line_height())
            && transform.translation == Val2::px(0.0, style.replacement_y_offset())
    }));

    let style_counts = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(format!("{:?}", row.2)).or_insert(0usize) += 1;
        counts
    });
    assert_eq!(style_counts["LabelUpperLeft"], 253);
    assert_eq!(style_counts["LabelUpperLeftService"], 1);
    assert_eq!(style_counts["BlankBoxUpperLeft"], 3);
    assert_eq!(style_counts["BlankBoxMiddleRight"], 59);
    assert_eq!(style_counts["ButtonMiddleCenter"], 2);
    assert_eq!(style_counts["EquipBarMiddleCenter"], 1);
    assert_eq!(style_counts["EquipFontMiddleRight"], 11);

    for (_, localized, _, _, _) in &rows {
        let expected: &[&str] = match localized.key.as_str() {
            "ui.vendor.shopkeeper" | "ui.vendor.item.name" => &["name"],
            "ui.vendor.service" => &["service"],
            "ui.vendor.item.level" => &["level"],
            "ui.vendor.item.vehicle_speed" => &["speed"],
            "ui.vendor.item.price" => &["price"],
            "ui.vendor.taros.digit" => &["digit"],
            "ui.inventory.item.count" | "ui.enchant.battery.count" => &["count"],
            "ui.inventory.item.quest" => &["item_id"],
            "ui.inventory.slot.weapon" => &["ordinal"],
            _ => &[],
        };
        assert!(
            localized
                .args
                .keys()
                .map(String::as_str)
                .eq(expected.iter().copied()),
            "{} has wrong args {:?}",
            localized.key,
            localized.args
        );
    }

    let mut nodes = world.query::<(&VendorUiElement, &Node, Option<&Text>, &ChildOf)>();
    let mut node = |wanted: VendorUiElement, world: &World| {
        nodes
            .iter(world)
            .find(|(element, ..)| **element == wanted)
            .map(|(_, node, text, parent)| {
                if text.is_some() {
                    assert_eq!(node.width, Val::Auto);
                    assert_eq!(node.height, Val::Auto);
                    world.get::<Node>(parent.parent()).unwrap().clone()
                } else {
                    node.clone()
                }
            })
            .unwrap()
    };
    let row_name = node(VendorUiElement::RowName(0), world);
    assert_eq!(
        [row_name.left, row_name.top, row_name.width, row_name.height],
        [
            px(VENDOR_ROW_NAME_RECT.left),
            px(VENDOR_ROW_NAME_RECT.top),
            px(VENDOR_ROW_NAME_RECT.width),
            px(VENDOR_ROW_NAME_RECT.height),
        ]
    );
    let row_price = node(VendorUiElement::RowPrice(0), world);
    assert_eq!(row_price.justify_content, JustifyContent::FlexEnd);
    assert_eq!(row_price.align_items, AlignItems::Center);
    let digit = node(VendorUiElement::TarosDigit(0), world);
    assert_eq!(digit.justify_content, JustifyContent::Center);
    assert_eq!(digit.align_items, AlignItems::Center);
    let service = node(VendorUiElement::VendorService, world);
    assert_eq!(service.padding.top, px(VENDOR_LABEL_PADDING_TOP));
    assert_eq!(service.padding.bottom, px(VENDOR_LABEL_PADDING_BOTTOM));
    assert_eq!(service.overflow, Overflow::clip());
    let redeem = node(VendorUiElement::RedeemCode, world);
    assert_eq!(redeem.left, px(VENDOR_PC_STUFF_REDEEM_CODE_RECT.left));
    assert_eq!(redeem.top, px(VENDOR_PC_STUFF_REDEEM_CODE_RECT.top));
    assert_eq!(redeem.padding.left, px(VENDOR_BUTTON_PADDING_LEFT));
    assert_eq!(redeem.padding.right, px(VENDOR_BUTTON_PADDING_RIGHT));
    assert_eq!(redeem.padding.top, px(VENDOR_BUTTON_PADDING_TOP));
    assert_eq!(redeem.padding.bottom, px(VENDOR_BUTTON_PADDING_BOTTOM));
}

#[test]
fn vendor_spawn_order_and_stretch_vs_nine_slice_match_called_source_branches() {
    use bevy::ui::widget::NodeImageMode;

    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(VendorUiPlugin);
    app.update();
    let world = app.world_mut();

    let mut marked = world.query::<(Entity, &VendorUiElement)>();
    let mut entity_for = |wanted: VendorUiElement, world: &World| {
        marked
            .iter(world)
            .find(|(_, element)| **element == wanted)
            .map(|(entity, _)| entity)
            .unwrap()
    };
    let render_markers = |entity: Entity, world: &World| {
        world
            .get::<Children>(entity)
            .unwrap()
            .iter()
            .flat_map(|child| {
                if let Some(marker) = world.get::<VendorUiElement>(child) {
                    vec![*marker]
                } else {
                    world
                        .get::<Children>(child)
                        .into_iter()
                        .flat_map(|children| children.iter())
                        .filter(|grandchild| world.get::<Text>(*grandchild).is_some())
                        .filter_map(|grandchild| world.get::<VendorUiElement>(grandchild).copied())
                        .collect()
                }
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(
        render_markers(entity_for(VendorUiElement::VendorDialog, world), world),
        vec![
            VendorUiElement::ListBack,
            VendorUiElement::BuyTab,
            VendorUiElement::BuybackTab,
            VendorUiElement::ListDivider,
            VendorUiElement::BuyTabSelected,
            VendorUiElement::BuybackTabSelected,
            VendorUiElement::BuyTabLabel,
            VendorUiElement::BuybackTabLabel,
            VendorUiElement::Table,
            VendorUiElement::ScrollTrack,
            VendorUiElement::ScrollThumb,
            VendorUiElement::ScrollUp,
            VendorUiElement::ScrollDown,
        ]
    );
    assert_eq!(
        render_markers(entity_for(VendorUiElement::Row(0), world), world),
        vec![
            VendorUiElement::RowFrameUnder(0),
            VendorUiElement::RowIcon(0),
            VendorUiElement::RowFrameOver(0),
            VendorUiElement::RowCount(0),
            VendorUiElement::RowName(0),
            VendorUiElement::RowLevel(0),
            VendorUiElement::RowVehicleSpeed(0),
            VendorUiElement::RowPrice(0),
            VendorUiElement::RowTaros(0),
        ]
    );

    let mut images = world.query::<(&VendorUiElement, &ImageNode)>();
    for element in [
        VendorUiElement::ListBack,
        VendorUiElement::TableShadow,
        VendorUiElement::ScrollTrack,
        VendorUiElement::GoToStuff,
        VendorUiElement::RedeemCode,
    ] {
        let (_, image) = images
            .iter(world)
            .find(|(candidate, _)| **candidate == element)
            .unwrap();
        assert!(
            matches!(image.image_mode, NodeImageMode::Sliced(_)),
            "{element:?} must preserve clean GUIStyle nine-slice"
        );
    }
    for element in [
        VendorUiElement::BuyTab,
        VendorUiElement::BuybackTab,
        VendorUiElement::Row(0),
        VendorUiElement::DexlabsBanner,
        VendorUiElement::TarosCounter,
        VendorUiElement::BatteryIcon(0),
        VendorUiElement::BatteryIcon(1),
    ] {
        let (_, image) = images
            .iter(world)
            .find(|(candidate, _)| **candidate == element)
            .unwrap();
        assert_eq!(
            image.image_mode,
            NodeImageMode::Stretch,
            "{element:?} must preserve clean GUI.DrawTexture stretch"
        );
    }
}

#[test]
fn clean_primary_owner_rects_fonts_depth_draw_branches_and_shared_bytes_are_frozen() {
    use ffone_client::vendor_ui as vendor;

    assert_eq!(VENDOR_GAME_OBJECT_PATH_ID, 1_324);
    assert_eq!(VENDOR_CN_VENDOR_COMPONENT_PATH_ID, 1_409);
    assert_eq!(VENDOR_CN_VENDOR_SCRIPT_PATH_ID, 1_016);
    assert_eq!(VENDOR_PC_STUFF_COMPONENT_PATH_ID, 1_410);
    assert_eq!(VENDOR_PC_STUFF_SCRIPT_PATH_ID, 1_045);
    assert_eq!(VENDOR_PANEL_COMPONENT_PATH_ID, 1_411);
    assert_eq!(VENDOR_PANEL_SCRIPT_PATH_ID, 1_097);
    assert_eq!(VENDOR_EQUIP_COMPONENT_PATH_ID, 1_412);
    assert_eq!(VENDOR_EQUIP_SCRIPT_PATH_ID, 1_027);
    assert_eq!(VENDOR_INVENTORY_MANAGER_COMPONENT_PATH_ID, 1_419);
    assert_eq!(VENDOR_INVENTORY_SKIN_PATH_ID, 1_366);
    assert_eq!(VENDOR_PANEL_GUI_DEPTH_0104, 10);
    assert_eq!(VENDOR_SHARED_GUI_DEPTH_0104, 9);
    assert_eq!(VENDOR_LABEL_SOURCE_FONT_PATH_ID, 977);
    assert_eq!(VENDOR_BUTTON_SOURCE_FONT_PATH_ID, 933);
    assert_eq!(VENDOR_EQUIP_SOURCE_FONT_PATH_ID, 970);
    assert_eq!(VENDOR_SERVICE_SOURCE_FONT_PATH_ID, 949);
    assert_eq!(
        (VENDOR_LABEL_FONT_SIZE, VENDOR_LABEL_LINE_HEIGHT),
        (12.0, 13.560_000_42)
    );
    assert_eq!(
        (VENDOR_BUTTON_FONT_SIZE, VENDOR_BUTTON_LINE_HEIGHT),
        (11.0, 11.300_000_19)
    );
    assert_eq!(
        (VENDOR_EQUIP_FONT_SIZE, VENDOR_EQUIP_LINE_HEIGHT),
        (8.0, 9.039_999_96)
    );
    assert_eq!(
        (VENDOR_SERVICE_FONT_SIZE, VENDOR_SERVICE_LINE_HEIGHT),
        (12.0, 13.560_000_42)
    );
    assert_eq!(VENDOR_TEXT_REPLACEMENT_Y_OFFSET, 0.0);
    assert_eq!(vendor::VENDOR_BUTTON_NORMAL_TEXT_RGB, [0.9, 0.9, 0.9]);
    assert_eq!(
        vendor::VENDOR_BUTTON_HOVER_TEXT_RGB,
        [0.229_838_71, 0.463_709_68, 1.0]
    );
    assert_eq!(VENDOR_PRIMARY_FIRST_PASS_DLL_SHA256.len(), 64);
    assert_eq!(VENDOR_SOURCE_MAIN_ARCHIVE_SHA256.len(), 64);
    assert_eq!(VENDOR_SOURCE_TUTORIAL_ARCHIVE_SHA256.len(), 64);
    assert_eq!(vendor::VENDOR_PRIMARY_ASSEMBLY_CSHARP_DLL_SHA256.len(), 64);

    for (style, name, path_id, alignment, padding, wrap, size, line_height) in [
        (
            VendorUiTextStyle::LabelUpperLeft,
            "label",
            977,
            0,
            [0.0, 0.0, 3.0, 3.0],
            true,
            12.0,
            13.560_000_42,
        ),
        (
            VendorUiTextStyle::LabelUpperLeftService,
            "label",
            949,
            0,
            [0.0, 0.0, 3.0, 3.0],
            true,
            12.0,
            13.560_000_42,
        ),
        (
            VendorUiTextStyle::BlankBoxUpperLeft,
            "blankbox",
            977,
            0,
            [0.0; 4],
            false,
            12.0,
            13.560_000_42,
        ),
        (
            VendorUiTextStyle::BlankBoxMiddleRight,
            "blankbox",
            977,
            5,
            [0.0; 4],
            false,
            12.0,
            13.560_000_42,
        ),
        (
            VendorUiTextStyle::ButtonMiddleCenter,
            "button",
            933,
            4,
            [6.0, 6.0, 3.0, 3.0],
            false,
            11.0,
            11.300_000_19,
        ),
        (
            VendorUiTextStyle::EquipBarMiddleCenter,
            "equipbar",
            970,
            4,
            [0.0; 4],
            false,
            8.0,
            9.039_999_96,
        ),
        (
            VendorUiTextStyle::EquipFontMiddleRight,
            "equipfont",
            970,
            5,
            [0.0; 4],
            false,
            8.0,
            9.039_999_96,
        ),
    ] {
        assert_eq!(style.source_style_name(), name);
        assert_eq!(style.source_font_path_id(), path_id);
        assert_eq!(style.legacy_alignment(), alignment);
        assert_eq!(style.padding(), padding);
        assert_eq!(style.content_offset(), [0.0, 0.0]);
        assert_eq!(style.word_wrap(), wrap);
        assert_eq!(style.font_size(), size);
        assert_eq!(style.line_height(), line_height);
        assert_eq!(style.replacement_y_offset(), 0.0);
    }

    assert_eq!(VENDOR_ROW_ITEM_BOX_RECT.width, 66.0);
    assert_eq!(VENDOR_ROW_ITEM_BOX_RECT.height, 66.0);
    assert_eq!(VENDOR_ROW_LEVEL_RECT.top, 22.0);
    assert_eq!(VENDOR_ROW_VEHICLE_SPEED_RECT.top, 37.0);
    assert_eq!(
        VENDOR_ROW_PRICE_RECT,
        vendor::VendorUiRect::new(262.0, 45.0, 134.0, 20.0)
    );
    assert_eq!(
        VENDOR_ROW_PRICE_ICON_RECT,
        vendor::VendorUiRect::new(406.0, 40.0, 23.0, 24.0)
    );
    assert_eq!(
        VENDOR_GO_TO_STUFF_RECT,
        vendor::VendorUiRect::new(313.0, 595.0, 161.0, 25.0)
    );
    assert_eq!(
        VENDOR_TABLE_SHADOW_RECT,
        vendor::VendorUiRect::new(2.0, 3.0, 450.0, 412.0)
    );
    assert_eq!(
        VENDOR_PC_STUFF_INVENTORY_SHADOW_RECT,
        vendor::VendorUiRect::new(3.0, 33.0, 350.0, 503.0)
    );
    assert_eq!(
        VENDOR_PC_STUFF_DEXLABS_RECT,
        vendor::VendorUiRect::new(170.0, 561.0, 203.0, 67.0)
    );
    assert_eq!(
        VENDOR_PC_STUFF_TAROS_COUNTER_RECT,
        vendor::VendorUiRect::new(20.0, 560.0, 149.0, 32.0)
    );
    assert_eq!(VENDOR_PC_STUFF_TAROS_DIGIT_RECTS.len(), 9);
    assert_eq!(VENDOR_BATTERY_FRAME_RECTS.len(), 2);
    assert_eq!(VENDOR_BATTERY_ICON_RECTS.len(), 2);
    assert_eq!(VENDOR_BATTERY_LABEL_RECTS.len(), 2);
    assert_eq!(VENDOR_BATTERY_COUNT_RECTS.len(), 2);

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for (path, path_id, expected_hash) in [
        (
            VENDOR_DEXLABS_PATH,
            VENDOR_DEXLABS_SOURCE_PATH_ID,
            VENDOR_DEXLABS_SHA256,
        ),
        (
            VENDOR_TAROS_COUNTER_PATH,
            VENDOR_TAROS_COUNTER_SOURCE_PATH_ID,
            VENDOR_TAROS_COUNTER_SHA256,
        ),
        (
            VENDOR_BOOST_ICON_PATH,
            VENDOR_BOOST_ICON_SOURCE_PATH_ID,
            VENDOR_BOOST_ICON_SHA256,
        ),
        (
            VENDOR_POTION_ICON_PATH,
            VENDOR_POTION_ICON_SOURCE_PATH_ID,
            VENDOR_POTION_ICON_SHA256,
        ),
    ] {
        assert!(path_id > 0);
        let hash = format!("{:X}", Sha256::digest(fs::read(root.join(path)).unwrap()));
        assert_eq!(hash, expected_hash, "shared clean asset changed: {path}");
    }
}

#[test]
fn vendor_source_has_no_raw_text_write_or_compatibility_passthrough() {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("crates/ffone-client/src/ui/vendor/mod.rs");
    let source = fs::read_to_string(source_path).unwrap();
    let production = source.split("#[cfg(test)]").next().unwrap();
    assert_eq!(
        production
            .lines()
            .filter(|line| { line.contains("Text::new(") && !line.contains("LocalizedText::new(") })
            .count(),
        5
    );
    assert!(!production.contains("ui.content.passthrough"));
    assert!(!production.contains("Text::new(format!"));
    assert!(!production.contains("text.0 ="));
    assert!(!production.contains("Mut<Text>"));
    for helper in [
        "vendor_tab_localized(value)",
        "vendor_title_localized(\"\")",
        "vendor_service_localized(\"\")",
        "vendor_item_name_localized(\"\")",
        "vendor_level_localized(None)",
        "vendor_vehicle_speed_localized(0)",
        "vendor_price_localized(None)",
        "inventory_count_localized(\"\")",
        "vendor_taros_digit_localized(\"0\")",
        "vendor_battery_count_localized(0)",
    ] {
        assert!(
            production.contains(helper),
            "missing key-first helper {helper}"
        );
    }
}

#[test]
fn production_vendor_service_and_item_copy_resolves_in_both_locales() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = ffone_client::tutorial_mission_content::TutorialMissionContent::open(
        &ffone_client::assets::AssetLocator::open(&root).unwrap(),
    )
    .unwrap();
    let (localization, en) = ffone_client::localization::Localization::open(&root, "en").unwrap();
    let (_, ru) = ffone_client::localization::Localization::open(&root, "ru").unwrap();
    let mut services = std::collections::BTreeSet::new();
    for npc in 0..10000 {
        let Some(keyed) = content.gameplay_npc_service_localized(npc) else {
            continue;
        };
        let fallback = &keyed.fallback;
        let number = keyed.key.clone();
        assert_eq!(localization.text(&en, &keyed), *fallback);
        if !fallback.is_empty() {
            let translated = localization.text(&ru, &keyed);
            assert_ne!(translated, *fallback, "untranslated service {number}");
            assert!(
                translated
                    .chars()
                    .any(|c| ('\u{0400}'..='\u{052f}').contains(&c))
            );
        }
        services.insert(number);
    }
    assert!(services.len() >= 20);
    for (kind, id) in [(0, 433), (4, 63)] {
        let (name, _) = content.gameplay_user_equip_item_text(kind, id).unwrap();
        assert_ne!(localization.text(&en, &name), localization.text(&ru, &name));
    }
}
