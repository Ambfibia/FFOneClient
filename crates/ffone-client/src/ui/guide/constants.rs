use super::*;

pub const GUIDE_UI_SOURCE_BUILD: &str = "retrobution-20260613";

pub const GUIDE_UI_PRIMARY_MAIN_BYTES: u64 = 7_000_415;

pub const GUIDE_UI_PRIMARY_MAIN_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const GUIDE_UI_PARITY_CAVEAT: &str = "The clean component, GUISkin, TableData labels and \
published Texture2D assets, key-first TextManager labels and exact style-local replacement-font \
metrics are represented. Runtime packet routing, camera sub-target ownership, the external help \
page and a normalized clean-client golden capture remain shell-level acceptance work.";

pub const GUIDE_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const GUIDE_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const GUIDE_CHALET_SMALL_FONT_SIZE: f32 = 12.0;

/// Exact `FusionFallGuideSkin.compubox` text content box. The outer source
/// Rect remains [`GUIDE_COMPUTRESS_FRAME_RECT`]; these insets are serialized
/// GUIStyle padding, not an invented replacement-font shift.
pub const GUIDE_COMPUTRESS_TEXT_PADDING: UiRect = UiRect {
    left: Val::Px(80.0),
    right: Val::Px(0.0),
    top: Val::Px(7.0),
    bottom: Val::Px(0.0),
};

/// Exact `FusionFallGuideSkin.window` title padding.
pub const GUIDE_CARD_TEXT_PADDING: UiRect = UiRect {
    left: Val::Px(30.0),
    right: Val::Px(0.0),
    top: Val::Px(10.0),
    bottom: Val::Px(0.0),
};

/// Exact `FusionFallGuideSkin.FrameWindow` title padding.
pub const GUIDE_CURRENT_TEXT_PADDING: UiRect = UiRect {
    left: Val::Px(0.0),
    right: Val::Px(0.0),
    top: Val::Px(2.0),
    bottom: Val::Px(0.0),
};

/// Exact default-label padding used by cost and modal headings.
pub const GUIDE_LABEL_TEXT_PADDING: UiRect = UiRect {
    left: Val::Px(0.0),
    right: Val::Px(0.0),
    top: Val::Px(3.0),
    bottom: Val::Px(3.0),
};

/// Exact `BigFont16` padding. Its asymmetric horizontal inset moves the
/// source content center two pixels right of the raw Rect center.
pub const GUIDE_BIG_FONT_TEXT_PADDING: UiRect = UiRect {
    left: Val::Px(10.0),
    right: Val::Px(6.0),
    top: Val::Px(4.0),
    bottom: Val::Px(6.0),
};

pub const GUIDE_COMPUTRESS_LABEL: &str = "COMPUTRESS:";

pub const GUIDE_CHOOSE_HEADING: &str = "CHOOSE A GUIDE:";

pub const GUIDE_CHANGE_HEADING: &str = "CHOOSE A NEW GUIDE:";

pub const GUIDE_CHOOSE_INTRO: &str = "Four different guides await your arrival in the past. Each \
will aid you in your quest to defeat Fuse, as well as offer unique missions and items. So choose \
wisely.";

pub const GUIDE_CHANGE_INTRO: &str = "If you change your guide, you will start receiving new guide \
missions. Any active guide missions you have now will be removed from your journal. Any guide \
items that you own can no longer be equipped.";

pub const GUIDE_CURRENT_LABEL: &str = "CURRENT GUIDE";

pub const GUIDE_COST_LABEL: &str = "cost";

pub const GUIDE_CANCEL_LABEL: &str = "CANCEL";

pub const GUIDE_CONFIRM_LABEL: &str = "CONFIRM";

pub const GUIDE_CONFIRM_TITLE: &str = "GUIDE CONFIRMATION";

pub const GUIDE_CHANGE_CONFIRM_TITLE: &str = "ALERT";

pub const GUIDE_WARP_TITLE: &str = "READY TO FIGHT FUSE?";

pub const GUIDE_WARP_BODY: &str = "Once you warp to the past, you will be unable to return to this \
area. Any active mission you have now will be removed from your journal.";

pub const GUIDE_ALREADY_CURRENT_MESSAGE_ID: i32 = 155;

pub const GUIDE_CHANGE_FAILURE_MESSAGE_ID: i32 = 154;

pub const GUIDE_ALREADY_CURRENT_MESSAGE: &str = "Can not change same guide.";

pub const GUIDE_CHANGE_FAILURE_MESSAGE: &str =
    "ERROR!\nYou do not have enough Fusion Matter to change your guide.";

pub const GUIDE_CHANGE_PRICES: [u32; 4] = [0; 4];

pub const GUIDE_FIRST_CHANGE_WARP_NPC_TABLE_ID: i32 = 1_425;
