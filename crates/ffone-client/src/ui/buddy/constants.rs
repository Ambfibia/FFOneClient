
pub const BUDDY_MAX_SLOTS: usize = 50;

pub const BUDDY_WARP_COOLDOWN_SECONDS: u32 = 60;

pub const BUDDY_LIST_ROW_STEP: f32 = 16.0;

pub const BUDDY_UI_SOURCE_BUILD: &str = "retrobution-20260613";

pub const BUDDY_UI_PRIMARY_MAIN_BYTES: u64 = 7_000_415;

pub const BUDDY_UI_PRIMARY_MAIN_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const BUDDY_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const BUDDY_JEFFE_12_FONT_SIZE: f32 = 10.0;

pub const BUDDY_CHALET_SMALL_FONT_SIZE: f32 = 12.0;

// Every reached clean GUIStyle has `m_ContentOffset.y = 0`. Keep replacement
// calibration explicit per style: this prevents a later replacement-font
// adjustment from silently moving unrelated Buddy copy.
pub const BUDDY_WINDOW_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_ITEM_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_TRANSPARENT3_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_DELETE_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_CANCEL_TEXT_Y_OFFSET: f32 = 0.0;

pub const BUDDY_LIST_TITLE: &str = " BUDDY LIST";

pub const BUDDY_DELETE_LABEL: &str = "DELETE";

pub const BUDDY_WARP_LABEL: &str = "WARP";

pub const BUDDY_ADD_LABEL: &str = "ADD";

pub const BUDDY_CANCEL_LABEL: &str = "CANCEL";

pub const BUDDY_ADD_TITLE: &str = "ADD BUDDY";

pub const BUDDY_ADD_INSTRUCTION: &str = "Please enter a character's full name to add a buddy";
