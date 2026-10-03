//! Binding of the item popup frame, fields, targets, buttons and calculator.

use super::super::asset_contract::UserEquipStaticAssetRole;
use super::super::components::UserEquipUiElement;
use super::super::geometry::{
    USER_EQUIP_GUM_NANO_BUTTON_RECTS, USER_EQUIP_GUM_NANO_FRAME_RECTS, UserEquipUiRect,
};
use super::super::images::{bind_rect, presentation_icon_handle, sliced_image};
use super::super::item_popup::{
    user_equip_popup_command_localized, user_equip_popup_command_rect,
    user_equip_popup_content_y_offset, user_equip_popup_field_rect,
};
use super::super::layout::user_equip_popup_rects;
use super::super::popup_state::{
    UserEquipPopupCommand, UserEquipPopupLayoutVariant, user_equip_gum_target_enabled,
};
use super::super::view_model::{
    user_equip_display_text_id, user_equip_item_type_localized, user_equip_range_localized,
    user_equip_rarity_localized, user_equip_weapon_type_localized,
};
use super::UserEquipBindContext;
use crate::localization::LocalizedText;
use bevy::{prelude::*, sprite::BorderRect};

/// Item popup frame, backdrop, title, icon, identity and equip info.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_item_popup_frame_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    localized: Option<Mut<LocalizedText>>,
    text_color: Option<Mut<TextColor>>,
    text_layout: Option<Mut<TextLayout>>,
    _interaction: Option<&Interaction>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        modal,
        popup,
        projection,
        environment,
        view,
        nano_mode,
        popup_layout_variant,
        endpoint_icon,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::ItemPopup => {
            node.display =
                if !nano_mode && modal.item_popup_active && popup.selected().is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            if let Some(endpoint) = popup.selected() {
                let rect = match popup_layout_variant {
                    Some(
                        UserEquipPopupLayoutVariant::GeneralStack
                        | UserEquipPopupLayoutVariant::GeneralGum,
                    ) => UserEquipUiRect::new(
                        view.layout.pc_stuff_panel.left + 25.0,
                        view.layout.pc_stuff_panel.top + 90.0,
                        310.0,
                        355.0,
                    ),
                    Some(
                        UserEquipPopupLayoutVariant::GeneralUse
                        | UserEquipPopupLayoutVariant::Chest,
                    ) => UserEquipUiRect::new(
                        view.layout.pc_stuff_panel.left + 25.0,
                        view.layout.pc_stuff_panel.top + 90.0,
                        310.0,
                        235.0,
                    ),
                    _ => user_equip_popup_rects(view.layout, endpoint).0,
                };
                bind_rect(&mut node, rect);
            }
        }
        UserEquipUiElement::ItemPopupBackdrop => {
            if let (Some(variant), Some(mut image)) = (popup_layout_variant, image) {
                let (rect, role) = match variant {
                    UserEquipPopupLayoutVariant::Equip => (
                        UserEquipUiRect::new(0.0, 14.0, 310.0, 435.0),
                        UserEquipStaticAssetRole::EquipPopup,
                    ),
                    UserEquipPopupLayoutVariant::Unequip => (
                        UserEquipUiRect::new(0.0, 0.0, 310.0, 448.0),
                        UserEquipStaticAssetRole::UnequipPopup,
                    ),
                    UserEquipPopupLayoutVariant::GeneralStack
                    | UserEquipPopupLayoutVariant::GeneralGum => (
                        UserEquipUiRect::new(0.0, 0.0, 310.0, 355.0),
                        UserEquipStaticAssetRole::GeneralDialog,
                    ),
                    UserEquipPopupLayoutVariant::GeneralUse
                    | UserEquipPopupLayoutVariant::Chest => (
                        UserEquipUiRect::new(0.0, 0.0, 310.0, 235.0),
                        UserEquipStaticAssetRole::UseDialog,
                    ),
                };
                bind_rect(&mut node, rect);
                image.image = assets.0.image(role);
            }
        }
        UserEquipUiElement::ItemPopupTitle => {
            bind_rect(
                &mut node,
                UserEquipUiRect::new(82.0, 16.0, 170.0, 40.0)
                    .translated(0.0, user_equip_popup_content_y_offset(popup_layout_variant)),
            );
            if let Some(mut text_color) = text_color {
                text_color.0 = if matches!(
                    popup_layout_variant,
                    Some(
                        UserEquipPopupLayoutVariant::GeneralStack
                            | UserEquipPopupLayoutVariant::GeneralGum
                    )
                ) {
                    Color::srgb(1.0, 1.0, 0.0)
                } else {
                    Color::srgb(0.8, 1.0, 1.0)
                };
            }
            if let Some(mut localized) = localized {
                if let Some(item) = popup.selected().and_then(|endpoint| {
                    projection
                        .as_deref()
                        .and_then(|items| items.item_at(endpoint))
                }) {
                    let value = environment
                        .content
                        .as_deref()
                        .and_then(|content| {
                            content.gameplay_user_equip_item_text(
                                item.item.item_type,
                                user_equip_display_text_id(item.item),
                            )
                        })
                        .map(|v| v.0)
                        .unwrap_or_else(|| {
                            LocalizedText::new("ui.inventory.popup.item_name", "{name}")
                                .with_arg("name", item.item.item_id.to_string())
                        });
                    if *localized != value {
                        *localized = value;
                    }
                }
            }
        }
        UserEquipUiElement::ItemPopupIconFrame => {
            bind_rect(
                &mut node,
                UserEquipUiRect::new(16.0, 16.0, 64.0, 64.0)
                    .translated(0.0, user_equip_popup_content_y_offset(popup_layout_variant)),
            );
            node.display = if matches!(
                popup_layout_variant,
                Some(
                    UserEquipPopupLayoutVariant::Equip
                        | UserEquipPopupLayoutVariant::Unequip
                        | UserEquipPopupLayoutVariant::GeneralStack
                        | UserEquipPopupLayoutVariant::GeneralGum
                )
            ) {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::ItemPopupIcon => {
            let content_y = user_equip_popup_content_y_offset(popup_layout_variant);
            bind_rect(
                &mut node,
                if matches!(
                    popup_layout_variant,
                    Some(
                        UserEquipPopupLayoutVariant::GeneralUse
                            | UserEquipPopupLayoutVariant::Chest
                    )
                ) {
                    UserEquipUiRect::new(16.0, 12.0, 68.0, 68.0)
                } else {
                    UserEquipUiRect::new(16.0, 16.0, 64.0, 64.0)
                }
                .translated(0.0, content_y),
            );
            if let (Some(mut image), Some(icon)) =
                (image, popup.selected().and_then(endpoint_icon))
            {
                let (mut handle, mut visible) =
                    presentation_icon_handle(icon, &asset_server, &assets.0);
                if let Some(path) = popup
                    .selected()
                    .and_then(|e| projection.as_deref()?.item_at(e))
                    .and_then(|i| {
                        environment
                            .content
                            .as_deref()?
                            .gameplay_item_display_icon(i.item)
                    })
                {
                    handle = asset_server.load(path.to_owned());
                    visible = true;
                }
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
                image.image = handle;
            } else {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::ItemPopupIdentity => {
            bind_rect(
                &mut node,
                UserEquipUiRect::new(12.0, 95.0, 280.0, 40.0)
                    .translated(0.0, user_equip_popup_content_y_offset(popup_layout_variant)),
            );
            if let Some(mut text_color) = text_color {
                text_color.0 = if matches!(
                    popup_layout_variant,
                    Some(
                        UserEquipPopupLayoutVariant::GeneralUse
                            | UserEquipPopupLayoutVariant::Chest
                    )
                ) {
                    Color::srgb(0.21, 1.0, 1.0)
                } else {
                    Color::WHITE
                };
            }
            node.overflow = Overflow::clip();
            if let Some(mut layout) = text_layout {
                layout.linebreak = LineBreak::WordBoundary;
            }
            if let Some(mut localized) = localized
                && let Some(item) = popup.selected().and_then(|endpoint| {
                    projection
                        .as_deref()
                        .and_then(|items| items.item_at(endpoint))
                })
            {
                let value = environment
                    .content
                    .as_deref()
                    .and_then(|content| {
                        content.gameplay_user_equip_item_text(
                            item.item.item_type,
                            user_equip_display_text_id(item.item),
                        )
                    })
                    .map(|v| v.1)
                    .unwrap_or_else(|| {
                        LocalizedText::new("ui.inventory.popup.description", "{description}")
                            .with_arg("description", "")
                    });
                if *localized != value {
                    *localized = value;
                }
            }
        }
        UserEquipUiElement::ItemPopupEquipInfo => {
            bind_rect(
                &mut node,
                match popup_layout_variant {
                    Some(UserEquipPopupLayoutVariant::Unequip) => {
                        UserEquipUiRect::new(2.0, 170.0, 305.0, 84.0)
                    }
                    _ => UserEquipUiRect::new(2.0, 171.0, 305.0, 84.0),
                },
            );
            node.display = if matches!(
                popup_layout_variant,
                Some(UserEquipPopupLayoutVariant::Equip | UserEquipPopupLayoutVariant::Unequip)
            ) && popup
                .selected()
                .and_then(|endpoint| {
                    projection
                        .as_deref()
                        .and_then(|items| items.item_at(endpoint))
                })
                .is_some_and(|item| item.item.item_type != 10)
            {
                Display::Flex
            } else {
                Display::None
            };
        }
        _ => {}
    }
}

/// Item popup fields, Gum/Nano targets, command buttons, trash and calculator.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_item_popup_control_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    localized: Option<Mut<LocalizedText>>,
    text_color: Option<Mut<TextColor>>,
    _text_layout: Option<Mut<TextLayout>>,
    interaction: Option<&Interaction>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        popup,
        projection,
        nano_projection,
        environment,
        popup_commands,
        popup_layout_variant,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::ItemPopupField(index) => {
            if let Some(rect) = user_equip_popup_field_rect(index, popup_layout_variant) {
                bind_rect(&mut node, rect);
            }
            let selected = popup.selected().and_then(|endpoint| {
                projection
                    .as_deref()
                    .and_then(|items| items.item_at(endpoint))
            });
            if let (Some(item), Some(mut localized)) = (selected, localized) {
                node.display = match popup_layout_variant {
                    Some(
                        UserEquipPopupLayoutVariant::GeneralStack
                        | UserEquipPopupLayoutVariant::GeneralGum
                        | UserEquipPopupLayoutVariant::GeneralUse,
                    ) => {
                        if index == 0 {
                            Display::Flex
                        } else {
                            Display::None
                        }
                    }
                    Some(UserEquipPopupLayoutVariant::Chest) | None => Display::None,
                    Some(
                        UserEquipPopupLayoutVariant::Equip
                        | UserEquipPopupLayoutVariant::Unequip,
                    ) => {
                        if index == 1 || (matches!(index, 2..=4) && item.item.item_type == 10) {
                            Display::None
                        } else {
                            Display::Flex
                        }
                    }
                };
                let detail = environment.content.as_deref().and_then(|content| {
                    content
                        .gameplay_user_equip_item_detail(item.item.item_type, item.item.item_id)
                });
                if let Some(mut color) = text_color {
                    let combined = (0..=3).contains(&item.item.item_type)
                        && (item.item.option >> 16) as i16 > 0;
                    if index == 13 {
                        color.0 = if !combined && detail.as_ref().is_some_and(|d| d.tradeable) {
                            Color::srgb(0., 1., 0.)
                        } else {
                            Color::srgb(1., 0., 0.)
                        };
                    }
                    if index == 11 {
                        color.0 = if item.item.item_type != 0 && item.item.item_type != 10 {
                            Color::srgb(0., 0., 1.)
                        } else {
                            Color::WHITE
                        };
                    }
                }
                *localized = match index {
                    0 if matches!(
                        popup_layout_variant,
                        Some(
                            UserEquipPopupLayoutVariant::GeneralStack
                                | UserEquipPopupLayoutVariant::GeneralGum
                        )
                    ) =>
                    {
                        LocalizedText::new("ui.inventory.popup.cost", "COST {cost}").with_arg(
                            "cost",
                            detail.as_ref().map_or(0, |detail| detail.level).to_string(),
                        )
                    }
                    0 => LocalizedText::new("ui.inventory.popup.level", "Level {level}")
                        .with_arg(
                            "level",
                            detail.as_ref().map_or(0, |detail| detail.level).to_string(),
                        ),
                    1 => LocalizedText::new("ui.inventory.popup.status", "STATUS"),
                    2 => LocalizedText::new("ui.content.passthrough", "{text}").with_arg(
                        "text",
                        detail
                            .as_ref()
                            .map_or(0, |detail| detail.point_rating)
                            .to_string(),
                    ),
                    3 => LocalizedText::new("ui.content.passthrough", "{text}").with_arg(
                        "text",
                        detail
                            .as_ref()
                            .map_or(0, |detail| detail.group_rating)
                            .to_string(),
                    ),
                    4 => LocalizedText::new("ui.content.passthrough", "{text}").with_arg(
                        "text",
                        detail
                            .as_ref()
                            .map_or(0, |detail| detail.defense_rating)
                            .to_string(),
                    ),
                    5 => LocalizedText::new("ui.inventory.popup.info", "INFO"),
                    6 => LocalizedText::new("ui.inventory.popup.type", "Type"),
                    7 => LocalizedText::new(
                        if item.item.item_type == 10 {
                            "ui.inventory.popup.speed"
                        } else {
                            "ui.inventory.popup.range"
                        },
                        if item.item.item_type == 10 {
                            "Speed"
                        } else {
                            "Range"
                        },
                    ),
                    8 => LocalizedText::new("ui.inventory.popup.rarity", "Rarity"),
                    9 => LocalizedText::new("ui.inventory.popup.trade", "Trade Availability"),
                    10 if item.item.item_type == 0 => user_equip_weapon_type_localized(
                        detail.as_ref().and_then(|detail| detail.target_mode),
                    ),
                    10 => user_equip_item_type_localized(item.item.item_type),
                    11 if item.item.item_type == 0 => user_equip_range_localized(
                        detail.as_ref().and_then(|detail| detail.equip_type),
                    ),
                    11 if item.item.item_type == 10 => {
                        LocalizedText::new("ui.inventory.vehicle_class", "{speed} Class")
                            .with_arg(
                                "speed",
                                detail
                                    .as_ref()
                                    .and_then(|detail| detail.vehicle_speed_class)
                                    .unwrap_or_default()
                                    .to_string(),
                            )
                    }
                    11 => LocalizedText::new("ui.inventory.popup.not_available", "N/A"),
                    12 if (0..=3).contains(&item.item.item_type)
                        && (item.item.option >> 16) as i16 > 0 =>
                    {
                        LocalizedText::new("ui.inventory.rarity.special", "Special")
                    }
                    12 => user_equip_rarity_localized(
                        detail.as_ref().and_then(|detail| detail.rarity),
                    ),
                    13 if !((0..=3).contains(&item.item.item_type)
                        && (item.item.option >> 16) as i16 > 0)
                        && detail.as_ref().is_some_and(|detail| detail.tradeable) =>
                    {
                        LocalizedText::new("ui.inventory.popup.trade_value", "Tradable")
                    }
                    13 => LocalizedText::new("ui.inventory.popup.not_tradable", "Not tradable"),
                    _ => LocalizedText::new("ui.content.passthrough", "{text}")
                        .with_arg("text", ""),
                };
            }
        }
        UserEquipUiElement::ItemPopupGumNanoFrame(index) => {
            bind_rect(&mut node, USER_EQUIP_GUM_NANO_FRAME_RECTS[index]);
            node.display =
                if popup_layout_variant == Some(UserEquipPopupLayoutVariant::GeneralGum) {
                    Display::Flex
                } else {
                    Display::None
                };
        }
        UserEquipUiElement::ItemPopupGumNanoIcon(index) => {
            bind_rect(&mut node, USER_EQUIP_GUM_NANO_FRAME_RECTS[index]);
            if popup_layout_variant == Some(UserEquipPopupLayoutVariant::GeneralGum)
                && nano_projection.status[index].nano_id.is_some()
            {
                if let Some(mut image) = image {
                    let (handle, visible) = presentation_icon_handle(
                        &nano_projection.status[index].nano_icon,
                        &asset_server,
                        &assets.0,
                    );
                    node.display = if visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    image.image = handle;
                    image.color = Color::WHITE;
                }
            } else {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::ItemPopupGumNanoButton(index) => {
            bind_rect(&mut node, USER_EQUIP_GUM_NANO_BUTTON_RECTS[index]);
            let has_nano = popup_layout_variant
                == Some(UserEquipPopupLayoutVariant::GeneralGum)
                && nano_projection.status[index].nano_id.is_some();
            node.display = if has_nano {
                Display::Flex
            } else {
                Display::None
            };
            let enabled = has_nano
                && popup
                    .selected()
                    .and_then(|endpoint| {
                        projection
                            .as_deref()
                            .and_then(|items| items.item_at(endpoint))
                    })
                    .zip(environment.content.as_deref())
                    .is_some_and(|(item, content)| {
                        user_equip_gum_target_enabled(
                            item.item.item_id,
                            index,
                            nano_projection,
                            content,
                        )
                    });
            if let Some(mut image) = image {
                let hovered = enabled
                    && interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    });
                *image = sliced_image(
                    assets.0.image(if hovered {
                        UserEquipStaticAssetRole::ButtonHover
                    } else {
                        UserEquipStaticAssetRole::ButtonNormal
                    }),
                    BorderRect {
                        min_inset: Vec2::new(6.0, 6.0),
                        max_inset: Vec2::new(6.0, 4.0),
                    },
                );
                image.color = if enabled {
                    Color::WHITE
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.3)
                };
            }
        }
        UserEquipUiElement::ItemPopupGumNanoButtonLabel(index) => {
            let has_nano = popup_layout_variant
                == Some(UserEquipPopupLayoutVariant::GeneralGum)
                && nano_projection.status[index].nano_id.is_some();
            node.display = if has_nano {
                Display::Flex
            } else {
                Display::None
            };
            let enabled = has_nano
                && popup
                    .selected()
                    .and_then(|endpoint| {
                        projection
                            .as_deref()
                            .and_then(|items| items.item_at(endpoint))
                    })
                    .zip(environment.content.as_deref())
                    .is_some_and(|(item, content)| {
                        user_equip_gum_target_enabled(
                            item.item.item_id,
                            index,
                            nano_projection,
                            content,
                        )
                    });
            if let Some(mut text_color) = text_color {
                text_color.0 = if enabled {
                    Color::srgb(0.9, 0.9, 0.9)
                } else {
                    Color::srgba(0.9, 0.9, 0.9, 0.3)
                };
            }
        }
        UserEquipUiElement::ItemPopupButton(index) => {
            if let Some(command) = popup_commands.get(index).copied() {
                node.display = Display::Flex;
                bind_rect(
                    &mut node,
                    user_equip_popup_command_rect(command, popup_layout_variant),
                );
                if let Some(mut image) = image {
                    let hovered = interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    });
                    let red = command == UserEquipPopupCommand::Delete
                        && matches!(
                            popup_layout_variant,
                            Some(
                                UserEquipPopupLayoutVariant::GeneralStack
                                    | UserEquipPopupLayoutVariant::GeneralGum
                            )
                        );
                    *image = sliced_image(
                        assets.0.image(if red {
                            if hovered {
                                UserEquipStaticAssetRole::RedButtonHover
                            } else {
                                UserEquipStaticAssetRole::RedButton
                            }
                        } else if hovered {
                            UserEquipStaticAssetRole::ButtonHover
                        } else {
                            UserEquipStaticAssetRole::ButtonNormal
                        }),
                        if red {
                            BorderRect::all(5.0)
                        } else {
                            BorderRect {
                                min_inset: Vec2::new(6.0, 6.0),
                                max_inset: Vec2::new(6.0, 4.0),
                            }
                        },
                    );
                }
            } else {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::ItemPopupTrash => {
            node.display = match popup_layout_variant {
                Some(UserEquipPopupLayoutVariant::Chest) => {
                    bind_rect(&mut node, UserEquipUiRect::new(10.0, 180.0, 32.0, 32.0));
                    Display::Flex
                }
                Some(
                    UserEquipPopupLayoutVariant::Equip | UserEquipPopupLayoutVariant::Unequip,
                ) => {
                    bind_rect(
                        &mut node,
                        UserEquipUiRect::new(10.0, 375.0, 32.0, 32.0).translated(
                            0.0,
                            user_equip_popup_content_y_offset(popup_layout_variant),
                        ),
                    );
                    Display::Flex
                }
                _ => Display::None,
            };
        }
        UserEquipUiElement::ItemPopupButtonLabel(index) => {
            if let Some(mut localized) = localized {
                *localized = popup_commands
                    .get(index)
                    .copied()
                    .map(user_equip_popup_command_localized)
                    .unwrap_or_else(|| {
                        LocalizedText::new("ui.content.passthrough", "{text}")
                            .with_arg("text", "")
                    });
            }
        }
        UserEquipUiElement::ItemPopupCalculatorBack
        | UserEquipUiElement::ItemPopupAmountLabel
        | UserEquipUiElement::ItemPopupAmountValue
        | UserEquipUiElement::ItemPopupKeypadLabel(_) => {
            node.display =
                if popup_layout_variant == Some(UserEquipPopupLayoutVariant::GeneralStack) {
                    Display::Flex
                } else {
                    Display::None
                };
            if let Some(mut localized) = localized {
                match *element {
                    UserEquipUiElement::ItemPopupAmountValue => {
                        let amount = popup
                            .selected()
                            .and_then(|endpoint| {
                                projection
                                    .as_deref()
                                    .and_then(|items| items.item_at(endpoint))
                            })
                            .map_or(0, |item| item.item.option.max(0));
                        *localized =
                            LocalizedText::new("ui.inventory.popup.amount_value", "{amount}")
                                .with_arg("amount", amount.to_string());
                    }
                    UserEquipUiElement::ItemPopupKeypadLabel(index) => {
                        const LABELS: [&str; 11] =
                            ["1", "2", "3", "4", "5", "6", "7", "8", "9", "C", "0"];
                        let label = LABELS.get(index).copied().unwrap_or_default();
                        *localized = if label == "C" {
                            LocalizedText::new("ui.inventory.popup.clear_short", "C")
                        } else {
                            LocalizedText::new("ui.inventory.popup.keypad_digit", "{digit}")
                                .with_arg("digit", label)
                        };
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
