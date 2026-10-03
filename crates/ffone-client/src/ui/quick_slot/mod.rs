//! Native, evidence-backed boundary for the clean Retrobution `cnQuickSlot`.
//!
//! The clean component is dormant by default (`localized.quickSlot =
//! eQuickSlot.eNone`). Its `Update` method nevertheless defines the exact
//! hotkey contract, while `CnGuiChat.OnOwnGUI` owns visibility, bottom-left
//! scaling, and the outer chat-relative group. This module keeps those two
//! concerns separate and owns no protocol or inventory transport.
//!
//! The four exact source textures are installed under semantic
//! `assets/game/ui/en/gameplay/quick-slot` routes. Loading those assets does not
//! change the clean visibility gate: without an explicit parity preview or
//! `LegacyQuickSlotMode::Quick`, the root remains hidden.

use std::{array, collections::VecDeque, error::Error, fmt};

use bevy::{asset::LoadState, prelude::*, ui::widget::ImageNode, window::PrimaryWindow};

use crate::mission_ui::{MissionUiModel, gameplay_chrome_visible};

use crate::ui_support::valid_ui_scale;

#[cfg(test)]
mod tests;

mod constants;
mod containers;
mod assets;
mod textures;
mod layout;
mod state;
mod types;
mod operations;
mod models;
mod frame;
mod commands;
mod validation;
mod interaction;
mod view;
mod input;

pub use constants::{
    QUICK_SLOT_SOURCE_BUILD, QUICK_SLOT_SOURCE_ARCHIVE, QUICK_SLOT_SOURCE_ARCHIVE_SHA256,
    QUICK_SLOT_COUNT, QUICK_SLOT_LARGE_CHAT_GROUP_X, QUICK_SLOT_SMALL_CHAT_GROUP_X,
    QUICK_SLOT_BACKGROUND_LOCAL_LEFT, QUICK_SLOT_BACKGROUND_LOCAL_TOP,
    QUICK_SLOT_PANEL_LOCAL_LEFT, QUICK_SLOT_PANEL_LOCAL_TOP, QUICK_SLOT_SIZE, QUICK_SLOT_GAP,
    QUICK_SLOT_STRIDE, QUICK_SLOT_COOLDOWN_INSET, QUICK_SLOT_COOLDOWN_SIZE,
    QUICK_SLOT_SOURCE_BACKGROUND_SHA256, QUICK_SLOT_SOURCE_OCCUPIED_STYLE_SHA256,
    QUICK_SLOT_SOURCE_EMPTY_STYLE_SHA256, QUICK_SLOT_SOURCE_COOLDOWN_SHA256,
    QUICK_SLOT_SOURCE_TEXTURES
};
pub use containers::QUICK_SLOT_SOURCE_SERIALIZED_FILE;
pub use assets::{
    QUICK_SLOT_GAME_HUD_PATH_ID, QUICK_SLOT_COMPONENT_PATH_ID, QUICK_SLOT_SCRIPT_PATH_ID,
    QUICK_SLOT_BACKGROUND_PATH_ID, QUICK_SLOT_INVENTORY_SKIN_PATH_ID,
    QUICK_SLOT_OCCUPIED_STYLE_PATH_ID, QUICK_SLOT_EMPTY_STYLE_PATH_ID, QUICK_SLOT_UI_Z_INDEX,
    QUICK_SLOT_BACKGROUND_PATH, QUICK_SLOT_OCCUPIED_STYLE_PATH, QUICK_SLOT_EMPTY_STYLE_PATH,
    QUICK_SLOT_COOLDOWN_PATH, QUICK_SLOT_SOURCE_BACKGROUND_PATH,
    QUICK_SLOT_SOURCE_OCCUPIED_STYLE_PATH, QUICK_SLOT_SOURCE_EMPTY_STYLE_PATH,
    QUICK_SLOT_SOURCE_COOLDOWN_PATH, QuickSlotAssetRole, QuickSlotUiAssetContract
};
pub use textures::{
    QUICK_SLOT_COOLDOWN_TEXTURE_PATH_ID, QuickSlotTextureRole, QuickSlotSourceTextureEvidence,
    is_semantic_png_path
};
use textures::is_semantic_ui_png_path;
pub use layout::{
    QUICK_SLOT_SCALE_REFERENCE_HEIGHT, QUICK_SLOT_SCALE_FACTOR, QUICK_SLOT_CHAT_WINDOW_HEIGHT,
    QUICK_SLOT_BACKGROUND_WIDTH, QUICK_SLOT_BACKGROUND_HEIGHT, QUICK_SLOT_PANEL_WIDTH,
    QUICK_SLOT_PANEL_HEIGHT, QUICK_SLOT_CONTENT_WIDTH, QUICK_SLOT_CONTENT_HEIGHT,
    QuickSlotUiRect, clean_quick_slot_ui_scale
};
use layout::bind_optional_rect;
pub use state::{LegacyQuickSlotMode, LegacyMacroChatMode};
use state::QuickSlotUiRuntimeAssets;
pub use types::{
    LegacyChatWindowStyle, QuickSlotUiConfig, QuickSlotParityPreview, LegacyQuickSlotEntry,
    QuickSlotActivationSource, QuickSlotUiOutbox, QuickSlotVisual, QuickSlotView,
    QuickSlotUiView, QuickSlotUiRoot, QuickSlotUiElement, QuickSlotUiSet, QuickSlotUiPlugin
};
use types::QuickSlotUiAssets;
pub use operations::{quick_slot_component_enabled, quick_slot_ui_view};
use operations::{has_content_hash_suffix, bind_quick_slot_ui};
pub use models::QuickSlotUiModel;
pub use frame::{QuickSlotInputFrame, quick_slot_input_frame_from_keys};
pub use commands::QuickSlotUiAction;
pub use validation::QuickSlotAssetContractError;
use interaction::{QuickSlotButton, handle_quick_slot_buttons};
use view::spawn_quick_slot_ui;
use input::read_quick_slot_hotkeys;
