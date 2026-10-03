use super::*;

pub const COMBI_SOURCE_BUILD: &str = "retrobution-20260613";

pub const COMBI_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";

pub const COMBI_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const COMBI_NPC_ID_0104: i32 = 3_219;

pub const COMBI_NPC_TYPE_0104: i32 = 26;

pub const COMBI_FIRST_USE_CONDITION_0104: i32 = 67;

pub const COMBI_ITEM_BASE_SIZE_0104: usize = 12;

pub const COMBI_SUCCESS_NEW_ITEM_SLOT_OFFSET_0104: usize = 0;

pub const COMBI_SUCCESS_NEW_ITEM_OFFSET_0104: usize = 4;

pub const COMBI_SUCCESS_STAT_SLOT_OFFSET_0104: usize = 16;

pub const COMBI_SUCCESS_CASH_SLOT_1_OFFSET_0104: usize = 20;

pub const COMBI_SUCCESS_CASH_SLOT_2_OFFSET_0104: usize = 24;

pub const COMBI_SUCCESS_TAROS_OFFSET_0104: usize = 28;

pub const COMBI_SUCCESS_FLAG_OFFSET_0104: usize = 32;

pub const COMBI_FAILURE_COSTUME_SLOT_OFFSET_0104: usize = 4;

pub const COMBI_FAILURE_STAT_SLOT_OFFSET_0104: usize = 8;

pub const COMBI_FAILURE_CASH_SLOT_1_OFFSET_0104: usize = 12;

pub const COMBI_FAILURE_CASH_SLOT_2_OFFSET_0104: usize = 16;

pub const COMBI_STYLE_SLOT_TYPE_0104: i32 = 19;

pub const COMBI_STATS_SLOT_TYPE_0104: i32 = 20;

pub const COMBI_WAIT_SECONDS_0104: f32 = 4.0;

pub const COMBI_RECIPE_ROW_COUNT_0104: usize = 38;

pub const COMBI_RECIPE_MAX_LEVEL_GAP_0104: usize = 36;

pub const COMBI_RECIPE_TABLE_SHA256: &str =
    "0041ee74a52ffa76a6d34b533218399d136f178d04c9ffa00aad04e69a403a7c";

const _: [(); INVENTORY_SLOT_COUNT_0104] = [(); 50];

const _: [(); EQUIPMENT_SLOT_COUNT_0104] = [(); 9];

const _: [(); COMBI_ITEM_BASE_SIZE_0104] = [(); ItemBase0104::SIZE];

const _: [(); USER_EQUIP_EQUIPMENT_STRIP_COUNT] = [(); EQUIPMENT_SLOT_COUNT_0104];

pub const COMBI_MESSAGE_254: &str = "ATTEMPTING TO COMBINE!\nIf successful, the two items you combine will be replaced with a brand-new item. If unsuccessful, you will keep your original items but still spend Taros. You can always make another combination attempt later.";

pub const COMBI_MESSAGE_255: &str = "OOPS!\nThe combination failed.  But you can always try again.  Your items are still available to combine, equip, sell, or trade.";

pub const COMBI_MESSAGE_256: &str =
    "NOT ENOUGH TAROS \nYou do not have enough Taros to make this combination attempt.";

pub const COMBI_MESSAGE_260: &str = "CANNOT MOVE TO CROC POT\nYou must unequip this item before adding it to the Croc Pot.\nExit and go to MY STUFF to unequip the item.";

pub const COMBI_COLOR_DEFAULT: CombiRgb0104 = CombiRgb0104::new(204.0 / 255.0, 1.0, 1.0);

pub const COMBI_COLOR_GREEN: CombiRgb0104 = CombiRgb0104::new(0.0, 1.0, 0.0);

pub const COMBI_COLOR_RED: CombiRgb0104 = CombiRgb0104::new(0.9, 0.2, 0.0);

pub const COMBI_COLOR_NOT_READY: CombiRgb0104 = CombiRgb0104::new(0.15, 0.31, 0.5);

pub const COMBI_COLOR_NOT_POSSIBLE: CombiRgb0104 = CombiRgb0104::new(0.6, 0.0, 0.0);

pub const COMBI_UI_DEFAULT_IMAGE_PATHS: [&str; CombiStaticAssetRole::COUNT] = [
    USER_EQUIP_BACKDROP_PATH,
    COMBI_PANEL_PATH,
    USER_EQUIP_RIGHT_PANEL_PATH,
    COMBI_BLACK_SHADE_PATH,
    COMBI_LOOK_ITEM_BG_PATH,
    COMBI_LOOK_ERROR_PATH,
    COMBI_STAT_ITEM_BG_PATH,
    COMBI_STAT_ERROR_PATH,
    COMBI_COMBINED_PATH,
    COMBI_TAROS_ICON_PATH,
    COMBI_SUCCESS_PATH,
    COMBI_NPC_ICON_PATH,
    COMBI_WAITING_PATH,
    COMBI_BUTTON_NORMAL_PATH,
    COMBI_BUTTON_HOVER_PATH,
    COMBI_RESTRICTED_ITEM_FRAME_PATH,
    USER_EQUIP_INVENTORY_PANEL_PATH,
    USER_EQUIP_SLOT_OCCUPIED_PATH,
    USER_EQUIP_SLOT_EMPTY_PATH,
    USER_EQUIP_EQUIP_TITLE_PATH,
    USER_EQUIP_CLOSE_PATH,
    USER_EQUIP_TRASH_PATH,
    USER_EQUIP_HELP_PATH,
    USER_EQUIP_CLOSE_HOVER_PATH,
    USER_EQUIP_HELP_HOVER_PATH,
];
