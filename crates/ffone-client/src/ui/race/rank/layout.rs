use super::*;

pub const RACE_RANK_CAMERA_HEIGHT: f32 = 1.2;

pub const RACE_RANK_INVENTORY_WIDTH: f32 = 1_020.0;

pub const RACE_RANK_INVENTORY_HEIGHT: f32 = 638.0;

pub const RACE_RANK_LEFT_WIDTH: f32 = 490.0;

pub const RACE_RANK_RIGHT_WIDTH: f32 = 472.0;

pub const RACE_RANK_BACKDROP_RECT: RaceUiRect = RaceUiRect::new(0.0, 0.0, 1_920.0, 1_440.0);

pub const RACE_RANK_LEFT_GROUP_RECT: RaceUiRect = RaceUiRect::new(0.0, 0.0, 490.0, 632.0);

pub const RACE_RANK_RIGHT_GROUP_RECT: RaceUiRect = RaceUiRect::new(0.0, 0.0, 472.0, 632.0);

pub const RACE_RANK_SHELL_RECT: RaceUiRect = RaceUiRect::new(-8.0, -7.0, 1_035.0, 653.0);

pub const RACE_RANK_TITLE_RECT: RaceUiRect = RaceUiRect::new(0.0, 0.0, 481.0, 87.0);

pub const RACE_RANK_CAMERA_RECT: RaceUiRect = RaceUiRect::new(230.0, 0.0, 200.0, 150.0);

pub const RACE_RANK_TITLE_COPY_RECT: RaceUiRect = RaceUiRect::new(10.0, 7.0, 164.0, 20.0);

pub const RACE_RANK_NPC_COPY_RECT: RaceUiRect = RaceUiRect::new(10.0, 26.0, 164.0, 20.0);

pub const RACE_RANK_LOCATION_BOX_RECT: RaceUiRect = RaceUiRect::new(0.0, 137.0, 477.0, 492.0);

pub const RACE_RANK_LOCATION_BLACK_RECT: RaceUiRect = RaceUiRect::new(2.0, 168.0, 473.0, 434.0);

pub const RACE_RANK_FIRST_ROW_RECT: RaceUiRect = RaceUiRect::new(7.0, 172.0, 460.0, 80.0);

pub const RACE_RANK_PREVIOUS_RECT: RaceUiRect = RaceUiRect::new(1.0, 603.0, 130.0, 25.0);

pub const RACE_RANK_NEXT_RECT: RaceUiRect = RaceUiRect::new(346.0, 603.0, 130.0, 25.0);

pub const RACE_RANK_PAGE_RECT: RaceUiRect = RaceUiRect::new(133.0, 605.0, 213.0, 20.0);

pub const RACE_RANK_LEFT_ARROW_RECT: RaceUiRect = RaceUiRect::new(11.0, 611.0, 6.0, 11.0);

pub const RACE_RANK_RIGHT_ARROW_RECT: RaceUiRect = RaceUiRect::new(460.0, 611.0, 6.0, 11.0);

pub const RACE_RANK_SELECTED_POINT_RECT: RaceUiRect = RaceUiRect::new(467.0, 202.0, 30.0, 22.0);

pub const RACE_RANK_RIGHT_LOCATION_RECT: RaceUiRect = RaceUiRect::new(8.0, 8.0, 220.0, 15.0);

pub const RACE_RANK_RIGHT_NAME_RECT: RaceUiRect = RaceUiRect::new(8.0, 23.0, 200.0, 40.0);

pub const RACE_RANK_RIGHT_AREA_RECT: RaceUiRect = RaceUiRect::new(8.0, 63.0, 220.0, 15.0);

pub const RACE_RANK_BIG_IMAGE_RECT: RaceUiRect = RaceUiRect::new(230.0, 3.0, 239.0, 127.0);

pub const RACE_RANK_HEADER_RANK_RECT: RaceUiRect = RaceUiRect::new(15.0, 178.0, 89.0, 20.0);

pub const RACE_RANK_HEADER_PLAYER_RECT: RaceUiRect = RaceUiRect::new(165.0, 178.0, 275.0, 20.0);

pub const RACE_RANK_HEADER_SCORE_RECT: RaceUiRect = RaceUiRect::new(367.0, 178.0, 107.0, 20.0);

pub const RACE_RANK_MY_BLACK_RECT: RaceUiRect = RaceUiRect::new(2.0, 200.0, 468.0, 60.0);

pub const RACE_RANK_TOP_BLACK_RECT: RaceUiRect = RaceUiRect::new(2.0, 310.0, 468.0, 300.0);

pub const RACE_RANK_MY_ZERO_RECT: RaceUiRect = RaceUiRect::new(35.0, 230.0, 0.0, 0.0);

pub const RACE_RANK_TOP_ZERO_RECT: RaceUiRect = RaceUiRect::new(35.0, 320.0, 0.0, 0.0);

pub const RACE_RANK_TAB_GROUP_RECT: RaceUiRect = RaceUiRect::new(0.0, 110.0, 474.0, 40.0);

pub const RACE_RANK_TAB_BOX_RECT: RaceUiRect = RaceUiRect::new(0.0, 19.0, 484.0, 22.0);

pub const RACE_RANK_MY_SKY_RECT: RaceUiRect = RaceUiRect::new(2.0, 40.0, 468.0, 25.0);

pub const RACE_RANK_TOP_SKY_RECT: RaceUiRect = RaceUiRect::new(2.0, 150.0, 468.0, 25.0);

pub const RACE_RANK_MY_BEST_RECT: RaceUiRect = RaceUiRect::new(10.0, 154.0, 400.0, 30.0);

pub const RACE_RANK_TOP_COPY_RECT: RaceUiRect = RaceUiRect::new(3.0, 264.0, 470.0, 30.0);

pub const RACE_RANK_WARNING_RECT: RaceUiRect = RaceUiRect::new(150.0, 220.0, 100.0, 5.0);

pub const RACE_RANK_HIGHLIGHT_RECT: RaceUiRect = RaceUiRect::new(2.0, 305.0, 500.0, 30.0);

pub const RACE_RANK_TODAY_VISUAL_RECT: RaceUiRect = RaceUiRect::new(0.0, 0.0, 100.0, 29.0);

pub const RACE_RANK_WEEK_VISUAL_RECT: RaceUiRect = RaceUiRect::new(68.0, 0.0, 153.0, 29.0);

pub const RACE_RANK_MONTH_VISUAL_RECT: RaceUiRect = RaceUiRect::new(188.0, 0.0, 153.0, 29.0);

pub const RACE_RANK_ALL_VISUAL_RECT: RaceUiRect = RaceUiRect::new(310.0, 0.0, 153.0, 29.0);

pub const RACE_RANK_TODAY_HIT_RECT: RaceUiRect = RaceUiRect::new(16.0, 2.0, 62.0, 15.0);

pub const RACE_RANK_WEEK_HIT_RECT: RaceUiRect = RaceUiRect::new(103.0, 2.0, 92.0, 13.0);

pub const RACE_RANK_MONTH_HIT_RECT: RaceUiRect = RaceUiRect::new(214.0, 2.0, 92.0, 13.0);

pub const RACE_RANK_ALL_HIT_RECT: RaceUiRect = RaceUiRect::new(350.0, 2.0, 92.0, 13.0);

pub const RACE_RANK_TAB_BAR_RECT: RaceUiRect = RaceUiRect::new(0.0, 20.0, 472.0, 20.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaceRankLayout {
    pub viewport: Vec2,
    pub backdrop: RaceUiRect,
    pub left_group: RaceUiRect,
    pub right_group: RaceUiRect,
    pub shell: RaceUiRect,
    pub close: RaceUiRect,
    pub help: RaceUiRect,
}

#[must_use]
pub fn race_rank_layout(viewport: Vec2, window_scroll: f32) -> RaceRankLayout {
    race_rank_layout_samples(viewport, window_scroll, window_scroll)
}

#[must_use]
pub fn race_rank_layout_samples(
    viewport: Vec2,
    left_scroll_sample: f32,
    right_scroll_sample: f32,
) -> RaceRankLayout {
    let width = viewport.x.max(0.0);
    let height = viewport.y.max(0.0);
    let mut left = (width - RACE_RANK_INVENTORY_WIDTH) * 0.5;
    let top = ((height - RACE_RANK_INVENTORY_HEIGHT) * 0.5).max(0.0);
    let mut right = RACE_RANK_INVENTORY_WIDTH - 517.0;
    if width > RACE_RANK_INVENTORY_WIDTH {
        right += (width - RACE_RANK_INVENTORY_WIDTH) * 0.5;
    }
    let left_scroll = if left_scroll_sample.is_finite() {
        left_scroll_sample.clamp(0.0, 1.0)
    } else {
        1.0
    };
    if left_scroll < 1.0 {
        let factor = 1.0 - (FRAC_PI_2 * left_scroll).sin();
        left -= (left + RACE_RANK_LEFT_WIDTH) * factor;
    }
    let right_scroll = if right_scroll_sample.is_finite() {
        right_scroll_sample.clamp(0.0, 1.0)
    } else {
        1.0
    };
    if right_scroll < 1.0 {
        let factor = 1.0 - (FRAC_PI_2 * right_scroll).sin();
        right += (RACE_RANK_INVENTORY_WIDTH - right) * factor;
    }
    RaceRankLayout {
        viewport: Vec2::new(width, height),
        backdrop: RaceUiRect::new(
            (width - RACE_RANK_BACKDROP_RECT.width) * 0.5,
            (height - RACE_RANK_BACKDROP_RECT.height) * 0.5,
            RACE_RANK_BACKDROP_RECT.width,
            RACE_RANK_BACKDROP_RECT.height,
        ),
        left_group: RaceUiRect::new(left, top, RACE_RANK_LEFT_WIDTH, 632.0),
        right_group: RaceUiRect::new(right, top, RACE_RANK_RIGHT_WIDTH, 632.0),
        shell: RaceUiRect::new(left - 8.0, top - 7.0, 1_035.0, 653.0),
        close: RaceUiRect::new(right + RACE_RANK_RIGHT_WIDTH + 5.0, top, 30.0, 30.0),
        help: RaceUiRect::new(
            right + RACE_RANK_RIGHT_WIDTH + 10.0,
            top + RACE_RANK_RIGHT_GROUP_RECT.height - 40.0,
            30.0,
            30.0,
        ),
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_race_rank_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<RaceRankModel>,
    mut roots: Query<
        &mut Node,
        (
            With<RaceRankPresentationRoot>,
            Without<RaceRankPresentationBackdrop>,
            Without<RaceRankPresentationShell>,
            Without<RaceRankPresentationLeft>,
            Without<RaceRankPresentationRight>,
        ),
    >,
    mut backdrops: Query<
        &mut Node,
        (
            With<RaceRankPresentationBackdrop>,
            Without<RaceRankPresentationRoot>,
            Without<RaceRankPresentationShell>,
            Without<RaceRankPresentationLeft>,
            Without<RaceRankPresentationRight>,
        ),
    >,
    mut shells: Query<
        &mut Node,
        (
            With<RaceRankPresentationShell>,
            Without<RaceRankPresentationBackdrop>,
            Without<RaceRankPresentationLeft>,
            Without<RaceRankPresentationRight>,
        ),
    >,
    mut lefts: Query<
        &mut Node,
        (
            With<RaceRankPresentationLeft>,
            Without<RaceRankPresentationBackdrop>,
            Without<RaceRankPresentationShell>,
            Without<RaceRankPresentationRight>,
        ),
    >,
    mut rights: Query<
        &mut Node,
        (
            With<RaceRankPresentationRight>,
            Without<RaceRankPresentationBackdrop>,
            Without<RaceRankPresentationShell>,
            Without<RaceRankPresentationLeft>,
        ),
    >,
    mut controls: Query<
        (&RaceRankPresentationControl, &mut Node),
        (
            Without<RaceRankPresentationRoot>,
            Without<RaceRankPresentationBackdrop>,
            Without<RaceRankPresentationShell>,
            Without<RaceRankPresentationLeft>,
            Without<RaceRankPresentationRight>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let layout = race_rank_layout_samples(
        Vec2::new(window.width(), window.height()),
        model.left_scroll_sample(),
        model.right_scroll_sample(),
    );
    for mut root in &mut roots {
        root.display = if model.phase().visible() {
            Display::Flex
        } else {
            Display::None
        };
        root.width = px(layout.viewport.x);
        root.height = px(layout.viewport.y);
    }
    for mut node in &mut backdrops {
        set_rect(&mut node, layout.backdrop);
    }
    for mut node in &mut shells {
        set_rect(&mut node, layout.shell);
    }
    for mut node in &mut lefts {
        set_rect(&mut node, layout.left_group);
    }
    for mut node in &mut rights {
        set_rect(&mut node, layout.right_group);
    }
    for (control, mut node) in &mut controls {
        match control.0 {
            RaceRankControl::Close => set_rect(&mut node, layout.close),
            RaceRankControl::HelpIgnored => set_rect(&mut node, layout.help),
            _ => {}
        }
    }
}

pub(super) fn set_rect(node: &mut Node, rect: RaceUiRect) {
    node.left = px(rect.x);
    node.top = px(rect.y);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
