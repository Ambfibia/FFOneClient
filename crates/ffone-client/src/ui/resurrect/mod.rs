//! Clean-Retrobution `ResurrectMode` UI and deterministic domain boundary.
//!
//! Primary evidence:
//! - `main.unity3d` SHA-256
//!   `59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F`;
//! - `sharedassets0.assets` GameObject path ID 1326, component path ID 1438,
//!   `FusionFallSysMessageSkin` path ID 1380;
//! - clean `Assembly - CSharp.dll` SHA-256
//!   `33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB`,
//!   `ResurrectMode::{InitMode,Update,OnGUI,SendRevivalPacket,ReceivePacket}`.
//! - clean `fusion-2.x.x` `UnityEngine.dll` SHA-256
//!   `90EF121A97F954D35A50FB27F7DBCF99EAFFEAFCBCF938EAE9910C29F3745E0C`,
//!   `GUILayoutGroup::{CalcWidth,CalcHeight,SetHorizontal,SetVertical}`.
//!
//! The module owns presentation and a typed FIFO outbox. It deliberately does
//! not own protocol encoding, nearest-XCom lookup, inventory/table loading,
//! camera grayscale, cursor locking, mode switching, or Nano selection. Those
//! are explicit shell boundaries. An unresolved nearest-XCom index fails
//! closed; callers may pass `Some(0)` when they intentionally reproduce the
//! clean `GetXCom()` no-match result.
//!
//! One clean-client quirk is preserved rather than repaired: `USE ITEM` is
//! drawn with `PhoenixButton` (44,124,150,25), after the Phoenix button and
//! icon. The separately serialized `UseItemButton` (209,124,150,25) is never
//! read by `OnGUI`. Consequently the item control is the topmost control when
//! both item and Phoenix choices are visible.

use std::collections::VecDeque;

use bevy::{
    prelude::*,
    sprite::BorderRect,
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::localization::{LocalizationSet, LocalizedText};

use crate::ui_support::legacy_screen_extent;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod layout_resurrect_ui_context;
mod layout_update_resurrect_content;
mod textures;
mod localization_resurrect_title_localization_key;
mod models;
mod view;
mod operations;

pub use layout_resurrect_ui_context::{
    RESURRECT_SOURCE_BUILD, RESURRECT_SOURCE_ARCHIVE, RESURRECT_SOURCE_ARCHIVE_SHA256,
    RESURRECT_SOURCE_SERIALIZED_FILE, RESURRECT_SOURCE_SERIALIZED_FILE_SHA256,
    RESURRECT_SOURCE_ASSEMBLY_SHA256, RESURRECT_SOURCE_UNITY_ENGINE_SHA256,
    RESURRECT_GAME_OBJECT_PATH_ID, RESURRECT_COMPONENT_PATH_ID, RESURRECT_SKIN_PATH_ID,
    RESURRECT_SCRIPT_PATH_ID, RESURRECT_SKILL_ICON_BACK_PATH_ID, RESURRECT_BLACK_BACK_PATH_ID,
    RESURRECT_BUTTON_NORMAL_PATH_ID, RESURRECT_BUTTON_HOVER_PATH_ID,
    RESURRECT_BUTTON_FONT_PATH_ID, RESURRECT_BODY_FONT_PATH_ID, RESURRECT_GRIM_PATH,
    RESURRECT_ICON_PATH, RESURRECT_SKILL_ICON_BACK_PATH, RESURRECT_BLACK_BACK_PATH,
    RESURRECT_DIALOG_PATH, RESURRECT_BUTTON_NORMAL_PATH, RESURRECT_BUTTON_HOVER_PATH,
    RESURRECT_PHOENIX_SELF_ICON_PATH, RESURRECT_PHOENIX_GROUP_ICON_PATH,
    RESURRECT_BUTTON_FONT_PATH, RESURRECT_BODY_FONT_PATH, RESURRECT_GRIM_SHA256,
    RESURRECT_ICON_SHA256, RESURRECT_SKILL_ICON_BACK_SHA256, RESURRECT_BLACK_BACK_SHA256,
    RESURRECT_DIALOG_SHA256, RESURRECT_BUTTON_NORMAL_SHA256, RESURRECT_BUTTON_HOVER_SHA256,
    RESURRECT_PHOENIX_SELF_ICON_SHA256, RESURRECT_PHOENIX_GROUP_ICON_SHA256,
    RESURRECT_BUTTON_FONT_SHA256, RESURRECT_BODY_FONT_SHA256, RESURRECT_REFERENCE_HEIGHT,
    RESURRECT_UI_SCALE_NUDGE, RESURRECT_TIMEOUT_SECONDS, RESURRECT_UI_Z_INDEX,
    RESURRECT_BACKDROP_ALPHA, RESURRECT_BUTTON_FONT_SIZE, RESURRECT_BUTTON_LINE_HEIGHT,
    RESURRECT_BODY_FONT_SIZE, RESURRECT_BODY_LINE_HEIGHT, RESURRECT_TEXT_Y_OFFSET,
    RESURRECT_LABEL_MARGIN, RESURRECT_LABEL_PADDING, RESURRECT_IMAGEWINDOW_MARGIN,
    RESURRECT_IMAGEWINDOW_PADDING, RESURRECT_BUTTON_MARGIN, RESURRECT_BUTTON_PADDING,
    RESURRECT_DIALOG_BORDER, RESURRECT_BUTTON_BORDER, RESURRECT_LABEL_COLOR,
    RESURRECT_BODY_COLOR, RESURRECT_COUNTDOWN_COLOR, RESURRECT_BUTTON_NORMAL_COLOR,
    RESURRECT_BUTTON_HOVER_COLOR, RESURRECT_WINDOW_RECT, RESURRECT_GO_BUTTON_RECT,
    RESURRECT_PHOENIX_BUTTON_RECT, RESURRECT_UNUSED_ITEM_BUTTON_RECT,
    RESURRECT_TEXT_LABEL_RECT, RESURRECT_GRIM_RECT, RESURRECT_ICON_RECT,
    RESURRECT_PHOENIX_SKILL_RECT, RESURRECT_PHOENIX_SKILL_BACK_RECT, RESURRECT_TITLE,
    RESURRECT_QUESTION, RESURRECT_COUNTDOWN_PREFIX, RESURRECT_SECONDS, RESURRECT_GO_LABEL,
    RESURRECT_REVIVE_LABEL, RESURRECT_USE_ITEM_LABEL, ResurrectFontRole, ResurrectTextAnchor,
    ResurrectTextStyle, ResurrectTextStyleSpec, RESURRECT_PARITY_STATUS,
    RESURRECT_UNRESOLVED_BOUNDARIES, ResurrectUiRect, ResurrectUiLayout,
    clean_resurrect_ui_scale, resurrect_ui_layout, ResurrectChoice, ResurrectInventorySlot,
    clean_resurrection_item_slot, ResurrectUiContext, ResurrectRequestOrigin,
    ResurrectRegenRequest, ResurrectEnterEffects, CLEAN_RESURRECT_ENTER_EFFECTS,
    ResurrectSuccessEffects, ResurrectUiAction, ResurrectUiOutbox, ResurrectRequestBlocker,
    ResurrectInputBoundary, ResurrectUiText, ResurrectUiAssets, ResurrectUiSet,
    ResurrectUiPlugin
};
use layout_resurrect_ui_context::{
    valid_ui_scale, ResurrectUiRoot, ResurrectDialog, ResurrectTextNode, ResurrectTextRole,
    ResurrectUiButton, ResurrectUiButtonLabel, ResurrectPhoenixVisual
};
pub use textures::{
    RESURRECT_GRIM_TEXTURE_PATH_ID, RESURRECT_ICON_TEXTURE_PATH_ID,
    RESURRECT_DIALOG_TEXTURE_PATH_ID, ResurrectTextureRole, ResurrectTextureContract,
    RESURRECT_TEXTURE_CONTRACTS
};
pub use localization_resurrect_title_localization_key::{
    RESURRECT_TITLE_LOCALIZATION_KEY, RESURRECT_QUESTION_LOCALIZATION_KEY,
    RESURRECT_COUNTDOWN_LOCALIZATION_KEY, RESURRECT_GO_LOCALIZATION_KEY,
    RESURRECT_REVIVE_LOCALIZATION_KEY, RESURRECT_USE_ITEM_LOCALIZATION_KEY
};
pub use models::ResurrectUiModel;
use view::spawn_resurrect_ui;
use operations::array_color;
