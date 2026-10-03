//! Standalone clean-Retrobution `eGameMode::NanoFreeTuning` parity boundary.
//!
//! The behavioral authority is the clean `NanoFreeTuningMode` and
//! `CnGuiNanoFreeTuning` managed code owned by `retrobution-20260613/main.unity3d`.
//! The serialized authority is `sharedassets0.assets`: inactive root path ID
//! 1353, mode/UI components 1486/1487, `FusionFallNanoSkin` 1379, black texture
//! 270, three-power panel 283, and button textures 640/309.
//!
//! Network dispatch, authoritative Nano/inventory mutation, effect playback,
//! cinematic camera control, cursor ownership, and world entities remain typed
//! boundaries. This module never predicts a successful tune or mutates a Nano.

use std::{collections::VecDeque, error::Error, fmt};

use bevy::{
    asset::LoadState,
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    ui::{FocusPolicy, RelativeCursorPosition, widget::NodeImageMode},
    window::PrimaryWindow,
};

use crate::localization::{LocalizationSet, LocalizedText};

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod presentation_tests;

mod state;
mod containers;
mod assets;
mod codec;
mod commands;
mod constants;
mod audio;
mod layout;
mod output;
mod localization_nano_free_tuning_localized_text;
mod interaction;
mod types;
mod validation;
mod models;
mod operations;
mod view;
mod systems;

pub use state::NANO_FREE_TUNING_GAME_MODE_SLOT;
pub use containers::NANO_FREE_TUNING_OBJECT_NAME;
pub use assets::{
    NANO_FREE_TUNING_GAME_OBJECT_PATH_ID, NANO_FREE_TUNING_TRANSFORM_PATH_ID,
    NANO_FREE_TUNING_MODE_COMPONENT_PATH_ID, NANO_FREE_TUNING_UI_COMPONENT_PATH_ID,
    NANO_FREE_TUNING_MODE_SCRIPT_PATH_ID, NANO_FREE_TUNING_UI_SCRIPT_PATH_ID,
    NANO_FREE_TUNING_SKIN_PATH_ID, NANO_FREE_TUNING_UI_Z_INDEX, NANO_FREE_TUNING_PANEL_PATH,
    NANO_FREE_TUNING_BLACK_PATH, NANO_FREE_TUNING_SELECT_NORMAL_PATH,
    NANO_FREE_TUNING_FONT_PATH, NANO_FREE_TUNING_JEFFE_16_FONT_PATH_ID,
    NANO_FREE_TUNING_JEFFE_14_FONT_PATH_ID, NANO_FREE_TUNING_JEFFE_08_FONT_PATH_ID,
    NanoFreeTuningAssetContract, NANO_FREE_TUNING_ASSET_CONTRACTS,
    NanoFreeTuningPresentationAssetStatus
};
use assets::update_nano_free_tuning_asset_status;
pub use codec::{
    NANO_TUNE_REQUEST_PACKET_ID, NANO_TUNE_SUCCESS_PACKET_ID, NANO_TUNE_FAILURE_PACKET_ID,
    NanoTuneWireIntent
};
pub use commands::{
    NANO_TUNE_REQUEST_SIZE, NANO_TUNE_REQUEST_ABI, NanoTuneRequest, NanoFreeTuningReplyBody,
    NanoFreeTuningReplyEnvelope, NanoFreeTuningEffectIntent, NanoFreeTuningWorldIntent,
    NanoFreeTuningCinematicIntent, NanoFreeTuningUiIntent, NanoFreeTuningIntent,
    NanoFreeTuningPendingRequest, NanoFreeTuningUiCommand, NanoFreeTuningUiCommandOutbox
};
pub use constants::{
    NANO_TUNE_SUCCESS_SIZE, NANO_TUNE_FAILURE_SIZE, NANO_TUNE_ITEM_SLOT_COUNT,
    NANO_FREE_TUNING_EFFECT_ID, NANO_FREE_TUNING_BULLET_TYPES, NANO_FREE_TUNING_ECOM_ICON,
    NANO_FREE_TUNING_EFFECT_DELAY_SECONDS, NANO_FREE_TUNING_PROJECTILE_DELAY_SECONDS,
    NANO_FREE_TUNING_REVEAL_DELAY_SECONDS, NANO_FREE_TUNING_CAMERA_APPROACH_SECONDS,
    NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS, NANO_FREE_TUNING_IDLE_SECONDS,
    NANO_FREE_TUNING_RESULT_HIDE_SECONDS, NANO_FREE_TUNING_IDLE_RANDOM_EXCLUSIVE_MAX,
    NANO_FREE_TUNING_BLACK_BAR_RATIO, NANO_FREE_TUNING_UI_DEPTH, NANO_FREE_TUNING_PANEL_PIVOT,
    NANO_FREE_TUNING_JEFFE_16_FONT_SIZE, NANO_FREE_TUNING_JEFFE_14_FONT_SIZE,
    NANO_FREE_TUNING_JEFFE_08_FONT_SIZE, NANO_FREE_TUNING_BIGBLUE_REPLACEMENT_Y_OFFSET,
    NANO_FREE_TUNING_BIGYELLOW_REPLACEMENT_Y_OFFSET,
    NANO_FREE_TUNING_LIGHT_BLUE_REPLACEMENT_Y_OFFSET,
    NANO_FREE_TUNING_YELLOW_SMALL_REPLACEMENT_Y_OFFSET,
    NANO_FREE_TUNING_BLUE_REPLACEMENT_Y_OFFSET, NANO_FREE_TUNING_PRIMARY_MAIN_SIZE,
    NANO_FREE_TUNING_PRIMARY_MAIN_SHA256, NANO_FREE_TUNING_MANAGED_ASSEMBLY_SIZE,
    NANO_FREE_TUNING_MANAGED_ASSEMBLY_SHA256, NANO_TUNE_SUCCESS_ABI, NANO_TUNE_FAILURE_ABI
};
pub use audio::{
    NANO_FREE_TUNING_CREATION_SOUND, NANO_FREE_TUNING_SELECT_SOUND, NanoFreeTuningSoundIntent
};
pub use layout::{
    NANO_FREE_TUNING_PANEL_WIDTH, NANO_FREE_TUNING_PANEL_HEIGHT,
    NANO_FREE_TUNING_PANEL_SOURCE_WIDTH, NANO_FREE_TUNING_PANEL_SOURCE_HEIGHT,
    NANO_FREE_TUNING_JEFFE_16_LINE_HEIGHT, NANO_FREE_TUNING_JEFFE_14_LINE_HEIGHT,
    NANO_FREE_TUNING_JEFFE_08_LINE_HEIGHT, NanoFreeTuningRect, NanoFreeTuningPowerLayout,
    NANO_FREE_TUNING_TITLE_RECT, NANO_FREE_TUNING_POWER_LAYOUTS,
    nano_free_tuning_black_bar_rects, nano_free_tuning_panel_rect,
    nano_free_tuning_dormant_panel_rect
};
use layout::sync_nano_free_tuning_layout;
pub use output::{
    NANO_FREE_TUNING_COPY_ACQUIRED, NANO_FREE_TUNING_COPY_SELECT_POWER,
    NANO_FREE_TUNING_COPY_SELECT, NANO_FREE_TUNING_COPY_BANG
};
pub use localization_nano_free_tuning_localized_text::{
    NANO_FREE_TUNING_ACQUIRED_LOCALIZATION_KEY, NANO_FREE_TUNING_TITLE_LOCALIZATION_KEY,
    NANO_FREE_TUNING_SELECT_LOCALIZATION_KEY, NANO_FREE_TUNING_BANG_LOCALIZATION_KEY,
    NANO_FREE_TUNING_NANO_NAME_LOCALIZATION_KEY, NANO_FREE_TUNING_POWER_NAME_LOCALIZATION_KEY,
    NANO_FREE_TUNING_POWER_TYPE_LOCALIZATION_KEY,
    NANO_FREE_TUNING_POWER_DESCRIPTION_LOCALIZATION_KEY
};
use localization_nano_free_tuning_localized_text::nano_free_tuning_localized_text;
pub use interaction::{
    NANO_FREE_TUNING_SELECT_HOVER_PATH, NANO_FREE_TUNING_BUTTON_HOVER_TEXT,
    NANO_FREE_TUNING_BUTTON_DISABLED_ALPHA, NANO_FREE_TUNING_BUTTON_REPLACEMENT_Y_OFFSET,
    NanoFreeTuningPowerButton
};
use interaction::{NanoFreeTuningButtonLabel, sync_nano_free_tuning_button_visuals};
pub use types::{
    NanoFreeTuningUiTextStyle, NanoFreeTuningIconLabelStyle, NanoFreeTuningAbiScalar,
    NanoFreeTuningAbiField, NanoFreeTuningPower, NanoFreeTuningContent,
    NanoFreeTuningTransform, NanoFreeTuningWorldSnapshot, NanoFreeTuningOpenContext,
    NanoTuneItemBase, NanoTuneSuccess, NanoTuneFailure, NanoFreeTuningProtocolFault,
    NanoFreeTuningFault, NanoFreeTuningPhase, NanoFreeTuningPresentationRoot,
    NanoFreeTuningPresentationPanel, NanoFreeTuningPresentationBar,
    NanoFreeTuningPresentationSet, NanoFreeTuningUiPlugin
};
use types::{
    NanoFreeTuningPresentationAssets, NanoFreeTuningAnnouncement, NanoFreeTuningPowerIcon,
    NanoFreeTuningText, NanoFreeTuningTextRole
};
pub use validation::{NanoFreeTuningContentError, NanoFreeTuningTransitionError};
pub use models::NanoFreeTuningModel;
pub use operations::nano_free_tuning_entry_first_use_condition;
use operations::queue_nano_free_tuning_controls;
use view::spawn_nano_free_tuning_presentation;
use systems::sync_nano_free_tuning_content;
