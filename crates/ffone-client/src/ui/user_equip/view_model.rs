//! Slot/gallery view models and localized item text helpers.

use super::asset_contract::is_user_equip_semantic_icon_path;
use super::catalog::{UserEquipEquipmentSlotSpec, UserEquipProjectedIcon};
use super::geometry::{
    USER_EQUIP_COMBINED_BADGE_LEFT, USER_EQUIP_COMBINED_BADGE_SIZE, USER_EQUIP_COMBINED_BADGE_TOP,
    USER_EQUIP_EQUIPMENT_STRIP_COUNT, USER_EQUIP_NANO_SLOT_SIZE, USER_EQUIP_NANO_SLOT_STRIDE,
    UserEquipUiRect,
};
use super::item_projection::{UserEquipItemModeProjection, UserEquipItemProjection};
use super::layout::UserEquipItemModeLayout;
use super::nano_projection::UserEquipNanoModeProjection;
use super::state::{UserEquipModalState, UserEquipUiState};
use crate::{inventory_runtime::INVENTORY_SLOT_COUNT_0104, localization::LocalizedText};
use bevy::prelude::*;
use ffone_protocol::ItemBase0104;
use std::array;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UserEquipSlotFrameVisual {
    #[default]
    Empty,
    Occupied,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum UserEquipPresentationIcon {
    #[default]
    Empty,
    Resolved(String),
    MissingChecker,
}

impl UserEquipPresentationIcon {
    #[must_use]
    pub fn from_projection(icon: &UserEquipProjectedIcon) -> Self {
        match icon {
            UserEquipProjectedIcon::Empty => Self::Empty,
            UserEquipProjectedIcon::Resolved(icon)
                if is_user_equip_semantic_icon_path(icon.runtime_path()) =>
            {
                Self::Resolved(icon.runtime_path().to_owned())
            }
            UserEquipProjectedIcon::Resolved(_) | UserEquipProjectedIcon::MissingChecker(_) => {
                Self::MissingChecker
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserEquipInventorySlotView {
    pub slot_index: usize,
    pub frame: UserEquipUiRect,
    pub frame_visual: UserEquipSlotFrameVisual,
    pub icon: UserEquipPresentationIcon,
    pub combined_badge: Option<UserEquipUiRect>,
    pub count_label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserEquipEquipmentSlotView {
    pub visual_index: usize,
    pub frame: UserEquipUiRect,
    pub label_rect: UserEquipUiRect,
    pub label: String,
    pub frame_visual: UserEquipSlotFrameVisual,
    pub icon: UserEquipPresentationIcon,
    pub combined_badge: Option<UserEquipUiRect>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserEquipNanoGallerySlotView {
    pub visual_index: usize,
    pub nano_id: i16,
    pub frame: UserEquipUiRect,
    pub owned: bool,
    pub equipped: bool,
    pub current_power: Option<i16>,
    pub icon: UserEquipPresentationIcon,
}

#[must_use]
pub fn user_equip_nano_gallery_view(
    layout: UserEquipItemModeLayout,
    projection: &UserEquipNanoModeProjection,
) -> Vec<UserEquipNanoGallerySlotView> {
    projection
        .gallery
        .iter()
        .enumerate()
        .map(|(visual_index, entry)| UserEquipNanoGallerySlotView {
            visual_index,
            nano_id: entry.nano_id,
            frame: UserEquipUiRect::new(
                layout.inventory_viewport.left + entry.column as f32 * USER_EQUIP_NANO_SLOT_STRIDE,
                layout.inventory_viewport.top + entry.row as f32 * USER_EQUIP_NANO_SLOT_STRIDE
                    - layout.scroll_y,
                USER_EQUIP_NANO_SLOT_SIZE,
                USER_EQUIP_NANO_SLOT_SIZE,
            ),
            owned: entry.owned,
            equipped: entry.equipped,
            current_power: entry.current_power,
            icon: entry.icon.clone(),
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserEquipItemModeView {
    pub layout: UserEquipItemModeLayout,
    pub controls_enabled: bool,
    pub inventory: [UserEquipInventorySlotView; INVENTORY_SLOT_COUNT_0104],
    pub equipment: [UserEquipEquipmentSlotView; USER_EQUIP_EQUIPMENT_STRIP_COUNT],
}

#[must_use]
pub fn user_equip_item_mode_view(
    viewport_width: u32,
    viewport_height: u32,
    state: UserEquipUiState,
    modal: UserEquipModalState,
    projection: &UserEquipItemModeProjection,
    static_assets_ready: bool,
) -> Option<UserEquipItemModeView> {
    let capabilities = state.input_capabilities(modal);
    if viewport_width == 0 || viewport_height == 0 || !static_assets_ready || !capabilities.draw {
        return None;
    }
    let layout = state.layout(viewport_width, viewport_height);
    let inventory = array::from_fn(|slot_index| {
        let projected = &projection.inventory[slot_index].item;
        let frame = layout
            .inventory_slot_rect(slot_index)
            .expect("authoritative inventory count and clean grid must agree");
        UserEquipInventorySlotView {
            slot_index,
            frame,
            frame_visual: if projected.empty {
                UserEquipSlotFrameVisual::Empty
            } else {
                UserEquipSlotFrameVisual::Occupied
            },
            icon: UserEquipPresentationIcon::from_projection(&projected.icon),
            combined_badge: projected.show_combined_badge.then(|| {
                UserEquipUiRect::new(
                    frame.left + USER_EQUIP_COMBINED_BADGE_LEFT,
                    frame.top + USER_EQUIP_COMBINED_BADGE_TOP,
                    USER_EQUIP_COMBINED_BADGE_SIZE,
                    USER_EQUIP_COMBINED_BADGE_SIZE,
                )
            }),
            count_label: inventory_item_overlay_text(projected),
        }
    });
    let equipment = array::from_fn(|visual_index| {
        let projected = &projection.equipment[visual_index];
        let frame = layout
            .equipment_slot_rect(visual_index)
            .expect("clean equipment strip count must agree");
        let label_rect = layout
            .equipment_label_rect(visual_index)
            .expect("clean equipment strip label count must agree");
        UserEquipEquipmentSlotView {
            visual_index,
            frame,
            label_rect,
            label: equipment_slot_label(projected.spec),
            frame_visual: if projected.item.empty {
                UserEquipSlotFrameVisual::Empty
            } else {
                UserEquipSlotFrameVisual::Occupied
            },
            icon: UserEquipPresentationIcon::from_projection(&projected.item.icon),
            combined_badge: projected.item.show_combined_badge.then(|| {
                UserEquipUiRect::new(
                    frame.left + USER_EQUIP_COMBINED_BADGE_LEFT,
                    frame.top + USER_EQUIP_COMBINED_BADGE_TOP,
                    USER_EQUIP_COMBINED_BADGE_SIZE,
                    USER_EQUIP_COMBINED_BADGE_SIZE,
                )
            }),
        }
    });
    Some(UserEquipItemModeView {
        layout,
        controls_enabled: capabilities.panel_controls,
        inventory,
        equipment,
    })
}

pub(super) fn inventory_item_overlay_text(item: &UserEquipItemProjection) -> Option<String> {
    if item.empty {
        None
    } else {
        match item.item.item_type {
            7 => Some(item.item.option.to_string()),
            8 => Some(format!("Quest {}", item.item.item_id)),
            _ => None,
        }
    }
}

pub(super) fn equipment_slot_label(spec: UserEquipEquipmentSlotSpec) -> String {
    match spec.label_ordinal {
        Some(ordinal) => format!("{} {ordinal}", spec.label_key),
        None => spec.label_key.to_owned(),
    }
}

/// Exact `Panel_Equip` GUI tint: empty labels use `(1,1,1,.8)` and occupied
/// labels use `(0.6,1,0,.8)`.
pub(super) fn equipment_slot_label_color(frame_visual: UserEquipSlotFrameVisual) -> Color {
    match frame_visual {
        UserEquipSlotFrameVisual::Empty => Color::srgba(1.0, 1.0, 1.0, 0.8),
        UserEquipSlotFrameVisual::Occupied => Color::srgba(0.6, 1.0, 0.0, 0.8),
    }
}

pub(crate) fn user_equip_display_text_id(item: ItemBase0104) -> i16 {
    let appearance = (item.option >> 16) as i16;
    if (0..=3).contains(&item.item_type) && appearance > 0 {
        appearance
    } else {
        item.item_id
    }
}

pub(crate) fn user_equip_item_type_localized(item_type: i16) -> LocalizedText {
    let (key, fallback) = match item_type {
        0 => ("ui.inventory.item_type.weapon", "Weapon"),
        1 => ("ui.inventory.item_type.body", "Body"),
        2 => ("ui.inventory.item_type.legs", "Legs"),
        3 => ("ui.inventory.item_type.shoes", "Shoes"),
        4 => ("ui.inventory.item_type.hat", "Hat"),
        5 => ("ui.inventory.item_type.glasses", "Glasses"),
        6 => ("ui.inventory.item_type.backpack", "Backpack"),
        7 => ("ui.inventory.item_type.general", "General"),
        8 => ("ui.inventory.item_type.quest", "Quest"),
        9 => ("ui.inventory.item_type.crate", "Crate"),
        10 => ("ui.inventory.item_type.vehicle", "Vehicle"),
        _ => ("ui.inventory.popup.not_available", "N/A"),
    };
    LocalizedText::new(key, fallback)
}

pub(crate) fn user_equip_weapon_type_localized(target_mode: Option<i32>) -> LocalizedText {
    let (key, fallback) = match target_mode {
        Some(1) => ("ui.inventory.weapon_type.melee", "Melee"),
        Some(2) => ("ui.inventory.weapon_type.pistol", "Pistol"),
        Some(3) => ("ui.inventory.weapon_type.shattergun", "Shattergun"),
        Some(4) => ("ui.inventory.weapon_type.rifle", "Rifle"),
        Some(5) => ("ui.inventory.weapon_type.rocket", "Rocket"),
        Some(6) => ("ui.inventory.weapon_type.thrown", "Thrown"),
        _ => ("ui.inventory.item_type.weapon", "Weapon"),
    };
    LocalizedText::new(key, fallback)
}

pub(crate) fn user_equip_range_localized(equip_type: Option<i32>) -> LocalizedText {
    let (key, fallback) = match equip_type {
        Some(0 | 1 | 6) => ("ui.inventory.range.short", "Short"),
        Some(3 | 4 | 5 | 8 | 9 | 10) => ("ui.inventory.range.medium", "Medium"),
        Some(2 | 7 | 11) => ("ui.inventory.range.long", "Long"),
        _ => ("ui.inventory.popup.not_available", "N/A"),
    };
    LocalizedText::new(key, fallback)
}

pub(crate) fn user_equip_rarity_localized(rarity: Option<i32>) -> LocalizedText {
    let (key, fallback) = match rarity {
        Some(1) => ("ui.inventory.rarity.common", "Common"),
        Some(2) => ("ui.inventory.rarity.uncommon", "Uncommon"),
        Some(3) => ("ui.inventory.rarity.rare", "Rare"),
        Some(4) => ("ui.inventory.rarity.ultra_rare", "Ultra rare"),
        Some(5) => ("ui.inventory.rarity.amazing", "Amazing!"),
        _ => ("ui.inventory.popup.not_available", "N/A"),
    };
    LocalizedText::new(key, fallback)
}
