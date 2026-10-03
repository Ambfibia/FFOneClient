//! Production boundary for clean Retrobution `eGameMode.Combi` (25).
//!
//! The clean authority is `cnCombiMode` from the `Assembly - CSharp.dll`
//! owned by primary `main.unity3d` (SHA-256
//! `59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F`).
//! The managed assembly was inspected offline with the repository FFSpy 7.2
//! `ilspycmd -t cnCombiMode`; it is never a runtime dependency.
//!
//! This owner deliberately separates three kinds of state:
//! - the immutable-until-authority inventory/Taros snapshot used by
//!   [`CombiMachine0104`];
//! - the passive [`CombiUiState0104`] / [`CombiUiOutbox0104`] presentation
//!   boundary;
//! - the one registered 0104 request and its strictly correlated reply.
//!
//! Selection, confirmation, and the four-second waiting presentation never
//! mutate inventory or Taros. A valid reply produces an explicit atomic
//! [`CombiAuthoritativeCommit0104`]; the main gameplay owner applies its
//! writes to the global inventory and currency authorities.

use std::{error::Error, fmt};

use bevy::prelude::Resource;
use ffone_protocol::{
    DecodedFrame, ItemBase0104, PayloadError, RegisteredGameplayRequest0104,
    RegisteredGameplayRequestError0104,
};

use crate::{
    combi_ui::{
        COMBI_FAILURE_CASH_SLOT_1_OFFSET_0104, COMBI_FAILURE_CASH_SLOT_2_OFFSET_0104,
        COMBI_FAILURE_COSTUME_SLOT_OFFSET_0104, COMBI_FAILURE_ERROR_OFFSET_0104,
        COMBI_FAILURE_PACKET_ID_0104, COMBI_FAILURE_PACKET_SIZE_0104,
        COMBI_FAILURE_STAT_SLOT_OFFSET_0104, COMBI_FIRST_USE_CONDITION_0104, COMBI_GAME_MODE_0104,
        COMBI_NPC_BUTTON_TYPE_0104, COMBI_NPC_ID_0104, COMBI_NPC_TYPE_0104,
        COMBI_REQUEST_CASH_SLOT_1_OFFSET_0104, COMBI_REQUEST_CASH_SLOT_2_OFFSET_0104,
        COMBI_REQUEST_COSTUME_SLOT_OFFSET_0104, COMBI_REQUEST_PACKET_ID_0104,
        COMBI_REQUEST_PACKET_SIZE_0104, COMBI_REQUEST_STAT_SLOT_OFFSET_0104,
        COMBI_SUCCESS_CASH_SLOT_1_OFFSET_0104, COMBI_SUCCESS_CASH_SLOT_2_OFFSET_0104,
        COMBI_SUCCESS_FLAG_OFFSET_0104, COMBI_SUCCESS_NEW_ITEM_OFFSET_0104,
        COMBI_SUCCESS_NEW_ITEM_SLOT_OFFSET_0104, COMBI_SUCCESS_PACKET_ID_0104,
        COMBI_SUCCESS_PACKET_SIZE_0104, COMBI_SUCCESS_STAT_SLOT_OFFSET_0104,
        COMBI_SUCCESS_TAROS_OFFSET_0104, CombiAuthoritativeSnapshot0104, CombiAuthorityReceipt0104,
        CombiFailureReply0104, CombiItemCatalog0104, CombiMachine0104, CombiModalChoice0104,
        CombiModeProjection0104, CombiPhase0104, CombiProjectionError0104,
        CombiReceiptCommitError0104, CombiRecipeTable0104, CombiReplyError0104, CombiRequest0104,
        CombiSelectionChange0104, CombiSelectionSlot0104, CombiStateError0104,
        CombiSuccessReply0104, CombiSystemModal0104, CombiUiCommand0104, CombiUiOutbox0104,
        CombiUiState0104, empty_item_0104, project_combi_mode_0104,
    },
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104,
    },
};

mod state;
mod codec;
mod audio;
mod animation;
mod constants;
mod types_combi_production_error0104;
mod types_combi_production_runtime0104;
mod commands;
mod validation;
mod operations;
mod input;
mod output;

pub use state::{
    COMBI_ACTIVE_INVENTORY_TAB_0104, COMBI_MY_STUFF_GAME_MODE_0104, CombiModeLease0104,
    CombiInventoryWrite0104
};
pub use codec::{
    COMBI_NORMAL_EXIT_SENDS_PACKET_0104, encode_combi_request_0104,
    decode_combi_gameplay_frame_0104
};
pub use audio::COMBI_SUCCESS_SOUND_TRUE_NAME_0104;
pub use animation::COMBI_MAKING_ANIMATION_EVENT_0104;
pub use constants::COMBI_GO_TO_STUFF_FIRST_USE_CONDITION_0104;
pub use types_combi_production_error0104::{
    CombiPlayerAuthority0104, CombiServiceSource0104, CombiOpenContext0104,
    CombiNpcAnimation0104, CombiCloseReason0104, CombiSystemMessage0104, CombiShellEffect0104,
    CombiAuthoritativeCommit0104, CombiProductionOutput0104, CombiGameplayFrame0104,
    CombiProductionError0104, CombiProductionSession0104, CombiProductionRuntime0104
};
pub use commands::CombiReplyPacket0104;
use validation::{
    validate_open_context, validate_player_authority, validate_selected_items_unchanged,
    require_ready
};
use operations::{snapshot_from_authority, modal_message, authoritative_commit};
use input::read_i32;
use output::write_i32;
