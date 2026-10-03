use super::*;

/// `FFGUIUtility.GetUiScale`: `max(1, Screen.height / 768 * 1.05)`.
pub const RULE_UI_REFERENCE_HEIGHT: f32 = 768.0;

pub const RULE_UI_SCALE_NUDGE: f32 = 1.05;

pub const RULE_UI_JEFFE_12_LINE_HEIGHT: f32 = 12.338_999_75;

pub const RULE_UI_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub const RULE_UI_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const RULE_UI_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RuleUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RuleUiRect {
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
    pub const fn translated(self, x: f32, y: f32) -> Self {
        Self::new(self.x + x, self.y + y, self.width, self.height)
    }

    #[must_use]
    pub const fn center(self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
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

pub const RULE_UI_BACKGROUND_RECT: RuleUiRect = RuleUiRect::new(0.0, 0.0, 1_920.0, 1_440.0);

pub const RULE_UI_WINDOW_RECT: RuleUiRect = RuleUiRect::new(0.0, 0.0, 1_020.0, 638.0);

pub const RULE_UI_CLOSE_RECT: RuleUiRect = RuleUiRect::new(987.0, 0.0, 30.0, 30.0);

pub const RULE_UI_BACK_RECT: RuleUiRect = RuleUiRect::new(500.0, 605.0, 80.0, 25.0);

pub const RULE_UI_RULE_BACK_RECT: RuleUiRect = RuleUiRect::new(-6.0, 42.0, 1_034.0, 551.0);

pub const RULE_UI_TITLE_RECT: RuleUiRect = RuleUiRect::new(15.0, -10.0, 500.0, 60.0);

pub const RULE_UI_SUBTITLE_1_RECT: RuleUiRect = RuleUiRect::new(0.0, -35.0, 500.0, 100.0);

pub const RULE_UI_SUBTITLE_2_RECT: RuleUiRect = RuleUiRect::new(130.0, 25.0, 100.0, 50.0);

pub const RULE_UI_CONTENT_1_RECT: RuleUiRect = RuleUiRect::new(130.0, 60.0, 380.0, 300.0);

pub const RULE_UI_SUBTITLE_3_RECT: RuleUiRect = RuleUiRect::new(560.0, 250.0, 200.0, 100.0);

pub const RULE_UI_CONTENT_2_RECT: RuleUiRect = RuleUiRect::new(560.0, 320.0, 320.0, 320.0);

pub const RULE_UI_LAST_COMMENT_RECT: RuleUiRect = RuleUiRect::new(0.0, 505.0, 1_000.0, 50.0);

pub const RULE_UI_PREVIOUS_RECT: RuleUiRect = RuleUiRect::new(560.0, 320.0, 320.0, 320.0);

pub const RULE_UI_NEXT_RECT: RuleUiRect = RuleUiRect::new(0.0, 505.0, 1_000.0, 50.0);

pub const RULE_UI_IMAGE_RECTS: [RuleUiRect; 4] = [
    RuleUiRect::new(15.0, 40.0, 104.0, 91.0),
    RuleUiRect::new(0.0, 150.0, 542.0, 361.0),
    RuleUiRect::new(550.0, 0.0, 452.0, 282.0),
    RuleUiRect::new(895.0, 305.0, 103.0, 201.0),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleScaledGroup {
    pub source: RuleUiRect,
    pub node_left: f32,
    pub node_top: f32,
    pub scale: Vec2,
    pub painted: RuleUiRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleUiLayout {
    pub viewport: Vec2,
    pub legacy_pivot: Vec2,
    pub fit_pivot: Vec2,
    pub ui_scale: f32,
    pub fit_scale: Vec2,
    pub combined_scale: Vec2,
    pub background: RuleScaledGroup,
    pub window: RuleScaledGroup,
}

#[must_use]
pub fn clean_rule_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / RULE_UI_REFERENCE_HEIGHT * RULE_UI_SCALE_NUDGE).max(1.0)
}

#[must_use]
pub fn clean_rule_fit_scale(viewport: Vec2) -> Vec2 {
    let width = finite_nonnegative(viewport.x);
    let height = finite_nonnegative(viewport.y);
    Vec2::new(
        (width / RULE_UI_WINDOW_RECT.width).clamp(0.0, 1.0),
        (height / RULE_UI_WINDOW_RECT.height).clamp(0.0, 1.0),
    )
}

#[must_use]
pub fn rule_ui_layout(viewport: Vec2, ui_scale: f32) -> RuleUiLayout {
    let width = legacy_screen_extent(viewport.x);
    let height = legacy_screen_extent(viewport.y);
    let viewport = Vec2::new(width as f32, height as f32);
    let legacy_pivot = Vec2::new((width / 2) as f32, (height / 2) as f32);
    let fit_pivot = viewport * 0.5;
    let ui_scale = valid_scale(ui_scale);
    let fit_scale = clean_rule_fit_scale(viewport);
    let combined_scale = fit_scale * ui_scale;

    // The backdrop uses integer division in `cnRule.OnGUI`; negative odd
    // differences therefore truncate toward zero exactly as C# does.
    let background = RuleUiRect::new(
        ((width - RULE_UI_BACKGROUND_RECT.width as i32) / 2) as f32,
        ((height - RULE_UI_BACKGROUND_RECT.height as i32) / 2) as f32,
        RULE_UI_BACKGROUND_RECT.width,
        RULE_UI_BACKGROUND_RECT.height,
    );
    let window = RuleUiRect::new(
        (viewport.x - RULE_UI_WINDOW_RECT.width) * 0.5,
        (viewport.y - RULE_UI_WINDOW_RECT.height) * 0.5,
        RULE_UI_WINDOW_RECT.width,
        RULE_UI_WINDOW_RECT.height,
    );

    RuleUiLayout {
        viewport,
        legacy_pivot,
        fit_pivot,
        ui_scale,
        fit_scale,
        combined_scale,
        background: scaled_group(background, legacy_pivot, ui_scale, fit_pivot, fit_scale),
        window: scaled_group(window, legacy_pivot, ui_scale, fit_pivot, fit_scale),
    }
}

pub(super) fn scaled_group(
    source: RuleUiRect,
    legacy_pivot: Vec2,
    ui_scale: f32,
    fit_pivot: Vec2,
    fit_scale: Vec2,
) -> RuleScaledGroup {
    let source_center = source.center();
    let after_ui_scale = legacy_pivot + (source_center - legacy_pivot) * ui_scale;
    let painted_center = fit_pivot + (after_ui_scale - fit_pivot) * fit_scale;
    let scale = fit_scale * ui_scale;
    RuleScaledGroup {
        source,
        node_left: painted_center.x - source.width * 0.5,
        node_top: painted_center.y - source.height * 0.5,
        scale,
        painted: RuleUiRect::new(
            painted_center.x - source.width * scale.x * 0.5,
            painted_center.y - source.height * scale.y * 0.5,
            source.width * scale.x,
            source.height * scale.y,
        ),
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_rule_ui_root_and_layout(
    model: Res<RuleUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        &mut Node,
        (
            With<RuleUiRoot>,
            Without<RuleUiBackground>,
            Without<RuleUiWindow>,
        ),
    >,
    mut backgrounds: Query<
        (&mut Node, &mut UiTransform),
        (
            With<RuleUiBackground>,
            Without<RuleUiRoot>,
            Without<RuleUiWindow>,
        ),
    >,
    mut rule_windows: Query<
        (&mut Node, &mut UiTransform),
        (
            With<RuleUiWindow>,
            Without<RuleUiRoot>,
            Without<RuleUiBackground>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = rule_ui_layout(viewport, model.effective_ui_scale(viewport.y));

    for mut root in &mut roots {
        root.width = px(layout.viewport.x);
        root.height = px(layout.viewport.y);
        root.display = if model.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (mut node, mut transform) in &mut backgrounds {
        apply_scaled_group(&mut node, &mut transform, layout.background);
    }
    for (mut node, mut transform) in &mut rule_windows {
        apply_scaled_group(&mut node, &mut transform, layout.window);
    }
}

pub(super) fn apply_scaled_group(node: &mut Node, transform: &mut UiTransform, layout: RuleScaledGroup) {
    node.left = px(layout.node_left);
    node.top = px(layout.node_top);
    node.width = px(layout.source.width);
    node.height = px(layout.source.height);
    transform.scale = layout.scale;
}
