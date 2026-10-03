use super::*;

pub const PC2PC_SOURCE_BUILD: &str = "retrobution-20260613";

pub const PC2PC_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";

pub const PC2PC_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const PC2PC_SOURCE_TUTORIAL_ARCHIVE: &str = "Tutorial.resourceFile";

pub const PC2PC_SOURCE_TUTORIAL_ARCHIVE_SHA256: &str =
    "49A684FF4236848D0B882A5D725FFBE99D5CFB5D2090DC8705350E97DD3FD024";

pub const PC2PC_OPEN_SECONDS: f32 = 1.0;

pub const PC2PC_PANEL_START_X: i32 = -475;

pub const PC2PC_OFFER_SLOT_COUNT: usize = 5;

pub const PC2PC_PROTOCOL_TRADE_ITEM_COUNT: usize = 12;

pub const PC2PC_OFFER_SLOT_SIZE: f32 = 62.0;

pub const PC2PC_OFFER_SLOT_STRIDE: f32 = 67.0;

pub const PC2PC_CHAT_MAX_STORED_LINES: usize = 41;

pub const PC2PC_JEFFE_12_FONT_SIZE: f32 = 12.0;

pub const PC2PC_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const PC2PC_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const PC2PC_CHALET_SMALL_FONT_SIZE: f32 = 12.0;

pub const PC2PC_LABEL_PADDING_TOP: f32 = 3.0;

pub const PC2PC_LABEL_PADDING_BOTTOM: f32 = 3.0;

pub const PC2PC_READY_NAME_GAP: f32 = 5.0;

const _: [(); INVENTORY_SLOT_COUNT_0104] = [(); 50];

const _: [(); EQUIPMENT_SLOT_COUNT_0104] = [(); 9];

const _: [(); USER_EQUIP_EQUIPMENT_STRIP_COUNT] = [(); 9];

pub const PC2PC_LOCAL_SLOT_ORIGIN: Vec2 = Vec2::new(51.0, 36.0);

pub const PC2PC_REMOTE_SLOT_ORIGIN: Vec2 = Vec2::new(118.0, 37.0);

pub const PC2PC_BLOCKED_CHAT_COMMANDS: &[&str] = &[
    "/emote",
    "/dance",
    "/motd",
    "/announce",
    "/bcast",
    "/nano_equip",
    "/nano_unequip",
    "nano_active",
    "/speed",
    "/jump",
    "/warp",
    "/goto",
    "/warptopc",
    "/itemN",
    "/itemQ",
    "/mapwarp",
    "/nano",
    "/nanoArr",
    "/summon",
    "/groupsummon",
    "/summonshiny",
    "/unsummon",
    "/nanoskill",
    "/mission",
    "/task",
    "/unstick_n",
    "/unstick_i",
    "/unstick_ui",
    "/locate_i",
    "/locate_ui",
    "/locate_n",
    "/teleport2me_n",
    "/teleport2me_i",
    "/teleport2me_ui",
    "/teleportXYZ_i",
    "/teleportXYZ_ui",
    "/teleportXYZ_n",
    "/teleportMapXYZ_i",
    "/teleportMapXYZ_n",
    "/teleportMapXYZ_ui",
    "/teleport_i_i",
    "/teleport_ui_ui",
    "/teleport_i_n",
    "/teleport_n_n",
    "/kick_i",
    "/kick_ui",
    "/kick_n",
    "/invisible",
    "/invulnerable",
    "/health",
    "/batteryW",
    "/batteryN",
    "/fusionmatter",
    "/taros",
    "/gmmarker",
    "/equipitem",
    "/viewloc",
    "/viweloc",
    "/mute_i_on",
    "/mute_i_off",
    "/mute_ui_on",
    "/mute_ui_off",
    "/mute_n_on",
    "/mute_n_off",
    "/hideui",
    "/viewcol",
    "/unstick",
    "/rateF",
    "/rateT",
];

pub const PC2PC_UI_DEFAULT_IMAGE_PATHS: [&str; Pc2pcStaticAssetRole::COUNT] = [
    PC2PC_TRADE_BACK_PATH,
    PC2PC_TRADE_AREA_PATH,
    PC2PC_LOCAL_OFFER_PATH,
    PC2PC_LOCAL_OFFER_READY_PATH,
    PC2PC_REMOTE_OFFER_PATH,
    PC2PC_REMOTE_OFFER_READY_PATH,
    PC2PC_CHAT_BOX_PATH,
    PC2PC_MONEY_BACK_PATH,
    PC2PC_CHAT_SEND_PATH,
    PC2PC_CHAT_SEND_HOVER_PATH,
    PC2PC_TAROS_PATH,
    PC2PC_FREE_CHAT_PATH,
    PC2PC_PORTRAIT_BACK_PATH,
    "ui/en/pc2pc/offer-rejected.png",
    PC2PC_BUTTON_PATH,
    PC2PC_BUTTON_HOVER_PATH,
    PC2PC_TEXT_FIELD_PATH,
    USER_EQUIP_BACKDROP_PATH,
    USER_EQUIP_RIGHT_PANEL_PATH,
    USER_EQUIP_INVENTORY_PANEL_PATH,
    USER_EQUIP_SLOT_OCCUPIED_PATH,
    USER_EQUIP_SLOT_EMPTY_PATH,
    USER_EQUIP_EQUIP_TITLE_PATH,
    USER_EQUIP_NANO_TAB_PATH,
    USER_EQUIP_CLOSE_PATH,
    USER_EQUIP_TRASH_PATH,
    USER_EQUIP_HELP_PATH,
    USER_EQUIP_COMBINED_PATH,
];
