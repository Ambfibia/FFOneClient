//! Clean-Retrobution `eGameMode.Combi` projection, authoritative transaction
//! boundary, and native UI.
//!
//! Primary authority is the clean `retrobution-20260613` `main.unity3d`
//! (`sharedassets0.assets`) plus its matching clean Assembly-CSharp:
//! `cnCombiMode`, `cnGuiCombi`, `InventoryManagerScript`, and the three exact
//! combination packet structs. The published clean TableData conversion is
//! used only through the strict `m_pCombiningTable.m_pCombiningData` parser.
//!
//! The two selected items are an overlay over the immutable 50-slot inventory.
//! Attach/detach never changes authoritative inventory or Taros. After the
//! clean `> 4.0` second wait this module emits a semantic request. A separately
//! validated authoritative reply produces a receipt; only an explicit receipt
//! commit changes the snapshot. This keeps the native slice incapable of
//! optimistic item removal, item replacement, or currency spending.

use crate::{
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104,
    },
    localization::{LocalizationSet, LocalizedText},
    user_equip_ui::{
        USER_EQUIP_BACKDROP_HEIGHT, USER_EQUIP_BACKDROP_PATH, USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_CLOSE_HOVER_PATH, USER_EQUIP_CLOSE_PATH, USER_EQUIP_CLOSE_RECT,
        USER_EQUIP_COMBINED_BADGE_LEFT, USER_EQUIP_COMBINED_BADGE_SIZE,
        USER_EQUIP_COMBINED_BADGE_TOP, USER_EQUIP_EQUIP_STRIP_RECT, USER_EQUIP_EQUIP_TITLE_PATH,
        USER_EQUIP_EQUIPMENT_CONTENT_RECT, USER_EQUIP_EQUIPMENT_SLOT_SIZE,
        USER_EQUIP_EQUIPMENT_SLOT_STRIDE, USER_EQUIP_EQUIPMENT_STRIP_COUNT,
        USER_EQUIP_EQUIPMENT_STRIP_ORDER, USER_EQUIP_EQUIPMENT_TITLE_RECT, USER_EQUIP_FONT_PATH,
        USER_EQUIP_HELP_HOVER_PATH, USER_EQUIP_HELP_PATH, USER_EQUIP_HELP_RECT,
        USER_EQUIP_INVENTORY_CONTENT_HEIGHT, USER_EQUIP_INVENTORY_CONTENT_WIDTH,
        USER_EQUIP_INVENTORY_PANEL_BORDER, USER_EQUIP_INVENTORY_PANEL_PATH,
        USER_EQUIP_INVENTORY_SCROLL_VELOCITY, USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_STRIDE, USER_EQUIP_INVENTORY_VIEWPORT_RECT,
        USER_EQUIP_ITEM_TAB_HIT_RECT, USER_EQUIP_PC_STUFF_RECT, USER_EQUIP_RIGHT_PANEL_BORDER,
        USER_EQUIP_RIGHT_PANEL_PATH, USER_EQUIP_SLOT_EMPTY_PATH, USER_EQUIP_SLOT_OCCUPIED_PATH,
        USER_EQUIP_TRASH_PATH, USER_EQUIP_TRASH_RECT, UserEquipUiRect, clamp_user_equip_scroll,
        user_equip_bevy_wheel_to_legacy_axis, user_equip_item_mode_layout,
    },
};
use bevy::{
    asset::LoadState,
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    sprite::BorderRect,
    ui::FocusPolicy,
    window::PrimaryWindow,
};
use ffone_protocol::ItemBase0104;
use serde_json::Value;
use std::{array, collections::VecDeque, error::Error, fmt};

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

use crate::ui_support::display_if;

#[cfg(test)]
mod tests;

mod constants;
mod containers;
mod assets;
mod state;
mod interaction;
mod frame;
mod commands;
mod validation;
mod layout;
mod types_combi_authoritative_snapshot0104;
mod types_combi_machine0104;
mod input_find_combining_arrays;
mod operations_classify_clean_client_chance;
mod binding_combi_ui;
mod projection;
mod localization_bind_optional_localized_text;
mod view_combi_main_group;
mod view_combi_overlays;

pub use constants::{
    COMBI_SOURCE_BUILD, COMBI_SOURCE_MAIN_ARCHIVE, COMBI_SOURCE_MAIN_ARCHIVE_SHA256,
    COMBI_NPC_ID_0104, COMBI_NPC_TYPE_0104, COMBI_FIRST_USE_CONDITION_0104,
    COMBI_ITEM_BASE_SIZE_0104, COMBI_SUCCESS_NEW_ITEM_SLOT_OFFSET_0104,
    COMBI_SUCCESS_NEW_ITEM_OFFSET_0104, COMBI_SUCCESS_STAT_SLOT_OFFSET_0104,
    COMBI_SUCCESS_CASH_SLOT_1_OFFSET_0104, COMBI_SUCCESS_CASH_SLOT_2_OFFSET_0104,
    COMBI_SUCCESS_TAROS_OFFSET_0104, COMBI_SUCCESS_FLAG_OFFSET_0104,
    COMBI_FAILURE_COSTUME_SLOT_OFFSET_0104, COMBI_FAILURE_STAT_SLOT_OFFSET_0104,
    COMBI_FAILURE_CASH_SLOT_1_OFFSET_0104, COMBI_FAILURE_CASH_SLOT_2_OFFSET_0104,
    COMBI_STYLE_SLOT_TYPE_0104, COMBI_STATS_SLOT_TYPE_0104, COMBI_WAIT_SECONDS_0104,
    COMBI_RECIPE_ROW_COUNT_0104, COMBI_RECIPE_MAX_LEVEL_GAP_0104, COMBI_RECIPE_TABLE_SHA256,
    COMBI_MESSAGE_254, COMBI_MESSAGE_255, COMBI_MESSAGE_256, COMBI_MESSAGE_260,
    COMBI_COLOR_DEFAULT, COMBI_COLOR_GREEN, COMBI_COLOR_RED, COMBI_COLOR_NOT_READY,
    COMBI_COLOR_NOT_POSSIBLE, COMBI_UI_DEFAULT_IMAGE_PATHS
};
pub use containers::COMBI_SOURCE_SERIALIZED_FILE;
pub use assets::{
    COMBI_GAME_OBJECT_PATH_ID, COMBI_MODE_COMPONENT_PATH_ID, COMBI_GUI_COMPONENT_PATH_ID,
    COMBI_PC_STUFF_COMPONENT_PATH_ID, COMBI_EQUIP_COMPONENT_PATH_ID, COMBI_SKIN_PATH_ID,
    COMBI_PRIMARY_CAMERA_PATH_ID, COMBI_WAITING_CAMERA_PATH_ID, COMBI_UI_Z_INDEX,
    COMBI_PANEL_PATH, COMBI_BLACK_SHADE_PATH, COMBI_LOOK_ITEM_BG_PATH,
    COMBI_STAT_ITEM_BG_PATH, COMBI_COMBINED_PATH, COMBI_TAROS_ICON_PATH, COMBI_SUCCESS_PATH,
    COMBI_NPC_ICON_PATH, COMBI_WAITING_PATH, COMBI_CHALET_FONT_PATH, COMBI_RECIPE_TABLE_PATH,
    CombiStaticAssetRole, CombiUiAssetContract, CombiUiAssetContractError0104,
    CombiStaticAssetReadiness0104, CombiUiAssetStatus0104
};
use assets::{safe_relative_asset_path, look_icon_path, stats_icon_path};
pub use state::{
    COMBI_GAME_MODE_0104, CombiSelectionSlot0104, CombiSelectionChange0104,
    CombiSelectionOverlay0104, CombiSelectionError0104, CombiInventorySlotProjection0104,
    CombiModeProjection0104, CombiStateError0104
};
use state::snapshot_inventory_item;
pub use interaction::{
    COMBI_NPC_BUTTON_TYPE_0104, COMBI_BUTTON_NORMAL_PATH, COMBI_BUTTON_HOVER_PATH,
    CombiInputCapabilities0104, CombiPointerState0104
};
use interaction::{
    collect_combi_ui_input, combi_inventory_slot_draggable, bind_hover_image,
    bind_button_visual
};
pub use frame::{
    COMBI_REQUEST_PACKET_ID_0104, COMBI_SUCCESS_PACKET_ID_0104, COMBI_FAILURE_PACKET_ID_0104,
    COMBI_REQUEST_PACKET_SIZE_0104, COMBI_SUCCESS_PACKET_SIZE_0104,
    COMBI_FAILURE_PACKET_SIZE_0104, COMBI_RESTRICTED_ITEM_FRAME_PATH,
    localized_combi_wire_error
};
use frame::{
    COMBI_HOVER_FRAME_WIDTH, bind_selection_frame, CombiHoverFrame, combi_drop_frame,
    bind_hover_frame
};
pub use commands::{
    COMBI_REQUEST_COSTUME_SLOT_OFFSET_0104, COMBI_REQUEST_STAT_SLOT_OFFSET_0104,
    COMBI_REQUEST_CASH_SLOT_1_OFFSET_0104, COMBI_REQUEST_CASH_SLOT_2_OFFSET_0104,
    CombiReplyError0104
};
pub use validation::{
    COMBI_FAILURE_ERROR_OFFSET_0104, COMBI_LOOK_ERROR_PATH, COMBI_STAT_ERROR_PATH
};
use validation::validate_reply_envelope;
pub use layout::{
    COMBI_REFERENCE_WIDTH, COMBI_REFERENCE_HEIGHT, COMBI_LEFT_GROUP_WIDTH,
    COMBI_LEFT_GROUP_HEIGHT, COMBI_PANEL_NATIVE_WIDTH, COMBI_PANEL_NATIVE_HEIGHT,
    COMBI_RIGHT_BACKPLATE_WIDTH, CombiUiRect, COMBI_PRIMARY_NPC_PREVIEW_RECT,
    COMBI_TITLE_RECT, COMBI_INTRO_RECT, COMBI_STYLE_TITLE_RECT, COMBI_STATS_TITLE_RECT,
    COMBI_COST_RECT, COMBI_TAROS_LABEL_RECT, COMBI_TAROS_ICON_RECT, COMBI_CHANCE_TITLE_RECT,
    COMBI_CHANCE_RECT, COMBI_CHANCE_LEVEL_RECT, COMBI_NEW_ITEM_TITLE_RECT,
    COMBI_LOOK_DROP_RECT, COMBI_LOOK_ERROR_RECT, COMBI_LOOK_SELECTED_SLOT_RECT,
    COMBI_LOOK_ICON_RECT, COMBI_LOOK_BADGE_RECT, COMBI_LOOK_NAME_RECT, COMBI_LOOK_LEVEL_RECT,
    COMBI_LOOK_DESCRIPTION_RECT, COMBI_LOOK_EMPTY_TEXT_RECT, COMBI_STAT_DROP_RECT,
    COMBI_STAT_ERROR_RECT, COMBI_STAT_SELECTED_SLOT_RECT, COMBI_STAT_LEVEL_RECT,
    COMBI_STAT_SECTION_RECT, COMBI_STAT_SINGLE_RECT, COMBI_STAT_MULTI_RECT,
    COMBI_STAT_DEFENSE_RECT, COMBI_INFO_SECTION_RECT, COMBI_STAT_EMPTY_TEXT_RECT,
    COMBI_CLEAR_ALL_RECT, COMBI_COMBINE_RECT, COMBI_SUCCESS_GROUP_RECT,
    COMBI_WAITING_GROUP_RECT, COMBI_WAITING_NPC_PREVIEW_RECT, COMBI_SUCCESS_NPC_ICON_RECT,
    COMBI_SUCCESS_HOORAY_RECT, COMBI_SUCCESS_MESSAGE_RECT, COMBI_SUCCESS_ICON_RECT,
    COMBI_SUCCESS_BADGE_RECT, COMBI_SUCCESS_NAME_RECT, COMBI_SUCCESS_LEVEL_RECT,
    COMBI_SUCCESS_DESCRIPTION_RECT, COMBI_SUCCESS_COMBINE_MORE_RECT,
    COMBI_SUCCESS_GO_TO_STUFF_RECT, COMBI_BUTTON_BORDER, CombiModeLayout0104,
    combi_mode_layout_0104
};
use layout::bind_combi_rect;
pub use types_combi_authoritative_snapshot0104::{
    CombiRgb0104, CombiSourceLocation0104, CombiItemMetadata0104, CombiItemCatalog0104,
    CombiAuthoritativeSnapshot0104, CombiReceiptCommitError0104, CombiRecipe0104,
    CombiRecipeTable0104, CombiRecipeTableError0104, CombiLookError0104, CombiStatsError0104,
    CombiStatComparison0104, CombiChance0104, CombiLookProjection0104,
    CombiStatsProjection0104, CombiProjectionError0104, CombiRequest0104,
    CombiSuccessReply0104, CombiFailureReply0104, CombiAuthorityReceipt0104,
    CombiSuccessPresentation0104, CombiSystemModal0104, CombiModalChoice0104, CombiPhase0104,
    CombiMachine0104
};
use types_combi_authoritative_snapshot0104::CombiPendingAttempt0104;
pub use types_combi_machine0104::{
    CombiExternalModalState0104, CombiUiState0104, CombiUiCommand0104, CombiUiOutbox0104,
    CombiCarriedItem0104, CombiInteractiveControl0104, CombiUiElement0104, CombiUiRoot0104,
    CombiUiSet0104, CombiUiPlugin
};
use types_combi_machine0104::CombiUiAssets;
use input_find_combining_arrays::{find_combining_arrays, parse_recipe_row};
use operations_classify_clean_client_chance::{
    recipe_i32, recipe_f32, compare_stat, classify_clean_client_chance, combi_fallback_text,
    combi_passthrough_text, combi_cost_text, combi_chance_level_text, combi_item_level_text,
    combi_stat_value_text, combi_range_text, combi_type_text, combi_passive,
    cancel_combi_carry
};
pub use operations_classify_clean_client_chance::{
    combined_appearance_item_id, combi_equipment_item_type, clean_range_label,
    empty_item_0104, expected_success_style_item
};
use binding_combi_ui::bind_combi_ui;
pub use binding_combi_ui::clean_type_label;
pub use projection::project_combi_mode_0104;
#[cfg(test)]
use projection::{project_inventory_slot, project_look, project_stats};
pub use localization_bind_optional_localized_text::localized_combi_equipped_item_message;
use localization_bind_optional_localized_text::{
    bind_localized_text, bind_optional_localized_text
};
use view_combi_main_group::spawn_combi_ui;
use view_combi_overlays::{spawn_combi_overlays, spawn_combi_button, spawn_combi_text};
