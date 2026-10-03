//! Clean-Retrobution `BankMode` projection, intent boundary, and UI.
//!
//! Primary authority:
//! - `Panel_BankScript` owns the 495x638 left panel, 6x34/200-slot
//!   scroll grid, and one-second sine slide.
//! - `cnBank` treats `iExtraBank == 1` as full access; every other value
//!   leaves slots 60..199 visible but locked.
//! - `InventoryManagerScript.OneClickItem` selects the first empty slot in
//!   the opposite location.
//! - `cnBank.ReceiveItemMove` sends `sP_CL2FE_REQ_ITEM_MOVE`; only
//!   `UserSlot.ItemMove` applies the authoritative success values.
//!
//! This module therefore never speculatively mutates bank or inventory
//! records. It only projects complete server snapshots and emits checked
//! outbound commands for a production owner to consume.

mod drag;
mod item_delete;
mod item_popup;
mod search;
pub use item_delete::{BankItemDeleteIntent0104, BankItemDeleteState};
pub(crate) use search::BankSearch;
pub use item_popup::BankItemPopupState;

use crate::{
    inventory_runtime::{
        EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104, InventoryRuntime0104,
    },
    localization::{LocalizationSet, LocalizedText},
    user_equip_ui::{
        USER_EQUIP_BACKDROP_HEIGHT, USER_EQUIP_BACKDROP_PATH, USER_EQUIP_BACKDROP_WIDTH,
        USER_EQUIP_CLOSE_PATH, USER_EQUIP_CLOSE_RECT, USER_EQUIP_COMBINED_BADGE_LEFT,
        USER_EQUIP_COMBINED_BADGE_SIZE, USER_EQUIP_COMBINED_BADGE_TOP, USER_EQUIP_COMBINED_PATH,
        USER_EQUIP_COUNT_LABEL_LEFT, USER_EQUIP_COUNT_LABEL_TOP, USER_EQUIP_EQUIP_TITLE_PATH,
        USER_EQUIP_EQUIPMENT_CONTENT_RECT, USER_EQUIP_EQUIPMENT_SLOT_SIZE,
        USER_EQUIP_EQUIPMENT_SLOT_STRIDE, USER_EQUIP_EQUIPMENT_STRIP_COUNT,
        USER_EQUIP_EQUIPMENT_STRIP_ORDER, USER_EQUIP_EQUIPMENT_TITLE_RECT, USER_EQUIP_FONT_PATH,
        USER_EQUIP_HELP_PATH, USER_EQUIP_HELP_RECT, USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
        USER_EQUIP_INVENTORY_CONTENT_WIDTH, USER_EQUIP_INVENTORY_PANEL_BORDER,
        USER_EQUIP_INVENTORY_PANEL_PATH, USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_STRIDE, USER_EQUIP_INVENTORY_VIEWPORT_RECT,
        USER_EQUIP_ITEM_TAB_HIT_RECT, USER_EQUIP_MISSING_CHECKER_SIZE, USER_EQUIP_PC_STUFF_RECT,
        USER_EQUIP_REGULAR_FONT_LINE_HEIGHT, USER_EQUIP_RIGHT_PANEL_BORDER,
        USER_EQUIP_RIGHT_PANEL_PATH, USER_EQUIP_SLOT_EMPTY_PATH, USER_EQUIP_SLOT_OCCUPIED_PATH,
        USER_EQUIP_SMALL_FONT_LINE_HEIGHT, USER_EQUIP_SMALL_FONT_SIZE, USER_EQUIP_TRASH_PATH,
        USER_EQUIP_TRASH_RECT, UserEquipCatalogQuery, UserEquipCatalogQueryError, UserEquipIconRef,
        UserEquipItemCatalog, UserEquipItemIds, UserEquipItemModeLayout,
        UserEquipItemModeProjection, UserEquipMissingIconReason, UserEquipPresentationIcon,
        UserEquipProjectedIcon, UserEquipUiRect, user_equip_item_mode_layout,
        user_equip_missing_checker_rgba,
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
use ffone_protocol::{
    BANK_SLOT_COUNT_0104, ItemBase0104, ItemMoveRequest0104, ItemMoveSuccessPacket0104,
    PcBankOpenRequest0104, PcBankOpenSuccess0104, PcLoadData0104,
};
use std::{array, collections::VecDeque, error::Error, fmt};

use crate::ui_support::display_if;

use crate::ui_support::stretched_image;

use crate::ui_support::sliced_image;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cold_icon_regression;

mod interaction;
mod constants;
mod assets;
mod textures;
mod layout;
mod types_bank_ui_text_style;
mod types_bank_ui_plugin;
mod validation;
mod projection;
mod output;
mod state;
mod commands;
mod operations;
mod frame;
mod localization_bank_equipment_slot_localized;
mod input_collect_bank_local_controls;
mod systems;
mod view;

pub use interaction::{
    BankItemDeleteButton, BANK_SCROLL_TRACK_PATH, BANK_SCROLL_THUMB_PATH, BANK_SCROLL_UP_PATH,
    BANK_SCROLL_DOWN_PATH, BANK_SCROLL_SHADOW_PATH, BANK_SLOT_BUTTON_PATH,
    BANK_SLOT_BUTTON_SOURCE_PATH_ID, BANK_SLOT_BUTTON_SHA256, BANK_SCROLL_VELOCITY,
    BANK_BUTTON_SOURCE_FONT_PATH_ID, BANK_BUTTON_FONT_SIZE, BANK_BUTTON_PADDING_LEFT,
    BANK_BUTTON_PADDING_RIGHT, BANK_BUTTON_PADDING_TOP, BANK_BUTTON_PADDING_BOTTOM,
    BankInputCapabilities, BankScrollTarget, bank_scroll_max, clamp_bank_scroll
};
use interaction::{
    collect_bank_ui_input, collect_bank_slot_input, bind_bank_button_visual, bind_bank_button_labels
};
pub use constants::{
    BANK_SOURCE_BUILD, BANK_SOURCE_MAIN_ARCHIVE, BANK_SOURCE_MAIN_ARCHIVE_SHA256,
    BANK_SOURCE_TUTORIAL_ARCHIVE, BANK_SOURCE_TUTORIAL_ARCHIVE_SHA256, BANK_GUI_DEPTH_0104,
    BANK_DEXLABS_SHA256, BANK_TAROS_COUNTER_SHA256, BANK_OPEN_SECONDS, BANK_PANEL_START_X,
    BANK_GRID_COLUMNS, BANK_GRID_ROWS, BANK_SLOT_SIZE, BANK_SLOT_STRIDE,
    BANK_FULL_ACCESS_VALUE, BANK_HALF_ACCESS_SLOT_COUNT, BANK_LABEL_FONT_SIZE,
    BANK_EQUIP_FONT_SIZE, BANK_LABEL_PADDING_TOP, BANK_LABEL_PADDING_BOTTOM,
    BANK_TEXT_REPLACEMENT_Y_OFFSET, BANK_COUNT_LABEL_LEFT, BANK_COUNT_LABEL_TOP,
    BANK_UI_DEFAULT_IMAGE_PATHS
};
use constants::BANK_FULL_MESSAGE_OWNER;
pub use assets::{
    BANK_GAME_OBJECT_PATH_ID, BANK_CONTROLLER_COMPONENT_PATH_ID,
    BANK_CONTROLLER_SCRIPT_PATH_ID, BANK_PANEL_COMPONENT_PATH_ID, BANK_PANEL_SCRIPT_PATH_ID,
    BANK_PC_STUFF_COMPONENT_PATH_ID, BANK_PC_STUFF_SCRIPT_PATH_ID,
    BANK_EQUIP_COMPONENT_PATH_ID, BANK_EQUIP_SCRIPT_PATH_ID,
    BANK_INVENTORY_MANAGER_COMPONENT_PATH_ID, BANK_INVENTORY_SKIN_PATH_ID,
    BANK_TRADE_BACK_PATH, BANK_PANEL_PATH, BANK_INFO_PATH, BANK_LOCKED_SLOT_PATH,
    BANK_DEXLABS_PATH, BANK_DEXLABS_SOURCE_PATH_ID, BANK_TAROS_COUNTER_PATH,
    BANK_TAROS_COUNTER_SOURCE_PATH_ID, BANK_UI_Z_INDEX, BANK_LABEL_SOURCE_FONT_PATH_ID,
    BANK_EQUIP_SOURCE_FONT_PATH_ID, BANK_BACKGROUND_ONLY_FONT_PATH_ID, BankStaticAssetRole,
    BankUiAssetContract, BANK_SEARCH_FONT_PATH
};
use assets::EmptyBankCatalog;
#[cfg(test)]
use assets::is_safe_relative_asset_path;
pub use textures::BANK_SLOT_BUTTON_RGBA_SHA256;
pub use layout::{
    BANK_REFERENCE_WIDTH, BANK_REFERENCE_HEIGHT, BANK_BACKPLATE_REFERENCE_WIDTH,
    BANK_BACKPLATE_REFERENCE_HEIGHT, BANK_PANEL_WIDTH, BANK_PANEL_HEIGHT, BANK_CONTENT_WIDTH,
    BANK_CONTENT_HEIGHT, BANK_VIEWPORT_WIDTH, BANK_VIEWPORT_HEIGHT, BANK_LABEL_LINE_HEIGHT,
    BANK_BUTTON_LINE_HEIGHT, BANK_EQUIP_LINE_HEIGHT, BANK_INFO_RECT, BANK_TITLE_RECT,
    BANK_DIALOG_RECT, BANK_TAB_RECT, BANK_VIEWPORT_RECT, BANK_SHADOW_RECT,
    BANK_SCROLL_UP_RECT, BANK_SCROLL_DOWN_RECT, BANK_SCROLL_TRACK_RECT,
    BANK_PC_STUFF_DEXLABS_RECT, BANK_PC_STUFF_TAROS_COUNTER_RECT,
    BANK_PC_STUFF_REDEEM_CODE_RECT, BANK_PC_STUFF_TAROS_DIGIT_RECTS, BANK_PANEL_BORDER,
    BANK_SHADOW_BORDER, BANK_SLOT_BUTTON_BORDER, BankUiRect, BankModeLayout, bank_mode_layout
};
use layout::bind_rect;
pub use types_bank_ui_text_style::{
    BankUiTextStyle, BankSlotLocation0104, BankSlotRef0104, BankSlotRefError0104,
    BankEquipEligibility, BankFailClosedEquipEligibility, BankSlotProjection0104,
    BankAuthoritativeSnapshot0104, BankAuthorityMutationReceipt0104,
    BankAuthorityMutationError0104, BankPcStuffAuthority0104, BankProjectionError0104,
    BankTransferIntent0104, BankTransferError0104, BankLifecyclePhase, BankUiCommand0104,
    BankUiOutbox0104, BankItemView, BankUiRoot
};
use types_bank_ui_text_style::BankUiAssets;
pub use types_bank_ui_plugin::{BankUiElement, BankDisabledControl, BankUiSet, BankUiPlugin};
use types_bank_ui_plugin::{
    BankTarosDigitText0104, BankCloseControl, BankHelpControl, BankRedeemControl,
    BankLocalRequests, BankSlotControl
};
pub use validation::{BankEquipValidation, BankUiAssetContractError};
use validation::{bank_item_needs_equip_validation, validate_authoritative_bank_item};
use projection::project_bank_icon;
use output::write_item_base_into_pc_load;
pub use state::{
    BankModeProjection0104, BankModalState, BankUiState, BankModeView, bank_mode_view,
    bank_mode_view_with_pc_stuff
};
use state::BankUiRuntimeAssets;
pub use commands::BankActionBlocked;
pub use operations::{
    bank_opening_eased_fraction, bank_taros_counter_digit_0104, bank_taros_counter_digits_0104
};
use operations::{equipment_slot_label, bind_bank_ui, bind_presentation_icon};
pub use frame::BankSlotFrameVisual;
use frame::{bank_slot_frame_image, inventory_slot_frame_image};
use localization_bank_equipment_slot_localized::{
    bank_full_localized, bank_static_localized, bank_count_localized,
    bank_taros_digit_localized, bank_equipment_slot_localized
};
use input_collect_bank_local_controls::collect_bank_local_controls;
use systems::advance_bank_lifecycle;
use view::spawn_bank_ui;
