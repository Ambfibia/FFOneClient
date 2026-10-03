//! Clean-client source archives, Unity path IDs and exact texture evidence behind the UserEquip assets.

use super::asset_paths::{
    USER_EQUIP_BACKDROP_PATH, USER_EQUIP_CALCULATOR_BACK_PATH, USER_EQUIP_CLOSE_PATH,
    USER_EQUIP_CLOTHES_PANEL_PATH, USER_EQUIP_COMBINED_PATH, USER_EQUIP_EQUIP_TITLE_PATH,
    USER_EQUIP_GENERAL_DIALOG_PATH, USER_EQUIP_HELP_PATH, USER_EQUIP_INVENTORY_PANEL_PATH,
    USER_EQUIP_NANO_BACK_PATH, USER_EQUIP_NANO_BLUE_PATH, USER_EQUIP_NANO_DIALOG_PATH,
    USER_EQUIP_NANO_RED_PATH, USER_EQUIP_NANO_TAB_PATH, USER_EQUIP_NANO_YELLOW_PATH,
    USER_EQUIP_RED_BUTTON_HOVER_PATH, USER_EQUIP_RED_BUTTON_PATH, USER_EQUIP_RIGHT_PANEL_PATH,
    USER_EQUIP_SLOT_EMPTY_PATH, USER_EQUIP_SLOT_OCCUPIED_PATH, USER_EQUIP_TRASH_PATH,
    USER_EQUIP_TURN_LEFT_HOVER_PATH, USER_EQUIP_TURN_LEFT_PATH, USER_EQUIP_TURN_RIGHT_HOVER_PATH,
    USER_EQUIP_TURN_RIGHT_PATH,
};

pub const USER_EQUIP_SOURCE_BUILD: &str = "retrobution-20260613";
pub const USER_EQUIP_EDITABLE_LAYOUT_PATH: &str = "ui/en/user-equip/user-equip.ffui.json";
pub const USER_EQUIP_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";
pub const USER_EQUIP_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";
pub const USER_EQUIP_SOURCE_MAIN_SERIALIZED_FILE: &str = "sharedassets0.assets";
pub const USER_EQUIP_SOURCE_TUTORIAL_ARCHIVE: &str = "Tutorial.resourceFile";
pub const USER_EQUIP_SOURCE_TUTORIAL_ARCHIVE_SHA256: &str =
    "49A684FF4236848D0B882A5D725FFBE99D5CFB5D2090DC8705350E97DD3FD024";
pub const USER_EQUIP_GAME_OBJECT_PATH_ID: i64 = 1_307;
pub const USER_EQUIP_CN_EQUIP_COMPONENT_PATH_ID: i64 = 1_401;
pub const USER_EQUIP_USER_CLOTHES_COMPONENT_PATH_ID: i64 = 1_402;
pub const USER_EQUIP_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_403;
pub const USER_EQUIP_EQUIP_PANEL_COMPONENT_PATH_ID: i64 = 1_404;
pub const USER_EQUIP_INVENTORY_MANAGER_COMPONENT_PATH_ID: i64 = 1_419;
pub const USER_EQUIP_INVENTORY_SKIN_PATH_ID: i64 = 1_366;
pub const USER_EQUIP_BACKDROP_PATH_ID: i64 = 330;
pub const USER_EQUIP_CLOTHES_PANEL_PATH_ID: i64 = 264;
pub const USER_EQUIP_RIGHT_PANEL_PATH_ID: i64 = 307;
pub const USER_EQUIP_INVENTORY_PANEL_PATH_ID: i64 = 136;
pub const USER_EQUIP_SLOT_OCCUPIED_PATH_ID: i64 = 50;
pub const USER_EQUIP_SLOT_EMPTY_PATH_ID: i64 = 239;
pub const USER_EQUIP_EQUIP_TITLE_PATH_ID: i64 = 82;
pub const USER_EQUIP_NANO_TAB_PATH_ID: i64 = 98;
pub const USER_EQUIP_CLOSE_PATH_ID: i64 = 105;
pub const USER_EQUIP_TRASH_PATH_ID: i64 = 386;
pub const USER_EQUIP_HELP_PATH_ID: i64 = 245;
pub const USER_EQUIP_COMBINED_PATH_ID: i64 = 636;
pub const USER_EQUIP_NANO_BACK_PATH_ID: i64 = 413;
pub const USER_EQUIP_NANO_DIALOG_PATH_ID: i64 = 377;
pub const USER_EQUIP_NANO_BLUE_PATH_ID: i64 = 417;
pub const USER_EQUIP_NANO_RED_PATH_ID: i64 = 349;
pub const USER_EQUIP_NANO_YELLOW_PATH_ID: i64 = 588;
pub const USER_EQUIP_TURN_LEFT_PATH_ID: i64 = 161;
pub const USER_EQUIP_TURN_LEFT_HOVER_PATH_ID: i64 = 471;
pub const USER_EQUIP_TURN_RIGHT_PATH_ID: i64 = 601;
pub const USER_EQUIP_TURN_RIGHT_HOVER_PATH_ID: i64 = 618;
pub const USER_EQUIP_SMALL_FONT_PATH_ID: i64 = 1127;
pub const USER_EQUIP_REGULAR_FONT_PATH_ID: i64 = 977;

pub const USER_EQUIP_SOURCE_BACKDROP_PATH: &str = "ui/en/shared/panelback.png";
pub const USER_EQUIP_SOURCE_CLOTHES_PANEL_PATH: &str = "ui/en/user-equip/colthback.png";
pub const USER_EQUIP_SOURCE_RIGHT_PANEL_PATH: &str = "ui/en/user-equip/equipmentback.png";
pub const USER_EQUIP_SOURCE_INVENTORY_PANEL_PATH: &str = "ui/en/user-equip/itemslotback.png";
pub const USER_EQUIP_SOURCE_SLOT_OCCUPIED_PATH: &str = "ui/en/gameplay/quick-slot/slotbox.png";
pub const USER_EQUIP_SOURCE_SLOT_EMPTY_PATH: &str = "ui/en/gameplay/quick-slot/slotboxempty.png";
pub const USER_EQUIP_SOURCE_EQUIP_TITLE_PATH: &str = "ui/en/user-equip/equipbar.png";
pub const USER_EQUIP_SOURCE_NANO_TAB_PATH: &str = "ui/en/shared/nanotab.png";
pub const USER_EQUIP_SOURCE_CLOSE_PATH: &str = "ui/en/rule/close.png";
pub const USER_EQUIP_SOURCE_TRASH_PATH: &str = "ui/en/user-equip/trash.png";
pub const USER_EQUIP_SOURCE_HELP_PATH: &str = "ui/en/world-map/controls/NanoMachineHelpButton.png";
pub const USER_EQUIP_SOURCE_COMBINED_PATH: &str = "ui/en/user-equip/combineditemicon.png";

pub const USER_EQUIP_BACKDROP_SHA256: &str =
    "FB9C6C8A4B8313766364FF3398070B35D8A4137F511ACCC4DDAAEA91BD0162F1";
pub const USER_EQUIP_CLOTHES_PANEL_SHA256: &str =
    "F28A44D1DCED93DF7ACBD3379623B7353046FC858E994DBE2A4D3C2CC778EE7E";
pub const USER_EQUIP_RIGHT_PANEL_SHA256: &str =
    "6B23404B01B04F2380A172D42362D86457216EEA677D3D728B6A3972AD4B86A3";
pub const USER_EQUIP_INVENTORY_PANEL_SHA256: &str =
    "E375934893069478D451FEC626284B1FF3C29B06DA12AC6AB134CD96E75E32EE";
pub const USER_EQUIP_SLOT_OCCUPIED_SHA256: &str =
    "712CC19391108F007D723107C953CDF7EBCAFE458BFE317AD3F298D3F601F915";
pub const USER_EQUIP_SLOT_EMPTY_SHA256: &str =
    "0F9D817C9C94B0E1A28B625EF02A26047A3D5F072EE0280822A2E99A66D7713C";
pub const USER_EQUIP_EQUIP_TITLE_SHA256: &str =
    "A719DA6205A939D444E82D9C813E395871D6E1E0FE2535852918B728DD1AA133";
pub const USER_EQUIP_NANO_TAB_SHA256: &str =
    "057FFA02CD0F8A288689B30C5310A307C866AA7F8525252C8E85643F9296193B";
pub const USER_EQUIP_CLOSE_SHA256: &str =
    "5F5F6BFCE0BF796D802714EA75FF49BEBF21BB536674143BBCB925A0BD57929D";
pub const USER_EQUIP_TRASH_SHA256: &str =
    "7381185D25FB02EAF70788C0374AA0EC1D216E620DE0A63D88FAA06D37175DAE";
pub const USER_EQUIP_HELP_SHA256: &str =
    "2695DAFC3747DC2D1E7F78EF0CBD1A0CDFEF1E4FDC1F00F2C45E1D8C690D6591";
pub const USER_EQUIP_COMBINED_SHA256: &str =
    "1050E1EFD1AFE851AAB9784A54AB524A771786CEC96C9DD6611FD66C3C66394C";
pub const USER_EQUIP_NANO_BACK_SHA256: &str =
    "6AB06EAAD9CFDA76094454189C25BC18F085B0AD82FF2216E3A27A03CE84B837";
pub const USER_EQUIP_NANO_DIALOG_SHA256: &str =
    "E48D16C7E7EA3F77DA64E9E3AFF355D8145DF8BE416E32240775BDE33AEBC6B4";
pub const USER_EQUIP_NANO_BLUE_SHA256: &str =
    "2FEB2E22AC68DBBC95B2A08F25542478458A7A856B738C71092FD25FA34E6A06";
pub const USER_EQUIP_NANO_RED_SHA256: &str =
    "5A7A42973A0F2C1B4C26CF640EFFCB3BEA074119D6716B8DD835040ACCFA1D42";
pub const USER_EQUIP_NANO_YELLOW_SHA256: &str =
    "A02163B28B34CD17BA6E00503CB55960B9596112A278D48F6A23A8E27CE4690A";
pub const USER_EQUIP_TURN_LEFT_SHA256: &str =
    "939558AE4C8BCCCBAF9652E0BAAABE211A953A763BA31D60C429A934DF446C01";
pub const USER_EQUIP_TURN_LEFT_HOVER_SHA256: &str =
    "A0BAC28BEBBCA7C158F997705B367C59217B127669EE71885F430F7C25897461";
pub const USER_EQUIP_TURN_RIGHT_SHA256: &str =
    "8BC6BB7BA710512C448BF63B7AE8F02F62D9A9F28E61F7D57EEEE8C42E6B0A23";
pub const USER_EQUIP_TURN_RIGHT_HOVER_SHA256: &str =
    "2018D8209A5D54B6C7282FD0B90B2C81C67C03818ADAE3024FB1A8A09527C782";
pub const USER_EQUIP_GENERAL_DIALOG_PATH_ID: i64 = 390;
pub const USER_EQUIP_GENERAL_DIALOG_SHA256: &str =
    "ED2E1569E44BEB4CEBDC9A591C6AD9570002DD98A858586C2E7E2C26ACA828B0";
pub const USER_EQUIP_CALCULATOR_BACK_PATH_ID: i64 = 385;
pub const USER_EQUIP_CALCULATOR_BACK_SHA256: &str =
    "E1F4481646709B92D60262A88883D4798C9AC1F51C3BA8CE27A8B5B3C2BE530A";
pub const USER_EQUIP_RED_BUTTON_PATH_ID: i64 = 282;
pub const USER_EQUIP_RED_BUTTON_SHA256: &str =
    "897F2E6F327C29BB0421887CBBEBF61901B35FB18209976EAF675C4CCB370822";
pub const USER_EQUIP_RED_BUTTON_HOVER_PATH_ID: i64 = 190;
pub const USER_EQUIP_RED_BUTTON_HOVER_SHA256: &str =
    "8DE7555635F7D1B3309A56265901C71248634D032232E81BD70D0F6538D7A719";
pub const USER_EQUIP_POPUP_TEXTURE_EVIDENCE: [(&str, i64, &str, (u32, u32)); 4] = [
    (
        USER_EQUIP_GENERAL_DIALOG_PATH,
        USER_EQUIP_GENERAL_DIALOG_PATH_ID,
        USER_EQUIP_GENERAL_DIALOG_SHA256,
        (310, 355),
    ),
    (
        USER_EQUIP_CALCULATOR_BACK_PATH,
        USER_EQUIP_CALCULATOR_BACK_PATH_ID,
        USER_EQUIP_CALCULATOR_BACK_SHA256,
        (136, 109),
    ),
    (
        USER_EQUIP_RED_BUTTON_PATH,
        USER_EQUIP_RED_BUTTON_PATH_ID,
        USER_EQUIP_RED_BUTTON_SHA256,
        (20, 25),
    ),
    (
        USER_EQUIP_RED_BUTTON_HOVER_PATH,
        USER_EQUIP_RED_BUTTON_HOVER_PATH_ID,
        USER_EQUIP_RED_BUTTON_HOVER_SHA256,
        (20, 25),
    ),
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UserEquipSourceArchive {
    Main,
    Tutorial,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UserEquipTextureRole {
    Backdrop,
    ClothesPanel,
    RightPanel,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    EquipTitle,
    NanoTab,
    Close,
    Trash,
    Help,
    Combined,
    NanoBack,
    NanoDialog,
    NanoBlue,
    NanoRed,
    NanoYellow,
    TurnLeft,
    TurnLeftHover,
    TurnRight,
    TurnRightHover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserEquipSourceTextureEvidence {
    pub role: UserEquipTextureRole,
    pub source_archive: UserEquipSourceArchive,
    pub source_path_id: i64,
    pub converted_path: &'static str,
    pub runtime_path: &'static str,
    pub sha256: &'static str,
    pub width: u32,
    pub height: u32,
}

pub const USER_EQUIP_SOURCE_TEXTURES: [UserEquipSourceTextureEvidence; 21] = [
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::Backdrop,
        source_archive: UserEquipSourceArchive::Tutorial,
        source_path_id: USER_EQUIP_BACKDROP_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_BACKDROP_PATH,
        runtime_path: USER_EQUIP_BACKDROP_PATH,
        sha256: USER_EQUIP_BACKDROP_SHA256,
        width: 1_920,
        height: 1_440,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::ClothesPanel,
        source_archive: UserEquipSourceArchive::Tutorial,
        source_path_id: USER_EQUIP_CLOTHES_PANEL_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_CLOTHES_PANEL_PATH,
        runtime_path: USER_EQUIP_CLOTHES_PANEL_PATH,
        sha256: USER_EQUIP_CLOTHES_PANEL_SHA256,
        width: 585,
        height: 651,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::RightPanel,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_RIGHT_PANEL_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_RIGHT_PANEL_PATH,
        runtime_path: USER_EQUIP_RIGHT_PANEL_PATH,
        sha256: USER_EQUIP_RIGHT_PANEL_SHA256,
        width: 32,
        height: 654,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::InventoryPanel,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_INVENTORY_PANEL_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_INVENTORY_PANEL_PATH,
        runtime_path: USER_EQUIP_INVENTORY_PANEL_PATH,
        sha256: USER_EQUIP_INVENTORY_PANEL_SHA256,
        width: 157,
        height: 119,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::SlotOccupied,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_SLOT_OCCUPIED_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_SLOT_OCCUPIED_PATH,
        runtime_path: USER_EQUIP_SLOT_OCCUPIED_PATH,
        sha256: USER_EQUIP_SLOT_OCCUPIED_SHA256,
        width: 62,
        height: 62,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::SlotEmpty,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_SLOT_EMPTY_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_SLOT_EMPTY_PATH,
        runtime_path: USER_EQUIP_SLOT_EMPTY_PATH,
        sha256: USER_EQUIP_SLOT_EMPTY_SHA256,
        width: 62,
        height: 62,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::EquipTitle,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_EQUIP_TITLE_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_EQUIP_TITLE_PATH,
        runtime_path: USER_EQUIP_EQUIP_TITLE_PATH,
        sha256: USER_EQUIP_EQUIP_TITLE_SHA256,
        width: 5,
        height: 7,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::NanoTab,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_NANO_TAB_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_NANO_TAB_PATH,
        runtime_path: USER_EQUIP_NANO_TAB_PATH,
        sha256: USER_EQUIP_NANO_TAB_SHA256,
        width: 129,
        height: 29,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::Close,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_CLOSE_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_CLOSE_PATH,
        runtime_path: USER_EQUIP_CLOSE_PATH,
        sha256: USER_EQUIP_CLOSE_SHA256,
        width: 32,
        height: 33,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::Trash,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_TRASH_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_TRASH_PATH,
        runtime_path: USER_EQUIP_TRASH_PATH,
        sha256: USER_EQUIP_TRASH_SHA256,
        width: 32,
        height: 32,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::Help,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_HELP_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_HELP_PATH,
        runtime_path: USER_EQUIP_HELP_PATH,
        sha256: USER_EQUIP_HELP_SHA256,
        width: 32,
        height: 32,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::Combined,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_COMBINED_PATH_ID,
        converted_path: USER_EQUIP_SOURCE_COMBINED_PATH,
        runtime_path: USER_EQUIP_COMBINED_PATH,
        sha256: USER_EQUIP_COMBINED_SHA256,
        width: 25,
        height: 25,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::NanoBack,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_NANO_BACK_PATH_ID,
        converted_path: USER_EQUIP_NANO_BACK_PATH,
        runtime_path: USER_EQUIP_NANO_BACK_PATH,
        sha256: USER_EQUIP_NANO_BACK_SHA256,
        width: 241,
        height: 114,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::NanoDialog,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_NANO_DIALOG_PATH_ID,
        converted_path: USER_EQUIP_NANO_DIALOG_PATH,
        runtime_path: USER_EQUIP_NANO_DIALOG_PATH,
        sha256: USER_EQUIP_NANO_DIALOG_SHA256,
        width: 126,
        height: 69,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::NanoBlue,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_NANO_BLUE_PATH_ID,
        converted_path: USER_EQUIP_NANO_BLUE_PATH,
        runtime_path: USER_EQUIP_NANO_BLUE_PATH,
        sha256: USER_EQUIP_NANO_BLUE_SHA256,
        width: 57,
        height: 72,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::NanoRed,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_NANO_RED_PATH_ID,
        converted_path: USER_EQUIP_NANO_RED_PATH,
        runtime_path: USER_EQUIP_NANO_RED_PATH,
        sha256: USER_EQUIP_NANO_RED_SHA256,
        width: 57,
        height: 72,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::NanoYellow,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_NANO_YELLOW_PATH_ID,
        converted_path: USER_EQUIP_NANO_YELLOW_PATH,
        runtime_path: USER_EQUIP_NANO_YELLOW_PATH,
        sha256: USER_EQUIP_NANO_YELLOW_SHA256,
        width: 57,
        height: 72,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::TurnLeft,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_TURN_LEFT_PATH_ID,
        converted_path: USER_EQUIP_TURN_LEFT_PATH,
        runtime_path: USER_EQUIP_TURN_LEFT_PATH,
        sha256: USER_EQUIP_TURN_LEFT_SHA256,
        width: 43,
        height: 78,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::TurnLeftHover,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_TURN_LEFT_HOVER_PATH_ID,
        converted_path: USER_EQUIP_TURN_LEFT_HOVER_PATH,
        runtime_path: USER_EQUIP_TURN_LEFT_HOVER_PATH,
        sha256: USER_EQUIP_TURN_LEFT_HOVER_SHA256,
        width: 43,
        height: 78,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::TurnRight,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_TURN_RIGHT_PATH_ID,
        converted_path: USER_EQUIP_TURN_RIGHT_PATH,
        runtime_path: USER_EQUIP_TURN_RIGHT_PATH,
        sha256: USER_EQUIP_TURN_RIGHT_SHA256,
        width: 43,
        height: 78,
    },
    UserEquipSourceTextureEvidence {
        role: UserEquipTextureRole::TurnRightHover,
        source_archive: UserEquipSourceArchive::Main,
        source_path_id: USER_EQUIP_TURN_RIGHT_HOVER_PATH_ID,
        converted_path: USER_EQUIP_TURN_RIGHT_HOVER_PATH,
        runtime_path: USER_EQUIP_TURN_RIGHT_HOVER_PATH,
        sha256: USER_EQUIP_TURN_RIGHT_HOVER_SHA256,
        width: 43,
        height: 78,
    },
];
