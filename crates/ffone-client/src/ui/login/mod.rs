//! Native replacement for the legacy FusionFall login form.

use crate::text_edit::{self, EditVisual, TextEdit, TextEditPlugin};
use std::collections::VecDeque;

use crate::{
    localization::{Language, Localization, LocalizationSet, LocalizedText, localized_status},
    option_ui::OptionUiModel,
};
use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState},
    audio::{AudioSink, AudioSinkPlayback, Volume},
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::{FontSmoothing, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

#[cfg(test)]
mod tests;

mod constants;
mod state;
mod containers;
mod assets;
mod textures;
mod interaction;
mod types;
mod layout;
mod operations;
mod frame;
mod models;
mod commands;
mod view;
mod systems;

pub use constants::{
    LOGIN_SOURCE_BUILD, LOGIN_SOURCE_MAIN_ARCHIVE, LOGIN_SOURCE_MAIN_ARCHIVE_BYTES,
    LOGIN_SOURCE_MAIN_ARCHIVE_SHA256, LOGIN_SOURCE_CREATION_ARCHIVE,
    LOGIN_SOURCE_CREATION_ARCHIVE_BYTES, LOGIN_SOURCE_CREATION_ARCHIVE_SHA256,
    LOGIN_COMMUNITY_URL, LOGIN_REGISTRATION_INSTRUCTIONS, LOGIN_IMAGE_SPECS,
    LOGIN_REPLACEMENT_FONT_SPECS, LOGIN_FALLBACK_BACKGROUND_SIZE, LOGIN_BACKGROUND_SIZE,
    LOGIN_PANEL_SIZE, LOGIN_JEFFE_FONT_SIZE, LOGIN_CHALET_FONT_SIZE, LOGIN_LABEL_Y_OFFSET,
    LOGIN_TEXT_FIELD_Y_OFFSET, LOGIN_LABEL_PADDING, LOGIN_TEXT_FIELD_PADDING
};
pub use state::{LOGIN_SOURCE_RUNTIME, LOGIN_STATUS_ADAPTER_Y_OFFSET};
use state::{login_background_mode, LoginLoadedBackgroundState, LoginStatusText};
pub use containers::{LOGIN_SOURCE_UNITY_ENGINE_SHA256, LOGIN_SOURCE_SERIALIZED_FILE};
pub use assets::{
    LOGIN_SOURCE_GAME_OBJECT_PATH_ID, LOGIN_COMPONENT_PATH_ID, LOGIN_COMPONENT_SCRIPT_PATH_ID,
    LOGIN_MODE_COMPONENT_PATH_ID, LOGIN_MODE_SCRIPT_PATH_ID, LOGIN_SKIN_PATH_ID,
    LOGIN_FALLBACK_BACKGROUND_PATH_ID, LOGIN_SOURCE_CREATION_ASSET, LOGIN_BACKGROUND_PATH,
    LOGIN_FALLBACK_BACKGROUND_PATH, LOGIN_PANEL_PATH, LOGIN_TEXT_FIELD_PATH, LOGIN_FONT_PATH,
    LOGIN_TEXT_FIELD_FONT_PATH, LOGIN_MUSIC_PATH, LOGIN_JEFFE_SOURCE_FONT_PATH_ID,
    LOGIN_CHALET_SOURCE_FONT_PATH_ID, LoginUiAssetStatus
};
use assets::update_login_ui_asset_status;
pub use textures::{
    LOGIN_BLACK_TEXTURE_PATH_ID, LOGIN_BLACK_TEXTURE_SIZE, LOGIN_BLACK_TEXTURE_SOURCE_BYTES,
    LOGIN_BLACK_TEXTURE_SOURCE_SHA256, LOGIN_BLACK_TEXTURE_DECODED_RGBA_SHA256,
    LOGIN_PANEL_TEXTURE_PATH_ID, LOGIN_BUTTON_NORMAL_TEXTURE_PATH_ID,
    LOGIN_BUTTON_HOVER_TEXTURE_PATH_ID, LOGIN_BUTTON_ACTIVE_TEXTURE_PATH_ID,
    LOGIN_TEXT_FIELD_TEXTURE_PATH_ID, LOGIN_LOADED_BACKGROUND_TEXTURE_PATH_ID,
    LOGIN_PANEL_TEXTURE_SIZE
};
pub use interaction::{
    LOGIN_BUTTON_PATH, LOGIN_BUTTON_OVER_PATH, LOGIN_BUTTON_ACTIVE_PATH,
    LOGIN_BUTTON_Y_OFFSET, LOGIN_BUTTON_PADDING
};
pub(crate) use interaction::{LoginSubmitButton, LoginCommunityButton, LoginRegisterButton};
use interaction::{
    LoginStyledButton, LoginButtonTextRole, sliced_button_image, handle_login_keyboard,
    handle_login_edit_pointer, handle_login_interactions, handle_login_language_interaction,
    login_button_text_color
};
pub use types::{
    LoginImageSpec, LoginReplacementFontSpec, LoginBackgroundMode0104, LoginTextStyle0104,
    LoginField, LoginSurface, LoginUiOutbox, LoginUiEffect, LoginUiEffectOutbox, LoginUiSet,
    NativeLoginUiPlugin
};
use types::{
    LoginUiAssets, LoginRoot, LoginLoadedBackground, LoginFallbackBackground, LoginPanel,
    LoginUsernameField, LoginPasswordField, LoginUsernameText, LoginPasswordText,
    LoginSubmitText, LoginLanguageButton, LoginLanguageText, LoginMusic
};
use layout::{
    LOGIN_USERNAME_LABEL_RECT, LOGIN_USERNAME_FIELD_RECT, LOGIN_PASSWORD_LABEL_RECT,
    LOGIN_PASSWORD_FIELD_RECT, LOGIN_SUBMIT_RECT, LOGIN_LANGUAGE_RECT, LOGIN_GLAYOUT_LEADING_FLEX,
    LOGIN_GLAYOUT_TRAILING_FLEX, UiSourceRect, UiResolvedRect, LoginGLayoutArea, LoginGLayoutColumn,
    LoginGLayoutFlexibleSpace, update_login_layout
};
#[cfg(test)]
use layout::login_ui_scale;
pub use layout::{
    LOGIN_GLAYOUT_TOP, LOGIN_GLAYOUT_EN_COMMUNITY_ADVANCE, LOGIN_GLAYOUT_EN_REGISTER_ADVANCE,
    LOGIN_GLAYOUT_EN_MIN_WIDTH, LOGIN_GLAYOUT_EN_X, LOGIN_GLAYOUT_BUTTON_HEIGHT,
    LOGIN_GLAYOUT_BUTTON_GAP, LOGIN_GLAYOUT_REGISTER_Y, LOGIN_JEFFE_LINE_HEIGHT,
    LOGIN_CHALET_LINE_HEIGHT, LOGIN_PANEL_BORDER, LOGIN_BUTTON_BORDER,
    LOGIN_TEXT_FIELD_BORDER, LOGIN_UI_SCALE_REFERENCE_HEIGHT, LOGIN_UI_SCALE_NUDGE
};
use operations::{
    login_can_assign_loaded_background, queue_login, bind_login_edit_visuals, bind_login_ui,
    control_login_music, bind_login_language
};
#[cfg(test)]
use operations::login_text_field_color;
use frame::login_background_frame;
pub use models::LoginUiModel;
pub use commands::LoginRequest;
use view::spawn_login_ui;
#[cfg(test)]
use systems::apply_login_key;
use systems::apply_login_edit_key;

mod browser;
mod browser_view;
mod account_view;
mod account_scroll;
mod symbols;
pub use browser::{LoginBrowser, LoginServer, ServerHealth};
