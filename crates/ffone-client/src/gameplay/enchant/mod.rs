//! Production boundary for clean Retrobution `eGameMode.Enchant` (29).
//!
//! The clean authority is `cnEnchantMode`/`cnGuiEnchant` from the primary
//! Retrobution `Assembly - CSharp.dll`.  That assembly was inspected offline
//! with the repository FFSpy 7.2 decompiler; it is never a runtime dependency.
//!
//! `P_CL2FE_REQ_PC_ITEM_ENCHANT` (`0x1300_00a4`, 20 bytes) exists in the clean
//! client ABI, but the pinned unmodified OpenFusion shard does not register a
//! handler for it.  This controller therefore fails closed at the confirmation
//! boundary.  It never invents a success, spends Taros, or edits inventory.
//! The implementation remains strict enough to accept the packet in the future
//! only if the protocol registry itself gains the exact handler and body ABI.
//!
//! Selection is presentation-only.  Inventory, Taros, and battery values stay
//! immutable until a correlated authoritative reply produces an explicit
//! [`EnchantAuthoritativeCommit0104`] for the gameplay owner to apply atomically.

use std::{array, error::Error, fmt};

use bevy::prelude::Resource;
use ffone_protocol::{
    DecodedFrame, ItemBase0104, PayloadError, PcDisassembleItemFailure0104,
    PcDisassembleItemSuccess0104, PcItemDeleteRequest0104, PcItemDeleteSuccess0104,
    RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104, WirePayload,
};

use crate::{
    enchant_ui::{
        ENCHANT_DELETE_REQUEST_PACKET_ID_0104, ENCHANT_DELETE_SUCCESS_PACKET_ID_0104,
        ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104, ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104,
        ENCHANT_FAILURE_PACKET_ID_0104, ENCHANT_FAILURE_PACKET_SIZE_0104, ENCHANT_GAME_MODE_0104,
        ENCHANT_MESSAGE_DELETE_ITEM_0104, ENCHANT_NPC_TYPE_0104, ENCHANT_REQUEST_PACKET_ID_0104,
        ENCHANT_REQUEST_PACKET_SIZE_0104, ENCHANT_SUCCESS_PACKET_ID_0104,
        ENCHANT_SUCCESS_PACKET_SIZE_0104, EnchantAttachmentSlot0104, EnchantAudioIntent0104,
        EnchantAuthoritativeReceipt0104, EnchantCameraIntent0104, EnchantExternalGates0104,
        EnchantFailure0104, EnchantIntent0104, EnchantInventorySlotProjection0104,
        EnchantItemPresentation0104, EnchantLifecycleIntent0104, EnchantModeModel0104,
        EnchantModeProjection0104, EnchantModelError0104, EnchantPhase0104, EnchantPopupIntent0104,
        EnchantRedeemWire0104, EnchantReplyDisposition0104, EnchantRequest0104,
        EnchantSelectableItem0104, EnchantSelectionIntent0104, EnchantSuccess0104,
        EnchantSupportPresentation0104, EnchantSystemCallback0104, EnchantUiCommand0104,
        EnchantUiOutbox0104,
    },
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104,
    },
};

#[cfg(test)]
mod tests;

mod constants;
mod state;
mod types_enchant_production_error0104;
mod types_enchant_production_runtime0104;
mod commands;
mod codec;
mod operations;
mod validation;
mod models;
mod output;

pub use constants::{ENCHANT_NPC_BUTTON_TYPE_0104, ENCHANT_GO_TO_STUFF_FIRST_USE_CONDITION_0104};
pub use state::{
    ENCHANT_ACTIVE_INVENTORY_TAB_0104, ENCHANT_MY_STUFF_GAME_MODE_0104,
    EnchantInventoryWrite0104, EnchantRuntimeModal0104
};
use state::{ensure_runtime_input_available, begin_inventory_drag, apply_selection_intent};
pub use types_enchant_production_error0104::{
    EnchantPlayerAuthority0104, EnchantServiceSource0104, EnchantOpenContext0104,
    EnchantAuthoritySnapshot0104, EnchantPresentationCatalog0104, EnchantOperation0104,
    EnchantAuthoritativeCommit0104, EnchantModalChoice0104, EnchantInputEffect0104,
    EnchantShellEffect0104, EnchantProductionOutput0104, EnchantGameplayFrame0104,
    EnchantProductionError0104, EnchantProductionSession0104, EnchantProductionRuntime0104
};
use types_enchant_production_error0104::{
    EnchantDrag0104, EnchantReservation0104, PendingEnchant0104, PendingDelete0104,
    PendingRequest0104
};
pub use commands::EnchantReplyPacket0104;
use commands::{
    ensure_no_pending_request, set_single_request, commit_enchant_reply, commit_delete_reply
};
pub use codec::{
    encode_enchant_request_0104, encode_enchant_delete_request_0104,
    encode_enchant_redeem_request_0104, decode_enchant_gameplay_frame_0104
};
pub use operations::prove_enchant_transport_registered_0104;
use operations::{
    snapshot_from_authority, open_delete_modal, expected_enchant_receipt,
    one_authoritative_receipt
};
use validation::{
    validate_open_context, validate_player_authority, validate_request_against_reservations,
    validate_enchant_success_identity, validate_enchant_failure_identity
};
use models::drain_model_intents;
use output::stage_enchant_write;
