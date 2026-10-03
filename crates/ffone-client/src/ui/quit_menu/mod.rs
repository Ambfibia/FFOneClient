//! Clean-Retrobution gameplay QuitMenu UI.
//!
//! The primary source is `main.unity3d` `QuitMenu` GameObject path ID 1346,
//! component path ID 1418, and `QuitMenuSkin` path ID 1394. `cnQuit.OnGUI`
//! renders a full-screen `back` style, one centered 206x188 `dlg`, and three
//! fixed buttons. This module preserves that presentation and emits typed
//! actions/audio cues without owning networking, application exit, gameplay
//! input, cursor state, or an audio mixer.
//!
//! `QuitMenuSkin.m_Font` points through external file ID 1/path ID 10102, but
//! clean `cnQuit.OnGUI` never draws text with that inherited root font: both
//! text-bearing styles used by the reachable surface (`Button` and
//! `CancelButton`) explicitly select `JEFFE___14` path ID 903. The external
//! font is therefore recorded as unused serialized evidence, not substituted
//! into the native UI.

mod document;

use std::collections::VecDeque;

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::localization::LocalizedText;

use crate::ui_support::legacy_screen_extent;

use crate::ui_support::valid_ui_scale;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod assets;
mod interaction;
mod state;
mod constants;
mod textures;
mod audio;
mod layout;
mod types;
mod models;
mod commands;
mod view;
mod operations;
mod systems;

pub use assets::{
    QUIT_MENU_GAME_OBJECT_PATH_ID, QUIT_MENU_COMPONENT_PATH_ID, QUIT_MENU_SKIN_PATH_ID,
    QUIT_MENU_ROOT_FONT_PATH_ID, QUIT_MENU_BACKDROP_PATH, QUIT_MENU_DIALOG_PATH,
    QUIT_MENU_CANCEL_NORMAL_PATH, QUIT_MENU_FONT_PATH, QUIT_MENU_UI_Z_INDEX
};
pub use interaction::{
    QUIT_MENU_BUTTON_FONT_PATH_ID, QUIT_MENU_BUTTON_NORMAL_PATH, QUIT_MENU_BUTTON_HOVER_PATH,
    QUIT_MENU_CANCEL_HOVER_PATH, QUIT_MENU_BUTTON_VERTICAL_GAP,
    QUIT_MENU_BUTTON_NORMAL_TEXT_COLOR, QUIT_MENU_BUTTON_HOVER_TEXT_COLOR,
    QUIT_MENU_CANCEL_HOVER_TEXT_COLOR, QuitMenuButtonKind, QuitMenuButtonVisual,
    QuitMenuButtonSpec, QUIT_MENU_BUTTONS, QuitMenuInputBoundary, QuitMenuButton
};
use interaction::{
    QuitMenuButtonLabel, handle_quit_menu_keyboard, handle_quit_menu_interactions,
    update_quit_menu_button_visuals, button_text_color
};
#[cfg(test)]
use interaction::activate_quit_menu_button;
pub use state::QUIT_MENU_PARITY_STATUS;
use state::QuitMenuPresentationState;
pub use constants::{
    QUIT_MENU_ROOT_FONT_CAVEAT, QUIT_MENU_FONT_SIZE, QUIT_MENU_TEXT_Y_OFFSET,
    QUIT_MENU_CANCEL_NORMAL_TEXT_COLOR, QUIT_MENU_CANCEL_ACTIVE_TEXT_COLOR
};
pub use textures::{
    QUIT_MENU_BACKDROP_TEXTURE_PATH_ID, QUIT_MENU_DIALOG_TEXTURE_PATH_ID,
    QUIT_MENU_BUTTON_NORMAL_TEXTURE_PATH_ID, QUIT_MENU_BUTTON_HOVER_TEXTURE_PATH_ID,
    QUIT_MENU_CANCEL_NORMAL_TEXTURE_PATH_ID, QUIT_MENU_CANCEL_HOVER_TEXTURE_PATH_ID
};
pub use audio::{
    QUIT_MENU_OPEN_SOUND_PATH, QUIT_MENU_CLOSE_SOUND_PATH, QUIT_MENU_BUTTON_SOUND_PATHS,
    QUIT_MENU_BUTTON_SOUND_GAIN, QuitMenuAudioCue, QuitMenuAudioOutbox,
    QuitMenuClickSoundSequence
};
pub use layout::{
    QUIT_MENU_REFERENCE_HEIGHT, QUIT_MENU_UI_SCALE_NUDGE, QUIT_MENU_FONT_LINE_HEIGHT,
    QUIT_MENU_DIALOG_RECT, QUIT_MENU_BUTTON_RECT, QUIT_MENU_BACKDROP_BORDER,
    QUIT_MENU_BUTTON_BORDER, QUIT_MENU_CANCEL_BORDER, QuitMenuUiRect,
    QuitMenuScaledGroupLayout, QuitMenuUiLayout, clean_quit_menu_ui_scale, quit_menu_ui_layout
};
use layout::{scaled_group_layout, update_quit_menu_layout, apply_scaled_group};
pub use types::{
    QuitMenuTextStyleSpec, QuitMenuTextStyle, QuitMenuDismissalSource, QuitMenuUiOutbox,
    QuitMenuUiRoot, QuitMenuBackdrop, QuitMenuDialog, QuitMenuUiSet, QuitMenuUiPlugin
};
use types::QuitMenuUiAssets;
pub use models::QuitMenuUiModel;
pub use commands::QuitMenuUiAction;
use view::spawn_quit_menu_ui;
pub use operations::dismiss_quit_menu_with_escape;
use systems::sync_quit_menu_visibility;
