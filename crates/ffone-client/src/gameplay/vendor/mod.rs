//! Production protocol-0104 owner for clean Retrobution VendorMode.
//!
//! This layer correlates one request at a time and applies no speculative
//! inventory, Taros, battery, or buyback mutation. The passive UI emits
//! semantic intents; this runtime maps them to exact wire requests and accepts
//! state only from the matching strict server reply.

use std::{collections::VecDeque, error::Error, fmt};

use bevy::prelude::Resource;
use ffone_protocol::{
    PcDisassembleItemRequest0104, PcItemDeleteRequest0104, VendorBatteryBuyRequest0104,
    VendorItemBuyRequest0104, VendorItemRestoreBuyRequest0104, VendorItemSellRequest0104,
    VendorPacket0104, VendorStartRequest0104, VendorTableUpdateRequest0104,
};

use crate::{
    inventory_runtime::{InventoryMutationError0104, InventoryRuntime0104},
    vendor_ui::{
        VendorCatalogEntry0104, VendorIntent0104, VendorRecentBuyEntry0104,
        VendorServerFailure0104, VendorSession0104,
    },
};

/// OpenFusion's current 0104 player buyback list keeps five entries.
pub const OPENFUSION_VENDOR_BUYBACK_CAPACITY_0104: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveVendorSourceNpc0104 {
    pub runtime_npc_id: i32,
    pub table_npc_id: i32,
    pub ai_type: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorOperation0104 {
    Start,
    TableUpdate,
    Buy,
    BatteryBuy,
    Sell,
    Restore,
    Delete,
    Disassemble,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingVendorRequest0104 {
    Start {
        npc_id: i32,
        vendor_id: i32,
    },
    TableUpdate {
        npc_id: i32,
        vendor_id: i32,
    },
    Buy {
        inventory_slot: i32,
        item: ffone_protocol::ItemBase0104,
    },
    BatteryBuy,
    Sell {
        inventory_slot: i32,
        item: ffone_protocol::ItemBase0104,
    },
    Restore {
        inventory_slot: i32,
        item: ffone_protocol::ItemBase0104,
        recent_index: usize,
    },
    Delete {
        item_location: i32,
        inventory_slot: i32,
    },
    Disassemble {
        inventory_slot: i32,
    },
}

impl PendingVendorRequest0104 {
    const fn operation(self) -> VendorOperation0104 {
        match self {
            Self::Start { .. } => VendorOperation0104::Start,
            Self::TableUpdate { .. } => VendorOperation0104::TableUpdate,
            Self::Buy { .. } => VendorOperation0104::Buy,
            Self::BatteryBuy => VendorOperation0104::BatteryBuy,
            Self::Sell { .. } => VendorOperation0104::Sell,
            Self::Restore { .. } => VendorOperation0104::Restore,
            Self::Delete { .. } => VendorOperation0104::Delete,
            Self::Disassemble { .. } => VendorOperation0104::Disassemble,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorOutboundRequest0104 {
    Start(VendorStartRequest0104),
    TableUpdate(VendorTableUpdateRequest0104),
    Buy(VendorItemBuyRequest0104),
    BatteryBuy(VendorBatteryBuyRequest0104),
    Sell(VendorItemSellRequest0104),
    Restore(VendorItemRestoreBuyRequest0104),
    Delete(PcItemDeleteRequest0104),
    Disassemble(PcDisassembleItemRequest0104),
}

impl VendorOutboundRequest0104 {
    #[must_use]
    pub const fn operation(self) -> VendorOperation0104 {
        match self {
            Self::Start(_) => VendorOperation0104::Start,
            Self::TableUpdate(_) => VendorOperation0104::TableUpdate,
            Self::Buy(_) => VendorOperation0104::Buy,
            Self::BatteryBuy(_) => VendorOperation0104::BatteryBuy,
            Self::Sell(_) => VendorOperation0104::Sell,
            Self::Restore(_) => VendorOperation0104::Restore,
            Self::Delete(_) => VendorOperation0104::Delete,
            Self::Disassemble(_) => VendorOperation0104::Disassemble,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorProductionEvent0104 {
    StartAccepted {
        table_request: VendorTableUpdateRequest0104,
    },
    TableAccepted,
    BuyAccepted {
        candy: i32,
    },
    BatteryAccepted {
        candy: i32,
        weapon_battery: i32,
        nano_battery: i32,
    },
    SellAccepted {
        candy: i32,
        sold_item: ffone_protocol::ItemBase0104,
    },
    RestoreAccepted {
        candy: i32,
    },
    DeleteAccepted,
    DisassembleAccepted,
    Failed {
        operation: VendorOperation0104,
        error_code: i32,
        ui_failure: Option<VendorServerFailure0104>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorProductionError0104 {
    AlreadyActive,
    NoActiveSession,
    RequestPending {
        operation: VendorOperation0104,
    },
    NoPendingRequest,
    UnexpectedReply {
        pending: VendorOperation0104,
        reply: VendorOperation0104,
    },
    ReplyIdentityMismatch {
        operation: VendorOperation0104,
        detail: &'static str,
    },
    IntentIdentityMismatch {
        operation: VendorOperation0104,
    },
    IntentStateMismatch {
        operation: VendorOperation0104,
        detail: &'static str,
    },
    ListIdOutOfRange {
        value: i32,
    },
    SlotOutOfRange {
        value: usize,
    },
    TableVendorMismatch {
        item_index: usize,
        expected_vendor_id: i32,
        packet_vendor_id: i32,
    },
    MissingRecentItem,
    Inventory(InventoryMutationError0104),
}

impl fmt::Display for VendorProductionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyActive => f.write_str("VendorMode already owns an active session"),
            Self::NoActiveSession => f.write_str("VendorMode has no accepted server session"),
            Self::RequestPending { operation } => {
                write!(f, "VendorMode request {operation:?} is still pending")
            }
            Self::NoPendingRequest => f.write_str("VendorMode reply has no pending request"),
            Self::UnexpectedReply { pending, reply } => write!(
                f,
                "VendorMode received {reply:?} while {pending:?} is pending"
            ),
            Self::ReplyIdentityMismatch { operation, detail } => {
                write!(
                    f,
                    "VendorMode {operation:?} reply identity mismatch: {detail}"
                )
            }
            Self::IntentIdentityMismatch { operation } => {
                write!(
                    f,
                    "VendorMode {operation:?} intent does not belong to the accepted session"
                )
            }
            Self::IntentStateMismatch { operation, detail } => {
                write!(f, "VendorMode {operation:?} intent is stale: {detail}")
            }
            Self::ListIdOutOfRange { value } => {
                write!(f, "VendorMode list ID {value} does not fit protocol i8")
            }
            Self::SlotOutOfRange { value } => {
                write!(
                    f,
                    "VendorMode inventory slot {value} does not fit protocol i32"
                )
            }
            Self::TableVendorMismatch {
                item_index,
                expected_vendor_id,
                packet_vendor_id,
            } => write!(
                f,
                "VendorMode table item {item_index} belongs to vendor {packet_vendor_id}, expected {expected_vendor_id}"
            ),
            Self::MissingRecentItem => {
                f.write_str("VendorMode restore reply has no identical recent-buy entry")
            }
            Self::Inventory(error) => write!(f, "VendorMode inventory update rejected: {error}"),
        }
    }
}

impl Error for VendorProductionError0104 {}

impl From<InventoryMutationError0104> for VendorProductionError0104 {
    fn from(value: InventoryMutationError0104) -> Self {
        Self::Inventory(value)
    }
}

#[derive(Debug, Default, Resource)]
pub struct VendorProductionRuntime0104 {
    active_source_npc: Option<ActiveVendorSourceNpc0104>,
    session: Option<VendorSession0104>,
    pending: Option<PendingVendorRequest0104>,
    catalog_entries: Vec<VendorCatalogEntry0104>,
    recent_entries: VecDeque<VendorRecentBuyEntry0104>,
}

impl VendorProductionRuntime0104 {
    #[must_use]
    pub const fn active_source_npc(&self) -> Option<ActiveVendorSourceNpc0104> {
        self.active_source_npc
    }

    #[must_use]
    pub const fn session(&self) -> Option<VendorSession0104> {
        self.session
    }

    #[must_use]
    pub fn catalog_entries(&self) -> &[VendorCatalogEntry0104] {
        &self.catalog_entries
    }

    pub fn recent_entries(&self) -> impl ExactSizeIterator<Item = &VendorRecentBuyEntry0104> {
        self.recent_entries.iter()
    }

    #[must_use]
    pub fn request_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Returns the exact outstanding delete endpoint for packet-family
    /// arbitration with UserEquip late-reply tombstones.
    #[must_use]
    pub const fn pending_delete_request(&self) -> Option<PcItemDeleteRequest0104> {
        match self.pending {
            Some(PendingVendorRequest0104::Delete {
                item_location,
                inventory_slot,
            }) => Some(PcItemDeleteRequest0104 {
                item_location,
                slot_num: inventory_slot,
            }),
            _ => None,
        }
    }

    #[must_use]
    pub fn modal_active(&self) -> bool {
        self.active_source_npc.is_some() || self.pending.is_some()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Local VendorMode close has no protocol packet. The OpenFusion buyback
    /// FIFO remains until disconnect, matching its per-player server state.
    pub fn close_mode(&mut self) {
        self.active_source_npc = None;
        self.session = None;
        self.pending = None;
        self.catalog_entries.clear();
    }

    pub fn begin_start(
        &mut self,
        source: ActiveVendorSourceNpc0104,
    ) -> Result<VendorOutboundRequest0104, VendorProductionError0104> {
        if self.active_source_npc.is_some() || self.pending.is_some() {
            return Err(VendorProductionError0104::AlreadyActive);
        }
        let request = VendorStartRequest0104 {
            // The placed NPC is checked for proximity; vendor_id selects its table.
            npc_id: source.runtime_npc_id,
            vendor_id: source.table_npc_id,
        };
        self.active_source_npc = Some(source);
        self.session = None;
        self.catalog_entries.clear();
        self.pending = Some(PendingVendorRequest0104::Start {
            npc_id: request.npc_id,
            vendor_id: request.vendor_id,
        });
        Ok(VendorOutboundRequest0104::Start(request))
    }

    pub fn begin_intent(
        &mut self,
        intent: VendorIntent0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<VendorOutboundRequest0104, VendorProductionError0104> {
        if let Some(pending) = self.pending {
            return Err(VendorProductionError0104::RequestPending {
                operation: pending.operation(),
            });
        }
        let session = self
            .session
            .ok_or(VendorProductionError0104::NoActiveSession)?;
        if inventory.owner_pc_id() <= 0 {
            return Err(VendorProductionError0104::IntentStateMismatch {
                operation: intent_operation(intent),
                detail: "authoritative inventory has no valid owner",
            });
        }

        let (pending, outbound) = match intent {
            VendorIntent0104::Buy(intent) => {
                ensure_identity(session, intent.identity, VendorOperation0104::Buy)?;
                self.ensure_catalog_item(intent.item, VendorOperation0104::Buy)?;
                let inventory_slot = wire_slot(intent.inventory_slot)?;
                ensure_empty_destination(
                    inventory,
                    intent.inventory_slot,
                    VendorOperation0104::Buy,
                )?;
                let list_id = wire_list_id(intent.identity.list_id)?;
                (
                    PendingVendorRequest0104::Buy {
                        inventory_slot,
                        item: intent.item,
                    },
                    VendorOutboundRequest0104::Buy(VendorItemBuyRequest0104 {
                        npc_id: intent.identity.npc_id,
                        vendor_id: intent.identity.vendor_id,
                        list_id,
                        item: intent.item,
                        inventory_slot,
                    }),
                )
            }
            VendorIntent0104::BuyGeneral(intent) => {
                ensure_identity(session, intent.identity, VendorOperation0104::Buy)?;
                self.ensure_catalog_item(intent.item, VendorOperation0104::Buy)?;
                let inventory_slot = wire_slot(intent.inventory_slot)?;
                ensure_empty_destination(
                    inventory,
                    intent.inventory_slot,
                    VendorOperation0104::Buy,
                )?;
                let list_id = wire_list_id(intent.identity.list_id)?;
                (
                    PendingVendorRequest0104::Buy {
                        inventory_slot,
                        item: intent.item,
                    },
                    VendorOutboundRequest0104::Buy(VendorItemBuyRequest0104 {
                        npc_id: intent.identity.npc_id,
                        vendor_id: intent.identity.vendor_id,
                        list_id,
                        item: intent.item,
                        inventory_slot,
                    }),
                )
            }
            VendorIntent0104::Battery(intent) => {
                ensure_identity(session, intent.identity, VendorOperation0104::BatteryBuy)?;
                self.ensure_catalog_item(intent.item, VendorOperation0104::BatteryBuy)?;
                let list_id = wire_list_id(intent.identity.list_id)?;
                (
                    PendingVendorRequest0104::BatteryBuy,
                    VendorOutboundRequest0104::BatteryBuy(VendorBatteryBuyRequest0104 {
                        npc_id: intent.identity.npc_id,
                        vendor_id: intent.identity.vendor_id,
                        list_id,
                        item: intent.item,
                    }),
                )
            }
            VendorIntent0104::Restore(intent) => {
                ensure_identity(session, intent.identity, VendorOperation0104::Restore)?;
                let inventory_slot = wire_slot(intent.inventory_slot)?;
                ensure_empty_destination(
                    inventory,
                    intent.inventory_slot,
                    VendorOperation0104::Restore,
                )?;
                let recent_index = intent
                    .restore_list_id
                    .checked_sub(1)
                    .and_then(|value| usize::try_from(value).ok());
                let recent_index = recent_index
                    .filter(|&index| {
                        self.recent_entries
                            .get(index)
                            .is_some_and(|entry| entry.item == intent.item)
                    })
                    .ok_or(VendorProductionError0104::IntentStateMismatch {
                        operation: VendorOperation0104::Restore,
                        detail: "buyback row no longer owns the requested item",
                    })?;
                let wire_item = ffone_protocol::ItemBase0104 {
                    time_limit: 0,
                    ..intent.item
                };
                let list_id = wire_list_id(intent.restore_list_id)?;
                (
                    PendingVendorRequest0104::Restore {
                        inventory_slot,
                        item: wire_item,
                        recent_index,
                    },
                    VendorOutboundRequest0104::Restore(VendorItemRestoreBuyRequest0104 {
                        npc_id: intent.identity.npc_id,
                        vendor_id: intent.identity.vendor_id,
                        list_id,
                        item: wire_item,
                        inventory_slot,
                    }),
                )
            }
            VendorIntent0104::Sell(intent) => {
                let inventory_slot = wire_slot(intent.inventory_slot)?;
                ensure_source_item(
                    inventory,
                    intent.inventory_slot,
                    intent.item,
                    VendorOperation0104::Sell,
                )?;
                (
                    PendingVendorRequest0104::Sell {
                        inventory_slot,
                        item: intent.item,
                    },
                    VendorOutboundRequest0104::Sell(VendorItemSellRequest0104 {
                        inventory_slot,
                        item_count: intent.count,
                    }),
                )
            }
            VendorIntent0104::Delete(intent) => {
                let inventory_slot = wire_slot(intent.inventory_slot)?;
                ensure_source_item(
                    inventory,
                    intent.inventory_slot,
                    intent.item,
                    VendorOperation0104::Delete,
                )?;
                let item_location = intent.location as i32;
                (
                    PendingVendorRequest0104::Delete {
                        item_location,
                        inventory_slot,
                    },
                    VendorOutboundRequest0104::Delete(PcItemDeleteRequest0104 {
                        item_location,
                        slot_num: inventory_slot,
                    }),
                )
            }
            VendorIntent0104::Disassemble(intent) => {
                let inventory_slot = wire_slot(intent.inventory_slot)?;
                ensure_source_item(
                    inventory,
                    intent.inventory_slot,
                    intent.item,
                    VendorOperation0104::Disassemble,
                )?;
                (
                    PendingVendorRequest0104::Disassemble { inventory_slot },
                    VendorOutboundRequest0104::Disassemble(PcDisassembleItemRequest0104 {
                        item_slot: inventory_slot,
                    }),
                )
            }
        };
        self.pending = Some(pending);
        Ok(outbound)
    }

    /// Clears exactly the outstanding send gate after a local transport
    /// failure. The caller closes the mode for failed start/table requests.
    pub fn cancel_pending(&mut self) -> Option<VendorOperation0104> {
        self.pending.take().map(PendingVendorRequest0104::operation)
    }

    pub fn apply_packet(
        &mut self,
        packet: VendorPacket0104,
        inventory: &mut InventoryRuntime0104,
    ) -> Result<VendorProductionEvent0104, VendorProductionError0104> {
        let pending = self
            .pending
            .ok_or(VendorProductionError0104::NoPendingRequest)?;
        let reply_operation = reply_operation(packet);
        if pending.operation() != reply_operation {
            return Err(VendorProductionError0104::UnexpectedReply {
                pending: pending.operation(),
                reply: reply_operation,
            });
        }

        match (pending, packet) {
            (
                PendingVendorRequest0104::Start { npc_id, vendor_id },
                VendorPacket0104::StartSuccess(reply),
            ) => {
                if reply.npc_id != npc_id || reply.vendor_id != vendor_id {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Start,
                        detail: "NPC/vendor pair differs from the request",
                    });
                }
                let source = self
                    .active_source_npc
                    .ok_or(VendorProductionError0104::NoActiveSession)?;
                self.session = Some(VendorSession0104 {
                    requested_npc_id: source.runtime_npc_id,
                    table_vendor_id: source.table_npc_id,
                    accepted_npc_id: reply.npc_id,
                });
                let table_request = VendorTableUpdateRequest0104 {
                    npc_id: reply.vendor_id,
                    vendor_id: reply.vendor_id,
                };
                self.pending = Some(PendingVendorRequest0104::TableUpdate {
                    npc_id: table_request.npc_id,
                    vendor_id: table_request.vendor_id,
                });
                Ok(VendorProductionEvent0104::StartAccepted { table_request })
            }
            (PendingVendorRequest0104::Start { .. }, VendorPacket0104::StartFailure(reply)) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::Start,
                    error_code: reply.error_code,
                    ui_failure: Some(VendorServerFailure0104::Start),
                })
            }
            (
                PendingVendorRequest0104::TableUpdate { npc_id, vendor_id },
                VendorPacket0104::TableSuccess(reply),
            ) => {
                let session = self
                    .session
                    .ok_or(VendorProductionError0104::NoActiveSession)?;
                if npc_id != vendor_id || session.table_vendor_id != vendor_id {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::TableUpdate,
                        detail: "vendor table changed before table reply",
                    });
                }
                let mut entries = Vec::new();
                for (item_index, vendor_item) in reply.items.into_iter().enumerate() {
                    if InventoryRuntime0104::item_is_empty(vendor_item.item) {
                        continue;
                    }
                    if vendor_item.vendor_id != vendor_id {
                        return Err(VendorProductionError0104::TableVendorMismatch {
                            item_index,
                            expected_vendor_id: vendor_id,
                            packet_vendor_id: vendor_item.vendor_id,
                        });
                    }
                    entries.push(VendorCatalogEntry0104 {
                        // Clean InventoryManager assigns sequential SlotID by
                        // non-empty response order and ignores iSortNum here.
                        source_slot_id: entries.len(),
                        item: vendor_item.item,
                    });
                }
                self.catalog_entries = entries;
                self.pending = None;
                Ok(VendorProductionEvent0104::TableAccepted)
            }
            (
                PendingVendorRequest0104::TableUpdate { .. },
                VendorPacket0104::TableFailure(reply),
            ) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::TableUpdate,
                    error_code: reply.error_code,
                    ui_failure: Some(VendorServerFailure0104::TableUpdate),
                })
            }
            (
                PendingVendorRequest0104::Buy {
                    inventory_slot,
                    item,
                },
                VendorPacket0104::BuySuccess(reply),
            ) => {
                if reply.inventory_slot != inventory_slot
                    || reply.item.item_type != item.item_type
                    || reply.item.item_id != item.item_id
                    || reply.item.option != item.option
                {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Buy,
                        detail: "slot or requested item identity differs",
                    });
                }
                inventory.apply_vendor_buy_success(reply)?;
                self.pending = None;
                Ok(VendorProductionEvent0104::BuyAccepted { candy: reply.candy })
            }
            (PendingVendorRequest0104::Buy { .. }, VendorPacket0104::BuyFailure(reply)) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::Buy,
                    error_code: reply.error_code,
                    ui_failure: Some(VendorServerFailure0104::Buy {
                        error_code: reply.error_code,
                    }),
                })
            }
            (PendingVendorRequest0104::BatteryBuy, VendorPacket0104::BatterySuccess(reply)) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::BatteryAccepted {
                    candy: reply.candy,
                    weapon_battery: reply.weapon_battery,
                    nano_battery: reply.nano_battery,
                })
            }
            (PendingVendorRequest0104::BatteryBuy, VendorPacket0104::BatteryFailure(reply)) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::BatteryBuy,
                    error_code: reply.error_code,
                    ui_failure: Some(VendorServerFailure0104::Battery),
                })
            }
            (
                PendingVendorRequest0104::Sell {
                    inventory_slot,
                    item,
                },
                VendorPacket0104::SellSuccess(reply),
            ) => {
                if reply.inventory_slot != inventory_slot
                    || reply.item.item_type != item.item_type
                    || reply.item.item_id != item.item_id
                {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Sell,
                        detail: "slot or sold item identity differs",
                    });
                }
                inventory.apply_vendor_sell_success(reply)?;
                self.recent_entries.push_back(VendorRecentBuyEntry0104 {
                    source_slot_id: self.recent_entries.len(),
                    item: reply.item,
                });
                while self.recent_entries.len() > OPENFUSION_VENDOR_BUYBACK_CAPACITY_0104 {
                    self.recent_entries.pop_front();
                }
                self.reindex_recent();
                self.pending = None;
                Ok(VendorProductionEvent0104::SellAccepted {
                    candy: reply.candy,
                    sold_item: reply.item,
                })
            }
            (PendingVendorRequest0104::Sell { .. }, VendorPacket0104::SellFailure(reply)) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::Sell,
                    error_code: reply.error_code,
                    ui_failure: Some(VendorServerFailure0104::Sell),
                })
            }
            (
                PendingVendorRequest0104::Restore {
                    inventory_slot,
                    item,
                    recent_index,
                },
                VendorPacket0104::RestoreSuccess(reply),
            ) => {
                if reply.inventory_slot != inventory_slot
                    || reply.item.item_type != item.item_type
                    || reply.item.item_id != item.item_id
                    || reply.item.option != item.option
                    || reply.item.time_limit != item.time_limit
                {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Restore,
                        detail: "slot or buyback item differs",
                    });
                }
                if self.recent_entries.get(recent_index).is_none() {
                    return Err(VendorProductionError0104::MissingRecentItem);
                }
                inventory.apply_vendor_restore_success(reply)?;
                self.recent_entries.remove(recent_index);
                self.reindex_recent();
                self.pending = None;
                Ok(VendorProductionEvent0104::RestoreAccepted { candy: reply.candy })
            }
            (PendingVendorRequest0104::Restore { .. }, VendorPacket0104::RestoreFailure(reply)) => {
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::Restore,
                    error_code: reply.error_code,
                    ui_failure: Some(VendorServerFailure0104::Restore {
                        error_code: reply.error_code,
                    }),
                })
            }
            (
                PendingVendorRequest0104::Delete {
                    item_location,
                    inventory_slot,
                },
                VendorPacket0104::ItemDeleteSuccess(reply),
            ) => {
                if reply.item_location != item_location || reply.slot_num != inventory_slot {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Delete,
                        detail: "location or slot differs",
                    });
                }
                inventory.apply_item_delete_success(reply)?;
                self.pending = None;
                Ok(VendorProductionEvent0104::DeleteAccepted)
            }
            (
                PendingVendorRequest0104::Disassemble { inventory_slot },
                VendorPacket0104::DisassembleSuccess(reply),
            ) => {
                if reply.new_item_slot != inventory_slot {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Disassemble,
                        detail: "new item slot differs",
                    });
                }
                inventory.apply_disassemble_success(reply)?;
                self.pending = None;
                Ok(VendorProductionEvent0104::DisassembleAccepted)
            }
            (
                PendingVendorRequest0104::Disassemble { inventory_slot },
                VendorPacket0104::DisassembleFailure(reply),
            ) => {
                if reply.item_slot != inventory_slot {
                    return Err(VendorProductionError0104::ReplyIdentityMismatch {
                        operation: VendorOperation0104::Disassemble,
                        detail: "failure slot differs",
                    });
                }
                self.pending = None;
                Ok(VendorProductionEvent0104::Failed {
                    operation: VendorOperation0104::Disassemble,
                    error_code: reply.error_code,
                    ui_failure: None,
                })
            }
            _ => unreachable!("reply operation was checked before the exhaustive pair"),
        }
    }

    fn reindex_recent(&mut self) {
        for (index, entry) in self.recent_entries.iter_mut().enumerate() {
            entry.source_slot_id = index;
        }
    }

    fn ensure_catalog_item(
        &self,
        item: ffone_protocol::ItemBase0104,
        operation: VendorOperation0104,
    ) -> Result<(), VendorProductionError0104> {
        if !self.catalog_entries.iter().any(|entry| {
            entry.item.item_type == item.item_type && entry.item.item_id == item.item_id
        }) {
            return Err(VendorProductionError0104::IntentStateMismatch {
                operation,
                detail: "item is absent from the accepted vendor table",
            });
        }
        Ok(())
    }
}

fn wire_slot(value: usize) -> Result<i32, VendorProductionError0104> {
    i32::try_from(value).map_err(|_| VendorProductionError0104::SlotOutOfRange { value })
}

const fn intent_operation(intent: VendorIntent0104) -> VendorOperation0104 {
    match intent {
        VendorIntent0104::Buy(_) | VendorIntent0104::BuyGeneral(_) => VendorOperation0104::Buy,
        VendorIntent0104::Battery(_) => VendorOperation0104::BatteryBuy,
        VendorIntent0104::Restore(_) => VendorOperation0104::Restore,
        VendorIntent0104::Sell(_) => VendorOperation0104::Sell,
        VendorIntent0104::Delete(_) => VendorOperation0104::Delete,
        VendorIntent0104::Disassemble(_) => VendorOperation0104::Disassemble,
    }
}

fn ensure_identity(
    session: VendorSession0104,
    identity: crate::vendor_ui::VendorRequestIdentity0104,
    operation: VendorOperation0104,
) -> Result<(), VendorProductionError0104> {
    if identity != session.request_identity() {
        return Err(VendorProductionError0104::IntentIdentityMismatch { operation });
    }
    Ok(())
}

fn ensure_empty_destination(
    inventory: &InventoryRuntime0104,
    slot: usize,
    operation: VendorOperation0104,
) -> Result<(), VendorProductionError0104> {
    let item = inventory.inventory().get(slot).copied().ok_or(
        VendorProductionError0104::IntentStateMismatch {
            operation,
            detail: "destination slot is outside inventory",
        },
    )?;
    if !InventoryRuntime0104::item_is_empty(item) {
        return Err(VendorProductionError0104::IntentStateMismatch {
            operation,
            detail: "destination slot is no longer empty",
        });
    }
    Ok(())
}

fn ensure_source_item(
    inventory: &InventoryRuntime0104,
    slot: usize,
    expected: ffone_protocol::ItemBase0104,
    operation: VendorOperation0104,
) -> Result<(), VendorProductionError0104> {
    let actual = inventory.inventory().get(slot).copied().ok_or(
        VendorProductionError0104::IntentStateMismatch {
            operation,
            detail: "source slot is outside inventory",
        },
    )?;
    if InventoryRuntime0104::item_is_empty(actual) || actual != expected {
        return Err(VendorProductionError0104::IntentStateMismatch {
            operation,
            detail: "source slot no longer owns the requested item",
        });
    }
    Ok(())
}

fn wire_list_id(value: i32) -> Result<i8, VendorProductionError0104> {
    i8::try_from(value).map_err(|_| VendorProductionError0104::ListIdOutOfRange { value })
}

const fn reply_operation(packet: VendorPacket0104) -> VendorOperation0104 {
    match packet {
        VendorPacket0104::StartSuccess(_) | VendorPacket0104::StartFailure(_) => {
            VendorOperation0104::Start
        }
        VendorPacket0104::TableSuccess(_) | VendorPacket0104::TableFailure(_) => {
            VendorOperation0104::TableUpdate
        }
        VendorPacket0104::BuySuccess(_) | VendorPacket0104::BuyFailure(_) => {
            VendorOperation0104::Buy
        }
        VendorPacket0104::BatterySuccess(_) | VendorPacket0104::BatteryFailure(_) => {
            VendorOperation0104::BatteryBuy
        }
        VendorPacket0104::SellSuccess(_) | VendorPacket0104::SellFailure(_) => {
            VendorOperation0104::Sell
        }
        VendorPacket0104::RestoreSuccess(_) | VendorPacket0104::RestoreFailure(_) => {
            VendorOperation0104::Restore
        }
        VendorPacket0104::ItemDeleteSuccess(_) => VendorOperation0104::Delete,
        VendorPacket0104::DisassembleSuccess(_) | VendorPacket0104::DisassembleFailure(_) => {
            VendorOperation0104::Disassemble
        }
    }
}

#[cfg(test)]
mod tests;
