use super::*;

pub const CHARACTER_CREATION_REFERENCE_WIDTH: f32 = 1920.0;

pub const CHARACTER_CREATION_REFERENCE_HEIGHT: f32 = 1440.0;

pub const CHARACTER_CREATION_APPEARANCE_WIDTH: f32 = 1020.0;

pub const CHARACTER_CREATION_APPEARANCE_HEIGHT: f32 = 638.0;

pub const CHARACTER_CREATION_NAME_WIDTH: f32 = 672.0;

pub const CHARACTER_CREATION_NAME_HEIGHT: f32 = 365.0;

pub const CHARACTER_CREATION_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const CHARACTER_CREATION_JEFFE_16_LINE_HEIGHT: f32 = 16.451_999_66;

pub const CHARACTER_CREATION_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const CHARACTER_CREATION_CHALET_REGULAR_LINE_HEIGHT: f32 = 14.083_999_63;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCreationRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LegacyCreationRect {
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
pub struct CharacterCreationLayout {
    pub viewport: Vec2,
    pub scale: f32,
    pub background: LegacyCreationRect,
    pub appearance: LegacyCreationRect,
    pub name: LegacyCreationRect,
    pub fullscreen: LegacyCreationRect,
}

impl CharacterCreationLayout {
    pub fn for_viewport(viewport: Vec2, scale: f32) -> Self {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let center = viewport * 0.5;
        let centered = |width: f32, height: f32| {
            LegacyCreationRect::new(
                (viewport.x - width) * 0.5,
                (viewport.y - height) * 0.5,
                width,
                height,
            )
            .scale_about(center, scale)
        };
        Self {
            viewport,
            scale,
            background: centered(
                CHARACTER_CREATION_REFERENCE_WIDTH,
                CHARACTER_CREATION_REFERENCE_HEIGHT,
            ),
            appearance: centered(
                CHARACTER_CREATION_APPEARANCE_WIDTH,
                CHARACTER_CREATION_APPEARANCE_HEIGHT,
            ),
            name: centered(
                CHARACTER_CREATION_NAME_WIDTH,
                CHARACTER_CREATION_NAME_HEIGHT,
            ),
            fullscreen: LegacyCreationRect::new(viewport.x - 45.0, viewport.y - 35.0, 35.0, 31.0)
                .scale_about(viewport, scale),
        }
    }
}

#[derive(Component)]
pub(super) struct AppearanceHeightText;

/// Exact `m_Border` values from `FusionFallCharCreation` pathId 1382 in
/// `sharedassets0.assets`. `None` means the source style has a zero border or
/// the texture was drawn directly through `GUI.DrawTexture`.
pub(super) fn source_style_border(path: &'static str) -> Option<BorderRect> {
    match path {
        CCBG => Some(BorderRect {
            min_inset: Vec2::new(10.0, 0.0),
            max_inset: Vec2::new(10.0, 0.0),
        }),
        CC_IN_12_BG => Some(BorderRect {
            min_inset: Vec2::new(5.0, 5.0),
            max_inset: Vec2::new(40.0, 40.0),
        }),
        CC_NAME_TAB_1 | CC_NAME_TAB_2 => Some(BorderRect {
            min_inset: Vec2::new(5.0, 21.0),
            max_inset: Vec2::new(5.0, 10.0),
        }),
        CC_NAME_BUTTON | CC_NAME_BUTTON_OVER => Some(BorderRect {
            min_inset: Vec2::new(35.0, 20.0),
            max_inset: Vec2::new(35.0, 0.0),
        }),
        CC_NAME_BG => Some(BorderRect {
            min_inset: Vec2::new(5.0, 0.0),
            max_inset: Vec2::new(5.0, 0.0),
        }),
        CHARACTER_CREATION_TEXT_FIELD_PATH => Some(BorderRect::all(4.0)),
        CC_CHECK_NORMAL | CC_CHECK_OVER | CC_CHECKED | CC_CHECKED_OVER | CC_COLOR_SELECTED
        | CC_FULLSCREEN | CC_WINDOWED | CC_FULLSCREEN_OVER | CC_WINDOWED_OVER => None,
        CC_CHARACTER_DISPLAY
        | CC_RIGHT_BG
        | CC_IN_3_BG
        | CC_BODY_DISPLAY
        | CC_BODY_LEFT
        | CC_BODY_LEFT_OVER
        | CC_BODY_RIGHT
        | CC_BODY_RIGHT_OVER
        | CC_ROTATE_LEFT
        | CC_ROTATE_LEFT_OVER
        | CC_ROTATE_RIGHT
        | CC_ROTATE_RIGHT_OVER
        | CC_ZOOM_IN
        | CC_ZOOM_IN_OVER
        | CC_ZOOM_OUT
        | CC_ZOOM_OUT_OVER
        | CC_CLOTHES_BG
        | CC_COLOR_INSIDE
        | CC_COLOR_OUTLINE
        | CC_NAME_DISPLAY
        | CC_NAME_SHADE
        | CC_SCROLL_BG
        | CC_SCROLL_UP
        | CC_SCROLL_UP_OVER
        | CC_SCROLL_DOWN
        | CC_SCROLL_DOWN_OVER
        | CC_BLUE_BUTTON
        | CC_BLUE_BUTTON_OVER
        | CC_RED_BUTTON
        | CC_RED_BUTTON_OVER
        | "ui/en/character/creation/clothes/CCClothesLeftBottomButtonNormal.png"
        | "ui/en/character/creation/clothes/CCClothesLeftBottomButtonOver.png"
        | "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png"
        | "ui/en/character/creation/clothes/CCClothesLeftButtonOver.png"
        | "ui/en/character/creation/clothes/CCClothesLeftTopButtonNormal.png"
        | "ui/en/character/creation/clothes/CCClothesLeftTopOver.png"
        | "ui/en/character/creation/clothes/CCClothesRightBottomButtonNormal.png"
        | "ui/en/character/creation/clothes/CCClothesRightBottomOver.png"
        | "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png"
        | "ui/en/character/creation/clothes/CCClothesRightButtonOver.png"
        | "ui/en/character/creation/clothes/CCClothesRightTopButtonNormal.png"
        | "ui/en/character/creation/clothes/CCClothesRightTopOver.png" => {
            Some(BorderRect::all(5.0))
        }
        CHARACTER_CREATION_BACKGROUND_PATH => None,
        _ => None,
    }
}

pub(super) fn character_creation_text_layout(style: CharacterCreationTextStyle) -> TextLayout {
    let justify = match style.spec().anchor {
        CharacterCreationTextAnchor::MiddleLeft => Justify::Left,
        CharacterCreationTextAnchor::MiddleCenter => Justify::Center,
    };
    TextLayout::new(
        justify,
        if style.spec().word_wrap {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        },
    )
}

pub(super) fn apply_scaled_group(
    node: &mut Node,
    transform: &mut UiTransform,
    visual: LegacyCreationRect,
    unscaled_size: Vec2,
    scale: f32,
) {
    node.left = px(visual.x + (visual.width - unscaled_size.x) * 0.5);
    node.top = px(visual.y + (visual.height - unscaled_size.y) * 0.5);
    node.width = px(unscaled_size.x);
    node.height = px(unscaled_size.y);
    transform.scale = Vec2::splat(scale);
}

pub(super) fn update_character_creation_layout(
    model: Res<CharacterCreationUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut nodes: ParamSet<(
        Query<(&mut Node, &mut Visibility), With<NativeCharacterCreationRoot>>,
        Query<(&mut Node, &mut UiTransform), With<CreationBackground>>,
        Query<(&mut Node, &mut UiTransform), With<AppearanceRoot>>,
        Query<(&mut Node, &mut UiTransform), With<NameRoot>>,
        Query<(&mut Node, &mut UiTransform), With<CreationFullscreen>>,
        Query<(&mut Node, &mut UiTransform), With<CreationPreviewControlsRoot>>,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = CharacterCreationLayout::for_viewport(viewport, model.ui_scale);
    if let Ok((mut node, mut visibility)) = nodes.p0().single_mut() {
        node.width = px(viewport.x);
        node.height = px(viewport.y);
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
            layout.background,
            Vec2::new(
                CHARACTER_CREATION_REFERENCE_WIDTH,
                CHARACTER_CREATION_REFERENCE_HEIGHT,
            ),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p2().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.appearance,
            Vec2::new(
                CHARACTER_CREATION_APPEARANCE_WIDTH,
                CHARACTER_CREATION_APPEARANCE_HEIGHT,
            ),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p3().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.name,
            Vec2::new(
                CHARACTER_CREATION_NAME_WIDTH,
                CHARACTER_CREATION_NAME_HEIGHT,
            ),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p4().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.fullscreen,
            Vec2::new(35.0, 31.0),
            layout.scale,
        );
    }
    if let Ok((mut node, mut transform)) = nodes.p5().single_mut() {
        apply_scaled_group(
            &mut node,
            &mut transform,
            layout.appearance,
            Vec2::new(
                CHARACTER_CREATION_APPEARANCE_WIDTH,
                CHARACTER_CREATION_APPEARANCE_HEIGHT,
            ),
            layout.scale,
        );
    }
}
