//! Passive clean-Retrobution buddy list and interaction boundary.
//!
//! The presentation authority is `CnGuiChat` plus
//! `cnBuddySystemManager` from the clean `retrobution-20260613` build.
//! The module intentionally stops at typed semantic intents. It does not
//! import protocol packet IDs, dispatch network traffic, or guess the
//! callback contract used by the legacy `GameObject.SendMessage` paths.

use std::{
    collections::{BTreeSet, VecDeque},
    error::Error,
    fmt,
};

use bevy::{
    input::{ButtonState, InputSystems, keyboard::KeyboardInput, mouse::{MouseScrollUnit, MouseWheel}},
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::{
    gameplay_ui::{GameplayUiAudioCue, GameplayUiAudioOutbox},
    localization::{LocalizationSet, LocalizedText},
};

use crate::ui_support::valid_ui_scale;

use crate::ui_support::sliced_image;

use crate::ui_support::stretched_image;

#[cfg(test)]
mod tests;

mod constants;
mod state;
mod layout;
mod interaction;
mod assets;
mod textures;
mod validation;
mod types;
mod commands;
mod models;
mod operations;
mod view;
mod localization_buddy_static_localized;
mod systems;

pub use constants::{
    BUDDY_MAX_SLOTS, BUDDY_WARP_COOLDOWN_SECONDS, BUDDY_LIST_ROW_STEP, BUDDY_UI_SOURCE_BUILD,
    BUDDY_UI_PRIMARY_MAIN_BYTES, BUDDY_UI_PRIMARY_MAIN_SHA256, BUDDY_JEFFE_14_FONT_SIZE,
    BUDDY_JEFFE_12_FONT_SIZE, BUDDY_CHALET_SMALL_FONT_SIZE, BUDDY_WINDOW_TEXT_Y_OFFSET,
    BUDDY_ITEM_TEXT_Y_OFFSET, BUDDY_TRANSPARENT3_TEXT_Y_OFFSET, BUDDY_DELETE_TEXT_Y_OFFSET,
    BUDDY_CANCEL_TEXT_Y_OFFSET, BUDDY_LIST_TITLE, BUDDY_DELETE_LABEL, BUDDY_WARP_LABEL,
    BUDDY_ADD_LABEL, BUDDY_CANCEL_LABEL, BUDDY_ADD_TITLE, BUDDY_ADD_INSTRUCTION
};
pub use state::{BUDDY_STATE_REFRESH_SECONDS, BuddyStateUpdate};
pub use layout::{
    BUDDY_LIST_ROW_HEIGHT, BUDDY_JEFFE_14_LINE_HEIGHT, BUDDY_JEFFE_12_LINE_HEIGHT,
    BUDDY_CHALET_SMALL_LINE_HEIGHT, BUDDY_WINDOW_BORDER, BUDDY_LIST_BACKGROUND_BORDER,
    BUDDY_LARGE_LIST_BACKGROUND_BORDER, BUDDY_SELECT_BORDER, BUDDY_BUTTON_BORDER,
    BUDDY_SCROLL_TRACK_BORDER, BuddyUiRect, BUDDY_GROUP_RECT, BUDDY_WINDOW_RECT,
    BUDDY_CONTENT_RECT, BUDDY_INNER_LIST_RECT, BUDDY_DELETE_RECT, BUDDY_WARP_RECT,
    BUDDY_ADD_RECT, BUDDY_FREECHAT_RECT, BUDDY_SCROLL_UP_RECT, BUDDY_SCROLL_TRACK_RECT,
    BUDDY_SCROLL_DOWN_RECT, BUDDY_SCROLL_THUMB_WIDTH, BUDDY_SCROLL_THUMB_MIN_HEIGHT,
    BUDDY_ADD_WINDOW_RECT, BUDDY_ADD_TITLE_RECT, BUDDY_ADD_INSTRUCTION_RECT,
    BUDDY_ADD_TEXT_FIELD_RECT, BUDDY_ADD_CANCEL_RECT, BUDDY_ADD_SUBMIT_RECT,
    BuddyScrollbarLayout, buddy_scrollbar_layout, BuddyInviteDisposition, BuddyPanelLayout,
    buddy_panel_layout, buddy_panel_layout_for_chat_width, BuddyAddDialogLayout,
    buddy_add_dialog_layout
};
pub(crate) use layout::buddy_text_layout;
use layout::{buddy_rect_contains, bind_buddy_layout};
pub use interaction::{
    BUDDY_SCROLL_VELOCITY, BUDDY_BLUE_BUTTON_PATH, BUDDY_BLUE_BUTTON_OVER_PATH,
    BUDDY_RED_BUTTON_PATH, BUDDY_RED_BUTTON_OVER_PATH, BUDDY_CANCEL_BUTTON_PATH,
    BUDDY_SCROLL_TRACK_PATH, BUDDY_SCROLL_THUMB_PATH, BUDDY_SCROLL_UP_PATH,
    BUDDY_SCROLL_DOWN_PATH, BUDDY_BLUE_BUTTON_TEXT_Y_OFFSET, BUDDY_RED_BUTTON_TEXT_Y_OFFSET,
    BUDDY_QUIT_BUTTON_TEXT_Y_OFFSET, BuddyScrollbarPart
};
use interaction::{
    BuddyAddInputText, BuddyKeyboardCapture, BuddyScrollbarDrag, begin_buddy_keyboard_capture,
    handle_buddy_keyboard, clear_captured_buddy_keyboard_messages, handle_buddy_scroll_input,
    handle_buddy_interactions
};
pub use assets::{
    BUDDY_UI_Z_INDEX, BUDDY_MODAL_Z_INDEX, BUDDY_GAME_OBJECT_PATH_ID, BUDDY_BOX_PATH,
    BUDDY_SELECT_PATH, BUDDY_FREECHAT_PATH, BUDDY_LIST_BACKGROUND_PATH,
    BUDDY_LARGE_LIST_BACKGROUND_PATH, BUDDY_ADD_DIALOG_PATH, BUDDY_ADD_OVERLAY_PATH,
    BUDDY_FONT_PATH, BUDDY_CHALET_FONT_PATH, BUDDY_JEFFE_14_SOURCE_FONT_PATH_ID,
    BUDDY_JEFFE_12_SOURCE_FONT_PATH_ID, BUDDY_CHALET_SMALL_SOURCE_FONT_PATH_ID,
    BUDDY_CHAT_SKIN_PATH_ID, BUDDY_POP_SKIN_PATH_ID, BUDDY_GUI_COMPONENT_PATH_ID
};
pub use textures::{
    BUDDY_LARGE_LIST_BACKGROUND_SOURCE_TEXTURE_PATH_ID,
    BUDDY_SCROLL_TRACK_SOURCE_TEXTURE_PATH_ID, BUDDY_SCROLL_THUMB_SOURCE_TEXTURE_PATH_ID,
    BUDDY_SCROLL_UP_SOURCE_TEXTURE_PATH_ID, BUDDY_SCROLL_DOWN_SOURCE_TEXTURE_PATH_ID
};
use textures::color_from_rgba;
pub use validation::{BUDDY_ADD_NAME_ERROR, BuddyUiError};
pub use types::{
    BuddyFontRole, BuddyTextAnchor, BuddyTextStyle, BuddyTextStyleSpec, BuddyChatWindowStyle,
    BuddyPresence, BuddyEntry, BuddyTarget, BuddyUiNotice, BuddyInvite, BuddyConfirmation,
    BuddyUiOutbox, BuddyRowView, BuddyUiView, BuddyUiRoot, BuddyUiSet, BuddyUiPlugin
};
use types::{
    BuddyUiAssets, BuddyPanel, BuddyListBackground, BuddyModalRoot, BuddyAddDialog, BuddyRow,
    BuddyRowPartKind, BuddyRowPart, BuddyControl, BuddyControlMarker
};
pub use commands::BuddyUiAction;
pub use models::BuddyUiModel;
use operations::{invite_response, buddy_text_font, bind_buddy_ui};
pub use operations::buddy_ui_view;
pub(crate) use operations::{buddy_text_color, buddy_text_node, buddy_text_transform};
use view::spawn_buddy_ui;
use localization_buddy_static_localized::{
    buddy_static_localized, buddy_row_localized, buddy_add_name_localized
};
use systems::{tick_buddy_ui, update_buddy_control_visuals};
