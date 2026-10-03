
pub const UPSELL_GAME_OBJECT_PATH_ID: i64 = 1_313;

pub const UPSELL_COMPONENT_PATH_ID: i64 = 1_615;

pub const UPSELL_SCRIPT_PATH_ID: i64 = 1_146;

pub const UPSELL_SKIN_PATH_ID: i64 = 1_383;

pub const UPSELL_CLOSE_NORMAL_PATH_ID: i64 = 506;

pub const UPSELL_GET_NORMAL_PATH_ID: i64 = 151;

pub const UPSELL_CONTINUE_NORMAL_PATH_ID: i64 = 580;

pub const UPSELL_NOT_NOW_NORMAL_PATH_ID: i64 = 481;

pub const UPSELL_ADVERTIS_PATH_ID: i64 = 647;

pub const UPSELL_JEFFE_14_FONT_PATH_ID: i64 = 903;

pub const UPSELL_JEFFE_12_FONT_PATH_ID: i64 = 953;

pub const UPSELL_PANELBACK_PATH: &str = "ui/en/shared/panelback.png";

pub const UPSELL_CLOSE_NORMAL_PATH: &str = "ui/en/shared/close_normal.png";

pub const UPSELL_GET_NORMAL_PATH: &str = "ui/en/gameplay/upsell/us_get.png";

pub const UPSELL_CONTINUE_NORMAL_PATH: &str = "ui/en/gameplay/upsell/us_continue.png";

pub const UPSELL_NOT_NOW_NORMAL_PATH: &str = "ui/en/gameplay/upsell/us_not.png";

pub const UPSELL_ADVERTIS_PATH: &str = "ui/en/gameplay/upsell/advertis.png";

pub const UPSELL_FONT_PATH: &str = "fonts/jeffe.otf";

pub const UPSELL_UI_Z_INDEX: i32 = i32::MAX - 55;

pub(super) fn valid_news_page_path(path: &str) -> bool {
    !path.is_empty()
        && path.trim() == path
        && !path.starts_with(['/', '\\'])
        && !path.contains(':')
        && !path.chars().any(char::is_control)
        && path
            .split(['/', '\\'])
            .all(|segment| !segment.is_empty() && segment != "..")
}
