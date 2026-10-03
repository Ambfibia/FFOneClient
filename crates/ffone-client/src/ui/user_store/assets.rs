use super::*;

pub const USER_STORE_GAME_OBJECT_PATH_ID: i64 = 1_312;

pub const USER_STORE_CN_STORE_COMPONENT_PATH_ID: i64 = 1_434;

pub const USER_STORE_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_435;

pub const USER_STORE_PANEL_COMPONENT_PATH_ID: i64 = 1_436;

pub const USER_STORE_EQUIP_COMPONENT_PATH_ID: i64 = 1_437;

pub const USER_STORE_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const USER_STORE_UI_Z_INDEX: i32 = 20;

pub const USER_STORE_BACKDROP_PATH: &str = "ui/en/user-equip/backdrop.png";

pub const USER_STORE_LEFT_PANEL_PATH: &str = "ui/en/vendor/vendor-panel.png";

pub const USER_STORE_INFO_PATH: &str = "ui/en/vendor/info.png";

pub const USER_STORE_LIST_BACK_PATH: &str = "ui/en/vendor/list-back.png";

pub const USER_STORE_LIST_DIVIDER_PATH: &str = "ui/en/vendor/list-divider.png";

pub const USER_STORE_ITEM_ROW_PATH: &str = "ui/en/vendor/item-row.png";

pub const USER_STORE_ITEM_TAB_PATH: &str = "ui/en/vendor/tab-buy-selected.png";

pub const USER_STORE_SHADOW_PATH: &str = "ui/en/vendor/scroll-shadow.png";

pub const USER_STORE_RIGHT_PANEL_PATH: &str = "ui/en/user-equip/right-panel.png";

pub const USER_STORE_INVENTORY_PANEL_PATH: &str = "ui/en/user-equip/inventory-panel.png";

pub const USER_STORE_SLOT_OCCUPIED_PATH: &str = "ui/en/user-equip/slot-occupied.png";

pub const USER_STORE_SLOT_EMPTY_PATH: &str = "ui/en/user-equip/slot-empty.png";

pub const USER_STORE_EQUIP_TITLE_PATH: &str = "ui/en/user-equip/equip-title.png";

pub const USER_STORE_CLOSE_PATH: &str = "ui/en/user-equip/close.png";

pub const USER_STORE_TRASH_PATH: &str = "ui/en/user-equip/trash.png";

pub const USER_STORE_HELP_PATH: &str = "ui/en/user-equip/help.png";

pub const USER_STORE_POPUP_BACKGROUND_PATH: &str = "ui/en/user-store/gumdlg.png";

pub const USER_STORE_POPUP_CALCULATOR_PATH: &str = "ui/en/email/calculator-pad.png";

pub const USER_STORE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const USER_STORE_BODY_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

// Clean `FusionFallInvenSkin` path ID 1366 owns these GUIStyle font objects.
// The outline-font sizes are the validated Cyrillic-capable replacements; the
// serialized fixed-raster line spacing remains authoritative.
pub const USER_STORE_JEFFE_12_SOURCE_FONT_PATH_ID: i64 = 977;

pub const USER_STORE_JEFFE_14_SOURCE_FONT_PATH_ID: i64 = 933;

pub const USER_STORE_JEFFE_16_SOURCE_FONT_PATH_ID: i64 = 1_008;

pub const USER_STORE_CHALET_SMALL_SOURCE_FONT_PATH_ID: i64 = 949;

pub const USER_STORE_SHARED_ASSET_PATHS: [&str; 17] = [
    USER_STORE_BACKDROP_PATH,
    USER_STORE_LEFT_PANEL_PATH,
    USER_STORE_INFO_PATH,
    USER_STORE_LIST_BACK_PATH,
    USER_STORE_LIST_DIVIDER_PATH,
    USER_STORE_ITEM_ROW_PATH,
    USER_STORE_ITEM_TAB_PATH,
    USER_STORE_BUTTON_NORMAL_PATH,
    USER_STORE_BUTTON_HOVER_PATH,
    USER_STORE_RESTRICTED_FRAME_PATH,
    USER_STORE_SHADOW_PATH,
    USER_STORE_RIGHT_PANEL_PATH,
    USER_STORE_INVENTORY_PANEL_PATH,
    USER_STORE_SLOT_OCCUPIED_PATH,
    USER_STORE_SLOT_EMPTY_PATH,
    USER_STORE_EQUIP_TITLE_PATH,
    USER_STORE_CLOSE_PATH,
];

pub const USER_STORE_ALL_ASSET_PATHS: [&str; 24] = [
    USER_STORE_BACKDROP_PATH,
    USER_STORE_LEFT_PANEL_PATH,
    USER_STORE_INFO_PATH,
    USER_STORE_LIST_BACK_PATH,
    USER_STORE_LIST_DIVIDER_PATH,
    USER_STORE_ITEM_ROW_PATH,
    USER_STORE_ITEM_TAB_PATH,
    USER_STORE_BUTTON_NORMAL_PATH,
    USER_STORE_BUTTON_HOVER_PATH,
    USER_STORE_RESTRICTED_FRAME_PATH,
    USER_STORE_SHADOW_PATH,
    USER_STORE_RIGHT_PANEL_PATH,
    USER_STORE_INVENTORY_PANEL_PATH,
    USER_STORE_SLOT_OCCUPIED_PATH,
    USER_STORE_SLOT_EMPTY_PATH,
    USER_STORE_EQUIP_TITLE_PATH,
    USER_STORE_CLOSE_PATH,
    USER_STORE_TRASH_PATH,
    USER_STORE_HELP_PATH,
    USER_STORE_POPUP_BACKGROUND_PATH,
    USER_STORE_POPUP_CALCULATOR_PATH,
    USER_STORE_POPUP_CLOSE_HOVER_PATH,
    USER_STORE_FONT_PATH,
    USER_STORE_BODY_FONT_PATH,
];

pub const USER_STORE_IMAGE_ASSET_PATHS: [&str; 22] = [
    USER_STORE_BACKDROP_PATH,
    USER_STORE_LEFT_PANEL_PATH,
    USER_STORE_INFO_PATH,
    USER_STORE_LIST_BACK_PATH,
    USER_STORE_LIST_DIVIDER_PATH,
    USER_STORE_ITEM_ROW_PATH,
    USER_STORE_ITEM_TAB_PATH,
    USER_STORE_BUTTON_NORMAL_PATH,
    USER_STORE_BUTTON_HOVER_PATH,
    USER_STORE_RESTRICTED_FRAME_PATH,
    USER_STORE_SHADOW_PATH,
    USER_STORE_RIGHT_PANEL_PATH,
    USER_STORE_INVENTORY_PANEL_PATH,
    USER_STORE_SLOT_OCCUPIED_PATH,
    USER_STORE_SLOT_EMPTY_PATH,
    USER_STORE_EQUIP_TITLE_PATH,
    USER_STORE_CLOSE_PATH,
    USER_STORE_TRASH_PATH,
    USER_STORE_HELP_PATH,
    USER_STORE_POPUP_BACKGROUND_PATH,
    USER_STORE_POPUP_CALCULATOR_PATH,
    USER_STORE_POPUP_CLOSE_HOVER_PATH,
];

pub const USER_STORE_POPUP_LOCAL_Z_INDEX: i32 = USER_STORE_PANEL_DEPTH - USER_STORE_POPUP_DEPTH;

pub const USER_STORE_POPUP_BACKGROUND_PATH_ID: i64 = 390;

pub const USER_STORE_POPUP_CALCULATOR_PATH_ID: i64 = 385;

pub(super) fn list_slot_index(list_slot: i32) -> Result<usize, UserStoreCodecError0104> {
    usize::try_from(list_slot)
        .ok()
        .filter(|slot| *slot < USER_STORE_LIST_CAPACITY)
        .ok_or(UserStoreCodecError0104::InvalidListSlot(list_slot))
}

pub(super) fn inventory_slot_index(inventory_slot: i32) -> Option<usize> {
    usize::try_from(inventory_slot)
        .ok()
        .filter(|slot| *slot < USER_STORE_INVENTORY_CAPACITY)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UserStoreStaticAssetReadiness0104 {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserStoreUiAssetStatus0104(pub UserStoreStaticAssetReadiness0104);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(super) enum UserStoreAssetRole0104 {
    Backdrop,
    StoreBackplate,
    Info,
    ListBack,
    ListDivider,
    ItemRow,
    ItemTab,
    ButtonNormal,
    ButtonHover,
    RestrictedFrame,
    Shadow,
    RightBackplate,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    EquipTitle,
    Close,
    Trash,
    Help,
    PopupBackground,
    PopupCalculator,
    PopupCloseHover,
}

impl UserStoreAssetRole0104 {
    pub(super) const COUNT: usize = 22;

    pub(super) const fn index(self) -> usize {
        self as usize
    }
}
