use super::*;

pub const OPTION_REFERENCE_WIDTH: f32 = 1_020.0;

pub const OPTION_REFERENCE_HEIGHT: f32 = 638.0;

/// Clean Font-object metrics retained while using the Cyrillic-capable native replacements.
pub const OPTION_JEFFE_12_LINE_HEIGHT: f32 = 13.56;

pub const OPTION_JEFFE_13_LINE_HEIGHT: f32 = 14.69;

pub const OPTION_JEFFE_14_LINE_HEIGHT: f32 = 11.3;

pub const OPTION_JEFFE_16_LINE_HEIGHT: f32 = 13.56;

pub const OPTION_COMIC_LINE_HEIGHT: f32 = 18.08;

pub const OPTION_CHALET_SMALL_LINE_HEIGHT: f32 = 13.56;

pub const OPTION_CHALET_LINE_HEIGHT: f32 = 15.82;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OptionUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl OptionUiRect {
    #[must_use]
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

pub const OPTION_WINDOW_RECT: OptionUiRect =
    OptionUiRect::new(0.0, 0.0, OPTION_REFERENCE_WIDTH, OPTION_REFERENCE_HEIGHT);

pub const OPTION_BACKDROP_RECT: OptionUiRect = OptionUiRect::new(-450.0, -401.0, 1_920.0, 1_440.0);

pub const OPTION_PAGE_RECT: OptionUiRect = OptionUiRect::new(20.0, 60.0, 960.0, 520.0);

pub const OPTION_CLOSE_RECT: OptionUiRect = OptionUiRect::new(987.0, 0.0, 30.0, 30.0);

pub const OPTION_APPLY_RECT: OptionUiRect = OptionUiRect::new(500.0, 600.0, 180.0, 25.0);

pub const OPTION_SAVE_RECT: OptionUiRect = OptionUiRect::new(695.0, 600.0, 180.0, 25.0);

pub const OPTION_DEAD_RESET_RECT: OptionUiRect = OptionUiRect::new(435.0, 600.0, 160.0, 25.0);

pub const OPTION_SOCIAL_REQUEST_HEADER_RECT: OptionUiRect =
    OptionUiRect::new(10.0, 220.0, 450.0, 45.0);

pub const OPTION_SOCIAL_REQUEST_LEFT_RECT: OptionUiRect =
    OptionUiRect::new(10.0, 250.0, 240.0, 260.0);

pub const OPTION_SOCIAL_REQUEST_CONNECTOR_RECT: OptionUiRect =
    OptionUiRect::new(250.0, 247.0, 26.0, 27.0);

pub const OPTION_SOCIAL_REQUEST_RIGHT_RECT: OptionUiRect =
    OptionUiRect::new(100.0, 264.0, 360.0, 246.0);

pub const OPTION_SOCIAL_BLOCKED_HEADER_RECT: OptionUiRect =
    OptionUiRect::new(490.0, 20.0, 450.0, 45.0);

pub const OPTION_SOCIAL_BLOCKED_BODY_RECT: OptionUiRect =
    OptionUiRect::new(490.0, 50.0, 450.0, 460.0);

pub const OPTION_SOCIAL_BLOCKED_LIST_RECT: OptionUiRect =
    OptionUiRect::new(565.0, 100.0, 300.0, 300.0);

pub const OPTION_SOCIAL_UNIGNORE_RECT: OptionUiRect = OptionUiRect::new(570.0, 415.0, 290.0, 25.0);

pub const OPTION_SOCIAL_DEFAULTS_RECT: OptionUiRect = OptionUiRect::new(275.0, 230.0, 180.0, 25.0);

pub const OPTION_GRAPHICS_LEFT_HEADER_RECT: OptionUiRect =
    OptionUiRect::new(10.0, 20.0, 450.0, 45.0);

pub const OPTION_GRAPHICS_RIGHT_HEADER_RECT: OptionUiRect =
    OptionUiRect::new(490.0, 20.0, 450.0, 45.0);

pub const OPTION_GRAPHICS_LEFT_BODY_RECT: OptionUiRect =
    OptionUiRect::new(10.0, 50.0, 240.0, 460.0);

pub const OPTION_GRAPHICS_LEFT_SIDE_RECT: OptionUiRect =
    OptionUiRect::new(220.0, 64.0, 240.0, 446.0);

pub const OPTION_GRAPHICS_RIGHT_BODY_RECT: OptionUiRect =
    OptionUiRect::new(490.0, 50.0, 240.0, 460.0);

pub const OPTION_GRAPHICS_RIGHT_SIDE_RECT: OptionUiRect =
    OptionUiRect::new(700.0, 64.0, 240.0, 446.0);

pub const OPTION_DISPLAY_HEADER_RECT: OptionUiRect = OptionUiRect::new(10.0, 20.0, 935.0, 45.0);

pub const OPTION_DISPLAY_BODY_RECT: OptionUiRect = OptionUiRect::new(10.0, 50.0, 650.0, 230.0);

pub const OPTION_DISPLAY_SIDE_RECT: OptionUiRect = OptionUiRect::new(560.0, 64.0, 385.0, 216.0);

pub const OPTION_CHAT_HEADER_RECT: OptionUiRect = OptionUiRect::new(10.0, 300.0, 935.0, 45.0);

pub const OPTION_CHAT_BODY_RECT: OptionUiRect = OptionUiRect::new(10.0, 330.0, 650.0, 180.0);

pub const OPTION_CHAT_SIDE_RECT: OptionUiRect = OptionUiRect::new(560.0, 344.0, 385.0, 166.0);

pub const OPTION_CONTROLS_HEADER_RECT: OptionUiRect = OptionUiRect::new(10.0, 20.0, 935.0, 45.0);

pub const OPTION_CONTROLS_INPUT_BODY_RECT: OptionUiRect =
    OptionUiRect::new(10.0, 50.0, 650.0, 90.0);

pub const OPTION_CONTROLS_INPUT_SIDE_RECT: OptionUiRect =
    OptionUiRect::new(560.0, 64.0, 385.0, 76.0);

pub const OPTION_KEYMAP_HEADER_RECT: OptionUiRect = OptionUiRect::new(10.0, 160.0, 935.0, 45.0);

pub const OPTION_KEYMAP_VIEW_RECT: OptionUiRect = OptionUiRect::new(25.0, 230.0, 900.0, 260.0);

pub const OPTION_KEYMAP_CONTENT_RECT: OptionUiRect = OptionUiRect::new(30.0, 235.0, 890.0, 250.0);

pub const OPTION_CONTROL_CONTENT_HEIGHT: f32 = 1_140.0;

pub const OPTION_BIG_LABEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 5.0),
    max_inset: Vec2::new(8.0, 5.0),
};

/// Clean `closebutton` uses the same 8/8/5/5 cap preservation as `button`.
pub const OPTION_CLOSE_BORDER: BorderRect = OPTION_BIG_LABEL_BORDER;

/// Clean `graphics_tab` preserves its three-pixel bottom edge while stretching.
pub const OPTION_GRAPHICS_TAB_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(0.0, 0.0),
    max_inset: Vec2::new(0.0, 3.0),
};

/// Clean built-in `toggle` keeps the 17-pixel checkbox cap.
pub const OPTION_TOGGLE_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(17.0, 0.0),
    max_inset: Vec2::new(0.0, 0.0),
};

pub const OPTION_DARK_BOX_BORDER: BorderRect = BorderRect::all(10.0);

pub const OPTION_PANEL_CONNECTOR_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 0.0),
    max_inset: Vec2::new(35.0, 0.0),
};

pub const OPTION_TEXT_FIELD_BORDER: BorderRect = BorderRect::all(2.0);

pub const OPTION_SCROLLBAR_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(2.0, 4.0),
    max_inset: Vec2::new(2.0, 4.0),
};

pub const OPTION_SCROLL_THUMB_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(2.0, 4.0),
    max_inset: Vec2::new(2.0, 4.0),
};

/// Serialized `verticalScrollbar` is 17 px wide with one pixel of right overflow.
pub const OPTION_SCROLLBAR_VISUAL_WIDTH: f32 = 18.0;

/// Serialized 15 px thumb draws two pixels beyond its left edge.
pub const OPTION_SCROLL_THUMB_VISUAL_WIDTH: f32 = 17.0;

pub const OPTION_SLIDER_TRACK_WIDTH: f32 = 170.0;

pub const OPTION_SLIDER_THUMB_WIDTH: f32 = 13.0;

pub const OPTION_SLIDER_THUMB_HEIGHT: f32 = 27.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptionUiLayout {
    pub viewport: Vec2,
    pub node_left: f32,
    pub node_top: f32,
    pub scale: Vec2,
    pub visual: OptionUiRect,
}

/// Mirrors the clean client: optional height-based UI scale, followed by
/// independent fit-to-1020x638 factors around the screen center.
#[must_use]
pub fn clean_option_ui_scale(viewport_height: f32, scale_ui: bool) -> f32 {
    if !scale_ui || !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / 768.0 * 1.05).max(1.0)
}

#[must_use]
pub fn option_ui_layout(viewport: Vec2, scale_ui: bool) -> OptionUiLayout {
    let viewport = Vec2::new(
        if viewport.x.is_finite() && viewport.x > 0.0 {
            viewport.x
        } else {
            0.0
        },
        if viewport.y.is_finite() && viewport.y > 0.0 {
            viewport.y
        } else {
            0.0
        },
    );
    let ui_scale = clean_option_ui_scale(viewport.y, scale_ui);
    let fit_x = (viewport.x / OPTION_REFERENCE_WIDTH).clamp(0.0, 1.0);
    let fit_y = (viewport.y / OPTION_REFERENCE_HEIGHT).clamp(0.0, 1.0);
    let scale = Vec2::new(ui_scale * fit_x, ui_scale * fit_y);
    let visual_width = OPTION_REFERENCE_WIDTH * scale.x;
    let visual_height = OPTION_REFERENCE_HEIGHT * scale.y;
    OptionUiLayout {
        viewport,
        node_left: (viewport.x - OPTION_REFERENCE_WIDTH) * 0.5,
        node_top: (viewport.y - OPTION_REFERENCE_HEIGHT) * 0.5,
        scale,
        visual: OptionUiRect::new(
            (viewport.x - visual_width) * 0.5,
            (viewport.y - visual_height) * 0.5,
            visual_width,
            visual_height,
        ),
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LegacyAxisDirection {
    Negative,
    Positive,
}

pub(super) fn update_option_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<OptionUiModel>,
    mut roots: Query<&mut Node, With<OptionUiRoot>>,
    mut windows_ui: Query<
        (&mut Node, &mut UiTransform),
        (With<OptionUiWindow>, Without<OptionUiRoot>),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = option_ui_layout(
        viewport,
        model.draft_options.display.scale_ui && window.resolution.scale_factor_override().is_none(),
    );
    for mut root in &mut roots {
        root.width = px(viewport.x);
        root.height = px(viewport.y);
    }
    for (mut node, mut transform) in &mut windows_ui {
        node.left = px(layout.node_left);
        node.top = px(layout.node_top);
        node.width = px(OPTION_REFERENCE_WIDTH);
        node.height = px(OPTION_REFERENCE_HEIGHT);
        transform.scale = layout.scale;
    }
}
