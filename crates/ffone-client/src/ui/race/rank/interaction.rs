use super::*;

pub const RACE_RANK_TODAY_HOVER_PATH: &str = "ui/en/race/rank/skin/today_hover.png";

pub const RACE_RANK_WIDE_HOVER_PATH: &str = "ui/en/race/rank/skin/wide_hover.png";

pub const RACE_RANK_PREVIOUS_HOVER_PATH: &str = "ui/en/race/rank/controls/previous_hover.png";

pub const RACE_RANK_NEXT_HOVER_PATH: &str = "ui/en/race/rank/controls/next_hover.png";

pub const RACE_RANK_LEFT_ARROW_HOVER_PATH: &str = "ui/en/race/rank/controls/arrow_left_hover.png";

pub const RACE_RANK_RIGHT_ARROW_HOVER_PATH: &str = "ui/en/race/rank/controls/arrow_right_hover.png";

pub const RACE_RANK_CLOSE_HOVER_PATH: &str = "ui/en/race/rank/controls/close_hover.png";

pub const RACE_RANK_HELP_HOVER_PATH: &str = "ui/en/race/rank/controls/help_hover.png";

pub const RACE_RANK_INVENTORY_CLOSE_HOVER_PATH_ID: i64 = 157;

#[derive(Clone, Copy, Debug, Default, Resource, Eq, PartialEq)]
pub struct RaceRankPresentationInput {
    pub system_popup_active: bool,
}
