use super::*;

pub const CASHMALL_SOURCE_BUILD: &str = "retrobution-20260613";

pub const CASHMALL_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";

pub const CASHMALL_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const CASHMALL_MANAGED_PANEL_SHA256: &str =
    "A1EC212735BF3A8171218BAC15A3757AFD35BB0C5DAC0071D11DD54BB30F2D47";

pub const CASHMALL_PANEL_START_X: i32 = -498;

pub const CASHMALL_OPEN_SECONDS: f32 = 1.0;

pub const CASHMALL_SLOT_TYPE: i32 = 9;

pub const CASHMALL_SLOT_SCAN_COUNT: usize = 20;

pub const CASHMALL_CACHED_ITEM_COUNT_0104: usize = 0;

pub const CASHMALL_USER_CASH_0104: i32 = 0;

pub const CASHMALL_CASH_DIGIT_COUNT: usize = 9;

pub const CASHMALL_TAB_COUNT: usize = 5;

/// Clean `Panel_Cashmall.OnGUI` depth. Lower Unity GUI depth draws later.
pub const CASHMALL_GUI_DEPTH_0104: i32 = 10;

/// Clean `Panel_PCStuffScript` and `Panel_Equip` depth.
pub const CASHMALL_SHARED_GUI_DEPTH_0104: i32 = 9;

pub const CASHMALL_PANEL_NATIVE_Z_0104: i32 = 0;

pub const CASHMALL_SHARED_NATIVE_Z_0104: i32 = 1;

/// Exact dead/retained paths in the clean managed code.
pub const CASHMALL_PURCHASE_NETWORK_CONTRACT_PRESENT: bool = false;

pub const CASHMALL_USER_CASH_ASSIGNMENT_REACHABLE: bool = false;

pub const CASHMALL_TAB_FILTERING_REACHABLE: bool = false;

pub const CASHMALL_SET_VENDOR_ITEM_REACHABLE: bool = false;

pub const CASHMALL_ITEM_BAR_ASSIGNED: bool = false;

pub const CASHMALL_NPC_ASSIGNMENT_FROM_CHAT_ENTRY_REACHABLE: bool = false;

pub const CASHMALL_HELP_RECEIVER_PRESENT: bool = false;

pub const CASHMALL_FREE_ASSETS_CLEARS_LOADED_TEXTURES: bool = false;

pub const CASHMALL_RECENT_BUY_SLOT12_BRANCH_REACHABLE: bool = false;

/// Approved replacement-font calibration for the exact serialized source
/// font objects. No additional Y compensation is present in the evidence.
pub const CASHMALL_LABEL_FONT_SIZE: f32 = 12.0;

pub const CASHMALL_SMALL_FONT_SIZE: f32 = 7.0;

pub const CASHMALL_LABEL_PADDING_TOP: f32 = 3.0;

pub const CASHMALL_LABEL_PADDING_BOTTOM: f32 = 3.0;

pub const CASHMALL_REPLACEMENT_FONT_Y_OFFSET: f32 = 0.0;

pub const CASHMALL_SOURCE_TEXTURES_0104: [CashmallTextureEvidence0104; 12] = [
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::BackBar,
        source_name: "backbar",
        path_id: 62,
        runtime_path: CASHMALL_BACK_BAR_PATH,
        width: 486,
        height: 13,
        png_bytes: 641,
        png_sha256: "024D2A6AE20223319553050076CD7AD5FFEEE64F12B6BF869868F23033392BF3",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::Cash,
        source_name: "cash",
        path_id: 551,
        runtime_path: CASHMALL_CASH_PATH,
        width: 149,
        height: 33,
        png_bytes: 2_834,
        png_sha256: "0983B6E30C3ACE56893621E4DD77F6E1A7EF3DA421BADBA4E61B58CB1DEB2DD8",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::FirstTabSelected,
        source_name: "firsttab",
        path_id: 648,
        runtime_path: CASHMALL_FIRST_TAB_SELECTED_PATH,
        width: 44,
        height: 29,
        png_bytes: 893,
        png_sha256: "D779EEC5CFDEB213C8485DC2C50E5A6331D1E2B21B49DBE1855C21B685DF1248",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::FirstTabNormal,
        source_name: "firsttabbut",
        path_id: 626,
        runtime_path: CASHMALL_FIRST_TAB_NORMAL_PATH,
        width: 44,
        height: 29,
        png_bytes: 1_162,
        png_sha256: "4B61094767D68B4F236A1DEA1B84B4A6402111B9E4C6C4931F9F716B784CFFA4",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::FirstTabHover,
        source_name: "firsttabbutover",
        path_id: 584,
        runtime_path: CASHMALL_FIRST_TAB_HOVER_PATH,
        width: 44,
        height: 29,
        png_bytes: 1_292,
        png_sha256: "42BE11CD3CBEB7AF052702692D299041784FF746DEF7B4E14166FA4DEC34C166",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::SecondTabSelected,
        source_name: "secondtab",
        path_id: 106,
        runtime_path: CASHMALL_SECOND_TAB_SELECTED_PATH,
        width: 81,
        height: 29,
        png_bytes: 1_161,
        png_sha256: "059486E0A38D370A8F25237F87C4C4B87E5C70D99098B9A5F71FF9D6D9391A89",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::SecondTabNormal,
        source_name: "secondtabbut",
        path_id: 465,
        runtime_path: CASHMALL_SECOND_TAB_NORMAL_PATH,
        width: 81,
        height: 29,
        png_bytes: 1_275,
        png_sha256: "05BDE1C17FC2D6C87B82F3EEEC4A99AF371F3C51E87E08BC27B7B7EAB6D1B208",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::SecondTabHover,
        source_name: "secondtabbutover",
        path_id: 189,
        runtime_path: CASHMALL_SECOND_TAB_HOVER_PATH,
        width: 81,
        height: 29,
        png_bytes: 1_161,
        png_sha256: "D8FE1D23B1F80C0D8F528473BF37EF0E1B2BB5E5D5B558A7456DE766EEA812D1",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::NanoTab,
        source_name: "nanotab",
        path_id: CASHMALL_NANO_TAB_TEXTURE_PATH_ID,
        runtime_path: USER_EQUIP_NANO_TAB_PATH,
        width: 129,
        height: 29,
        png_bytes: 1_576,
        png_sha256: "5451D50D8812CE502C44213C79BD93390A6F193D1C030A412DE10AFE61FF6B27",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::NanoTabHover,
        source_name: "nanotabover",
        path_id: CASHMALL_NANO_TAB_HOVER_TEXTURE_PATH_ID,
        runtime_path: CASHMALL_NANO_TAB_HOVER_PATH,
        width: 129,
        height: 29,
        png_bytes: 1_092,
        png_sha256: "93DB8FD618025D81CF1F289204D2D5CE04945321F908E7B9A4B883A96F08751D",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::DexlabsBanner,
        source_name: "dexlabsbut",
        path_id: CASHMALL_DEXLABS_TEXTURE_PATH_ID,
        runtime_path: CASHMALL_DEXLABS_PATH,
        width: 203,
        height: 67,
        png_bytes: 11_819,
        png_sha256: "FD6757A9872AB53250D23A56F83E398E1FC76A27F3BC4F2D7003B1805689F49D",
    },
    CashmallTextureEvidence0104 {
        role: CashmallTextureRole0104::TarosCounter,
        source_name: "Taros",
        path_id: CASHMALL_TAROS_COUNTER_TEXTURE_PATH_ID,
        runtime_path: CASHMALL_TAROS_COUNTER_PATH,
        width: 149,
        height: 33,
        png_bytes: 3_201,
        png_sha256: "E280860ECE99E9D3EAA5C2F33CD0A3FB633AB4D0301492AFA4DB18B4D9A0ECD0",
    },
];

pub const CASHMALL_HIDDEN_CHAT_BOUNDARY_0104: CashmallHiddenChatBoundary0104 =
    CashmallHiddenChatBoundary0104 {
        request_game_mode_event: [2, 0],
        requested_game_mode: CASHMALL_GAME_MODE_0104,
        receive_init_event: [2, 3, 0],
    };

pub const CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104: [&str; CashmallStaticAssetRole0104::COUNT] = [
    USER_EQUIP_BACKDROP_PATH,
    VENDOR_PANEL_PATH,
    USER_EQUIP_RIGHT_PANEL_PATH,
    VENDOR_INFO_PATH,
    VENDOR_LIST_BACK_PATH,
    CASHMALL_BACK_BAR_PATH,
    CASHMALL_CASH_PATH,
    CASHMALL_FIRST_TAB_SELECTED_PATH,
    CASHMALL_FIRST_TAB_NORMAL_PATH,
    CASHMALL_FIRST_TAB_HOVER_PATH,
    CASHMALL_SECOND_TAB_SELECTED_PATH,
    CASHMALL_SECOND_TAB_NORMAL_PATH,
    CASHMALL_SECOND_TAB_HOVER_PATH,
    VENDOR_BUTTON_NORMAL_PATH,
    VENDOR_BUTTON_HOVER_PATH,
    VENDOR_RESTRICTED_ITEM_FRAME_PATH,
    VENDOR_SCROLL_SHADOW_PATH,
    USER_EQUIP_INVENTORY_PANEL_PATH,
    USER_EQUIP_NANO_TAB_PATH,
    CASHMALL_NANO_TAB_HOVER_PATH,
    USER_EQUIP_SLOT_OCCUPIED_PATH,
    USER_EQUIP_SLOT_EMPTY_PATH,
    USER_EQUIP_COMBINED_PATH,
    USER_EQUIP_EQUIP_TITLE_PATH,
    USER_EQUIP_CLOSE_PATH,
    USER_EQUIP_TRASH_PATH,
    USER_EQUIP_HELP_PATH,
    CASHMALL_DEXLABS_PATH,
    CASHMALL_TAROS_COUNTER_PATH,
];
