use super::*;

pub const UPSELL_SOURCE_BUILD: &str = "retrobution-20260613";

pub const UPSELL_SCREEN_PIVOT_CENTER_VALUE: i32 = 4;

pub const UPSELL_LEVEL_IMAGE_PATHS: [&str; 4] = [
    "ui/en/gameplay/upsell/upsell_lvl1.png",
    "ui/en/gameplay/upsell/upsell_lvl2.png",
    "ui/en/gameplay/upsell/upsell_lvl3.png",
    "ui/en/gameplay/upsell/upsell_lvl4.png",
];

pub const UPSELL_PAGE_FADE_PER_SECOND: f32 = 2.0;

pub const UPSELL_PAGE_CHANGE_START_ALPHA: f32 = 0.01;

pub const UPSELL_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const UPSELL_JEFFE_12_FONT_SIZE: f32 = 10.0;

pub const UPSELL_CONTINUE_NORMAL_TEXT_COLOR: [f32; 4] = [0.784_313_74, 1.0, 1.0, 1.0];

pub const UPSELL_NOT_NOW_NORMAL_TEXT_COLOR: [f32; 4] = [0.786_290_35, 1.0, 1.0, 1.0];

pub const UPSELL_NOT_NOW_ACTIVE_TEXT_COLOR: [f32; 4] = [0.794_354_86, 1.0, 1.0, 1.0];

pub const UPSELL_CONTINUE_PLAYING_LABEL: &str = "CONTINUE PLAYING";

pub const UPSELL_NOT_RIGHT_NOW_LABEL: &str = "NOT RIGHT NOW";

pub const UPSELL_CONTINUE_LABEL: &str = "CONTINUE";

pub const UPSELL_NEWS_REACHABILITY: UpsellSourceReachability =
    UpsellSourceReachability::ReceiveInit;

pub const UPSELL_UPGRADE_REACHABILITY: UpsellSourceReachability =
    UpsellSourceReachability::RetainedExplicit;

pub const UPSELL_LEAVE_FUTURE_REACHABILITY: UpsellSourceReachability =
    UpsellSourceReachability::UnregisteredDead;
