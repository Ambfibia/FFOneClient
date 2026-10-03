//! Clean-Retrobution `eGameMode.Cashmall` contract and standalone Bevy UI.
//!
//! This is intentionally the half-wired Cash Mall that shipped in the clean
//! client, not a newly designed cash shop. The only observed entry is the
//! hidden `CnGuiChat` `/cashmall` command. `cnCashmallMode.ReceivePacket` is a
//! no-op, `Panel_Cashmall.iUserCash` is never assigned, every tab renders the
//! same fixed slot-type-9 scan, and row activation delegates to the existing
//! local `InventoryManagerScript.VendorClickItem` popup boundary. There is no
//! Cash Mall purchase packet contract here.
//!
//! The legacy Unity containers are offline evidence only. Runtime rendering
//! uses semantic native assets below `assets/game`, including the exact shared
//! Vendor and UserEquip shell assets rather than copied Cash Mall variants.

use crate::{
    inventory_runtime::INVENTORY_SLOT_COUNT_0104,
    localization::{LocalizationSet, LocalizedText},
    user_equip_ui::{
        USER_EQUIP_BACKDROP_HEIGHT, USER_EQUIP_BACKDROP_PATH, USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_CLOSE_PATH, USER_EQUIP_CLOSE_RECT, USER_EQUIP_COMBINED_BADGE_LEFT,
        USER_EQUIP_COMBINED_BADGE_SIZE, USER_EQUIP_COMBINED_BADGE_TOP, USER_EQUIP_COMBINED_PATH,
        USER_EQUIP_COUNT_LABEL_LEFT, USER_EQUIP_COUNT_LABEL_TOP, USER_EQUIP_EQUIP_STRIP_RECT,
        USER_EQUIP_EQUIP_TITLE_PATH, USER_EQUIP_EQUIPMENT_CONTENT_RECT,
        USER_EQUIP_EQUIPMENT_SLOT_SIZE, USER_EQUIP_EQUIPMENT_SLOT_STRIDE,
        USER_EQUIP_EQUIPMENT_STRIP_COUNT, USER_EQUIP_EQUIPMENT_STRIP_ORDER,
        USER_EQUIP_EQUIPMENT_TITLE_RECT, USER_EQUIP_FONT_PATH, USER_EQUIP_HELP_PATH,
        USER_EQUIP_HELP_RECT, USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
        USER_EQUIP_INVENTORY_CONTENT_WIDTH, USER_EQUIP_INVENTORY_PANEL_BORDER,
        USER_EQUIP_INVENTORY_PANEL_PATH, USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_STRIDE, USER_EQUIP_INVENTORY_VIEWPORT_RECT,
        USER_EQUIP_ITEM_TAB_HIT_RECT, USER_EQUIP_NANO_TAB_HIT_RECT, USER_EQUIP_NANO_TAB_PATH,
        USER_EQUIP_PC_STUFF_RECT, USER_EQUIP_RIGHT_PANEL_BORDER, USER_EQUIP_RIGHT_PANEL_PATH,
        USER_EQUIP_SLOT_EMPTY_PATH, USER_EQUIP_SLOT_OCCUPIED_PATH, USER_EQUIP_TRASH_PATH,
        USER_EQUIP_TRASH_RECT, UserEquipEquipmentSlotKind, UserEquipItemModeLayout,
        UserEquipItemModeProjection, UserEquipPresentationIcon, UserEquipSlotFrameVisual,
        user_equip_item_mode_layout, user_equip_missing_checker_rgba,
    },
    vendor_ui::{
        VENDOR_BUTTON_BORDER, VENDOR_BUTTON_HOVER_PATH, VENDOR_BUTTON_NORMAL_PATH,
        VENDOR_INFO_PATH, VENDOR_LIST_BACK_BORDER, VENDOR_LIST_BACK_PATH, VENDOR_PANEL_PATH,
        VENDOR_RESTRICTED_ITEM_FRAME_PATH, VENDOR_SCROLL_SHADOW_BORDER, VENDOR_SCROLL_SHADOW_PATH,
    },
};
use bevy::{
    asset::{LoadState, RenderAssetUsages},
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::BorderRect,
    text::LineHeight,
    window::PrimaryWindow,
};
use ffone_protocol::ItemBase0104;
use std::{array, collections::VecDeque, error::Error, fmt};

use crate::ui_support::display_if as display_cashmall_if_0104;

use crate::ui_support::stretched_image as stretched_cashmall_image_0104;

use crate::ui_support::sliced_image as sliced_cashmall_image_0104;

#[cfg(test)]
mod localization_tests;

mod constants;
mod containers;
mod state;
mod commands;
mod assets;
mod textures;
mod interaction;
mod layout;
mod audio;
mod frame;
mod types_cashmall_text_style0104;
mod types_cashmall_ui_state0104;
mod operations;
mod validation;
mod systems;
mod view;
mod localization_cashmall_equipment_slot_localize;

pub use constants::{
    CASHMALL_SOURCE_BUILD, CASHMALL_SOURCE_MAIN_ARCHIVE, CASHMALL_SOURCE_MAIN_ARCHIVE_SHA256,
    CASHMALL_MANAGED_PANEL_SHA256, CASHMALL_PANEL_START_X, CASHMALL_OPEN_SECONDS,
    CASHMALL_SLOT_TYPE, CASHMALL_SLOT_SCAN_COUNT, CASHMALL_CACHED_ITEM_COUNT_0104,
    CASHMALL_USER_CASH_0104, CASHMALL_CASH_DIGIT_COUNT, CASHMALL_TAB_COUNT,
    CASHMALL_GUI_DEPTH_0104, CASHMALL_SHARED_GUI_DEPTH_0104, CASHMALL_PANEL_NATIVE_Z_0104,
    CASHMALL_SHARED_NATIVE_Z_0104, CASHMALL_PURCHASE_NETWORK_CONTRACT_PRESENT,
    CASHMALL_USER_CASH_ASSIGNMENT_REACHABLE, CASHMALL_TAB_FILTERING_REACHABLE,
    CASHMALL_SET_VENDOR_ITEM_REACHABLE, CASHMALL_ITEM_BAR_ASSIGNED,
    CASHMALL_NPC_ASSIGNMENT_FROM_CHAT_ENTRY_REACHABLE, CASHMALL_HELP_RECEIVER_PRESENT,
    CASHMALL_FREE_ASSETS_CLEARS_LOADED_TEXTURES, CASHMALL_RECENT_BUY_SLOT12_BRANCH_REACHABLE,
    CASHMALL_LABEL_FONT_SIZE, CASHMALL_SMALL_FONT_SIZE, CASHMALL_LABEL_PADDING_TOP,
    CASHMALL_LABEL_PADDING_BOTTOM, CASHMALL_REPLACEMENT_FONT_Y_OFFSET,
    CASHMALL_SOURCE_TEXTURES_0104, CASHMALL_HIDDEN_CHAT_BOUNDARY_0104,
    CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104
};
pub use containers::CASHMALL_SOURCE_SERIALIZED_FILE;
pub use state::{
    CASHMALL_MANAGED_MODE_SHA256, CASHMALL_GAME_MODE_0104,
    CASHMALL_INVENTORY_MODE_INITIALIZATION_REACHABLE, CashmallPlayerInventoryItem0104,
    CashmallModeProjection0104, CashmallModeView0104, cashmall_mode_view_0104
};
pub use commands::{
    CASHMALL_HIDDEN_CHAT_COMMAND_0104, CASHMALL_HELP_EVENT_ID, CASHMALL_VENDOR_POPUP_ACTION,
    CashmallActionBlocked0104
};
pub use assets::{
    CASHMALL_GAME_OBJECT_PATH_ID, CASHMALL_TRANSFORM_PATH_ID, CASHMALL_MODE_COMPONENT_PATH_ID,
    CASHMALL_PANEL_COMPONENT_PATH_ID, CASHMALL_PC_STUFF_COMPONENT_PATH_ID,
    CASHMALL_EQUIP_COMPONENT_PATH_ID, CASHMALL_MODE_SCRIPT_PATH_ID,
    CASHMALL_PANEL_SCRIPT_PATH_ID, CASHMALL_PC_STUFF_SCRIPT_PATH_ID,
    CASHMALL_EQUIP_SCRIPT_PATH_ID, CASHMALL_SKIN_PATH_ID, CASHMALL_INVENTORY_SKIN_PATH_ID,
    CASHMALL_ITEM_BAR_PATH_ID, CASHMALL_TAB_VISUAL_FONT_PATH_ID, CASHMALL_LABEL_FONT_PATH_ID,
    CASHMALL_SMALL_FONT_PATH_ID, CASHMALL_UI_Z_INDEX, CASHMALL_BACK_BAR_PATH,
    CASHMALL_CASH_PATH, CASHMALL_FIRST_TAB_SELECTED_PATH, CASHMALL_FIRST_TAB_NORMAL_PATH,
    CASHMALL_SECOND_TAB_SELECTED_PATH, CASHMALL_SECOND_TAB_NORMAL_PATH, CASHMALL_DEXLABS_PATH,
    CASHMALL_TAROS_COUNTER_PATH, cashmall_safe_relative_asset_path_0104,
    CashmallStaticAssetRole0104, CashmallUiAssetContract0104,
    CashmallUiAssetContractError0104, CashmallStaticAssetReadiness0104,
    CashmallUiAssetStatus0104, cashmall_tab_asset_role_0104
};
pub use textures::{
    CASHMALL_CASH_TEXTURE_PATH_ID, CASHMALL_INFO_TEXTURE_PATH_ID,
    CASHMALL_NANO_TAB_TEXTURE_PATH_ID, CASHMALL_NANO_TAB_HOVER_TEXTURE_PATH_ID,
    CASHMALL_DEXLABS_TEXTURE_PATH_ID, CASHMALL_TAROS_COUNTER_TEXTURE_PATH_ID,
    CashmallTextureRole0104, CashmallTextureEvidence0104
};
pub use interaction::{
    CASHMALL_BUTTON_FONT_PATH_ID, CASHMALL_AUTOMATIC_SCROLLBAR_REACHABLE,
    CASHMALL_BUTTON_FONT_SIZE, CASHMALL_BUTTON_PADDING_LEFT, CASHMALL_BUTTON_PADDING_RIGHT,
    CASHMALL_BUTTON_PADDING_TOP, CASHMALL_BUTTON_PADDING_BOTTOM,
    CASHMALL_FIRST_TAB_HOVER_PATH, CASHMALL_SECOND_TAB_HOVER_PATH,
    CASHMALL_NANO_TAB_HOVER_PATH, CashmallInputCapabilities0104, CashmallScrollTarget0104,
    CashmallPointerButton0104, CashmallVendorClickBoundary0104
};
use interaction::{
    CashmallHoverState0104, collect_cashmall_ui_input_0104, cashmall_button_text_color_0104
};
pub use layout::{
    CASHMALL_REFERENCE_WIDTH, CASHMALL_REFERENCE_HEIGHT, CASHMALL_BACKPLATE_REFERENCE_WIDTH,
    CASHMALL_BACKPLATE_REFERENCE_HEIGHT, CASHMALL_PANEL_WIDTH, CASHMALL_PANEL_HEIGHT,
    CASHMALL_ROW_HEIGHT, CASHMALL_ROW_WIDTH, CASHMALL_ROW_VISUAL_HEIGHT,
    CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT, CASHMALL_VENDOR_POPUP_RECT,
    CASHMALL_LABEL_FONT_LINE_HEIGHT, CASHMALL_BUTTON_FONT_LINE_HEIGHT,
    CASHMALL_SMALL_FONT_LINE_HEIGHT, CASHMALL_FIRST_TAB_BORDER, CASHMALL_SECOND_TAB_BORDER,
    CashmallUiRect0104, CASHMALL_DIALOG_RECT, CASHMALL_LIST_BACK_RECT, CASHMALL_TABLE_RECT,
    CASHMALL_LIST_VIEWPORT_RECT, CASHMALL_TABLE_SHADOW_RECT, CASHMALL_INFO_RECT,
    CASHMALL_TITLE_RECT, CASHMALL_NPC_NAME_RECT, CASHMALL_CASH_RECT,
    CASHMALL_LIST_DIVIDER_RECT, CASHMALL_GO_TO_STUFF_RECT, CASHMALL_ROW_ITEM_BOX_RECT,
    CASHMALL_ROW_NAME_RECT, CASHMALL_ROW_LEVEL_RECT, CASHMALL_ROW_PRICE_RECT,
    CASHMALL_SCROLL_UP_RECT, CASHMALL_SCROLL_DOWN_RECT, CASHMALL_SCROLL_TRACK_RECT,
    CASHMALL_TAB_RECTS, CASHMALL_TAB_HIT_RECTS, CASHMALL_LIST_BACK_BORDER,
    CASHMALL_SCROLL_SHADOW_BORDER, CASHMALL_BUTTON_BORDER,
    CASHMALL_PC_STUFF_INVENTORY_PANEL_RECT, CASHMALL_PC_STUFF_NANO_TAB_RECT,
    CASHMALL_PC_STUFF_NANO_TAB_HIT_RECT, CASHMALL_PC_STUFF_DEXLABS_RECT,
    CASHMALL_PC_STUFF_TAROS_COUNTER_RECT, CASHMALL_PC_STUFF_REDEEM_CODE_RECT,
    CASHMALL_PC_STUFF_TAROS_DIGIT_RECTS, CashmallModeLayout0104, cashmall_mode_layout_0104,
    cashmall_tab_border_0104
};
use layout::bind_cashmall_rect_0104;
pub use audio::{CASHMALL_TAB_SOUND, CashmallAudioCue0104};
pub use frame::{CASHMALL_RECEIVE_PACKET_MUTATES_STATE, CashmallSlotFrameVisual0104};
use frame::cashmall_shared_slot_frame_image_0104;
pub use types_cashmall_text_style0104::{
    CashmallTextStyle0104, CashmallTab0104, CashmallTabLabelSource0104, CashmallTabVisual0104,
    CashmallTabView0104, CashmallIconRef0104, CashmallIconRefError0104,
    CashmallPresentationIcon0104, CashmallRowProjection0104, CashmallLifecyclePhase0104,
    CashmallModalState0104, CashmallCloseGate0104, CashmallOpenSource0104,
    CashmallHiddenChatBoundary0104, CashmallCloseSource0104, CashmallTabActivation0104,
    CashmallGoToStuffBoundary0104, CashmallCloseBoundary0104, CashmallLocalEffect0104,
    CashmallUiOutbox0104, CashmallUiState0104
};
use types_cashmall_ui_state0104::{CashmallUiAssets0104, CashmallInteractiveControl0104};
pub use types_cashmall_ui_state0104::{
    CashmallUiRoot0104, CashmallUiElement0104, CashmallUiSet0104, CashmallUiPlugin0104
};
pub use operations::{
    cashmall_tab_view_0104, cashmall_scan_slot9_0104, cashmall_opening_eased_fraction_0104,
    cashmall_cash_digits_0104, cashmall_cash_text_0104, cashmall_taros_counter_digit_0104
};
use operations::{
    cashmall_missing_checker_image_0104, cashmall_passthrough_text_0104,
    cashmall_item_level_text_0104, cashmall_resolved_fallback_0104, bind_cashmall_ui_0104
};
use validation::cashmall_item_needs_equip_validation;
use systems::advance_cashmall_lifecycle_0104;
use view::spawn_cashmall_ui_0104;
use localization_cashmall_equipment_slot_localize::{
    cashmall_equipment_slot_localized_text_0104, cashmall_tab_localized_text_0104,
    bind_cashmall_localized_text_0104, bind_optional_cashmall_localized_text_0104
};
