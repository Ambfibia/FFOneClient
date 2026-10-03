//! Clean-Retrobution `eGameMode::RaceMode` (16) parity boundary.
//!
//! `cnRaceMode` leaves the serialized start window unreachable: `OnGUI` is
//! guarded by `EcomType != StartEcom` and then tests `EcomType == StartEcom`.
//! `FailEcom` opens system message 159. The native lifecycle shows a separate
//! HUD while running and paints the result only after server success, avoiding
//! zero rewards and start sounds on rejected requests.
//!
//! Packet bytes, world-ring activation, ECom ownership, inventory mutation,
//! cursor ownership, camera sub-targets, sound, voice, item lookup, and mode
//! switching remain external. The pure model emits typed requests/effects
//! with the exact clean ABI instead of calling those systems itself.

use std::collections::VecDeque;

use bevy::{
    asset::LoadState,
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use ffone_protocol::{RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104};

use crate::localization::{LocalizationSet, LocalizedText};

use super::{RACE_CHALET_FONT_PATH, RACE_JEFFE_FONT_PATH, RaceUiRect, color};

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod state;
mod containers;
mod assets;
mod codec;
mod commands;
mod constants;
mod types;
mod validation;
mod layout;
mod interaction;
mod textures;
mod output;
mod localization_race_mode_localization_entries;
mod input;
mod audio;
mod models;
mod operations;
mod view;

pub use state::{
    RACE_MODE_GAME_MODE_ID, RACE_RANK_GAME_MODE_ID, RACE_RECORD_MODE, RACE_MODE_TIME_KEY,
    RACE_MODE_PODS_KEY, RACE_MODE_SCORE_KEY, RACE_MODE_MY_BEST_KEY, RACE_MODE_REWARD_KEY,
    RACE_MODE_ACCEPT_KEY, RACE_MODE_INVENTORY_FULL_KEY, RACE_MODE_TIME_VALUE_KEY,
    RACE_MODE_PODS_VALUE_KEY, RACE_MODE_SCORE_VALUE_KEY, RACE_MODE_FUSION_MATTER_KEY,
    RACE_MODE_ITEM_NAME_KEY, RACE_MODE_ITEM_LEVEL_KEY, RACE_MODE_RATING_NONE_KEY,
    RACE_MODE_RATING_GENIUS_KEY, RACE_MODE_RATING_AWESOME_KEY, RACE_MODE_RATING_GOOD_KEY,
    RACE_MODE_RATING_NOT_BAD_KEY, RACE_MODE_RATING_BLEH_KEY, RACE_MODE_UNRESOLVED_BOUNDARIES,
    RacePlayerState, RaceModeOpenContext, RaceModeReply, RaceModeEffect, RaceModeOutput,
    RaceModePhase, RaceModeUiCommand, RaceModeUiCommandOutbox, RaceModePresentationRoot,
    RaceModePresentationPanel, RaceModePresentationStar, RaceModePresentationSet,
    RaceModeUiPlugin
};
use state::{
    RaceModePresentationAssets, RaceModePresentationBar, RaceModePresentationText, RaceModeTextRole,
    RaceModeOptionalItemNode, RaceModeOptionalItemIcon, RaceModeFusionMatterNode
};
pub use containers::RACE_MODE_OBJECT_NAME;
pub use assets::{
    RACE_MODE_GAME_OBJECT_PATH_ID, RACE_MODE_TRANSFORM_PATH_ID, RACE_MODE_COMPONENT_PATH_ID,
    RACE_MODE_SCRIPT_PATH_ID, RACE_MODE_SKIN_PATH_ID, RACE_RESULT_UI_Z_INDEX,
    RACE_RESULT_BACKGROUND_PATH, RACE_RESULT_BLACK_PATH, RACE_RESULT_FUSION_MATTER_PATH,
    RACE_RESULT_ITEM_BAR_PATH, RACE_RESULT_STAR_PATH, RACE_RESULT_STAR_EMPTY_PATH,
    RaceModePresentationAssetStatus
};
use assets::update_race_mode_asset_status;
pub use codec::{
    RACE_START_REQUEST_PACKET_ID, RACE_END_REQUEST_PACKET_ID, RACE_CANCEL_REQUEST_PACKET_ID,
    RACE_START_SUCCESS_PACKET_ID, RACE_START_FAILURE_PACKET_ID, RACE_END_SUCCESS_PACKET_ID,
    RACE_END_FAILURE_PACKET_ID, RACE_CANCEL_SUCCESS_PACKET_ID, RACE_CANCEL_FAILURE_PACKET_ID,
    RACE_DORMANT_INVENTORY_FULL_PACKET_ID, RaceReplyCodecError0104, decode_race_reply_0104
};
pub use commands::{
    RACE_START_REQUEST_SIZE, RACE_END_REQUEST_SIZE, RACE_CANCEL_REQUEST_SIZE,
    RACE_START_REQUEST_ABI, RACE_END_REQUEST_ABI, RACE_CANCEL_REQUEST_ABI, RaceRequestKind,
    RaceRequestIntent, RaceReplyEnvelope
};
use commands::PendingRaceRequest;
pub use constants::{
    RACE_START_SUCCESS_SIZE, RACE_END_SUCCESS_SIZE, RACE_REWARD_ITEM_SIZE, RACE_TICKET_SLOT,
    RACE_START_SUCCESS_ABI, RACE_END_SUCCESS_ABI, RACE_START_ECOM_ICON, RACE_END_ECOM_ICON,
    RACE_ECOM_MANAGER_ID, RACE_ECOM_MAKE_FUNCTION, RACE_ECOM_DELETE_FUNCTION,
    RACE_FAIL_SYSTEM_MESSAGE_ID, RACE_FAIL_SYSTEM_MESSAGE_KEY, RACE_FATIGUE_MESSAGE_BOX_ID,
    RACE_FATIGUE_MESSAGE_BOX_TYPE, RACE_FIRST_USE_REWARD_CONDITION,
    RACE_FIRST_USE_LOW_ENERGY_CONDITION, RACE_LOW_ENERGY_WARNING, RACE_EMPTY_ENERGY_WARNING,
    RACE_RESULT_PIVOT, RACE_RESULT_STAR_COUNT, RACE_RESULT_TEXTURES, RACE_REWARD_ROW_STRIDE
};
pub use types::{
    RaceAbiScalar, RaceAbiField, RaceEcomType, RaceTopRecord, RaceNpcContext, RaceRewardItem,
    RaceEndSuccess, RaceResult, RaceEcomOperation, RaceRewardPresentation
};
pub use validation::{RACE_START_ERROR_8_MESSAGE_ID, RaceModeOpenError, RaceModeReplyError};
pub use layout::{
    RACE_RESULT_PANEL_WIDTH, RACE_RESULT_PANEL_HEIGHT, RACE_END_WINDOW_RECT, RACE_STAR_RECT,
    RACE_PERFECT_RECT, RACE_MY_BEST_RECT, RACE_TIME_RECT, RACE_TIME_DATA_RECT,
    RACE_PODS_DATA_RECT, RACE_SCORE_DATA_RECT, RACE_BEST_TIME_RECT, RACE_BEST_TIME_DATA_RECT,
    RACE_BEST_PODS_DATA_RECT, RACE_BEST_SCORE_DATA_RECT, RACE_REWARD_RECT, RACE_ITEM_BAR_RECT,
    RACE_ITEM_RECT, RACE_ACCEPT_RECT, RACE_INVENTORY_FULL_RECT
};
use layout::sync_race_mode_layout;
pub use interaction::{
    RACE_RESULT_BUTTON_NORMAL_PATH, RACE_RESULT_BUTTON_HOVER_PATH,
    RACE_RESULT_BUTTON_ACTIVE_PATH, RaceModePresentationInput
};
use interaction::{RaceModeAcceptButton, sync_race_mode_button_visual};
pub use textures::{RaceResultTextureRole, RaceResultTextureContract};
pub use output::{
    RACE_COPY_TIME, RACE_COPY_PODS, RACE_COPY_SCORE, RACE_COPY_MY_BEST, RACE_COPY_REWARD,
    RACE_COPY_LEVEL, RACE_COPY_INVENTORY_FULL, RACE_COPY_FUSION_MATTER, RACE_COPY_ACCEPT,
    race_time_copy, race_score_copy, race_rank_copy
};
use output::write_i32;
pub use localization_race_mode_localization_entries::{
    RaceModeLocalizationEntry, RACE_MODE_LOCALIZATION_ENTRIES, race_rank_localized,
    race_time_localized, race_pods_localized, race_score_localized,
    race_fusion_matter_localized, race_reward_item_name_localized,
    race_reward_item_level_localized
};
use localization_race_mode_localization_entries::{
    initial_race_mode_localized, localized_fallback
};
use input::{read_i16, read_i32, read_u64};
pub use audio::RaceSound;
pub use models::RaceModeModel;
pub use operations::race_filled_star_count;
use view::spawn_race_mode_presentation;
