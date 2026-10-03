//! Equipment slot order, item catalog queries, icon references and slot endpoints.

use super::geometry::USER_EQUIP_EQUIPMENT_STRIP_COUNT;
use crate::{
    inventory_runtime::InventoryRuntime0104, tutorial_mission_content::TutorialMissionContent,
};
use bevy::prelude::*;
use ffone_protocol::ItemBase0104;
use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UserEquipEquipmentSlotKind {
    Head,
    Face,
    Back,
    UpperBody,
    LowerBody,
    Foot,
    PrimaryWeapon,
    SecondaryWeapon,
    Vehicle,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UserEquipEquipmentSlotSpec {
    pub kind: UserEquipEquipmentSlotKind,
    pub visual_index: usize,
    pub wire_slot_index: usize,
    /// The `sItemType` passed by clean `Panel_Equip.DoEquipPanelType`.
    ///
    /// The secondary weapon deliberately uses General (`7`) even though the
    /// authoritative item in wire slot 7 is a Hand/weapon (`0`).
    pub legacy_panel_item_type: i16,
    pub label_key: &'static str,
    /// Clean localizes `WEAPON` and then appends ` 1` / ` 2`.
    pub label_ordinal: Option<u8>,
}

pub const USER_EQUIP_EQUIPMENT_STRIP_ORDER: [UserEquipEquipmentSlotSpec;
    USER_EQUIP_EQUIPMENT_STRIP_COUNT] = [
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::Head,
        visual_index: 0,
        wire_slot_index: 4,
        legacy_panel_item_type: 4,
        label_key: "HEAD",
        label_ordinal: None,
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::Face,
        visual_index: 1,
        wire_slot_index: 5,
        legacy_panel_item_type: 5,
        label_key: "FACE",
        label_ordinal: None,
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::Back,
        visual_index: 2,
        wire_slot_index: 6,
        legacy_panel_item_type: 6,
        label_key: "BACK",
        label_ordinal: None,
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::UpperBody,
        visual_index: 3,
        wire_slot_index: 1,
        legacy_panel_item_type: 1,
        label_key: "CHEST",
        label_ordinal: None,
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::LowerBody,
        visual_index: 4,
        wire_slot_index: 2,
        legacy_panel_item_type: 2,
        label_key: "LEGS",
        label_ordinal: None,
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::Foot,
        visual_index: 5,
        wire_slot_index: 3,
        legacy_panel_item_type: 3,
        label_key: "FEET",
        label_ordinal: None,
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::PrimaryWeapon,
        visual_index: 6,
        wire_slot_index: 0,
        legacy_panel_item_type: 0,
        label_key: "WEAPON",
        label_ordinal: Some(1),
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::SecondaryWeapon,
        visual_index: 7,
        wire_slot_index: 7,
        legacy_panel_item_type: 7,
        label_key: "WEAPON",
        label_ordinal: Some(2),
    },
    UserEquipEquipmentSlotSpec {
        kind: UserEquipEquipmentSlotKind::Vehicle,
        visual_index: 8,
        wire_slot_index: 8,
        legacy_panel_item_type: 10,
        label_key: "VEHICLE",
        label_ordinal: None,
    },
];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UserEquipCatalogKind {
    Equipment { item_table: u8 },
    Nano,
    QuestLegacyNull,
    Chest,
    NanoTune,
    Npc,
    Skill,
    SkillBuff,
    General,
}

impl UserEquipCatalogKind {
    #[must_use]
    pub const fn from_item_type(item_type: i16) -> Self {
        match item_type {
            0 => Self::Equipment { item_table: 25 },
            1 => Self::Equipment { item_table: 23 },
            2 => Self::Equipment { item_table: 22 },
            3 => Self::Equipment { item_table: 24 },
            4 => Self::Equipment { item_table: 20 },
            5 => Self::Equipment { item_table: 19 },
            6 => Self::Equipment { item_table: 17 },
            10 => Self::Equipment { item_table: 26 },
            19 => Self::Nano,
            8 => Self::QuestLegacyNull,
            9 => Self::Chest,
            24 => Self::NanoTune,
            30 => Self::Npc,
            27 => Self::Skill,
            138 => Self::SkillBuff,
            _ => Self::General,
        }
    }

    #[must_use]
    pub const fn item_table(self) -> u8 {
        match self {
            Self::Equipment { item_table } => item_table,
            Self::Nano | Self::NanoTune => 9,
            Self::QuestLegacyNull => 29,
            Self::Chest => 28,
            Self::Npc => 10,
            Self::Skill | Self::SkillBuff => 12,
            Self::General => 27,
        }
    }

    #[must_use]
    pub const fn item_subtable(self) -> u8 {
        match self {
            Self::SkillBuff => 1,
            _ => 0,
        }
    }

    #[must_use]
    pub const fn icon_subtable(self) -> u8 {
        match self {
            Self::Nano | Self::Npc => 3,
            Self::NanoTune => 6,
            _ => 2,
        }
    }

    #[must_use]
    pub const fn legacy_returns_null(self) -> bool {
        matches!(self, Self::QuestLegacyNull)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UserEquipItemIds {
    pub base_item_id: i16,
    pub combined_look_id: Option<u16>,
    /// The dormant clean `korEnchant == None` branch would consume these bits.
    pub low_option_bits: u16,
}

impl UserEquipItemIds {
    #[must_use]
    pub const fn from_item(item: ItemBase0104) -> Self {
        let option = item.option as u32;
        let raw_look = (option >> 16) as u16;
        let kind = UserEquipCatalogKind::from_item_type(item.item_type);
        let combined_look_id =
            if matches!(kind, UserEquipCatalogKind::Equipment { .. }) && raw_look > 0 {
                Some(raw_look)
            } else {
                None
            };
        Self {
            base_item_id: item.item_id,
            combined_look_id,
            low_option_bits: option as u16,
        }
    }

    #[must_use]
    pub const fn icon_item_row_id(self) -> i32 {
        match self.combined_look_id {
            Some(look_id) => look_id as i32,
            None => self.base_item_id as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserEquipCatalogQuery {
    pub item_type: i16,
    pub base_item_id: i16,
    pub item_row_id: i32,
    pub combined_look_id: Option<u16>,
    pub kind: UserEquipCatalogKind,
}

impl UserEquipCatalogQuery {
    pub fn from_non_empty_item(item: ItemBase0104) -> Result<Self, UserEquipCatalogQueryError> {
        if InventoryRuntime0104::item_is_empty(item) {
            return Err(UserEquipCatalogQueryError::EmptyItem {
                item_type: item.item_type,
                item_id: item.item_id,
            });
        }
        if item.item_type < 0 {
            return Err(UserEquipCatalogQueryError::MalformedIdentity {
                item_type: item.item_type,
                item_id: item.item_id,
            });
        }
        let ids = UserEquipItemIds::from_item(item);
        Ok(Self {
            item_type: item.item_type,
            base_item_id: item.item_id,
            item_row_id: ids.icon_item_row_id(),
            combined_look_id: ids.combined_look_id,
            kind: UserEquipCatalogKind::from_item_type(item.item_type),
        })
    }

    pub(super) fn has_exact_avatar_util_identity(self) -> bool {
        if self.item_type < 0
            || self.base_item_id <= 0
            || self.kind != UserEquipCatalogKind::from_item_type(self.item_type)
        {
            return false;
        }
        match (self.kind, self.combined_look_id) {
            (UserEquipCatalogKind::Equipment { .. }, Some(look_id)) => {
                look_id > 0 && self.item_row_id == i32::from(look_id)
            }
            (UserEquipCatalogKind::Equipment { .. }, None) => {
                self.item_row_id == i32::from(self.base_item_id)
            }
            (_, None) => self.item_row_id == i32::from(self.base_item_id),
            (_, Some(_)) => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipCatalogQueryError {
    EmptyItem { item_type: i16, item_id: i16 },
    MalformedIdentity { item_type: i16, item_id: i16 },
}

impl fmt::Display for UserEquipCatalogQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyItem { item_type, item_id } => {
                write!(f, "item type={item_type} id={item_id} is empty")
            }
            Self::MalformedIdentity { item_type, item_id } => write!(
                f,
                "item type={item_type} id={item_id} cannot name a clean catalog row"
            ),
        }
    }
}

impl Error for UserEquipCatalogQueryError {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserEquipIconRef {
    pub(super) runtime_path: String,
}

impl UserEquipIconRef {
    pub fn new(runtime_path: impl Into<String>) -> Result<Self, UserEquipIconRefError> {
        let runtime_path = runtime_path.into();
        if runtime_path.trim().is_empty() {
            return Err(UserEquipIconRefError::EmptyRuntimePath);
        }
        Ok(Self { runtime_path })
    }

    #[must_use]
    pub fn runtime_path(&self) -> &str {
        &self.runtime_path
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipIconRefError {
    EmptyRuntimePath,
}

impl fmt::Display for UserEquipIconRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRuntimePath => f.write_str("UserEquip icon runtime path is empty"),
        }
    }
}

impl Error for UserEquipIconRefError {}

/// Typed boundary to a clean TableData-derived item/icon catalog.
///
/// Returning `None` is not repaired with a guessed semantic path. The
/// projection emits [`UserEquipProjectedIcon::MissingChecker`] instead.
pub trait UserEquipItemCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef>;
}

/// Production clean-TableData adapter. The queried item row and icon
/// subtable are selected solely by the exact `AvatarUtil` mapping represented
/// by [`UserEquipCatalogKind`]. Missing native files stay misses so the
/// presentation uses the procedural clean checker.
impl UserEquipItemCatalog for TutorialMissionContent {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        if !query.has_exact_avatar_util_identity() || query.kind.legacy_returns_null() {
            return None;
        }
        let definition = self.gameplay_user_equip_icon(
            query.kind.item_table(),
            query.kind.item_subtable(),
            query.item_row_id,
            query.kind.icon_subtable(),
        )?;
        UserEquipIconRef::new(definition.icon_path.as_deref()?).ok()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipMissingIconReason {
    MalformedIdentity { item_type: i16, item_id: i16 },
    QuestLegacyNull { query: UserEquipCatalogQuery },
    CatalogMiss { query: UserEquipCatalogQuery },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserEquipProjectedIcon {
    Empty,
    Resolved(UserEquipIconRef),
    MissingChecker(UserEquipMissingIconReason),
}

/// Exact inventory protocol endpoint selected by a clean Item-mode control.
/// Inventory location is `1`; equipped location is `0`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UserEquipSlotEndpoint {
    Inventory {
        slot_index: usize,
    },
    Equipment {
        visual_index: usize,
        wire_slot_index: usize,
    },
}

impl UserEquipSlotEndpoint {
    #[must_use]
    pub const fn legacy_location(self) -> i32 {
        match self {
            Self::Inventory { .. } => 1,
            Self::Equipment { .. } => 0,
        }
    }

    #[must_use]
    pub const fn wire_slot_index(self) -> usize {
        match self {
            Self::Inventory { slot_index }
            | Self::Equipment {
                wire_slot_index: slot_index,
                ..
            } => slot_index,
        }
    }

    #[must_use]
    pub const fn is_inventory(self) -> bool {
        matches!(self, Self::Inventory { .. })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipProjectionSource {
    Inventory {
        slot_index: usize,
    },
    Equipment {
        visual_index: usize,
        wire_slot_index: usize,
    },
}

impl UserEquipProjectionSource {
    #[must_use]
    pub const fn endpoint(self) -> UserEquipSlotEndpoint {
        match self {
            Self::Inventory { slot_index } => UserEquipSlotEndpoint::Inventory { slot_index },
            Self::Equipment {
                visual_index,
                wire_slot_index,
            } => UserEquipSlotEndpoint::Equipment {
                visual_index,
                wire_slot_index,
            },
        }
    }
}
