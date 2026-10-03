//! Native Bevy port of `CnGuiCharSelection`.
//!
//! Runtime loading is restricted to the semantic PNG/OGG/font tree below
//! `assets/game`. The original Unity bundle and IMGUI runtime are not inputs.

use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    gameplay_ui::GAMEPLAY_UI_CAMERA_ORDER,
    localization::{
        Language, Localization, LocalizationSet, LocalizedText, UiTextAutoFit,
        localized_world_location_text,
    },
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

#[path = "../../character_selection_world_names.rs"]
mod character_selection_world_names;

macro_rules! selection_image_spec {
    ($role:literal, $path:expr, $asset:expr, $path_id:expr, $width:literal, $height:literal, $bytes:literal, $sha:literal, $draw:ident) => {
        CharacterSelectionImageSpec {
            role: $role,
            path: $path,
            source_asset: $asset,
            source_path_id: $path_id,
            width: $width,
            height: $height,
            bytes: $bytes,
            sha256: $sha,
            draw_mode: CharacterSelectionImageDrawMode::$draw,
        }
    };
}

macro_rules! selection_background_spec {
    ($path_id:literal, $bytes:literal, $sha:literal) => {
        CharacterSelectionBackgroundSpec {
            source_path_id: $path_id,
            bytes: $bytes,
            sha256: $sha,
        }
    };
}

#[cfg(test)]
mod tests;

mod layout;
mod state_character_selection_image_specs;
mod state_character_selection_assets;
mod state_bind_character_selection_controls;
mod frame;
mod assets;
mod audio_play_character_selection_sound;
mod interaction;
mod constants;
mod types;
mod input_resolve_character_selection_loca;
mod models;
mod operations;
mod view;

pub use layout::{
    CHARACTER_SELECTION_REFERENCE_WIDTH, CHARACTER_SELECTION_REFERENCE_HEIGHT,
    CHARACTER_SELECTION_BASELINE_WIDTH, CHARACTER_SELECTION_BASELINE_HEIGHT,
    CHARACTER_SELECTION_PREVIEW_WIDTH, CHARACTER_SELECTION_PREVIEW_HEIGHT,
    CHARACTER_SELECTION_JEFFE_14_LINE_HEIGHT, CHARACTER_SELECTION_JEFFE_12_LINE_HEIGHT,
    CHARACTER_SELECTION_JEFFE_16_LINE_HEIGHT, CHARACTER_SELECTION_CHALET_SMALL_LINE_HEIGHT,
    CHARACTER_SELECTION_CHALET_REGULAR_LINE_HEIGHT, LegacySelectionRect,
    CharacterSelectionLayout
};
use layout::{SLOT_BUTTON_RECTS, character_selection_text_layout, update_character_selection_layout};
pub use state_character_selection_image_specs::{
    CHARACTER_SELECTION_BACKGROUND_SPEED, CHARACTER_SELECTION_PRIMARY_MAIN_UNITY3D_BYTES,
    CHARACTER_SELECTION_PRIMARY_MAIN_UNITY3D_SHA256,
    CHARACTER_SELECTION_PRIMARY_CREATION_RESOURCE_BYTES,
    CHARACTER_SELECTION_PRIMARY_CREATION_RESOURCE_SHA256,
    CHARACTER_SELECTION_PRIMARY_PLAYER_RESOURCE_BYTES,
    CHARACTER_SELECTION_PRIMARY_PLAYER_RESOURCE_SHA256, CHARACTER_SELECTION_GUI_DEPTH,
    CHARACTER_SELECTION_JEFFE_14_FONT_SIZE, CHARACTER_SELECTION_JEFFE_12_FONT_SIZE,
    CHARACTER_SELECTION_JEFFE_16_FONT_SIZE, CHARACTER_SELECTION_CHALET_SMALL_FONT_SIZE,
    CHARACTER_SELECTION_CHALET_REGULAR_FONT_SIZE, CHARACTER_SELECTION_STYLE_PADDING,
    CHARACTER_SELECTION_CANCEL_PADDING, CHARACTER_SELECTION_TRANSPARENT2_Y_OFFSET,
    CHARACTER_SELECTION_TRANSPARENT3_Y_OFFSET, CHARACTER_SELECTION_CHAR_NAME_UP_Y_OFFSET,
    CHARACTER_SELECTION_CHAR_NAME_DOWN_Y_OFFSET, CHARACTER_SELECTION_CHAR_LEVEL_UP_Y_OFFSET,
    CHARACTER_SELECTION_DELETE_TEXT_Y_OFFSET, CHARACTER_SELECTION_AVATAR_NAME_Y_OFFSET,
    CHARACTER_SELECTION_ENTER_GAME_Y_OFFSET, CHARACTER_SELECTION_CANCEL_Y_OFFSET,
    CHARACTER_SELECTION_PREVIEW_ROTATION_SPEED_DEGREES, CharacterSelectionImageDrawMode,
    CharacterSelectionImageSpec, CHARACTER_SELECTION_IMAGE_SPECS,
    CharacterSelectionBackgroundSpec, CHARACTER_SELECTION_BACKGROUND_SPECS,
    CHARACTER_SELECTION_MODAL_CAMERA_ORDER, CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER,
    CharacterSelectionFontRole, CharacterSelectionTextAnchor, CharacterSelectionTextStyle,
    CharacterSelectionTextStyleSpec
};
pub use state_character_selection_assets::{
    ResolvedCharacterSelectionLocation, CharacterSelectionPending,
    CharacterSelectionCapability, CharacterPreviewStatus, CharacterSelectionUiAction,
    CharacterSelectionUiOutbox, NativeCharacterSelectionUiPlugin, CharacterSelectionUiSet,
    NativeCharacterSelectionRoot
};
use state_character_selection_assets::{
    character_selection_location_part, CharacterSelectionBackgroundClock, CharacterSelectionRandom,
    CharacterSelectionAssets, SelectionChrome, SelectionAvatarGroup, SelectionPreviewArea,
    SelectionPreviewLoading,
    SelectionRotateLeft, SelectionRotateRight, SelectionPanel, SelectionQuit, SelectionFullscreen,
    SelectionSlotName, SelectionSlotLevel, SelectionSlotLocation, SelectionSlotEmptyLabel,
    SelectionSlotPortraitArea, SelectionSlotDiskBack, SelectionSlotDiskFront, SelectionSlotLock,
    SelectionAvatarName, SelectionEnter, SelectionEnterText, SelectionCreate, SelectionCreateText,
    SelectionDelete, SelectionDeleteText, SelectionMusicToggle, SelectionQuitText, SelectionMusic,
    SelectionDeleteModal, SelectionBaseCamera, SelectionModalCamera, SelectionPortraitOverlayCamera,
    SelectionPortraitOverlayRoot, SelectionDeletePanel, SelectionDeleteFieldText,
    SelectionDeleteCancel, SelectionDeleteCancelText, SelectionDeleteConfirm,
    SelectionDeleteConfirmText, character_selection_text_font, character_selection_text_color,
    character_selection_text_node
};
use state_bind_character_selection_controls::{
    bind_character_selection_slot_text, bind_character_selection_decorations,
    sync_character_selection_camera_activity, bind_character_selection_control_visibility,
    bind_character_selection_preview_loading,
    bind_character_selection_controls, bind_character_selection_delete_modal,
    control_character_selection_music
};
pub use frame::{
    CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH, CHARACTER_SELECTION_BACKGROUND_FRAME_HEIGHT,
    CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT
};
use frame::SelectionBackgroundFrame;
pub use assets::{
    CHARACTER_SELECTION_MUSIC_PATH, CHARACTER_SELECTION_CHROME_PATH,
    CHARACTER_SELECTION_SLOT_EMPTY_PATH, CHARACTER_SELECTION_SLOT_NORMAL_PATH,
    CHARACTER_SELECTION_SLOT_OVER_PATH, CHARACTER_SELECTION_SLOT_LOCKED_PATH,
    CHARACTER_SELECTION_LOCK_PATH, CHARACTER_SELECTION_ENTER_PATH,
    CHARACTER_SELECTION_ENTER_OVER_PATH, CHARACTER_SELECTION_FULLSCREEN_PATH,
    CHARACTER_SELECTION_WINDOWED_PATH, CHARACTER_SELECTION_DELETE_WINDOW_PATH,
    CHARACTER_SELECTION_DELETE_BACKDROP_PATH, CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
    CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH, CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH,
    CHARACTER_SELECTION_FULLSCREEN_OVER_PATH, CHARACTER_SELECTION_WINDOWED_OVER_PATH,
    CHARACTER_SELECTION_ROTATE_LEFT_PATH, CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH,
    CHARACTER_SELECTION_ROTATE_RIGHT_PATH, CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH,
    CHARACTER_SELECTION_DISK_BACK_PATH, CHARACTER_SELECTION_DISK_FRONT_PATH,
    CHARACTER_SELECTION_CHALET_FONT_PATH, CHARACTER_SELECTION_JEFFE_FONT_PATH,
    CHARACTER_SELECTION_PRIMARY_ASSET_FILE, CHARACTER_SELECTION_GAME_OBJECT_PATH_ID,
    CHARACTER_SELECTION_SKIN_PATH_ID, CHARACTER_SELECTION_GUI_COMPONENT_PATH_ID,
    CHARACTER_SELECTION_MODE_COMPONENT_PATH_ID, CHARACTER_SELECTION_GUI_SCRIPT_PATH_ID,
    CHARACTER_SELECTION_MODE_SCRIPT_PATH_ID, CHARACTER_SELECTION_CHROME_SOURCE_PATH_ID,
    CHARACTER_SELECTION_DELETE_BACKDROP_SOURCE_PATH_ID,
    CHARACTER_SELECTION_DELETE_WINDOW_SOURCE_PATH_ID,
    CHARACTER_SELECTION_SLOT_EMPTY_SOURCE_PATH_ID,
    CHARACTER_SELECTION_SLOT_NORMAL_SOURCE_PATH_ID,
    CHARACTER_SELECTION_SLOT_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_SLOT_LOCKED_SOURCE_PATH_ID, CHARACTER_SELECTION_LOCK_SOURCE_PATH_ID,
    CHARACTER_SELECTION_ENTER_SOURCE_PATH_ID, CHARACTER_SELECTION_ENTER_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_FULLSCREEN_SOURCE_PATH_ID,
    CHARACTER_SELECTION_FULLSCREEN_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_WINDOWED_SOURCE_PATH_ID,
    CHARACTER_SELECTION_WINDOWED_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_MUSIC_NORMAL_SOURCE_PATH_ID,
    CHARACTER_SELECTION_MUSIC_ON_SOURCE_PATH_ID,
    CHARACTER_SELECTION_RED_NORMAL_SOURCE_PATH_ID,
    CHARACTER_SELECTION_RED_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_BLUE_NORMAL_SOURCE_PATH_ID,
    CHARACTER_SELECTION_BLUE_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_CANCEL_NORMAL_SOURCE_PATH_ID,
    CHARACTER_SELECTION_ROTATE_LEFT_SOURCE_PATH_ID,
    CHARACTER_SELECTION_ROTATE_LEFT_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_ROTATE_RIGHT_SOURCE_PATH_ID,
    CHARACTER_SELECTION_ROTATE_RIGHT_OVER_SOURCE_PATH_ID,
    CHARACTER_SELECTION_DISK_BACK_SOURCE_PATH_ID,
    CHARACTER_SELECTION_DISK_FRONT_SOURCE_PATH_ID, CHARACTER_SELECTION_JEFFE_14_PATH_ID,
    CHARACTER_SELECTION_JEFFE_12_PATH_ID, CHARACTER_SELECTION_JEFFE_16_PATH_ID,
    CHARACTER_SELECTION_CHALET_SMALL_PATH_ID, CHARACTER_SELECTION_CHALET_REGULAR_PATH_ID,
    CharacterSelectionAssetStatus
};
use assets::update_character_selection_asset_status;
pub use audio_play_character_selection_sound::{
    CHARACTER_SELECTION_BUTTON_SOUND_PATHS, CHARACTER_SELECTION_DELETE_YES_SOUND_PATH,
    CHARACTER_SELECTION_DELETE_NO_SOUND_PATH
};
use audio_play_character_selection_sound::{
    play_character_selection_sound, play_character_selection_button_sound
};
pub use interaction::{
    CHARACTER_SELECTION_RED_BUTTON_PATH, CHARACTER_SELECTION_RED_BUTTON_OVER_PATH,
    CHARACTER_SELECTION_BLUE_BUTTON_PATH, CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH,
    CHARACTER_SELECTION_QUIT_BUTTON_Y_OFFSET, CHARACTER_SELECTION_CREATE_BUTTON_Y_OFFSET
};
use interaction::{
    CHARACTER_SELECTION_BASE_INTERACTION_Z_INDEX,
    CHARACTER_SELECTION_PORTRAIT_OVERLAY_INTERACTION_Z_INDEX,
    CHARACTER_SELECTION_MODAL_INTERACTION_Z_INDEX, SelectionSlotButton,
    legacy_sliced_button_image, handle_character_selection_interactions
};
use constants::{
    SLOT_NAME_Y, SLOT_LEVEL_Y, SLOT_LOCATION_Y, SLOT_EMPTY_Y, SLOT_AVATAR_Y, SLOT_DISK_BACK_Y,
    SLOT_DISK_FRONT_Y, SLOT_LOCK_Y
};
pub use types::{CharacterLocationBackground, OccupiedCharacterSlotUi, CharacterSlotUi};
pub use input_resolve_character_selection_loca::resolve_character_selection_location;
pub use models::CharacterSelectionUiModel;
use operations::{absolute_node, image_node, tinted_image_node, edit_character_delete_name};
use view::spawn_character_selection_ui;
