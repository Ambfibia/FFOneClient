use super::*;

pub const RACE_RESULT_PANEL_WIDTH: f32 = 398.0;

pub const RACE_RESULT_PANEL_HEIGHT: f32 = 434.0;

pub const RACE_END_WINDOW_RECT: RaceUiRect =
    RaceUiRect::new(0.0, 0.0, RACE_RESULT_PANEL_WIDTH, RACE_RESULT_PANEL_HEIGHT);

pub const RACE_STAR_RECT: RaceUiRect = RaceUiRect::new(158.0, 14.0, 19.0, 17.0);

pub const RACE_PERFECT_RECT: RaceUiRect = RaceUiRect::new(202.0, 14.0, 120.0, 20.0);

pub const RACE_MY_BEST_RECT: RaceUiRect = RaceUiRect::new(205.0, 50.0, 100.0, 10.0);

pub const RACE_TIME_RECT: RaceUiRect = RaceUiRect::new(20.0, 83.0, 62.0, 12.0);

pub const RACE_TIME_DATA_RECT: RaceUiRect = RaceUiRect::new(100.0, 78.0, 65.0, 16.0);

pub const RACE_PODS_DATA_RECT: RaceUiRect = RaceUiRect::new(100.0, 98.0, 65.0, 16.0);

pub const RACE_SCORE_DATA_RECT: RaceUiRect = RaceUiRect::new(100.0, 128.0, 80.0, 16.0);

pub const RACE_BEST_TIME_RECT: RaceUiRect = RaceUiRect::new(210.0, 70.0, 62.0, 12.0);

pub const RACE_BEST_TIME_DATA_RECT: RaceUiRect = RaceUiRect::new(295.0, 65.0, 65.0, 16.0);

pub const RACE_BEST_PODS_DATA_RECT: RaceUiRect = RaceUiRect::new(295.0, 85.0, 65.0, 16.0);

pub const RACE_BEST_SCORE_DATA_RECT: RaceUiRect = RaceUiRect::new(295.0, 117.0, 80.0, 16.0);

pub const RACE_REWARD_RECT: RaceUiRect = RaceUiRect::new(10.0, 195.0, 80.0, 20.0);

pub const RACE_ITEM_BAR_RECT: RaceUiRect = RaceUiRect::new(13.0, 216.0, 367.0, 76.0);

pub const RACE_ITEM_RECT: RaceUiRect = RaceUiRect::new(20.0, 224.0, 62.0, 62.0);

pub const RACE_ACCEPT_RECT: RaceUiRect = RaceUiRect::new(230.0, 393.0, 154.0, 26.0);

/// Exact clean bug: `itemRect.width - 100`, i.e. a negative label width.
pub const RACE_INVENTORY_FULL_RECT: RaceUiRect = RaceUiRect::new(120.0, 254.0, -38.0, 20.0);

pub(super) fn sync_race_mode_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<RaceModeModel>,
    mut roots: Query<&mut Node, With<RaceModePresentationRoot>>,
    mut panels: Query<
        &mut Node,
        (
            With<RaceModePresentationPanel>,
            Without<RaceModePresentationRoot>,
            Without<RaceModePresentationBar>,
        ),
    >,
    mut bars: Query<
        (&RaceModePresentationBar, &mut Node),
        (
            Without<RaceModePresentationRoot>,
            Without<RaceModePresentationPanel>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let width = window.width();
    let height = window.height();
    let band = ((height - RACE_RESULT_PANEL_HEIGHT) * 0.5).max(0.0);
    for mut root in &mut roots {
        root.display = if model.phase().paints() {
            Display::Flex
        } else {
            Display::None
        };
        root.width = px(width);
        root.height = px(height);
    }
    for mut panel in &mut panels {
        panel.left = px((width - RACE_RESULT_PANEL_WIDTH).max(0.0));
        panel.top = px(band);
    }
    for (bar, mut node) in &mut bars {
        node.top = px(if bar.bottom { height - band } else { 0.0 });
        node.height = px(band);
    }
}
