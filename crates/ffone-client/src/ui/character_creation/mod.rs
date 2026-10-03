//! Native Bevy port of `CnGuiNameCreation` and `CnGuiCharCreation`.
//!
//! The runtime consumes only semantic PNG/OGG/font files below `assets/game`.
//! Unity bundles, Unity IMGUI, and the legacy player are not runtime inputs.
//!
//! The two legacy screens intentionally remain separate:
//!
//! 1. `CnGuiNameCreation` reserves a generated or custom name.
//! 2. after the server accepts that name, `CnGuiCharCreation` saves appearance.
//!
//! Network ownership therefore stays outside this module. UI intent is emitted
//! through [`CharacterCreationUiOutbox`], while the authoritative network layer
//! advances [`CharacterCreationUiModel::screen`] only after a server reply.

use std::{
    collections::{BTreeMap, VecDeque},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use crate::localization::Localization;
use crate::{
    gameplay_audio::RetrobutionAudioMix,
    gameplay_ui::GAMEPLAY_UI_CAMERA_ORDER,
    localization::{LocalizationSet, LocalizedText, UiTextAutoFit},
    system_message_ui::SystemMessageUiModel,
};
use bevy::{
    asset::LoadState,
    audio::{AudioSink, AudioSinkPlayback, Volume},
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::LineHeight,
    ui::FocusPolicy,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

macro_rules! image_spec {
    ($name:literal, $path:literal, $width:literal, $height:literal) => {
        CharacterCreationImageSpec {
            true_name: $name,
            path: $path,
            width: $width,
            height: $height,
        }
    };
}

pub mod barber_ui;

#[cfg(test)]
mod tests;

mod layout;
mod constants;
mod assets;
mod materials;
mod validation;
mod state;
mod textures;
mod audio_control_sound;
mod interaction;
mod types_character_creation_text_style;
mod types_native_character_creation_ui_plugin;
mod models;
mod operations;
mod commands;
mod view_appearance;
mod view_name_creation;
mod systems;

pub use layout::{
    CHARACTER_CREATION_REFERENCE_WIDTH, CHARACTER_CREATION_REFERENCE_HEIGHT,
    CHARACTER_CREATION_APPEARANCE_WIDTH, CHARACTER_CREATION_APPEARANCE_HEIGHT,
    CHARACTER_CREATION_NAME_WIDTH, CHARACTER_CREATION_NAME_HEIGHT,
    CHARACTER_CREATION_JEFFE_14_LINE_HEIGHT, CHARACTER_CREATION_JEFFE_16_LINE_HEIGHT,
    CHARACTER_CREATION_CHALET_SMALL_LINE_HEIGHT,
    CHARACTER_CREATION_CHALET_REGULAR_LINE_HEIGHT, LegacyCreationRect, CharacterCreationLayout
};
use layout::{
    AppearanceHeightText, source_style_border, character_creation_text_layout,
    update_character_creation_layout
};
pub use constants::{
    CHARACTER_CREATION_PRIMARY_MAIN_UNITY3D_BYTES,
    CHARACTER_CREATION_PRIMARY_MAIN_UNITY3D_SHA256,
    CHARACTER_CREATION_PRIMARY_RESOURCE_FILE_BYTES,
    CHARACTER_CREATION_PRIMARY_RESOURCE_FILE_SHA256, CHARACTER_CREATION_NAME_GUI_DEPTH,
    CHARACTER_CREATION_APPEARANCE_GUI_DEPTH, CHARACTER_CREATION_JEFFE_14_FONT_SIZE,
    CHARACTER_CREATION_JEFFE_16_FONT_SIZE, CHARACTER_CREATION_CHALET_SMALL_FONT_SIZE,
    CHARACTER_CREATION_CHALET_REGULAR_FONT_SIZE, CHARACTER_CREATION_TEXT_Y_OFFSET,
    CHARACTER_CREATION_TEXT_FIELD_SOURCE_FILE_ID,
    CHARACTER_CREATION_PREVIEW_ROTATION_SPEED_DEGREES,
    CHARACTER_CREATION_PREVIEW_MIN_DISTANCE, CHARACTER_CREATION_PREVIEW_MAX_DISTANCE,
    CHARACTER_CREATION_FOREGROUND_CAMERA_ORDER,
    CHARACTER_CREATION_PREVIEW_ZOOM_DURATION_SECONDS, CHARACTER_CREATION_PREVIEW_ZOOM_SPEED,
    CHARACTER_CREATION_IMAGE_SPECS, CHARACTER_CREATION_SHARED_IMAGE_SPECS,
    CHARACTER_CREATION_ENGINE_IMAGE_SPECS, SKIN_COLORS, HAIR_COLORS, EYE_COLORS
};
use constants::{
    CCBG, CC_CHARACTER_DISPLAY, CC_RIGHT_BG, CC_IN_12_BG, CC_IN_3_BG, CC_BODY_DISPLAY,
    CC_BODY_LEFT, CC_BODY_LEFT_OVER, CC_BODY_RIGHT, CC_BODY_RIGHT_OVER, CC_CHECKED,
    CC_CHECKED_OVER, CC_ROTATE_LEFT, CC_ROTATE_LEFT_OVER, CC_ROTATE_RIGHT,
    CC_ROTATE_RIGHT_OVER, CC_ZOOM_IN, CC_ZOOM_IN_OVER, CC_ZOOM_OUT, CC_ZOOM_OUT_OVER,
    CC_CLOTHES_BG, CC_COLOR_INSIDE, CC_COLOR_OUTLINE, CC_NAME_BG, CC_NAME_DISPLAY,
    CC_NAME_SHADE, CC_NAME_TAB_1, CC_NAME_TAB_2, CC_FULLSCREEN, CC_WINDOWED,
    CC_FULLSCREEN_OVER, CC_WINDOWED_OVER
};
pub use assets::{
    CHARACTER_CREATION_SKIN_PATH_ID, CHARACTER_CREATION_BSD_PATH_ID,
    CHARACTER_CREATION_APPEARANCE_GAME_OBJECT_PATH_ID,
    CHARACTER_CREATION_APPEARANCE_GUI_COMPONENT_PATH_ID,
    CHARACTER_CREATION_APPEARANCE_MODE_COMPONENT_PATH_ID,
    CHARACTER_CREATION_NAME_GAME_OBJECT_PATH_ID,
    CHARACTER_CREATION_NAME_GUI_COMPONENT_PATH_ID,
    CHARACTER_CREATION_NAME_MODE_COMPONENT_PATH_ID,
    CHARACTER_CREATION_NAME_SUBMIT_COMPONENT_PATH_ID,
    CHARACTER_CREATION_SIMPLE_CAMERA_COMPONENT_PATH_ID, CHARACTER_CREATION_JEFFE_14_PATH_ID,
    CHARACTER_CREATION_JEFFE_16_PATH_ID, CHARACTER_CREATION_CHALET_SMALL_PATH_ID,
    CHARACTER_CREATION_CHALET_REGULAR_PATH_ID, CHARACTER_CREATION_TEXT_FIELD_PATH,
    CHARACTER_CREATION_TEXT_FIELD_SOURCE_PATH_ID, CHARACTER_CREATION_BACKGROUND_PATH,
    CHARACTER_CREATION_FONT_PATH, CHARACTER_CREATION_DISPLAY_FONT_PATH,
    CHARACTER_CREATION_MUSIC_PATH, CharacterCreationAssetStatus
};
use assets::update_character_creation_asset_status;
pub use materials::{
    CHARACTER_CREATION_RENDER_CAMERA_GAME_OBJECT_PATH_ID,
    CHARACTER_CREATION_RENDER_CAMERA_COMPONENT_PATH_ID
};
pub use validation::{
    CHARACTER_CREATION_PRIMARY_KOREAN_CHECK_REACHABLE, CharacterNameValidationError,
    validate_custom_name
};
use validation::{CC_CHECK_NORMAL, CC_CHECK_OVER};
pub use state::{
    CHARACTER_CREATION_PRIMARY_CLASS_SELECTION_REACHABLE, CharacterNameMode,
    CharacterCreationPreviewStatus
};
use state::{CC_COLOR_SELECTED, ModeGeneratedRoot, ModeCustomRoot, SelectedColor};
pub use textures::CHARACTER_CREATION_TEXT_FIELD_PNG_SHA256;
pub use audio_control_sound::{
    CHARACTER_CREATION_BUTTON_SOUND_PATHS, CHARACTER_CREATION_CONTINUE_SOUND_PATH,
    CHARACTER_CREATION_RANDOM_SOUND_PATH, CHARACTER_CREATION_TAB_SOUND_PATH,
    CHARACTER_CREATION_COLOR_SOUND_PATH, CHARACTER_CREATION_HEIGHT_DOWN_SOUND_PATH,
    CHARACTER_CREATION_HEIGHT_UP_SOUND_PATH, CHARACTER_CREATION_GIRTH_NARROW_SOUND_PATH,
    CHARACTER_CREATION_GIRTH_WIDE_SOUND_PATH
};
use audio_control_sound::{CharacterCreationSound, control_sound, play_sound_cue};
#[cfg(test)]
use audio_control_sound::CharacterCreationSoundCue;
use interaction::{
    CC_NAME_BUTTON, CC_NAME_BUTTON_OVER, CC_SCROLL_BG, CC_SCROLL_UP, CC_SCROLL_UP_OVER,
    CC_SCROLL_DOWN, CC_SCROLL_DOWN_OVER, CC_BLUE_BUTTON, CC_BLUE_BUTTON_OVER, CC_RED_BUTTON,
    CC_RED_BUTTON_OVER, CharacterCreationButton, CreationNameModeButton, source_button_states,
    handle_character_creation_interactions
};
pub use types_character_creation_text_style::{
    CharacterCreationFontRole, CharacterCreationTextAnchor, CharacterCreationTextStyle,
    CharacterCreationTextStyleSpec, CharacterCreationScreen, CharacterGender, AppearanceField,
    CharacterCreationOptionCounts, CharacterAppearance, CharacterNamePart, CharacterNameLists,
    GeneratedCharacterName, CustomCharacterName, CharacterCreationPending,
    CharacterCreationCapability, CharacterCreationUiOutbox, NativeCharacterCreationRoot,
    CharacterCreationImageSpec, NativeCharacterCreationUiPlugin, CharacterCreationUiSet
};
use types_character_creation_text_style::{
    CharacterCreationControl, CreationClothingIcon, CreationBackground, AppearanceRoot,
    NameRoot, CreationFullscreen, CharacterCreationBaseCamera,
    CharacterCreationForegroundCamera, CreationPreviewControlsRoot, CreationMusic,
    AppearanceBodyText, AppearanceHairText, AppearanceFaceText, CustomNameText,
    GeneratedNameText, NameColumnText, ColorSwatch, CharacterCreationRandom,
    CharacterCreationColorKind, CharacterCreationAssets, CharacterCreationStartupSet
};
pub use models::CharacterCreationUiModel;
use operations::{
    compose_legacy_last_name, character_creation_assets_active, image_node, sliced_image_node,
    source_style_image_node, character_creation_text_font, character_creation_text_color,
    character_creation_text_node, bind_character_creation_visibility,
    bind_character_creation_controls, bind_character_creation_clothing_icons,
    bind_character_creation_text, bind_character_creation_colors, randomize_appearance,
    control_on_current_screen, repeat_character_creation_camera_controls,
    edit_custom_character_name, control_character_creation_music
};
#[cfg(test)]
use operations::indexed_appearance_label;
pub use commands::{CharacterCreationCameraAction, CharacterCreationUiAction};
use view_appearance::{
    spawn_image, spawn_label, spawn_image_button, spawn_name_mode_button, spawn_text_button,
    spawn_character_creation_ui
};
use view_name_creation::{spawn_color, spawn_name_creation};
use systems::{sync_character_creation_camera_activity, apply_pending_appearance_randomization};
