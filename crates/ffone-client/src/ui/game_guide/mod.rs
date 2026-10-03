//! Native owner for clean Retrobution's `cnHelpMode` / `cnGuiHelp` overlay.
//!
//! NanoCom's `GAME GUIDE` event `(2, 18)` belongs to `GameFrame` and opens
//! this help overlay. It is unrelated to game mode 20 (`cnGuideMode`), which
//! owns mentor selection and changing.

use bevy::{
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use serde_json::Value;

use crate::{
    gameplay_audio::GameplayAudioRuntime,
    guide_ui::{GuideUiRect, clean_guide_ui_scale},
    localization::{LocalizationSet, LocalizedText},
};

#[cfg(test)]
mod tests;

mod assets;
mod interaction;
mod layout;
mod constants;
mod types;
mod models;
mod state;
mod view;
mod operations;
mod systems;

pub use assets::{
    GAME_GUIDE_BACKDROP_PATH, GAME_GUIDE_BACKGROUND_PATH, GAME_GUIDE_CLOSE_PATH,
    GAME_GUIDE_CLOSE_OVER_PATH, GAME_GUIDE_TITLE_BAR_PATH, GAME_GUIDE_JEFFE_FONT_PATH,
    GAME_GUIDE_CHALET_FONT_PATH
};
use assets::{GameGuideCatalog, screenshot_asset_path};
pub use interaction::{
    GAME_GUIDE_HELP_BUTTON_PATH, GAME_GUIDE_HELP_BUTTON_OVER_PATH, GAME_GUIDE_SCROLL_UP_PATH,
    GAME_GUIDE_SCROLL_DOWN_PATH, GAME_GUIDE_SCROLL_TRACK_PATH, GAME_GUIDE_SCROLL_THUMB_PATH,
    GAME_GUIDE_NAV_BUTTON_PATH, GAME_GUIDE_NAV_BUTTON_OVER_PATH,
    GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET, GAME_GUIDE_NAV_BUTTON_TEXT_Y_OFFSET
};
use interaction::{
    LEGACY_SCROLL_STEP, GameGuideButtonVisual, GameGuideScrollbar, GameGuideScrollThumb,
    GameGuideButtonLabel, help_button, navigation_button, sync_game_guide_scroll,
    sync_game_guide_button_visuals
};
pub use layout::{
    GAME_GUIDE_WINDOW_RECT, GAME_GUIDE_TITLE_RECT, GAME_GUIDE_TOPIC_PROMPT_RECT,
    GAME_GUIDE_MAIN_SCROLL_RECT, GAME_GUIDE_MAIN_BUTTON_RECT, GAME_GUIDE_SUB_BUTTON_RECT,
    GAME_GUIDE_MAIN_TITLE_RECT, GAME_GUIDE_CLOSE_RECT, GAME_GUIDE_PREVIOUS_RECT,
    GAME_GUIDE_NEXT_RECT, GAME_GUIDE_CONTENT_VIEW_RECT, GAME_GUIDE_CONTENT_RECT,
    GAME_GUIDE_SCREENSHOT_RECT
};
use layout::{CONTENT_VIEW_HEIGHT, rect_node, sync_game_guide_layout};
pub use constants::{GAME_GUIDE_INITIAL_MAIN_TOPIC, GAME_GUIDE_INITIAL_SUB_TOPIC};
use constants::{MAX_MAIN_TOPICS, MAX_SUB_TOPICS, CONTENT_GAP, LABEL_CYAN, BODY_BLUE};
use types::{
    HelpRange, HelpContent, GameGuideControl, GameGuideMainLabel, GameGuideSubLabel,
    GameGuideWindow, GameGuideBackdrop, GameGuideContentStack, GameGuideAssets
};
pub use types::{GameGuideUiRoot, GameGuideUiPlugin};
pub use models::GameGuideUiModel;
use state::GameGuideSelectedHeading;
use view::{spawn_text, spawn_game_guide_ui};
use operations::{handle_game_guide_controls, rebuild_game_guide_content};
#[cfg(test)]
use operations::current_page_id;
use systems::sync_game_guide_labels;
