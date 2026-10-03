use super::*;

pub const GUIDE_UI_REFERENCE_HEIGHT: f32 = 768.0;

pub const GUIDE_UI_SCALE_NUDGE: f32 = 1.05;

pub const GUIDE_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const GUIDE_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub const GUIDE_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const GUIDE_CARD_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(10.0, 35.0),
    max_inset: Vec2::new(10.0, 10.0),
};

pub const GUIDE_DIALOG_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(40.0, 16.0),
    max_inset: Vec2::new(40.0, 40.0),
};

pub const GUIDE_BLUE_BUTTON_BORDER: BorderRect = BorderRect::all(5.0);

pub const GUIDE_CANCEL_BUTTON_BORDER: BorderRect = BorderRect::all(5.0);

pub const GUIDE_BACKGROUND_RECT: GuideUiRect = GuideUiRect::new(0.0, 0.0, 1_920.0, 1_440.0);

pub const GUIDE_WINDOW_RECT: GuideUiRect = GuideUiRect::new(0.0, 0.0, 1_036.0, 654.0);

pub const GUIDE_COMPUTRESS_ICON_RECT: GuideUiRect = GuideUiRect::new(22.0, 22.0, 64.0, 64.0);

pub const GUIDE_HEADING_RECT: GuideUiRect = GuideUiRect::new(0.0, 45.0, 1_036.0, 30.0);

pub const GUIDE_INTRO_RECT: GuideUiRect = GuideUiRect::new(95.0, 35.0, 265.0, 70.0);

pub const GUIDE_CARD_RECT: GuideUiRect = GuideUiRect::new(0.0, 166.0, 200.0, 124.0);

pub const GUIDE_COMMENT_RECT: GuideUiRect = GuideUiRect::new(10.0, 35.0, 180.0, 50.0);

pub const GUIDE_TOGGLE_RECT: GuideUiRect = GuideUiRect::new(170.0, 5.0, 22.0, 22.0);

pub const GUIDE_COST_BAR_RECT: GuideUiRect = GuideUiRect::new(0.0, 572.0, 1_036.0, 20.0);

pub const GUIDE_PRIMARY_BUTTON_RECT: GuideUiRect = GuideUiRect::new(840.0, 605.0, 180.0, 35.0);

pub const GUIDE_CANCEL_BUTTON_RECT: GuideUiRect = GuideUiRect::new(10.0, 617.0, 150.0, 25.0);

pub const GUIDE_CLOSE_BUTTON_RECT: GuideUiRect = GuideUiRect::new(994.0, 10.0, 32.0, 32.0);

pub const GUIDE_HELP_BUTTON_RECT: GuideUiRect = GuideUiRect::new(987.0, 530.0, 32.0, 32.0);

pub const GUIDE_MODAL_RECT: GuideUiRect = GuideUiRect::new(258.0, 245.0, 520.0, 164.0);

pub const GUIDE_MODAL_ICON_RECT: GuideUiRect = GuideUiRect::new(60.0, 27.0, 62.0, 62.0);

pub const GUIDE_MODAL_CONFIRM_RECT: GuideUiRect = GuideUiRect::new(320.0, 124.0, 150.0, 25.0);

pub const GUIDE_MODAL_CANCEL_RECT: GuideUiRect = GuideUiRect::new(44.0, 124.0, 150.0, 25.0);

pub const GUIDE_WARP_BUTTON_RECT: GuideUiRect = GuideUiRect::new(280.0, 124.0, 200.0, 25.0);

pub const GUIDE_MODAL_TEXT_RECT: GuideUiRect = GuideUiRect::new(398.0, 265.0, 340.0, 104.0);

/// `GUILayout.Label(default label)` consumes one JEFFE14 line plus its exact
/// top/bottom padding before the clean `GUILayout.Space(10)` call.
pub const GUIDE_MODAL_TITLE_RECT: GuideUiRect = GuideUiRect::new(
    GUIDE_MODAL_TEXT_RECT.x,
    GUIDE_MODAL_TEXT_RECT.y,
    GUIDE_MODAL_TEXT_RECT.width,
    GUIDE_JEFFE_14_LINE_HEIGHT + 6.0,
);

pub const GUIDE_MODAL_BODY_RECT: GuideUiRect = GuideUiRect::new(
    GUIDE_MODAL_TEXT_RECT.x,
    GUIDE_MODAL_TEXT_RECT.y + GUIDE_MODAL_TITLE_RECT.height + 10.0,
    GUIDE_MODAL_TEXT_RECT.width,
    GUIDE_MODAL_TEXT_RECT.height - GUIDE_MODAL_TITLE_RECT.height - 10.0,
);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GuideUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl GuideUiRect {
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuideScaledGroup {
    pub source: GuideUiRect,
    pub node_left: f32,
    pub node_top: f32,
    pub scale: f32,
    pub painted: GuideUiRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuideUiLayout {
    pub viewport: Vec2,
    pub pivot: Vec2,
    pub scale: f32,
    pub background: GuideScaledGroup,
    pub window: GuideScaledGroup,
    pub overlay: GuideScaledGroup,
}

/// Exact default `cnGraphicOption.GetUiScale` contract:
/// `max(1, Screen.height / 768 * 1.05)`.
#[must_use]
pub fn clean_guide_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / GUIDE_UI_REFERENCE_HEIGHT * GUIDE_UI_SCALE_NUDGE).max(1.0)
}

#[must_use]
pub fn guide_ui_layout(viewport: Vec2, ui_scale: f32) -> GuideUiLayout {
    let width = legacy_screen_extent(viewport.x);
    let height = legacy_screen_extent(viewport.y);
    let viewport = Vec2::new(width as f32, height as f32);
    let pivot = Vec2::new((width / 2) as f32, (height / 2) as f32);
    let scale = valid_scale(ui_scale);
    let background = GuideUiRect::new(
        ((width - GUIDE_BACKGROUND_RECT.width as i32) / 2) as f32,
        ((height - GUIDE_BACKGROUND_RECT.height as i32) / 2) as f32,
        GUIDE_BACKGROUND_RECT.width,
        GUIDE_BACKGROUND_RECT.height,
    );
    let window = GuideUiRect::new(
        ((width - GUIDE_WINDOW_RECT.width as i32) / 2) as f32,
        ((height - GUIDE_WINDOW_RECT.height as i32) / 2) as f32,
        GUIDE_WINDOW_RECT.width,
        GUIDE_WINDOW_RECT.height,
    );
    GuideUiLayout {
        viewport,
        pivot,
        scale,
        background: scaled_group(background, pivot, scale),
        window: scaled_group(window, pivot, scale),
        overlay: scaled_group(
            GuideUiRect::new(0.0, 0.0, viewport.x, viewport.y),
            pivot,
            scale,
        ),
    }
}

pub(super) fn scaled_group(source: GuideUiRect, pivot: Vec2, scale: f32) -> GuideScaledGroup {
    let center = source.center();
    let painted_center = pivot + (center - pivot) * scale;
    GuideScaledGroup {
        source,
        node_left: painted_center.x - source.width * 0.5,
        node_top: painted_center.y - source.height * 0.5,
        scale,
        painted: GuideUiRect::new(
            painted_center.x - source.width * scale * 0.5,
            painted_center.y - source.height * scale * 0.5,
            source.width * scale,
            source.height * scale,
        ),
    }
}

/// Exact `cnGuideMode` cost-label/FM-icon relation once the legacy label
/// renderer has measured the localized cost string.
#[must_use]
pub fn guide_cost_text_and_icon_rects(text_width: f32) -> (GuideUiRect, GuideUiRect) {
    let width = if text_width.is_finite() {
        text_width.max(0.0)
    } else {
        0.0
    };
    (
        GuideUiRect::new(
            GUIDE_WINDOW_RECT.width * 0.5 - width * 0.5 - 15.0,
            572.0,
            width,
            20.0,
        ),
        GuideUiRect::new(
            GUIDE_WINDOW_RECT.width * 0.5 + width - 15.0,
            567.0,
            31.0,
            31.0,
        ),
    )
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_guide_ui_root_and_layout(
    model: Res<GuideUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: Res<GuideUiAssets>,
    mut elements: Query<
        (
            &mut Node,
            Option<&mut UiTransform>,
            Option<&mut ImageNode>,
            Option<&GuideUiRoot>,
            Option<&GuideUiBackground>,
            Option<&GuideUiSelectionWindow>,
            Option<&GuideUiWarpWindow>,
            Option<&GuideUiConfirmationOverlay>,
            Option<&GuideUiConfirmationWindow>,
        ),
        Or<(
            With<GuideUiRoot>,
            With<GuideUiBackground>,
            With<GuideUiSelectionWindow>,
            With<GuideUiWarpWindow>,
            With<GuideUiConfirmationOverlay>,
            With<GuideUiConfirmationWindow>,
        )>,
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = guide_ui_layout(viewport, model.effective_ui_scale(viewport.y));
    let selection_visible = model.visible && model.phase == GuideUiPhase::MentorSelection;
    let warp_visible = model.visible && model.phase == GuideUiPhase::WarpWarning;
    let confirmation_visible =
        selection_visible && model.confirmation_open && model.selected.is_some();

    for (
        mut node,
        mut transform,
        mut image,
        root,
        background,
        selection,
        warp,
        overlay,
        confirmation,
    ) in &mut elements
    {
        if root.is_some() {
            node.width = px(layout.viewport.x);
            node.height = px(layout.viewport.y);
            node.display = display(model.visible);
            continue;
        }

        let Some(transform) = transform.as_deref_mut() else {
            continue;
        };
        if background.is_some() {
            node.display = display(model.visible);
            apply_scaled_group(&mut node, transform, layout.background);
            if let Some(image) = image.as_deref_mut() {
                image.image = if model.phase == GuideUiPhase::WarpWarning {
                    assets.select_background.clone()
                } else {
                    assets.change_background.clone()
                };
            }
        } else if selection.is_some() {
            node.display = display(selection_visible);
            apply_scaled_group(&mut node, transform, layout.window);
        } else if warp.is_some() {
            node.display = display(warp_visible);
            apply_scaled_group(&mut node, transform, layout.window);
        } else if overlay.is_some() {
            node.display = display(confirmation_visible);
            apply_scaled_group(&mut node, transform, layout.overlay);
        } else if confirmation.is_some() {
            node.display = display(confirmation_visible);
            apply_scaled_group(&mut node, transform, layout.window);
        }
    }
}

pub(super) fn sync_guide_ui_cost_geometry(
    mut cost_texts: Query<(&ComputedNode, &mut Node), With<GuideUiCostText>>,
    mut cost_icons: Query<&mut Node, (With<GuideUiCostIcon>, Without<GuideUiCostText>)>,
) {
    let Ok((computed, mut text_node)) = cost_texts.single_mut() else {
        return;
    };
    let (text_rect, icon_rect) = guide_cost_text_and_icon_rects(computed.size().x);
    text_node.left = px(text_rect.x);
    text_node.top = px(text_rect.y);
    text_node.width = Val::Auto;
    text_node.height = px(text_rect.height);
    let Ok(mut icon_node) = cost_icons.single_mut() else {
        return;
    };
    apply_rect(&mut icon_node, icon_rect);
}

pub(super) fn apply_scaled_group(node: &mut Node, transform: &mut UiTransform, layout: GuideScaledGroup) {
    node.left = px(layout.node_left);
    node.top = px(layout.node_top);
    node.width = px(layout.source.width);
    node.height = px(layout.source.height);
    transform.scale = Vec2::splat(layout.scale);
}

pub(super) fn apply_rect(node: &mut Node, rect: GuideUiRect) {
    node.left = px(rect.x);
    node.top = px(rect.y);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
