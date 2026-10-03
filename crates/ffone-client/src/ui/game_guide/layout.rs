use super::*;

pub const GAME_GUIDE_WINDOW_RECT: GuideUiRect = GuideUiRect::new(0.0, 0.0, 1_036.0, 654.0);

pub const GAME_GUIDE_TITLE_RECT: GuideUiRect = GuideUiRect::new(18.0, 15.0, 150.0, 20.0);

pub const GAME_GUIDE_TOPIC_PROMPT_RECT: GuideUiRect = GuideUiRect::new(8.0, 52.0, 90.0, 10.0);

pub const GAME_GUIDE_MAIN_SCROLL_RECT: GuideUiRect = GuideUiRect::new(13.0, 70.0, 158.0, 321.0);

pub const GAME_GUIDE_MAIN_BUTTON_RECT: GuideUiRect = GuideUiRect::new(10.0, 80.0, 135.0, 16.0);

pub const GAME_GUIDE_SUB_BUTTON_RECT: GuideUiRect = GuideUiRect::new(200.0, 78.0, 256.0, 16.0);

pub const GAME_GUIDE_MAIN_TITLE_RECT: GuideUiRect = GuideUiRect::new(195.0, 48.0, 250.0, 20.0);

pub const GAME_GUIDE_CLOSE_RECT: GuideUiRect = GuideUiRect::new(980.0, 12.0, 30.0, 30.0);

pub const GAME_GUIDE_PREVIOUS_RECT: GuideUiRect = GuideUiRect::new(192.0, 607.0, 187.0, 27.0);

pub const GAME_GUIDE_NEXT_RECT: GuideUiRect = GuideUiRect::new(815.0, 607.0, 187.0, 27.0);

pub const GAME_GUIDE_CONTENT_VIEW_RECT: GuideUiRect = GuideUiRect::new(199.0, 200.0, 810.0, 380.0);

pub const GAME_GUIDE_CONTENT_RECT: GuideUiRect = GuideUiRect::new(200.0, 203.0, 790.0, 22.0);

pub const GAME_GUIDE_SCREENSHOT_RECT: GuideUiRect = GuideUiRect::new(420.0, 0.0, 350.0, 262.0);

pub(super) const CONTENT_VIEW_HEIGHT: f32 = 380.0;

pub(super) fn rect_node(rect: GuideUiRect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.x),
        top: px(rect.y),
        width: px(rect.width),
        height: px(rect.height),
        ..default()
    }
}

pub(super) fn sync_game_guide_layout(
    model: Res<GameGuideUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        &mut Node,
        (
            With<GameGuideUiRoot>,
            Without<GameGuideWindow>,
            Without<GameGuideBackdrop>,
        ),
    >,
    mut window_nodes: Query<
        (&mut Node, &mut UiTransform),
        (
            With<GameGuideWindow>,
            Without<GameGuideUiRoot>,
            Without<GameGuideBackdrop>,
        ),
    >,
    mut backdrops: Query<
        (&mut Node, &mut UiTransform),
        (
            With<GameGuideBackdrop>,
            Without<GameGuideWindow>,
            Without<GameGuideUiRoot>,
        ),
    >,
) {
    for mut root in &mut roots {
        root.display = if model.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !model.visible {
        return;
    }
    if let Ok(viewport) = windows.single() {
        let scale = clean_guide_ui_scale(viewport.height()) * model.ui_scale.max(0.01);
        for (mut node, mut transform) in &mut window_nodes {
            node.left = px((viewport.width() - GAME_GUIDE_WINDOW_RECT.width) * 0.5);
            node.top = px((viewport.height() - GAME_GUIDE_WINDOW_RECT.height) * 0.5);
            transform.scale = Vec2::splat(scale);
        }
        for (mut node, mut transform) in &mut backdrops {
            node.left = px((viewport.width() - 1_920.0) * 0.5);
            node.top = px((viewport.height() - 1_440.0) * 0.5);
            transform.scale = Vec2::splat(scale);
        }
    }
}
