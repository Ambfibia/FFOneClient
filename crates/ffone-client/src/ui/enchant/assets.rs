use super::*;

pub const ENCHANT_GAME_OBJECT_PATH_ID: i64 = 1_291;

pub const ENCHANT_TRANSFORM_PATH_ID: i64 = 1_186;

pub const ENCHANT_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_476;

pub const ENCHANT_EQUIP_COMPONENT_PATH_ID: i64 = 1_477;

pub const ENCHANT_MODE_COMPONENT_PATH_ID: i64 = 1_478;

pub const ENCHANT_GUI_COMPONENT_PATH_ID: i64 = 1_479;

pub const ENCHANT_PRIMARY_CAMERA_PATH_ID: i64 = 1_480;

pub const ENCHANT_PRIMARY_CAMERA_CONTROLLER_PATH_ID: i64 = 1_481;

pub const ENCHANT_WAITING_CAMERA_PATH_ID: i64 = 1_482;

pub const ENCHANT_WAITING_CAMERA_CONTROLLER_PATH_ID: i64 = 1_483;

pub const ENCHANT_SKIN_PATH_ID: i64 = 1_367;

pub const ENCHANT_SKIN_FONT_PATH_ID: i64 = 1_008;

pub const ENCHANT_PANEL_PATH: &str = "ui/en/enchant/EnchantBG.png";

pub const ENCHANT_RAW_COVER_PATH: &str = "ui/en/enchant/EnchantCover.png";

pub const ENCHANT_ITEM_COVER_PATH: &str = "ui/en/enchant/EnchantItemCover.png";

pub const ENCHANT_QUANTITY_COVER_PATH: &str = "ui/en/enchant/EnchantRawCover.png";

pub const ENCHANT_X_MARK_PATH: &str = "ui/en/enchant/EnchantXmark.png";

pub const ENCHANT_LEVEL_BADGE_PATH: &str = "ui/en/enchant/EnchantLevel.png";

pub const ENCHANT_WAITING_PATH: &str = "ui/en/enchant/EnchantWaiting.png";

pub const ENCHANT_WAIT_PROGRESS_PATH: &str = "ui/en/enchant/EnchantWaitProgress.png";

pub const ENCHANT_BLACK_SHADE_PATH: &str = "ui/en/combi/black-shade.png";

pub const ENCHANT_SUCCESS_PATH: &str = "ui/en/combi/success.png";

pub const ENCHANT_NPC_ICON_PATH: &str = "ui/en/combi/npc-icon.png";

pub const ENCHANT_BACKDROP_PATH: &str = "ui/en/user-equip/backdrop.png";

pub const ENCHANT_RIGHT_PANEL_PATH: &str = "ui/en/user-equip/right-panel.png";

pub const ENCHANT_INVENTORY_PANEL_PATH: &str = "ui/en/user-equip/inventory-panel.png";

pub const ENCHANT_SLOT_OCCUPIED_PATH: &str = "ui/en/user-equip/slot-occupied.png";

pub const ENCHANT_SLOT_EMPTY_PATH: &str = "ui/en/user-equip/slot-empty.png";

pub const ENCHANT_EQUIP_TITLE_PATH: &str = "ui/en/user-equip/equip-title.png";

pub const ENCHANT_CLOSE_PATH: &str = "ui/en/user-equip/close.png";

pub const ENCHANT_TRASH_PATH: &str = "ui/en/user-equip/trash.png";

pub const ENCHANT_HELP_PATH: &str = "ui/en/user-equip/help.png";

pub const ENCHANT_COMBINED_PATH: &str = "ui/en/user-equip/combined.png";

pub const ENCHANT_DEXLABS_PATH: &str = "ui/en/enchant/dexlabsbut.png";

pub const ENCHANT_TAROS_COUNTER_PATH: &str = "ui/en/enchant/Taros.png";

pub const ENCHANT_BOOST_ICON_PATH: &str = "ui/en/enchant/BoostIcon.png";

pub const ENCHANT_POTION_ICON_PATH: &str = "ui/en/enchant/PostionIcon.png";

pub const ENCHANT_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const ENCHANT_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const ENCHANT_RECIPE_TABLE_PATH: &str = crate::assets::TABLE_SET_PATH;

pub const ENCHANT_UI_Z_INDEX_0104: i32 = 19;

pub const ENCHANT_INVENTORY_SKIN_PATH_ID_0104: i64 = 1_366;

pub const ENCHANT_JEFFE_13_FONT_PATH_ID_0104: i64 = 1_168;

pub const ENCHANT_CHALET_14_FONT_PATH_ID_0104: i64 = 1_148;

pub const ENCHANT_JEFFE_12_FONT_PATH_ID_0104: i64 = 977;

pub const ENCHANT_CHALET_SMALL_FONT_PATH_ID_0104: i64 = 949;

pub const ENCHANT_JEFFE_40_FONT_PATH_ID_0104: i64 = 905;

pub const ENCHANT_JEFFE_08_FONT_PATH_ID_0104: i64 = 1_127;

pub const ENCHANT_INVENTORY_SMALL_FONT_PATH_ID_0104: i64 = 970;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnchantStaticAssetRole0104 {
    Backdrop,
    Panel,
    RightBackplate,
    BlackShade,
    RawCover,
    QuantityCover,
    XMark,
    LevelBadge,
    Waiting,
    WaitProgress,
    Success,
    NpcIcon,
    ButtonNormal,
    ButtonHover,
    Combined,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    RestrictedItemFrame,
    EquipTitle,
    Close,
    Trash,
    Help,
    DexlabsBanner,
    TarosCounter,
    BoostIcon,
    PotionIcon,
}

impl EnchantStaticAssetRole0104 {
    pub const COUNT: usize = 27;

    pub(super) const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EnchantStaticAssetReadiness0104 {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct EnchantUiAssetStatus0104(pub EnchantStaticAssetReadiness0104);

#[must_use]
pub fn enchant_safe_relative_asset_path_0104(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && !path.split('/').any(|part| part.is_empty() || part == "..")
        && !path.contains(':')
}
