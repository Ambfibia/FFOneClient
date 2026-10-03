use super::*;

pub const NANOCOM_MESSAGE_WIDTH: f32 = 372.0;

pub const NANOCOM_MESSAGE_HEIGHT: f32 = 122.0;

pub const NANOCOM_WINDOW_WIDTH: f32 = 174.0;

pub const NANOCOM_SCALE_REFERENCE_HEIGHT: f32 = 768.0;

pub const NANOCOM_SCALE_FACTOR: f32 = 1.05;

pub const NANOCOM_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const NANOCOM_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const NANOCOM_MESSAGE_TITLE_LINE_HEIGHT: f32 = 17.822_999_95;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanocomRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl NanocomRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.x),
            top: px(self.y),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

pub const NANOCOM_COMPACT_ICON_RECT: NanocomRect = NanocomRect::new(60.0, 15.0, 64.0, 64.0);

pub const NANOCOM_NANO_ICON_RECT: NanocomRect = NanocomRect::new(55.0, 10.0, 64.0, 64.0);

pub const NANOCOM_COMPACT_TITLE_RECT: NanocomRect = NanocomRect::new(130.0, 2.0, 200.0, 30.0);

pub const NANOCOM_COMPACT_BODY_RECT: NanocomRect = NanocomRect::new(120.0, 25.0, 180.0, 70.0);

/// `MenuMessageText` after flattening its serialized L10/T4/R6/B6 padding.
pub const NANOCOM_COMPACT_BODY_CONTENT_RECT: NanocomRect =
    NanocomRect::new(130.0, 29.0, 164.0, 60.0);

pub const NANOCOM_EXPANDED_DIALOG_RECT: NanocomRect = NanocomRect::new(0.0, 0.0, 520.0, 164.0);

pub const NANOCOM_EXPANDED_MESSAGE_AREA_RECT: NanocomRect =
    NanocomRect::new(70.0, 30.0, 415.0, 87.0);

pub const NANOCOM_EXPANDED_ICON_RECT: NanocomRect = NanocomRect::new(25.0, 15.0, 62.0, 62.0);

pub const NANOCOM_EXPANDED_TITLE_RECT: NanocomRect = NanocomRect::new(100.0, -5.0, 500.0, 50.0);

pub const NANOCOM_EXPANDED_TEXT_RECT: NanocomRect = NanocomRect::new(90.0, 30.0, 340.0, 104.0);

/// `centerbox2` after flattening its serialized L10/T4/R6/B6 padding.
pub const NANOCOM_EXPANDED_TEXT_CONTENT_RECT: NanocomRect =
    NanocomRect::new(100.0, 34.0, 324.0, 94.0);

pub const NANOCOM_DECLINE_RECT: NanocomRect = NanocomRect::new(44.0, 124.0, 150.0, 25.0);

pub const NANOCOM_ACCEPT_RECT: NanocomRect = NanocomRect::new(334.0, 124.0, 150.0, 25.0);

pub const NANOCOM_DIALOG_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(40.0, 16.0),
    max_inset: Vec2::new(40.0, 40.0),
};

pub const NANOCOM_MESSAGE_AREA_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 2.0),
    max_inset: Vec2::new(8.0, 4.0),
};

pub const NANOCOM_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 5.0),
    max_inset: Vec2::new(8.0, 5.0),
};

#[must_use]
pub fn clean_nanocom_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / NANOCOM_SCALE_REFERENCE_HEIGHT * NANOCOM_SCALE_FACTOR).max(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanocomCompactLayout {
    /// Unscaled Bevy node bounds adjusted for center-origin `UiTransform`.
    pub node: NanocomRect,
    /// Bounds painted after the top-right-pivot clean-client scale.
    pub painted: NanocomRect,
    pub scale: f32,
}

#[must_use]
pub fn nanocom_compact_layout(
    viewport: Vec2,
    reveal_remaining: f32,
    ui_scale: f32,
) -> NanocomCompactLayout {
    let scale = valid_ui_scale(ui_scale);
    let remaining = reveal_remaining.clamp(0.0, 1.0);
    let squared = remaining * remaining;
    let sine = (squared * std::f32::consts::FRAC_PI_2).sin();
    let easing = sine * sine;
    let logical_left = viewport.x - NANOCOM_MESSAGE_WIDTH - NANOCOM_REVEALED_RIGHT_MARGIN
        + NANOCOM_MESSAGE_WIDTH * easing;
    let logical_width = NANOCOM_MESSAGE_WIDTH * (1.0 - easing);
    let painted = NanocomRect::new(
        viewport.x + (logical_left - viewport.x) * scale,
        0.0,
        logical_width * scale,
        NANOCOM_MESSAGE_HEIGHT * scale,
    );
    NanocomCompactLayout {
        node: NanocomRect::new(
            painted.x + logical_width * (scale - 1.0) * 0.5,
            NANOCOM_MESSAGE_HEIGHT * (scale - 1.0) * 0.5,
            logical_width,
            NANOCOM_MESSAGE_HEIGHT,
        ),
        painted,
        scale,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanocomExpandedLayout {
    pub node: NanocomRect,
    pub painted: NanocomRect,
    pub scale: f32,
}

#[must_use]
pub fn nanocom_expanded_layout(viewport: Vec2, ui_scale: f32) -> NanocomExpandedLayout {
    let scale = valid_ui_scale(ui_scale);
    let node = NanocomRect::new(
        viewport.x * 0.5 - NANOCOM_EXPANDED_DIALOG_RECT.width * 0.5,
        viewport.y * 0.5 - NANOCOM_EXPANDED_DIALOG_RECT.height * 0.5,
        NANOCOM_EXPANDED_DIALOG_RECT.width,
        NANOCOM_EXPANDED_DIALOG_RECT.height,
    );
    NanocomExpandedLayout {
        painted: NanocomRect::new(
            viewport.x * 0.5 - NANOCOM_EXPANDED_DIALOG_RECT.width * scale * 0.5,
            viewport.y * 0.5 - NANOCOM_EXPANDED_DIALOG_RECT.height * scale * 0.5,
            NANOCOM_EXPANDED_DIALOG_RECT.width * scale,
            NANOCOM_EXPANDED_DIALOG_RECT.height * scale,
        ),
        node,
        scale,
    }
}
