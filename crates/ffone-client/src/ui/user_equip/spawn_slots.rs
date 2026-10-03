//! Spawning of inventory, equipment, Nano gallery, battery and Nano status slots.

use super::asset_contract::UserEquipStaticAssetRole;
use super::binding::{equipment_slot_localized, inventory_count_localized};
use super::catalog::{USER_EQUIP_EQUIPMENT_STRIP_ORDER, UserEquipSlotEndpoint};
use super::components::{
    UserEquipEquipmentSlotLabelText, UserEquipNanoSlotControl, UserEquipSlotControl,
    UserEquipUiAssets, UserEquipUiElement,
};
use super::geometry::{
    USER_EQUIP_BOOST_ICON_RECT, USER_EQUIP_BOOST_LABEL_RECT, USER_EQUIP_BOOST_RECT,
    USER_EQUIP_BOOST_VALUE_RECT, USER_EQUIP_COMBINED_BADGE_LEFT, USER_EQUIP_COMBINED_BADGE_SIZE,
    USER_EQUIP_COMBINED_BADGE_TOP, USER_EQUIP_COUNT_FONT_SIZE, USER_EQUIP_COUNT_LABEL_LEFT,
    USER_EQUIP_COUNT_LABEL_TOP, USER_EQUIP_EQUIPMENT_CONTENT_RECT, USER_EQUIP_EQUIPMENT_SLOT_SIZE,
    USER_EQUIP_EQUIPMENT_SLOT_STRIDE, USER_EQUIP_INVENTORY_COLUMNS, USER_EQUIP_INVENTORY_SLOT_SIZE,
    USER_EQUIP_INVENTORY_SLOT_STRIDE, USER_EQUIP_NANO_ATTRIBUTE_RECT, USER_EQUIP_NANO_COLUMNS,
    USER_EQUIP_NANO_NAME_RECT, USER_EQUIP_NANO_PREVIEW_RECTS, USER_EQUIP_NANO_SKILL_RECT,
    USER_EQUIP_NANO_SLOT_LABEL_RECT, USER_EQUIP_NANO_SLOT_SIZE, USER_EQUIP_NANO_SLOT_STRIDE,
    USER_EQUIP_NANO_STAMINA_RECT, USER_EQUIP_NANO_STATUS_RECTS, USER_EQUIP_NANO_TYPE_RECTS,
    USER_EQUIP_POTION_ICON_RECT, USER_EQUIP_POTION_LABEL_RECT, USER_EQUIP_POTION_RECT,
    USER_EQUIP_POTION_VALUE_RECT, USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
    USER_EQUIP_SMALL_FONT_LINE_HEIGHT, USER_EQUIP_SMALL_FONT_SIZE, USER_EQUIP_TAB_FONT_SIZE,
    UserEquipUiRect,
};
use super::images::stretched_image;
use super::spawn::{spawn_bound_text_styled, user_equip_equipfont_node};
use super::view_model::equipment_slot_label;
use crate::localization::LocalizedText;
use bevy::{prelude::*, text::LineHeight};

pub(super) fn spawn_nano_gallery_slot(
    parent: &mut ChildSpawnerCommands,
    visual_index: usize,
    assets: &UserEquipUiAssets,
) {
    let column = visual_index % USER_EQUIP_NANO_COLUMNS;
    let row = visual_index / USER_EQUIP_NANO_COLUMNS;
    let frame = UserEquipUiRect::new(
        column as f32 * USER_EQUIP_NANO_SLOT_STRIDE,
        row as f32 * USER_EQUIP_NANO_SLOT_STRIDE,
        USER_EQUIP_NANO_SLOT_SIZE,
        USER_EQUIP_NANO_SLOT_SIZE,
    );
    parent
        .spawn((
            Button,
            UserEquipUiElement::NanoSlotFrame(visual_index),
            UserEquipNanoSlotControl(visual_index),
            frame.node(),
            stretched_image(assets.image(UserEquipStaticAssetRole::SlotOccupied)),
        ))
        .with_children(|slot| {
            slot.spawn((
                UserEquipUiElement::NanoSlotIcon(visual_index),
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(
                        0.0,
                        0.0,
                        USER_EQUIP_NANO_SLOT_SIZE,
                        USER_EQUIP_NANO_SLOT_SIZE,
                    )
                    .node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_inventory_slot(
    parent: &mut ChildSpawnerCommands,
    slot: usize,
    assets: &UserEquipUiAssets,
) {
    let column = slot % USER_EQUIP_INVENTORY_COLUMNS;
    let row = slot / USER_EQUIP_INVENTORY_COLUMNS;
    let frame = UserEquipUiRect::new(
        column as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        row as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
    );
    parent
        .spawn((
            Button,
            UserEquipUiElement::InventorySlotFrame(slot),
            BackgroundColor(Color::NONE),
            UserEquipSlotControl(UserEquipSlotEndpoint::Inventory { slot_index: slot }),
            frame.node(),
            stretched_image(assets.image(UserEquipStaticAssetRole::SlotEmpty)),
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                UserEquipUiElement::InventorySlotIcon(slot),
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(
                        0.0,
                        0.0,
                        USER_EQUIP_INVENTORY_SLOT_SIZE,
                        USER_EQUIP_INVENTORY_SLOT_SIZE,
                    )
                    .node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            slot_node.spawn((
                UserEquipUiElement::InventorySlotBadge(slot),
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(
                        USER_EQUIP_COMBINED_BADGE_LEFT,
                        USER_EQUIP_COMBINED_BADGE_TOP,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                    )
                    .node()
                },
                stretched_image(assets.image(UserEquipStaticAssetRole::Combined)),
                Pickable::IGNORE,
            ));
            slot_node.spawn((
                UserEquipUiElement::InventorySlotCount(slot),
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(
                        USER_EQUIP_COUNT_LABEL_LEFT,
                        USER_EQUIP_COUNT_LABEL_TOP,
                        USER_EQUIP_INVENTORY_SLOT_SIZE,
                        USER_EQUIP_INVENTORY_SLOT_SIZE,
                    )
                    .node()
                },
                Text::new(""),
                (
                    TextFont {
                        font: (assets.font.clone()).into(),
                        font_size: (USER_EQUIP_COUNT_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                inventory_count_localized(""),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_equipment_slot(
    parent: &mut ChildSpawnerCommands,
    visual_index: usize,
    assets: &UserEquipUiAssets,
) {
    let frame = UserEquipUiRect::new(
        USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
        USER_EQUIP_EQUIPMENT_CONTENT_RECT.top
            + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE,
        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
    );
    parent
        .spawn((
            Button,
            UserEquipUiElement::EquipmentSlotFrame(visual_index),
            UserEquipSlotControl(UserEquipSlotEndpoint::Equipment {
                visual_index,
                wire_slot_index: USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index,
            }),
            frame.node(),
            stretched_image(assets.image(UserEquipStaticAssetRole::SlotEmpty)),
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                UserEquipUiElement::EquipmentSlotIcon(visual_index),
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(
                        0.0,
                        0.0,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                    )
                    .node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            slot_node.spawn((
                UserEquipUiElement::EquipmentSlotBadge(visual_index),
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(
                        USER_EQUIP_COMBINED_BADGE_LEFT,
                        USER_EQUIP_COMBINED_BADGE_TOP,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                    )
                    .node()
                },
                stretched_image(assets.image(UserEquipStaticAssetRole::Combined)),
                Pickable::IGNORE,
            ));
            slot_node
                .spawn((
                    UserEquipUiElement::EquipmentSlotLabel(visual_index),
                    user_equip_equipfont_node(UserEquipUiRect::new(0.0, -2.0, 60.0, 19.0)),
                    Pickable::IGNORE,
                ))
                .with_children(|label| {
                    label.spawn((
                        UserEquipEquipmentSlotLabelText(visual_index),
                        Text::new(equipment_slot_label(
                            USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index],
                        )),
                        (
                            TextFont {
                                font: (assets.font.clone()).into(),
                                font_size: (USER_EQUIP_SMALL_FONT_SIZE).into(),
                                ..default()
                            },
                            LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT),
                        ),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                        TextLayout::new(Justify::Left, LineBreak::NoWrap),
                        equipment_slot_localized(USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index]),
                        Pickable::IGNORE,
                    ));
                });
        });
}

pub(super) fn spawn_battery_half_slots(parent: &mut ChildSpawnerCommands, assets: &UserEquipUiAssets) {
    for (
        slot_element,
        icon_element,
        label_element,
        value_element,
        slot_rect,
        icon_rect,
        label_rect,
        value_rect,
        icon_role,
        label,
        key,
    ) in [
        (
            UserEquipUiElement::BoostSlot,
            UserEquipUiElement::BoostIcon,
            UserEquipUiElement::BoostLabel,
            UserEquipUiElement::BoostValue,
            USER_EQUIP_BOOST_RECT,
            USER_EQUIP_BOOST_ICON_RECT,
            USER_EQUIP_BOOST_LABEL_RECT,
            USER_EQUIP_BOOST_VALUE_RECT,
            UserEquipStaticAssetRole::BoostIcon,
            "BOOSTS",
            "ui.inventory.slot.boosts",
        ),
        (
            UserEquipUiElement::PotionSlot,
            UserEquipUiElement::PotionIcon,
            UserEquipUiElement::PotionLabel,
            UserEquipUiElement::PotionValue,
            USER_EQUIP_POTION_RECT,
            USER_EQUIP_POTION_ICON_RECT,
            USER_EQUIP_POTION_LABEL_RECT,
            USER_EQUIP_POTION_VALUE_RECT,
            UserEquipStaticAssetRole::PotionIcon,
            "POTIONS",
            "ui.inventory.slot.potions",
        ),
    ] {
        let slot_rect = UserEquipUiRect::new(
            slot_rect.left + USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
            slot_rect.top + USER_EQUIP_EQUIPMENT_CONTENT_RECT.top,
            slot_rect.width,
            slot_rect.height,
        );
        let label_rect = UserEquipUiRect::new(
            label_rect.left + USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
            label_rect.top + USER_EQUIP_EQUIPMENT_CONTENT_RECT.top,
            label_rect.width,
            label_rect.height,
        );
        let icon_rect = UserEquipUiRect::new(
            icon_rect.left + USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
            icon_rect.top + USER_EQUIP_EQUIPMENT_CONTENT_RECT.top,
            icon_rect.width,
            icon_rect.height,
        );
        let value_rect = UserEquipUiRect::new(
            value_rect.left + USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
            value_rect.top + USER_EQUIP_EQUIPMENT_CONTENT_RECT.top,
            value_rect.width,
            value_rect.height,
        );
        parent.spawn((
            slot_element,
            slot_rect.node(),
            stretched_image(assets.image(UserEquipStaticAssetRole::SlotEmpty)),
            Pickable::IGNORE,
        ));
        parent.spawn((
            icon_element,
            icon_rect.node(),
            stretched_image(assets.image(icon_role)),
            Pickable::IGNORE,
        ));
        // Serialized `equipfont` is MiddleRight. The battery number is a
        // default UpperLeft GUI.Label with the skin's 3 px top padding.
        spawn_bound_text_styled(
            parent,
            label_element,
            label_rect,
            label,
            key,
            label,
            assets.font.clone(),
            USER_EQUIP_SMALL_FONT_SIZE,
            USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
            Color::WHITE,
            Justify::Right,
        );
        spawn_bound_text_styled(
            parent,
            value_element,
            value_rect,
            "0",
            "ui.inventory.item.count",
            "{count}",
            assets.font.clone(),
            USER_EQUIP_TAB_FONT_SIZE,
            USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
            Color::WHITE,
            Justify::Left,
        );
    }
}

pub(super) fn spawn_nano_status_panels(parent: &mut ChildSpawnerCommands, assets: &UserEquipUiAssets) {
    for slot_index in 0..3 {
        parent
            .spawn((
                UserEquipUiElement::NanoStatusPanel(slot_index),
                USER_EQUIP_NANO_STATUS_RECTS[slot_index].node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::NanoDialog)),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    UserEquipUiElement::NanoStatusSlotLabel(slot_index),
                    USER_EQUIP_NANO_SLOT_LABEL_RECT.node(),
                    Text::new(""),
                    (
                        TextFont {
                            font: (assets.font.clone()).into(),
                            font_size: (USER_EQUIP_SMALL_FONT_SIZE).into(),
                            ..default()
                        },
                        LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT),
                    ),
                    TextColor(Color::WHITE),
                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                    LocalizedText::new("ui.inventory.nano.slot", "NANO {ordinal}")
                        .with_arg("ordinal", (slot_index + 1).to_string()),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::NanoStatusName(slot_index),
                    USER_EQUIP_NANO_NAME_RECT.node(),
                    Text::new("EMPTY"),
                    (
                        TextFont {
                            font: (assets.font.clone()).into(),
                            font_size: (USER_EQUIP_SMALL_FONT_SIZE).into(),
                            ..default()
                        },
                        LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT),
                    ),
                    TextColor(Color::srgb(0.794_354_86, 1.0, 1.0)),
                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                    LocalizedText::new("ui.inventory.nano.empty", "EMPTY"),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::NanoStatusAttribute(slot_index),
                    Node {
                        display: Display::None,
                        ..USER_EQUIP_NANO_ATTRIBUTE_RECT.node()
                    },
                    Text::new(""),
                    (
                        TextFont {
                            font: (assets.font.clone()).into(),
                            font_size: (USER_EQUIP_SMALL_FONT_SIZE).into(),
                            ..default()
                        },
                        LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT),
                    ),
                    TextColor(Color::WHITE),
                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                    LocalizedText::new("ui.inventory.nano.attribute", "{attribute}")
                        .with_arg("attribute", ""),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::NanoStatusSkill(slot_index),
                    Node {
                        display: Display::None,
                        ..USER_EQUIP_NANO_SKILL_RECT.node()
                    },
                    ImageNode::default(),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::NanoStatusStamina(slot_index),
                    Node {
                        width: px(0),
                        ..USER_EQUIP_NANO_STAMINA_RECT.node()
                    },
                    stretched_image(assets.image(UserEquipStaticAssetRole::HpBar)),
                    Pickable::IGNORE,
                ));
            });
        // Clean draw order is window text, live RenderClothes actor, then the
        // affinity overlay; sibling order preserves the same overlap behavior.
        parent.spawn((
            UserEquipUiElement::NanoStatusPortrait(slot_index),
            Node {
                display: Display::None,
                ..USER_EQUIP_NANO_PREVIEW_RECTS[slot_index].node()
            },
            ImageNode::default(),
            Pickable::IGNORE,
        ));
        parent.spawn((
            UserEquipUiElement::NanoStatusType(slot_index),
            Node {
                display: Display::None,
                ..USER_EQUIP_NANO_TYPE_RECTS[slot_index].node()
            },
            ImageNode::default(),
            Pickable::IGNORE,
        ));
    }
}
