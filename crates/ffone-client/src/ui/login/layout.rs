use super::*;

pub(super) const LOGIN_USERNAME_LABEL_RECT: UiSourceRect = UiSourceRect::new(50.0, 15.0, 200.0, 20.0);

pub(super) const LOGIN_USERNAME_FIELD_RECT: UiSourceRect = UiSourceRect::new(45.0, 40.0, 280.0, 25.0);

pub(super) const LOGIN_PASSWORD_LABEL_RECT: UiSourceRect = UiSourceRect::new(50.0, 75.0, 200.0, 20.0);

pub(super) const LOGIN_PASSWORD_FIELD_RECT: UiSourceRect = UiSourceRect::new(45.0, 100.0, 280.0, 25.0);

pub(super) const LOGIN_SUBMIT_RECT: UiSourceRect = UiSourceRect::new(45.0, 135.0, 280.0, 35.0);

pub(super) const LOGIN_LANGUAGE_RECT: UiSourceRect = UiSourceRect::new(45.0, 178.0, 280.0, 35.0);

// These controls are not drawn from serialized `buttonRect2`. `DoLoginWindow`
// resets `rectPos` to (0, 0), then builds a 370x275 GUILayout area containing
// horizontal FlexibleSpace / a vertical column / FlexibleSpace. Clean
// `JEFFE___16` advances make "Discord Community" 187 px; Button padding adds
// 10+6 px, so the English column is 203 px wide. The old fusion-2.x.x
// GUILayoutGroup rounds its centered x to 83. Button line spacing plus 3+6 px
// padding rounds to 25 px; the explicit Space(5) and collapsed 4 px margins
// put the second button at y=234. The column remains content-sized so a longer
// localized label grows both buttons together, exactly as GUILayout does.
pub const LOGIN_GLAYOUT_TOP: f32 = 200.0;

pub const LOGIN_GLAYOUT_EN_COMMUNITY_ADVANCE: f32 = 187.0;

pub const LOGIN_GLAYOUT_EN_REGISTER_ADVANCE: f32 = 182.0;

pub const LOGIN_GLAYOUT_EN_MIN_WIDTH: f32 = LOGIN_GLAYOUT_EN_COMMUNITY_ADVANCE + 10.0 + 6.0;

pub const LOGIN_GLAYOUT_EN_X: f32 = 83.0;

pub const LOGIN_GLAYOUT_BUTTON_HEIGHT: f32 = 25.0;

pub const LOGIN_GLAYOUT_BUTTON_GAP: f32 = 9.0;

pub const LOGIN_GLAYOUT_REGISTER_Y: f32 = 234.0;

pub(super) const LOGIN_GLAYOUT_LEADING_FLEX: f32 = 10_000.0;

// The source column contributes two horizontal stretch units (one per Button).
// Bevy rounds flex edges instead of Unity's independently rounded x/width, so
// those two sub-pixel units are folded into the trailing spacer. This preserves
// Unity's exact integer button rect while retaining its 10000:2:10000 bias.
pub(super) const LOGIN_GLAYOUT_TRAILING_FLEX: f32 = 10_002.0;

pub const LOGIN_JEFFE_LINE_HEIGHT: f32 = 16.451_999_66;

pub const LOGIN_CHALET_LINE_HEIGHT: f32 = 14.083_999_63;

pub const LOGIN_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 18.0),
    max_inset: Vec2::new(8.0, 8.0),
};

pub const LOGIN_BUTTON_BORDER: BorderRect = BorderRect::all(5.0);

pub const LOGIN_TEXT_FIELD_BORDER: BorderRect = BorderRect::all(2.0);

pub const LOGIN_UI_SCALE_REFERENCE_HEIGHT: f32 = 768.0;

pub const LOGIN_UI_SCALE_NUDGE: f32 = 1.05;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiSourceRect {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

impl UiSourceRect {
    pub(super) const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiResolvedRect {
    pub(super) origin: Vec2,
    pub(super) size: Vec2,
}

pub(super) fn login_ui_scale(viewport_height: f32, enabled: bool, window_canvas_scaled: bool) -> f32 {
    if enabled && !window_canvas_scaled {
        (viewport_height / LOGIN_UI_SCALE_REFERENCE_HEIGHT * LOGIN_UI_SCALE_NUDGE).max(1.0)
    } else {
        1.0
    }
}

#[derive(Component)]
pub(super) struct LoginGLayoutArea;

#[derive(Component)]
pub(super) struct LoginGLayoutColumn;

#[derive(Component)]
pub(super) struct LoginGLayoutFlexibleSpace;

pub(super) fn update_login_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    option_ui: Res<OptionUiModel>,
    model: Option<Res<LoginUiModel>>,
    mut fallback_background: Single<
        &mut Node,
        (
            With<LoginFallbackBackground>,
            Without<LoginLoadedBackground>,
            Without<LoginPanel>,
        ),
    >,
    mut loaded_background: Single<
        &mut Node,
        (
            With<LoginLoadedBackground>,
            Without<LoginFallbackBackground>,
            Without<LoginPanel>,
        ),
    >,
    mut panel: Single<
        (&mut Node, &mut UiTransform),
        (
            With<LoginPanel>,
            Without<LoginFallbackBackground>,
            Without<LoginLoadedBackground>,
        ),
    >,
    mut actions: Query<(&mut Node, Has<LoginSubmitButton>, Has<LoginLanguageButton>),
        (Or<(With<LoginSubmitButton>, With<LoginLanguageButton>, With<LoginGLayoutArea>)>,
         Without<LoginPanel>, Without<LoginFallbackBackground>, Without<LoginLoadedBackground>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let fallback = LoginBackgroundMode0104::FallbackScaleToFit
        .resolved_rect(viewport)
        .expect("ScaleToFit always has a source rectangle");
    apply_resolved_rect(&mut fallback_background, fallback);
    let loaded = LoginBackgroundMode0104::LoadedScaleAndCrop
        .resolved_rect(viewport)
        .expect("ScaleAndCrop always has a source rectangle");
    apply_resolved_rect(&mut loaded_background, loaded);

    let extended = model.as_ref().is_some_and(|m| m.visible);
    let height = LOGIN_PANEL_SIZE.y + if extended {24.0} else {0.0};
    let panel_origin = (viewport - Vec2::new(LOGIN_PANEL_SIZE.x, height)) * 0.5;
    panel.0.height = px(height);
    for (mut node, submit, language) in &mut actions {
        let top = if submit { LOGIN_SUBMIT_RECT.y } else if language { LOGIN_LANGUAGE_RECT.y }
            else { LOGIN_LANGUAGE_RECT.y + LOGIN_LANGUAGE_RECT.height + 8.0 };
        node.top = px(top + if extended {24.0} else {0.0});
    }
    panel.0.left = px(panel_origin.x);
    panel.0.top = px(panel_origin.y);
    panel.1.scale = Vec2::splat(login_ui_scale(
        viewport.y,
        option_ui.persisted_options.display.scale_ui,
        window.resolution.scale_factor_override().is_some(),
    ));
}

pub(super) fn apply_resolved_rect(node: &mut Node, rect: UiResolvedRect) {
    node.left = px(rect.origin.x);
    node.top = px(rect.origin.y);
    node.width = px(rect.size.x);
    node.height = px(rect.size.y);
}
