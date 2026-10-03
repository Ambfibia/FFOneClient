use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    rect: GuideUiRect,
    key: impl Into<String>,
    fallback: impl Into<String>,
    font: Handle<Font>,
    font_size: f32,
    line_height: f32,
    color: Color,
    justify: Justify,
) -> Entity {
    let fallback = fallback.into();
    parent
        .spawn((
            rect_node(rect),
            Text::new(fallback.clone()),
            LocalizedText::new(key, fallback),
            (
                TextFont {
                    font: (font).into(),
                    font_size: (font_size).into(),
                    ..default()
                },
                LineHeight::Px(line_height),
            ),
            TextColor(color),
            TextLayout::new(justify, LineBreak::WordBoundary),
            Pickable::IGNORE,
        ))
        .id()
}

pub(super) fn spawn_game_guide_ui(
    mut commands: Commands,
    assets: Res<GameGuideAssets>,
    existing: Query<(), With<GameGuideUiRoot>>,
) {
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            GameGuideUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                overflow: Overflow::clip(),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(i32::MAX - 45),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                GameGuideBackdrop,
                rect_node(GuideUiRect::new(0.0, 0.0, 1_920.0, 1_440.0)),
                UiTransform::default(),
                ImageNode {
                    image: assets.backdrop.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable::IGNORE,
            ));
            root.spawn((
                GameGuideWindow,
                rect_node(GAME_GUIDE_WINDOW_RECT),
                UiTransform::default(),
                ImageNode {
                    image: assets.background.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|window| {
                spawn_text(
                    window,
                    GAME_GUIDE_TITLE_RECT,
                    "ui.game_guide.title",
                    "GAME GUIDE",
                    assets.jeffe_font.clone(),
                    16.0,
                    16.45,
                    LABEL_CYAN,
                    Justify::Left,
                );
                spawn_text(
                    window,
                    GAME_GUIDE_TOPIC_PROMPT_RECT,
                    "ui.game_guide.choose_topic",
                    "CHOOSE A TOPIC:",
                    assets.jeffe_font.clone(),
                    8.0,
                    8.23,
                    LABEL_CYAN,
                    Justify::Left,
                );

                window
                    .spawn((
                        Node {
                            overflow: Overflow::clip(),
                            ..rect_node(GAME_GUIDE_MAIN_SCROLL_RECT)
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|viewport| {
                        for slot in 0..MAX_MAIN_TOPICS {
                            // Legacy buttons live in the scroll-view coordinate space.  Their
                            // serialized x=10 is three pixels left of the viewport's x=13.
                            let rect = GuideUiRect::new(
                                GAME_GUIDE_MAIN_BUTTON_RECT.x - GAME_GUIDE_MAIN_SCROLL_RECT.x,
                                GAME_GUIDE_MAIN_BUTTON_RECT.y - GAME_GUIDE_MAIN_SCROLL_RECT.y
                                    + slot as f32 * 21.0,
                                GAME_GUIDE_MAIN_BUTTON_RECT.width,
                                GAME_GUIDE_MAIN_BUTTON_RECT.height,
                            );
                            let button = help_button(
                                viewport,
                                rect,
                                GameGuideControl::Main(slot + 1),
                                &assets,
                            );
                            viewport.commands().entity(button).with_children(|button| {
                                let entity = spawn_text(
                                    button,
                                    GuideUiRect::new(
                                        6.0,
                                        GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET,
                                        rect.width - 6.0,
                                        rect.height - GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET,
                                    ),
                                    "ui.game_guide.unused",
                                    "",
                                    assets.chalet_font.clone(),
                                    10.0,
                                    10.06,
                                    LABEL_CYAN,
                                    Justify::Left,
                                );
                                button
                                    .commands()
                                    .entity(entity)
                                    .insert((GameGuideMainLabel(slot + 1), GameGuideButtonLabel));
                            });
                        }
                    });

                let heading = spawn_text(
                    window,
                    GAME_GUIDE_MAIN_TITLE_RECT,
                    "ui.game_guide.unused",
                    "",
                    assets.chalet_font.clone(),
                    14.0,
                    14.08,
                    LABEL_CYAN,
                    Justify::Left,
                );
                window
                    .commands()
                    .entity(heading)
                    .insert(GameGuideSelectedHeading);

                for slot in 0..MAX_SUB_TOPICS {
                    let column = slot / 5;
                    let row = slot % 5;
                    let rect = GuideUiRect::new(
                        GAME_GUIDE_SUB_BUTTON_RECT.x + column as f32 * 266.0,
                        GAME_GUIDE_SUB_BUTTON_RECT.y + row as f32 * 21.0,
                        GAME_GUIDE_SUB_BUTTON_RECT.width,
                        GAME_GUIDE_SUB_BUTTON_RECT.height,
                    );
                    let button = help_button(window, rect, GameGuideControl::Sub(slot), &assets);
                    window.commands().entity(button).with_children(|button| {
                        let entity = spawn_text(
                            button,
                            GuideUiRect::new(
                                6.0,
                                GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET,
                                rect.width - 6.0,
                                rect.height - GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET,
                            ),
                            "ui.game_guide.unused",
                            "",
                            assets.chalet_font.clone(),
                            10.0,
                            10.06,
                            LABEL_CYAN,
                            Justify::Left,
                        );
                        button
                            .commands()
                            .entity(entity)
                            .insert((GameGuideSubLabel(slot), GameGuideButtonLabel));
                    });
                }

                window
                    .spawn((
                        Node {
                            overflow: Overflow::clip(),
                            ..rect_node(GAME_GUIDE_CONTENT_VIEW_RECT)
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|viewport| {
                        viewport.spawn((
                            GameGuideContentStack {
                                page_id: usize::MAX,
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(1.0),
                                top: px(3.0),
                                width: px(790.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(CONTENT_GAP),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        viewport
                            .spawn((
                                GameGuideScrollbar,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(793.0),
                                    top: px(0.0),
                                    width: px(17.0),
                                    height: px(380.0),
                                    ..default()
                                },
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Button,
                                    GameGuideControl::ScrollUp,
                                    rect_node(GuideUiRect::new(0.0, 0.0, 17.0, 12.0)),
                                    ImageNode {
                                        image: assets.scroll_up.clone(),
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                ));
                                bar.spawn((
                                    rect_node(GuideUiRect::new(0.0, 12.0, 17.0, 356.0)),
                                    ImageNode {
                                        image: assets.scroll_track.clone(),
                                        image_mode: NodeImageMode::Sliced(TextureSlicer {
                                            border: BorderRect::axes(2.0, 4.0),
                                            center_scale_mode: SliceScaleMode::Stretch,
                                            sides_scale_mode: SliceScaleMode::Stretch,
                                            max_corner_scale: 1.0,
                                        }),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                                bar.spawn((
                                    GameGuideScrollThumb,
                                    rect_node(GuideUiRect::new(1.0, 12.0, 15.0, 40.0)),
                                    ImageNode {
                                        image: assets.scroll_thumb.clone(),
                                        image_mode: NodeImageMode::Sliced(TextureSlicer {
                                            border: BorderRect::axes(2.0, 4.0),
                                            center_scale_mode: SliceScaleMode::Stretch,
                                            sides_scale_mode: SliceScaleMode::Stretch,
                                            max_corner_scale: 1.0,
                                        }),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                                bar.spawn((
                                    Button,
                                    GameGuideControl::ScrollDown,
                                    rect_node(GuideUiRect::new(0.0, 368.0, 17.0, 12.0)),
                                    ImageNode {
                                        image: assets.scroll_down.clone(),
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                ));
                            });
                    });

                navigation_button(
                    window,
                    GAME_GUIDE_PREVIOUS_RECT,
                    GameGuideControl::Previous,
                    "ui.game_guide.previous_section",
                    "< PREVIOUS SECTION",
                    &assets,
                );
                navigation_button(
                    window,
                    GAME_GUIDE_NEXT_RECT,
                    GameGuideControl::Next,
                    "ui.game_guide.next_section",
                    "NEXT SECTION >",
                    &assets,
                );
                window.spawn((
                    Button,
                    GameGuideControl::Close,
                    GameGuideButtonVisual::Close,
                    rect_node(GAME_GUIDE_CLOSE_RECT),
                    ImageNode {
                        image: assets.close.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                ));
            });
        });
}
