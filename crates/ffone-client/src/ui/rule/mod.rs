//! Clean-Retrobution `eGameMode.Rule` UI and state contract.
//!
//! Primary authority is the unmodified `retrobution-20260613` build:
//! `main.unity3d` owns the inactive `Rule` GameObject (path ID 1278), its
//! `cnRule` component (path ID 1616), and `FusionFallRule` skin (path ID
//! 1381). The two reachable pages come from the clean converted RulesTable.
//! Their artwork is published from `Tutorial.resourceFile`,
//! `CharacterCreation.resourceFile`, and `Icons.resourceFile` under
//! `assets/game/ui/rule`.
//!
//! The plugin is intentionally a passive mode surface. It owns exact page
//! state, geometry, rendering, buttons, the Escape close-gate handshake, and
//! typed action/audio outboxes. The gameplay shell remains responsible for
//! `GameFrame` mode activation, global Help/system-popup state, cursor
//! application, event-bus routing, and localized button strings. No runtime
//! path opens a Unity container or an `.ffclient` cache.

use std::collections::VecDeque;

use crate::localization::{LocalizationSet, LocalizedText};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    sprite::BorderRect,
    text::{LineBreak, LineHeight},
    window::PrimaryWindow,
};

use crate::ui_support::legacy_screen_extent;

use crate::ui_support::valid_ui_scale as valid_scale;

use crate::ui_support::stretched_image as stretch_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod constants;
mod state;
mod assets;
mod textures;
mod interaction;
mod audio;
mod layout;
mod frame;
mod types;
mod validation;
mod operations;
mod models;
mod commands;
mod input_resolve_rule_ui_escape_close_gat;
mod localization_sync_rule_ui_localized_labels;
mod view;
mod systems;

pub use constants::{
    RULE_UI_SOURCE_BUILD, RULE_UI_ROOT_FONT_EXTERNAL_FILE_ID,
    RULE_UI_SCREEN_PIVOT_CENTER_VALUE, RULE_UI_ROOT_INITIALLY_ACTIVE, RULE_UI_MAIN_SHA256,
    RULE_UI_TUTORIAL_SHA256, RULE_UI_CHARACTER_CREATION_SHA256, RULE_UI_ICONS_SHA256,
    RULE_UI_PARITY_CAVEAT, RULE_UI_VEHICLE_IMAGE_PATHS, RULE_UI_COMBINE_IMAGE_PATHS,
    RULE_UI_IMAGE_PATHS, RULE_UI_JEFFE_12_FONT_SIZE, RULE_UI_JEFFE_16_FONT_SIZE,
    RULE_UI_CHALET_SMALL_FONT_SIZE, RULE_UI_CYAN_TEXT_COLOR, RULE_UI_BLUE_TEXT_COLOR,
    RULE_UI_BACK_NORMAL_TEXT_COLOR, RULE_UI_DISABLED_ALPHA, RULE_UI_BACK_LABEL_KEY,
    RULE_UI_PREVIOUS_LABEL_KEY, RULE_UI_NEXT_LABEL_KEY, RULE_UI_VEHICLE_STRINGS,
    RULE_UI_COMBINING_STRINGS, RULE_UI_VEHICLE_TEXT_KEYS, RULE_UI_COMBINING_TEXT_KEYS,
    RULE_UI_PAGES
};
pub use state::{RULE_UI_GAME_MODE_VALUE, RULE_UI_PARITY_STATUS};
pub use assets::{
    RULE_UI_GAME_OBJECT_PATH_ID, RULE_UI_COMPONENT_PATH_ID, RULE_UI_SCRIPT_PATH_ID,
    RULE_UI_SKIN_PATH_ID, RULE_UI_ROOT_FONT_EXTERNAL_PATH_ID, RULE_UI_JEFFE_12_FONT_PATH_ID,
    RULE_UI_JEFFE_16_FONT_PATH_ID, RULE_UI_CHALET_SMALL_FONT_PATH_ID,
    RULE_UI_INVENTORY_SKIN_PATH_ID, RULE_UI_PANEL_BACK_PATH, RULE_UI_RULE_BACK_PATH,
    RULE_UI_BACK_NORMAL_PATH, RULE_UI_NAV_NORMAL_PATH, RULE_UI_CLOSE_NORMAL_PATH,
    RULE_UI_JEFFE_FONT_PATH, RULE_UI_CHALET_FONT_PATH, RULE_UI_Z_INDEX
};
pub use textures::{
    RULE_UI_BACK_NORMAL_TEXTURE_PATH_ID, RULE_UI_BUTTON_NORMAL_TEXTURE_PATH_ID,
    RULE_UI_BUTTON_HOVER_TEXTURE_PATH_ID, RULE_UI_CLOSE_NORMAL_TEXTURE_PATH_ID,
    RULE_UI_CLOSE_HOVER_TEXTURE_PATH_ID
};
use textures::rgba;
pub use interaction::{
    RULE_UI_BACK_HOVER_PATH, RULE_UI_NAV_HOVER_PATH, RULE_UI_CLOSE_HOVER_PATH,
    RULE_UI_BUTTON_NORMAL_TEXT_COLOR, RULE_UI_BUTTON_HOVER_TEXT_COLOR, RuleUiInputBoundary,
    RuleUiButtonKind, activate_rule_ui_button, RuleUiButton
};
use interaction::{
    handle_rule_ui_keyboard, handle_rule_ui_interactions, sync_rule_ui_button_visuals
};
pub use audio::{
    RULE_UI_BUTTON_SOUND_PATHS, RULE_UI_BUTTON_SOUND_GAIN, RuleUiAudioCue, RuleUiAudioOutbox
};
pub use layout::{
    RULE_UI_REFERENCE_HEIGHT, RULE_UI_SCALE_NUDGE, RULE_UI_JEFFE_12_LINE_HEIGHT,
    RULE_UI_JEFFE_16_LINE_HEIGHT, RULE_UI_CHALET_SMALL_LINE_HEIGHT, RULE_UI_BUTTON_BORDER,
    RuleUiRect, RULE_UI_BACKGROUND_RECT, RULE_UI_WINDOW_RECT, RULE_UI_CLOSE_RECT,
    RULE_UI_BACK_RECT, RULE_UI_RULE_BACK_RECT, RULE_UI_TITLE_RECT, RULE_UI_SUBTITLE_1_RECT,
    RULE_UI_SUBTITLE_2_RECT, RULE_UI_CONTENT_1_RECT, RULE_UI_SUBTITLE_3_RECT,
    RULE_UI_CONTENT_2_RECT, RULE_UI_LAST_COMMENT_RECT, RULE_UI_PREVIOUS_RECT,
    RULE_UI_NEXT_RECT, RULE_UI_IMAGE_RECTS, RuleScaledGroup, RuleUiLayout,
    clean_rule_ui_scale, clean_rule_fit_scale, rule_ui_layout
};
use layout::sync_rule_ui_root_and_layout;
pub use frame::{RULE_UI_FRAME_RECT, RuleUiFrame};
pub use types::{
    RuleUiLabels, RulePageId, RulePageSpec, RuleUiDismissalSource, RuleUiOutbox, RuleUiRoot,
    RuleUiBackground, RuleUiWindow, RuleUiIllustration, RuleUiTextRole, RuleUiTextElement,
    RuleUiSet, RuleUiPlugin
};
use types::RuleUiAssets;
pub use validation::RulePageIndexError;
use operations::{finite_nonnegative, exit_rule_ui};
pub use models::RuleUiModel;
pub use commands::{RuleUiAction, request_rule_ui_escape_close};
pub use input_resolve_rule_ui_escape_close_gat::resolve_rule_ui_escape_close_gate;
use localization_sync_rule_ui_localized_labels::{
    rule_page_text_localized, rule_button_localized, sync_rule_ui_localized_labels
};
use view::spawn_rule_ui;
use systems::sync_rule_ui_page;
