//! Pure protocol-0104 inventory/equipment authority.
//!
//! Primary clean-client evidence:
//! - `InventoryManagerScript.GetEquipSlot` reads exactly 9 equipment slots.
//! - `InventoryManagerScript.GetInventorySlot` reads exactly 50 general
//!   inventory slots.
//! - `OCSlotEntity.SetItemBase` treats `sItemBase.iID <= 0` as empty without
//!   rewriting the other serialized fields.
//! - `UserSlot.ItemMove` applies `sP_FE2CL_PC_ITEM_MOVE_SUCC.FromSlotItem` and
//!   `ToSlotItem` as authoritative post-mutation slot values.
//!
//! This module intentionally owns no Bevy state and performs no speculative
//! stack, equip, or item-use simulation. Every mutation is validated in full
//! before any slot is written.

use ffone_protocol::{
    EquipChangePacket0104, ItemBase0104, ItemMoveSuccessPacket0104, ItemReward0104,
    ItemUseDecodeError0104, ItemUsePacket0104, ItemUseSuccessPacket0104, NanoTuneSuccess0104,
    PcDisassembleItemSuccess0104, PcItemDeleteSuccess0104, PcLoadData0104,
    VendorItemBuySuccess0104, VendorItemRestoreBuySuccess0104, VendorItemSellSuccess0104,
    WirePayload,
};
use std::fmt;

pub const EQUIPMENT_SLOT_COUNT_0104: usize = PcLoadData0104::EQUIPMENT_COUNT;
pub const INVENTORY_SLOT_COUNT_0104: usize = PcLoadData0104::INVENTORY_COUNT;

/// Exact protocol-0104 `eItemLocation` values represented by this runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum InventoryLocation0104 {
    Equipment = 0,
    Inventory = 1,
}

impl InventoryLocation0104 {
    #[must_use]
    pub const fn wire_value(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn capacity(self) -> usize {
        match self {
            Self::Equipment => EQUIPMENT_SLOT_COUNT_0104,
            Self::Inventory => INVENTORY_SLOT_COUNT_0104,
        }
    }

    fn from_wire(
        raw: i32,
        provenance: InventoryMutationProvenance0104,
    ) -> Result<Self, InventoryMutationError0104> {
        match raw {
            0 => Ok(Self::Equipment),
            1 => Ok(Self::Inventory),
            _ => Err(InventoryMutationError0104::UnsupportedLocation {
                provenance,
                raw_location: raw,
            }),
        }
    }
}

/// A bounds-checked slot reference.
///
/// Fields stay private so a value returned by this module always addresses a
/// real slot in the 9+50 protocol-0104 snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InventorySlotRef0104 {
    location: InventoryLocation0104,
    index: usize,
}

impl InventorySlotRef0104 {
    #[must_use]
    pub const fn location(self) -> InventoryLocation0104 {
        self.location
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn wire_index(self) -> i32 {
        self.index as i32
    }
}

/// Why the current value of a slot is authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventorySlotAuthority0104 {
    PcLoadData2Cl,
    PcItemMoveSuccess,
    PcEquipChange,
    PcItemUseSuccess,
    VendorItemBuySuccess,
    VendorItemSellSuccess,
    VendorItemRestoreBuySuccess,
    NanoTuneSuccess,
    EmailSendSuccess,
    PcItemDeleteSuccess,
    PcDisassembleItemSuccess,
    RewardItemReply,
}

/// Packet/source contract that requested a runtime mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryMutationProvenance0104 {
    PcItemMoveSuccess,
    PcEquipChange,
    PcItemUseSuccess,
    VendorItemBuySuccess,
    VendorItemSellSuccess,
    VendorItemRestoreBuySuccess,
    NanoTuneSuccess,
    EmailSendSuccess,
    PcItemDeleteSuccess,
    PcDisassembleItemSuccess,
    RewardItemReply,
}

/// The type and table ID available to clean QuickSlot registration/use.
///
/// `option` and `time_limit` are deliberately absent: QuickSlot's exact wire
/// contract carries only `iItemType` and `iItemID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemIdentity0104 {
    item_type: i16,
    item_id: i16,
}

impl ItemIdentity0104 {
    pub fn new(item_type: i16, item_id: i16) -> Result<Self, InventoryLookupError0104> {
        if item_type < 0 || item_id <= 0 {
            return Err(InventoryLookupError0104::MalformedIdentity { item_type, item_id });
        }
        Ok(Self { item_type, item_id })
    }

    #[must_use]
    pub const fn item_type(self) -> i16 {
        self.item_type
    }

    #[must_use]
    pub const fn item_id(self) -> i16 {
        self.item_id
    }
}

/// Identity-based lookup failures are deterministic and never select an
/// arbitrary duplicate stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryLookupError0104 {
    MalformedIdentity {
        item_type: i16,
        item_id: i16,
    },
    NotFound {
        identity: ItemIdentity0104,
    },
    AmbiguousIdentity {
        identity: ItemIdentity0104,
        first: InventorySlotRef0104,
        second: InventorySlotRef0104,
    },
}

impl fmt::Display for InventoryLookupError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedIdentity { item_type, item_id } => write!(
                f,
                "malformed protocol-0104 item identity type={item_type} id={item_id}"
            ),
            Self::NotFound { identity } => write!(
                f,
                "inventory item type={} id={} was not found",
                identity.item_type, identity.item_id
            ),
            Self::AmbiguousIdentity {
                identity,
                first,
                second,
            } => write!(
                f,
                "inventory item type={} id={} is ambiguous at slots {} and {}",
                identity.item_type,
                identity.item_id,
                first.index(),
                second.index()
            ),
        }
    }
}

impl std::error::Error for InventoryLookupError0104 {}

/// Predicate-based lookup supports TableData-owned semantics such as
/// resurrection-item subtype resolution without coupling this pure runtime to
/// TableData. Multiple matches fail closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventorySelectionError0104 {
    NotFound,
    Ambiguous {
        first: InventorySlotRef0104,
        second: InventorySlotRef0104,
    },
}

impl fmt::Display for InventorySelectionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("no non-empty inventory slot matched"),
            Self::Ambiguous { first, second } => write!(
                f,
                "inventory selection is ambiguous at slots {} and {}",
                first.index(),
                second.index()
            ),
        }
    }
}

impl std::error::Error for InventorySelectionError0104 {}

/// Successful mutation receipt. `secondary` is present only when a move
/// authoritatively writes two different slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryMutationReceipt0104 {
    pub provenance: InventoryMutationProvenance0104,
    pub primary: InventorySlotRef0104,
    pub secondary: Option<InventorySlotRef0104>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryMutationError0104 {
    ExactPayloadSize {
        provenance: InventoryMutationProvenance0104,
        expected: usize,
        actual: usize,
    },
    MinimumPayloadSize {
        provenance: InventoryMutationProvenance0104,
        minimum: usize,
        actual: usize,
    },
    UnsupportedLocation {
        provenance: InventoryMutationProvenance0104,
        raw_location: i32,
    },
    NegativeSlot {
        provenance: InventoryMutationProvenance0104,
        location: InventoryLocation0104,
        raw_slot: i32,
    },
    SlotOutOfBounds {
        provenance: InventoryMutationProvenance0104,
        location: InventoryLocation0104,
        raw_slot: i32,
        capacity: usize,
    },
    OwnerMismatch {
        provenance: InventoryMutationProvenance0104,
        expected_pc_id: i32,
        packet_pc_id: i32,
    },
    MalformedItemIdentity {
        provenance: InventoryMutationProvenance0104,
        slot: InventorySlotRef0104,
        item_type: i16,
        item_id: i16,
    },
    ConflictingSameSlotValues {
        provenance: InventoryMutationProvenance0104,
        slot: InventorySlotRef0104,
        first: ItemBase0104,
        second: ItemBase0104,
    },
    ItemUseDecode {
        error: ItemUseDecodeError0104,
    },
}

impl InventoryMutationError0104 {
    #[must_use]
    pub const fn provenance(&self) -> InventoryMutationProvenance0104 {
        match self {
            Self::ExactPayloadSize { provenance, .. }
            | Self::MinimumPayloadSize { provenance, .. }
            | Self::UnsupportedLocation { provenance, .. }
            | Self::NegativeSlot { provenance, .. }
            | Self::SlotOutOfBounds { provenance, .. }
            | Self::OwnerMismatch { provenance, .. }
            | Self::MalformedItemIdentity { provenance, .. }
            | Self::ConflictingSameSlotValues { provenance, .. } => *provenance,
            Self::ItemUseDecode { .. } => InventoryMutationProvenance0104::PcItemUseSuccess,
        }
    }
}

impl fmt::Display for InventoryMutationError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExactPayloadSize {
                provenance,
                expected,
                actual,
            } => write!(
                f,
                "{provenance:?} requires exactly {expected} bytes, received {actual}"
            ),
            Self::MinimumPayloadSize {
                provenance,
                minimum,
                actual,
            } => write!(
                f,
                "{provenance:?} requires at least {minimum} bytes, received {actual}"
            ),
            Self::UnsupportedLocation {
                provenance,
                raw_location,
            } => write!(
                f,
                "{provenance:?} references unsupported item location {raw_location}"
            ),
            Self::NegativeSlot {
                provenance,
                location,
                raw_slot,
            } => write!(
                f,
                "{provenance:?} references negative {location:?} slot {raw_slot}"
            ),
            Self::SlotOutOfBounds {
                provenance,
                location,
                raw_slot,
                capacity,
            } => write!(
                f,
                "{provenance:?} references {location:?} slot {raw_slot}, capacity is {capacity}"
            ),
            Self::OwnerMismatch {
                provenance,
                expected_pc_id,
                packet_pc_id,
            } => write!(
                f,
                "{provenance:?} belongs to PC {packet_pc_id}, expected {expected_pc_id}"
            ),
            Self::MalformedItemIdentity {
                provenance,
                slot,
                item_type,
                item_id,
            } => write!(
                f,
                "{provenance:?} carries malformed item type={item_type} id={item_id} for {:?} slot {}",
                slot.location(),
                slot.index()
            ),
            Self::ConflictingSameSlotValues {
                provenance,
                slot,
                first,
                second,
            } => write!(
                f,
                "{provenance:?} carries two values for {:?} slot {}: {first:?} and {second:?}",
                slot.location(),
                slot.index()
            ),
            Self::ItemUseDecode { error } => {
                write!(f, "PcItemUseSuccess is malformed or unproven: {error}")
            }
        }
    }
}

impl std::error::Error for InventoryMutationError0104 {}

/// Exact 9+50 local player item snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryRuntime0104 {
    owner_pc_id: i32,
    equipment: [ItemBase0104; EQUIPMENT_SLOT_COUNT_0104],
    inventory: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
    equipment_authority: [InventorySlotAuthority0104; EQUIPMENT_SLOT_COUNT_0104],
    inventory_authority: [InventorySlotAuthority0104; INVENTORY_SLOT_COUNT_0104],
}

impl InventoryRuntime0104 {
    /// Loads the two arrays losslessly through the protocol crate's exact
    /// `sPCLoadData2CL` accessors.
    #[must_use]
    pub fn from_pc_load(owner_pc_id: i32, load: &PcLoadData0104) -> Self {
        Self {
            owner_pc_id,
            equipment: load.equipment(),
            inventory: load.inventory(),
            equipment_authority: [InventorySlotAuthority0104::PcLoadData2Cl;
                EQUIPMENT_SLOT_COUNT_0104],
            inventory_authority: [InventorySlotAuthority0104::PcLoadData2Cl;
                INVENTORY_SLOT_COUNT_0104],
        }
    }

    #[must_use]
    pub const fn owner_pc_id(&self) -> i32 {
        self.owner_pc_id
    }

    #[must_use]
    pub const fn equipment(&self) -> &[ItemBase0104; EQUIPMENT_SLOT_COUNT_0104] {
        &self.equipment
    }

    #[must_use]
    pub const fn inventory(&self) -> &[ItemBase0104; INVENTORY_SLOT_COUNT_0104] {
        &self.inventory
    }

    #[must_use]
    pub fn item(&self, slot: InventorySlotRef0104) -> ItemBase0104 {
        match slot.location {
            InventoryLocation0104::Equipment => self.equipment[slot.index],
            InventoryLocation0104::Inventory => self.inventory[slot.index],
        }
    }

    #[must_use]
    pub fn authority(&self, slot: InventorySlotRef0104) -> InventorySlotAuthority0104 {
        match slot.location {
            InventoryLocation0104::Equipment => self.equipment_authority[slot.index],
            InventoryLocation0104::Inventory => self.inventory_authority[slot.index],
        }
    }

    /// Clean `OCSlotEntity.SetItemBase` emptiness rule. No other serialized
    /// field is canonicalized or inferred.
    #[must_use]
    pub const fn item_is_empty(item: ItemBase0104) -> bool {
        item.item_id <= 0
    }

    /// Resolves the single general-inventory slot carrying the exact
    /// QuickSlot identity. Duplicate identities are never silently resolved.
    pub fn find_unique_inventory_slot(
        &self,
        identity: ItemIdentity0104,
    ) -> Result<InventorySlotRef0104, InventoryLookupError0104> {
        let mut found = None;
        for (index, item) in self.inventory.iter().copied().enumerate() {
            if Self::item_is_empty(item)
                || item.item_type != identity.item_type
                || item.item_id != identity.item_id
            {
                continue;
            }
            let slot = InventorySlotRef0104 {
                location: InventoryLocation0104::Inventory,
                index,
            };
            if let Some(first) = found {
                return Err(InventoryLookupError0104::AmbiguousIdentity {
                    identity,
                    first,
                    second: slot,
                });
            }
            found = Some(slot);
        }
        found.ok_or(InventoryLookupError0104::NotFound { identity })
    }

    /// TableData-aware callers can resolve a semantic item class while this
    /// module remains a pure protocol runtime. Empty sentinels are never
    /// passed to `matches`.
    pub fn find_unique_inventory_slot_matching(
        &self,
        mut matches: impl FnMut(ItemBase0104) -> bool,
    ) -> Result<InventorySlotRef0104, InventorySelectionError0104> {
        let mut found = None;
        for (index, item) in self.inventory.iter().copied().enumerate() {
            if Self::item_is_empty(item) || !matches(item) {
                continue;
            }
            let slot = InventorySlotRef0104 {
                location: InventoryLocation0104::Inventory,
                index,
            };
            if let Some(first) = found {
                return Err(InventorySelectionError0104::Ambiguous {
                    first,
                    second: slot,
                });
            }
            found = Some(slot);
        }
        found.ok_or(InventorySelectionError0104::NotFound)
    }

    pub fn apply_item_move_success_payload(
        &mut self,
        payload: &[u8],
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcItemMoveSuccess;
        require_exact_payload(payload, ItemMoveSuccessPacket0104::SIZE, provenance)?;
        self.apply_item_move_success(
            ItemMoveSuccessPacket0104::decode(payload)
                .expect("protocol item-move payload size was checked"),
        )
    }

    /// Applies one authoritative `sItemReward` post-state. Quest-inventory
    /// location 2 is owned by the caller's separate QInven snapshot.
    pub fn apply_reward_item_post_state(
        &mut self,
        reward: ItemReward0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::RewardItemReply;
        let slot = resolve_slot(reward.inventory_location, reward.slot, provenance)?;
        validate_authoritative_item(reward.item, slot, provenance)?;
        self.write_slot(
            slot,
            reward.item,
            InventorySlotAuthority0104::RewardItemReply,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    /// Applies both post-state items atomically. In particular, this does not
    /// simulate a swap from the pre-packet snapshot.
    pub fn apply_item_move_success(
        &mut self,
        packet: ItemMoveSuccessPacket0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcItemMoveSuccess;
        let from = resolve_slot(packet.from_location, packet.from_slot_num, provenance)?;
        let to = resolve_slot(packet.to_location, packet.to_slot_num, provenance)?;
        validate_authoritative_item(packet.from_slot_item, from, provenance)?;
        validate_authoritative_item(packet.to_slot_item, to, provenance)?;
        if from == to && packet.from_slot_item != packet.to_slot_item {
            return Err(InventoryMutationError0104::ConflictingSameSlotValues {
                provenance,
                slot: from,
                first: packet.from_slot_item,
                second: packet.to_slot_item,
            });
        }

        self.write_slot(
            from,
            packet.from_slot_item,
            InventorySlotAuthority0104::PcItemMoveSuccess,
        );
        if to != from {
            self.write_slot(
                to,
                packet.to_slot_item,
                InventorySlotAuthority0104::PcItemMoveSuccess,
            );
        }
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: from,
            secondary: (to != from).then_some(to),
        })
    }

    pub fn apply_equip_change_payload(
        &mut self,
        payload: &[u8],
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcEquipChange;
        require_exact_payload(payload, EquipChangePacket0104::SIZE, provenance)?;
        self.apply_equip_change(
            EquipChangePacket0104::decode(payload)
                .expect("protocol equip-change payload size was checked"),
        )
    }

    /// Applies only a change belonging to this runtime's local PC. Remote
    /// equipment broadcasts belong to the remote-entity lifecycle instead.
    pub fn apply_equip_change(
        &mut self,
        packet: EquipChangePacket0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcEquipChange;
        if packet.pc_id != self.owner_pc_id {
            return Err(InventoryMutationError0104::OwnerMismatch {
                provenance,
                expected_pc_id: self.owner_pc_id,
                packet_pc_id: packet.pc_id,
            });
        }
        let slot = resolve_slot(
            InventoryLocation0104::Equipment.wire_value(),
            packet.equip_slot_num,
            provenance,
        )?;
        validate_authoritative_item(packet.equip_slot_item, slot, provenance)?;
        self.write_slot(
            slot,
            packet.equip_slot_item,
            InventorySlotAuthority0104::PcEquipChange,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    /// Validates the complete `eST`-dependent result tail before applying the
    /// authoritative `RemainItem`.
    ///
    /// Clean-client result layouts are accepted for every non-negative target
    /// count they prove. Positive-count skill types with no proven tail ABI,
    /// truncated/extended tails, and negative counts all fail before any slot
    /// is written.
    pub fn apply_item_use_success_payload(
        &mut self,
        payload: &[u8],
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let packet = ItemUseSuccessPacket0104::decode(payload)
            .map_err(|error| InventoryMutationError0104::ItemUseDecode { error })?;
        self.apply_item_use_success(&packet)
    }

    /// Applies only the server reply that owns local inventory post-state.
    /// Failure packets and `P_FE2CL_PC_ITEM_USE` broadcasts never own a local
    /// inventory slot and therefore return `Ok(None)` without mutation.
    pub fn apply_item_use_packet(
        &mut self,
        packet: &ItemUsePacket0104,
    ) -> Result<Option<InventoryMutationReceipt0104>, InventoryMutationError0104> {
        match packet {
            ItemUsePacket0104::Success(packet) => self.apply_item_use_success(packet).map(Some),
            ItemUsePacket0104::Failure(_) | ItemUsePacket0104::Broadcast(_) => Ok(None),
        }
    }

    pub fn apply_item_use_success(
        &mut self,
        packet: &ItemUseSuccessPacket0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcItemUseSuccess;
        let packet = packet.prefix();
        if packet.pc_id != self.owner_pc_id {
            return Err(InventoryMutationError0104::OwnerMismatch {
                provenance,
                expected_pc_id: self.owner_pc_id,
                packet_pc_id: packet.pc_id,
            });
        }
        let slot = resolve_slot(packet.item_location, packet.slot_num, provenance)?;
        validate_authoritative_item(packet.remaining_item, slot, provenance)?;
        self.write_slot(
            slot,
            packet.remaining_item,
            InventorySlotAuthority0104::PcItemUseSuccess,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    /// Applies only the inventory item owned by a correlated vendor-buy
    /// success. Currency is intentionally owned by the production runtime.
    pub fn apply_vendor_buy_success(
        &mut self,
        packet: VendorItemBuySuccess0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::VendorItemBuySuccess;
        let slot = resolve_slot(
            InventoryLocation0104::Inventory.wire_value(),
            packet.inventory_slot,
            provenance,
        )?;
        validate_authoritative_item(packet.item, slot, provenance)?;
        self.write_slot(
            slot,
            packet.item,
            InventorySlotAuthority0104::VendorItemBuySuccess,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    /// Mirrors clean `cnVendor.ReceiveItemSellSuccess`: general items retain
    /// the authoritative `ItemStay`; every other type preserves the returned
    /// record and clears only its item ID.
    pub fn apply_vendor_sell_success(
        &mut self,
        packet: VendorItemSellSuccess0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::VendorItemSellSuccess;
        let slot = resolve_slot(
            InventoryLocation0104::Inventory.wire_value(),
            packet.inventory_slot,
            provenance,
        )?;
        let post_item = if packet.item.item_type == 7 {
            packet.item_stay
        } else {
            ItemBase0104 {
                item_id: 0,
                ..packet.item
            }
        };
        validate_authoritative_item(post_item, slot, provenance)?;
        self.write_slot(
            slot,
            post_item,
            InventorySlotAuthority0104::VendorItemSellSuccess,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    /// Clean restore writes the server-owned identity/option but explicitly
    /// forces the local time limit to zero.
    pub fn apply_vendor_restore_success(
        &mut self,
        packet: VendorItemRestoreBuySuccess0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::VendorItemRestoreBuySuccess;
        let slot = resolve_slot(
            InventoryLocation0104::Inventory.wire_value(),
            packet.inventory_slot,
            provenance,
        )?;
        let post_item = ItemBase0104 {
            time_limit: 0,
            ..packet.item
        };
        validate_authoritative_item(post_item, slot, provenance)?;
        self.write_slot(
            slot,
            post_item,
            InventorySlotAuthority0104::VendorItemRestoreBuySuccess,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    /// Mirrors `cnOwnAvatarStatus.ReceiveNanoTune`: every reply entry whose
    /// `aiItemSlotNum` is not `-1` replaces that general-inventory slot with
    /// the paired authoritative `aItem` value.
    ///
    /// All ten entries are validated before the first write. Repeated slot
    /// numbers are accepted only when their paired post-state is identical.
    pub fn apply_nano_tune_success(
        &mut self,
        packet: NanoTuneSuccess0104,
    ) -> Result<Vec<InventorySlotRef0104>, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::NanoTuneSuccess;
        let mut writes: Vec<(InventorySlotRef0104, ItemBase0104)> = Vec::new();
        for (raw_slot, item) in packet.item_slots.into_iter().zip(packet.items) {
            if raw_slot == -1 {
                continue;
            }
            let slot = resolve_slot(
                InventoryLocation0104::Inventory.wire_value(),
                raw_slot,
                provenance,
            )?;
            validate_authoritative_item(item, slot, provenance)?;
            if let Some((_, first)) = writes.iter().find(|(known, _)| *known == slot) {
                if *first != item {
                    return Err(InventoryMutationError0104::ConflictingSameSlotValues {
                        provenance,
                        slot,
                        first: *first,
                        second: item,
                    });
                }
                continue;
            }
            writes.push((slot, item));
        }

        for (slot, item) in writes.iter().copied() {
            self.write_slot(slot, item, InventorySlotAuthority0104::NanoTuneSuccess);
        }
        Ok(writes.into_iter().map(|(slot, _)| slot).collect())
    }

    /// Applies the post-send inventory records carried by
    /// `sP_FE2CL_REP_PC_SEND_EMAIL_SUCC`. Clean EmailMode forwards only the
    /// non-general attachment records after setting their item ID to zero;
    /// callers preserve that quirk and pass the resulting slot/value pairs.
    /// The complete batch is validated before the first authoritative write.
    pub fn apply_email_send_success(
        &mut self,
        writes: &[(i32, ItemBase0104)],
    ) -> Result<Vec<InventorySlotRef0104>, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::EmailSendSuccess;
        let mut staged: Vec<(InventorySlotRef0104, ItemBase0104)> =
            Vec::with_capacity(writes.len());
        for &(wire_index, item) in writes {
            let slot = resolve_slot(
                InventoryLocation0104::Inventory.wire_value(),
                wire_index,
                provenance,
            )?;
            validate_authoritative_item(item, slot, provenance)?;
            if let Some((_, first)) = staged.iter().find(|(known, _)| *known == slot) {
                if *first != item {
                    return Err(InventoryMutationError0104::ConflictingSameSlotValues {
                        provenance,
                        slot,
                        first: *first,
                        second: item,
                    });
                }
                continue;
            }
            staged.push((slot, item));
        }
        for (slot, item) in staged.iter().copied() {
            self.write_slot(slot, item, InventorySlotAuthority0104::EmailSendSuccess);
        }
        Ok(staged.into_iter().map(|(slot, _)| slot).collect())
    }

    /// The 0104 delete reply owns only location and slot. OpenFusion clears
    /// type/ID/option while leaving the serialized time-limit word untouched.
    pub fn apply_item_delete_success(
        &mut self,
        packet: PcItemDeleteSuccess0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcItemDeleteSuccess;
        let slot = resolve_slot(packet.item_location, packet.slot_num, provenance)?;
        let current = self.item(slot);
        let post_item = ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: current.time_limit,
        };
        self.write_slot(
            slot,
            post_item,
            InventorySlotAuthority0104::PcItemDeleteSuccess,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    pub fn apply_disassemble_success(
        &mut self,
        packet: PcDisassembleItemSuccess0104,
    ) -> Result<InventoryMutationReceipt0104, InventoryMutationError0104> {
        let provenance = InventoryMutationProvenance0104::PcDisassembleItemSuccess;
        let slot = resolve_slot(
            InventoryLocation0104::Inventory.wire_value(),
            packet.new_item_slot,
            provenance,
        )?;
        validate_authoritative_item(packet.new_item, slot, provenance)?;
        self.write_slot(
            slot,
            packet.new_item,
            InventorySlotAuthority0104::PcDisassembleItemSuccess,
        );
        Ok(InventoryMutationReceipt0104 {
            provenance,
            primary: slot,
            secondary: None,
        })
    }

    fn write_slot(
        &mut self,
        slot: InventorySlotRef0104,
        item: ItemBase0104,
        authority: InventorySlotAuthority0104,
    ) {
        match slot.location {
            InventoryLocation0104::Equipment => {
                self.equipment[slot.index] = item;
                self.equipment_authority[slot.index] = authority;
            }
            InventoryLocation0104::Inventory => {
                self.inventory[slot.index] = item;
                self.inventory_authority[slot.index] = authority;
            }
        }
    }
}

fn resolve_slot(
    raw_location: i32,
    raw_slot: i32,
    provenance: InventoryMutationProvenance0104,
) -> Result<InventorySlotRef0104, InventoryMutationError0104> {
    let location = InventoryLocation0104::from_wire(raw_location, provenance)?;
    if raw_slot < 0 {
        return Err(InventoryMutationError0104::NegativeSlot {
            provenance,
            location,
            raw_slot,
        });
    }
    let index = raw_slot as usize;
    if index >= location.capacity() {
        return Err(InventoryMutationError0104::SlotOutOfBounds {
            provenance,
            location,
            raw_slot,
            capacity: location.capacity(),
        });
    }
    Ok(InventorySlotRef0104 { location, index })
}

fn validate_authoritative_item(
    item: ItemBase0104,
    slot: InventorySlotRef0104,
    provenance: InventoryMutationProvenance0104,
) -> Result<(), InventoryMutationError0104> {
    // The clean empty sentinel is governed by ID only. A non-empty item with
    // a negative type cannot name any clean sItemType and is malformed.
    if item.item_id > 0 && item.item_type < 0 {
        return Err(InventoryMutationError0104::MalformedItemIdentity {
            provenance,
            slot,
            item_type: item.item_type,
            item_id: item.item_id,
        });
    }
    Ok(())
}

fn require_exact_payload(
    payload: &[u8],
    expected: usize,
    provenance: InventoryMutationProvenance0104,
) -> Result<(), InventoryMutationError0104> {
    if payload.len() != expected {
        return Err(InventoryMutationError0104::ExactPayloadSize {
            provenance,
            expected,
            actual: payload.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
