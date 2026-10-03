use super::*;

pub const NANOCOM_SOURCE_BUILD: &str = "retrobution-20260613";

pub const NANOCOM_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";

pub const NANOCOM_SOURCE_MAIN_ARCHIVE_BYTES: u64 = 7_000_415;

pub const NANOCOM_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const NANOCOM_SOURCE_ASSEMBLY: &str = "Assembly - CSharp.dll";

pub const NANOCOM_SOURCE_ASSEMBLY_BYTES: u64 = 1_517_568;

pub const NANOCOM_SOURCE_ASSEMBLY_SHA256: &str =
    "33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB";

pub const NANOCOM_SOURCE_UI_CLASS: &str = "cnGUINanocom";

pub const NANOCOM_SOURCE_LOGIN_METHOD: &str = "cnMissionManager.ReceiveStartGames";

pub const NANOCOM_SOURCE_MISSION_METHOD: &str = "cnMissionManager.SetMissionMessage";

pub const NANOCOM_SOURCE_ICONS_ARCHIVE: &str = "Icons.resourceFile";

pub const NANOCOM_SOURCE_ICONS_ARCHIVE_BYTES: u64 = 5_800_411;

pub const NANOCOM_SOURCE_ICONS_ARCHIVE_SHA256: &str =
    "A05602D6E96E2E74ECAD207F42E519605259434210DE8DA19E422B30D642E544";

pub const NANOCOM_COMPONENT_DUMP_SHA256: &str =
    "88E4FAEA06F3D2213C5F8F49E156B8C08D44607D437B17B227330F07D7722E34";

pub const NANOCOM_HUD_SKIN_DUMP_SHA256: &str =
    "94655C47469E24884DC873EEDDBF7B9185048ED9203B5DDEF0BA85A35810AA0B";

pub const NANOCOM_GUI_DEPTH: i32 = 8;

pub const NANOCOM_TYPE_9_LIFETIME_SECONDS: f32 = 10.0;

pub const NANOCOM_NANO_LIFETIME_SECONDS: f32 = 10.0;

pub const NANOCOM_BUDDY_LIFETIME_SECONDS: f32 = 20.0;

pub const NANOCOM_REVEAL_SECONDS: f32 = 0.5;

/// Right-side gap of the fully revealed native panel. The clean GUI formula
/// contributes 109 px; the deterministic Retrobution crop proves that the
/// Bevy HUD adapter needs another 13 px leftward compensation to preserve the
/// panel-to-minimap relationship.
pub const NANOCOM_REVEALED_RIGHT_MARGIN: f32 = 122.0;

pub const NANOCOM_COMPACT_TITLE: &str = "Buddy request";

pub const NANOCOM_EXPANDED_TITLE: &str = "BUDDY INVITE RECEIVED";

pub const NANOCOM_GROUP_COMPACT_TITLE: &str = "Group Invitation";

pub const NANOCOM_GROUP_EXPANDED_TITLE: &str = "GROUP INVITE RECEIVED";

pub const NANOCOM_NANO_MISSION_TITLE: &str = "New Nano Mission!";

pub const NANOCOM_ACCEPT_LABEL: &str = "Accept";

pub const NANOCOM_DECLINE_LABEL: &str = "Decline";

pub const NANOCOM_JEFFE_14_FONT_SIZE: f32 = 12.0;

/// Approved replacement calibration against the primary Computress login
/// crop. The 11 px vector face reproduces the clean raster glyph bounds; the
/// recovered 164 px content column supplies the clean three-line breaks.
pub const NANOCOM_CHALET_SMALL_FONT_SIZE: f32 = 11.0;

pub const NANOCOM_MESSAGE_TITLE_FONT_SIZE: f32 = 11.0;

pub const NANOCOM_COMPACT_BODY_FALLBACK: &str = "{name} has invited you to be buddies.\n\nPress Enter to accept or decline.\n{seconds} seconds remaining. ";

pub const NANOCOM_INVITATION_FALLBACK: &str = "{name} has invited you to be buddies.";

pub const NANOCOM_EXPIRATION_FALLBACK: &str = "Buddy will expire :{seconds} seconds";

pub const NANOCOM_GROUP_COMPACT_BODY_FALLBACK: &str = "{name} has invited you to join a group.\n\nPress Enter to accept or decline.\n{seconds} seconds remaining. ";

pub const NANOCOM_GROUP_INVITATION_FALLBACK: &str = "{name} has invited you to join a group.";

pub const NANOCOM_GROUP_EXPIRATION_FALLBACK: &str = "Group will expire :{seconds} seconds";

pub const NANOCOM_GROUP_INVITATION_SUFFIX: &str = " has invited you to join a group.";

pub const NANOCOM_OVERLAY_ALPHA: f32 = 0.5;

pub const NANOCOM_REACHED_TEXTURES: [NanocomTextureEvidence; 12] = [
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 290,
        source_name: "nanocom_message_etc",
        width: 326,
        height: 119,
        runtime_path: NANOCOM_BUDDY_FRAME_PATH,
        runtime_bytes: 4_468,
        runtime_sha256: "B68F3285ADB3E1E8116A59DE8AF83A2ECB4AC72596CA8D58CB1E6630423C9901",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 79,
        source_name: "messicon_buddy",
        width: 64,
        height: 63,
        runtime_path: NANOCOM_BUDDY_ICON_PATH,
        runtime_bytes: 3_827,
        runtime_sha256: "8D263B789EFF04AD712F610C813E8BE6D666A2CB919A0DE29ABF4B3E722AD810",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 26,
        source_name: "messicon_group",
        width: 64,
        height: 63,
        runtime_path: NANOCOM_GROUP_ICON_PATH,
        runtime_bytes: 3_957,
        runtime_sha256: "10C9F9901859DD216A11ABD069AF5CFDC2F45C8E8C125C07FEE7E28E6757919B",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 565,
        source_name: "nanocom_message_npc",
        width: 321,
        height: 119,
        runtime_path: NANOCOM_TYPE_9_FRAME_PATH,
        runtime_bytes: 5_727,
        runtime_sha256: "A920C06948D67BE1E869D4AB627015E8550A81DCF618F231568B5382E6EC9207",
    },
    NanocomTextureEvidence {
        source_container: "Icons.resourceFile/CustomAssetBundle-784fa24bcf2da4f5eabe9547958616eb",
        source_path_id: 444,
        source_name: "npcicon_87",
        width: 64,
        height: 64,
        runtime_path: NANOCOM_NUMBUH_TWO_ICON_PATH,
        runtime_bytes: 6_478,
        runtime_sha256: "8EEEB4FEEECE08377840AB4AB32A08C623E4AA93D553C8DB60190D22DF174442",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 398,
        source_name: "systemDialogBox",
        width: 110,
        height: 164,
        runtime_path: NANOCOM_DIALOG_PATH,
        runtime_bytes: 3_691,
        runtime_sha256: "A581AF8AA96F51CE9558D823BF6D250034E0E393FA3670943912620F35ED837C",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 233,
        source_name: "MessageArea",
        width: 425,
        height: 87,
        runtime_path: NANOCOM_MESSAGE_AREA_PATH,
        runtime_bytes: 1_247,
        runtime_sha256: "BFAC8FF1E2FC48CB90878C0A20D4E441700F708691734BAA694B09AC15571C39",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 640,
        source_name: "blue_button_normal",
        width: 20,
        height: 25,
        runtime_path: NANOCOM_BLUE_BUTTON_PATH,
        runtime_bytes: 587,
        runtime_sha256: "6A841912FB35EB3C6EECEAF24176ECD3E1258157F96ED59B481B83F7C022BEF3",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 309,
        source_name: "blue_button_over",
        width: 20,
        height: 25,
        runtime_path: NANOCOM_BLUE_BUTTON_OVER_PATH,
        runtime_bytes: 452,
        runtime_sha256: "E23A8BB2DBB1A90E785A21F776693E94BEB6AE542D55996231172F4EF3DD8C73",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 282,
        source_name: "red_button_normal",
        width: 20,
        height: 25,
        runtime_path: NANOCOM_RED_BUTTON_PATH,
        runtime_bytes: 641,
        runtime_sha256: "71592F7FE1153EB8B35D381EB01A4414D53228F5A90BDABA1990A98D49507A5D",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 190,
        source_name: "red_button_over",
        width: 20,
        height: 25,
        runtime_path: NANOCOM_RED_BUTTON_OVER_PATH,
        runtime_bytes: 705,
        runtime_sha256: "63FD2753C7F014396999A70F8769D430A7594A3F03481E88015DD93ED74E392E",
    },
    NanocomTextureEvidence {
        source_container: NANOCOM_SOURCE_SERIALIZED_FILE,
        source_path_id: 137,
        source_name: "nanocom_message_nano",
        width: 372,
        height: 122,
        runtime_path: NANOCOM_NANO_FRAME_PATH,
        runtime_bytes: 27_271,
        runtime_sha256: "AE1EAE32C160A1FC34079C02EB8BF92AEB63BCB2AF834F208C8B157AF74FE449",
    },
];

/// Production drains echoes every gameplay frame; this only bounds queues
/// installed without a chat owner, such as previews and harnesses.
pub(super) const NANOCOM_CHAT_ECHO_LIMIT: usize = 64;
