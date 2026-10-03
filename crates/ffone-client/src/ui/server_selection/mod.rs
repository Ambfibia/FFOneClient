//! Standalone clean-Retrobution `eGameMode::ServerSelection` contract.
//!
//! `main.unity3d` is the parity authority for the inactive
//! `ServerSelectionMode` root, `CnServerSelectionMode`,
//! `CnGuiServerSelection`, `FusionFallSkin`, and every texture published by
//! this module. The managed and serialized dumps named below are offline
//! navigation derivatives of that raw owner; no runtime path opens a Unity
//! container or `.ffclient` cache.
//!
//! The source has deliberately small scope: one server, twenty-five channel
//! rows, a 60-second shard-list refresh, three external web/quit buttons, and
//! a server-select packet boundary. The oddities are retained: index zero of
//! the 26-byte shard array is ignored, closed channels remain selectable,
//! the initial scroll height is 402 before the first GUI pass, Escape has no
//! handler, My Account and Homepage open the same URL, and no named audio
//! cue is emitted by either source class. Clean Retrobution's fixed `US`
//! locale bypasses slot 26 entirely; only the dormant `KOREA` login branch
//! opens it. The production plugin therefore remains hidden unless an
//! explicit Korea-localization extension owns the route.

use std::collections::VecDeque;

use bevy::{
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    sprite::BorderRect,
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::localization::{LocalizationSet, LocalizedText};

use crate::ui_support::stretched_image as stretch_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod state;
mod localization_server_selection_localization_en;
mod assets;
mod layout;
mod frame;
mod interaction;
mod output;
mod operations;
mod models;
mod audio;
mod view;
mod types;

pub use state::{
    SERVER_SELECTION_SOURCE_BUILD, SERVER_SELECTION_SOURCE_ARCHIVE,
    SERVER_SELECTION_SOURCE_ARCHIVE_SIZE, SERVER_SELECTION_SOURCE_ARCHIVE_SHA256,
    SERVER_SELECTION_MANAGED_MODE_SHA256, SERVER_SELECTION_MANAGED_GUI_SHA256,
    SERVER_SELECTION_GAME_MODE_SLOT, SERVER_SELECTION_DEFAULT_US_REACHABLE,
    SERVER_SELECTION_ROOT_NAME, SERVER_SELECTION_ROOT_INITIALLY_ACTIVE,
    SERVER_SELECTION_SERVER_COUNT, SERVER_SELECTION_ALLOCATED_SERVER_RECORDS,
    SERVER_SELECTION_SHARD_ARRAY_LEN, SERVER_SELECTION_FIRST_SHARD,
    SERVER_SELECTION_LAST_SHARD, SERVER_SELECTION_REFRESH_SECONDS,
    SERVER_SELECTION_STATUS_MESSAGE_ID, SERVER_SELECTION_URL, SERVER_SELECTION_UI_DEPTH,
    SERVER_SELECTION_JEFFE_16_FONT_SIZE, SERVER_SELECTION_LABEL_TOP_PADDING,
    SERVER_SELECTION_JEFFE_14_FONT_SIZE, SERVER_SELECTION_PARITY_STATUS,
    SERVER_SELECTION_PARITY_CAVEAT, SERVER_SELECTION_HEADER_SERVER_CHANNEL_KEY,
    SERVER_SELECTION_HEADER_STATUS_KEY, SERVER_SELECTION_SERVER_COLLAPSED_KEY,
    SERVER_SELECTION_SERVER_EXPANDED_KEY, SERVER_SELECTION_CHANNEL_ROW_KEY,
    SERVER_SELECTION_STATUS_NONE_KEY, SERVER_SELECTION_STATUS_CLOSED_KEY,
    SERVER_SELECTION_STATUS_EMPTY_KEY, SERVER_SELECTION_STATUS_NORMAL_KEY,
    SERVER_SELECTION_STATUS_BUSY_KEY, SERVER_SELECTION_CONNECT_KEY,
    SERVER_SELECTION_MY_ACCOUNT_KEY, SERVER_SELECTION_HOMEPAGE_KEY, SERVER_SELECTION_QUIT_KEY,
    SERVER_SELECTION_EXTERNAL_CALL_FUNCTION, SERVER_SELECTION_EXTERNAL_CALL_ARGUMENT,
    SERVER_SELECTION_EXTERNAL_CALL_ARGUMENT_COUNT, SERVER_SELECTION_IMAGE_PATHS,
    SERVER_SELECTION_FIRST_CHANNEL_TOP, SERVER_SELECTION_CHANNEL_STRIDE,
    ServerSelectionPopulation, ServerSelectionLoginSnapshot, ServerSelectionInit,
    ServerSelectionPhase, ServerSelectionUiEffect, ServerSelectionWebSource,
    ServerSelectionUiOutbox, ServerSelectionUiAssets, ServerSelectionUiElement,
    ServerSelectionUiControl, ServerSelectionUiTextRole, ServerSelectionUiSet,
    ServerSelectionUiPlugin
};
pub use localization_server_selection_localization_en::{
    SERVER_SELECTION_US_LOCALE_VALUE, SERVER_SELECTION_KOREA_LOCALE_VALUE,
    ServerSelectionLocalizationEntry, SERVER_SELECTION_LOCALIZATION_ENTRIES,
    server_selection_server_heading_localized, server_selection_channel_localized
};
use localization_server_selection_localization_en::{
    initial_server_selection_localized, localized_fallback
};
pub use assets::{
    SERVER_SELECTION_ROOT_PATH_ID, SERVER_SELECTION_TRANSFORM_PATH_ID,
    SERVER_SELECTION_MODE_COMPONENT_PATH_ID, SERVER_SELECTION_GUI_COMPONENT_PATH_ID,
    SERVER_SELECTION_MODE_SCRIPT_PATH_ID, SERVER_SELECTION_GUI_SCRIPT_PATH_ID,
    SERVER_SELECTION_SKIN_PATH_ID, SERVER_SELECTION_UI_Z_INDEX, SERVER_SELECTION_FONT_PATH,
    SERVER_SELECTION_SKIN_LEDGER_PATH, SERVER_SELECTION_JEFFE_16_SOURCE_PATH_ID,
    SERVER_SELECTION_JEFFE_14_SOURCE_PATH_ID, SERVER_SELECTION_BACKGROUND_PATH,
    SERVER_SELECTION_PANEL_PATH, SERVER_SELECTION_INNER_PATH,
    SERVER_SELECTION_ROW_SELECTED_PATH, SERVER_SELECTION_RED_NORMAL_PATH,
    ServerSelectionAssetContract, SERVER_SELECTION_ASSET_CONTRACTS
};
pub use layout::{
    SERVER_SELECTION_INITIAL_SCROLL_HEIGHT, SERVER_SELECTION_COLLAPSED_SCROLL_HEIGHT,
    SERVER_SELECTION_EXPANDED_SCROLL_HEIGHT, SERVER_SELECTION_REFERENCE_BACKGROUND_WIDTH,
    SERVER_SELECTION_REFERENCE_BACKGROUND_HEIGHT, SERVER_SELECTION_JEFFE_16_LINE_HEIGHT,
    SERVER_SELECTION_JEFFE_14_LINE_HEIGHT, ServerSelectionUiRect,
    SERVER_SELECTION_PANEL_LOCAL_RECT, SERVER_SELECTION_INNER_RECT,
    SERVER_SELECTION_VIEWPORT_RECT, SERVER_SELECTION_CONNECT_RECT,
    SERVER_SELECTION_SERVER_ROW_RECT, SERVER_SELECTION_CHANNEL_STATUS_RECT,
    ServerSelectionUiLayout
};
use layout::bind_server_selection_layout;
pub use frame::{
    SERVER_SELECTION_REQ_SHARD_LIST_PACKET_ID, SERVER_SELECTION_REQ_SHARD_LIST_PACKET_SIZE,
    SERVER_SELECTION_REP_SHARD_LIST_PACKET_ID, SERVER_SELECTION_REP_SHARD_LIST_PACKET_SIZE,
    SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_ID,
    SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_SIZE
};
pub use interaction::{
    SERVER_SELECTION_ROW_HOVER_PATH, SERVER_SELECTION_BUTTON_NORMAL_PATH,
    SERVER_SELECTION_BUTTON_HOVER_PATH, SERVER_SELECTION_BUTTON_ACTIVE_PATH,
    SERVER_SELECTION_RED_HOVER_PATH, SERVER_SELECTION_SCROLL_TRACK_PATH,
    SERVER_SELECTION_SCROLL_THUMB_PATH, SERVER_SELECTION_SCROLL_UP_PATH,
    SERVER_SELECTION_SCROLL_DOWN_PATH, ServerSelectionKeyboardContract,
    SERVER_SELECTION_KEYBOARD_CONTRACT, SERVER_SELECTION_SCROLL_STEP,
    ServerSelectionInputBoundary
};
use interaction::{
    LegacyButtonKind, LegacyButtonLabel, centered_button_node, legacy_button_text_color,
    collect_server_selection_input, bind_server_selection_button_images
};
pub use output::{
    SERVER_SELECTION_COPY_HEADER_SERVER_CHANNEL, SERVER_SELECTION_COPY_HEADER_STATUS,
    SERVER_SELECTION_COPY_SERVER_COLLAPSED, SERVER_SELECTION_COPY_SERVER_EXPANDED,
    SERVER_SELECTION_COPY_CHANNEL_ROW, SERVER_SELECTION_COPY_STATUS_CLOSED,
    SERVER_SELECTION_COPY_STATUS_EMPTY, SERVER_SELECTION_COPY_STATUS_NORMAL,
    SERVER_SELECTION_COPY_STATUS_BUSY, SERVER_SELECTION_COPY_CONNECT,
    SERVER_SELECTION_COPY_MY_ACCOUNT, SERVER_SELECTION_COPY_HOMEPAGE,
    SERVER_SELECTION_COPY_QUIT
};
use operations::{finite_nonnegative, legacy_text_font, absolute_node, transparent_row_image};
pub use models::ServerSelectionUiModel;
pub use audio::SERVER_SELECTION_AUDIO_ROUTES;
use view::spawn_server_selection_ui;
use types::LegacyTextSize;
