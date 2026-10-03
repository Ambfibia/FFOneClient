//! Production protocol boundary for clean Retrobution `UserEquip` Item mode.
//!
//! The UI emits typed slot endpoints; this module validates those endpoints
//! against the current authoritative 9+50 snapshot, builds the exact 0104
//! requests, and correlates replies. It never applies a speculative item
//! mutation. [`InventoryRuntime0104`] remains the sole local slot authority.

use crate::{
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryLocation0104,
        InventoryRuntime0104,
    },
    user_equip_ui::{USER_EQUIP_EQUIPMENT_STRIP_ORDER, UserEquipSlotEndpoint},
};
use bevy::prelude::Resource;
use ffone_protocol::{
    CharacterEquipSlot0104, InventoryPacket0104, ItemBase0104, ItemChestOpenFailure0104,
    ItemChestOpenRequest0104, ItemChestOpenSuccess0104, ItemMoveRequest0104,
    ItemMoveSuccessPacket0104, ItemUsePacket0104, ItemUseRequest0104, PcItemDeleteRequest0104,
    PcItemDeleteSuccess0104,
};
use std::{error::Error, fmt};

pub const USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104: f32 = 10.0;
/// A timed-out request keeps ownership of its packet family briefly so a
/// delayed authoritative reply cannot be misrouted to another feature. The
/// tombstone itself is bounded: it must never reserve Vendor/QuickSlot packet
/// families for the rest of the world session.
pub const USER_EQUIP_LATE_REPLY_GRACE_SECONDS_0104: f32 = 30.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipPendingRequest0104 {
    Move(ItemMoveRequest0104),
    Use(ItemUseRequest0104),
    ChestOpen(ItemChestOpenRequest0104),
    Delete(PcItemDeleteRequest0104),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipPendingKind0104 {
    Move,
    Use,
    ChestOpen,
    Delete,
}

impl UserEquipPendingRequest0104 {
    #[must_use]
    pub const fn kind(self) -> UserEquipPendingKind0104 {
        match self {
            Self::Move(_) => UserEquipPendingKind0104::Move,
            Self::Use(_) => UserEquipPendingKind0104::Use,
            Self::ChestOpen(_) => UserEquipPendingKind0104::ChestOpen,
            Self::Delete(_) => UserEquipPendingKind0104::Delete,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct UserEquipReplyTombstone0104 {
    request: UserEquipPendingRequest0104,
    elapsed_seconds: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct UserEquipProductionRuntime0104 {
    pending: Option<UserEquipPendingRequest0104>,
    /// Timed-out requests remain independent, bounded reply-routing
    /// tombstones. Neither distinct families nor consecutive requests of one
    /// family overwrite each other during the late-reply grace period.
    expired: Vec<UserEquipReplyTombstone0104>,
    pending_elapsed_seconds: f32,
}

impl UserEquipProductionRuntime0104 {
    #[must_use]
    pub const fn pending(&self) -> Option<UserEquipPendingRequest0104> {
        self.pending
    }

    #[must_use]
    pub const fn send_pending(&self) -> bool {
        self.pending.is_some()
    }

    #[must_use]
    pub fn owns_move_reply_family(&self) -> bool {
        matches!(self.pending, Some(UserEquipPendingRequest0104::Move(_)))
            || self
                .expired
                .iter()
                .any(|expired| expired.request.kind() == UserEquipPendingKind0104::Move)
    }

    /// Combat only waits for the currently in-flight move that touches the
    /// equipped Hand endpoint. General inventory/apparel moves and bounded
    /// late-reply tombstones must not suppress ordinary world attacks.
    #[must_use]
    pub fn active_hand_move_pending(&self) -> bool {
        matches!(
            self.pending,
            Some(UserEquipPendingRequest0104::Move(request))
                if move_request_touches_equipment_hand(request)
        )
    }

    #[must_use]
    pub fn owns_use_reply_family(&self) -> bool {
        matches!(self.pending, Some(UserEquipPendingRequest0104::Use(_)))
            || self
                .expired
                .iter()
                .any(|expired| expired.request.kind() == UserEquipPendingKind0104::Use)
    }

    #[must_use]
    pub fn owns_chest_open_reply_family(&self) -> bool {
        matches!(
            self.pending,
            Some(UserEquipPendingRequest0104::ChestOpen(_))
        ) || self
            .expired
            .iter()
            .any(|expired| expired.request.kind() == UserEquipPendingKind0104::ChestOpen)
    }

    #[must_use]
    pub fn owns_delete_reply_family(&self) -> bool {
        matches!(self.pending, Some(UserEquipPendingRequest0104::Delete(_)))
            || self
                .expired
                .iter()
                .any(|expired| expired.request.kind() == UserEquipPendingKind0104::Delete)
    }

    pub fn begin(
        &mut self,
        request: UserEquipPendingRequest0104,
    ) -> Result<(), UserEquipProductionError0104> {
        if let Some(pending) = self.pending {
            return Err(UserEquipProductionError0104::RequestPending(pending.kind()));
        }
        if self
            .expired
            .iter()
            .any(|expired| late_reply_identity_conflicts(expired.request, request))
        {
            // Protocol 0104 carries no nonce. Retrying an identity that the
            // reply ABI cannot distinguish would let an old reply unlock a
            // request that is still genuinely in flight.
            return Err(UserEquipProductionError0104::AmbiguousLateReply(
                request.kind(),
            ));
        }
        self.pending = Some(request);
        self.pending_elapsed_seconds = 0.0;
        Ok(())
    }

    pub fn cancel(&mut self) -> Option<UserEquipPendingRequest0104> {
        self.pending_elapsed_seconds = 0.0;
        self.pending.take()
    }

    /// Closes the interactive UserEquip session without surrendering routing
    /// ownership of an already-sent request. A mutually-exclusive modal such
    /// as BankMode may replace the shell before the shard reply arrives; the
    /// bounded tombstone prevents that late reply from satisfying the new
    /// modal's unrelated request.
    pub fn close_session_preserving_late_replies(&mut self) -> Option<UserEquipPendingRequest0104> {
        let request = self.cancel();
        if let Some(request) = request {
            self.expired.push(UserEquipReplyTombstone0104 {
                request,
                elapsed_seconds: 0.0,
            });
        }
        request
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advances the clean send lock. OpenFusion silently drops some invalid
    /// item moves, so native production releases a request after a bounded
    /// interval instead of deadlocking `MY STUFF` forever.
    pub fn tick(&mut self, delta_seconds: f32) -> Option<UserEquipPendingRequest0104> {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return None;
        }
        for expired in &mut self.expired {
            expired.elapsed_seconds += delta_seconds;
        }
        self.expired
            .retain(|expired| expired.elapsed_seconds < USER_EQUIP_LATE_REPLY_GRACE_SECONDS_0104);
        if self.pending.is_none() {
            return None;
        }
        self.pending_elapsed_seconds += delta_seconds;
        if self.pending_elapsed_seconds < USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104 {
            return None;
        }
        let expired = self.close_session_preserving_late_replies();
        expired
    }

    /// Accepts the authoritative move reply for the one in-flight request.
    /// OpenFusion reverses endpoint labels for its non-stack swap response, so
    /// correlation deliberately treats the endpoint pair as unordered while
    /// the inventory reducer still applies packet fields literally.
    pub fn observe_inventory_packet(&mut self, packet: &InventoryPacket0104) -> bool {
        let InventoryPacket0104::ItemMoveSuccess(reply) = packet else {
            return false;
        };
        if let Some(UserEquipPendingRequest0104::Move(request)) = self.pending
            && item_move_reply_matches_request(request, *reply)
        {
            self.cancel();
            return true;
        }
        if let Some(index) = self.expired.iter().position(|expired| {
            let UserEquipPendingRequest0104::Move(request) = expired.request else {
                return false;
            };
            item_move_reply_matches_request(request, *reply)
        }) {
            self.expired.remove(index);
            return true;
        }
        false
    }

    pub fn observe_item_use_packet(&mut self, packet: &ItemUsePacket0104) -> bool {
        self.observe_item_use_packet_for_owner(packet, None)
    }

    /// Correlates a use reply to both its immutable endpoint and, for success
    /// packets, the authoritative local PC. Failure ABI carries no PC ID and
    /// remains owned by the one active UserEquip use lock.
    pub fn observe_item_use_packet_for_owner(
        &mut self,
        packet: &ItemUsePacket0104,
        owner_pc_id: Option<i32>,
    ) -> bool {
        let matches_request = |request: ItemUseRequest0104| match packet {
            ItemUsePacket0104::Success(success) => {
                let prefix = success.prefix();
                owner_pc_id.is_none_or(|owner_pc_id| prefix.pc_id == owner_pc_id)
                    && prefix.item_location == request.item_location
                    && prefix.slot_num == request.slot_num
            }
            // The failure ABI has no request identity. UserEquip owns at most
            // one send-locked use, and gameplay/QuickSlot input is suppressed
            // while this modal is open, making this the exact active owner.
            ItemUsePacket0104::Failure(_) => true,
            ItemUsePacket0104::Broadcast(_) => false,
        };
        if let Some(UserEquipPendingRequest0104::Use(request)) = self.pending
            && matches_request(request)
        {
            self.cancel();
            return true;
        }
        if let Some(index) = self.expired.iter().position(|expired| {
            let UserEquipPendingRequest0104::Use(request) = expired.request else {
                return false;
            };
            matches_request(request)
        }) {
            self.expired.remove(index);
            return true;
        }
        false
    }

    pub fn observe_delete_success(&mut self, reply: PcItemDeleteSuccess0104) -> bool {
        let matches_request = |request: PcItemDeleteRequest0104| {
            reply.item_location == request.item_location && reply.slot_num == request.slot_num
        };
        if let Some(UserEquipPendingRequest0104::Delete(request)) = self.pending
            && matches_request(request)
        {
            self.cancel();
            return true;
        }
        if let Some(index) = self.expired.iter().position(|expired| {
            let UserEquipPendingRequest0104::Delete(request) = expired.request else {
                return false;
            };
            matches_request(request)
        }) {
            self.expired.remove(index);
            return true;
        }
        false
    }

    pub fn observe_chest_open_success(&mut self, reply: ItemChestOpenSuccess0104) -> bool {
        self.observe_chest_open_slot(reply.slot_num)
    }

    pub fn observe_chest_open_failure(&mut self, reply: ItemChestOpenFailure0104) -> bool {
        self.observe_chest_open_slot(reply.slot_num)
    }

    fn observe_chest_open_slot(&mut self, slot_num: i32) -> bool {
        let matches_request = |request: ItemChestOpenRequest0104| request.slot_num == slot_num;
        if let Some(UserEquipPendingRequest0104::ChestOpen(request)) = self.pending
            && matches_request(request)
        {
            self.cancel();
            return true;
        }
        if let Some(index) = self.expired.iter().position(|expired| {
            let UserEquipPendingRequest0104::ChestOpen(request) = expired.request else {
                return false;
            };
            matches_request(request)
        }) {
            self.expired.remove(index);
            return true;
        }
        false
    }
}

#[must_use]
const fn move_request_touches_equipment_hand(request: ItemMoveRequest0104) -> bool {
    let equipment = InventoryLocation0104::Equipment as i32;
    let hand = CharacterEquipSlot0104::Hand as i32;
    (request.from_location == equipment && request.from_slot_num == hand)
        || (request.to_location == equipment && request.to_slot_num == hand)
}

/// Reply identity is weaker than request byte equality: move endpoints may be
/// reversed, item-use failure contains no endpoint, and delete success carries
/// only location+slot. Cross-family requests remain independent.
#[must_use]
const fn late_reply_identity_conflicts(
    expired: UserEquipPendingRequest0104,
    next: UserEquipPendingRequest0104,
) -> bool {
    match (expired, next) {
        (UserEquipPendingRequest0104::Move(old), UserEquipPendingRequest0104::Move(new)) => {
            let same = old.from_location == new.from_location
                && old.from_slot_num == new.from_slot_num
                && old.to_location == new.to_location
                && old.to_slot_num == new.to_slot_num;
            let reversed = old.from_location == new.to_location
                && old.from_slot_num == new.to_slot_num
                && old.to_location == new.from_location
                && old.to_slot_num == new.from_slot_num;
            same || reversed
        }
        // A failure reply has no immutable endpoint, so every in-family retry
        // is ambiguous until all Use tombstones expire.
        (UserEquipPendingRequest0104::Use(_), UserEquipPendingRequest0104::Use(_)) => true,
        (
            UserEquipPendingRequest0104::ChestOpen(old),
            UserEquipPendingRequest0104::ChestOpen(new),
        ) => old.slot_num == new.slot_num,
        (UserEquipPendingRequest0104::Delete(old), UserEquipPendingRequest0104::Delete(new)) => {
            old.item_location == new.item_location && old.slot_num == new.slot_num
        }
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipProductionError0104 {
    RequestPending(UserEquipPendingKind0104),
    AmbiguousLateReply(UserEquipPendingKind0104),
    InvalidInventorySlot(usize),
    InvalidEquipmentEndpoint {
        visual_index: usize,
        wire_slot_index: usize,
    },
    EmptySource(UserEquipSlotEndpoint),
    UnsupportedUseItemType(i16),
    UnsupportedChestItemType(i16),
    SameEndpoint(UserEquipSlotEndpoint),
    IncompatibleEquipmentTarget {
        item_type: i16,
        wire_slot_index: usize,
    },
    OccupiedUnequipTarget(usize),
    IncompatibleEquipmentSwap {
        item_type: i16,
        wire_slot_index: usize,
    },
}

impl fmt::Display for UserEquipProductionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RequestPending(kind) => write!(formatter, "{kind:?} request is already pending"),
            Self::AmbiguousLateReply(kind) => write!(
                formatter,
                "{kind:?} request duplicates an endpoint awaiting its bounded late reply"
            ),
            Self::InvalidInventorySlot(slot) => {
                write!(
                    formatter,
                    "inventory slot {slot} is outside the 50-slot authority"
                )
            }
            Self::InvalidEquipmentEndpoint {
                visual_index,
                wire_slot_index,
            } => write!(
                formatter,
                "equipment endpoint visual {visual_index} / wire {wire_slot_index} is not in the clean strip"
            ),
            Self::EmptySource(source) => write!(formatter, "item source {source:?} is empty"),
            Self::UnsupportedUseItemType(item_type) => write!(
                formatter,
                "item type {item_type} cannot be used from clean UserEquip Item mode"
            ),
            Self::UnsupportedChestItemType(item_type) => write!(
                formatter,
                "item type {item_type} cannot be opened as a chest from clean UserEquip Item mode"
            ),
            Self::SameEndpoint(endpoint) => {
                write!(formatter, "item move endpoint {endpoint:?} is a no-op")
            }
            Self::IncompatibleEquipmentTarget {
                item_type,
                wire_slot_index,
            } => write!(
                formatter,
                "item type {item_type} cannot be equipped in wire slot {wire_slot_index}"
            ),
            Self::OccupiedUnequipTarget(slot) => write!(
                formatter,
                "clean unequip requires an empty inventory target, but slot {slot} is occupied"
            ),
            Self::IncompatibleEquipmentSwap {
                item_type,
                wire_slot_index,
            } => write!(
                formatter,
                "equipment swap would place item type {item_type} in incompatible wire slot {wire_slot_index}"
            ),
        }
    }
}

impl Error for UserEquipProductionError0104 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedEndpoint0104 {
    location: InventoryLocation0104,
    index: usize,
}

impl ResolvedEndpoint0104 {
    #[must_use]
    const fn wire_location(self) -> i32 {
        self.location.wire_value()
    }

    #[must_use]
    const fn wire_slot(self) -> i32 {
        self.index as i32
    }
}

fn resolve_endpoint(
    endpoint: UserEquipSlotEndpoint,
) -> Result<ResolvedEndpoint0104, UserEquipProductionError0104> {
    match endpoint {
        UserEquipSlotEndpoint::Inventory { slot_index } => {
            if slot_index >= INVENTORY_SLOT_COUNT_0104 {
                return Err(UserEquipProductionError0104::InvalidInventorySlot(
                    slot_index,
                ));
            }
            Ok(ResolvedEndpoint0104 {
                location: InventoryLocation0104::Inventory,
                index: slot_index,
            })
        }
        UserEquipSlotEndpoint::Equipment {
            visual_index,
            wire_slot_index,
        } => {
            let valid = wire_slot_index < EQUIPMENT_SLOT_COUNT_0104
                && USER_EQUIP_EQUIPMENT_STRIP_ORDER
                    .get(visual_index)
                    .is_some_and(|spec| spec.wire_slot_index == wire_slot_index);
            if !valid {
                return Err(UserEquipProductionError0104::InvalidEquipmentEndpoint {
                    visual_index,
                    wire_slot_index,
                });
            }
            Ok(ResolvedEndpoint0104 {
                location: InventoryLocation0104::Equipment,
                index: wire_slot_index,
            })
        }
    }
}

fn endpoint_item(runtime: &InventoryRuntime0104, endpoint: ResolvedEndpoint0104) -> ItemBase0104 {
    match endpoint.location {
        InventoryLocation0104::Equipment => runtime.equipment()[endpoint.index],
        InventoryLocation0104::Inventory => runtime.inventory()[endpoint.index],
    }
}

#[must_use]
pub const fn equipment_wire_slot_accepts_item(item_type: i16, wire_slot_index: usize) -> bool {
    match item_type {
        0 => wire_slot_index == 0 || wire_slot_index == 7,
        1..=6 => wire_slot_index == item_type as usize,
        10 => wire_slot_index == 8,
        _ => false,
    }
}

pub fn prepare_user_equip_move_0104(
    runtime: &InventoryRuntime0104,
    from: UserEquipSlotEndpoint,
    to: UserEquipSlotEndpoint,
) -> Result<ItemMoveRequest0104, UserEquipProductionError0104> {
    if from == to {
        return Err(UserEquipProductionError0104::SameEndpoint(from));
    }
    let from_resolved = resolve_endpoint(from)?;
    let to_resolved = resolve_endpoint(to)?;
    if from_resolved == to_resolved {
        return Err(UserEquipProductionError0104::SameEndpoint(from));
    }
    let from_item = endpoint_item(runtime, from_resolved);
    if InventoryRuntime0104::item_is_empty(from_item) {
        return Err(UserEquipProductionError0104::EmptySource(from));
    }
    let to_item = endpoint_item(runtime, to_resolved);
    if to_resolved.location == InventoryLocation0104::Equipment
        && !equipment_wire_slot_accepts_item(from_item.item_type, to_resolved.index)
    {
        return Err(UserEquipProductionError0104::IncompatibleEquipmentTarget {
            item_type: from_item.item_type,
            wire_slot_index: to_resolved.index,
        });
    }
    if from_resolved.location == InventoryLocation0104::Equipment
        && to_resolved.location == InventoryLocation0104::Inventory
        && !InventoryRuntime0104::item_is_empty(to_item)
    {
        return Err(UserEquipProductionError0104::OccupiedUnequipTarget(
            to_resolved.index,
        ));
    }
    if from_resolved.location == InventoryLocation0104::Equipment
        && to_resolved.location == InventoryLocation0104::Equipment
        && !InventoryRuntime0104::item_is_empty(to_item)
        && !equipment_wire_slot_accepts_item(to_item.item_type, from_resolved.index)
    {
        return Err(UserEquipProductionError0104::IncompatibleEquipmentSwap {
            item_type: to_item.item_type,
            wire_slot_index: from_resolved.index,
        });
    }

    Ok(ItemMoveRequest0104 {
        from_location: from_resolved.wire_location(),
        from_slot_num: from_resolved.wire_slot(),
        to_location: to_resolved.wire_location(),
        to_slot_num: to_resolved.wire_slot(),
    })
}

pub fn prepare_user_equip_use_0104(
    runtime: &InventoryRuntime0104,
    slot_index: usize,
    nano_slot: i16,
) -> Result<ItemUseRequest0104, UserEquipProductionError0104> {
    let endpoint = UserEquipSlotEndpoint::Inventory { slot_index };
    let resolved = resolve_endpoint(endpoint)?;
    let item = endpoint_item(runtime, resolved);
    if InventoryRuntime0104::item_is_empty(item) {
        return Err(UserEquipProductionError0104::EmptySource(endpoint));
    }
    if item.item_type != 7 {
        return Err(UserEquipProductionError0104::UnsupportedUseItemType(
            item.item_type,
        ));
    }
    Ok(ItemUseRequest0104 {
        item_location: InventoryLocation0104::Inventory.wire_value(),
        slot_num: resolved.wire_slot(),
        nano_slot,
    })
}

pub fn prepare_user_equip_chest_open_0104(
    runtime: &InventoryRuntime0104,
    slot_index: usize,
) -> Result<ItemChestOpenRequest0104, UserEquipProductionError0104> {
    let endpoint = UserEquipSlotEndpoint::Inventory { slot_index };
    let resolved = resolve_endpoint(endpoint)?;
    let item = endpoint_item(runtime, resolved);
    if InventoryRuntime0104::item_is_empty(item) {
        return Err(UserEquipProductionError0104::EmptySource(endpoint));
    }
    if item.item_type != 9 {
        return Err(UserEquipProductionError0104::UnsupportedChestItemType(
            item.item_type,
        ));
    }
    Ok(ItemChestOpenRequest0104 {
        item_location: InventoryLocation0104::Inventory.wire_value(),
        slot_num: resolved.wire_slot(),
        // Clean `CnEquip.UseItem` reconstructs `ChestItem` and explicitly
        // zeroes its time limit instead of forwarding the inventory value.
        chest_item: ItemBase0104 {
            time_limit: 0,
            ..item
        },
    })
}

pub fn prepare_user_equip_delete_0104(
    runtime: &InventoryRuntime0104,
    slot_index: usize,
) -> Result<PcItemDeleteRequest0104, UserEquipProductionError0104> {
    let endpoint = UserEquipSlotEndpoint::Inventory { slot_index };
    let resolved = resolve_endpoint(endpoint)?;
    if InventoryRuntime0104::item_is_empty(endpoint_item(runtime, resolved)) {
        return Err(UserEquipProductionError0104::EmptySource(endpoint));
    }
    Ok(PcItemDeleteRequest0104 {
        item_location: InventoryLocation0104::Inventory.wire_value(),
        slot_num: resolved.wire_slot(),
    })
}

#[must_use]
pub const fn item_move_reply_matches_request(
    request: ItemMoveRequest0104,
    reply: ItemMoveSuccessPacket0104,
) -> bool {
    let request_from = (request.from_location, request.from_slot_num);
    let request_to = (request.to_location, request.to_slot_num);
    let reply_from = (reply.from_location, reply.from_slot_num);
    let reply_to = (reply.to_location, reply.to_slot_num);
    (request_from.0 == reply_from.0
        && request_from.1 == reply_from.1
        && request_to.0 == reply_to.0
        && request_to.1 == reply_to.1)
        || (request_from.0 == reply_to.0
            && request_from.1 == reply_to.1
            && request_to.0 == reply_from.0
            && request_to.1 == reply_from.1)
}

#[cfg(test)]
mod tests;
