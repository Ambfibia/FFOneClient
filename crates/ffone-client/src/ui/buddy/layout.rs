use super::*;

pub const BUDDY_LIST_ROW_HEIGHT: f32 = 15.5;

pub const BUDDY_JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;

pub const BUDDY_JEFFE_12_LINE_HEIGHT: f32 = 12.338_999_75;

pub const BUDDY_CHALET_SMALL_LINE_HEIGHT: f32 = 12.071_999_55;

pub const BUDDY_WINDOW_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(10.0, 30.0),
    max_inset: Vec2::new(6.0, 40.0),
};

pub const BUDDY_LIST_BACKGROUND_BORDER: BorderRect = BorderRect::all(3.0);

pub const BUDDY_LARGE_LIST_BACKGROUND_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 4.0),
    max_inset: Vec2::new(8.0, 4.0),
};

pub const BUDDY_SELECT_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(20.0, 0.0),
    max_inset: Vec2::new(0.0, 0.0),
};

pub const BUDDY_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 4.0),
    max_inset: Vec2::new(6.0, 4.0),
};

pub const BUDDY_SCROLL_TRACK_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(2.0, 4.0),
    max_inset: Vec2::new(2.0, 4.0),
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuddyUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BuddyUiRect {
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

// Effective clean `CnGuiChat` serialized geometry.
pub const BUDDY_GROUP_RECT: BuddyUiRect = BuddyUiRect::new(0.0, 0.0, 293.0, 150.0);

pub const BUDDY_WINDOW_RECT: BuddyUiRect = BuddyUiRect::new(0.0, 15.0, 293.0, 135.0);

pub const BUDDY_CONTENT_RECT: BuddyUiRect = BuddyUiRect::new(-1.0, 32.0, 290.0, 91.0);

pub const BUDDY_INNER_LIST_RECT: BuddyUiRect = BuddyUiRect::new(4.0, 37.0, 280.0, 81.0);

pub const BUDDY_DELETE_RECT: BuddyUiRect = BuddyUiRect::new(30.0, 127.0, 80.0, 20.0);

pub const BUDDY_WARP_RECT: BuddyUiRect = BuddyUiRect::new(115.0, 127.0, 80.0, 20.0);

pub const BUDDY_ADD_RECT: BuddyUiRect = BuddyUiRect::new(200.0, 127.0, 80.0, 20.0);

pub const BUDDY_FREECHAT_RECT: BuddyUiRect = BuddyUiRect::new(0.0, 0.0, 15.0, 15.0);

// `GUI.BeginScrollView(BuddyContentRect inset by five, content width -16)`
// leaves exactly the rightmost sixteen pixels to the active menu-skin
// vertical scrollbar. Its up/down styles are fixed-height 12; the remaining
// 57 pixels are the sliced track.
pub const BUDDY_SCROLL_UP_RECT: BuddyUiRect = BuddyUiRect::new(264.0, 0.0, 16.0, 12.0);

pub const BUDDY_SCROLL_TRACK_RECT: BuddyUiRect = BuddyUiRect::new(264.0, 12.0, 16.0, 57.0);

pub const BUDDY_SCROLL_DOWN_RECT: BuddyUiRect = BuddyUiRect::new(264.0, 69.0, 16.0, 12.0);

pub const BUDDY_SCROLL_THUMB_WIDTH: f32 = 15.0;

pub const BUDDY_SCROLL_THUMB_MIN_HEIGHT: f32 = 15.0;

pub const BUDDY_ADD_WINDOW_RECT: BuddyUiRect = BuddyUiRect::new(0.0, 0.0, 526.0, 164.0);

pub const BUDDY_ADD_TITLE_RECT: BuddyUiRect = BuddyUiRect::new(176.0, 15.0, 173.0, 17.0);

pub const BUDDY_ADD_INSTRUCTION_RECT: BuddyUiRect = BuddyUiRect::new(118.0, 36.0, 288.0, 17.0);

pub const BUDDY_ADD_TEXT_FIELD_RECT: BuddyUiRect = BuddyUiRect::new(75.0, 73.0, 378.0, 24.0);

pub const BUDDY_ADD_CANCEL_RECT: BuddyUiRect = BuddyUiRect::new(40.0, 125.0, 152.0, 27.0);

pub const BUDDY_ADD_SUBMIT_RECT: BuddyUiRect = BuddyUiRect::new(335.0, 125.0, 152.0, 27.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuddyScrollbarLayout {
    pub visible: bool,
    pub track: BuddyUiRect,
    pub up: BuddyUiRect,
    pub down: BuddyUiRect,
    pub thumb: BuddyUiRect,
}

#[must_use]
pub fn buddy_scrollbar_layout(visible_rows: usize, scroll_y: f32) -> BuddyScrollbarLayout {
    let content_height = visible_rows as f32 * BUDDY_LIST_ROW_STEP;
    let maximum_scroll = (content_height - BUDDY_INNER_LIST_RECT.height).max(0.0);
    let visible = maximum_scroll > 0.0;
    let thumb_height = if visible {
        (BUDDY_INNER_LIST_RECT.height / content_height * BUDDY_SCROLL_TRACK_RECT.height).clamp(
            BUDDY_SCROLL_THUMB_MIN_HEIGHT,
            BUDDY_SCROLL_TRACK_RECT.height,
        )
    } else {
        BUDDY_SCROLL_TRACK_RECT.height
    };
    let fraction = if visible {
        scroll_y.clamp(0.0, maximum_scroll) / maximum_scroll
    } else {
        0.0
    };
    BuddyScrollbarLayout {
        visible,
        track: BUDDY_SCROLL_TRACK_RECT,
        up: BUDDY_SCROLL_UP_RECT,
        down: BUDDY_SCROLL_DOWN_RECT,
        thumb: BuddyUiRect::new(
            BUDDY_SCROLL_TRACK_RECT.x,
            BUDDY_SCROLL_TRACK_RECT.y + fraction * (BUDDY_SCROLL_TRACK_RECT.height - thumb_height),
            BUDDY_SCROLL_THUMB_WIDTH,
            thumb_height,
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuddyInviteDisposition {
    Queued,
    IgnoredBlocked,
    AutoDeclined(BuddyUiAction),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuddyPanelLayout {
    /// Bevy node position adjusted for center-origin `UiTransform` scaling.
    pub node_left: f32,
    pub node_top: f32,
    /// Actual painted bounds after the transform.
    pub painted_left: f32,
    pub painted_top: f32,
    pub painted_width: f32,
    pub painted_height: f32,
    pub scale: f32,
}

#[must_use]
pub fn buddy_panel_layout(
    viewport: Vec2,
    style: BuddyChatWindowStyle,
    quick_slot_active: bool,
    ui_scale: f32,
) -> BuddyPanelLayout {
    let scale = valid_ui_scale(ui_scale);
    // Only `DoSmallWindowChat` applies the serialized quick-slot trim. The
    // reached `DoLargeWindowChat` branch never subtracts these forty pixels.
    let source_height = if style == BuddyChatWindowStyle::Small && quick_slot_active {
        BUDDY_GROUP_RECT.height - 40.0
    } else {
        BUDDY_GROUP_RECT.height
    };
    let painted_left = style.legacy_group_x() * scale;
    let painted_top = viewport.y - source_height * scale;
    BuddyPanelLayout {
        node_left: painted_left + BUDDY_GROUP_RECT.width * (scale - 1.0) * 0.5,
        node_top: painted_top + source_height * (scale - 1.0) * 0.5,
        painted_left,
        painted_top,
        painted_width: BUDDY_GROUP_RECT.width * scale,
        painted_height: source_height * scale,
        scale,
    }
}

#[must_use]
pub fn buddy_panel_layout_for_chat_width(
    viewport: Vec2,
    style: BuddyChatWindowStyle,
    quick_slot_active: bool,
    ui_scale: f32,
    large_chat_width: f32,
) -> BuddyPanelLayout {
    let mut layout = buddy_panel_layout(viewport, style, quick_slot_active, ui_scale);
    if style == BuddyChatWindowStyle::Large {
        let width_delta = (large_chat_width.clamp(325.0, 550.0) - 440.0) * layout.scale;
        layout.node_left += width_delta;
        layout.painted_left += width_delta;
    }
    layout
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuddyAddDialogLayout {
    pub left: f32,
    pub top: f32,
    pub scale: f32,
}

#[must_use]
pub fn buddy_add_dialog_layout(viewport: Vec2, ui_scale: f32) -> BuddyAddDialogLayout {
    BuddyAddDialogLayout {
        left: viewport.x * 0.5 - BUDDY_ADD_WINDOW_RECT.width * 0.5,
        top: viewport.y * 0.5 - BUDDY_ADD_WINDOW_RECT.height * 0.5,
        scale: valid_ui_scale(ui_scale),
    }
}

pub(crate) fn buddy_text_layout(style: BuddyTextStyle) -> TextLayout {
    let spec = style.spec();
    TextLayout::new(
        match spec.anchor {
            BuddyTextAnchor::UpperLeft | BuddyTextAnchor::MiddleLeft => Justify::Left,
            BuddyTextAnchor::MiddleCenter => Justify::Center,
        },
        if spec.word_wrap {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        },
    )
}

pub(super) fn buddy_rect_contains(rect: BuddyUiRect, point: Vec2) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

pub(super) fn bind_buddy_layout(
    model: Res<BuddyUiModel>,
    assets: Res<BuddyUiAssets>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut previous_layout: Local<Option<(u32, u32, BuddyChatWindowStyle, bool, u32, u32)>>,
    mut panels: Query<(&mut Node, &mut UiTransform), With<BuddyPanel>>,
    mut dialogs: Query<(&mut Node, &mut UiTransform), (With<BuddyAddDialog>, Without<BuddyPanel>)>,
    mut list_backgrounds: Query<&mut ImageNode, With<BuddyListBackground>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout_key = (
        viewport.x.to_bits(),
        viewport.y.to_bits(),
        model.chat_window_style,
        model.quick_slot_active,
        model.effective_ui_scale().to_bits(),
        model.large_chat_width.to_bits(),
    );
    if previous_layout.as_ref() == Some(&layout_key) {
        return;
    }
    *previous_layout = Some(layout_key);
    let panel_layout = buddy_panel_layout_for_chat_width(
        viewport,
        model.chat_window_style,
        model.quick_slot_active,
        model.effective_ui_scale(),
        model.large_chat_width,
    );
    for (mut node, mut transform) in &mut panels {
        node.left = px(panel_layout.node_left);
        node.top = px(panel_layout.node_top);
        node.height = px(
            if model.chat_window_style == BuddyChatWindowStyle::Small && model.quick_slot_active {
                BUDDY_GROUP_RECT.height - 40.0
            } else {
                BUDDY_GROUP_RECT.height
            },
        );
        transform.scale = Vec2::splat(panel_layout.scale);
    }
    let dialog_layout = buddy_add_dialog_layout(viewport, model.effective_ui_scale());
    for (mut node, mut transform) in &mut dialogs {
        node.left = px(dialog_layout.left);
        node.top = px(dialog_layout.top);
        transform.scale = Vec2::splat(dialog_layout.scale);
    }
    for mut image in &mut list_backgrounds {
        *image = match model.chat_window_style {
            BuddyChatWindowStyle::Small => {
                sliced_image(assets.list_background.clone(), BUDDY_LIST_BACKGROUND_BORDER)
            }
            BuddyChatWindowStyle::Large => sliced_image(
                assets.large_list_background.clone(),
                BUDDY_LARGE_LIST_BACKGROUND_BORDER,
            ),
        };
    }
}
