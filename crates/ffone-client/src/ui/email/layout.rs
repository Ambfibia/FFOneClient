use super::*;

pub const EMAIL_UI_JEFFE_12_LINE_HEIGHT: f32 = 13.560_000_42;

pub const EMAIL_UI_JEFFE_14_LINE_HEIGHT: f32 = 11.300_000_19;

pub const EMAIL_UI_JEFFE_16_LINE_HEIGHT: f32 = 13.560_000_42;

pub const EMAIL_UI_JEFFE_06_LINE_HEIGHT: f32 = 6.780_000_21;

pub const EMAIL_UI_CHALET_SMALL_LINE_HEIGHT: f32 = 13.560_000_42;

pub const EMAIL_UI_REFERENCE_WIDTH: i32 = 1_020;

pub const EMAIL_UI_REFERENCE_HEIGHT: i32 = 638;

pub const EMAIL_UI_BACKPLATE_WIDTH: i32 = 1_036;

pub const EMAIL_UI_BACKPLATE_HEIGHT: i32 = 653;

pub const EMAIL_UI_BACKGROUND_WIDTH: f32 = 1_920.0;

pub const EMAIL_UI_BACKGROUND_HEIGHT: f32 = 1_440.0;

pub const EMAIL_UI_REFERENCE_SCALE_HEIGHT: f32 = 768.0;

pub const EMAIL_UI_SCALE_NUDGE: f32 = 1.05;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EmailUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl EmailUiRect {
    #[must_use]
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn translated(self, left: f32, top: f32) -> Self {
        Self::new(self.left + left, self.top + top, self.width, self.height)
    }

    #[must_use]
    pub const fn center(self) -> Vec2 {
        Vec2::new(self.left + self.width * 0.5, self.top + self.height * 0.5)
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.left),
            top: px(self.top),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

pub const EMAIL_UI_LIST_WINDOW_RECT: EmailUiRect = EmailUiRect::new(0.0, 0.0, 567.0, 634.0);

pub const EMAIL_UI_LIST_INNER_RECT: EmailUiRect = EmailUiRect::new(6.0, 3.0, 561.0, 631.0);

pub const EMAIL_UI_LIST_BACK_RECT: EmailUiRect = EmailUiRect::new(0.0, 15.0, 561.0, 229.0);

pub const EMAIL_UI_DATA_BACK_RECT: EmailUiRect = EmailUiRect::new(0.0, 248.0, 560.0, 382.0);

pub const EMAIL_UI_SEND_MAIL_RECT: EmailUiRect = EmailUiRect::new(390.0, 0.0, 160.0, 25.0);

pub const EMAIL_UI_GUIDE_TAB_RECT: EmailUiRect = EmailUiRect::new(0.0, 0.0, 181.0, 25.0);

pub const EMAIL_UI_GUIDE_TAB_HIT_RECT: EmailUiRect = EmailUiRect::new(13.0, 5.0, 130.0, 15.0);

pub const EMAIL_UI_PLAYER_TAB_RECT: EmailUiRect = EmailUiRect::new(146.0, 0.0, 209.0, 29.0);

pub const EMAIL_UI_PLAYER_TAB_HIT_RECT: EmailUiRect = EmailUiRect::new(180.0, 5.0, 135.0, 15.0);

pub const EMAIL_UI_INACTIVE_TAB_FILL_RECT: EmailUiRect = EmailUiRect::new(0.0, 0.0, 343.0, 30.0);

pub const EMAIL_UI_FROM_TITLE_RECT: EmailUiRect = EmailUiRect::new(1.0, 36.0, 203.0, 20.0);

pub const EMAIL_UI_SUBJECT_TITLE_RECT: EmailUiRect = EmailUiRect::new(204.0, 36.0, 219.0, 20.0);

pub const EMAIL_UI_DATE_TITLE_RECT: EmailUiRect = EmailUiRect::new(421.0, 36.0, 100.0, 20.0);

pub const EMAIL_UI_FROM_BOX_RECT: EmailUiRect = EmailUiRect::new(1.0, 58.0, 203.0, 28.0);

pub const EMAIL_UI_SUBJECT_BOX_RECT: EmailUiRect = EmailUiRect::new(204.0, 58.0, 219.0, 28.0);

pub const EMAIL_UI_DATE_BOX_RECT: EmailUiRect = EmailUiRect::new(421.0, 58.0, 100.0, 28.0);

pub const EMAIL_UI_PREVIOUS_RECT: EmailUiRect = EmailUiRect::new(1.0, 202.0, 120.0, 26.0);

pub const EMAIL_UI_NEXT_RECT: EmailUiRect = EmailUiRect::new(440.0, 202.0, 120.0, 26.0);

pub const EMAIL_UI_PAGE_SELECTION_RECT: EmailUiRect = EmailUiRect::new(210.0, 205.0, 30.0, 20.0);

pub const EMAIL_UI_PAGE_OF_RECT: EmailUiRect = EmailUiRect::new(240.0, 205.0, 20.0, 20.0);

pub const EMAIL_UI_PAGE_NUMBER_RECT: EmailUiRect = EmailUiRect::new(260.0, 205.0, 30.0, 20.0);

pub const EMAIL_UI_PAGE_LABEL_RECT: EmailUiRect = EmailUiRect::new(280.0, 205.0, 60.0, 20.0);

pub const EMAIL_UI_ROW_SELECTION_RECT: EmailUiRect = EmailUiRect::new(2.0, 58.0, 556.0, 23.0);

pub const EMAIL_UI_DETAIL_ICON_RECT: EmailUiRect = EmailUiRect::new(10.0, 10.0, 79.0, 68.0);

/// `Panel_EmailList.DoData` labels the `FFSlotEmpty` backdrop (62x62) and then
/// the 64x64 NPC icon into `rectGuideIcon` with FusionFallInvenSkin `label`
/// (UpperLeft, ImageLeft, padding 3/0/3/0). Unity fits each image into the
/// 79x62 content box without upscaling, so both occupy this rectangle.
pub const EMAIL_UI_DETAIL_GUIDE_ICON_IMAGE_RECT: EmailUiRect =
    EmailUiRect::new(10.0, 13.0, 62.0, 62.0);

pub const EMAIL_UI_DETAIL_FROM_RECT: EmailUiRect = EmailUiRect::new(10.0, 20.0, 60.0, 20.0);

pub const EMAIL_UI_DETAIL_FROM_NAME_RECT: EmailUiRect = EmailUiRect::new(70.0, 17.0, 220.0, 20.0);

pub const EMAIL_UI_DETAIL_SUBJECT_RECT: EmailUiRect = EmailUiRect::new(10.0, 45.0, 80.0, 20.0);

pub const EMAIL_UI_DETAIL_SUBJECT_NAME_RECT: EmailUiRect =
    EmailUiRect::new(95.0, 42.0, 200.0, 20.0);

pub const EMAIL_UI_DETAIL_RECEIVED_RECT: EmailUiRect = EmailUiRect::new(378.0, 45.0, 90.0, 20.0);

pub const EMAIL_UI_DETAIL_RECEIVED_DAY_RECT: EmailUiRect =
    EmailUiRect::new(465.0, 42.0, 90.0, 20.0);

pub const EMAIL_UI_DETAIL_TEXT_RECT: EmailUiRect = EmailUiRect::new(10.0, 82.0, 538.0, 140.0);

pub const EMAIL_UI_DETAIL_ITEM_RECT: EmailUiRect = EmailUiRect::new(40.0, 230.0, 67.0, 67.0);

pub const EMAIL_UI_ACCEPT_ALL_RECT: EmailUiRect = EmailUiRect::new(40.0, 305.0, 264.0, 20.0);

pub const EMAIL_UI_ACCEPT_TAROS_RECT: EmailUiRect = EmailUiRect::new(324.0, 305.0, 205.0, 20.0);

pub const EMAIL_UI_TAROS_LABEL_RECT: EmailUiRect = EmailUiRect::new(324.0, 260.0, 160.0, 20.0);

pub const EMAIL_UI_REMOVE_BUDDY_RECT: EmailUiRect = EmailUiRect::new(377.0, 12.0, 156.0, 20.0);

pub const EMAIL_UI_DELETE_RECT: EmailUiRect = EmailUiRect::new(10.0, 342.0, 152.0, 26.0);

pub const EMAIL_UI_REPLY_RECT: EmailUiRect = EmailUiRect::new(437.0, 342.0, 114.0, 26.0);

pub const EMAIL_UI_COMPOSE_WINDOW_RECT: EmailUiRect = EmailUiRect::new(0.0, 0.0, 940.0, 634.0);

pub const EMAIL_UI_COMPOSE_INNER_RECT: EmailUiRect = EmailUiRect::new(6.0, 3.0, 565.0, 600.0);

pub const EMAIL_UI_COMPOSE_TITLE_RECT: EmailUiRect = EmailUiRect::new(14.0, 2.0, 130.0, 15.0);

pub const EMAIL_UI_COMPOSE_CLOSE_RECT: EmailUiRect = EmailUiRect::new(534.0, 15.0, 30.0, 30.0);

pub const EMAIL_UI_COMPOSE_TO_RECT: EmailUiRect = EmailUiRect::new(20.0, 37.0, 80.0, 20.0);

pub const EMAIL_UI_COMPOSE_TO_NAME_RECT: EmailUiRect = EmailUiRect::new(115.0, 40.0, 280.0, 20.0);

pub const EMAIL_UI_COMPOSE_SUBJECT_RECT: EmailUiRect = EmailUiRect::new(20.0, 77.0, 80.0, 20.0);

pub const EMAIL_UI_COMPOSE_SUBJECT_FIELD_RECT: EmailUiRect =
    EmailUiRect::new(115.0, 80.0, 234.0, 20.0);

pub const EMAIL_UI_COMPOSE_BUDDY_RECT: EmailUiRect = EmailUiRect::new(408.0, 36.0, 115.0, 22.0);

pub const EMAIL_UI_COMPOSE_BODY_RECT: EmailUiRect = EmailUiRect::new(18.0, 120.0, 536.0, 206.0);

pub const EMAIL_UI_COMPOSE_BODY_FIELD_RECT: EmailUiRect =
    EmailUiRect::new(28.0, 130.0, 516.0, 186.0);

pub const EMAIL_UI_COMPOSE_ATTACHMENT_ICON_RECT: EmailUiRect =
    EmailUiRect::new(25.0, 347.0, 26.0, 20.0);

pub const EMAIL_UI_COMPOSE_ATTACHMENT_LABEL_RECT: EmailUiRect =
    EmailUiRect::new(50.0, 344.0, 130.0, 20.0);

pub const EMAIL_UI_COMPOSE_ITEM_RECT: EmailUiRect = EmailUiRect::new(30.0, 392.0, 68.0, 64.0);

pub const EMAIL_UI_COMPOSE_ADD_TAROS_RECT: EmailUiRect =
    EmailUiRect::new(327.0, 453.0, 206.0, 20.0);

pub const EMAIL_UI_COMPOSE_TAROS_RECT: EmailUiRect = EmailUiRect::new(341.0, 411.0, 145.0, 20.0);

pub const EMAIL_UI_COMPOSE_POSTAGE_RECT: EmailUiRect = EmailUiRect::new(10.0, 510.0, 545.0, 18.0);

pub const EMAIL_UI_COMPOSE_CANCEL_RECT: EmailUiRect = EmailUiRect::new(15.0, 550.0, 150.0, 25.0);

pub const EMAIL_UI_COMPOSE_SEND_RECT: EmailUiRect = EmailUiRect::new(416.0, 550.0, 130.0, 25.0);

pub const EMAIL_UI_BUDDY_POPUP_RECT: EmailUiRect = EmailUiRect::new(568.0, 28.0, 358.0, 255.0);

pub const EMAIL_UI_BUDDY_LIST_VIEWPORT_RECT: EmailUiRect =
    EmailUiRect::new(20.0, 54.0, 310.0, 178.0);

pub const EMAIL_UI_BUDDY_LIST_CONTENT_WIDTH: f32 = 270.0;

pub const EMAIL_UI_BUDDY_ROW_WIDTH: f32 = 280.0;

pub const EMAIL_UI_BUDDY_ROW_HEIGHT: f32 = 30.0;

pub const EMAIL_UI_CALCULATOR_POPUP_RECT: EmailUiRect =
    EmailUiRect::new(564.0, 287.0, 210.0, 255.0);

pub const EMAIL_UI_RIGHT_PANEL_RECT: EmailUiRect = EmailUiRect::new(0.0, 0.0, 380.0, 632.0);

pub const EMAIL_UI_INVENTORY_VIEWPORT_RECT: EmailUiRect = EmailUiRect::new(6.0, 36.0, 366.0, 500.0);

pub const EMAIL_UI_RIGHT_CLOSE_RECT: EmailUiRect = EmailUiRect::new(400.0, 5.0, 30.0, 30.0);

pub const EMAIL_UI_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(6.0, 6.0),
    max_inset: Vec2::new(6.0, 4.0),
};

pub const EMAIL_UI_RED_BUTTON_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 5.0),
    max_inset: Vec2::new(5.0, 5.0),
};

pub const EMAIL_UI_DATA_BACK_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(11.0, 11.0),
    max_inset: Vec2::new(8.0, 11.0),
};

pub const EMAIL_UI_DATA_BOX_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(4.0, 4.0),
    max_inset: Vec2::new(4.0, 4.0),
};

pub const EMAIL_UI_RIGHT_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(5.0, 0.0),
    max_inset: Vec2::new(5.0, 0.0),
};

pub const EMAIL_UI_INVENTORY_PANEL_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(136.0, 33.0),
    max_inset: Vec2::new(10.0, 77.0),
};

#[must_use]
pub fn clean_email_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / EMAIL_UI_REFERENCE_SCALE_HEIGHT * EMAIL_UI_SCALE_NUDGE).max(1.0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EmailUiLayout {
    pub viewport: Vec2,
    pub scale: Vec2,
    pub background: EmailUiRect,
    pub left_backplate: EmailUiRect,
    pub right_backplate: EmailUiRect,
    pub list_window: EmailUiRect,
    pub compose_window: EmailUiRect,
    pub right_window: EmailUiRect,
}

#[must_use]
pub fn email_ui_layout(
    viewport: Vec2,
    ui_scale: f32,
    compose_open_elapsed_seconds: f32,
) -> EmailUiLayout {
    email_ui_layout_with_opening(
        viewport,
        ui_scale,
        compose_open_elapsed_seconds,
        EMAIL_UI_OPEN_SECONDS,
    )
}

#[must_use]
pub fn email_ui_layout_with_opening(
    viewport: Vec2,
    ui_scale: f32,
    compose_open_elapsed_seconds: f32,
    right_open_elapsed_seconds: f32,
) -> EmailUiLayout {
    let width = legacy_extent(viewport.x);
    let height = legacy_extent(viewport.y);
    let viewport = Vec2::new(width as f32, height as f32);
    let fit_scale = Vec2::new(
        (viewport.x / EMAIL_UI_REFERENCE_WIDTH as f32).clamp(0.0, 1.0),
        (viewport.y / EMAIL_UI_REFERENCE_HEIGHT as f32).clamp(0.0, 1.0),
    );
    let ui_scale = if ui_scale.is_finite() && ui_scale > 0.0 {
        ui_scale
    } else {
        1.0
    };
    let scale = fit_scale * ui_scale;
    let center_x = if width > EMAIL_UI_REFERENCE_WIDTH {
        (width - EMAIL_UI_REFERENCE_WIDTH) / 2
    } else {
        0
    };
    let top = ((height - EMAIL_UI_REFERENCE_HEIGHT) / 2).max(0);
    let backplate_left = (width - EMAIL_UI_BACKPLATE_WIDTH) / 2;
    let backplate_top = (height - EMAIL_UI_BACKPLATE_HEIGHT) / 2;
    let compose_remaining = 1.0 - email_compose_open_fraction(compose_open_elapsed_seconds);
    let compose_x = center_x as f32
        - (EMAIL_UI_COMPOSE_WINDOW_RECT.width - EMAIL_UI_BUDDY_POPUP_RECT.width)
            * compose_remaining;
    let right_target_x = (center_x + 585) as f32;
    let right_remaining = 1.0 - email_compose_open_fraction(right_open_elapsed_seconds);
    let right_x = right_target_x + (EMAIL_UI_RIGHT_OFFSCREEN_X - right_target_x) * right_remaining;
    EmailUiLayout {
        viewport,
        scale,
        background: EmailUiRect::new(
            ((width - EMAIL_UI_BACKGROUND_WIDTH as i32) / 2) as f32,
            ((height - EMAIL_UI_BACKGROUND_HEIGHT as i32) / 2) as f32,
            EMAIL_UI_BACKGROUND_WIDTH,
            EMAIL_UI_BACKGROUND_HEIGHT,
        ),
        left_backplate: EmailUiRect::new(backplate_left as f32, backplate_top as f32, 585.0, 653.0),
        right_backplate: EmailUiRect::new(
            (backplate_left + 585) as f32,
            backplate_top as f32,
            451.0,
            653.0,
        ),
        list_window: EMAIL_UI_LIST_WINDOW_RECT.translated(center_x as f32, top as f32),
        compose_window: EMAIL_UI_COMPOSE_WINDOW_RECT.translated(compose_x, top as f32),
        right_window: EMAIL_UI_RIGHT_PANEL_RECT.translated(right_x, top as f32),
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_email_ui_layout(
    model: Res<EmailUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<&mut Node, With<EmailUiRoot>>,
    mut backgrounds: Query<
        (&mut Node, &mut UiTransform),
        (
            With<EmailUiBackground>,
            Without<EmailUiRoot>,
            Without<EmailUiLeftBackplate>,
            Without<EmailUiRightBackplate>,
            Without<EmailUiListPanel>,
            Without<EmailUiComposePanel>,
            Without<EmailUiRightPanel>,
        ),
    >,
    mut left_backplates: Query<
        (&mut Node, &mut UiTransform),
        (
            With<EmailUiLeftBackplate>,
            Without<EmailUiRoot>,
            Without<EmailUiBackground>,
            Without<EmailUiRightBackplate>,
            Without<EmailUiListPanel>,
            Without<EmailUiComposePanel>,
            Without<EmailUiRightPanel>,
        ),
    >,
    mut right_backplates: Query<
        (&mut Node, &mut UiTransform),
        (
            With<EmailUiRightBackplate>,
            Without<EmailUiRoot>,
            Without<EmailUiBackground>,
            Without<EmailUiLeftBackplate>,
            Without<EmailUiListPanel>,
            Without<EmailUiComposePanel>,
            Without<EmailUiRightPanel>,
        ),
    >,
    mut list_panels: Query<
        (&mut Node, &mut UiTransform),
        (
            With<EmailUiListPanel>,
            Without<EmailUiRoot>,
            Without<EmailUiBackground>,
            Without<EmailUiLeftBackplate>,
            Without<EmailUiRightBackplate>,
            Without<EmailUiComposePanel>,
            Without<EmailUiRightPanel>,
        ),
    >,
    mut compose_panels: Query<
        (&mut Node, &mut UiTransform),
        (
            With<EmailUiComposePanel>,
            Without<EmailUiRoot>,
            Without<EmailUiBackground>,
            Without<EmailUiLeftBackplate>,
            Without<EmailUiRightBackplate>,
            Without<EmailUiListPanel>,
            Without<EmailUiRightPanel>,
        ),
    >,
    mut right_panels: Query<
        (&mut Node, &mut UiTransform),
        (
            With<EmailUiRightPanel>,
            Without<EmailUiRoot>,
            Without<EmailUiBackground>,
            Without<EmailUiLeftBackplate>,
            Without<EmailUiRightBackplate>,
            Without<EmailUiListPanel>,
            Without<EmailUiComposePanel>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = email_ui_layout_with_opening(
        viewport,
        model.effective_ui_scale(viewport.y),
        model.opening_elapsed_seconds,
        model.right_opening_elapsed_seconds,
    );
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
        apply_group(&mut node, &mut transform, layout.background, layout.scale);
    }
    for (mut node, mut transform) in &mut left_backplates {
        apply_group(
            &mut node,
            &mut transform,
            layout.left_backplate,
            layout.scale,
        );
        node.display = if model.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (mut node, mut transform) in &mut right_backplates {
        apply_group(
            &mut node,
            &mut transform,
            layout.right_backplate,
            layout.scale,
        );
    }
    for (mut node, mut transform) in &mut list_panels {
        apply_group(&mut node, &mut transform, layout.list_window, layout.scale);
        node.display = if model.visible && model.screen == EmailScreen::List {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (mut node, mut transform) in &mut compose_panels {
        apply_group(
            &mut node,
            &mut transform,
            layout.compose_window,
            layout.scale,
        );
        node.display = if model.visible && model.screen == EmailScreen::Compose {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (mut node, mut transform) in &mut right_panels {
        apply_group(&mut node, &mut transform, layout.right_window, layout.scale);
    }
}
