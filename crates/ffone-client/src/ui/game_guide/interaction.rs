use super::*;

pub const GAME_GUIDE_HELP_BUTTON_PATH: &str =
    "ui/en/gameplay/game-guide/controls/help-button-normal.png";

pub const GAME_GUIDE_HELP_BUTTON_OVER_PATH: &str =
    "ui/en/gameplay/game-guide/controls/help-button-hover.png";

pub const GAME_GUIDE_SCROLL_UP_PATH: &str = "ui/en/gameplay/game-guide/controls/scroll-up.png";

pub const GAME_GUIDE_SCROLL_DOWN_PATH: &str = "ui/en/gameplay/game-guide/controls/scroll-down.png";

pub const GAME_GUIDE_SCROLL_TRACK_PATH: &str =
    "ui/en/gameplay/game-guide/controls/scroll-track.png";

pub const GAME_GUIDE_SCROLL_THUMB_PATH: &str =
    "ui/en/gameplay/game-guide/controls/scroll-thumb.png";

pub const GAME_GUIDE_NAV_BUTTON_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const GAME_GUIDE_NAV_BUTTON_OVER_PATH: &str = "ui/en/gameplay/chat/blue_button_over.png";

pub(super) const LEGACY_SCROLL_STEP: f32 = 200.0;

/// Baseline compensation for the approved Chalet replacement in the
/// upper-left 16 px help-button content box.
pub const GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET: f32 = 3.0;

/// Vertical position of the 12.34 px JEFFE replacement line inside the
/// standard 27 px button content box: top padding 3 + centered in 18 px.
pub const GAME_GUIDE_NAV_BUTTON_TEXT_Y_OFFSET: f32 = 5.83;

pub(super) const HOVER_BLUE: Color = Color::srgb(0.0, 0.2, 0.4);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GameGuideButtonVisual {
    Help,
    Navigation,
    Close,
}

#[derive(Component)]
pub(super) struct GameGuideScrollbar;

#[derive(Component)]
pub(super) struct GameGuideScrollThumb;

#[derive(Component)]
pub(super) struct GameGuideButtonLabel;

pub(super) fn help_button(
    parent: &mut ChildSpawnerCommands,
    rect: GuideUiRect,
    control: GameGuideControl,
    assets: &GameGuideAssets,
) -> Entity {
    parent
        .spawn((
            Button,
            control,
            GameGuideButtonVisual::Help,
            rect_node(rect),
            ImageNode {
                image: assets.help_button.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .id()
}

pub(super) fn navigation_button(
    parent: &mut ChildSpawnerCommands,
    rect: GuideUiRect,
    control: GameGuideControl,
    key: &'static str,
    fallback: &'static str,
    assets: &GameGuideAssets,
) {
    parent
        .spawn((
            Button,
            control,
            GameGuideButtonVisual::Navigation,
            rect_node(rect),
            ImageNode {
                image: assets.nav_button.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(5.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
        ))
        .with_children(|button| {
            let entity = spawn_text(
                button,
                GuideUiRect::new(0.0, GAME_GUIDE_NAV_BUTTON_TEXT_Y_OFFSET, rect.width, 12.34),
                key,
                fallback,
                assets.jeffe_font.clone(),
                12.0,
                12.34,
                LABEL_CYAN,
                Justify::Center,
            );
            button
                .commands()
                .entity(entity)
                .insert(GameGuideButtonLabel);
        });
}

pub(super) fn sync_game_guide_scroll(
    mut model: ResMut<GameGuideUiModel>,
    mut stacks: Query<(&ComputedNode, &mut Node), With<GameGuideContentStack>>,
    mut scrollbars: Query<&mut Node, (With<GameGuideScrollbar>, Without<GameGuideContentStack>)>,
    mut thumbs: Query<
        &mut Node,
        (
            With<GameGuideScrollThumb>,
            Without<GameGuideScrollbar>,
            Without<GameGuideContentStack>,
        ),
    >,
) {
    if !model.visible {
        return;
    }
    if let Ok((computed, mut stack_node)) = stacks.single_mut() {
        let content_height = computed.size().y;
        let max_scroll = (content_height - CONTENT_VIEW_HEIGHT).max(0.0);
        model.content_scroll_y = model.content_scroll_y.clamp(0.0, max_scroll);
        stack_node.top = px(3.0 - model.content_scroll_y);
        let scrollbar_visible = content_height > CONTENT_VIEW_HEIGHT;
        for mut node in &mut scrollbars {
            node.display = if scrollbar_visible {
                Display::Flex
            } else {
                Display::None
            };
        }
        if scrollbar_visible {
            let track_height = 356.0;
            let thumb_height =
                (track_height * CONTENT_VIEW_HEIGHT / content_height).clamp(16.0, track_height);
            let travel = track_height - thumb_height;
            let fraction = if max_scroll > 0.0 {
                model.content_scroll_y / max_scroll
            } else {
                0.0
            };
            for mut node in &mut thumbs {
                node.top = px(12.0 + travel * fraction);
                node.height = px(thumb_height);
            }
        }
    }
}

pub(super) fn sync_game_guide_button_visuals(
    assets: Res<GameGuideAssets>,
    buttons: Query<
        (Entity, &Interaction, &GameGuideButtonVisual),
        Or<(Changed<Interaction>, Changed<GameGuideButtonVisual>)>,
    >,
    mut images: Query<&mut ImageNode>,
    children: Query<&Children>,
    mut labels: Query<&mut TextColor, With<GameGuideButtonLabel>>,
) {
    for (entity, interaction, visual) in &buttons {
        let over = *interaction != Interaction::None;
        if let Ok(mut image) = images.get_mut(entity) {
            image.image = match visual {
                GameGuideButtonVisual::Help if over => assets.help_button_over.clone(),
                GameGuideButtonVisual::Help => assets.help_button.clone(),
                GameGuideButtonVisual::Navigation if over => assets.nav_button_over.clone(),
                GameGuideButtonVisual::Navigation => assets.nav_button.clone(),
                GameGuideButtonVisual::Close if over => assets.close_over.clone(),
                GameGuideButtonVisual::Close => assets.close.clone(),
            };
        }
        if let Ok(button_children) = children.get(entity) {
            for child in button_children.iter() {
                if let Ok(mut color) = labels.get_mut(child) {
                    color.0 = if over { HOVER_BLUE } else { LABEL_CYAN };
                }
            }
        }
    }
}
