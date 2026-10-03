use super::*;

pub(super) fn spawn_race_mode_presentation(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = RaceModePresentationAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            RaceModePresentationRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            GlobalZIndex(RACE_RESULT_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            for bottom in [false, true] {
                root.spawn((
                    RaceModePresentationBar { bottom },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        width: percent(100),
                        ..default()
                    },
                    ImageNode {
                        image: assets.black.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            root.spawn((
                RaceModePresentationPanel,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(RACE_RESULT_PANEL_WIDTH),
                    height: px(RACE_RESULT_PANEL_HEIGHT),
                    overflow: Overflow::visible(),
                    ..default()
                },
                ImageNode {
                    image: assets.background.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                for index in 0..RACE_RESULT_STAR_COUNT {
                    let rect = RaceUiRect::new(
                        RACE_STAR_RECT.x - index as f32 * 22.0,
                        RACE_STAR_RECT.y,
                        RACE_STAR_RECT.width,
                        RACE_STAR_RECT.height,
                    );
                    panel.spawn((
                        RaceModePresentationStar {
                            index,
                            filled: false,
                        },
                        rect.node(),
                        stretched_image(assets.star_empty.clone()),
                        Pickable::IGNORE,
                    ));
                    panel.spawn((
                        RaceModePresentationStar {
                            index,
                            filled: true,
                        },
                        Node {
                            display: Display::None,
                            ..rect.node()
                        },
                        stretched_image(assets.star.clone()),
                        Pickable::IGNORE,
                    ));
                }

                spawn_race_text(
                    panel,
                    RaceModeTextRole::RankCopy,
                    RACE_PERFECT_RECT,
                    &assets.jeffe,
                    14.0,
                    Justify::Left,
                    [0.8, 1.0, 1.0, 1.0],
                );
                spawn_static_race_text(
                    panel,
                    RACE_TIME_RECT,
                    RACE_MODE_TIME_KEY,
                    RACE_COPY_TIME,
                    &assets.jeffe,
                    14.0,
                    Justify::Right,
                    [0.8, 1.0, 1.0, 1.0],
                );
                spawn_static_race_text(
                    panel,
                    RaceUiRect::new(20.0, 103.0, 62.0, 12.0),
                    RACE_MODE_PODS_KEY,
                    RACE_COPY_PODS,
                    &assets.jeffe,
                    14.0,
                    Justify::Right,
                    [0.8, 1.0, 1.0, 1.0],
                );
                spawn_static_race_text(
                    panel,
                    RaceUiRect::new(20.0, 133.0, 62.0, 12.0),
                    RACE_MODE_SCORE_KEY,
                    RACE_COPY_SCORE,
                    &assets.jeffe,
                    14.0,
                    Justify::Right,
                    [0.8, 1.0, 1.0, 1.0],
                );
                for (role, rect) in [
                    (RaceModeTextRole::CurrentTime, RACE_TIME_DATA_RECT),
                    (RaceModeTextRole::CurrentPods, RACE_PODS_DATA_RECT),
                    (RaceModeTextRole::CurrentScore, RACE_SCORE_DATA_RECT),
                ] {
                    spawn_race_text(
                        panel,
                        role,
                        rect,
                        &assets.chalet,
                        12.0,
                        Justify::Left,
                        [1.0, 1.0, 0.0, 1.0],
                    );
                }
                spawn_static_race_text(
                    panel,
                    RACE_MY_BEST_RECT,
                    RACE_MODE_MY_BEST_KEY,
                    RACE_COPY_MY_BEST,
                    &assets.jeffe,
                    6.0,
                    Justify::Left,
                    [0.0, 0.2, 0.4, 1.0],
                );
                spawn_static_race_text(
                    panel,
                    RACE_BEST_TIME_RECT,
                    RACE_MODE_TIME_KEY,
                    RACE_COPY_TIME,
                    &assets.jeffe,
                    14.0,
                    Justify::Right,
                    [0.0, 0.8, 1.0, 1.0],
                );
                spawn_static_race_text(
                    panel,
                    RaceUiRect::new(210.0, 90.0, 62.0, 12.0),
                    RACE_MODE_PODS_KEY,
                    RACE_COPY_PODS,
                    &assets.jeffe,
                    14.0,
                    Justify::Right,
                    [0.0, 0.8, 1.0, 1.0],
                );
                spawn_static_race_text(
                    panel,
                    RaceUiRect::new(210.0, 120.0, 62.0, 12.0),
                    RACE_MODE_SCORE_KEY,
                    RACE_COPY_SCORE,
                    &assets.jeffe,
                    14.0,
                    Justify::Right,
                    [0.0, 0.8, 1.0, 1.0],
                );
                for (role, rect) in [
                    (RaceModeTextRole::BestTime, RACE_BEST_TIME_DATA_RECT),
                    (RaceModeTextRole::BestPods, RACE_BEST_PODS_DATA_RECT),
                    (RaceModeTextRole::BestScore, RACE_BEST_SCORE_DATA_RECT),
                ] {
                    spawn_race_text(
                        panel,
                        role,
                        rect,
                        &assets.chalet,
                        12.0,
                        Justify::Left,
                        [0.8, 1.0, 1.0, 1.0],
                    );
                }
                spawn_static_race_text(
                    panel,
                    RACE_REWARD_RECT,
                    RACE_MODE_REWARD_KEY,
                    RACE_COPY_REWARD,
                    &assets.jeffe,
                    14.0,
                    Justify::Left,
                    [0.0, 0.2, 0.4, 1.0],
                );

                panel.spawn((
                    RaceModeOptionalItemNode,
                    RACE_ITEM_BAR_RECT.node(),
                    stretched_image(assets.item_bar.clone()),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    RaceModeOptionalItemNode,
                    RaceModeOptionalItemIcon,
                    RACE_ITEM_RECT.node(),
                    ImageNode::default(),
                    Pickable::IGNORE,
                ));
                spawn_race_optional_text(
                    panel,
                    RaceModeTextRole::ItemName,
                    RaceUiRect::new(100.0, 224.0, 200.0, 20.0),
                    &assets.jeffe,
                    [0.8, 1.0, 1.0, 1.0],
                );
                // Deliberately the same y as item name in clean OnEndGUI.
                spawn_race_optional_text(
                    panel,
                    RaceModeTextRole::ItemLevel,
                    RaceUiRect::new(100.0, 224.0, 200.0, 20.0),
                    &assets.jeffe,
                    [1.0, 1.0, 0.0, 1.0],
                );
                spawn_race_optional_text(
                    panel,
                    RaceModeTextRole::InventoryFull,
                    RACE_INVENTORY_FULL_RECT,
                    &assets.chalet,
                    [1.0, 0.0, 0.0, 1.0],
                );

                panel.spawn((
                    RaceModeFusionMatterNode,
                    RACE_ITEM_BAR_RECT.node(),
                    stretched_image(assets.item_bar.clone()),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    RaceModeFusionMatterNode,
                    RACE_ITEM_RECT.node(),
                    stretched_image(assets.fusion_matter.clone()),
                    Pickable::IGNORE,
                ));
                spawn_race_text(
                    panel,
                    RaceModeTextRole::FusionMatter,
                    RaceUiRect::new(100.0, 224.0, 200.0, 40.0),
                    &assets.jeffe,
                    14.0,
                    Justify::Left,
                    [0.8, 1.0, 1.0, 1.0],
                );

                panel
                    .spawn((
                        Button,
                        RaceModeAcceptButton,
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..RACE_ACCEPT_RECT.node()
                        },
                        sliced_image(assets.button_normal.clone(), BorderRect::all(5.0)),
                    ))
                    .with_children(|button| {
                        let localized = LocalizedText::new(RACE_MODE_ACCEPT_KEY, RACE_COPY_ACCEPT);
                        button.spawn((
                            Text::new(localized_fallback(&localized)),
                            localized,
                            (
                                TextFont {
                                    font: (assets.jeffe.clone()).into(),
                                    font_size: (14.0).into(),
                                    ..default()
                                },
                                LineHeight::Px(14.0),
                            ),
                            TextColor(Color::WHITE),
                            TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            Pickable::IGNORE,
                        ));
                    });
            });
        });
}

pub(super) fn spawn_race_text(
    parent: &mut ChildSpawnerCommands,
    role: RaceModeTextRole,
    rect: RaceUiRect,
    font: &Handle<Font>,
    font_size: f32,
    justify: Justify,
    tint: [f32; 4],
) {
    let localized = initial_race_mode_localized(role);
    parent.spawn((
        RaceModePresentationText(role),
        Text::new(localized_fallback(&localized)),
        localized,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(font_size.max(1.0)),
        ),
        TextColor(color(tint)),
        TextLayout::new(justify, LineBreak::NoWrap),
        rect.node(),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_static_race_text(
    parent: &mut ChildSpawnerCommands,
    rect: RaceUiRect,
    key: &'static str,
    copy: &'static str,
    font: &Handle<Font>,
    font_size: f32,
    justify: Justify,
    tint: [f32; 4],
) {
    let localized = LocalizedText::new(key, copy);
    parent.spawn((
        Text::new(localized_fallback(&localized)),
        localized,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(font_size.max(1.0)),
        ),
        TextColor(color(tint)),
        TextLayout::new(justify, LineBreak::NoWrap),
        rect.node(),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_race_optional_text(
    parent: &mut ChildSpawnerCommands,
    role: RaceModeTextRole,
    rect: RaceUiRect,
    font: &Handle<Font>,
    tint: [f32; 4],
) {
    let localized = initial_race_mode_localized(role);
    parent.spawn((
        RaceModeOptionalItemNode,
        RaceModePresentationText(role),
        Text::new(localized_fallback(&localized)),
        localized,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (14.0).into(),
                ..default()
            },
            LineHeight::Px(14.0),
        ),
        TextColor(color(tint)),
        TextLayout::new(Justify::Left, LineBreak::NoWrap),
        rect.node(),
        Pickable::IGNORE,
    ));
}
