
pub const EMAIL_UI_GAME_OBJECT_PATH_ID: i64 = 1_275;

pub const EMAIL_UI_LIST_COMPONENT_PATH_ID: i64 = 1_553;

pub const EMAIL_UI_COMPOSE_COMPONENT_PATH_ID: i64 = 1_554;

pub const EMAIL_UI_MODE_COMPONENT_PATH_ID: i64 = 1_555;

pub const EMAIL_UI_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_556;

pub const EMAIL_UI_LIST_SCRIPT_PATH_ID: i64 = 1_153;

pub const EMAIL_UI_COMPOSE_SCRIPT_PATH_ID: i64 = 1_125;

pub const EMAIL_UI_MODE_SCRIPT_PATH_ID: i64 = 975;

pub const EMAIL_UI_PC_STUFF_SCRIPT_PATH_ID: i64 = 1_045;

pub const EMAIL_UI_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const EMAIL_UI_LIST_PATH: &str = "ui/en/email/list.png";

pub const EMAIL_UI_GUIDE_TAB_PATH: &str = "ui/en/email/guide-tab.png";

pub const EMAIL_UI_PLAYER_TAB_PATH: &str = "ui/en/email/player-tab.png";

pub const EMAIL_UI_INACTIVE_TAB_FILL_PATH: &str = "ui/en/email/inactive-tab-fill.png";

pub const EMAIL_UI_ATTACHMENT_PATH: &str = "ui/en/email/attachment.png";

pub const EMAIL_UI_BUDDY_POPUP_PATH: &str = "ui/en/email/buddy-popup.png";

pub const EMAIL_UI_CALCULATOR_PAD_PATH: &str = "ui/en/email/calculator-pad.png";

pub const EMAIL_UI_CALCULATOR_POPUP_PATH: &str = "ui/en/email/calculator-popup.png";

pub const EMAIL_UI_DATA_BACK_PATH: &str = "ui/en/email/data-back.png";

pub const EMAIL_UI_DATA_BOX_PATH: &str = "ui/en/email/data-box.png";

pub const EMAIL_UI_SORT_ARROW_PATH: &str = "ui/en/email/sort-arrow.png";

pub const EMAIL_UI_TAROS_PATH: &str = "ui/en/email/taros.png";

pub const EMAIL_UI_PREVIOUS_PATH: &str = "ui/en/email/previous.png";

pub const EMAIL_UI_NEXT_PATH: &str = "ui/en/email/next.png";

pub const EMAIL_UI_COMPOSE_PATH: &str = "ui/en/email/compose.png";

pub const EMAIL_UI_LIST_SELECTION_PATH: &str = "ui/en/email/list-selection.png";

pub const EMAIL_UI_BACKDROP_PATH: &str = "ui/en/user-equip/backdrop.png";

pub const EMAIL_UI_RIGHT_PANEL_PATH: &str = "ui/en/user-equip/right-panel.png";

pub const EMAIL_UI_INVENTORY_PANEL_PATH: &str = "ui/en/user-equip/inventory-panel.png";

pub const EMAIL_UI_SLOT_OCCUPIED_PATH: &str = "ui/en/user-equip/slot-occupied.png";

pub const EMAIL_UI_SLOT_EMPTY_PATH: &str = "ui/en/user-equip/slot-empty.png";

pub const EMAIL_UI_CLOSE_PATH: &str = "ui/en/user-equip/close.png";

/// Approved Cyrillic-capable replacement for the clean JEFFE bitmap Fonts.
pub const EMAIL_UI_FONT_PATH: &str = "fonts/jeffe.otf";

/// Approved Cyrillic-capable replacement for clean `ChaletBook-Regular Small`.
pub const EMAIL_UI_BODY_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

// `FusionFallInvenSkin` path ID 1366 is the authority for these fixed-raster
// Font references and GUIStyle metrics. The outline-font sizes are the
// independently validated replacements already used by the PC2PC slice, which
// is owned by the same clean skin; line heights retain this skin's serialized
// fixed-raster values.
pub const EMAIL_UI_JEFFE_12_SOURCE_FONT_PATH_ID: i64 = 977;

pub const EMAIL_UI_JEFFE_14_SOURCE_FONT_PATH_ID: i64 = 933;

pub const EMAIL_UI_JEFFE_16_SOURCE_FONT_PATH_ID: i64 = 1_008;

pub const EMAIL_UI_JEFFE_06_SOURCE_FONT_PATH_ID: i64 = 970;

pub const EMAIL_UI_CHALET_SMALL_SOURCE_FONT_PATH_ID: i64 = 949;

pub const EMAIL_UI_Z_INDEX: i32 = 20;

/// Clean `Panel_NewEmail` draws its buddy/calculator windows at GUI depth 8,
/// above the inventory and email backing passes (depths 9-11).
pub const EMAIL_UI_POPUP_Z_INDEX: i32 = EMAIL_UI_Z_INDEX + 1;
