use super::*;

pub const GUIDE_CURRENT_FRAME_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(14.0, 25.0),
    max_inset: Vec2::new(14.0, 14.0),
};

pub const GUIDE_COMPUTRESS_FRAME_BORDER: BorderRect = BorderRect::all(8.0);

pub const GUIDE_ICON_FRAME_BORDER: BorderRect = BorderRect::all(4.0);

pub const GUIDE_CARD_FRAME_PATH: &str = "ui/en/gameplay/guide/window.png";

pub const GUIDE_CURRENT_FRAME_PATH: &str = "ui/en/gameplay/guide/cur_frame.png";

pub const GUIDE_COMPUTRESS_FRAME_PATH: &str = "ui/en/gameplay/guide/comptress_frame.png";

pub const GUIDE_ICON_FRAME_PATH: &str = "ui/en/gameplay/guide/ncp_icon_back.png";

pub const GUIDE_COMPUTRESS_FRAME_RECT: GuideUiRect = GuideUiRect::new(12.0, 12.0, 365.0, 110.0);

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct GuideUiMentorCurrentFrame {
    pub(super) mentor: GuideMentor,
}
