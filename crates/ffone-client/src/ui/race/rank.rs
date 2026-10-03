//! Clean-Retrobution `eGameMode::RaceRankMode` (17) parity boundary.
//!
//! The three declared EP-rank request/reply pairs are dormant in
//! `cnRaceRankMode`: `ReceivePacket` reads and discards the packet type.
//! Observable rankings are fetched with an HTTP form containing only
//! `PCUID` then `EP_ID`. This module therefore emits typed HTTP intents and
//! never invents a packet payload for the private-byte reply declarations.
//!
//! The location catalog is a semantic projection of clean TableData. It
//! preserves the two stable sorts (`SortIndex`, then positive EP id), WarpName
//! copy, WorldName `ZoneName` copy, and exact EP big/small textures.

use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    f32::consts::FRAC_PI_2,
};

use bevy::{
    asset::LoadState,
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use serde::Deserialize;

use crate::localization::{LocalizationSet, LocalizedText};

use super::{RACE_CHALET_FONT_PATH, RACE_JEFFE_FONT_PATH, RaceUiRect, color};

use crate::ui_support::stretched_image as stretch;

use crate::ui_support::sliced_image as sliced;

#[cfg(test)]
mod tests;

mod containers;
mod assets;
mod constants;
mod layout;
mod frame;
mod commands;
mod types;
mod textures;
mod localization_race_rank_localization_entries;
mod interaction;
mod validation;
mod input;
mod operations;
mod models;
mod state;
mod view;
mod systems;

pub use containers::{
    RACE_RANK_MODE_OBJECT_NAME, RACE_RANK_SERIALIZED_OBJECT_NAME,
    RACE_RANK_CAMERA_OBJECT_NAME, RaceRankSerializedStyleSource,
    RACE_RANK_SERIALIZED_STYLE_SOURCES
};
pub use assets::{
    RACE_RANK_GAME_OBJECT_PATH_ID, RACE_RANK_TRANSFORM_PATH_ID, RACE_RANK_COMPONENT_PATH_ID,
    RACE_RANK_SCRIPT_PATH_ID, RACE_RANK_SKIN_PATH_ID, RACE_RANK_CAMERA_PATH_ID,
    RACE_RANK_CAMERA_GAME_OBJECT_PATH_ID, RACE_RANK_CAMERA_TRANSFORM_PATH_ID,
    RACE_RANK_CAMERA_COMPONENT_PATH_ID, RACE_RANK_CAMERA_SCRIPT_PATH_ID,
    RACE_RANK_CATALOG_SCHEMA, RACE_RANK_CATALOG_PATH, RACE_RANK_CATALOG_SHA256,
    RACE_RANK_UI_Z_INDEX, RACE_RANK_BACKDROP_PATH, RACE_RANK_SHELL_PATH, RACE_RANK_TITLE_PATH,
    RACE_RANK_LOCATION_BACK_PATH, RACE_RANK_BLACK_PATH, RACE_RANK_LOCATION_ROW_PATH,
    RACE_RANK_LOCATION_SELECTED_PATH, RACE_RANK_SELECTION_OUTLINE_PATH,
    RACE_RANK_SELECTION_POINT_PATH, RACE_RANK_RIGHT_BACK_PATH, RACE_RANK_ROW_PATH,
    RACE_RANK_ROW_SELECTED_PATH, RACE_RANK_MY_BACK_PATH, RACE_RANK_SKY_PATH,
    RACE_RANK_HIGHLIGHT_PATH, RACE_RANK_TAB_BAR_PATH, RACE_RANK_TAB_BOX_PATH,
    RACE_RANK_TODAY_ACTIVE_PATH, RACE_RANK_TODAY_IDLE_PATH, RACE_RANK_WIDE_ACTIVE_PATH,
    RACE_RANK_WIDE_IDLE_PATH, RACE_RANK_PREVIOUS_PATH, RACE_RANK_NEXT_PATH,
    RACE_RANK_LEFT_ARROW_PATH, RACE_RANK_RIGHT_ARROW_PATH, RACE_RANK_CLOSE_PATH,
    RACE_RANK_HELP_PATH, RACE_RANK_STATIC_ASSET_PATHS, RACE_RANK_INVENTORY_SKIN_PATH_ID,
    RaceRankCatalogSource, RaceRankCatalog, RaceRankPresentationAssetStatus
};
use assets::update_race_rank_asset_status;
pub use constants::{
    RACE_RANK_CAMERA_NEAR, RACE_RANK_CAMERA_FAR, RACE_RANK_CAMERA_FOV,
    RACE_RANK_CAMERA_CULLING_LAYER, RACE_RANK_CAMERA_DISTANCE, RACE_RANK_CAMERA_YAW_DEGREES,
    RACE_RANK_CAMERA_TARGET_NAME, RACE_RANK_PACKETS_ARE_DORMANT,
    RACE_RANK_LEGACY_FALLBACK_URL, RACE_RANK_RETROBUTION_URL, RACE_RANK_HTTP_METHOD,
    RACE_RANK_HTTP_FIELD_PCUID, RACE_RANK_HTTP_FIELD_EP_ID, RACE_RANK_HTTP_POLL_SECONDS,
    RACE_RANK_LOCATION_COUNT, RACE_RANK_PAGE_SIZE, RACE_RANK_PERIOD_COUNT,
    RACE_RANK_TOP_COUNT, RACE_RANK_TITLE_KEY, RACE_RANK_LOCATIONS_KEY,
    RACE_RANK_LOCATION_LABEL_KEY, RACE_RANK_HEADER_RANK_KEY, RACE_RANK_HEADER_PLAYER_KEY,
    RACE_RANK_HEADER_TOP_SCORE_KEY, RACE_RANK_NO_SCORE_KEY, RACE_RANK_PERIOD_TODAY_KEY,
    RACE_RANK_PERIOD_WEEK_KEY, RACE_RANK_PERIOD_MONTH_KEY, RACE_RANK_PERIOD_ALL_TIME_KEY,
    RACE_RANK_BEST_TODAY_KEY, RACE_RANK_BEST_WEEK_KEY, RACE_RANK_BEST_MONTH_KEY,
    RACE_RANK_BEST_ALL_TIME_KEY, RACE_RANK_TOP_TODAY_KEY, RACE_RANK_TOP_WEEK_KEY,
    RACE_RANK_TOP_MONTH_KEY, RACE_RANK_TOP_ALL_TIME_KEY, RACE_RANK_NPC_NAME_KEY,
    RACE_RANK_LOCATION_NAME_KEY, RACE_RANK_AREA_NAME_KEY, RACE_RANK_ROW_NAME_KEY,
    RACE_RANK_ROW_AREA_KEY, RACE_RANK_PAGE_KEY, RACE_RANK_VALUE_KEY,
    RACE_RANK_PLAYER_VALUE_KEY, RACE_RANK_SCORE_VALUE_KEY, RACE_RANK_ROW_STRIDE,
    RACE_RANK_UNRESOLVED_BOUNDARIES
};
pub use layout::{
    RACE_RANK_CAMERA_HEIGHT, RACE_RANK_INVENTORY_WIDTH, RACE_RANK_INVENTORY_HEIGHT,
    RACE_RANK_LEFT_WIDTH, RACE_RANK_RIGHT_WIDTH, RACE_RANK_BACKDROP_RECT,
    RACE_RANK_LEFT_GROUP_RECT, RACE_RANK_RIGHT_GROUP_RECT, RACE_RANK_SHELL_RECT,
    RACE_RANK_TITLE_RECT, RACE_RANK_CAMERA_RECT, RACE_RANK_TITLE_COPY_RECT,
    RACE_RANK_NPC_COPY_RECT, RACE_RANK_LOCATION_BOX_RECT, RACE_RANK_LOCATION_BLACK_RECT,
    RACE_RANK_FIRST_ROW_RECT, RACE_RANK_PREVIOUS_RECT, RACE_RANK_NEXT_RECT,
    RACE_RANK_PAGE_RECT, RACE_RANK_LEFT_ARROW_RECT, RACE_RANK_RIGHT_ARROW_RECT,
    RACE_RANK_SELECTED_POINT_RECT, RACE_RANK_RIGHT_LOCATION_RECT, RACE_RANK_RIGHT_NAME_RECT,
    RACE_RANK_RIGHT_AREA_RECT, RACE_RANK_BIG_IMAGE_RECT, RACE_RANK_HEADER_RANK_RECT,
    RACE_RANK_HEADER_PLAYER_RECT, RACE_RANK_HEADER_SCORE_RECT, RACE_RANK_MY_BLACK_RECT,
    RACE_RANK_TOP_BLACK_RECT, RACE_RANK_MY_ZERO_RECT, RACE_RANK_TOP_ZERO_RECT,
    RACE_RANK_TAB_GROUP_RECT, RACE_RANK_TAB_BOX_RECT, RACE_RANK_MY_SKY_RECT,
    RACE_RANK_TOP_SKY_RECT, RACE_RANK_MY_BEST_RECT, RACE_RANK_TOP_COPY_RECT,
    RACE_RANK_WARNING_RECT, RACE_RANK_HIGHLIGHT_RECT, RACE_RANK_TODAY_VISUAL_RECT,
    RACE_RANK_WEEK_VISUAL_RECT, RACE_RANK_MONTH_VISUAL_RECT, RACE_RANK_ALL_VISUAL_RECT,
    RACE_RANK_TODAY_HIT_RECT, RACE_RANK_WEEK_HIT_RECT, RACE_RANK_MONTH_HIT_RECT,
    RACE_RANK_ALL_HIT_RECT, RACE_RANK_TAB_BAR_RECT, RaceRankLayout, race_rank_layout,
    race_rank_layout_samples
};
use layout::sync_race_rank_layout;
pub use frame::{
    RACE_RANK_GET_LIST_REQUEST_PACKET_ID, RACE_RANK_GET_DETAIL_REQUEST_PACKET_ID,
    RACE_RANK_GET_PC_INFO_REQUEST_PACKET_ID, RACE_RANK_GET_LIST_REPLY_PACKET_ID,
    RACE_RANK_GET_DETAIL_REPLY_PACKET_ID, RACE_RANK_GET_PC_INFO_REPLY_PACKET_ID,
    RACE_RANK_ICON_FRAME_PATH
};
pub use commands::{
    RACE_RANK_GET_LIST_REQUEST_SIZE, RACE_RANK_GET_DETAIL_REQUEST_SIZE,
    RACE_RANK_GET_PC_INFO_REQUEST_SIZE, RACE_RANK_GET_LIST_REQUEST_ABI,
    RACE_RANK_GET_DETAIL_REQUEST_ABI, RACE_RANK_GET_PC_INFO_REQUEST_ABI, RaceRankHttpIntent,
    RaceRankUiCommand, RaceRankUiCommandOutbox
};
pub use types::{
    RaceRankDeclaredAbiField, RaceRankLocation, RaceRankPeriod, RaceRankScore, RaceRankScores,
    RaceRankEffect, RaceRankOutput, RaceRankOpenContext, RaceRankPhase,
    RaceRankPresentationRoot, RaceRankPresentationLeft, RaceRankPresentationRight,
    RaceRankPresentationNpcCameraSlot, RaceRankPresentationLocationRow,
    RaceRankPresentationScoreRow, RaceRankPresentationSet, RaceRankUiPlugin
};
use types::{
    RaceRankPresentationAssets, RaceRankPresentationBackdrop, RaceRankPresentationShell,
    RaceRankPresentationControl, RaceRankControl, RaceRankPresentationRowPart,
    RaceRankRowPart, RaceRankPresentationPersonalText, RaceRankPresentationTopText,
    RaceRankScoreColumn, RaceRankPresentationTabVisual
};
pub use textures::{
    RACE_RANK_PNG_SET_SHA256, RACE_RANK_TAB_TEXTURE_PATH, RACE_RANK_DIRECT_TEXTURE_PATH_IDS
};
pub use localization_race_rank_localization_entries::{
    RaceRankLocalizationEntry, RACE_RANK_LOCALIZATION_ENTRIES
};
use localization_race_rank_localization_entries::{
    localized_fallback, race_rank_location_name_localized, race_rank_area_name_localized,
    race_rank_npc_name_localized, race_rank_row_localized, race_rank_score_localized
};
pub use interaction::{
    RACE_RANK_TODAY_HOVER_PATH, RACE_RANK_WIDE_HOVER_PATH, RACE_RANK_PREVIOUS_HOVER_PATH,
    RACE_RANK_NEXT_HOVER_PATH, RACE_RANK_LEFT_ARROW_HOVER_PATH,
    RACE_RANK_RIGHT_ARROW_HOVER_PATH, RACE_RANK_CLOSE_HOVER_PATH, RACE_RANK_HELP_HOVER_PATH,
    RACE_RANK_INVENTORY_CLOSE_HOVER_PATH_ID, RaceRankPresentationInput
};
pub use validation::{
    RaceRankCatalogError, RaceRankParseError, RaceRankOpenError, RaceRankTransitionError,
    RaceRankCompletionError
};
pub use input::parse_clean_rank_response;
use input::{find_score_open, parse_score_tag};
use operations::{
    section, score_tags, ordered_attribute, row_text, rank_text, queue_race_rank_controls
};
pub use models::RaceRankModel;
use state::{RaceRankPresentationSelectedPart, RaceRankSelectedPart, sync_race_rank_selected};
use view::spawn_race_rank_presentation;
use systems::{
    advance_race_rank_slide, sync_race_rank_rows, sync_race_rank_scores, sync_race_rank_tabs,
    sync_race_rank_control_visuals
};
