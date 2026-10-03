use super::*;

pub const TRANSPORTATION_WORLD_EXTENT: f32 = 8_192.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportationUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TransportationUiRect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn contains(self, point: TransportationUiPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    pub(super) fn is_valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

pub const TRANSPORTATION_BACKDROP_RECT: TransportationUiRect =
    TransportationUiRect::new(0.0, 0.0, 1_920.0, 1_440.0);

pub const TRANSPORTATION_WINDOW_RECT: TransportationUiRect =
    TransportationUiRect::new(0.0, 0.0, 1_036.0, 654.0);

pub const TRANSPORTATION_RIGHT_BACK_RECT: TransportationUiRect =
    TransportationUiRect::new(379.0, 0.0, 658.0, 652.0);

pub const TRANSPORTATION_LEFT_BACK_RECT: TransportationUiRect =
    TransportationUiRect::new(0.0, 0.0, 380.0, 652.0);

pub const TRANSPORTATION_LEFT_BOX_RECT: TransportationUiRect =
    TransportationUiRect::new(17.0, 65.0, 343.0, 577.0);

pub const TRANSPORTATION_SELECT_RECT: TransportationUiRect =
    TransportationUiRect::new(20.0, 148.0, 340.0, 445.0);

pub const TRANSPORTATION_MAP_RECT: TransportationUiRect =
    TransportationUiRect::new(390.0, 10.0, 590.0, 630.0);

pub const TRANSPORTATION_CLOSE_RECT: TransportationUiRect =
    TransportationUiRect::new(995.0, 8.0, 32.0, 32.0);

pub const TRANSPORTATION_GO_RECT: TransportationUiRect =
    TransportationUiRect::new(213.0, 604.0, 132.0, 26.0);

pub const TRANSPORTATION_TURBO_TOGGLE_RECT: TransportationUiRect =
    TransportationUiRect::new(33.0, 607.0, 20.0, 20.0);

pub const TRANSPORTATION_TURBO_BACKGROUND_RECT: TransportationUiRect =
    TransportationUiRect::new(30.0, 604.0, 26.0, 26.0);

pub const TRANSPORTATION_TURBO_INTERACT_RECT: TransportationUiRect =
    TransportationUiRect::new(30.0, 604.0, 170.0, 26.0);

pub const TRANSPORTATION_TURBO_LABEL_RECT: TransportationUiRect =
    TransportationUiRect::new(60.0, 610.0, 200.0, 26.0);

pub const TRANSPORTATION_TITLE_RECT: TransportationUiRect =
    TransportationUiRect::new(10.0, 10.0, 300.0, 30.0);

pub const TRANSPORTATION_SUBTITLE_RECT: TransportationUiRect =
    TransportationUiRect::new(10.0, 25.0, 300.0, 20.0);

pub const TRANSPORTATION_CAMERA_RECT: TransportationUiRect =
    TransportationUiRect::new(20.0, 34.0, 90.0, 90.0);

pub const TRANSPORTATION_BUBBLE_RECT: TransportationUiRect =
    TransportationUiRect::new(127.0, 53.0, 200.0, 44.0);

pub const TRANSPORTATION_WHERE_TO_RECT: TransportationUiRect =
    TransportationUiRect::new(182.0, 65.0, 140.0, 20.0);

pub const TRANSPORTATION_SCROLL_CONTENT_WIDTH: f32 = 322.0;

pub const TRANSPORTATION_SCROLLBAR_RECT: TransportationUiRect =
    TransportationUiRect::new(323.0, 12.0, 17.0, 421.0);

pub const TRANSPORTATION_SCROLL_UP_RECT: TransportationUiRect =
    TransportationUiRect::new(323.0, 0.0, 17.0, 12.0);

pub const TRANSPORTATION_SCROLL_DOWN_RECT: TransportationUiRect =
    TransportationUiRect::new(323.0, 433.0, 17.0, 12.0);

pub const TRANSPORTATION_SCROLL_THUMB_RECT: TransportationUiRect =
    TransportationUiRect::new(324.0, 12.0, 15.0, 15.0);

pub const TRANSPORTATION_FUTURE_ZONE_RECT: TransportationUiRect =
    TransportationUiRect::new(5_632.0, 512.0, 2_150.4, 2_150.4);

pub const TRANSPORTATION_DARK_ZONE_RECT: TransportationUiRect =
    TransportationUiRect::new(1_024.0, 5_120.0, 2_560.0, 2_048.0);

pub const TRANSPORTATION_TUTORIAL_RECT: TransportationUiRect =
    TransportationUiRect::new(0.0, 0.0, 1_536.0, 1_536.0);

pub const TRANSPORTATION_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const TRANSPORTATION_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub(super) fn transportation_marker_rect(
    world_position: TransportationUiPoint,
    view: TransportationUiRect,
) -> Option<TransportationUiRect> {
    let normalized = TransportationUiPoint::new(
        world_position.x / TRANSPORTATION_WORLD_EXTENT,
        world_position.y / TRANSPORTATION_WORLD_EXTENT,
    );
    if !normalized.is_finite() || !view.contains(normalized) {
        return None;
    }
    let width = 36.0;
    let height = 36.0;
    let x = TRANSPORTATION_MAP_RECT.x
        + (normalized.x - view.x) / view.width * TRANSPORTATION_MAP_RECT.width
        - width * 0.5;
    let y = TRANSPORTATION_MAP_RECT.y + TRANSPORTATION_MAP_RECT.height
        - (normalized.y - view.y) / view.height * TRANSPORTATION_MAP_RECT.height
        - height * 0.5;
    let rect = TransportationUiRect::new(x, y, width, height);
    if rect.x < TRANSPORTATION_MAP_RECT.x
        || rect.y < TRANSPORTATION_MAP_RECT.y
        || rect.x + rect.width > TRANSPORTATION_MAP_RECT.x + TRANSPORTATION_MAP_RECT.width
        || rect.y + rect.height > TRANSPORTATION_MAP_RECT.y + TRANSPORTATION_MAP_RECT.height
    {
        None
    } else {
        Some(rect)
    }
}

pub(super) fn apply_transportation_rect(node: &mut Node, rect: TransportationUiRect) {
    node.left = px(rect.x);
    node.top = px(rect.y);
    node.width = px(rect.width);
    node.height = px(rect.height);
}

pub(super) fn transportation_source_rect(view: TransportationUiRect) -> Option<BevyRect> {
    if !view.is_valid() {
        return None;
    }
    let width = 2_048.0;
    let height = 2_048.0;
    Some(BevyRect {
        min: Vec2::new(
            (view.x * width).clamp(0.0, width),
            ((1.0 - view.y - view.height) * height).clamp(0.0, height),
        ),
        max: Vec2::new(
            ((view.x + view.width) * width).clamp(0.0, width),
            ((1.0 - view.y) * height).clamp(0.0, height),
        ),
    })
}
