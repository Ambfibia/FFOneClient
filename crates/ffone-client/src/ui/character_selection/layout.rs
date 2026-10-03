use super::*;

pub const CHARACTER_SELECTION_REFERENCE_WIDTH: f32 = 1920.0;

pub const CHARACTER_SELECTION_REFERENCE_HEIGHT: f32 = 1440.0;

pub const CHARACTER_SELECTION_BASELINE_WIDTH: f32 = 1264.0;

pub const CHARACTER_SELECTION_BASELINE_HEIGHT: f32 = 681.0;

pub const CHARACTER_SELECTION_PREVIEW_WIDTH: f32 = 550.0;

pub const CHARACTER_SELECTION_PREVIEW_HEIGHT: f32 = 583.0;

pub const CHARACTER_SELECTION_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const CHARACTER_SELECTION_JEFFE_12_LINE_HEIGHT: f32 = 12.338_999_75;

pub const CHARACTER_SELECTION_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub const CHARACTER_SELECTION_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const CHARACTER_SELECTION_CHALET_REGULAR_LINE_HEIGHT: f32 = 14.083_999_63;

pub(super) const SLOT_BUTTON_RECTS: [LegacySelectionRect; 4] = [
    LegacySelectionRect::new(43.0, 43.0, 413.0, 67.0),
    LegacySelectionRect::new(43.0, 141.0, 413.0, 67.0),
    LegacySelectionRect::new(43.0, 237.0, 413.0, 67.0),
    LegacySelectionRect::new(43.0, 334.0, 413.0, 67.0),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySelectionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LegacySelectionRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn center(self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn scale_about(self, pivot: Vec2, scale: f32) -> Self {
        let center = pivot + (self.center() - pivot) * scale;
        Self::new(
            center.x - self.width * scale * 0.5,
            center.y - self.height * scale * 0.5,
            self.width * scale,
            self.height * scale,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterSelectionLayout {
    pub viewport: Vec2,
    pub scale: f32,
    pub chrome: LegacySelectionRect,
    pub avatar_group: LegacySelectionRect,
    pub selection_group: LegacySelectionRect,
    pub quit: LegacySelectionRect,
    pub fullscreen: LegacySelectionRect,
}

impl CharacterSelectionLayout {
    pub fn for_viewport(viewport: Vec2, scale: f32) -> Self {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let center = viewport * 0.5;
        let chrome = LegacySelectionRect::new(
            (viewport.x - CHARACTER_SELECTION_REFERENCE_WIDTH) * 0.5,
            (viewport.y - CHARACTER_SELECTION_REFERENCE_HEIGHT) * 0.5,
            CHARACTER_SELECTION_REFERENCE_WIDTH,
            CHARACTER_SELECTION_REFERENCE_HEIGHT,
        )
        .scale_about(center, scale);
        let avatar_group = LegacySelectionRect::new(
            viewport.x * 0.5 - 510.0,
            viewport.y * 0.5 - 315.0,
            600.0,
            600.0,
        )
        .scale_about(center, scale);
        let selection_group = LegacySelectionRect::new(
            viewport.x * 0.5 + 30.0,
            viewport.y * 0.5 - 288.0,
            478.0,
            600.0,
        )
        .scale_about(center, scale);
        let quit = LegacySelectionRect::new(20.0, viewport.y - 38.0, 90.0, 23.0)
            .scale_about(Vec2::new(0.0, viewport.y), scale);
        let fullscreen = LegacySelectionRect::new(viewport.x - 45.0, viewport.y - 35.0, 35.0, 31.0)
            .scale_about(viewport, scale);
        Self {
            viewport,
            scale,
            chrome,
            avatar_group,
            selection_group,
            quit,
            fullscreen,
        }
    }

    pub fn background_frame(self, index: usize, elapsed_seconds: f32) -> LegacySelectionRect {
        let strip_width = CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH
            * CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT as f32;
        let distance = (elapsed_seconds.max(0.0) * CHARACTER_SELECTION_BACKGROUND_SPEED)
            .rem_euclid(strip_width);
        // UpdateRotateBackground wraps each 542px frame only after that
        // frame's right edge leaves the viewport. Wrapping the whole strip at
        // once creates the moving black rectangles seen in production.
        let unwrapped_x = index as f32 * CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH - distance;
        let frame_x = if unwrapped_x + CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH <= 0.0 {
            unwrapped_x + strip_width
        } else {
            unwrapped_x
        };
        let logical = LegacySelectionRect::new(
            frame_x,
            (self.viewport.y - CHARACTER_SELECTION_REFERENCE_HEIGHT) * 0.5 + 418.0,
            CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH,
            CHARACTER_SELECTION_BACKGROUND_FRAME_HEIGHT,
        );
        logical.scale_about(self.viewport * 0.5, self.scale)
    }
}

pub(super) fn character_selection_text_layout(style: CharacterSelectionTextStyle) -> TextLayout {
    let justify = match style.spec().anchor {
        CharacterSelectionTextAnchor::MiddleLeft => Justify::Left,
        CharacterSelectionTextAnchor::MiddleCenter => Justify::Center,
    };
    TextLayout::new(justify, LineBreak::NoWrap)
}

pub(super) fn apply_scaled_group(
    node: &mut Node,
    transform: &mut UiTransform,
    visual: LegacySelectionRect,
    unscaled_size: Vec2,
    scale: f32,
) {
    node.left = px(visual.x + (visual.width - unscaled_size.x) * 0.5);
    node.top = px(visual.y + (visual.height - unscaled_size.y) * 0.5);
    node.width = px(unscaled_size.x);
    node.height = px(unscaled_size.y);
    transform.scale = Vec2::splat(scale);
}

pub(super) fn update_character_selection_layout(
    time: Res<Time>,
    model: Res<CharacterSelectionUiModel>,
    mut background_clock: ResMut<CharacterSelectionBackgroundClock>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut nodes: ParamSet<(
        Query<(&mut Node, &mut Visibility), With<NativeCharacterSelectionRoot>>,
        Query<(&mut Node, &mut UiTransform), With<SelectionChrome>>,
        Query<(&mut Node, &mut UiTransform), With<SelectionAvatarGroup>>,
        Query<
            (
                &mut Node,
                &mut UiTransform,
                Option<&mut Visibility>,
                Option<&SelectionPortraitOverlayRoot>,
            ),
            Or<(With<SelectionPanel>, With<SelectionPortraitOverlayRoot>)>,
        >,
        Query<(&mut Node, &mut UiTransform), With<SelectionQuit>>,
        Query<(&mut Node, &mut UiTransform), With<SelectionFullscreen>>,
        Query<(&SelectionBackgroundFrame, &mut Node, &mut UiTransform)>,
        Query<(&mut Node, &mut UiTransform), With<SelectionDeletePanel>>,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = CharacterSelectionLayout::for_viewport(viewport, model.ui_scale);
    let background_elapsed = background_clock.update(model.visible, time.delta_secs());
    if let Ok((mut root, mut visibility)) = nodes.p0().single_mut() {
        root.width = px(viewport.x);
        root.height = px(viewport.y);
        *visibility = if model.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok((mut node, mut transform)) = nodes.p1().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.chrome,
            Vec2::new(
                CHARACTER_SELECTION_REFERENCE_WIDTH,
                CHARACTER_SELECTION_REFERENCE_HEIGHT,
            ),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p2().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.avatar_group,
            Vec2::splat(600.0),
            layout.scale,
        );
    }
    for (mut node, mut transform, visibility, portrait_overlay) in &mut nodes.p3() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.selection_group,
            Vec2::new(478.0, 600.0),
            layout.scale,
        );
        if portrait_overlay.is_some() {
            *visibility.expect("the portrait overlay root owns visibility") = if model.visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    if let Ok((mut node, mut transform)) = nodes.p4().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.quit,
            Vec2::new(90.0, 23.0),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p5().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.fullscreen,
            Vec2::new(35.0, 31.0),
            layout.scale,
        );
    }
    for (frame, mut node, mut transform) in &mut nodes.p6() {
        let rect = layout.background_frame(frame.0, background_elapsed);
        apply_scaled_group(
            &mut node,
            &mut transform,
            rect,
            Vec2::new(
                CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH,
                CHARACTER_SELECTION_BACKGROUND_FRAME_HEIGHT,
            ),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p7().single_mut() {
        node.left = px(viewport.x * 0.5 - 263.0);
        node.top = px(viewport.y * 0.5 - 82.0);
        node.width = px(526.0);
        node.height = px(164.0);
        transform.scale = Vec2::splat(layout.scale);
    }
}
