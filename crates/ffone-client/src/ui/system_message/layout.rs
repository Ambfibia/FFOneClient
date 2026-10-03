use super::*;

pub const SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const SYSTEM_MESSAGE_JEFFE_12_LINE_HEIGHT: f32 = 12.338_999_75;

pub const SYSTEM_MESSAGE_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const SYSTEM_MESSAGE_SCALE_REFERENCE_HEIGHT: f32 = 768.0;

pub const SYSTEM_MESSAGE_SCALE_FACTOR: f32 = 1.05;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemMessageUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl SystemMessageUiRect {
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

/// Effective serialized clean-component geometry.
///
/// `SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT` intentionally differs from
/// the managed constructor default (`310,124,250,25`): Unity serialization
/// overrides it with the values below on path ID 1468.
pub const SYSTEM_MESSAGE_WINDOW_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(0.0, 0.0, 600.0, 164.0);

pub const SYSTEM_MESSAGE_CONTENT_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(140.0, 20.0, 420.0, 104.0);

pub const SYSTEM_MESSAGE_OK_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(454.0, 124.0, 110.0, 25.0);

pub const SYSTEM_MESSAGE_CANCEL_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(44.0, 124.0, 150.0, 25.0);

pub const SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(320.0, 124.0, 245.0, 25.0);

pub const SYSTEM_MESSAGE_ICON_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(60.0, 27.0, 62.0, 62.0);

pub const SYSTEM_MESSAGE_ICON_QUANTITY_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(65.0, 32.0, 62.0, 62.0);

pub const SYSTEM_MESSAGE_COMPARISON_ICON_RECTS: [SystemMessageUiRect; 2] = [
    SystemMessageUiRect::new(371.0, 27.0, 62.0, 62.0),
    SystemMessageUiRect::new(444.0, 27.0, 62.0, 62.0),
];

pub const SYSTEM_MESSAGE_COMPARISON_BADGE_RECTS: [SystemMessageUiRect; 2] = [
    SystemMessageUiRect::new(405.0, 61.0, 26.0, 26.0),
    SystemMessageUiRect::new(478.0, 61.0, 26.0, 26.0),
];

pub const SYSTEM_MESSAGE_DELETE_MISSION_CONTENT_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(140.0, 20.0, 400.0, 104.0);

pub const SYSTEM_MESSAGE_COMBINATION_FAILURE_CONTENT_RECT: SystemMessageUiRect =
    SystemMessageUiRect::new(140.0, 20.0, 220.0, 104.0);

pub const SYSTEM_MESSAGE_DIALOG_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(40.0, 16.0),
    max_inset: Vec2::new(40.0, 40.0),
};

pub const SYSTEM_MESSAGE_BUTTON_BORDER: BorderRect = BorderRect::all(5.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemMessageButtonLayout {
    pub primary: SystemMessageButtonSpec,
    pub secondary: Option<SystemMessageButtonSpec>,
}

impl SystemMessageButtonLayout {
    #[must_use]
    pub const fn count(self) -> usize {
        if self.secondary.is_some() { 2 } else { 1 }
    }

    #[must_use]
    pub const fn contains(self, choice: SystemMessageChoice) -> bool {
        match choice {
            SystemMessageChoice::Primary => true,
            SystemMessageChoice::Secondary => self.secondary.is_some(),
        }
    }
}

/// Exact clean `DrawAll` button labels, styles, counts, and serialized rects.
#[must_use]
pub const fn system_message_button_layout(
    button_type: SystemMessageButtonType,
) -> SystemMessageButtonLayout {
    use SystemMessageButtonType as Type;
    use SystemMessageButtonVisual as Visual;
    use SystemMessageChoice as Choice;

    match button_type {
        Type::None | Type::Ok | Type::CombinationFailure => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "OKAY",
                Visual::Standard,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: None,
        },
        Type::OkCancel => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "OKAY",
                Visual::Standard,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Cancel,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::YesNo => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "YES",
                Visual::Standard,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "NO",
                Visual::Standard,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::ExitCharacterCreation => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "EXIT CHARACTER CREATION",
                Visual::Destructive,
                SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Cancel,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::DeleteMission => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "DELETE MISSION",
                Visual::Destructive,
                SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Standard,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::TutorialSkip => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "SKIP",
                Visual::Standard,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CONTINUE",
                Visual::Standard,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::LeaveGroup => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "LEAVE",
                Visual::Destructive,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Standard,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::DeleteItem => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "DELETE",
                Visual::Destructive,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Cancel,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::CancelWarp => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "WARP",
                Visual::Standard,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Cancel,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
        Type::CombinationConfirm => SystemMessageButtonLayout {
            primary: button(
                Choice::Primary,
                "CONTINUE",
                Visual::Standard,
                SYSTEM_MESSAGE_OK_RECT,
            ),
            secondary: Some(button(
                Choice::Secondary,
                "CANCEL",
                Visual::Standard,
                SYSTEM_MESSAGE_CANCEL_RECT,
            )),
        },
    }
}

/// Clean `FFGUIUtility` center-pivot scale used by this UI pass.
#[must_use]
pub fn clean_system_message_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / SYSTEM_MESSAGE_SCALE_REFERENCE_HEIGHT * SYSTEM_MESSAGE_SCALE_FACTOR).max(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemMessageLayerLayout {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

/// Returns the unscaled Bevy node geometry plus center-origin transform scale
/// that reproduces `FFGUIUtility.ScaleAroundPivot(ScreenPivot.Center)`.
#[must_use]
pub fn system_message_layer_layout(
    viewport: Vec2,
    stack_index: usize,
    ui_scale: f32,
) -> SystemMessageLayerLayout {
    let scale = valid_ui_scale(ui_scale);
    let offset = SYSTEM_MESSAGE_STACK_OFFSET * (stack_index + 1) as f32 * scale;
    SystemMessageLayerLayout {
        left: viewport.x * 0.5 - SYSTEM_MESSAGE_WINDOW_RECT.width * 0.5 + offset,
        top: viewport.y * 0.5 - SYSTEM_MESSAGE_WINDOW_RECT.height * 0.5 + offset,
        width: SYSTEM_MESSAGE_WINDOW_RECT.width,
        height: SYSTEM_MESSAGE_WINDOW_RECT.height,
        scale,
    }
}

pub(super) fn layout_text_node(
    style: SystemMessageTextStyle,
    width: Option<f32>,
    height: Option<f32>,
    use_guilayout_margin: bool,
) -> Node {
    let spec = style.spec();
    let margin = if use_guilayout_margin {
        UiRect {
            left: px(spec.margin[0]),
            right: px(spec.margin[1]),
            // `GUILayoutGroup` suppresses the first child's top margin. The
            // reached branches all place these labels first in their group.
            bottom: px(spec.margin[3]),
            ..default()
        }
    } else {
        UiRect::ZERO
    };
    Node {
        width: width.map_or_else(|| percent(100), px),
        height: height.map_or(Val::Auto, px),
        flex_shrink: 0.0,
        margin,
        padding: style_padding(spec.padding),
        ..default()
    }
}

pub const fn system_message_button_line_height(visual: SystemMessageButtonVisual) -> f32 {
    system_message_button_text_style(visual).spec().line_height
}

pub(super) fn update_system_message_layout(
    model: Res<SystemMessageUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut previous_layout: Local<Option<(u32, u32, u32)>>,
    mut layers: Query<(Ref<SystemMessageLayer>, &mut Node, &mut UiTransform)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let scale = model.effective_ui_scale(viewport.y);
    let layout_key = (viewport.x.to_bits(), viewport.y.to_bits(), scale.to_bits());
    let viewport_changed = previous_layout.as_ref() != Some(&layout_key);
    *previous_layout = Some(layout_key);
    for (layer, mut node, mut transform) in &mut layers {
        if !viewport_changed && !layer.is_changed() {
            continue;
        }
        let layout = system_message_layer_layout(viewport, layer.0, scale);
        node.left = px(layout.left);
        node.top = px(layout.top);
        node.width = px(layout.width);
        node.height = px(layout.height);
        transform.scale = Vec2::splat(layout.scale);
    }
}
