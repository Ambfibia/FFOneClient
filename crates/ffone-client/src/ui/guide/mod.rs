//! Clean-Retrobution `cnGuideMode` UI/domain contract.
//!
//! Authority:
//! - clean `retrobution-20260613/main.unity3d`;
//! - `GuideMode` GameObject path ID 1285;
//! - `cnGuideMode` component path ID 1415;
//! - `FusionFallGuideSkin` path ID 1393;
//! - the clean native TableData publication
//!   `data/tables/xdt.json`.
//!
//! This module is deliberately passive. It owns the exact legacy layout,
//! labels, visual asset selection, confirmation state and typed UI outboxes.
//! The gameplay shell remains responsible for camera sub-targeting, cursor
//! ownership, help/system-message modes, packet encoding and applying server
//! replies. No runtime path below reads a Unity container or an `.ffclient`
//! cache.

use std::collections::VecDeque;

use crate::localization::{Language, Localization, LocalizationSet, LocalizedText};
use bevy::{
    prelude::*,
    sprite::BorderRect,
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::ui_support::legacy_screen_extent;

use crate::ui_support::valid_ui_scale as valid_scale;

use crate::ui_support::display_if as display;

use crate::ui_support::stretched_image as stretch_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod assets;
mod constants;
mod state;
mod layout;
mod frame;
mod textures;
mod interaction;
mod audio;
mod commands;
mod types;
mod models;
mod input;
mod operations;
mod localization_guide_static_localized_text;
mod systems;
mod view;

pub use assets::{
    GUIDE_UI_GAME_OBJECT_PATH_ID, GUIDE_UI_COMPONENT_PATH_ID, GUIDE_UI_SKIN_PATH_ID,
    GUIDE_UI_Z_INDEX, GUIDE_JEFFE_14_SOURCE_FONT_PATH_ID, GUIDE_JEFFE_16_SOURCE_FONT_PATH_ID,
    GUIDE_CHALET_SMALL_SOURCE_FONT_PATH_ID, GUIDE_SELECT_BACKGROUND_PATH,
    GUIDE_CHANGE_BACKGROUND_PATH, GUIDE_PANEL_PATH, GUIDE_MEMBER_BACK_PATH,
    GUIDE_SELECTED_EFFECT_PATH, GUIDE_ALERT_ICON_PATH, GUIDE_CONFIRM_ICON_PATH,
    GUIDE_COST_BAR_PATH, GUIDE_FUSION_MATTER_ICON_PATH, GUIDE_COMPUTRESS_ICON_PATH,
    GUIDE_DIALOG_PATH, GUIDE_TOGGLE_OFF_PATH, GUIDE_TOGGLE_ON_PATH, GUIDE_JEFFE_FONT_PATH,
    GUIDE_CHALET_FONT_PATH
};
pub use constants::{
    GUIDE_UI_SOURCE_BUILD, GUIDE_UI_PRIMARY_MAIN_BYTES, GUIDE_UI_PRIMARY_MAIN_SHA256,
    GUIDE_UI_PARITY_CAVEAT, GUIDE_JEFFE_14_FONT_SIZE, GUIDE_JEFFE_16_FONT_SIZE,
    GUIDE_CHALET_SMALL_FONT_SIZE, GUIDE_COMPUTRESS_TEXT_PADDING, GUIDE_CARD_TEXT_PADDING,
    GUIDE_CURRENT_TEXT_PADDING, GUIDE_LABEL_TEXT_PADDING, GUIDE_BIG_FONT_TEXT_PADDING,
    GUIDE_COMPUTRESS_LABEL, GUIDE_CHOOSE_HEADING, GUIDE_CHANGE_HEADING, GUIDE_CHOOSE_INTRO,
    GUIDE_CHANGE_INTRO, GUIDE_CURRENT_LABEL, GUIDE_COST_LABEL, GUIDE_CANCEL_LABEL,
    GUIDE_CONFIRM_LABEL, GUIDE_CONFIRM_TITLE, GUIDE_CHANGE_CONFIRM_TITLE, GUIDE_WARP_TITLE,
    GUIDE_WARP_BODY, GUIDE_ALREADY_CURRENT_MESSAGE_ID, GUIDE_CHANGE_FAILURE_MESSAGE_ID,
    GUIDE_ALREADY_CURRENT_MESSAGE, GUIDE_CHANGE_FAILURE_MESSAGE, GUIDE_CHANGE_PRICES,
    GUIDE_FIRST_CHANGE_WARP_NPC_TABLE_ID
};
pub use state::{GUIDE_UI_PARITY_STATUS, GUIDE_FIRST_CHANGE_NEXT_MODE, GuideUiSelectionWindow};
use state::{
    GuideUiMentorSelectedEffect, GuideUiSelectionHeading, GuideUiSelectionIntro,
    sync_guide_ui_selection
};
pub use layout::{
    GUIDE_UI_REFERENCE_HEIGHT, GUIDE_UI_SCALE_NUDGE, GUIDE_JEFFE_14_LINE_HEIGHT,
    GUIDE_JEFFE_16_LINE_HEIGHT, GUIDE_CHALET_SMALL_LINE_HEIGHT, GUIDE_CARD_BORDER,
    GUIDE_DIALOG_BORDER, GUIDE_BLUE_BUTTON_BORDER, GUIDE_CANCEL_BUTTON_BORDER,
    GUIDE_BACKGROUND_RECT, GUIDE_WINDOW_RECT, GUIDE_COMPUTRESS_ICON_RECT, GUIDE_HEADING_RECT,
    GUIDE_INTRO_RECT, GUIDE_CARD_RECT, GUIDE_COMMENT_RECT, GUIDE_TOGGLE_RECT,
    GUIDE_COST_BAR_RECT, GUIDE_PRIMARY_BUTTON_RECT, GUIDE_CANCEL_BUTTON_RECT,
    GUIDE_CLOSE_BUTTON_RECT, GUIDE_HELP_BUTTON_RECT, GUIDE_MODAL_RECT, GUIDE_MODAL_ICON_RECT,
    GUIDE_MODAL_CONFIRM_RECT, GUIDE_MODAL_CANCEL_RECT, GUIDE_WARP_BUTTON_RECT,
    GUIDE_MODAL_TEXT_RECT, GUIDE_MODAL_TITLE_RECT, GUIDE_MODAL_BODY_RECT, GuideUiRect,
    GuideScaledGroup, GuideUiLayout, clean_guide_ui_scale, guide_ui_layout,
    guide_cost_text_and_icon_rects
};
use layout::{sync_guide_ui_root_and_layout, sync_guide_ui_cost_geometry, apply_rect};
pub use frame::{
    GUIDE_CURRENT_FRAME_BORDER, GUIDE_COMPUTRESS_FRAME_BORDER, GUIDE_ICON_FRAME_BORDER,
    GUIDE_CARD_FRAME_PATH, GUIDE_CURRENT_FRAME_PATH, GUIDE_COMPUTRESS_FRAME_PATH,
    GUIDE_ICON_FRAME_PATH, GUIDE_COMPUTRESS_FRAME_RECT
};
use frame::GuideUiMentorCurrentFrame;
pub use textures::GUIDE_BLACK_TEXTURE_PATH;
pub use interaction::{
    GUIDE_BLUE_BUTTON_PATH, GUIDE_BLUE_BUTTON_OVER_PATH, GUIDE_CANCEL_BUTTON_PATH,
    GUIDE_CLOSE_BUTTON_PATH, GUIDE_CLOSE_BUTTON_OVER_PATH, GUIDE_HELP_BUTTON_PATH,
    GUIDE_HELP_BUTTON_OVER_PATH, GUIDE_CHOOSE_BUTTON_LABEL, GUIDE_CHANGE_BUTTON_LABEL,
    GUIDE_WARP_BUTTON_LABEL, GuideUiInputBoundary, guide_ui_button_enabled
};
use interaction::{
    GuideUiButtonVisual, GuideUiCommandButton, GuideUiButtonLabel, handle_guide_ui_interactions,
    sync_guide_ui_button_visuals, button_image, button_text_color, button_padding
};
pub use audio::{GUIDE_YES_SOUND_PATH, GUIDE_NO_SOUND_PATH, GuideUiAudioCue, GuideUiAudioOutbox};
pub use commands::{
    GUIDE_HELP_EVENT_RECEIVER, GUIDE_HELP_EVENT_GROUP, GUIDE_HELP_EVENT_FUNCTION,
    GuideUiCommand, GuideUiAction, apply_guide_ui_command
};
pub use types::{
    GuideMentor, GuideUiPurpose, GuideUiPhase, GuideUiBlocker, GuideUiDismissalSource,
    GuideUiOutbox, GuideChangeSuccess, GuideMentorView, GuideConfirmationView, GuideUiView,
    GuideUiRoot, GuideUiBackground, GuideUiWarpWindow, GuideUiConfirmationOverlay,
    GuideUiConfirmationWindow, GuideUiMentorCard, GuideUiTextRole, GuideUiTextElement,
    GuideUiSet, GuideUiPlugin
};
use types::{
    GuideUiAssets, GuideUiCostGroup, GuideUiCostText, GuideUiCostIcon, GuideUiConfirmationArt,
    GuideUiConfirmationTitle, GuideUiConfirmationBody
};
pub use models::GuideUiModel;
pub use input::{
    resolve_guide_change_success, resolve_correlated_guide_change_success,
    resolve_guide_change_failure
};
pub use operations::guide_ui_view;
use operations::{guide_confirmation_view, guide_passthrough_text, absolute_node};
pub use localization_guide_static_localized_text::GUIDE_MENTOR_NAME_LOCALIZATION_KEYS;
use localization_guide_static_localized_text::{
    guide_static_localized_text, guide_selection_heading_localized_text,
    guide_selection_intro_localized_text, guide_primary_button_localized_text,
    guide_cost_localized_text, guide_confirmation_title_localized_text,
    guide_confirmation_body_localized_text, guide_confirmation_body_localized_text_with_name,
    guide_localized_mentor_name, set_localized_text
};
use systems::sync_guide_ui_confirmation;
use view::spawn_guide_ui;
