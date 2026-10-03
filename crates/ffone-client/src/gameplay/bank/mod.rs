//! Production protocol/runtime boundary for clean Retrobution `BankMode`.
//!
//! Clean authority (`retrobution-20260613`):
//! - `NpcIconMode.StartBank` creates GameMode 11 linked to MainGame 5.
//! - `cnBank.ReceiveBankStart` selects inventory tab 0, unlocks the cursor,
//!   shows all three panels immediately, and sends `PC_BANK_OPEN`.
//! - only `iExtraBank == 1` unlocks all 200 bank slots; every other value
//!   leaves slots 60..199 visible but locked.
//! - `UserSlot.ItemMove` applies the two `sItemBase` values carried by
//!   `PC_ITEM_MOVE_SUCC` as authoritative post-state.
//! - normal `cnBank.BankOut` is local and never sends `PC_BANK_CLOSE`, even
//!   though the published 0104 protocol contains close request/reply ABIs.
//!
//! This owner therefore keeps exactly one open/move request in flight, rejects
//! malformed or identity-mismatched replies without changing state, and emits
//! a fully validated atomic move commit. It never speculatively swaps items.
//!
//! # Main integration boundary
//!
//! `main.rs` must register this resource, drain `BankUiOutbox0104`, translate
//! [`BankOutboundRequest0104`] to `NetworkCommand::OpenBank`/`MoveItem`, and
//! route the bank/item-move classifiers here before generic inventory mutation.
//! For an accepted move it must apply every inventory write from
//! [`BankAuthoritativeMoveCommit0104`] to the global inventory authority and
//! then rebuild `BankModeProjection0104` from `snapshot_after()` in the same
//! system transaction. The shell also owns GameMode 11/5 visibility, special
//! state flag 16, cursor restoration, camera release, UI-mode audio, and the
//! access-required system popup. No legacy Unity file is a runtime dependency.

use std::{error::Error, fmt};

use bevy::prelude::Resource;
use ffone_net::{BankGameplayFrame0104, InventoryGameplayFrame0104};
use ffone_protocol::{
    BANK_SLOT_COUNT_0104, DecodedFrame, InventoryPacket0104, ItemBase0104, ItemMoveRequest0104,
    ItemMoveSuccessPacket0104, PayloadError, PcBankOpenRequest0104, PcBankReply0104,
};

use crate::{
    bank_ui::{
        BANK_FULL_ACCESS_VALUE, BANK_HALF_ACCESS_SLOT_COUNT, BankAuthoritativeSnapshot0104,
        BankAuthorityMutationError0104, BankAuthorityMutationReceipt0104, BankSlotLocation0104,
        BankSlotRef0104, BankSlotRefError0104, BankUiCommand0104,
    },
    inventory_runtime::{INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104},
};

pub const BANK_GAME_MODE_0104: u8 = 11;
pub const BANK_LINKED_MAIN_GAME_MODE_0104: u8 = 5;
pub const BANK_GUI_MODE_0104: u8 = 1;
pub const BANK_ACTIVE_INVENTORY_TAB_0104: i32 = 0;
pub const BANK_INVENTORY_TAB_SWITCHING_ENABLED_0104: bool = false;
pub const BANK_SPECIAL_STATE_FLAG_0104: i32 = 16;
pub const BANK_ACCESS_REQUIRED_ERROR_CODE_0104: i32 = 2;
pub const BANK_NORMAL_EXIT_SENDS_CLOSE_PACKET_0104: bool = false;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankOpenIdentity0104 {
    pub owner_pc_id: i32,
    pub npc_id: i32,
}

impl From<PcBankOpenRequest0104> for BankOpenIdentity0104 {
    fn from(value: PcBankOpenRequest0104) -> Self {
        Self {
            owner_pc_id: value.pc_id,
            npc_id: value.npc_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BankAccessTier0104 {
    /// `Panel_BankScript.Half=true`: slots 0..59 are usable and 60..199 stay
    /// visible with the locked-slot surface.
    BaseSixty,
    /// `Panel_BankScript.Half=false`: all 200 slots are usable.
    FullTwoHundred,
}

impl BankAccessTier0104 {
    #[must_use]
    pub const fn from_extra_bank(raw_extra_bank: i32) -> Self {
        if raw_extra_bank == BANK_FULL_ACCESS_VALUE {
            Self::FullTwoHundred
        } else {
            Self::BaseSixty
        }
    }

    #[must_use]
    pub const fn accessible_slots(self) -> usize {
        match self {
            Self::BaseSixty => BANK_HALF_ACCESS_SLOT_COUNT,
            Self::FullTwoHundred => BANK_SLOT_COUNT_0104,
        }
    }

    #[must_use]
    pub const fn slot_locked(self, slot: BankSlotRef0104) -> bool {
        matches!(self, Self::BaseSixty)
            && matches!(slot.location(), BankSlotLocation0104::Bank)
            && slot.index() >= BANK_HALF_ACCESS_SLOT_COUNT
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankTabPolicy0104 {
    pub active_inventory_tab: i32,
    pub tab_switching_enabled: bool,
}

impl Default for BankTabPolicy0104 {
    fn default() -> Self {
        Self {
            active_inventory_tab: BANK_ACTIVE_INVENTORY_TAB_0104,
            tab_switching_enabled: BANK_INVENTORY_TAB_SWITCHING_ENABLED_0104,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankModeLease0104 {
    pub game_mode: u8,
    pub linked_game_mode: u8,
    pub gui_mode: u8,
    pub hides_main_menu: bool,
    pub blocks_gameplay_input: bool,
    pub cursor_locked_during_mode: bool,
    pub special_state_flag: i32,
    pub tab_policy: BankTabPolicy0104,
    pub normal_exit_sends_bank_close_packet: bool,
}

impl Default for BankModeLease0104 {
    fn default() -> Self {
        Self {
            game_mode: BANK_GAME_MODE_0104,
            linked_game_mode: BANK_LINKED_MAIN_GAME_MODE_0104,
            gui_mode: BANK_GUI_MODE_0104,
            hides_main_menu: true,
            blocks_gameplay_input: true,
            cursor_locked_during_mode: false,
            special_state_flag: BANK_SPECIAL_STATE_FLAG_0104,
            tab_policy: BankTabPolicy0104::default(),
            normal_exit_sends_bank_close_packet: BANK_NORMAL_EXIT_SENDS_CLOSE_PACKET_0104,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankSession0104 {
    source: BankOpenIdentity0104,
    snapshot: BankAuthoritativeSnapshot0104,
}

impl BankSession0104 {
    #[must_use]
    pub const fn source(&self) -> BankOpenIdentity0104 {
        self.source
    }

    #[must_use]
    pub const fn snapshot(&self) -> &BankAuthoritativeSnapshot0104 {
        &self.snapshot
    }

    #[must_use]
    pub const fn raw_extra_bank(&self) -> i32 {
        self.snapshot.extra_bank()
    }

    #[must_use]
    pub const fn access_tier(&self) -> BankAccessTier0104 {
        BankAccessTier0104::from_extra_bank(self.raw_extra_bank())
    }

    #[must_use]
    pub const fn tab_policy(&self) -> BankTabPolicy0104 {
        BankTabPolicy0104 {
            active_inventory_tab: BANK_ACTIVE_INVENTORY_TAB_0104,
            tab_switching_enabled: BANK_INVENTORY_TAB_SWITCHING_ENABLED_0104,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BankOperation0104 {
    Open,
    ItemMove,
    CloseReply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingBankRequest0104 {
    Open(BankOpenIdentity0104),
    ItemMove(ItemMoveRequest0104),
}

impl PendingBankRequest0104 {
    const fn operation(self) -> BankOperation0104 {
        match self {
            Self::Open(_) => BankOperation0104::Open,
            Self::ItemMove(_) => BankOperation0104::ItemMove,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankOutboundRequest0104 {
    Open(PcBankOpenRequest0104),
    ItemMove(ItemMoveRequest0104),
}

impl BankOutboundRequest0104 {
    #[must_use]
    pub const fn operation(self) -> BankOperation0104 {
        match self {
            Self::Open(_) => BankOperation0104::Open,
            Self::ItemMove(_) => BankOperation0104::ItemMove,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPostStateWrite0104 {
    pub slot: BankSlotRef0104,
    pub item: ItemBase0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankAuthoritativeMoveCommit0104 {
    request: ItemMoveRequest0104,
    reply: ItemMoveSuccessPacket0104,
    receipt: BankAuthorityMutationReceipt0104,
    post_state_writes: [Option<BankPostStateWrite0104>; 2],
    snapshot_after: BankAuthoritativeSnapshot0104,
}

impl BankAuthoritativeMoveCommit0104 {
    #[must_use]
    pub const fn request(&self) -> ItemMoveRequest0104 {
        self.request
    }

    #[must_use]
    pub const fn reply(&self) -> ItemMoveSuccessPacket0104 {
        self.reply
    }

    #[must_use]
    pub const fn receipt(&self) -> BankAuthorityMutationReceipt0104 {
        self.receipt
    }

    #[must_use]
    pub const fn snapshot_after(&self) -> &BankAuthoritativeSnapshot0104 {
        &self.snapshot_after
    }

    pub fn post_state_writes(&self) -> impl Iterator<Item = BankPostStateWrite0104> + '_ {
        self.post_state_writes.iter().flatten().copied()
    }

    pub fn inventory_writes(&self) -> impl Iterator<Item = BankPostStateWrite0104> + '_ {
        self.post_state_writes()
            .filter(|write| write.slot.location() == BankSlotLocation0104::Inventory)
    }

    pub fn bank_writes(&self) -> impl Iterator<Item = BankPostStateWrite0104> + '_ {
        self.post_state_writes()
            .filter(|write| write.slot.location() == BankSlotLocation0104::Bank)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankProductionEvent0104 {
    OpenAccepted {
        source: BankOpenIdentity0104,
        raw_extra_bank: i32,
        access_tier: BankAccessTier0104,
        tab_policy: BankTabPolicy0104,
    },
    OpenFailed {
        source: BankOpenIdentity0104,
        error_code: i32,
        show_access_required_popup: bool,
        close_immediately: bool,
    },
    ItemMoveCommitted(BankAuthoritativeMoveCommit0104),
    ClosedLocally {
        source: BankOpenIdentity0104,
    },
    ClosedByServerSuccess {
        source: BankOpenIdentity0104,
        reply_pc_id: i32,
    },
    ClosedByServerFailure {
        source: BankOpenIdentity0104,
        error_code: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankUiCommandDisposition0104 {
    Send(BankOutboundRequest0104),
    Local(BankProductionEvent0104),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankProductionError0104 {
    AlreadyActive,
    NotActive,
    NoAcceptedSession,
    RequestPending {
        operation: BankOperation0104,
    },
    NoPendingRequest,
    UnexpectedReply {
        pending: Option<BankOperation0104>,
        reply: BankOperation0104,
    },
    InventoryOwnerMismatch {
        expected_pc_id: i32,
        inventory_pc_id: i32,
    },
    OpenBankItemMalformed {
        bank_slot: usize,
        item_type: i16,
        item_id: i16,
    },
    InvalidMoveSlot(BankSlotRefError0104),
    SameSlotNoOp {
        slot: BankSlotRef0104,
    },
    LockedBankSlot {
        slot: BankSlotRef0104,
    },
    EmptyMoveSource {
        slot: BankSlotRef0104,
    },
    MoveReplyEndpointMismatch {
        request: ItemMoveRequest0104,
        reply: ItemMoveSuccessPacket0104,
    },
    CloseReplyOwnerMismatch {
        expected_pc_id: i32,
        reply_pc_id: i32,
    },
    BankAuthority(BankAuthorityMutationError0104),
    MalformedBankFrame {
        frame: DecodedFrame,
        error: PayloadError,
    },
    MalformedItemMoveFrame {
        frame: DecodedFrame,
        error: PayloadError,
    },
}

impl fmt::Display for BankProductionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyActive => f.write_str("BankMode is already active or opening"),
            Self::NotActive => f.write_str("BankMode is not active"),
            Self::NoAcceptedSession => f.write_str("BankMode has no accepted bank snapshot"),
            Self::RequestPending { operation } => {
                write!(f, "BankMode request {operation:?} is still pending")
            }
            Self::NoPendingRequest => f.write_str("BankMode reply has no pending request"),
            Self::UnexpectedReply { pending, reply } => write!(
                f,
                "BankMode received {reply:?} while pending operation is {pending:?}"
            ),
            Self::InventoryOwnerMismatch {
                expected_pc_id,
                inventory_pc_id,
            } => write!(
                f,
                "BankMode belongs to PC {expected_pc_id}, inventory belongs to PC {inventory_pc_id}"
            ),
            Self::OpenBankItemMalformed {
                bank_slot,
                item_type,
                item_id,
            } => write!(
                f,
                "bank-open slot {bank_slot} has malformed type={item_type} id={item_id}"
            ),
            Self::InvalidMoveSlot(error) => write!(f, "BankMode move slot is invalid: {error}"),
            Self::SameSlotNoOp { slot } => write!(
                f,
                "BankMode move is a no-op at {:?} slot {}",
                slot.location(),
                slot.index()
            ),
            Self::LockedBankSlot { slot } => {
                write!(f, "BankMode bank slot {} is locked", slot.index())
            }
            Self::EmptyMoveSource { slot } => write!(
                f,
                "BankMode {:?} slot {} has no source item",
                slot.location(),
                slot.index()
            ),
            Self::MoveReplyEndpointMismatch { request, reply } => write!(
                f,
                "BankMode move reply endpoints ({}, {}, {}, {}) do not match request ({}, {}, {}, {})",
                reply.from_location,
                reply.from_slot_num,
                reply.to_location,
                reply.to_slot_num,
                request.from_location,
                request.from_slot_num,
                request.to_location,
                request.to_slot_num
            ),
            Self::CloseReplyOwnerMismatch {
                expected_pc_id,
                reply_pc_id,
            } => write!(
                f,
                "bank-close reply belongs to PC {reply_pc_id}, expected {expected_pc_id}"
            ),
            Self::BankAuthority(error) => write!(f, "BankMode post-state rejected: {error}"),
            Self::MalformedBankFrame { frame, error } => write!(
                f,
                "BankMode packet {:#010x} is malformed: {error}",
                frame.packet_type
            ),
            Self::MalformedItemMoveFrame { frame, error } => write!(
                f,
                "BankMode item-move packet {:#010x} is malformed: {error}",
                frame.packet_type
            ),
        }
    }
}

impl Error for BankProductionError0104 {}

impl From<BankSlotRefError0104> for BankProductionError0104 {
    fn from(value: BankSlotRefError0104) -> Self {
        Self::InvalidMoveSlot(value)
    }
}

impl From<BankAuthorityMutationError0104> for BankProductionError0104 {
    fn from(value: BankAuthorityMutationError0104) -> Self {
        Self::BankAuthority(value)
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct BankProductionRuntime0104 {
    opening: Option<BankOpenIdentity0104>,
    session: Option<BankSession0104>,
    pending: Option<PendingBankRequest0104>,
}

impl BankProductionRuntime0104 {
    #[must_use]
    pub const fn opening(&self) -> Option<BankOpenIdentity0104> {
        self.opening
    }

    #[must_use]
    pub const fn session(&self) -> Option<&BankSession0104> {
        self.session.as_ref()
    }

    #[must_use]
    pub const fn mode_lease(&self) -> Option<BankModeLease0104> {
        if self.opening.is_some() || self.session.is_some() {
            Some(BankModeLease0104 {
                game_mode: BANK_GAME_MODE_0104,
                linked_game_mode: BANK_LINKED_MAIN_GAME_MODE_0104,
                gui_mode: BANK_GUI_MODE_0104,
                hides_main_menu: true,
                blocks_gameplay_input: true,
                cursor_locked_during_mode: false,
                special_state_flag: BANK_SPECIAL_STATE_FLAG_0104,
                tab_policy: BankTabPolicy0104 {
                    active_inventory_tab: BANK_ACTIVE_INVENTORY_TAB_0104,
                    tab_switching_enabled: BANK_INVENTORY_TAB_SWITCHING_ENABLED_0104,
                },
                normal_exit_sends_bank_close_packet: BANK_NORMAL_EXIT_SENDS_CLOSE_PACKET_0104,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn modal_active(&self) -> bool {
        self.opening.is_some() || self.session.is_some()
    }

    #[must_use]
    pub const fn request_pending(&self) -> bool {
        self.pending.is_some()
    }

    #[must_use]
    pub const fn pending_operation(&self) -> Option<BankOperation0104> {
        match self.pending {
            Some(pending) => Some(pending.operation()),
            None => None,
        }
    }

    /// Exact request identity used by the shared item-move reply arbiter.
    /// This is intentionally read-only: another modal may inspect ownership,
    /// but only BankMode may commit or release the request.
    #[must_use]
    pub const fn pending_item_move_request(&self) -> Option<ItemMoveRequest0104> {
        match self.pending {
            Some(PendingBankRequest0104::ItemMove(request)) => Some(request),
            Some(PendingBankRequest0104::Open(_)) | None => None,
        }
    }

    #[must_use]
    pub const fn owns_item_move_reply(&self, reply: ItemMoveSuccessPacket0104) -> bool {
        match self.pending_item_move_request() {
            Some(request) => move_endpoints_match(request, reply),
            None => false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn begin_open(
        &mut self,
        request: PcBankOpenRequest0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<BankOutboundRequest0104, BankProductionError0104> {
        if self.modal_active() || self.pending.is_some() {
            return Err(BankProductionError0104::AlreadyActive);
        }
        if inventory.owner_pc_id() != request.pc_id {
            return Err(BankProductionError0104::InventoryOwnerMismatch {
                expected_pc_id: request.pc_id,
                inventory_pc_id: inventory.owner_pc_id(),
            });
        }
        let source = BankOpenIdentity0104::from(request);
        self.opening = Some(source);
        self.pending = Some(PendingBankRequest0104::Open(source));
        Ok(BankOutboundRequest0104::Open(request))
    }

    pub fn begin_item_move(
        &mut self,
        request: ItemMoveRequest0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<BankOutboundRequest0104, BankProductionError0104> {
        if let Some(pending) = self.pending {
            return Err(BankProductionError0104::RequestPending {
                operation: pending.operation(),
            });
        }
        let session = self
            .session
            .as_ref()
            .ok_or(BankProductionError0104::NoAcceptedSession)?;
        if inventory.owner_pc_id() != session.source.owner_pc_id {
            return Err(BankProductionError0104::InventoryOwnerMismatch {
                expected_pc_id: session.source.owner_pc_id,
                inventory_pc_id: inventory.owner_pc_id(),
            });
        }

        let refreshed = snapshot_with_inventory(&session.snapshot, inventory)?;
        let from = BankSlotRef0104::from_wire(request.from_location, request.from_slot_num)?;
        let to = BankSlotRef0104::from_wire(request.to_location, request.to_slot_num)?;
        if from == to {
            return Err(BankProductionError0104::SameSlotNoOp { slot: from });
        }
        let access = BankAccessTier0104::from_extra_bank(refreshed.extra_bank());
        if access.slot_locked(from) {
            return Err(BankProductionError0104::LockedBankSlot { slot: from });
        }
        if access.slot_locked(to) {
            return Err(BankProductionError0104::LockedBankSlot { slot: to });
        }
        if InventoryRuntime0104::item_is_empty(refreshed.item_at(from)) {
            return Err(BankProductionError0104::EmptyMoveSource { slot: from });
        }

        self.session
            .as_mut()
            .expect("accepted session was validated")
            .snapshot = refreshed;
        self.pending = Some(PendingBankRequest0104::ItemMove(request));
        Ok(BankOutboundRequest0104::ItemMove(request))
    }

    pub fn apply_ui_command(
        &mut self,
        command: BankUiCommand0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<BankUiCommandDisposition0104, BankProductionError0104> {
        match command {
            BankUiCommand0104::Open(request) => self
                .begin_open(request, inventory)
                .map(BankUiCommandDisposition0104::Send),
            BankUiCommand0104::ItemMove(request) => self
                .begin_item_move(request, inventory)
                .map(BankUiCommandDisposition0104::Send),
            BankUiCommand0104::ExitMode => self
                .close_locally()
                .map(BankUiCommandDisposition0104::Local),
        }
    }

    pub fn close_locally(&mut self) -> Result<BankProductionEvent0104, BankProductionError0104> {
        if let Some(pending) = self.pending {
            return Err(BankProductionError0104::RequestPending {
                operation: pending.operation(),
            });
        }
        let source = self
            .active_source()
            .ok_or(BankProductionError0104::NotActive)?;
        self.reset();
        Ok(BankProductionEvent0104::ClosedLocally { source })
    }

    /// Clear the clean `bSend` equivalent after the socket rejected an
    /// outbound write. The caller decides whether an opening mode should exit.
    pub fn cancel_pending_transport(&mut self) -> Option<BankOperation0104> {
        self.pending.take().map(PendingBankRequest0104::operation)
    }

    /// Refresh general inventory/equipment authority without changing the
    /// accepted bank array or `iExtraBank`. Main should call this after another
    /// authoritative inventory mutation observed while BankMode remains open.
    pub fn refresh_inventory_authority(
        &mut self,
        inventory: &InventoryRuntime0104,
    ) -> Result<(), BankProductionError0104> {
        if let Some(pending) = self.pending {
            return Err(BankProductionError0104::RequestPending {
                operation: pending.operation(),
            });
        }
        let session = self
            .session
            .as_mut()
            .ok_or(BankProductionError0104::NoAcceptedSession)?;
        session.snapshot = snapshot_with_inventory(&session.snapshot, inventory)?;
        Ok(())
    }

    pub fn apply_bank_frame(
        &mut self,
        classified: BankGameplayFrame0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<Option<BankProductionEvent0104>, BankProductionError0104> {
        match classified {
            BankGameplayFrame0104::Passthrough(_) => Ok(None),
            BankGameplayFrame0104::Malformed { frame, error } => {
                Err(BankProductionError0104::MalformedBankFrame { frame, error })
            }
            BankGameplayFrame0104::Decoded { packet, .. } => {
                self.apply_bank_reply(packet, inventory).map(Some)
            }
        }
    }

    pub fn apply_inventory_frame(
        &mut self,
        classified: InventoryGameplayFrame0104,
    ) -> Result<Option<BankProductionEvent0104>, BankProductionError0104> {
        match classified {
            InventoryGameplayFrame0104::Passthrough(_) => Ok(None),
            InventoryGameplayFrame0104::Malformed { frame, error }
                if frame.packet_type == ffone_protocol::packet::P_FE2CL_PC_ITEM_MOVE_SUCC =>
            {
                Err(BankProductionError0104::MalformedItemMoveFrame { frame, error })
            }
            InventoryGameplayFrame0104::Malformed { .. } => Ok(None),
            InventoryGameplayFrame0104::Decoded {
                packet: InventoryPacket0104::EquipChange(_),
                ..
            } => Ok(None),
            InventoryGameplayFrame0104::Decoded {
                packet: InventoryPacket0104::ItemMoveSuccess(packet),
                ..
            } => self.apply_item_move_success(packet).map(Some),
        }
    }

    pub fn apply_bank_reply(
        &mut self,
        packet: PcBankReply0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<BankProductionEvent0104, BankProductionError0104> {
        match packet {
            PcBankReply0104::CloseSuccess(reply) => {
                let source = self
                    .active_source()
                    .ok_or(BankProductionError0104::NotActive)?;
                if reply.pc_id != source.owner_pc_id {
                    return Err(BankProductionError0104::CloseReplyOwnerMismatch {
                        expected_pc_id: source.owner_pc_id,
                        reply_pc_id: reply.pc_id,
                    });
                }
                self.reset();
                Ok(BankProductionEvent0104::ClosedByServerSuccess {
                    source,
                    reply_pc_id: reply.pc_id,
                })
            }
            PcBankReply0104::CloseFailure(reply) => {
                let source = self
                    .active_source()
                    .ok_or(BankProductionError0104::NotActive)?;
                self.reset();
                Ok(BankProductionEvent0104::ClosedByServerFailure {
                    source,
                    error_code: reply.error_code,
                })
            }
            PcBankReply0104::OpenSuccess(reply) => {
                let Some(PendingBankRequest0104::Open(source)) = self.pending else {
                    return Err(BankProductionError0104::UnexpectedReply {
                        pending: self.pending_operation(),
                        reply: BankOperation0104::Open,
                    });
                };
                if self.opening != Some(source) {
                    return Err(BankProductionError0104::UnexpectedReply {
                        pending: self.pending_operation(),
                        reply: BankOperation0104::Open,
                    });
                }
                validate_open_bank_items(&reply.bank_items)?;
                let snapshot = BankAuthoritativeSnapshot0104::from_open_success(
                    source.owner_pc_id,
                    source.npc_id,
                    &reply,
                    inventory,
                )
                .map_err(|_| BankProductionError0104::InventoryOwnerMismatch {
                    expected_pc_id: source.owner_pc_id,
                    inventory_pc_id: inventory.owner_pc_id(),
                })?;
                let raw_extra_bank = snapshot.extra_bank();
                let access_tier = BankAccessTier0104::from_extra_bank(raw_extra_bank);
                self.session = Some(BankSession0104 { source, snapshot });
                self.opening = None;
                self.pending = None;
                Ok(BankProductionEvent0104::OpenAccepted {
                    source,
                    raw_extra_bank,
                    access_tier,
                    tab_policy: BankTabPolicy0104::default(),
                })
            }
            PcBankReply0104::OpenFailure(reply) => {
                let Some(PendingBankRequest0104::Open(source)) = self.pending else {
                    return Err(BankProductionError0104::UnexpectedReply {
                        pending: self.pending_operation(),
                        reply: BankOperation0104::Open,
                    });
                };
                let show_access_required_popup =
                    reply.error_code == BANK_ACCESS_REQUIRED_ERROR_CODE_0104;
                let close_immediately = !show_access_required_popup;
                self.pending = None;
                if close_immediately {
                    self.opening = None;
                }
                Ok(BankProductionEvent0104::OpenFailed {
                    source,
                    error_code: reply.error_code,
                    show_access_required_popup,
                    close_immediately,
                })
            }
        }
    }

    pub fn apply_item_move_success(
        &mut self,
        reply: ItemMoveSuccessPacket0104,
    ) -> Result<BankProductionEvent0104, BankProductionError0104> {
        let Some(PendingBankRequest0104::ItemMove(request)) = self.pending else {
            return Err(match self.pending {
                None => BankProductionError0104::NoPendingRequest,
                Some(pending) => BankProductionError0104::UnexpectedReply {
                    pending: Some(pending.operation()),
                    reply: BankOperation0104::ItemMove,
                },
            });
        };
        if !move_endpoints_match(request, reply) {
            return Err(BankProductionError0104::MoveReplyEndpointMismatch { request, reply });
        }
        let session = self
            .session
            .as_ref()
            .ok_or(BankProductionError0104::NoAcceptedSession)?;

        // Validate both post-state values against a clone before committing
        // either the bank snapshot or the optional inventory writes.
        let mut snapshot_after = session.snapshot.clone();
        let receipt = snapshot_after.apply_item_move_success(reply)?;
        let post_state_writes = post_state_writes(reply, receipt);
        let commit = BankAuthoritativeMoveCommit0104 {
            request,
            reply,
            receipt,
            post_state_writes,
            snapshot_after,
        };

        self.session
            .as_mut()
            .expect("accepted session was checked")
            .snapshot = commit.snapshot_after.clone();
        self.pending = None;
        Ok(BankProductionEvent0104::ItemMoveCommitted(commit))
    }

    fn active_source(&self) -> Option<BankOpenIdentity0104> {
        self.session
            .as_ref()
            .map(BankSession0104::source)
            .or(self.opening)
    }
}

fn snapshot_with_inventory(
    snapshot: &BankAuthoritativeSnapshot0104,
    inventory: &InventoryRuntime0104,
) -> Result<BankAuthoritativeSnapshot0104, BankProductionError0104> {
    if inventory.owner_pc_id() != snapshot.owner_pc_id() {
        return Err(BankProductionError0104::InventoryOwnerMismatch {
            expected_pc_id: snapshot.owner_pc_id(),
            inventory_pc_id: inventory.owner_pc_id(),
        });
    }
    let open = ffone_protocol::PcBankOpenSuccess0104 {
        bank_items: *snapshot.bank(),
        extra_bank: snapshot.extra_bank(),
    };
    BankAuthoritativeSnapshot0104::from_open_success(
        snapshot.owner_pc_id(),
        snapshot.npc_id(),
        &open,
        inventory,
    )
    .map_err(|_| BankProductionError0104::InventoryOwnerMismatch {
        expected_pc_id: snapshot.owner_pc_id(),
        inventory_pc_id: inventory.owner_pc_id(),
    })
}

fn validate_open_bank_items(
    bank: &[ItemBase0104; BANK_SLOT_COUNT_0104],
) -> Result<(), BankProductionError0104> {
    for (bank_slot, item) in bank.iter().copied().enumerate() {
        if item.item_id > 0 && item.item_type < 0 {
            return Err(BankProductionError0104::OpenBankItemMalformed {
                bank_slot,
                item_type: item.item_type,
                item_id: item.item_id,
            });
        }
    }
    Ok(())
}

const fn move_endpoints_match(
    request: ItemMoveRequest0104,
    reply: ItemMoveSuccessPacket0104,
) -> bool {
    let same_order = request.from_location == reply.from_location
        && request.from_slot_num == reply.from_slot_num
        && request.to_location == reply.to_location
        && request.to_slot_num == reply.to_slot_num;
    let reversed = request.from_location == reply.to_location
        && request.from_slot_num == reply.to_slot_num
        && request.to_location == reply.from_location
        && request.to_slot_num == reply.from_slot_num;
    same_order || reversed
}

fn post_state_writes(
    reply: ItemMoveSuccessPacket0104,
    receipt: BankAuthorityMutationReceipt0104,
) -> [Option<BankPostStateWrite0104>; 2] {
    let first = BankPostStateWrite0104 {
        slot: receipt.primary,
        item: reply.from_slot_item,
    };
    let second = receipt.secondary.map(|slot| BankPostStateWrite0104 {
        slot,
        item: reply.to_slot_item,
    });
    [Some(first), second]
}

const _: [(); INVENTORY_SLOT_COUNT_0104] = [(); 50];
const _: [(); BANK_SLOT_COUNT_0104] = [(); 200];
