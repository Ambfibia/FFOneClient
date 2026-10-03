use super::*;

pub const QUICK_SLOT_SCALE_REFERENCE_HEIGHT: f32 = 768.0;

pub const QUICK_SLOT_SCALE_FACTOR: f32 = 1.05;

pub const QUICK_SLOT_CHAT_WINDOW_HEIGHT: f32 = 150.0;

pub const QUICK_SLOT_BACKGROUND_WIDTH: f32 = 290.0;

pub const QUICK_SLOT_BACKGROUND_HEIGHT: f32 = 39.0;

pub const QUICK_SLOT_PANEL_WIDTH: f32 = 300.0;

pub const QUICK_SLOT_PANEL_HEIGHT: f32 = 100.0;

pub const QUICK_SLOT_CONTENT_WIDTH: f32 = 300.0;

pub const QUICK_SLOT_CONTENT_HEIGHT: f32 = 39.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuickSlotUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl QuickSlotUiRect {
    #[must_use]
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    pub(super) fn scaled_around_bottom_left(self, viewport_height: f32, scale: f32) -> Self {
        Self {
            left: self.left * scale,
            top: viewport_height + (self.top - viewport_height) * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.left),
            top: px(self.top),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

#[must_use]
pub fn clean_quick_slot_ui_scale(viewport_height: f32, scale_factor: f32) -> f32 {
    if !viewport_height.is_finite()
        || viewport_height <= 0.0
        || !scale_factor.is_finite()
        || scale_factor <= 0.0
    {
        return 1.0;
    }
    (viewport_height / QUICK_SLOT_SCALE_REFERENCE_HEIGHT * scale_factor).max(1.0)
}

pub(super) fn bind_optional_rect(node: &mut Node, rect: Option<QuickSlotUiRect>) {
    let Some(rect) = rect else {
        node.display = Display::None;
        return;
    };
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
