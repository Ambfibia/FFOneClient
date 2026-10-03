//! Standalone clean-Retrobution `UserStore` / street-stall mode-28 contract.
//!
//! `main.unity3d` is the parity authority for inactive GameObject 1312 and its
//! `cnStore`, `Panel_PCStuffScript`, `Panel_UserStore`, and `Panel_Equip`
//! components (1434..1437).  The managed sources named by the evidence file
//! own behavior; the serialized objects own references. Runtime code consumes
//! the validated native GumPopup background plus byte-proven shared Vendor,
//! Email and UserEquip primitives; it never opens a Unity container or
//! `.ffclient` cache.
//!
//! Street-stall replies are authoritative transaction boundaries.  Requests
//! never mutate Taros, inventory, or listing state.  A malformed, stray, or
//! mismatched reply is rejected without clearing the send lock or partially
//! applying a state change.  The one exception to request correlation is the
//! seller notification: another player can buy an open listing asynchronously.

use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    fmt,
};

use bevy::{
    asset::{LoadState, RenderAssetUsages},
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::BorderRect,
    text::LineHeight,
    window::PrimaryWindow,
};
use ffone_protocol::ItemBase0104;

use crate::localization::{LocalizationSet, LocalizedText, UiTextAutoFit};

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod constants;
mod containers;
mod state;
mod assets;
mod layout;
mod interaction;
mod commands;
mod codec;
mod types_user_store_authority0104;
mod types_user_store_ui_state0104;
mod types_user_store_popup_text_style0104;
mod validation;
mod input_read_i32;
mod output;
mod binding_user_store_ui;
mod operations_user_store_missing_checker_image;
mod systems;
mod projection;
mod view_user_store_popup;
mod view_user_store_image;
mod textures;

pub use constants::{
    USER_STORE_SOURCE_BUILD, USER_STORE_SOURCE_ARCHIVE, USER_STORE_SOURCE_ARCHIVE_SHA256,
    USER_STORE_OPEN_SECONDS, USER_STORE_LIST_CAPACITY, USER_STORE_EQUIPMENT_CAPACITY,
    USER_STORE_GENERAL_ITEM_TYPE, USER_STORE_MY_SLOT_TYPE, USER_STORE_OTHER_SLOT_TYPE,
    USER_STORE_TITLE_SUFFIX_TYPO, USER_STORE_CANNOT_UNREGISTER, USER_STORE_CANNOT_REGISTER,
    USER_STORE_REGISTER_OVER_MAX, USER_STORE_TARGET_CLOSED_MESSAGE_KEY,
    USER_STORE_JEFFE_12_FONT_SIZE, USER_STORE_JEFFE_14_FONT_SIZE,
    USER_STORE_JEFFE_16_FONT_SIZE, USER_STORE_CHALET_SMALL_FONT_SIZE, EMPTY_ITEM_0104,
    USER_STORE_PANEL_DEPTH, USER_STORE_POPUP_DEPTH, USER_STORE_POPUP_TYPE,
    USER_STORE_POPUP_MAX_PRICE, USER_STORE_GENERIC_CALCULATOR_DIGIT_AREA,
    STREETSTALL_REQ_READY, STREETSTALL_REQ_CANCEL, STREETSTALL_REQ_REGISTER_ITEM,
    STREETSTALL_REQ_UNREGISTER_ITEM, STREETSTALL_REQ_SALE_START, STREETSTALL_REQ_ITEM_LIST,
    STREETSTALL_REQ_ITEM_BUY, STREETSTALL_REP_READY_SUCCESS, STREETSTALL_REP_READY_FAIL,
    STREETSTALL_REP_CANCEL_SUCCESS, STREETSTALL_REP_CANCEL_FAIL,
    STREETSTALL_REP_REGISTER_ITEM_SUCCESS, STREETSTALL_REP_REGISTER_ITEM_FAIL,
    STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS, STREETSTALL_REP_UNREGISTER_ITEM_FAIL,
    STREETSTALL_REP_SALE_START_SUCCESS, STREETSTALL_REP_SALE_START_FAIL,
    STREETSTALL_REP_ITEM_LIST, STREETSTALL_REP_ITEM_LIST_FAIL,
    STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER, STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER,
    STREETSTALL_REP_ITEM_BUY_FAIL, STREETSTALL_READY_SUCCESS_SIZE, STREETSTALL_FAILURE_SIZE,
    STREETSTALL_CANCEL_SUCCESS_SIZE, STREETSTALL_REGISTER_SUCCESS_SIZE,
    STREETSTALL_UNREGISTER_SUCCESS_SIZE, STREETSTALL_SALE_START_SUCCESS_SIZE,
    STREETSTALL_ITEM_LIST_HEADER_SIZE, STREETSTALL_ITEM_LIST_RECORD_SIZE,
    STREETSTALL_ITEM_BUY_SUCCESS_SIZE
};
pub use containers::USER_STORE_SOURCE_SERIALIZED_FILE;
pub use state::{
    USER_STORE_GAME_MODE_SLOT, USER_STORE_INVENTORY_CAPACITY, USER_STORE_INVENTORY_COLUMNS,
    USER_STORE_INVENTORY_ROWS, USER_STORE_INVENTORY_SLOT_SIZE,
    USER_STORE_INVENTORY_SLOT_STRIDE, UserStoreInventoryProjection0104
};
pub use assets::{
    USER_STORE_GAME_OBJECT_PATH_ID, USER_STORE_CN_STORE_COMPONENT_PATH_ID,
    USER_STORE_PC_STUFF_COMPONENT_PATH_ID, USER_STORE_PANEL_COMPONENT_PATH_ID,
    USER_STORE_EQUIP_COMPONENT_PATH_ID, USER_STORE_INVENTORY_SKIN_PATH_ID,
    USER_STORE_UI_Z_INDEX, USER_STORE_BACKDROP_PATH, USER_STORE_LEFT_PANEL_PATH,
    USER_STORE_INFO_PATH, USER_STORE_LIST_BACK_PATH, USER_STORE_LIST_DIVIDER_PATH,
    USER_STORE_ITEM_ROW_PATH, USER_STORE_ITEM_TAB_PATH, USER_STORE_SHADOW_PATH,
    USER_STORE_RIGHT_PANEL_PATH, USER_STORE_INVENTORY_PANEL_PATH,
    USER_STORE_SLOT_OCCUPIED_PATH, USER_STORE_SLOT_EMPTY_PATH, USER_STORE_EQUIP_TITLE_PATH,
    USER_STORE_CLOSE_PATH, USER_STORE_TRASH_PATH, USER_STORE_HELP_PATH,
    USER_STORE_POPUP_BACKGROUND_PATH, USER_STORE_POPUP_CALCULATOR_PATH, USER_STORE_FONT_PATH,
    USER_STORE_BODY_FONT_PATH, USER_STORE_JEFFE_12_SOURCE_FONT_PATH_ID,
    USER_STORE_JEFFE_14_SOURCE_FONT_PATH_ID, USER_STORE_JEFFE_16_SOURCE_FONT_PATH_ID,
    USER_STORE_CHALET_SMALL_SOURCE_FONT_PATH_ID, USER_STORE_SHARED_ASSET_PATHS,
    USER_STORE_ALL_ASSET_PATHS, USER_STORE_IMAGE_ASSET_PATHS, USER_STORE_POPUP_LOCAL_Z_INDEX,
    USER_STORE_POPUP_BACKGROUND_PATH_ID, USER_STORE_POPUP_CALCULATOR_PATH_ID,
    UserStoreStaticAssetReadiness0104, UserStoreUiAssetStatus0104
};
use assets::{list_slot_index, inventory_slot_index, UserStoreAssetRole0104};
pub use layout::{
    USER_STORE_REFERENCE_WIDTH, USER_STORE_REFERENCE_HEIGHT, USER_STORE_SHELL_WIDTH,
    USER_STORE_SHELL_HEIGHT, USER_STORE_PANEL_WIDTH, USER_STORE_PANEL_HEIGHT,
    USER_STORE_ROW_HEIGHT, USER_STORE_ROW_WIDTH, USER_STORE_ROW_VISUAL_HEIGHT,
    USER_STORE_INVENTORY_CONTENT_HEIGHT, USER_STORE_INVENTORY_VIEWPORT_HEIGHT,
    USER_STORE_JEFFE_12_LINE_HEIGHT, USER_STORE_JEFFE_14_LINE_HEIGHT,
    USER_STORE_JEFFE_16_LINE_HEIGHT, USER_STORE_CHALET_SMALL_LINE_HEIGHT, UserStoreUiRect,
    USER_STORE_DIALOG_RECT, USER_STORE_LIST_BACK_RECT, USER_STORE_TABLE_RECT,
    USER_STORE_LIST_VIEWPORT_RECT, USER_STORE_TABLE_SHADOW_RECT, USER_STORE_INFO_RECT,
    USER_STORE_TITLE_RECT, USER_STORE_ITEM_TAB_RECT, USER_STORE_ITEM_TAB_HIT_RECT,
    USER_STORE_LIST_DIVIDER_RECT, USER_STORE_PRIMARY_BUTTON_RECT, USER_STORE_GO_TO_GAME_RECT,
    USER_STORE_ROW_ITEM_RECT, USER_STORE_ROW_NAME_RECT, USER_STORE_ROW_LEVEL_RECT,
    USER_STORE_ROW_PRICE_RECT, USER_STORE_ROW_SELLER_NET_RECT, USER_STORE_PC_STUFF_RECT,
    USER_STORE_INVENTORY_VIEWPORT_RECT, USER_STORE_CLOSE_RECT, USER_STORE_TRASH_RECT,
    USER_STORE_HELP_RECT, USER_STORE_EQUIP_PANEL_RECT, USER_STORE_EQUIP_CONTENT_RECT,
    USER_STORE_POPUP_RECT, USER_STORE_POPUP_IN_POSITION, USER_STORE_POPUP_OUT_POSITION,
    USER_STORE_POPUP_ICON_RECT, USER_STORE_POPUP_NAME_RECT, USER_STORE_POPUP_COST_RECT,
    USER_STORE_POPUP_DESCRIPTION_RECT, USER_STORE_POPUP_CLOSE_RECT,
    USER_STORE_POPUP_VALUE_TYPE_RECT, USER_STORE_POPUP_CALCULATOR_RECT,
    USER_STORE_POPUP_VALUE_RECT, USER_STORE_POPUP_REGISTER_ACTION_RECT,
    USER_STORE_POPUP_LISTING_ACTION_RECT, USER_STORE_GENERIC_CALCULATOR_RECT,
    USER_STORE_GENERIC_CALCULATOR_VALUE_RECT, USER_STORE_POPUP_BUTTON_RECTS,
    USER_STORE_POPUP_CLEAR_RECT, USER_STORE_POPUP_NO_OP_RECT, USER_STORE_LIST_BACK_BORDER,
    USER_STORE_RIGHT_PANEL_BORDER, USER_STORE_INVENTORY_PANEL_BORDER,
    USER_STORE_BUTTON_BORDER, USER_STORE_POPUP_BUTTON_BORDER, user_store_popup_rect_0104,
    UserStoreUiLayout0104, user_store_layout_0104
};
use layout::bind_user_store_rect;
pub use interaction::{
    USER_STORE_SCROLL_VELOCITY, USER_STORE_SCROLL_INPUT_CLAMP, USER_STORE_BUTTON_NORMAL_PATH,
    USER_STORE_BUTTON_HOVER_PATH, USER_STORE_POPUP_CLOSE_HOVER_PATH, UserStoreRowButton0104,
    UserStoreInputBoundary0104, user_store_inventory_scroll_max,
    clamp_user_store_inventory_scroll
};
use interaction::{user_store_primary_button_text, collect_user_store_input_0104};
pub use commands::{
    USER_STORE_HELP_EVENT_ID, USER_STORE_GENERIC_CALCULATOR_ACTION_AREA,
    STREETSTALL_READY_REQUEST_SIZE, STREETSTALL_CANCEL_REQUEST_SIZE,
    STREETSTALL_REGISTER_REQUEST_SIZE, STREETSTALL_UNREGISTER_REQUEST_SIZE,
    STREETSTALL_SALE_START_REQUEST_SIZE, STREETSTALL_ITEM_LIST_REQUEST_SIZE,
    STREETSTALL_ITEM_BUY_REQUEST_SIZE, UserStoreReplyReject0104, UserStoreRequestBlock0104,
    UserStoreActionOutcome0104, apply_user_store_popup_action_0104
};
use commands::user_store_popup_action_text;
pub use codec::{
    USER_STORE_RESTRICTED_FRAME_PATH, UserStoreCodecError0104, decode_user_store_reply_0104
};
use codec::encode_item;
#[cfg(test)]
use codec::decode_item;
pub use types_user_store_authority0104::{
    UserStoreMode0104, UserStorePacket0104, UserStoreReadySuccess0104,
    UserStoreListingRecord0104, UserStoreItemListSuccess0104, UserStoreRegisterSuccess0104,
    UserStoreSaleStartSuccess0104, UserStoreBuyBuyerSuccess0104,
    UserStoreBuySellerSuccess0104, UserStoreReply0104, UserStoreListing0104,
    UserStoreAuthority0104, UserStorePendingRequest0104, UserStoreModalState0104,
    UserStorePopupKind0104, UserStoreItemPopup0104, UserStorePopupPresentation0104,
    UserStoreUiCommand0104, UserStoreUiOutbox0104, UserStoreUiState0104
};
pub use types_user_store_ui_state0104::{
    UserStoreItemMetadata0104, UserStoreItemCatalog0104, UserStorePresentationIcon0104,
    UserStoreProjectedItem0104, UserStoreListingRowProjection0104, UserStoreUiProjection0104
};
pub use types_user_store_popup_text_style0104::{
    UserStorePreviewMode0104, UserStoreUiRoot0104, UserStoreUiElement0104,
    UserStoreUiTextRole0104, UserStorePopupTextStyle0104, UserStoreUiSet0104,
    UserStoreUiPlugin0104
};
use types_user_store_popup_text_style0104::{
    UserStoreUiAssets0104, UserStoreInteractiveControl0104
};
use validation::{require_exact, user_store_error_text};
use input_read_i32::{read_i32, read_f32};
use output::write_i32;
pub use binding_user_store_ui::{
    user_store_item_is_empty, user_store_opening_eased_fraction,
    user_store_seller_display_taros, seed_user_store_preview_0104,
    seed_user_store_popup_preview_0104, user_store_format_number
};
use binding_user_store_ui::{
    push_entry_commands, user_store_text_font, user_store_passthrough_text, user_store_level_text,
    user_store_popup_cost_text, user_store_popup_value_text, user_store_taros_text,
    user_store_resolved_fallback, adopt_user_store_popup_outcome_0104, bind_user_store_ui_0104
};
use operations_user_store_missing_checker_image::user_store_missing_checker_image;
pub use systems::UserStoreApplyError0104;
use systems::advance_user_store_ui_0104;
use projection::project_user_store_item;
pub use projection::project_user_store_ui_0104;
use view_user_store_popup::spawn_user_store_ui_0104;
use view_user_store_image::spawn_user_store_image;
pub use textures::user_store_missing_checker_rgba_0104;
