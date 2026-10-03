//! Read-only projection of authoritative inventory/equipment data into item-mode slots.

use super::asset_contract::is_user_equip_semantic_icon_path;
use super::catalog::{
    USER_EQUIP_EQUIPMENT_STRIP_ORDER, UserEquipCatalogQuery, UserEquipCatalogQueryError,
    UserEquipEquipmentSlotKind, UserEquipEquipmentSlotSpec, UserEquipItemCatalog, UserEquipItemIds,
    UserEquipMissingIconReason, UserEquipProjectedIcon, UserEquipProjectionSource,
    UserEquipSlotEndpoint,
};
use super::geometry::{USER_EQUIP_EQUIPMENT_STRIP_COUNT, USER_EQUIP_INVENTORY_COLUMNS};
use super::view_model::UserEquipPresentationIcon;
use crate::inventory_runtime::{INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104};
use bevy::prelude::*;
use ffone_protocol::ItemBase0104;
use std::array;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEquipItemProjection {
    pub source: UserEquipProjectionSource,
    /// Lossless authoritative `sItemBase`; empty sentinels are not normalized.
    pub item: ItemBase0104,
    pub empty: bool,
    pub ids: UserEquipItemIds,
    pub show_combined_badge: bool,
    pub icon: UserEquipProjectedIcon,
}

impl UserEquipItemProjection {
    pub(super) fn from_authoritative(
        source: UserEquipProjectionSource,
        item: ItemBase0104,
        catalog: &impl UserEquipItemCatalog,
    ) -> Self {
        let empty = InventoryRuntime0104::item_is_empty(item);
        let ids = UserEquipItemIds::from_item(item);
        let show_combined_badge = !empty
            && ids.combined_look_id.is_some()
            && match source {
                UserEquipProjectionSource::Inventory { .. } => (0..4).contains(&item.item_type),
                UserEquipProjectionSource::Equipment { .. } => true,
            };
        let icon = if empty {
            UserEquipProjectedIcon::Empty
        } else {
            match UserEquipCatalogQuery::from_non_empty_item(item) {
                Err(UserEquipCatalogQueryError::MalformedIdentity { item_type, item_id }) => {
                    UserEquipProjectedIcon::MissingChecker(
                        UserEquipMissingIconReason::MalformedIdentity { item_type, item_id },
                    )
                }
                Err(UserEquipCatalogQueryError::EmptyItem { .. }) => {
                    unreachable!("non-empty projection was checked before catalog query")
                }
                Ok(query) if query.kind.legacy_returns_null() => {
                    UserEquipProjectedIcon::MissingChecker(
                        UserEquipMissingIconReason::QuestLegacyNull { query },
                    )
                }
                Ok(query) => match catalog.resolve_icon(query) {
                    Some(icon) => UserEquipProjectedIcon::Resolved(icon),
                    None => UserEquipProjectedIcon::MissingChecker(
                        UserEquipMissingIconReason::CatalogMiss { query },
                    ),
                },
            }
        };
        Self {
            source,
            item,
            empty,
            ids,
            show_combined_badge,
            icon,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEquipInventorySlotProjection {
    pub slot_index: usize,
    pub column: usize,
    pub row: usize,
    pub item: UserEquipItemProjection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEquipEquipmentSlotProjection {
    pub spec: UserEquipEquipmentSlotSpec,
    pub item: UserEquipItemProjection,
}

/// Clean `EquipPopup`/`UnequipPopup` `colorGreen` and `colorRed`.
pub const USER_EQUIP_RATING_BETTER_TINT: Color = Color::srgb(0.0, 1.0, 0.0);
pub const USER_EQUIP_RATING_WORSE_TINT: Color = Color::srgb(0.9, 0.2, 0.0);

/// Clean `EquipPopup.ValueColor`/`UnequipPopup.ValueColor` for the point,
/// group and defense ratings, applied as `GUI.color` tints over `base`.
///
/// An unequipped weapon or armor piece (hand, upper, lower, foot) compares
/// with the item in its own equipment slot, or with zero when that slot is
/// empty; other unequipped types keep `base`. An equipped item compares with
/// zero. Higher is green, lower is red and equal keeps `base`.
#[must_use]
pub fn user_equip_rating_colors(
    base: Color,
    item_type: i16,
    ratings: [i32; 3],
    same_slot_ratings: Option<[i32; 3]>,
    item_is_equipped: bool,
) -> [Color; 3] {
    if !item_is_equipped && !(0..=3).contains(&item_type) {
        return [base; 3];
    }
    let baseline = if item_is_equipped {
        [0; 3]
    } else {
        same_slot_ratings.unwrap_or([0; 3])
    };
    let base_rgba = base.to_srgba();
    std::array::from_fn(|index| {
        let tint = match ratings[index].cmp(&baseline[index]) {
            std::cmp::Ordering::Greater => USER_EQUIP_RATING_BETTER_TINT,
            std::cmp::Ordering::Less => USER_EQUIP_RATING_WORSE_TINT,
            std::cmp::Ordering::Equal => return base,
        }
        .to_srgba();
        Color::srgba(
            base_rgba.red * tint.red,
            base_rgba.green * tint.green,
            base_rgba.blue * tint.blue,
            base_rgba.alpha * tint.alpha,
        )
    })
}

/// [`user_equip_rating_colors`] for table items resolved through `content`.
#[must_use]
pub fn user_equip_item_rating_colors(
    content: &crate::tutorial_mission_content::TutorialMissionContent,
    base: Color,
    item: ItemBase0104,
    same_slot_item: Option<ItemBase0104>,
    item_is_equipped: bool,
) -> [Color; 3] {
    let ratings = |item: ItemBase0104| {
        content
            .gameplay_user_equip_item_detail(item.item_type, item.item_id)
            .map(|detail| {
                [
                    detail.point_rating,
                    detail.group_rating,
                    detail.defense_rating,
                ]
            })
    };
    user_equip_rating_colors(
        base,
        item.item_type,
        ratings(item).unwrap_or([0; 3]),
        same_slot_item.and_then(ratings),
        item_is_equipped,
    )
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct UserEquipItemModeProjection {
    pub owner_pc_id: i32,
    pub inventory: [UserEquipInventorySlotProjection; INVENTORY_SLOT_COUNT_0104],
    pub equipment: [UserEquipEquipmentSlotProjection; USER_EQUIP_EQUIPMENT_STRIP_COUNT],
}

impl Default for UserEquipItemModeProjection {
    fn default() -> Self {
        let empty_item = ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        };
        let inventory = array::from_fn(|slot_index| UserEquipInventorySlotProjection {
            slot_index,
            column: slot_index % USER_EQUIP_INVENTORY_COLUMNS,
            row: slot_index / USER_EQUIP_INVENTORY_COLUMNS,
            item: UserEquipItemProjection {
                source: UserEquipProjectionSource::Inventory { slot_index },
                item: empty_item,
                empty: true,
                ids: UserEquipItemIds::from_item(empty_item),
                show_combined_badge: false,
                icon: UserEquipProjectedIcon::Empty,
            },
        });
        let equipment = array::from_fn(|visual_index| {
            let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
            UserEquipEquipmentSlotProjection {
                spec,
                item: UserEquipItemProjection {
                    source: UserEquipProjectionSource::Equipment {
                        visual_index,
                        wire_slot_index: spec.wire_slot_index,
                    },
                    item: empty_item,
                    empty: true,
                    ids: UserEquipItemIds::from_item(empty_item),
                    show_combined_badge: false,
                    icon: UserEquipProjectedIcon::Empty,
                },
            }
        });
        Self {
            owner_pc_id: 0,
            inventory,
            equipment,
        }
    }
}

impl UserEquipItemModeProjection {
    #[must_use]
    pub fn from_authoritative(
        runtime: &InventoryRuntime0104,
        catalog: &impl UserEquipItemCatalog,
    ) -> Self {
        let inventory = std::array::from_fn(|slot_index| UserEquipInventorySlotProjection {
            slot_index,
            column: slot_index % USER_EQUIP_INVENTORY_COLUMNS,
            row: slot_index / USER_EQUIP_INVENTORY_COLUMNS,
            item: UserEquipItemProjection::from_authoritative(
                UserEquipProjectionSource::Inventory { slot_index },
                runtime.inventory()[slot_index],
                catalog,
            ),
        });
        let equipment = std::array::from_fn(|visual_index| {
            let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
            UserEquipEquipmentSlotProjection {
                spec,
                item: UserEquipItemProjection::from_authoritative(
                    UserEquipProjectionSource::Equipment {
                        visual_index,
                        wire_slot_index: spec.wire_slot_index,
                    },
                    runtime.equipment()[spec.wire_slot_index],
                    catalog,
                ),
            }
        });
        Self {
            owner_pc_id: runtime.owner_pc_id(),
            inventory,
            equipment,
        }
    }

    /// Replaces the complete UI projection from the current authoritative
    /// 9+50 snapshot. There is intentionally no slot-level speculative update.
    pub fn rebuild_from_authoritative(
        &mut self,
        runtime: &InventoryRuntime0104,
        catalog: &impl UserEquipItemCatalog,
    ) {
        *self = Self::from_authoritative(runtime, catalog);
    }

    /// Clears a disconnected/session-reset projection without retaining stale
    /// inventory behind the hidden UI.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Occupied equipment slot compared by clean `EquipPopup.ValueColor`
    /// (`PII.iSlotType == 0 && PII.SlotID == ItemType`).
    #[must_use]
    pub fn equipped_item(&self, wire_slot_index: usize) -> Option<ItemBase0104> {
        self.equipment
            .iter()
            .find(|slot| slot.spec.wire_slot_index == wire_slot_index && !slot.item.empty)
            .map(|slot| slot.item.item)
    }

    #[must_use]
    pub fn item_at(&self, endpoint: UserEquipSlotEndpoint) -> Option<&UserEquipItemProjection> {
        match endpoint {
            UserEquipSlotEndpoint::Inventory { slot_index } => {
                self.inventory.get(slot_index).map(|slot| &slot.item)
            }
            UserEquipSlotEndpoint::Equipment {
                visual_index,
                wire_slot_index,
            } => self
                .equipment
                .get(visual_index)
                .filter(|slot| slot.spec.wire_slot_index == wire_slot_index)
                .map(|slot| &slot.item),
        }
    }

    /// Clean `InventoryManagerScript.SendEquip(false)` ignores the hovered
    /// inventory cell and chooses the first empty authoritative inventory slot.
    #[must_use]
    pub fn first_empty_inventory_slot(&self) -> Option<usize> {
        self.inventory
            .iter()
            .find(|slot| slot.item.empty)
            .map(|slot| slot.slot_index)
    }

    #[must_use]
    pub fn endpoint_is_empty(&self, endpoint: UserEquipSlotEndpoint) -> bool {
        self.item_at(endpoint).is_some_and(|item| item.empty)
    }
}

pub(super) fn semantic_presentation_icon(path: Option<&str>) -> UserEquipPresentationIcon {
    match path.filter(|path| is_user_equip_semantic_icon_path(path)) {
        Some(path) => UserEquipPresentationIcon::Resolved(path.to_owned()),
        None => UserEquipPresentationIcon::MissingChecker,
    }
}

#[must_use]
pub fn user_equip_destination_accepts_item(
    item: ItemBase0104,
    destination: UserEquipSlotEndpoint,
) -> bool {
    match destination {
        UserEquipSlotEndpoint::Inventory { slot_index } => slot_index < INVENTORY_SLOT_COUNT_0104,
        UserEquipSlotEndpoint::Equipment {
            visual_index,
            wire_slot_index,
        } => USER_EQUIP_EQUIPMENT_STRIP_ORDER
            .get(visual_index)
            .filter(|spec| spec.wire_slot_index == wire_slot_index)
            .is_some_and(|spec| match spec.kind {
                UserEquipEquipmentSlotKind::SecondaryWeapon => item.item_type == 0,
                _ => spec.legacy_panel_item_type == item.item_type,
            }),
    }
}
