//! Clean-Retrobution `VendorMode` projection, intent boundary, and UI.
//!
//! Primary authority:
//! - `cnVendor` owns start/table-update sequencing and the checked
//!   buy/general/battery/restore/sell/delete/disassemble request decisions;
//! - `Panel_Vendor` owns the 498x638 left panel, 80px rows, buy/buyback tabs,
//!   one-second sine entrance, and 1036x653 two-piece shell;
//! - `Panel_PCStuffScript`, `Panel_Equip`, and `InventoryManagerScript` own the
//!   shared 9+50 item projection and close/help/modal gates.
//!
//! This module deliberately does not encode packets and never speculatively
//! mutates inventory, currency, or the recent-buy FIFO. It projects complete
//! authoritative snapshots and emits typed semantic intents for a production
//! transport owner to encode. Server successes must be followed by a fresh
//! authoritative snapshot before this projection changes.

use crate::{
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104,
    },
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
        USER_EQUIP_ITEM_TAB_HIT_RECT, USER_EQUIP_MISSING_CHECKER_SIZE, USER_EQUIP_PC_STUFF_RECT,
        USER_EQUIP_REGULAR_FONT_LINE_HEIGHT, USER_EQUIP_RIGHT_PANEL_BORDER,
        USER_EQUIP_RIGHT_PANEL_PATH, USER_EQUIP_SLOT_EMPTY_PATH, USER_EQUIP_SLOT_OCCUPIED_PATH,
        USER_EQUIP_SMALL_FONT_LINE_HEIGHT, USER_EQUIP_SMALL_FONT_SIZE, USER_EQUIP_TRASH_PATH,
        USER_EQUIP_TRASH_RECT, UserEquipItemIds, UserEquipItemModeLayout, UserEquipUiRect,
        user_equip_item_mode_layout, user_equip_missing_checker_rgba,
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

mod chest_open;
mod item_popup;
pub use chest_open::{VendorChestOpenIntent, VendorChestOpenState};
pub use item_popup::VendorItemPopupState;

use crate::ui_support::display_if;

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cold_icon_regression;

mod constants;
mod containers;
mod assets;
mod materials;
mod interaction;
mod frame;
mod layout;
mod commands;
mod types_vendor_ui_text_style;
mod types_vendor_ui_element;
mod validation;
mod state_vendor_mode_projection0104;
mod state_vendor_ui_state;
mod operations;
mod audio;
mod systems;
mod view_vendor_panel;
mod view_vendor_equipment_slot;
mod localization_equipment_slot_localized;

pub use constants::{
    VENDOR_SOURCE_BUILD, VENDOR_SOURCE_MAIN_ARCHIVE, VENDOR_SOURCE_MAIN_ARCHIVE_SHA256,
    VENDOR_SOURCE_TUTORIAL_ARCHIVE, VENDOR_SOURCE_TUTORIAL_ARCHIVE_SHA256,
    VENDOR_PANEL_GUI_DEPTH_0104, VENDOR_SHARED_GUI_DEPTH_0104,
    VENDOR_PRIMARY_ASSEMBLY_CSHARP_DLL_SHA256, VENDOR_DEXLABS_SHA256,
    VENDOR_TAROS_COUNTER_SHA256, VENDOR_BOOST_ICON_SHA256, VENDOR_POTION_ICON_SHA256,
    VENDOR_LABEL_FONT_SIZE, VENDOR_EQUIP_FONT_SIZE, VENDOR_SERVICE_FONT_SIZE,
    VENDOR_LABEL_PADDING_TOP, VENDOR_LABEL_PADDING_BOTTOM, VENDOR_TEXT_REPLACEMENT_Y_OFFSET,
    VENDOR_PANEL_START_X, VENDOR_OPEN_SECONDS, VENDOR_RECENT_BUY_CAPACITY,
    VENDOR_UI_DEFAULT_IMAGE_PATHS
};
pub use containers::VENDOR_SOURCE_SERIALIZED_FILE;
pub use assets::{
    VENDOR_GAME_OBJECT_PATH_ID, VENDOR_CN_VENDOR_COMPONENT_PATH_ID,
    VENDOR_CN_VENDOR_SCRIPT_PATH_ID, VENDOR_PC_STUFF_COMPONENT_PATH_ID,
    VENDOR_PC_STUFF_SCRIPT_PATH_ID, VENDOR_PANEL_COMPONENT_PATH_ID,
    VENDOR_PANEL_SCRIPT_PATH_ID, VENDOR_EQUIP_COMPONENT_PATH_ID, VENDOR_EQUIP_SCRIPT_PATH_ID,
    VENDOR_INVENTORY_MANAGER_COMPONENT_PATH_ID, VENDOR_INVENTORY_SKIN_PATH_ID,
    VENDOR_PANEL_PATH, VENDOR_INFO_PATH, VENDOR_LIST_BACK_PATH, VENDOR_LIST_DIVIDER_PATH,
    VENDOR_ITEM_ROW_PATH, VENDOR_TAB_BUY_SELECTED_PATH, VENDOR_TAB_BUY_NORMAL_PATH,
    VENDOR_TAB_BUYBACK_SELECTED_PATH, VENDOR_TAB_BUYBACK_NORMAL_PATH, VENDOR_TAROS_ICON_PATH,
    VENDOR_DEXLABS_PATH, VENDOR_DEXLABS_SOURCE_PATH_ID, VENDOR_TAROS_COUNTER_PATH,
    VENDOR_TAROS_COUNTER_SOURCE_PATH_ID, VENDOR_BOOST_ICON_PATH,
    VENDOR_BOOST_ICON_SOURCE_PATH_ID, VENDOR_POTION_ICON_PATH,
    VENDOR_POTION_ICON_SOURCE_PATH_ID, VENDOR_SERVICE_FONT_PATH,
    VENDOR_LABEL_SOURCE_FONT_PATH_ID, VENDOR_EQUIP_SOURCE_FONT_PATH_ID,
    VENDOR_SERVICE_SOURCE_FONT_PATH_ID, VENDOR_CATALOG_CAPACITY, VENDOR_UI_Z_INDEX,
    VendorCatalogEntry0104, VendorCatalogRowProjection0104, VendorStaticAssetRole,
    VendorUiAssetContract, VendorStaticAssetReadiness, VendorUiAssetStatus
};
use assets::is_safe_relative_asset_path;
pub use materials::VENDOR_PRIMARY_FIRST_PASS_DLL_SHA256;
pub use interaction::{
    VENDOR_TAB_BUY_HOVER_PATH, VENDOR_TAB_BUYBACK_HOVER_PATH, VENDOR_BUTTON_NORMAL_PATH,
    VENDOR_BUTTON_HOVER_PATH, VENDOR_SCROLL_TRACK_PATH, VENDOR_SCROLL_THUMB_PATH,
    VENDOR_SCROLL_UP_PATH, VENDOR_SCROLL_DOWN_PATH, VENDOR_SCROLL_SHADOW_PATH,
    VENDOR_BUTTON_SOURCE_FONT_PATH_ID, VENDOR_BUTTON_FONT_SIZE, VENDOR_BUTTON_PADDING_LEFT,
    VENDOR_BUTTON_PADDING_RIGHT, VENDOR_BUTTON_PADDING_TOP, VENDOR_BUTTON_PADDING_BOTTOM,
    VENDOR_BUTTON_NORMAL_TEXT_RGB, VENDOR_BUTTON_HOVER_TEXT_RGB, VENDOR_SCROLL_VELOCITY,
    VENDOR_SCROLL_INPUT_CLAMP, VendorInputCapabilities, VendorScrollTarget, vendor_scroll_max,
    clamp_vendor_scroll
};
use interaction::{VendorHoverState, collect_vendor_ui_input, vendor_button_text_color};
pub use frame::{
    VENDOR_RESTRICTED_ITEM_FRAME_PATH, VENDOR_BATTERY_FRAME_RECTS, VendorSlotFrameVisual0104
};
use frame::vendor_slot_frame_image;
pub use layout::{
    VENDOR_LABEL_LINE_HEIGHT, VENDOR_BUTTON_LINE_HEIGHT, VENDOR_EQUIP_LINE_HEIGHT,
    VENDOR_SERVICE_LINE_HEIGHT, VENDOR_REFERENCE_WIDTH, VENDOR_REFERENCE_HEIGHT,
    VENDOR_BACKPLATE_REFERENCE_WIDTH, VENDOR_BACKPLATE_REFERENCE_HEIGHT, VENDOR_PANEL_WIDTH,
    VENDOR_PANEL_HEIGHT, VENDOR_ROW_HEIGHT, VENDOR_ROW_WIDTH, VENDOR_ROW_VISUAL_HEIGHT,
    VendorUiRect, VENDOR_NPC_PREVIEW_RECT, VENDOR_DIALOG_RECT, VENDOR_LIST_BACK_RECT,
    VENDOR_TABLE_RECT, VENDOR_LIST_VIEWPORT_RECT, VENDOR_TABLE_SHADOW_RECT, VENDOR_INFO_RECT,
    VENDOR_TITLE_RECT, VENDOR_SERVICE_RECT, VENDOR_BUY_TAB_RECT, VENDOR_BUY_TAB_HIT_RECT,
    VENDOR_BUYBACK_TAB_RECT, VENDOR_BUYBACK_TAB_HIT_RECT, VENDOR_LIST_DIVIDER_RECT,
    VENDOR_GO_TO_STUFF_RECT, VENDOR_ROW_ITEM_BOX_RECT, VENDOR_ROW_NAME_RECT,
    VENDOR_ROW_LEVEL_RECT, VENDOR_ROW_VEHICLE_SPEED_RECT, VENDOR_ROW_PRICE_RECT,
    VENDOR_ROW_PRICE_ICON_RECT, VENDOR_SCROLL_UP_RECT, VENDOR_SCROLL_DOWN_RECT,
    VENDOR_SCROLL_TRACK_RECT, VENDOR_PC_STUFF_INVENTORY_SHADOW_RECT,
    VENDOR_PC_STUFF_DEXLABS_RECT, VENDOR_PC_STUFF_TAROS_COUNTER_RECT,
    VENDOR_PC_STUFF_REDEEM_CODE_RECT, VENDOR_PC_STUFF_TAROS_DIGIT_RECTS,
    VENDOR_BATTERY_ICON_RECTS, VENDOR_BATTERY_LABEL_RECTS, VENDOR_BATTERY_COUNT_RECTS,
    VENDOR_LIST_BACK_BORDER, VENDOR_SCROLL_TRACK_BORDER, VENDOR_SCROLL_SHADOW_BORDER,
    VENDOR_BUTTON_BORDER, VendorModeLayout, vendor_mode_layout
};
use layout::bind_vendor_rect;
pub use commands::{
    VENDOR_HELP_EVENT_ID, VendorRequestIdentity0104, VendorActionOutcome0104,
    VendorItemActionPopup0104, VendorActionBlocked0104
};
pub use types_vendor_ui_text_style::{
    VendorUiTextStyle, VendorTab0104, VendorSession0104, VendorIconRef,
    VendorItemMetadata0104, VendorItemCatalog0104, VendorMissingCatalog0104,
    VendorEquipEligibility0104, VendorFailClosedEquipEligibility0104,
    VendorEquipValidation0104, VendorMissingIconReason0104, VendorPresentationIcon0104,
    VendorRecentBuyEntry0104, VendorRecentBuyRowProjection0104, VendorProjectedItem0104,
    VendorEquipmentSlotProjection0104, VendorProjectionError0104, VendorBuyIntent0104,
    VendorBuyGeneralIntent0104, VendorBatteryIntent0104, VendorRestoreIntent0104,
    VendorSellIntent0104, VendorDeleteIntent0104, VendorDisassembleIntent0104,
    VendorIntent0104, VendorSystemMessageId0104, VendorSystemMessageCallback0104,
    VendorSystemMessage0104, VendorConfirmationCallback0104, VendorConfirmation0104,
    VendorSilentBlock0104, VendorQuantityContract0104, VendorItemPopupSource0104,
    VendorPopupCommit0104, VendorActivationOutcome0104, VendorServerFailure0104,
    VendorLifecyclePhase, VendorCloseGate0104, VendorUiCommand0104, VendorUiOutbox0104,
    VendorRowView0104, VendorSlotView0104
};
use types_vendor_ui_text_style::VendorUiAssets;
pub use types_vendor_ui_element::{VendorUiRoot, VendorUiElement, VendorUiSet, VendorUiPlugin};
use types_vendor_ui_element::VendorInteractiveControl;
pub use validation::{VendorIconRefError, VendorUiAssetContractError};
use validation::equip_validation;
pub use state_vendor_mode_projection0104::{
    VendorInventorySlotProjection0104, VendorModeProjection0104, VendorInventoryLocation0104,
    VendorInventoryDropTarget0104, VendorModalState, VendorUiState
};
pub use state_vendor_ui_state::{VendorModeView0104, vendor_mode_view};
use operations::{
    projected_icon, recent_price, vendor_row_view, vendor_slot_view, dispatch_vendor_activation,
    equipment_slot_label, bind_vendor_ui, bind_vendor_text_value
};
#[cfg(test)]
use operations::bind_vendor_icon;
pub use operations::{
    vendor_server_failure_outcome, vendor_opening_eased_fraction,
    vendor_taros_counter_digit_0104, vendor_taros_counter_digits_0104
};
pub use audio::{VendorUiAudioCue0104, VendorUiAudioOutbox0104};
use systems::advance_vendor_lifecycle;
use view_vendor_panel::spawn_vendor_ui;
use view_vendor_equipment_slot::{spawn_vendor_equipment_slot, spawn_vendor_text};
use localization_equipment_slot_localized::{
    inventory_count_localized, equipment_slot_localized, vendor_tab_localized,
    vendor_title_localized, vendor_service_localized, vendor_item_name_localized,
    vendor_level_localized, vendor_price_localized, vendor_vehicle_speed_localized,
    inventory_quest_localized, vendor_taros_digit_localized, vendor_battery_count_localized
};
