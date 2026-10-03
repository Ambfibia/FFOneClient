//! Clean-Retrobution `cnUpsell` UI/domain contract.
//!
//! Authority:
//! - clean `retrobution-20260613/main.unity3d`;
//! - `upsell` GameObject path ID 1313;
//! - `cnUpsell` component path ID 1615 and script path ID 1146;
//! - `FusionFallUpsell` skin path ID 1383;
//! - fully read clean `cnUpsell::{ReceiveInit,Update,OnGUI,DoUpgradeGUI,
//!   DoNewsGUI,InitUpgradeGUI,InitNewsGUI}`.
//!
//! The clean `ReceiveInit` entry is intentionally narrower than the class:
//! it always enters News mode `1` or `2` from the camera pay-zone flag. Mode
//! `0` (`Upgrade`) remains in `guiModes`/`initGuiFuncs`, but has no clean
//! `ReceiveInit` or other live callsite; native callers must opt into it via
//! [`UpsellUiModel::open_retained_upgrade`]. `DoLeaveFutureGUI` and
//! `InitLeaveFutureGUI` are even less reachable: they are not registered in
//! either three-entry function array and have no numeric mode. This module
//! therefore exposes neither a fabricated LeaveFuture mode nor a TimeWarp
//! action.
//!
//! News pages stay blank until an owning provider injects native asset paths.
//! The unused local `upsellpage` texture is not a fallback, and the dead
//! `bPlaySound` field does not justify Computress autoplay. Presentation,
//! exact source geometry, page state and typed FIFO actions/audio are owned
//! here; external page fetching, pay-page side effects, cursor restoration and
//! the legacy event-bus Escape guards remain shell boundaries.

use std::collections::VecDeque;

use crate::localization::{LocalizationSet, LocalizedText};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    sprite::BorderRect,
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::ui_support::legacy_screen_extent;

use crate::ui_support::valid_ui_scale;

use crate::ui_support::stretched_image as stretch_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod constants;
mod assets;
mod interaction;
mod audio;
mod layout;
mod state;
mod types;
mod validation;
mod models;
mod operations;
mod commands;
mod view;
mod systems;
mod localization_upsell_label_localized;

pub use constants::{
    UPSELL_SOURCE_BUILD, UPSELL_SCREEN_PIVOT_CENTER_VALUE, UPSELL_LEVEL_IMAGE_PATHS,
    UPSELL_PAGE_FADE_PER_SECOND, UPSELL_PAGE_CHANGE_START_ALPHA, UPSELL_JEFFE_14_FONT_SIZE,
    UPSELL_JEFFE_12_FONT_SIZE, UPSELL_CONTINUE_NORMAL_TEXT_COLOR,
    UPSELL_NOT_NOW_NORMAL_TEXT_COLOR, UPSELL_NOT_NOW_ACTIVE_TEXT_COLOR,
    UPSELL_CONTINUE_PLAYING_LABEL, UPSELL_NOT_RIGHT_NOW_LABEL, UPSELL_CONTINUE_LABEL,
    UPSELL_NEWS_REACHABILITY, UPSELL_UPGRADE_REACHABILITY, UPSELL_LEAVE_FUTURE_REACHABILITY
};
pub use assets::{
    UPSELL_GAME_OBJECT_PATH_ID, UPSELL_COMPONENT_PATH_ID, UPSELL_SCRIPT_PATH_ID,
    UPSELL_SKIN_PATH_ID, UPSELL_CLOSE_NORMAL_PATH_ID, UPSELL_GET_NORMAL_PATH_ID,
    UPSELL_CONTINUE_NORMAL_PATH_ID, UPSELL_NOT_NOW_NORMAL_PATH_ID, UPSELL_ADVERTIS_PATH_ID,
    UPSELL_JEFFE_14_FONT_PATH_ID, UPSELL_JEFFE_12_FONT_PATH_ID, UPSELL_PANELBACK_PATH,
    UPSELL_CLOSE_NORMAL_PATH, UPSELL_GET_NORMAL_PATH, UPSELL_CONTINUE_NORMAL_PATH,
    UPSELL_NOT_NOW_NORMAL_PATH, UPSELL_ADVERTIS_PATH, UPSELL_FONT_PATH, UPSELL_UI_Z_INDEX
};
use assets::valid_news_page_path;
pub use interaction::{
    UPSELL_CLOSE_HOVER_PATH_ID, UPSELL_GET_HOVER_PATH_ID, UPSELL_CONTINUE_HOVER_PATH_ID,
    UPSELL_NOT_NOW_HOVER_PATH_ID, UPSELL_NEWS_BUTTON_NORMAL_PATH_ID,
    UPSELL_NEWS_BUTTON_HOVER_PATH_ID, UPSELL_CLOSE_HOVER_PATH, UPSELL_GET_HOVER_PATH,
    UPSELL_CONTINUE_HOVER_PATH, UPSELL_NOT_NOW_HOVER_PATH, UPSELL_NEWS_BUTTON_NORMAL_PATH,
    UPSELL_NEWS_BUTTON_HOVER_PATH, UPSELL_NEWS_BUTTON_NORMAL_TEXT_COLOR,
    UPSELL_NEWS_BUTTON_HOVER_TEXT_COLOR, UPSELL_NEWS_BUTTON_ACTIVE_TEXT_COLOR,
    UPSELL_CONTINUE_HOVER_TEXT_COLOR, UPSELL_NOT_NOW_HOVER_TEXT_COLOR, UpsellInputBoundary,
    UpsellUiButtonKind, UpsellUiButton, upsell_button_enabled
};
use interaction::{
    UpsellUiButtonLabel, handle_upsell_keyboard, handle_upsell_interactions,
    sync_upsell_button_visuals, UpsellUiButtonVisual, button_visual, button_image, button_padding
};
pub use audio::{UPSELL_ACTION_SUCCESS_SOUND_PATH, UpsellUiAudioCue, UpsellUiAudioOutbox};
pub use layout::{
    UPSELL_REFERENCE_HEIGHT, UPSELL_UI_SCALE_NUDGE, UPSELL_JEFFE_14_LINE_HEIGHT,
    UPSELL_JEFFE_12_LINE_HEIGHT, UPSELL_NEWS_BUTTON_BORDER, UPSELL_CONTINUE_BORDER,
    UPSELL_NOT_NOW_BORDER, UPSELL_BACKGROUND_RECT, UPSELL_UPGRADE_DIALOG_RECT,
    UPSELL_NEWS_DIALOG_RECT, UPSELL_SERIALIZED_CLOSE_RECT, UPSELL_SERIALIZED_CANCEL_RECT,
    UPSELL_SERIALIZED_GET_RECT, UPSELL_SERIALIZED_NOT_NOW_RECT,
    UPSELL_SERIALIZED_ADVERTIS_RECT, UPSELL_LEVEL_ONE_CANCEL_RECT, UPSELL_LEVEL_ONE_GET_RECT,
    UPSELL_LEVEL_ONE_NOT_NOW_RECT, UPSELL_LEVEL_ONE_ADVERTIS_RECT,
    UPSELL_OTHER_LEVEL_CANCEL_RECT, UPSELL_OTHER_LEVEL_GET_RECT,
    UPSELL_OTHER_LEVEL_NOT_NOW_RECT, UPSELL_OTHER_LEVEL_ADVERTIS_RECT, UPSELL_NEWS_CLOSE_RECT,
    UPSELL_NEWS_CONTINUE_RECT, UPSELL_NEWS_HIDDEN_GET_RECT, UPSELL_NEWS_HIDDEN_ADVERTIS_RECT,
    UpsellUiRect, UpsellScaledGroup, UpsellUiLayout, clean_upsell_ui_scale, upsell_ui_layout,
    UpsellUpgradeGeometry, upsell_upgrade_geometry
};
use layout::{sync_upsell_root_and_layout, sync_upsell_element_geometry, button_rect};
pub use state::UpsellUiMode;
pub use types::{
    UpsellSourceReachability, UpsellUiOutbox, UpsellUiRoot, UpsellUiBackdrop, UpsellUiDialog,
    UpsellUiAdvertis, UpsellUiSet, UpsellUiPlugin
};
use types::UpsellUiAssets;
pub use validation::UpsellUiError;
pub use models::UpsellUiModel;
use operations::{valid_level, continue_news};
pub use operations::dismiss_upsell_with_escape;
pub use commands::{UpsellUiCommand, UpsellUiAction, apply_upsell_ui_command};
use view::spawn_upsell_ui;
use systems::advance_upsell_page_fade;
use localization_upsell_label_localized::upsell_label_localized;
