//! Binding of Nano gallery, inventory and equipment slots.

use super::super::catalog::UserEquipSlotEndpoint;
use super::super::components::UserEquipUiElement;
use super::super::images::{presentation_icon_handle, slot_frame_image};
use super::super::view_model::UserEquipPresentationIcon;
use super::{UserEquipBindContext, inventory_count_localized};
use crate::localization::LocalizedText;
use bevy::prelude::*;

/// Nano gallery, inventory and equipment slot frames, icons, badges and counts.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_slot_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    localized: Option<Mut<LocalizedText>>,
    _text_color: Option<Mut<TextColor>>,
    _text_layout: Option<Mut<TextLayout>>,
    _interaction: Option<&Interaction>,
    background: Option<Mut<BackgroundColor>>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        drag,
        view,
        nano_view,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::NanoSlotFrame(slot) => {
            node.display = if nano_view.get(slot).is_some() {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoSlotIcon(slot) => {
            if let Some(mut image) = image {
                let icon = nano_view
                    .get(slot)
                    .map(|entry| &entry.icon)
                    .unwrap_or(&UserEquipPresentationIcon::Empty);
                let (handle, visible) = presentation_icon_handle(icon, &asset_server, &assets.0);
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
                image.image = handle;
            }
        }
        UserEquipUiElement::InventorySlotFrame(slot) => {
            if let Some(mut background) = background {
                background.0 = if inventory_slot_available(context, slot) {
                    Color::NONE
                } else {
                    Color::srgba(0.85, 0.02, 0.02, 0.8)
                };
            }
            if let Some(mut image) = image {
                image.image = slot_frame_image(view.inventory[slot].frame_visual, &assets.0);
                image.color = if inventory_slot_available(context, slot) { Color::WHITE } else { Color::srgba(1.0, 0.08, 0.08, 0.4) };
            }
        }
        UserEquipUiElement::InventorySlotIcon(slot) => {
            if let Some(mut image) = image {
                let (handle, visible) =
                    presentation_icon_handle(&view.inventory[slot].icon, &asset_server, &assets.0);
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
                image.image = handle;
                image.color = inventory_icon_color(
                    drag.source() == Some(UserEquipSlotEndpoint::Inventory { slot_index: slot }),
                );
            }
        }
        UserEquipUiElement::InventorySlotBadge(slot) => {
            node.display = if view.inventory[slot].combined_badge.is_some() {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::InventorySlotCount(slot) => {
            if let Some(mut localized) = localized {
                if let Some(label) = view.inventory[slot].count_label.as_deref() {
                    node.display = Display::Flex;
                    *localized = inventory_count_localized(label);
                } else {
                    node.display = Display::None;
                    *localized = inventory_count_localized("");
                }
            }
        }
        UserEquipUiElement::EquipmentSlotFrame(slot) => {
            if let Some(mut image) = image {
                image.image = slot_frame_image(view.equipment[slot].frame_visual, &assets.0);
            }
        }
        UserEquipUiElement::EquipmentSlotIcon(slot) => {
            if let Some(mut image) = image {
                let (handle, visible) =
                    presentation_icon_handle(&view.equipment[slot].icon, &asset_server, &assets.0);
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
                image.image = handle;
                image.color = if drag.source().is_some_and(|source| {
                    matches!(source, UserEquipSlotEndpoint::Equipment { visual_index, .. } if visual_index == slot)
                }) {
                    Color::srgba(1.0, 1.0, 1.0, 0.5)
                } else {
                    Color::WHITE
                };
            }
        }
        UserEquipUiElement::EquipmentSlotBadge(slot) => {
            node.display = if view.equipment[slot].combined_badge.is_some() {
                Display::Flex
            } else {
                Display::None
            };
        }
        _ => {}
    }
}

fn inventory_slot_available(context: &UserEquipBindContext<'_, '_, '_>, slot: usize) -> bool {
    context.environment.content.as_deref().is_none_or(|content| {
        context.projection.as_deref().is_none_or(|projection| {
            let item = &projection.inventory[slot].item;
            item.empty || content.gameplay_inventory_item_available(
                item.item, context.presentation.level, context.presentation.gender,
                context.presentation.guide,
            )
        })
    })
}

fn inventory_icon_color(dragging: bool) -> Color {
    let alpha = if dragging { 0.5 } else { 1.0 };
    Color::srgba(1.0, 1.0, 1.0, alpha)
}
