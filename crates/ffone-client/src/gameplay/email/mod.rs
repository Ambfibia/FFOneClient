//! Production lifecycle, projection, and strict beta-20100104 transport for
//! clean Retrobution `EmailMode`.
//!
//! The clean client owns the IMGUI and local event-bus behavior in
//! [`crate::email_ui`]. This module owns the production lease around that
//! passive UI, projects authoritative buddy/inventory feeds, validates every
//! queued action before it reaches the socket, and preserves non-email frames
//! losslessly for the next gameplay owner. Inventory and Taros never change
//! from a prediction: only correlated 0104 replies produce explicit commits.

use std::{array, error::Error, fmt};

use bevy::prelude::Resource;
use ffone_protocol::{
    DecodedFrame, ItemBase0104, RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104,
};

use crate::buddy_ui::{BUDDY_MAX_SLOTS, BuddyUiModel};
use crate::email_ui::{
    EMAIL_ATTACHMENT_COUNT, EMAIL_DELETE_BATCH_COUNT, EMAIL_INVENTORY_SLOT_COUNT,
    EMAIL_REP_DELETE_FAILURE_ID, EMAIL_REP_DELETE_FAILURE_SIZE, EMAIL_REP_DELETE_SUCCESS_ID,
    EMAIL_REP_DELETE_SUCCESS_SIZE, EMAIL_REP_NEW_ID, EMAIL_REP_NEW_SIZE,
    EMAIL_REP_PAGE_LIST_FAILURE_ID, EMAIL_REP_PAGE_LIST_FAILURE_SIZE,
    EMAIL_REP_PAGE_LIST_SUCCESS_ID, EMAIL_REP_PAGE_LIST_SUCCESS_SIZE, EMAIL_REP_READ_FAILURE_ID,
    EMAIL_REP_READ_FAILURE_SIZE, EMAIL_REP_READ_SUCCESS_ID, EMAIL_REP_READ_SUCCESS_SIZE,
    EMAIL_REP_RECEIVE_ALL_FAILURE_ID, EMAIL_REP_RECEIVE_ALL_FAILURE_SIZE,
    EMAIL_REP_RECEIVE_ALL_SUCCESS_ID, EMAIL_REP_RECEIVE_ALL_SUCCESS_SIZE,
    EMAIL_REP_RECEIVE_CASH_FAILURE_ID, EMAIL_REP_RECEIVE_CASH_FAILURE_SIZE,
    EMAIL_REP_RECEIVE_CASH_SUCCESS_ID, EMAIL_REP_RECEIVE_CASH_SUCCESS_SIZE,
    EMAIL_REP_RECEIVE_ITEM_FAILURE_ID, EMAIL_REP_RECEIVE_ITEM_FAILURE_SIZE,
    EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID, EMAIL_REP_RECEIVE_ITEM_SUCCESS_SIZE,
    EMAIL_REP_SEND_FAILURE_ID, EMAIL_REP_SEND_FAILURE_SIZE, EMAIL_REP_SEND_SUCCESS_ID,
    EMAIL_REP_SEND_SUCCESS_SIZE, EMAIL_UI_GAME_MODE_VALUE, EmailBuddy, EmailCloseSource,
    EmailFolder, EmailGuideMessage, EmailInventorySlotView, EmailNetworkInbox0104,
    EmailNetworkRuntime0104, EmailOutgoingItem, EmailReadMessage, EmailReply, EmailRequest,
    EmailScreen, EmailSummary, EmailSystemTime, EmailTransportOutbox, EmailUiAction,
    EmailUiAudioCue, EmailUiAudioOutbox, EmailUiModel, EmailUiOutbox, EmailWireItem,
    apply_email_reply, open_email_ui, request_email_close, resolve_email_computress_gate,
    resolve_email_escape_gate, switch_email_folder,
};
use crate::inventory_runtime::{INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104};

#[cfg(test)]
mod tests;

mod constants;
mod codec;
mod types_email_pending0104;
mod types_email_production_runtime0104;
mod state;
mod assets;
mod projects;
mod operations;
mod validation;
mod commands;
mod output;
mod input;

use constants::{
    EMAIL_ITEM_BASE_SIZE, EMAIL_OUTGOING_ITEM_SIZE, EMAIL_SUMMARY_SIZE,
    EMAIL_SUMMARY_FIRST_OFFSET, EMAIL_SUBJECT_UNITS, EMAIL_CONTENT_UNITS,
    EMAIL_FIRST_NAME_UNITS, EMAIL_LAST_NAME_UNITS
};
pub use constants::EMAIL_BUDDY_MAX_COUNT_0104;
pub use codec::{
    EmailWirePacket0104, EmailEncodeError0104, EmailDecodeError0104,
    EMAIL_NORMAL_EXIT_SENDS_PACKET_0104, EmailFrameDisposition0104, encode_email_request_0104,
    decode_email_reply_0104
};
use codec::{write_wire_item, read_wire_item};
pub use types_email_pending0104::{
    EmailPending0104, EmailTransportRuntime0104, EmailItemFeaturePolicy0104,
    EmailPlayerAuthority0104, EmailOpenContext0104, EmailItemCatalog0104,
    EmailAttachmentRejection0104, EmailAuthoritySource0104, EmailAuthoritativeCommit0104,
    EmailProductionOutput0104, EmailProductionError0104, EmailProductionSession0104,
    EmailProductionRuntime0104
};
pub use state::{
    EmailRuntimeDelivery0104, EmailRuntimeError0104, EMAIL_ACTIVE_INVENTORY_TAB_0104,
    EMAIL_INVENTORY_MAIL_MODE_OPEN_0104, EMAIL_INVENTORY_MAIL_MODE_CLOSED_0104,
    EmailModeLease0104, EmailInventoryAuthority0104, EmailInventoryWrite0104,
    email_inventory_authority_0104
};
pub use assets::EmailItemCatalogMetadata0104;
use assets::is_safe_email_icon_path;
pub use projects::{project_email_buddies_0104, project_email_inventory_0104};
pub use operations::{
    email_buddies_from_buddy_ui_0104, email_item_from_base, item_base_from_email
};
use validation::{
    validate_player_authority, validate_staged_attachments_unchanged, validate_request_reachable,
    validate_reply_against_request, require_non_negative_authoritative_taros, require_size
};
#[cfg(test)]
use validation::validate_attachment_eligibility;
use commands::{remove_send_commit_action, request_unreachable};
use output::{write_marshaled_utf16, write_outgoing_item, write_i16, write_i32, write_i64};
use input::{
    read_fixed_utf16, read_outgoing_item, read_email_indices, read_summary, read_i16, read_i32,
    read_i64
};
