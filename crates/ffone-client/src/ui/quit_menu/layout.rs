use super::*;

pub const QUIT_MENU_REFERENCE_HEIGHT: f32 = 768.0;

pub const QUIT_MENU_UI_SCALE_NUDGE: f32 = 1.05;

pub const QUIT_MENU_FONT_LINE_HEIGHT: f32 = 13.710_000_04;

pub const QUIT_MENU_DIALOG_RECT: QuitMenuUiRect = QuitMenuUiRect::new(0.0, 0.0, 206.0, 188.0);

pub const QUIT_MENU_BUTTON_RECT: QuitMenuUiRect = QuitMenuUiRect::new(13.0, 16.0, 175.0, 45.0);

pub const QUIT_MENU_BACKDROP_BORDER: BorderRect = BorderRect::all(1.0);

pub const QUIT_MENU_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

pub const QUIT_MENU_CANCEL_BORDER: BorderRect = BorderRect::all(5.0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuitMenuUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl QuitMenuUiRect {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuitMenuScaledGroupLayout {
    /// Unscaled Bevy layout origin. The visual is transformed around its own
    /// center after this origin is adjusted to match the legacy screen pivot.
    pub node_left: f32,
    pub node_top: f32,
    pub source_width: f32,
    pub source_height: f32,
    pub scale: f32,
    /// Final visual bounds after the center-origin transform.
    pub visual: QuitMenuUiRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuitMenuUiLayout {
    pub viewport: Vec2,
    pub pivot: Vec2,
    pub scale: f32,
    pub backdrop: QuitMenuScaledGroupLayout,
    pub dialog: QuitMenuScaledGroupLayout,
}

/// Exact default `FFGUIUtility.GetUiScale` contract:
/// `max(1, Screen.height / 768 * 1.05)`.
#[must_use]
pub fn clean_quit_menu_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / QUIT_MENU_REFERENCE_HEIGHT * QUIT_MENU_UI_SCALE_NUDGE).max(1.0)
}

#[must_use]
pub fn quit_menu_ui_layout(viewport: Vec2, ui_scale: f32) -> QuitMenuUiLayout {
    let width = legacy_screen_extent(viewport.x);
    let height = legacy_screen_extent(viewport.y);
    let viewport = Vec2::new(width as f32, height as f32);
    let pivot = Vec2::new((width / 2) as f32, (height / 2) as f32);
    let scale = valid_ui_scale(ui_scale);

    let dialog_x = ((width - QUIT_MENU_DIALOG_RECT.width as i32) / 2) as f32;
    let dialog_y = ((height - QUIT_MENU_DIALOG_RECT.height as i32) / 2) as f32;
    let dialog = scaled_group_layout(
        QuitMenuUiRect::new(
            dialog_x,
            dialog_y,
            QUIT_MENU_DIALOG_RECT.width,
            QUIT_MENU_DIALOG_RECT.height,
        ),
        pivot,
        scale,
    );
    let backdrop = scaled_group_layout(
        QuitMenuUiRect::new(0.0, 0.0, viewport.x, viewport.y),
        pivot,
        scale,
    );

    QuitMenuUiLayout {
        viewport,
        pivot,
        scale,
        backdrop,
        dialog,
    }
}

pub(super) fn scaled_group_layout(
    source: QuitMenuUiRect,
    pivot: Vec2,
    scale: f32,
) -> QuitMenuScaledGroupLayout {
    let source_center = source.center();
    let visual_center = pivot + (source_center - pivot) * scale;
    let node_left = visual_center.x - source.width * 0.5;
    let node_top = visual_center.y - source.height * 0.5;
    QuitMenuScaledGroupLayout {
        node_left,
        node_top,
        source_width: source.width,
        source_height: source.height,
        scale,
        visual: QuitMenuUiRect::new(
            visual_center.x - source.width * scale * 0.5,
            visual_center.y - source.height * scale * 0.5,
            source.width * scale,
            source.height * scale,
        ),
    }
}

pub(super) fn update_quit_menu_layout(
    model: Res<QuitMenuUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        &mut Node,
        (
            With<QuitMenuUiRoot>,
            Without<QuitMenuBackdrop>,
            Without<QuitMenuDialog>,
        ),
    >,
    mut backdrops: Query<
        (&mut Node, &mut UiTransform),
        (
            With<QuitMenuBackdrop>,
            Without<QuitMenuUiRoot>,
            Without<QuitMenuDialog>,
        ),
    >,
    mut dialogs: Query<
        (&mut Node, &mut UiTransform),
        (
            With<QuitMenuDialog>,
            Without<QuitMenuUiRoot>,
            Without<QuitMenuBackdrop>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = quit_menu_ui_layout(viewport, model.effective_ui_scale(viewport.y));
    for mut root in &mut roots {
        root.width = px(layout.viewport.x);
        root.height = px(layout.viewport.y);
    }
    for (mut node, mut transform) in &mut backdrops {
        apply_scaled_group(&mut node, &mut transform, layout.backdrop);
    }
    for (mut node, mut transform) in &mut dialogs {
        apply_scaled_group(&mut node, &mut transform, layout.dialog);
    }
}

pub(super) fn apply_scaled_group(
    node: &mut Node,
    transform: &mut UiTransform,
    layout: QuitMenuScaledGroupLayout,
) {
    node.left = px(layout.node_left);
    node.top = px(layout.node_top);
    node.width = px(layout.source_width);
    node.height = px(layout.source_height);
    transform.scale = Vec2::splat(layout.scale);
}
