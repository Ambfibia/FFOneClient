//! Clean-Retrobution `eGameMode.Pc2pc` authority, intent boundary, and UI.
//!
//! Primary authority:
//! - `cnTrade` owns the ordered `from/to/requester` identity and the
//!   offer/confirm/cancel packet state machine.
//! - `Panel_Trade` owns the 550x700 left shell, two five-slot offer rows,
//!   Taros offers, embedded chat, readiness labels, and one-second sine slide.
//! - `InventoryManagerScript`, `Panel_PCStuff`, and `Panel_Equip` own the
//!   shared 50-slot inventory plus nine-slot equipment strip on the right.
//! - `UserSlot.ReceiveTradeSucc` applies only final server-delivered items.
//!
//! The legacy client temporarily rewrote its inventory while an offer was
//! pending and restored cached values on cancel. FFOne keeps the server-owned
//! inventory immutable and projects the correlated `InvenItem` post-offer
//! values into a separate trade-availability overlay. Cancellation discards
//! that overlay. Only a correlated final success commits inventory and Taros.

use crate::{
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104,
    },
    localization::{LocalizationSet, LocalizedText},
    user_equip_ui::{
        USER_EQUIP_BACKDROP_HEIGHT, USER_EQUIP_BACKDROP_PATH, USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_CLOSE_PATH, USER_EQUIP_CLOSE_RECT, USER_EQUIP_COMBINED_BADGE_LEFT,
        USER_EQUIP_COMBINED_BADGE_SIZE, USER_EQUIP_COMBINED_BADGE_TOP, USER_EQUIP_COMBINED_PATH,
        USER_EQUIP_COUNT_FONT_SIZE, USER_EQUIP_COUNT_LABEL_LEFT, USER_EQUIP_COUNT_LABEL_TOP,
        USER_EQUIP_EQUIP_TITLE_PATH, USER_EQUIP_EQUIPMENT_CONTENT_RECT,
        USER_EQUIP_EQUIPMENT_SLOT_SIZE, USER_EQUIP_EQUIPMENT_SLOT_STRIDE,
        USER_EQUIP_EQUIPMENT_STRIP_COUNT, USER_EQUIP_EQUIPMENT_STRIP_ORDER,
        USER_EQUIP_EQUIPMENT_TITLE_RECT, USER_EQUIP_FONT_PATH, USER_EQUIP_HELP_PATH,
        USER_EQUIP_HELP_RECT, USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
        USER_EQUIP_INVENTORY_CONTENT_WIDTH, USER_EQUIP_INVENTORY_PANEL_BORDER,
        USER_EQUIP_INVENTORY_PANEL_PATH, USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_STRIDE, USER_EQUIP_INVENTORY_VIEWPORT_RECT,
        USER_EQUIP_ITEM_TAB_HIT_RECT, USER_EQUIP_MISSING_CHECKER_SIZE,
        USER_EQUIP_NANO_TAB_HIT_RECT, USER_EQUIP_NANO_TAB_PATH, USER_EQUIP_NANO_TAB_TEXTURE_RECT,
        USER_EQUIP_REGULAR_FONT_LINE_HEIGHT, USER_EQUIP_RIGHT_PANEL_BORDER,
        USER_EQUIP_RIGHT_PANEL_PATH, USER_EQUIP_SLOT_EMPTY_PATH, USER_EQUIP_SLOT_OCCUPIED_PATH,
        USER_EQUIP_SMALL_FONT_LINE_HEIGHT, USER_EQUIP_SMALL_FONT_SIZE, USER_EQUIP_TAB_FONT_SIZE,
        USER_EQUIP_TRASH_PATH, USER_EQUIP_TRASH_RECT, UserEquipCatalogQuery,
        UserEquipCatalogQueryError, UserEquipItemCatalog, UserEquipItemModeLayout,
        UserEquipItemModeProjection, UserEquipMissingIconReason, UserEquipPresentationIcon,
        UserEquipProjectedIcon, UserEquipUiRect, user_equip_item_mode_layout,
        user_equip_missing_checker_rgba,
    },
};
use bevy::{
    asset::{LoadState, RenderAssetUsages},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::BorderRect,
    text::LineHeight,
    window::PrimaryWindow,
};
use ffone_protocol::{
    DecodedFrame, FixedUtf16, ItemBase0104, PayloadError, PcLoadData0104,
    RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104,
};
use std::{array, collections::VecDeque, error::Error, fmt};

use crate::ui_support::display_if;

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod constants;
mod containers;
mod assets;
mod interaction;
mod ingress;
mod gestures;
pub use ingress::decode_pc2pc_session_frame_0104;
mod layout;
mod codec;
mod types_pc2pc_offer_runtime0104;
mod types_pc2pc_authoritative_snapshot0104;
mod types_pc2pc_ui_model0104;
mod types_pc2pc_text_style0104;
mod projection;
mod commands;
mod validation;
mod operations;
mod output;
mod state;
mod view;
mod localization_equipment_slot_localized_text;

pub use constants::{
    PC2PC_SOURCE_BUILD, PC2PC_SOURCE_MAIN_ARCHIVE, PC2PC_SOURCE_MAIN_ARCHIVE_SHA256,
    PC2PC_SOURCE_TUTORIAL_ARCHIVE, PC2PC_SOURCE_TUTORIAL_ARCHIVE_SHA256, PC2PC_OPEN_SECONDS,
    PC2PC_PANEL_START_X, PC2PC_OFFER_SLOT_COUNT, PC2PC_PROTOCOL_TRADE_ITEM_COUNT,
    PC2PC_OFFER_SLOT_SIZE, PC2PC_OFFER_SLOT_STRIDE, PC2PC_CHAT_MAX_STORED_LINES,
    PC2PC_JEFFE_12_FONT_SIZE, PC2PC_JEFFE_14_FONT_SIZE, PC2PC_JEFFE_16_FONT_SIZE,
    PC2PC_CHALET_SMALL_FONT_SIZE, PC2PC_LABEL_PADDING_TOP, PC2PC_LABEL_PADDING_BOTTOM,
    PC2PC_READY_NAME_GAP, PC2PC_LOCAL_SLOT_ORIGIN, PC2PC_REMOTE_SLOT_ORIGIN,
    PC2PC_BLOCKED_CHAT_COMMANDS, PC2PC_UI_DEFAULT_IMAGE_PATHS
};
pub use containers::PC2PC_SOURCE_SERIALIZED_FILE;
pub use assets::{
    PC2PC_GAME_OBJECT_PATH_ID, PC2PC_PANEL_COMPONENT_PATH_ID, PC2PC_PANEL_SCRIPT_PATH_ID,
    PC2PC_PC_STUFF_COMPONENT_PATH_ID, PC2PC_MENU_CHAT_COMPONENT_PATH_ID,
    PC2PC_EQUIP_COMPONENT_PATH_ID, PC2PC_INVENTORY_SKIN_PATH_ID, PC2PC_CHAT_SKIN_PATH_ID,
    PC2PC_TRADE_BACK_PATH, PC2PC_TRADE_AREA_PATH, PC2PC_LOCAL_OFFER_PATH,
    PC2PC_LOCAL_OFFER_READY_PATH, PC2PC_REMOTE_OFFER_PATH, PC2PC_REMOTE_OFFER_READY_PATH,
    PC2PC_CHAT_BOX_PATH, PC2PC_MONEY_BACK_PATH, PC2PC_CHAT_SEND_PATH, PC2PC_TAROS_PATH,
    PC2PC_FREE_CHAT_PATH, PC2PC_PORTRAIT_BACK_PATH, PC2PC_TEXT_FIELD_PATH,
    PC2PC_JEFFE_FONT_PATH, PC2PC_CHALET_FONT_PATH, PC2PC_UI_Z_INDEX,
    PC2PC_JEFFE_12_SOURCE_FONT_PATH_ID, PC2PC_JEFFE_14_SOURCE_FONT_PATH_ID,
    PC2PC_JEFFE_16_SOURCE_FONT_PATH_ID, PC2PC_CHALET_SMALL_SOURCE_FONT_PATH_ID,
    Pc2pcStaticAssetRole, Pc2pcUiAssetContract, Pc2pcStaticAssetReadiness
};
pub use interaction::{
    PC2PC_CHAT_SEND_HOVER_PATH, PC2PC_BUTTON_PATH, PC2PC_BUTTON_HOVER_PATH,
    PC2PC_CHAT_MAX_INPUT_CHARS, PC2PC_BUTTON_PADDING_LEFT, PC2PC_BUTTON_PADDING_RIGHT,
    PC2PC_BUTTON_PADDING_TOP, PC2PC_BUTTON_PADDING_BOTTOM, Pc2pcButtonLabel
};
pub use layout::{
    PC2PC_REFERENCE_WIDTH, PC2PC_REFERENCE_HEIGHT, PC2PC_BACKPLATE_REFERENCE_WIDTH,
    PC2PC_BACKPLATE_REFERENCE_HEIGHT, PC2PC_PANEL_WIDTH, PC2PC_PANEL_HEIGHT,
    PC2PC_CHAT_LINE_HEIGHT, PC2PC_JEFFE_12_LINE_HEIGHT, PC2PC_JEFFE_14_LINE_HEIGHT,
    PC2PC_JEFFE_16_LINE_HEIGHT, PC2PC_CHALET_SMALL_LINE_HEIGHT, PC2PC_TRADE_AREA_BORDER,
    PC2PC_MONEY_BACK_BORDER, PC2PC_BUTTON_BORDER, Pc2pcUiRect, PC2PC_TRADE_AREA_RECT,
    PC2PC_LOCAL_OFFER_RECT, PC2PC_REMOTE_OFFER_RECT, PC2PC_LOCAL_MONEY_RECT,
    PC2PC_REMOTE_MONEY_RECT, PC2PC_ADD_TAROS_RECT, PC2PC_LOCAL_PORTRAIT_RECT,
    PC2PC_REMOTE_PORTRAIT_RECT, PC2PC_LOCAL_TITLE_RECT, PC2PC_REMOTE_TITLE_RECT,
    PC2PC_MAIN_BUTTON_RECT, PC2PC_READY_NAME_RECT, PC2PC_READY_SUBJECT_RECT,
    PC2PC_CHAT_BOX_RECT, PC2PC_CHAT_LIST_VIEW_RECT, PC2PC_CHAT_INPUT_RECT,
    PC2PC_CHAT_SEND_RECT, Pc2pcOfferDirection0104, Pc2pcModeLayout, pc2pc_mode_layout
};
use layout::bind_rect;
pub use codec::{
    PC2PC_TRADE_OFFER_REQUEST_PACKET_ID_0104, PC2PC_TRADE_OFFER_CANCEL_REQUEST_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_ACCEPT_REQUEST_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_REFUSAL_REQUEST_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_RESPONSE_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_CANCEL_RESPONSE_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_SUCCESS_RESPONSE_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_REFUSAL_RESPONSE_PACKET_ID_0104,
    PC2PC_TRADE_OFFER_ABORT_RESPONSE_PACKET_ID_0104, Pc2pcOfferRequestCodecError0104,
    Pc2pcOfferFrameError0104, decode_pc2pc_offer_frame_0104, Pc2pcRequestCodecError0104
};
use codec::{encode_trade_item, bind_offer_frame};
pub use types_pc2pc_offer_runtime0104::{
    Pc2pcPair0104, Pc2pcIdentityError0104, Pc2pcEnvelope0104, Pc2pcOfferRequest0104,
    Pc2pcOfferReply0104, Pc2pcPendingOffer0104, Pc2pcOfferRuntime0104,
    Pc2pcOfferFlowError0104, Pc2pcOfferIngressError0104, Pc2pcParticipant0104,
    Pc2pcParticipantNames0104, Pc2pcTradeItem0104, Pc2pcTradeItemError0104,
    Pc2pcAuthoritativeSnapshot0104
};
pub use types_pc2pc_authoritative_snapshot0104::{
    Pc2pcSnapshotError0104, Pc2pcAuthorityError0104, Pc2pcFinalCommitReceipt0104,
    Pc2pcLifecyclePhase, Pc2pcBackendCapabilities, Pc2pcPortraitRef, Pc2pcPortraitBindings,
    Pc2pcChatLine0104, Pc2pcConfirmIntent0104, Pc2pcCancelIntent0104,
    Pc2pcRegisterItemIntent0104, Pc2pcUnregisterItemIntent0104, Pc2pcRegisterCashIntent0104,
    Pc2pcChatIntent0104, Pc2pcIntent0104, Pc2pcPendingRequest0104, Pc2pcUiOutbox0104,
    Pc2pcServerOutcome0104, Pc2pcLastFailure0104, Pc2pcCorrelationError0104,
    Pc2pcEquipEligibility, Pc2pcFailClosedEquipEligibility, Pc2pcOfferSlotProjection0104,
    Pc2pcUiModel0104
};
use types_pc2pc_ui_model0104::Pc2pcUiAssets;
pub use types_pc2pc_ui_model0104::{Pc2pcUiRoot, Pc2pcUiElement, Pc2pcTextStyle0104};
pub use types_pc2pc_text_style0104::{Pc2pcDisabledControl, Pc2pcUiSet, Pc2pcUiPlugin};
pub use projection::{Pc2pcSessionIdentity0104, Pc2pcSessionEndReason0104};
use projection::project_pc2pc_icon;
pub use commands::{
    Pc2pcOfferRequestKind0104, Pc2pcOfferReplyKind0104, Pc2pcRequestFailureKind0104,
    Pc2pcActionError0104
};
use commands::pending_request;
use validation::{validate_trade_item, validate_final_pending, validate_runtime_asset_path};
pub use validation::{
    Pc2pcPortraitRefError, Pc2pcEquipValidation, Pc2pcAssetPathError,
    Pc2pcUiAssetContractError
};
use operations::{
    correlate_local_offer_stay, take_local_pending_confirm, take_matching_failure_pending,
    pc2pc_fallback_text, pc2pc_passthrough_text, pc2pc_count_text, pc2pc_taros_amount_text,
    pc2pc_remote_offer_text, pc2pc_chat_log_text, equipment_slot_label, bind_pc2pc_ui
};
use output::write_item_base;
pub use state::{Pc2pcModalState, Pc2pcUiState, Pc2pcModeProjection0104};
use state::tick_pc2pc_state;
use view::spawn_pc2pc_ui;
use localization_equipment_slot_localized_text::{
    equipment_slot_localized_text, bind_localized_text
};
