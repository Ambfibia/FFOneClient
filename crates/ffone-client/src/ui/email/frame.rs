use super::*;

pub const EMAIL_SUBJECT_WIRE_UNITS: usize = 32;

pub const EMAIL_CONTENT_WIRE_UNITS: usize = 512;

pub const EMAIL_UI_FRAME_PATH: &str = "ui/en/email/frame.png";

pub const EMAIL_UI_FRAME_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 0.0),
    max_inset: Vec2::new(5.0, 0.0),
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailWireItem {
    pub item_type: i16,
    pub item_id: i16,
    pub option: i32,
    pub time_limit: i32,
}

impl EmailWireItem {
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.item_id <= 0
    }
}
