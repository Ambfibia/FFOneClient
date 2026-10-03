//! Standalone clean-Retrobution `eGameMode::Launcher` UI and interaction contract.
//!
//! This is the in-client environmental launcher reached from an
//! `EpLauncherTrigger`, not the external OpenFusion desktop launcher. The
//! primary authority is `retrobution-20260613`: `main.unity3d` owns the
//! source-spelled inactive `LuncherMode` root, `CnGuiLauncher`, `cnLauncher`,
//! and `LauncherSkin`; `Tutorial.resourceFile` owns `launchcross`.
//!
//! The module retains the exact fixed-pixel screen geometry, triangular
//! power oscillator, serialized aim speed/launch offset, system-popup input
//! pause, Computress Escape gate, death cancellation, launch-vector
//! calculation, legacy audio ordering, and typed shell effects. World
//! trigger discovery, camera application, renderer visibility, movement,
//! packet serialization, and GameFrame mode routing remain explicit outbox
//! boundaries. No runtime path opens a Unity container or `.ffclient` cache.

use std::{collections::VecDeque, error::Error, fmt};

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::localization::{LocalizationSet, LocalizedText};

use crate::ui_support::stretched_image as stretch_image;

#[cfg(test)]
mod tests;

mod constants;
mod state;
mod assets;
mod systems;
mod audio;
mod localization_launcher_ui_power_localization_k;
mod layout;
mod operations;
mod types;
mod validation;
mod frame;
mod interaction;
mod models;
mod view;

pub use constants::{
    LAUNCHER_UI_SOURCE_BUILD, LAUNCHER_UI_ROOT_NAME, LAUNCHER_UI_ROOT_INITIALLY_ACTIVE,
    LAUNCHER_UI_CAMERA_LAUNCH_OFFSET, LAUNCHER_UI_DEPTH, LAUNCHER_UI_MAIN_SIZE,
    LAUNCHER_UI_MAIN_SHA256, LAUNCHER_UI_TUTORIAL_SIZE, LAUNCHER_UI_TUTORIAL_SHA256,
    LAUNCHER_UI_CHARACTER_CREATION_SIZE, LAUNCHER_UI_CHARACTER_CREATION_SHA256,
    LAUNCHER_UI_MANAGED_GUI_SHA256, LAUNCHER_UI_MANAGED_LOGIC_SHA256,
    LAUNCHER_UI_PARITY_CAVEAT, LAUNCHER_UI_IMAGE_PATHS, LAUNCHER_UI_POWER_LABEL_KEY,
    LAUNCHER_UI_TIP_LABEL_KEY, LAUNCHER_UI_DEFAULT_FONT_SIZE, LAUNCHER_UI_SMALL_FONT_SIZE,
    LAUNCHER_UI_CROSS_SIZE, LAUNCHER_UI_GAUGE_AREA_X, LAUNCHER_UI_GAUGE_AREA_Y
};
pub use state::{LAUNCHER_UI_GAME_MODE_SLOT, LAUNCHER_UI_PARITY_STATUS, LauncherUiExternalState};
pub use assets::{
    LAUNCHER_UI_ROOT_PATH_ID, LAUNCHER_UI_TRANSFORM_PATH_ID,
    LAUNCHER_UI_GUI_COMPONENT_PATH_ID, LAUNCHER_UI_LOGIC_COMPONENT_PATH_ID,
    LAUNCHER_UI_GUI_SCRIPT_PATH_ID, LAUNCHER_UI_LOGIC_SCRIPT_PATH_ID,
    LAUNCHER_UI_SKIN_PATH_ID, LAUNCHER_UI_Z_INDEX, LAUNCHER_UI_BACKDROP_PATH,
    LAUNCHER_UI_CROSSHAIR_PATH, LAUNCHER_UI_GAUGE_PATH, LAUNCHER_UI_GAUGE_BAR_PATH,
    LAUNCHER_UI_FONT_PATH, LAUNCHER_UI_DEFAULT_FONT_PATH_ID, LAUNCHER_UI_SMALL_FONT_PATH_ID,
    LauncherUiAssetContract, LAUNCHER_UI_ASSET_CONTRACTS
};
pub use systems::LAUNCHER_UI_ROTATE_SPEED_PER_FIXED_UPDATE;
use systems::{apply_launcher_fixed_aim, sync_launcher_labels};
pub use audio::{
    LAUNCHER_UI_CLICK_ON_AUDIO_PATH, LAUNCHER_UI_START_POWER_AUDIO_PATH,
    LAUNCHER_UI_POWER_PULSE_AUDIO_PATH, LAUNCHER_UI_STOP_POWER_AUDIO_PATH,
    LAUNCHER_UI_FIRING_AUDIO_PATH, LauncherUiAudioCue
};
pub use localization_launcher_ui_power_localization_k::{
    LAUNCHER_UI_POWER_LOCALIZATION_KEY, LAUNCHER_UI_TIP_LOCALIZATION_KEY
};
pub use layout::{
    LAUNCHER_UI_DEFAULT_LINE_HEIGHT, LAUNCHER_UI_SMALL_LINE_HEIGHT, LAUNCHER_UI_GAUGE_WIDTH,
    LAUNCHER_UI_GAUGE_HEIGHT, LAUNCHER_UI_GAUGE_AREA_WIDTH, LAUNCHER_UI_GAUGE_AREA_HEIGHT,
    LAUNCHER_UI_BAR_WIDTH, LAUNCHER_UI_BAR_HEIGHT, LAUNCHER_UI_BACKDROP_BORDER,
    LauncherUiRect, LAUNCHER_UI_CROSS_SOURCE_RECT, LAUNCHER_UI_GAUGE_SOURCE_RECT,
    LAUNCHER_UI_GAUGE_AREA_SOURCE_RECT, LAUNCHER_UI_BAR_SOURCE_RECT,
    LAUNCHER_UI_POWER_SOURCE_RECT, LAUNCHER_UI_TIP_SOURCE_RECT, LauncherUiLayout
};
use layout::sync_launcher_layout;
use operations::{
    finite_or_zero, vec3_is_finite, absolute_node, centered_label_node, sliced_backdrop_image
};
pub use operations::launcher_forward;
pub use types::{
    LauncherTriggerSpec, LauncherUiPhase, LauncherUiDismissalSource, LauncherShot,
    LauncherUiEffect, LauncherUiOutbox, LauncherUiLabels, LauncherUiAssets, LauncherUiElement,
    LauncherUiTextRole, LauncherUiSet, LauncherUiPlugin
};
pub use validation::LauncherUiOpenError;
pub use frame::{
    LAUNCHER_REQUEST_PACKET_ID, LAUNCHER_REQUEST_PACKET_SIZE, LAUNCHER_BROADCAST_PACKET_ID,
    LAUNCHER_BROADCAST_PACKET_SIZE, LauncherUiFrameInput
};
pub use interaction::LauncherUiInputBoundary;
use interaction::handle_launcher_keyboard;
pub use models::LauncherUiModel;
use view::spawn_launcher_ui;
