use super::*;

pub const QUICK_SLOT_SOURCE_BUILD: &str = "retrobution-20260613";

pub const QUICK_SLOT_SOURCE_ARCHIVE: &str = "main.unity3d";

pub const QUICK_SLOT_SOURCE_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const QUICK_SLOT_COUNT: usize = 8;

pub const QUICK_SLOT_LARGE_CHAT_GROUP_X: f32 = 450.0;

pub const QUICK_SLOT_SMALL_CHAT_GROUP_X: f32 = 400.0;

pub const QUICK_SLOT_BACKGROUND_LOCAL_LEFT: f32 = 0.0;

pub const QUICK_SLOT_BACKGROUND_LOCAL_TOP: f32 = 110.0;

pub const QUICK_SLOT_PANEL_LOCAL_LEFT: f32 = 6.0;

pub const QUICK_SLOT_PANEL_LOCAL_TOP: f32 = 114.0;

pub const QUICK_SLOT_SIZE: f32 = 34.0;

pub const QUICK_SLOT_GAP: f32 = 1.0;

pub const QUICK_SLOT_STRIDE: f32 = QUICK_SLOT_SIZE + QUICK_SLOT_GAP;

pub const QUICK_SLOT_COOLDOWN_INSET: f32 = 1.0;

pub const QUICK_SLOT_COOLDOWN_SIZE: f32 = 32.0;

pub const QUICK_SLOT_SOURCE_BACKGROUND_SHA256: &str =
    "EDF438D4626E1909601CE596EFC50ECCA1CA1B843F1E69E9D9CEB7745CD6E4E7";

pub const QUICK_SLOT_SOURCE_OCCUPIED_STYLE_SHA256: &str =
    "2777645986C2B0EA46CC527FAA9ED7C8471AF4A5667B272DF123B60990ABF3EA";

pub const QUICK_SLOT_SOURCE_EMPTY_STYLE_SHA256: &str =
    "4B0EACB923AF7210AE924C68EDB4F0C46D0D3F30D113239AF78A1DD5D3B826DA";

pub const QUICK_SLOT_SOURCE_COOLDOWN_SHA256: &str =
    "02790A13362C4AB24FF7629F5ED49312C81F665AC6FF522A0B17DA7B73031973";

pub const QUICK_SLOT_SOURCE_TEXTURES: [QuickSlotSourceTextureEvidence; 4] = [
    QuickSlotSourceTextureEvidence {
        role: QuickSlotTextureRole::Background,
        converted_path: QUICK_SLOT_SOURCE_BACKGROUND_PATH,
        runtime_path: QUICK_SLOT_BACKGROUND_PATH,
        sha256: QUICK_SLOT_SOURCE_BACKGROUND_SHA256,
        source_path_id: QUICK_SLOT_BACKGROUND_PATH_ID,
        width: 290,
        height: 39,
    },
    QuickSlotSourceTextureEvidence {
        role: QuickSlotTextureRole::OccupiedStyle,
        converted_path: QUICK_SLOT_SOURCE_OCCUPIED_STYLE_PATH,
        runtime_path: QUICK_SLOT_OCCUPIED_STYLE_PATH,
        sha256: QUICK_SLOT_SOURCE_OCCUPIED_STYLE_SHA256,
        source_path_id: QUICK_SLOT_OCCUPIED_STYLE_PATH_ID,
        width: 62,
        height: 62,
    },
    QuickSlotSourceTextureEvidence {
        role: QuickSlotTextureRole::EmptyStyle,
        converted_path: QUICK_SLOT_SOURCE_EMPTY_STYLE_PATH,
        runtime_path: QUICK_SLOT_EMPTY_STYLE_PATH,
        sha256: QUICK_SLOT_SOURCE_EMPTY_STYLE_SHA256,
        source_path_id: QUICK_SLOT_EMPTY_STYLE_PATH_ID,
        width: 62,
        height: 62,
    },
    QuickSlotSourceTextureEvidence {
        role: QuickSlotTextureRole::Cooldown,
        converted_path: QUICK_SLOT_SOURCE_COOLDOWN_PATH,
        runtime_path: QUICK_SLOT_COOLDOWN_PATH,
        sha256: QUICK_SLOT_SOURCE_COOLDOWN_SHA256,
        source_path_id: QUICK_SLOT_COOLDOWN_TEXTURE_PATH_ID,
        width: 64,
        height: 64,
    },
];
