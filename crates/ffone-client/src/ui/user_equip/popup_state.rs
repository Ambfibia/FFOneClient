//! Item popup commands, layout variants and drag/drop state.

use super::actions::UserEquipUiAction;
use super::catalog::UserEquipSlotEndpoint;
use super::item_projection::{UserEquipItemModeProjection, user_equip_destination_accepts_item};
use super::nano_projection::UserEquipNanoModeProjection;
use crate::{
    inventory_runtime::InventoryRuntime0104, tutorial_mission_content::TutorialMissionContent,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipPopupCommand {
    Equip,
    EquipPrimary,
    EquipSecondary,
    Unequip,
    Use,
    Open,
    Delete,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UserEquipPopupLayoutVariant {
    Equip,
    Unequip,
    GeneralStack,
    GeneralGum,
    GeneralUse,
    Chest,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserEquipItemPopupState {
    pub(super) selected: Option<UserEquipSlotEndpoint>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserEquipDragState {
    pub(super) source: Option<UserEquipSlotEndpoint>,
}

impl UserEquipDragState {
    #[must_use]
    pub const fn source(&self) -> Option<UserEquipSlotEndpoint> {
        self.source
    }

    pub fn begin(&mut self, source: UserEquipSlotEndpoint) {
        self.source = Some(source);
    }

    pub fn cancel(&mut self) {
        self.source = None;
    }

    #[must_use]
    pub fn finish(
        &mut self,
        destination: Option<UserEquipSlotEndpoint>,
        projection: &UserEquipItemModeProjection,
    ) -> Option<UserEquipUiAction> {
        let source = self.source.take()?;
        let destination = destination?;
        user_equip_drag_move(source, destination, projection)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserEquipNanoViewerState {
    pub(super) selected_visual_index: Option<usize>,
}

impl UserEquipNanoViewerState {
    #[must_use]
    pub const fn selected_visual_index(&self) -> Option<usize> {
        self.selected_visual_index
    }

    pub fn open(&mut self, visual_index: usize) {
        self.selected_visual_index = Some(visual_index);
    }

    pub fn close(&mut self) {
        self.selected_visual_index = None;
    }
}

#[must_use]
pub fn user_equip_drag_move(
    from: UserEquipSlotEndpoint,
    hovered_to: UserEquipSlotEndpoint,
    projection: &UserEquipItemModeProjection,
) -> Option<UserEquipUiAction> {
    if from == hovered_to {
        return None;
    }
    let item = projection.item_at(from)?.item;
    if InventoryRuntime0104::item_is_empty(item) {
        return None;
    }
    let to = match (from, hovered_to) {
        (UserEquipSlotEndpoint::Equipment { .. }, UserEquipSlotEndpoint::Inventory { .. }) => {
            UserEquipSlotEndpoint::Inventory {
                slot_index: projection.first_empty_inventory_slot()?,
            }
        }
        (_, destination) => destination,
    };
    if !user_equip_destination_accepts_item(item, to)
        || !user_equip_swap_destination_accepts_item(from, to, projection)
    {
        return None;
    }
    Some(UserEquipUiAction::MoveItem { from, to })
}

pub(super) fn user_equip_swap_destination_accepts_item(
    from: UserEquipSlotEndpoint,
    to: UserEquipSlotEndpoint,
    projection: &UserEquipItemModeProjection,
) -> bool {
    let UserEquipSlotEndpoint::Equipment { .. } = from else {
        return true;
    };
    let UserEquipSlotEndpoint::Equipment { .. } = to else {
        return true;
    };
    let Some(destination_item) = projection.item_at(to) else {
        return false;
    };
    destination_item.empty || user_equip_destination_accepts_item(destination_item.item, from)
}

/// Clean `Panel_PCStuffScript` registers its large trash control as a drop
/// target. The server delete ABI accepts inventory slots only, so equipment
/// and empty bag cells remain fail-closed here.
#[must_use]
pub fn user_equip_trash_drop(
    from: UserEquipSlotEndpoint,
    projection: &UserEquipItemModeProjection,
) -> Option<UserEquipUiAction> {
    let UserEquipSlotEndpoint::Inventory { slot_index } = from else {
        return None;
    };
    (!projection.endpoint_is_empty(from))
        .then_some(UserEquipUiAction::DeleteInventoryItem { slot_index })
}

impl UserEquipItemPopupState {
    #[must_use]
    pub const fn selected(&self) -> Option<UserEquipSlotEndpoint> {
        self.selected
    }

    pub fn open(&mut self, endpoint: UserEquipSlotEndpoint) {
        self.selected = Some(endpoint);
    }

    pub fn close(&mut self) {
        self.selected = None;
    }

    #[must_use]
    pub fn commands(
        &self,
        projection: &UserEquipItemModeProjection,
        content: Option<&TutorialMissionContent>,
    ) -> Vec<UserEquipPopupCommand> {
        let Some(endpoint) = self.selected else {
            return Vec::new();
        };
        let Some(projected) = projection.item_at(endpoint) else {
            return Vec::new();
        };
        if projected.empty {
            return Vec::new();
        }
        match endpoint {
            UserEquipSlotEndpoint::Equipment { .. } => vec![UserEquipPopupCommand::Unequip],
            UserEquipSlotEndpoint::Inventory { .. } => match projected.item.item_type {
                0 => vec![
                    UserEquipPopupCommand::EquipPrimary,
                    UserEquipPopupCommand::EquipSecondary,
                ],
                1..=6 | 10 => vec![UserEquipPopupCommand::Equip],
                7 => match user_equip_general_popup_variant(
                    content.and_then(|content| content.general_item_type(projected.item.item_id)),
                ) {
                    UserEquipPopupLayoutVariant::GeneralUse => {
                        vec![UserEquipPopupCommand::Use]
                    }
                    UserEquipPopupLayoutVariant::GeneralStack
                    | UserEquipPopupLayoutVariant::GeneralGum => {
                        vec![UserEquipPopupCommand::Delete]
                    }
                    _ => unreachable!("general items resolve only to General popup variants"),
                },
                9 => vec![UserEquipPopupCommand::Open],
                _ => Vec::new(),
            },
        }
    }
}

pub(super) fn user_equip_popup_layout_variant(
    popup: &UserEquipItemPopupState,
    projection: &UserEquipItemModeProjection,
    content: Option<&TutorialMissionContent>,
) -> Option<UserEquipPopupLayoutVariant> {
    let endpoint = popup.selected()?;
    let item = projection.item_at(endpoint)?;
    if item.empty {
        return None;
    }
    Some(match endpoint {
        UserEquipSlotEndpoint::Equipment { .. } => UserEquipPopupLayoutVariant::Unequip,
        UserEquipSlotEndpoint::Inventory { .. } => match item.item.item_type {
            7 => user_equip_general_popup_variant(
                content.and_then(|content| content.general_item_type(item.item.item_id)),
            ),
            9 => UserEquipPopupLayoutVariant::Chest,
            _ => UserEquipPopupLayoutVariant::Equip,
        },
    })
}

/// Primary `PopupControll.GeneralPopup` dispatch. The semantic GeneralItem
/// subtype, rather than protocol `sItemType == 7`, owns the actual dialog.
/// Missing metadata stays on the non-destructive Gum/stack presentation.
#[must_use]
pub(super) const fn user_equip_general_popup_variant(
    general_item_type: Option<i32>,
) -> UserEquipPopupLayoutVariant {
    match general_item_type {
        Some(3) => UserEquipPopupLayoutVariant::GeneralGum,
        Some(4 | 5 | 7 | 8 | 9 | 11) => UserEquipPopupLayoutVariant::GeneralUse,
        _ => UserEquipPopupLayoutVariant::GeneralStack,
    }
}

/// Primary `GumPopup.EnableGum`: attribute `4` feeds every Nano style;
/// otherwise the serialized attribute is the one-based Nano style.
#[must_use]
pub const fn user_equip_gum_attribute_accepts_style(
    stim_pack_attribute: Option<i32>,
    nano_id: Option<i16>,
    nano_style: Option<u8>,
) -> bool {
    let (Some(stim_pack_attribute), Some(_), Some(nano_style)) =
        (stim_pack_attribute, nano_id, nano_style)
    else {
        return false;
    };
    stim_pack_attribute == 4 || stim_pack_attribute == nano_style as i32 + 1
}

#[must_use]
pub fn user_equip_gum_target_enabled(
    item_id: i16,
    nano_slot: usize,
    nano_projection: &UserEquipNanoModeProjection,
    content: &TutorialMissionContent,
) -> bool {
    content.general_item_type(item_id) == Some(3)
        && nano_projection.status.get(nano_slot).is_some_and(|status| {
            user_equip_gum_attribute_accepts_style(
                content.general_item_stim_pack_attribute(item_id),
                status.nano_id,
                status.style,
            )
        })
}
