//! Binding of the UserEquip window frame, scroll views, buttons, help panel and drag image.

use super::super::asset_contract::UserEquipStaticAssetRole;
use super::super::components::UserEquipUiElement;
use super::super::geometry::{
    USER_EQUIP_INVENTORY_CONTENT_HEIGHT, USER_EQUIP_INVENTORY_CONTENT_WIDTH,
    USER_EQUIP_INVENTORY_PANEL_BORDER, USER_EQUIP_INVENTORY_SLOT_SIZE,
    USER_EQUIP_NANO_CONTENT_HEIGHT, USER_EQUIP_NANO_PANEL_BORDER, USER_EQUIP_SCROLL_TRACK_RECT,
    UserEquipUiRect,
};
use super::super::images::{bind_rect, presentation_icon_handle, sliced_image};
use super::super::item_popup::user_equip_popup_content_y_offset;
use super::super::layout::UserEquipScrollbarMetrics;
use super::super::state::{UserEquipAvatarTurnDirection, user_equip_avatar_turn_asset_role};
use super::UserEquipBindContext;
use crate::localization::LocalizedText;
use bevy::prelude::*;

/// Window frame, backplates, avatar preview, scroll views, shadows, buttons, help panel and drag image.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_window_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    _localized: Option<Mut<LocalizedText>>,
    _text_color: Option<Mut<TextColor>>,
    _text_layout: Option<Mut<TextLayout>>,
    interaction: Option<&Interaction>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        state,
        modal,
        drag,
        avatar_presentation,
        inventory_preview,
        window,
        view,
        nano_mode,
        popup_layout_variant,
        endpoint_icon,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::Backdrop => bind_rect(&mut node, view.layout.full_backdrop),
        UserEquipUiElement::ClothesBackplate => {
            bind_rect(&mut node, view.layout.clothes_backplate)
        }
        UserEquipUiElement::RightBackplate => {
            bind_rect(&mut node, view.layout.pc_stuff_backplate)
        }
        UserEquipUiElement::UserClothesPanel => {
            bind_rect(&mut node, view.layout.user_clothes_panel)
        }
        UserEquipUiElement::AvatarPreview => {
            bind_rect(&mut node, avatar_presentation.avatar_rect());
            if let (Some(mut image), Some(preview)) = (image, inventory_preview.as_deref()) {
                image.image = preview.0.clone();
            } else if inventory_preview.is_none() {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::TurnLeftPositioned | UserEquipUiElement::TurnRightPositioned => {
            let direction = if *element == UserEquipUiElement::TurnLeftPositioned {
                UserEquipAvatarTurnDirection::LeftPositioned
            } else {
                UserEquipAvatarTurnDirection::RightPositioned
            };
            if let Some(mut image) = image {
                image.image = assets.0.image(user_equip_avatar_turn_asset_role(
                    direction,
                    interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    }),
                ));
            }
        }
        UserEquipUiElement::PcStuffPanel => {
            bind_rect(&mut node, view.layout.pc_stuff_panel);
        }
        UserEquipUiElement::PcStuffPanelBackground => {
            if let Some(mut image) = image {
                *image = sliced_image(
                    assets.0.image(if nano_mode {
                        UserEquipStaticAssetRole::NanoBack
                    } else {
                        UserEquipStaticAssetRole::InventoryPanel
                    }),
                    if nano_mode {
                        USER_EQUIP_NANO_PANEL_BORDER
                    } else {
                        USER_EQUIP_INVENTORY_PANEL_BORDER
                    },
                );
            }
        }
        UserEquipUiElement::EquipmentPanel => bind_rect(&mut node, view.layout.equipment_panel),
        UserEquipUiElement::InventoryContent => {
            node.left = px(0);
            node.top = px(-view.layout.scroll_y);
            node.width = px(USER_EQUIP_INVENTORY_CONTENT_WIDTH);
            node.height = px(USER_EQUIP_INVENTORY_CONTENT_HEIGHT);
        }
        UserEquipUiElement::InventoryViewport => {
            node.display = if nano_mode {
                Display::None
            } else {
                Display::Flex
            };
        }
        UserEquipUiElement::NanoViewport => {
            node.display = if nano_mode {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoContent => {
            node.left = px(0);
            node.top = px(-view.layout.scroll_y);
            node.width = px(USER_EQUIP_INVENTORY_CONTENT_WIDTH);
            node.height = px(USER_EQUIP_NANO_CONTENT_HEIGHT);
        }
        UserEquipUiElement::ScrollTrack
        | UserEquipUiElement::ScrollUp
        | UserEquipUiElement::ScrollDown => {}
        UserEquipUiElement::ScrollThumb => {
            let metrics = UserEquipScrollbarMetrics::for_mode(state.mode());
            node.left = px(357.0);
            node.top =
                px(USER_EQUIP_SCROLL_TRACK_RECT.top
                    + metrics.thumb_offset(view.layout.scroll_y));
            node.width = px(13.0);
            node.height = px(metrics.thumb_height);
        }

        UserEquipUiElement::InventoryShadowA => {
            node.display = if nano_mode {
                Display::None
            } else {
                Display::Flex
            };
        }
        UserEquipUiElement::InventoryShadowB => {
            node.display = if nano_mode {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::Close => {
            if let Some(mut image) = image {
                image.image = assets.0.image(
                    if interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    }) {
                        UserEquipStaticAssetRole::CloseHover
                    } else {
                        UserEquipStaticAssetRole::Close
                    },
                );
            }
        }
        UserEquipUiElement::ItemPopupClose => {
            bind_rect(
                &mut node,
                UserEquipUiRect::new(277.0, 0.0, 32.0, 32.0)
                    .translated(0.0, user_equip_popup_content_y_offset(popup_layout_variant)),
            );
            if let Some(mut image) = image {
                image.image = assets.0.image(
                    if interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    }) {
                        UserEquipStaticAssetRole::CloseHover
                    } else {
                        UserEquipStaticAssetRole::Close
                    },
                );
            }
        }
        UserEquipUiElement::Trash => {
            if let Some(mut image) = image {
                image.image = assets.0.image(
                    if interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    }) {
                        UserEquipStaticAssetRole::TrashHover
                    } else {
                        UserEquipStaticAssetRole::Trash
                    },
                );
            }
        }
        UserEquipUiElement::Help => {
            if let Some(mut image) = image {
                image.image = assets.0.image(
                    if interaction.is_some_and(|interaction| {
                        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                    }) {
                        UserEquipStaticAssetRole::HelpHover
                    } else {
                        UserEquipStaticAssetRole::Help
                    },
                );
            }
        }

        UserEquipUiElement::HelpPanel => {
            node.display = if modal.help_active {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::DraggedItem => {
            if let (Some(source), Some(cursor), Some(mut image)) =
                (drag.source(), window.cursor_position(), image)
                && let Some(icon) = endpoint_icon(source)
            {
                let (handle, visible) =
                    presentation_icon_handle(icon, &asset_server, &assets.0);
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
                node.left = px(cursor.x - USER_EQUIP_INVENTORY_SLOT_SIZE * 0.5);
                node.top = px(cursor.y - USER_EQUIP_INVENTORY_SLOT_SIZE * 0.5);
                image.image = handle;
                image.color = Color::WHITE;
            } else {
                node.display = Display::None;
            }
        }
        _ => {}
    }
}
