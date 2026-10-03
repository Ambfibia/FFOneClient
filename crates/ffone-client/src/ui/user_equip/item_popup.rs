//! Item popup spawning, button colors and popup geometry/localization helpers.

use super::asset_contract::UserEquipStaticAssetRole;
use super::components::{
    UserEquipGumNanoButtonControl, UserEquipPopupButtonControl, UserEquipPopupCloseControl,
    UserEquipPopupTrashControl, UserEquipUiAssets, UserEquipUiElement,
};
use super::geometry::{
    USER_EQUIP_GUM_NANO_BUTTON_RECTS, USER_EQUIP_GUM_NANO_FRAME_RECTS, USER_EQUIP_POPUP_RECT,
    USER_EQUIP_REGULAR_FONT_LINE_HEIGHT, UserEquipUiRect,
};
use super::images::{sliced_image, stretched_image};
use super::item_projection::UserEquipItemModeProjection;
use super::nano_projection::UserEquipNanoModeProjection;
use super::popup_state::{
    UserEquipItemPopupState, UserEquipPopupCommand, UserEquipPopupLayoutVariant,
    user_equip_gum_target_enabled,
};
use super::spawn::spawn_bound_text_styled;
use crate::{localization::LocalizedText, tutorial_mission_content::TutorialMissionContent};
use bevy::{prelude::*, sprite::BorderRect, text::LineHeight};

pub(super) fn spawn_item_popup(parent: &mut ChildSpawnerCommands, assets: &UserEquipUiAssets) {
    parent
        .spawn((
            UserEquipUiElement::ItemPopup,
            crate::ui::shared::controller::ControllerUiBoundary,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(px(12)),
                row_gap: px(7),
                ..USER_EQUIP_POPUP_RECT.node()
            },
        ))
        .with_children(|popup| {
            crate::item_card::spawn(popup, crate::item_card::CardOwner::Inventory, &assets.font);
            popup.spawn((
                UserEquipUiElement::ItemPopupBackdrop,
                UserEquipUiRect::new(0.0, 14.0, 310.0, 435.0).node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::EquipPopup)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            popup.spawn((
                UserEquipUiElement::ItemPopupEquipInfo,
                UserEquipUiRect::new(2.0, 157.0, 305.0, 84.0).node(),
                sliced_image(
                    assets.image(UserEquipStaticAssetRole::EquipInfo),
                    BorderRect::axes(0.0, 5.0),
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            popup.spawn((
                UserEquipUiElement::ItemPopupCalculatorBack,
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(87.0, 162.0, 136.0, 109.0).node()
                },
                stretched_image(assets.image(UserEquipStaticAssetRole::CalculatorBack)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            popup.spawn((
                UserEquipUiElement::ItemPopupIconFrame,
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(16.0, 16.0, 64.0, 64.0).node()
                },
                stretched_image(assets.image(UserEquipStaticAssetRole::SlotOccupied)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            popup.spawn((
                UserEquipUiElement::ItemPopupIcon,
                UserEquipUiRect::new(16.0, 16.0, 64.0, 64.0).node(),
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_bound_text_styled(
                popup,
                UserEquipUiElement::ItemPopupIdentity,
                UserEquipUiRect::new(12.0, 95.0, 280.0, 40.0),
                "",
                "ui.inventory.popup.description",
                "{description}",
                assets.font.clone(),
                8.0,
                10.968,
                Color::WHITE,
                Justify::Left,
            );
            popup.spawn((
                Button,
                UserEquipUiElement::ItemPopupClose,
                UserEquipPopupCloseControl,
                UserEquipUiRect::new(277.0, 0.0, 32.0, 32.0).node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::Close)),
                ZIndex(20),
            ));
            popup.spawn((
                Button,
                UserEquipUiElement::ItemPopupTrash,
                UserEquipPopupTrashControl,
                UserEquipUiRect::new(10.0, 375.0, 32.0, 32.0).node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::Trash)),
            ));
            spawn_bound_text_styled(
                popup,
                UserEquipUiElement::ItemPopupTitle,
                UserEquipUiRect::new(82.0, 16.0, 170.0, 40.0),
                "",
                "ui.inventory.popup.item_name",
                "{name}",
                assets.font.clone(),
                12.0,
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                Color::srgb(0.8, 1.0, 1.0),
                Justify::Left,
            );
            let fields = [
                (
                    UserEquipUiRect::new(82.0, 50.0, 170.0, 20.0),
                    "ui.inventory.popup.level",
                    "Level {level}",
                    Justify::Left,
                    Color::srgb(1.0, 1.0, 0.0),
                ),
                (
                    UserEquipUiRect::new(5.0, 140.0, 200.0, 20.0),
                    "ui.inventory.popup.status",
                    "STATUS",
                    Justify::Left,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(28.0, 219.0, 80.0, 20.0),
                    "ui.inventory.popup.point_value",
                    "{value}",
                    Justify::Center,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(115.0, 219.0, 80.0, 20.0),
                    "ui.inventory.popup.group_value",
                    "{value}",
                    Justify::Center,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(207.0, 219.0, 80.0, 20.0),
                    "ui.inventory.popup.defense_value",
                    "{value}",
                    Justify::Center,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(10.0, 238.0, 200.0, 20.0),
                    "ui.inventory.popup.info",
                    "INFO",
                    Justify::Left,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(8.0, 256.0, 130.0, 20.0),
                    "ui.inventory.popup.type",
                    "Type",
                    Justify::Right,
                    Color::srgb(0.78, 1.0, 1.0),
                ),
                (
                    UserEquipUiRect::new(8.0, 276.0, 130.0, 20.0),
                    "ui.inventory.popup.range",
                    "Range",
                    Justify::Right,
                    Color::srgb(0.78, 1.0, 1.0),
                ),
                (
                    UserEquipUiRect::new(8.0, 296.0, 130.0, 20.0),
                    "ui.inventory.popup.rarity",
                    "Rarity",
                    Justify::Right,
                    Color::srgb(0.78, 1.0, 1.0),
                ),
                (
                    UserEquipUiRect::new(8.0, 316.0, 130.0, 20.0),
                    "ui.inventory.popup.trade",
                    "Trade Availability",
                    Justify::Right,
                    Color::srgb(0.78, 1.0, 1.0),
                ),
                (
                    UserEquipUiRect::new(150.0, 260.0, 130.0, 20.0),
                    "ui.inventory.popup.type_value",
                    "{value}",
                    Justify::Center,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(150.0, 280.0, 130.0, 20.0),
                    "ui.inventory.popup.range_value",
                    "{value}",
                    Justify::Center,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(150.0, 300.0, 130.0, 20.0),
                    "ui.inventory.popup.rarity_value",
                    "{value}",
                    Justify::Center,
                    Color::WHITE,
                ),
                (
                    UserEquipUiRect::new(150.0, 320.0, 130.0, 20.0),
                    "ui.inventory.popup.trade_value",
                    "{value}",
                    Justify::Center,
                    Color::srgb(0.0, 1.0, 0.0),
                ),
            ];
            for (index, (rect, key, fallback, justify, color)) in fields.into_iter().enumerate() {
                // Clean default GUI.Label roles (level/status/info) use
                // JEFFE12. Rating and INFO-table roles use JEFFE08.
                let font_size = if matches!(index, 0 | 1 | 5) {
                    12.0
                } else {
                    8.0
                };
                spawn_bound_text_styled(
                    popup,
                    UserEquipUiElement::ItemPopupField(index),
                    rect,
                    fallback,
                    key,
                    fallback,
                    assets.font.clone(),
                    font_size,
                    if matches!(index, 0 | 1 | 5) {
                        USER_EQUIP_REGULAR_FONT_LINE_HEIGHT
                    } else {
                        10.968
                    },
                    color,
                    justify,
                );
            }
            spawn_bound_text_styled(
                popup,
                UserEquipUiElement::ItemPopupAmountLabel,
                UserEquipUiRect::new(87.0, 140.0, 137.0, 20.0),
                "Amount",
                "ui.inventory.popup.amount",
                "Amount",
                assets.font.clone(),
                12.0,
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                Color::WHITE,
                Justify::Center,
            );
            spawn_bound_text_styled(
                popup,
                UserEquipUiElement::ItemPopupAmountValue,
                UserEquipUiRect::new(88.0, 163.0, 132.0, 20.0),
                "0",
                "ui.inventory.popup.amount_value",
                "{amount}",
                assets.font.clone(),
                12.0,
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Right,
            );
            let keypad = [
                ("1", 88.0, 191.0),
                ("2", 133.0, 191.0),
                ("3", 178.0, 191.0),
                ("4", 88.0, 211.0),
                ("5", 133.0, 211.0),
                ("6", 178.0, 211.0),
                ("7", 88.0, 231.0),
                ("8", 133.0, 231.0),
                ("9", 178.0, 231.0),
                ("C", 88.0, 251.0),
                ("0", 133.0, 251.0),
            ];
            for (index, (label, left, top)) in keypad.into_iter().enumerate() {
                let (key, fallback) = if label == "C" {
                    ("ui.inventory.popup.clear_short", "C")
                } else {
                    ("ui.inventory.popup.keypad_digit", "{digit}")
                };
                spawn_bound_text_styled(
                    popup,
                    UserEquipUiElement::ItemPopupKeypadLabel(index),
                    UserEquipUiRect::new(left, top, 44.0, 19.0),
                    label,
                    key,
                    fallback,
                    assets.font.clone(),
                    16.0,
                    USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                    Color::srgba(0.8, 1.0, 1.0, 0.55),
                    Justify::Center,
                );
            }
            for index in 0..3 {
                popup.spawn((
                    UserEquipUiElement::ItemPopupGumNanoFrame(index),
                    Node {
                        display: Display::None,
                        ..USER_EQUIP_GUM_NANO_FRAME_RECTS[index].node()
                    },
                    stretched_image(assets.image(UserEquipStaticAssetRole::SlotOccupied)),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
                popup.spawn((
                    UserEquipUiElement::ItemPopupGumNanoIcon(index),
                    Node {
                        display: Display::None,
                        ..USER_EQUIP_GUM_NANO_FRAME_RECTS[index].node()
                    },
                    ImageNode::default(),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
                popup
                    .spawn((
                        Button,
                        UserEquipUiElement::ItemPopupGumNanoButton(index),
                        UserEquipGumNanoButtonControl(index),
                        Node {
                            display: Display::None,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..USER_EQUIP_GUM_NANO_BUTTON_RECTS[index].node()
                        },
                        sliced_image(
                            assets.image(UserEquipStaticAssetRole::ButtonNormal),
                            BorderRect {
                                min_inset: Vec2::new(6.0, 6.0),
                                max_inset: Vec2::new(6.0, 4.0),
                            },
                        ),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            UserEquipUiElement::ItemPopupGumNanoButtonLabel(index),
                            Text::new("GIVE"),
                            (
                                TextFont {
                                    font: (assets.font.clone()).into(),
                                    font_size: (14.0).into(),
                                    ..default()
                                },
                                LineHeight::Px(11.3),
                            ),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            LocalizedText::new("ui.inventory.action.give", "GIVE"),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ));
                    });
            }
            for index in 0..4 {
                popup
                    .spawn((
                        Button,
                        UserEquipUiElement::ItemPopupButton(index),
                        UserEquipPopupButtonControl(index),
                        Node {
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            left: px(160),
                            top: px(380),
                            width: px(130),
                            height: px(28),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        sliced_image(
                            assets.image(UserEquipStaticAssetRole::ButtonNormal),
                            BorderRect {
                                min_inset: Vec2::new(6.0, 6.0),
                                max_inset: Vec2::new(6.0, 4.0),
                            },
                        ),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            UserEquipUiElement::ItemPopupButtonLabel(index),
                            Text::new(""),
                            (
                                TextFont {
                                    font: (assets.font.clone()).into(),
                                    font_size: (14.0).into(),
                                    ..default()
                                },
                                LineHeight::Px(11.3),
                            ),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            LocalizedText::new("ui.content.passthrough", "{text}")
                                .with_arg("text", ""),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ));
                    });
            }
        });
}

pub(super) fn bind_user_equip_popup_button_text_color(
    popup: Res<UserEquipItemPopupState>,
    projection: Res<UserEquipItemModeProjection>,
    nano_projection: Res<UserEquipNanoModeProjection>,
    content: Option<Res<TutorialMissionContent>>,
    buttons: Query<(&Interaction, &Children, &UserEquipPopupButtonControl)>,
    gum_buttons: Query<(&Interaction, &Children, &UserEquipGumNanoButtonControl)>,
    mut labels: Query<(&UserEquipUiElement, &mut TextColor), Without<UserEquipPopupButtonControl>>,
) {
    let commands = popup.commands(&projection, content.as_deref());
    for (interaction, children, control) in &buttons {
        let hovered = matches!(interaction, Interaction::Hovered | Interaction::Pressed);
        let color = if commands.get(control.0) == Some(&UserEquipPopupCommand::Delete) {
            if hovered { Color::BLACK } else { Color::WHITE }
        } else if hovered {
            Color::srgb(0.23, 0.464, 1.0)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        };
        for child in children.iter() {
            if let Ok((UserEquipUiElement::ItemPopupButtonLabel(_), mut text_color)) =
                labels.get_mut(child)
            {
                text_color.0 = color;
            }
        }
    }
    for (interaction, children, control) in &gum_buttons {
        let enabled = popup
            .selected()
            .and_then(|endpoint| projection.item_at(endpoint))
            .zip(content.as_deref())
            .is_some_and(|(item, content)| {
                user_equip_gum_target_enabled(
                    item.item.item_id,
                    control.0,
                    &nano_projection,
                    content,
                )
            });
        let color = if !enabled {
            Color::srgba(0.9, 0.9, 0.9, 0.3)
        } else if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            Color::srgb(0.23, 0.464, 1.0)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        };
        for child in children.iter() {
            if let Ok((UserEquipUiElement::ItemPopupGumNanoButtonLabel(_), mut text_color)) =
                labels.get_mut(child)
            {
                text_color.0 = color;
            }
        }
    }
}

pub(super) fn user_equip_popup_variant_layout_key(
    element: UserEquipUiElement,
    variant: UserEquipPopupLayoutVariant,
) -> Option<String> {
    let prefix = match variant {
        UserEquipPopupLayoutVariant::Equip => return None,
        UserEquipPopupLayoutVariant::Unequip => "item_unequip_",
        UserEquipPopupLayoutVariant::GeneralStack => "item_general_stack_",
        UserEquipPopupLayoutVariant::GeneralGum => "item_general_gum_",
        UserEquipPopupLayoutVariant::GeneralUse => "item_general_use_",
        UserEquipPopupLayoutVariant::Chest => "item_chest_",
    };
    let suffix = match element {
        UserEquipUiElement::ItemPopupBackdrop => "backdrop".to_owned(),
        UserEquipUiElement::ItemPopupEquipInfo => "equip_info".to_owned(),
        UserEquipUiElement::ItemPopupIcon => "icon".to_owned(),
        UserEquipUiElement::ItemPopupTitle => "title".to_owned(),
        UserEquipUiElement::ItemPopupClose => "close".to_owned(),
        UserEquipUiElement::ItemPopupTrash => "trash".to_owned(),
        UserEquipUiElement::ItemPopupIdentity => "description".to_owned(),
        UserEquipUiElement::ItemPopupButton(index) => format!("button_{index}"),
        UserEquipUiElement::ItemPopupGumNanoFrame(index) => format!("nano_frame_{index}"),
        UserEquipUiElement::ItemPopupGumNanoIcon(index) => format!("nano_icon_{index}"),
        UserEquipUiElement::ItemPopupGumNanoButton(index) => format!("nano_button_{index}"),
        UserEquipUiElement::ItemPopupField(index) => format!("field_{index}"),
        _ => return None,
    };
    Some(format!("{prefix}{suffix}"))
}

pub(super) fn user_equip_popup_command_localized(command: UserEquipPopupCommand) -> LocalizedText {
    let (key, fallback) = match command {
        UserEquipPopupCommand::Equip => ("ui.inventory.action.equip", "EQUIP"),
        UserEquipPopupCommand::EquipPrimary => {
            ("ui.inventory.action.equip_primary", "EQUIP WEAPON 1")
        }
        UserEquipPopupCommand::EquipSecondary => {
            ("ui.inventory.action.equip_secondary", "EQUIP WEAPON 2")
        }
        UserEquipPopupCommand::Unequip => ("ui.inventory.action.unequip", "UNEQUIP"),
        UserEquipPopupCommand::Use => ("ui.inventory.action.use", "USE ITEM"),
        UserEquipPopupCommand::Open => ("ui.inventory.action.open", "OPEN"),
        UserEquipPopupCommand::Delete => ("ui.inventory.action.delete", "DELETE"),
        UserEquipPopupCommand::Cancel => ("ui.common.cancel", "CANCEL"),
    };
    LocalizedText::new(key, fallback)
}

#[must_use]
pub(super) const fn user_equip_popup_field_rect(
    index: usize,
    variant: Option<UserEquipPopupLayoutVariant>,
) -> Option<UserEquipUiRect> {
    let rect = match (index, variant) {
        (0, _) => UserEquipUiRect::new(82.0, 50.0, 170.0, 20.0),
        (1, _) => UserEquipUiRect::new(5.0, 140.0, 200.0, 20.0),
        (2, _) => UserEquipUiRect::new(28.0, 219.0, 80.0, 20.0),
        (3, Some(UserEquipPopupLayoutVariant::Unequip)) => {
            UserEquipUiRect::new(110.0, 219.0, 80.0, 20.0)
        }
        (3, _) => UserEquipUiRect::new(115.0, 219.0, 80.0, 20.0),
        (4, Some(UserEquipPopupLayoutVariant::Unequip)) => {
            UserEquipUiRect::new(192.0, 219.0, 80.0, 20.0)
        }
        (4, _) => UserEquipUiRect::new(207.0, 219.0, 80.0, 20.0),
        (5, _) => UserEquipUiRect::new(10.0, 238.0, 200.0, 20.0),
        (6, _) => UserEquipUiRect::new(8.0, 256.0, 130.0, 20.0),
        (7, _) => UserEquipUiRect::new(8.0, 276.0, 130.0, 20.0),
        (8, _) => UserEquipUiRect::new(8.0, 296.0, 130.0, 20.0),
        (9, _) => UserEquipUiRect::new(8.0, 316.0, 130.0, 20.0),
        (10, _) => UserEquipUiRect::new(150.0, 260.0, 130.0, 20.0),
        (11, _) => UserEquipUiRect::new(150.0, 280.0, 130.0, 20.0),
        (12, _) => UserEquipUiRect::new(150.0, 300.0, 130.0, 20.0),
        (13, _) => UserEquipUiRect::new(150.0, 320.0, 130.0, 20.0),
        _ => return None,
    };
    Some(rect.translated(0.0, user_equip_popup_content_y_offset(variant)))
}

#[must_use]
pub(super) const fn user_equip_popup_command_rect(
    command: UserEquipPopupCommand,
    variant: Option<UserEquipPopupLayoutVariant>,
) -> UserEquipUiRect {
    let rect = match command {
        UserEquipPopupCommand::EquipPrimary => UserEquipUiRect::new(120.0, 360.0, 170.0, 28.0),
        UserEquipPopupCommand::EquipSecondary => UserEquipUiRect::new(120.0, 390.0, 170.0, 28.0),
        UserEquipPopupCommand::Equip | UserEquipPopupCommand::Unequip => {
            UserEquipUiRect::new(160.0, 380.0, 130.0, 28.0)
        }
        // Clean `Use`/`Delete` ownership varies by PopupControll subtype. Use
        // stays on the primary action rail; delete is the exact trash control.
        UserEquipPopupCommand::Use | UserEquipPopupCommand::Open => {
            UserEquipUiRect::new(160.0, 180.0, 130.0, 28.0)
        }
        UserEquipPopupCommand::Delete
            if matches!(
                variant,
                Some(
                    UserEquipPopupLayoutVariant::GeneralStack
                        | UserEquipPopupLayoutVariant::GeneralGum
                )
            ) =>
        {
            UserEquipUiRect::new(79.0, 301.0, 150.0, 25.0)
        }
        UserEquipPopupCommand::Delete => UserEquipUiRect::new(10.0, 375.0, 32.0, 32.0),
        UserEquipPopupCommand::Cancel => UserEquipUiRect::new(277.0, 0.0, 32.0, 32.0),
    };
    rect.translated(0.0, user_equip_popup_content_y_offset(variant))
}

#[must_use]
pub(super) const fn user_equip_popup_content_y_offset(variant: Option<UserEquipPopupLayoutVariant>) -> f32 {
    match variant {
        Some(UserEquipPopupLayoutVariant::Equip) => 14.0,
        Some(UserEquipPopupLayoutVariant::Unequip) => 13.0,
        _ => 0.0,
    }
}
