use super::*;

pub const UPSELL_REFERENCE_HEIGHT: f32 = 768.0;

pub const UPSELL_UI_SCALE_NUDGE: f32 = 1.05;

pub const UPSELL_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const UPSELL_JEFFE_12_LINE_HEIGHT: f32 = 12.338_999_75;

pub const UPSELL_NEWS_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

pub const UPSELL_CONTINUE_BORDER: BorderRect = BorderRect::all(5.0);

pub const UPSELL_NOT_NOW_BORDER: BorderRect = BorderRect::all(5.0);

pub const UPSELL_BACKGROUND_RECT: UpsellUiRect = UpsellUiRect::new(0.0, 0.0, 1_920.0, 1_440.0);

pub const UPSELL_UPGRADE_DIALOG_RECT: UpsellUiRect = UpsellUiRect::new(0.0, 0.0, 1_112.0, 641.0);

pub const UPSELL_NEWS_DIALOG_RECT: UpsellUiRect = UpsellUiRect::new(0.0, 0.0, 1_032.0, 650.0);

pub const UPSELL_SERIALIZED_CLOSE_RECT: UpsellUiRect = UpsellUiRect::new(1_018.0, 14.0, 26.0, 26.0);

pub const UPSELL_SERIALIZED_CANCEL_RECT: UpsellUiRect =
    UpsellUiRect::new(223.0, 555.0, 288.0, 85.0);

pub const UPSELL_SERIALIZED_GET_RECT: UpsellUiRect = UpsellUiRect::new(700.0, 483.0, 334.0, 105.0);

pub const UPSELL_SERIALIZED_NOT_NOW_RECT: UpsellUiRect =
    UpsellUiRect::new(795.0, 596.0, 153.0, 26.0);

pub const UPSELL_SERIALIZED_ADVERTIS_RECT: UpsellUiRect =
    UpsellUiRect::new(300.0, 100.0, 92.0, 18.0);

pub const UPSELL_LEVEL_ONE_CANCEL_RECT: UpsellUiRect = UPSELL_SERIALIZED_CANCEL_RECT;

pub const UPSELL_LEVEL_ONE_GET_RECT: UpsellUiRect = UPSELL_SERIALIZED_GET_RECT;

pub const UPSELL_LEVEL_ONE_NOT_NOW_RECT: UpsellUiRect = UPSELL_SERIALIZED_NOT_NOW_RECT;

pub const UPSELL_LEVEL_ONE_ADVERTIS_RECT: UpsellUiRect = UpsellUiRect::new(690.0, 9.0, 92.0, 18.0);

pub const UPSELL_OTHER_LEVEL_CANCEL_RECT: UpsellUiRect =
    UpsellUiRect::new(55.0, 555.0, 288.0, 85.0);

pub const UPSELL_OTHER_LEVEL_GET_RECT: UpsellUiRect = UpsellUiRect::new(544.0, 520.0, 334.0, 105.0);

pub const UPSELL_OTHER_LEVEL_NOT_NOW_RECT: UpsellUiRect =
    UpsellUiRect::new(883.0, 597.0, 153.0, 26.0);

pub const UPSELL_OTHER_LEVEL_ADVERTIS_RECT: UpsellUiRect =
    UpsellUiRect::new(351.0, 8.0, 92.0, 18.0);

pub const UPSELL_NEWS_CLOSE_RECT: UpsellUiRect = UpsellUiRect::new(987.0, 10.0, 31.0, 31.0);

pub const UPSELL_NEWS_CONTINUE_RECT: UpsellUiRect = UpsellUiRect::new(10.0, 610.0, 132.0, 27.0);

pub const UPSELL_NEWS_HIDDEN_GET_RECT: UpsellUiRect =
    UpsellUiRect::new(-1_000.0, -1_000.0, 334.0, 105.0);

pub const UPSELL_NEWS_HIDDEN_ADVERTIS_RECT: UpsellUiRect =
    UpsellUiRect::new(-1_000.0, -1_000.0, 92.0, 18.0);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UpsellUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UpsellUiRect {
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
pub struct UpsellScaledGroup {
    pub source: UpsellUiRect,
    pub node_left: f32,
    pub node_top: f32,
    pub scale: f32,
    pub painted: UpsellUiRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpsellUiLayout {
    pub viewport: Vec2,
    pub pivot: Vec2,
    pub scale: f32,
    pub background: UpsellScaledGroup,
    pub dialog: UpsellScaledGroup,
}

/// Exact default `cnGraphicOption.GetUiScale` contract:
/// `max(1, Screen.height / 768 * 1.05)`.
#[must_use]
pub fn clean_upsell_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / UPSELL_REFERENCE_HEIGHT * UPSELL_UI_SCALE_NUDGE).max(1.0)
}

#[must_use]
pub fn upsell_ui_layout(viewport: Vec2, ui_scale: f32, mode: UpsellUiMode) -> UpsellUiLayout {
    let width = legacy_screen_extent(viewport.x);
    let height = legacy_screen_extent(viewport.y);
    let viewport = Vec2::new(width as f32, height as f32);
    let pivot = Vec2::new((width / 2) as f32, (height / 2) as f32);
    let scale = valid_ui_scale(ui_scale);
    let dialog_size = mode.dialog_rect();
    let background = UpsellUiRect::new(
        ((width - UPSELL_BACKGROUND_RECT.width as i32) / 2) as f32,
        ((height - UPSELL_BACKGROUND_RECT.height as i32) / 2) as f32,
        UPSELL_BACKGROUND_RECT.width,
        UPSELL_BACKGROUND_RECT.height,
    );
    let dialog = UpsellUiRect::new(
        ((width - dialog_size.width as i32) / 2) as f32,
        ((height - dialog_size.height as i32) / 2) as f32,
        dialog_size.width,
        dialog_size.height,
    );
    UpsellUiLayout {
        viewport,
        pivot,
        scale,
        background: scaled_group(background, pivot, scale),
        dialog: scaled_group(dialog, pivot, scale),
    }
}

pub(super) fn scaled_group(source: UpsellUiRect, pivot: Vec2, scale: f32) -> UpsellScaledGroup {
    let source_center = source.center();
    let painted_center = pivot + (source_center - pivot) * scale;
    UpsellScaledGroup {
        source,
        node_left: painted_center.x - source.width * 0.5,
        node_top: painted_center.y - source.height * 0.5,
        scale,
        painted: UpsellUiRect::new(
            painted_center.x - source.width * scale * 0.5,
            painted_center.y - source.height * scale * 0.5,
            source.width * scale,
            source.height * scale,
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpsellUpgradeGeometry {
    pub dialog: UpsellUiRect,
    pub close: UpsellUiRect,
    pub continue_playing: UpsellUiRect,
    pub get_upgrade: UpsellUiRect,
    pub not_right_now: UpsellUiRect,
    pub advertis: UpsellUiRect,
}

#[must_use]
pub const fn upsell_upgrade_geometry(level: u8) -> Option<UpsellUpgradeGeometry> {
    if level == 0 || level > 4 {
        return None;
    }
    let level_one = level == 1;
    Some(UpsellUpgradeGeometry {
        dialog: UPSELL_UPGRADE_DIALOG_RECT,
        close: UPSELL_SERIALIZED_CLOSE_RECT,
        continue_playing: if level_one {
            UPSELL_LEVEL_ONE_CANCEL_RECT
        } else {
            UPSELL_OTHER_LEVEL_CANCEL_RECT
        },
        get_upgrade: if level_one {
            UPSELL_LEVEL_ONE_GET_RECT
        } else {
            UPSELL_OTHER_LEVEL_GET_RECT
        },
        not_right_now: if level_one {
            UPSELL_LEVEL_ONE_NOT_NOW_RECT
        } else {
            UPSELL_OTHER_LEVEL_NOT_NOW_RECT
        },
        advertis: if level_one {
            UPSELL_LEVEL_ONE_ADVERTIS_RECT
        } else {
            UPSELL_OTHER_LEVEL_ADVERTIS_RECT
        },
    })
}

pub(super) fn sync_upsell_root_and_layout(
    model: Res<UpsellUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    assets: Res<UpsellUiAssets>,
    mut roots: Query<
        &mut Node,
        (
            With<UpsellUiRoot>,
            Without<UpsellUiBackdrop>,
            Without<UpsellUiDialog>,
        ),
    >,
    mut backdrops: Query<
        (&mut Node, &mut UiTransform),
        (
            With<UpsellUiBackdrop>,
            Without<UpsellUiRoot>,
            Without<UpsellUiDialog>,
        ),
    >,
    mut dialogs: Query<
        (&mut Node, &mut UiTransform, &mut ImageNode),
        (
            With<UpsellUiDialog>,
            Without<UpsellUiRoot>,
            Without<UpsellUiBackdrop>,
        ),
    >,
) {
    let mode = model.active_mode.unwrap_or(UpsellUiMode::NewsPayZone);
    for mut root in &mut roots {
        root.display = if model.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = upsell_ui_layout(viewport, model.effective_ui_scale(viewport.y), mode);
    for mut root in &mut roots {
        root.width = px(layout.viewport.x);
        root.height = px(layout.viewport.y);
    }
    for (mut node, mut transform) in &mut backdrops {
        apply_scaled_group(&mut node, &mut transform, layout.background);
    }
    for (mut node, mut transform, mut image) in &mut dialogs {
        apply_scaled_group(&mut node, &mut transform, layout.dialog);
        match mode {
            UpsellUiMode::Upgrade => {
                image.image = model
                    .level
                    .and_then(|level| assets.level_image(level))
                    .unwrap_or_default();
                image.color = Color::WHITE;
            }
            UpsellUiMode::NewsPayZone | UpsellUiMode::NewsFreeZone => {
                image.image = model
                    .current_news_page_path()
                    .map(|path| asset_server.load(path.to_owned()))
                    .unwrap_or_default();
                image.color = Color::srgba(1.0, 1.0, 1.0, model.page_alpha);
            }
        }
        image.image_mode = NodeImageMode::Stretch;
    }
}

pub(super) fn sync_upsell_element_geometry(
    model: Res<UpsellUiModel>,
    mut buttons: Query<(&UpsellUiButton, &mut Node)>,
    mut advertis: Query<&mut Node, (With<UpsellUiAdvertis>, Without<UpsellUiButton>)>,
) {
    for (button, mut node) in &mut buttons {
        if let Some(rect) = button_rect(&model, button.kind) {
            node.display = Display::Flex;
            apply_rect(&mut node, rect);
            let visual = button_visual(&model, button.kind)
                .expect("visible Upsell button must have a visual");
            node.padding = button_padding(visual);
        } else {
            node.display = Display::None;
        }
    }
    for mut node in &mut advertis {
        if let Some(rect) = advertis_rect(&model) {
            node.display = Display::Flex;
            apply_rect(&mut node, rect);
        } else {
            node.display = Display::None;
        }
    }
}

pub(super) fn button_rect(model: &UpsellUiModel, kind: UpsellUiButtonKind) -> Option<UpsellUiRect> {
    if !model.visible {
        return None;
    }
    match model.active_mode? {
        UpsellUiMode::Upgrade => {
            let geometry = upsell_upgrade_geometry(model.level?)?;
            match kind {
                UpsellUiButtonKind::Close => Some(geometry.close),
                UpsellUiButtonKind::Continue => Some(geometry.continue_playing),
                UpsellUiButtonKind::GetUpgrade => Some(geometry.get_upgrade),
                UpsellUiButtonKind::NotRightNow => Some(geometry.not_right_now),
            }
        }
        UpsellUiMode::NewsPayZone | UpsellUiMode::NewsFreeZone => match kind {
            UpsellUiButtonKind::Close => Some(UPSELL_NEWS_CLOSE_RECT),
            UpsellUiButtonKind::Continue => Some(UPSELL_NEWS_CONTINUE_RECT),
            UpsellUiButtonKind::GetUpgrade | UpsellUiButtonKind::NotRightNow => None,
        },
    }
}

pub(super) fn advertis_rect(model: &UpsellUiModel) -> Option<UpsellUiRect> {
    if !model.visible || model.active_mode != Some(UpsellUiMode::Upgrade) {
        return None;
    }
    Some(upsell_upgrade_geometry(model.level?)?.advertis)
}

pub(super) fn apply_scaled_group(node: &mut Node, transform: &mut UiTransform, group: UpsellScaledGroup) {
    node.left = px(group.node_left);
    node.top = px(group.node_top);
    node.width = px(group.source.width);
    node.height = px(group.source.height);
    transform.scale = Vec2::splat(group.scale);
}

pub(super) fn apply_rect(node: &mut Node, rect: UpsellUiRect) {
    node.left = px(rect.x);
    node.top = px(rect.y);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
