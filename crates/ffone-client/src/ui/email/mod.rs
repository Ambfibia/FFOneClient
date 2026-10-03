//! Clean-Retrobution `eGameMode.Email` state, transport boundary, and UI.
//!
//! The parity authority is the unmodified `retrobution-20260613` build. Its
//! `main.unity3d/sharedassets0.assets` owns the inactive `EmailMode` root,
//! `EmailMode`, `Panel_EmailList`, `Panel_NewEmail`, `Panel_PCStuffScript`,
//! `FusionFallInvenSkin`, and the embedded email textures. `emailnew.png` and
//! `panelback.png` are owned by `Tutorial.resourceFile`. Converted PNGs are
//! ordinary native assets; this module never opens Unity containers or the
//! `.ffclient` navigation cache at runtime.
//!
//! Packet structs are deliberately represented as typed semantic values here,
//! not byte encoders. Their clean packet IDs and ABI sizes are part of the
//! contract so a production transport owner can encode them without letting
//! UI code mutate inventory, Taros, or server mail speculatively.

use std::{array, collections::VecDeque, error::Error, fmt};

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput, mouse::AccumulatedMouseScroll},
    prelude::*,
    sprite::BorderRect,
    text::{LineBreak, LineHeight},
    window::PrimaryWindow,
};
use ffone_client_network::network::{NetworkBridge, NetworkCommand};
use ffone_protocol::{DecodedFrame, RegisteredGameplayRequest0104};

use crate::{
    email_runtime::{EmailRuntimeDelivery0104, EmailTransportRuntime0104},
    localization::{LocalizationSet, LocalizedText, LocalizedTextLimit, UiTextAutoFit},
};

use crate::ui_support::legacy_screen_extent as legacy_extent;

#[path = "body_scroll.rs"]
mod body_scroll;

use crate::ui_support::stretched_image as stretch_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod constants;
mod state;
mod assets;
mod interaction;
mod frame;
mod validation;
mod input_resolve_email_escape_gate;
mod layout;
mod animation;
mod types_email_ui_text_style;
mod types_email_ui_plugin;
mod commands;
mod models;
mod operations_flush_email_transport_outbox;
mod operations_email_summary_sender_text;
mod audio;
mod view_email_list_panel;
mod view_email_calculator_popup;
mod systems;
mod localization_email_localized_text;

pub use constants::{
    EMAIL_UI_SOURCE_BUILD, EMAIL_UI_ROOT_INITIALLY_ACTIVE, EMAIL_UI_MAIN_SHA256,
    EMAIL_UI_TUTORIAL_SHA256, EMAIL_PAGE_SIZE, EMAIL_PAGE_MAX, EMAIL_ATTACHMENT_COUNT,
    EMAIL_DELETE_BATCH_COUNT, EMAIL_BASE_POSTAGE, EMAIL_ITEM_POSTAGE,
    EMAIL_SEND_DELAY_SECONDS, EMAIL_REQ_PAGE_LIST_ID, EMAIL_REQ_DELETE_ID, EMAIL_REQ_SEND_ID,
    EMAIL_REQ_RECEIVE_ITEM_ID, EMAIL_REQ_RECEIVE_CASH_ID, EMAIL_REQ_RECEIVE_ALL_ID,
    EMAIL_REP_NEW_ID, EMAIL_REP_PAGE_LIST_SUCCESS_ID, EMAIL_REP_PAGE_LIST_FAILURE_ID,
    EMAIL_REP_DELETE_SUCCESS_ID, EMAIL_REP_DELETE_FAILURE_ID, EMAIL_REP_SEND_SUCCESS_ID,
    EMAIL_REP_SEND_FAILURE_ID, EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID,
    EMAIL_REP_RECEIVE_ITEM_FAILURE_ID, EMAIL_REP_RECEIVE_CASH_SUCCESS_ID,
    EMAIL_REP_RECEIVE_CASH_FAILURE_ID, EMAIL_REP_RECEIVE_ALL_SUCCESS_ID,
    EMAIL_REP_RECEIVE_ALL_FAILURE_ID, EMAIL_REQ_PAGE_LIST_SIZE, EMAIL_REQ_DELETE_SIZE,
    EMAIL_REQ_SEND_SIZE, EMAIL_REQ_RECEIVE_ITEM_SIZE, EMAIL_REQ_RECEIVE_CASH_SIZE,
    EMAIL_REQ_RECEIVE_ALL_SIZE, EMAIL_REP_NEW_SIZE, EMAIL_REP_PAGE_LIST_SUCCESS_SIZE,
    EMAIL_REP_PAGE_LIST_FAILURE_SIZE, EMAIL_REP_DELETE_SUCCESS_SIZE,
    EMAIL_REP_DELETE_FAILURE_SIZE, EMAIL_REP_SEND_SUCCESS_SIZE, EMAIL_REP_SEND_FAILURE_SIZE,
    EMAIL_REP_RECEIVE_ITEM_SUCCESS_SIZE, EMAIL_REP_RECEIVE_ITEM_FAILURE_SIZE,
    EMAIL_REP_RECEIVE_CASH_SUCCESS_SIZE, EMAIL_REP_RECEIVE_CASH_FAILURE_SIZE,
    EMAIL_REP_RECEIVE_ALL_SUCCESS_SIZE, EMAIL_REP_RECEIVE_ALL_FAILURE_SIZE,
    EMAIL_UI_JEFFE_12_FONT_SIZE, EMAIL_UI_JEFFE_14_FONT_SIZE, EMAIL_UI_JEFFE_16_FONT_SIZE,
    EMAIL_UI_JEFFE_06_FONT_SIZE, EMAIL_UI_CHALET_SMALL_FONT_SIZE, EMAIL_UI_LABEL_PADDING_TOP,
    EMAIL_UI_LABEL_PADDING_BOTTOM, EMAIL_UI_POSTAGE_LABEL_Y_OFFSET,
    EMAIL_UI_POSTAGE_LABEL_GAP, EMAIL_UI_IMAGE_PATHS, EMAIL_UI_OPEN_SECONDS,
    EMAIL_UI_RIGHT_OFFSCREEN_X
};
pub use state::{
    EMAIL_UI_GAME_MODE_VALUE, EMAIL_INVENTORY_SLOT_COUNT, EMAIL_TIME_TRAVEL_SPECIAL_STATE,
    EMAIL_UI_INVENTORY_COLUMNS, EMAIL_UI_INVENTORY_SLOT_SIZE, EMAIL_UI_INVENTORY_SLOT_STRIDE,
    EmailInventorySlotView, request_delete_selected_email, EmailUiRowSelection,
    EmailUiInventorySlot, EmailUiInventoryIcon
};
use state::{
    email_inventory_source_valid, email_inventory_slot_staged,
    first_free_email_inventory_slot, email_page_selection_text
};
pub use assets::{
    EMAIL_UI_GAME_OBJECT_PATH_ID, EMAIL_UI_LIST_COMPONENT_PATH_ID,
    EMAIL_UI_COMPOSE_COMPONENT_PATH_ID, EMAIL_UI_MODE_COMPONENT_PATH_ID,
    EMAIL_UI_PC_STUFF_COMPONENT_PATH_ID, EMAIL_UI_LIST_SCRIPT_PATH_ID,
    EMAIL_UI_COMPOSE_SCRIPT_PATH_ID, EMAIL_UI_MODE_SCRIPT_PATH_ID,
    EMAIL_UI_PC_STUFF_SCRIPT_PATH_ID, EMAIL_UI_INVENTORY_SKIN_PATH_ID, EMAIL_UI_LIST_PATH,
    EMAIL_UI_GUIDE_TAB_PATH, EMAIL_UI_PLAYER_TAB_PATH, EMAIL_UI_INACTIVE_TAB_FILL_PATH,
    EMAIL_UI_ATTACHMENT_PATH, EMAIL_UI_BUDDY_POPUP_PATH, EMAIL_UI_CALCULATOR_PAD_PATH,
    EMAIL_UI_CALCULATOR_POPUP_PATH, EMAIL_UI_DATA_BACK_PATH, EMAIL_UI_DATA_BOX_PATH,
    EMAIL_UI_SORT_ARROW_PATH, EMAIL_UI_TAROS_PATH, EMAIL_UI_PREVIOUS_PATH, EMAIL_UI_NEXT_PATH,
    EMAIL_UI_COMPOSE_PATH, EMAIL_UI_LIST_SELECTION_PATH, EMAIL_UI_BACKDROP_PATH,
    EMAIL_UI_RIGHT_PANEL_PATH, EMAIL_UI_INVENTORY_PANEL_PATH, EMAIL_UI_SLOT_OCCUPIED_PATH,
    EMAIL_UI_SLOT_EMPTY_PATH, EMAIL_UI_CLOSE_PATH, EMAIL_UI_FONT_PATH,
    EMAIL_UI_BODY_FONT_PATH, EMAIL_UI_JEFFE_12_SOURCE_FONT_PATH_ID,
    EMAIL_UI_JEFFE_14_SOURCE_FONT_PATH_ID, EMAIL_UI_JEFFE_16_SOURCE_FONT_PATH_ID,
    EMAIL_UI_JEFFE_06_SOURCE_FONT_PATH_ID, EMAIL_UI_CHALET_SMALL_SOURCE_FONT_PATH_ID,
    EMAIL_UI_Z_INDEX, EMAIL_UI_POPUP_Z_INDEX
};
pub use interaction::{
    EMAIL_SUBJECT_INPUT_LIMIT, EMAIL_CONTENT_INPUT_LIMIT, EMAIL_UI_GUIDE_TAB_HOVER_PATH,
    EMAIL_UI_PLAYER_TAB_HOVER_PATH, EMAIL_UI_PREVIOUS_HOVER_PATH, EMAIL_UI_NEXT_HOVER_PATH,
    EMAIL_UI_BUTTON_PATH, EMAIL_UI_BUTTON_HOVER_PATH, EMAIL_UI_RED_BUTTON_PATH,
    EMAIL_UI_RED_BUTTON_HOVER_PATH, EMAIL_UI_CLOSE_HOVER_PATH, EMAIL_UI_BUTTON_PADDING_LEFT,
    EMAIL_UI_BUTTON_PADDING_RIGHT, EMAIL_UI_BUTTON_PADDING_TOP,
    EMAIL_UI_BUTTON_PADDING_BOTTOM, EMAIL_UI_SHARED_SCROLL_VELOCITY,
    EMAIL_UI_BUDDY_SCROLL_STEP_LIMIT, email_buddy_scroll_max, EmailCalculatorInput,
    EmailInputBoundary, input_email_calculator, EmailUiButtonKind, EmailUiButton,
    EmailUiRowButton
};
use interaction::{
    EmailUiItemDragSource, EmailUiItemDragState, handle_email_keyboard, handle_email_buddy_scroll,
    handle_email_item_interactions, handle_email_interactions, sync_email_ui_button_visuals
};
#[cfg(test)]
use interaction::email_ui_button_enabled;
pub use frame::{
    EMAIL_SUBJECT_WIRE_UNITS, EMAIL_CONTENT_WIRE_UNITS, EMAIL_UI_FRAME_PATH,
    EMAIL_UI_FRAME_BORDER, EmailWireItem
};
pub use validation::{EMAIL_REQ_UPDATE_CHECK_ID, EMAIL_REQ_UPDATE_CHECK_SIZE, EmailComposeError};
pub use input_resolve_email_escape_gate::{
    EMAIL_REQ_READ_ID, EMAIL_REP_READ_SUCCESS_ID, EMAIL_REP_READ_FAILURE_ID,
    EMAIL_REQ_READ_SIZE, EMAIL_REP_READ_SUCCESS_SIZE, EMAIL_REP_READ_FAILURE_SIZE,
    EmailReadMessage, resolve_email_escape_gate, resolve_email_computress_gate
};
pub use layout::{
    EMAIL_UI_JEFFE_12_LINE_HEIGHT, EMAIL_UI_JEFFE_14_LINE_HEIGHT,
    EMAIL_UI_JEFFE_16_LINE_HEIGHT, EMAIL_UI_JEFFE_06_LINE_HEIGHT,
    EMAIL_UI_CHALET_SMALL_LINE_HEIGHT, EMAIL_UI_REFERENCE_WIDTH, EMAIL_UI_REFERENCE_HEIGHT,
    EMAIL_UI_BACKPLATE_WIDTH, EMAIL_UI_BACKPLATE_HEIGHT, EMAIL_UI_BACKGROUND_WIDTH,
    EMAIL_UI_BACKGROUND_HEIGHT, EMAIL_UI_REFERENCE_SCALE_HEIGHT, EMAIL_UI_SCALE_NUDGE,
    EmailUiRect, EMAIL_UI_LIST_WINDOW_RECT, EMAIL_UI_LIST_INNER_RECT, EMAIL_UI_LIST_BACK_RECT,
    EMAIL_UI_DATA_BACK_RECT, EMAIL_UI_SEND_MAIL_RECT, EMAIL_UI_GUIDE_TAB_RECT,
    EMAIL_UI_GUIDE_TAB_HIT_RECT, EMAIL_UI_PLAYER_TAB_RECT, EMAIL_UI_PLAYER_TAB_HIT_RECT,
    EMAIL_UI_INACTIVE_TAB_FILL_RECT, EMAIL_UI_FROM_TITLE_RECT, EMAIL_UI_SUBJECT_TITLE_RECT,
    EMAIL_UI_DATE_TITLE_RECT, EMAIL_UI_FROM_BOX_RECT, EMAIL_UI_SUBJECT_BOX_RECT,
    EMAIL_UI_DATE_BOX_RECT, EMAIL_UI_PREVIOUS_RECT, EMAIL_UI_NEXT_RECT,
    EMAIL_UI_PAGE_SELECTION_RECT, EMAIL_UI_PAGE_OF_RECT, EMAIL_UI_PAGE_NUMBER_RECT,
    EMAIL_UI_PAGE_LABEL_RECT, EMAIL_UI_ROW_SELECTION_RECT, EMAIL_UI_DETAIL_ICON_RECT,
    EMAIL_UI_DETAIL_GUIDE_ICON_IMAGE_RECT, EMAIL_UI_DETAIL_FROM_RECT,
    EMAIL_UI_DETAIL_FROM_NAME_RECT, EMAIL_UI_DETAIL_SUBJECT_RECT,
    EMAIL_UI_DETAIL_SUBJECT_NAME_RECT, EMAIL_UI_DETAIL_RECEIVED_RECT,
    EMAIL_UI_DETAIL_RECEIVED_DAY_RECT, EMAIL_UI_DETAIL_TEXT_RECT, EMAIL_UI_DETAIL_ITEM_RECT,
    EMAIL_UI_ACCEPT_ALL_RECT, EMAIL_UI_ACCEPT_TAROS_RECT, EMAIL_UI_TAROS_LABEL_RECT,
    EMAIL_UI_REMOVE_BUDDY_RECT, EMAIL_UI_DELETE_RECT, EMAIL_UI_REPLY_RECT,
    EMAIL_UI_COMPOSE_WINDOW_RECT, EMAIL_UI_COMPOSE_INNER_RECT, EMAIL_UI_COMPOSE_TITLE_RECT,
    EMAIL_UI_COMPOSE_CLOSE_RECT, EMAIL_UI_COMPOSE_TO_RECT, EMAIL_UI_COMPOSE_TO_NAME_RECT,
    EMAIL_UI_COMPOSE_SUBJECT_RECT, EMAIL_UI_COMPOSE_SUBJECT_FIELD_RECT,
    EMAIL_UI_COMPOSE_BUDDY_RECT, EMAIL_UI_COMPOSE_BODY_RECT, EMAIL_UI_COMPOSE_BODY_FIELD_RECT,
    EMAIL_UI_COMPOSE_ATTACHMENT_ICON_RECT, EMAIL_UI_COMPOSE_ATTACHMENT_LABEL_RECT,
    EMAIL_UI_COMPOSE_ITEM_RECT, EMAIL_UI_COMPOSE_ADD_TAROS_RECT, EMAIL_UI_COMPOSE_TAROS_RECT,
    EMAIL_UI_COMPOSE_POSTAGE_RECT, EMAIL_UI_COMPOSE_CANCEL_RECT, EMAIL_UI_COMPOSE_SEND_RECT,
    EMAIL_UI_BUDDY_POPUP_RECT, EMAIL_UI_BUDDY_LIST_VIEWPORT_RECT,
    EMAIL_UI_BUDDY_LIST_CONTENT_WIDTH, EMAIL_UI_BUDDY_ROW_WIDTH, EMAIL_UI_BUDDY_ROW_HEIGHT,
    EMAIL_UI_CALCULATOR_POPUP_RECT, EMAIL_UI_RIGHT_PANEL_RECT,
    EMAIL_UI_INVENTORY_VIEWPORT_RECT, EMAIL_UI_RIGHT_CLOSE_RECT, EMAIL_UI_BUTTON_BORDER,
    EMAIL_UI_RED_BUTTON_BORDER, EMAIL_UI_DATA_BACK_BORDER, EMAIL_UI_DATA_BOX_BORDER,
    EMAIL_UI_RIGHT_PANEL_BORDER, EMAIL_UI_INVENTORY_PANEL_BORDER, clean_email_ui_scale,
    EmailUiLayout, email_ui_layout, email_ui_layout_with_opening
};
use layout::sync_email_ui_layout;
pub use animation::{EMAIL_UI_CLIP_TITLE_RECT, EMAIL_UI_CLIP_BOX_RECT};
pub use types_email_ui_text_style::{
    EmailOutgoingItem, EmailSystemTime, EmailSummary, EmailGuideMessage, EmailBuddy,
    EmailComposeDraft, EmailScreen, EmailFolder, EmailPopup, EmailComposeFocus, EmailUiOutbox,
    EmailTransportOutbox, EmailNetworkRuntime0104, EmailNetworkInbox0104, EmailCloseSource,
    EmailUiRoot, EmailUiBackground, EmailUiLeftBackplate, EmailUiRightBackplate,
    EmailUiListPanel, EmailUiComposePanel, EmailUiRightPanel, EmailUiBuddyRow,
    EmailUiBuddyListContent, EmailUiRowAttachment, EmailUiTextRole, EmailUiTextElement,
    EmailUiTextStyle, EmailUiGuideIcon, EmailUiGuideIconShift, EmailUiAttachmentSlot
};
use types_email_ui_text_style::EmailUiAssets;
pub use types_email_ui_plugin::{EmailUiSet, EmailUiPlugin};
pub use commands::{
    EmailRequest, EmailReply, EmailUiAction, request_email_close, apply_email_reply
};
pub use models::EmailUiModel;
use models::sync_email_ui_model;
use operations_flush_email_transport_outbox::{
    email_system_message, exit_email_ui, send_failure_message, receive_item_failure_message,
    receive_cash_failure_message, truncate_utf16, truncate_legacy_list_text, utf16_units,
    flush_email_transport_outbox_0104, consume_email_network_inbox_0104, rebuild_email_buddy_rows,
    pressed_email_item_source, first_free_email_attachment_slot, exactly_one, email_fallback_text,
    email_passthrough_text, email_player_label_text
};
pub use operations_flush_email_transport_outbox::{
    open_email_ui, switch_email_folder, select_email_row, change_email_page,
    begin_email_compose, select_email_buddy, commit_email_calculator, close_email_compose,
    send_composed_email, accept_email_item, accept_all_email_items, accept_email_cash,
    confirm_delete_email, email_compose_open_fraction
};
use operations_email_summary_sender_text::{
    email_summary_sender_text, email_recipient_text, email_day_text, email_page_number_text,
    email_detail_taros_text, email_compose_taros_text, email_postage_amount_text,
    email_calculator_value_text, email_calculator_digit_text
};
#[cfg(test)]
use operations_email_summary_sender_text::email_text;
pub use audio::{EmailUiAudioCue, EmailUiAudioOutbox};
pub use view_email_list_panel::{attach_email_inventory_item, detach_email_inventory_item};
use view_email_list_panel::spawn_email_ui;
#[cfg(test)]
use view_email_list_panel::spawn_email_detail;
use view_email_calculator_popup::{
    spawn_email_buddy_row, spawn_email_calculator_popup, spawn_email_right_panel, spawn_text,
    spawn_text_with, spawn_text_entity, spawn_button, spawn_hit_button, spawn_image_button
};
use systems::{
    tick_email_opening, apply_group, sync_email_buddy_list_content,
    sync_email_guide_sender_icon, sync_email_buddy_row_visuals
};
use localization_email_localized_text::email_localized_text;
