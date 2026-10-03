use super::*;

pub(super) fn spawn_race_rank_presentation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<RaceRankCatalog>,
) {
    let assets = RaceRankPresentationAssets::load(&asset_server, &catalog);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            RaceRankPresentationRoot,
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
            GlobalZIndex(RACE_RANK_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                RaceRankPresentationBackdrop,
                RACE_RANK_BACKDROP_RECT.node(),
                stretch(assets.image(RACE_RANK_BACKDROP_PATH)),
                Pickable::IGNORE,
                ZIndex(0),
            ));
            root.spawn((
                RaceRankPresentationShell,
                RACE_RANK_SHELL_RECT.node(),
                stretch(assets.image(RACE_RANK_SHELL_PATH)),
                Pickable::IGNORE,
                ZIndex(1),
            ));
            root.spawn((
                RaceRankPresentationLeft,
                RACE_RANK_LEFT_GROUP_RECT.node(),
                Pickable::IGNORE,
                ZIndex(2),
            ))
            .with_children(|left| spawn_race_rank_left(left, &assets));
            root.spawn((
                RaceRankPresentationRight,
                RACE_RANK_RIGHT_GROUP_RECT.node(),
                sliced(
                    assets.image(RACE_RANK_RIGHT_BACK_PATH),
                    BorderRect::all(10.0),
                ),
                Pickable::IGNORE,
                ZIndex(2),
            ))
            .with_children(|right| spawn_race_rank_right(right, &assets));
            spawn_rank_control(
                root,
                RaceRankControl::Close,
                RaceUiRect::new(0.0, 0.0, 30.0, 30.0),
                assets.image(RACE_RANK_CLOSE_PATH),
            );
            spawn_rank_control(
                root,
                RaceRankControl::HelpIgnored,
                RaceUiRect::new(0.0, 0.0, 30.0, 30.0),
                assets.image(RACE_RANK_HELP_PATH),
            );
        });
}

pub(super) fn spawn_race_rank_left(parent: &mut ChildSpawnerCommands, assets: &RaceRankPresentationAssets) {
    parent.spawn((
        RACE_RANK_TITLE_RECT.node(),
        stretch(assets.image(RACE_RANK_TITLE_PATH)),
        Pickable::IGNORE,
    ));
    parent.spawn((
        RaceRankPresentationNpcCameraSlot,
        RACE_RANK_CAMERA_RECT.node(),
        BackgroundColor(Color::srgb(0.192, 0.302, 0.475)),
        Pickable::IGNORE,
    ));
    rank_text(
        parent,
        RACE_RANK_TITLE_COPY_RECT,
        LocalizedText::new(RACE_RANK_TITLE_KEY, "RANKINGS"),
        &assets.jeffe,
        16.0,
        [0.8, 1.0, 1.0, 1.0],
        Justify::Left,
        None,
    );
    rank_text(
        parent,
        RACE_RANK_NPC_COPY_RECT,
        race_rank_npc_name_localized(""),
        &assets.chalet,
        12.0,
        [1.0, 1.0, 0.0, 1.0],
        Justify::Left,
        Some(RaceRankSelectedPart::NpcName),
    );
    parent.spawn((
        RACE_RANK_LOCATION_BOX_RECT.node(),
        sliced(
            assets.image(RACE_RANK_LOCATION_BACK_PATH),
            BorderRect {
                min_inset: Vec2::new(10.0, 30.0),
                max_inset: Vec2::new(8.0, 30.0),
            },
        ),
        Pickable::IGNORE,
    ));
    parent.spawn((
        RACE_RANK_LOCATION_BLACK_RECT.node(),
        stretch(assets.image(RACE_RANK_BLACK_PATH)),
        Pickable::IGNORE,
    ));
    rank_text(
        parent,
        RaceUiRect::new(10.0, 150.0, 180.0, 20.0),
        LocalizedText::new(RACE_RANK_LOCATIONS_KEY, "LOCATIONS"),
        &assets.jeffe,
        14.0,
        [0.8, 1.0, 1.0, 1.0],
        Justify::Left,
        None,
    );
    for slot in 0..RACE_RANK_PAGE_SIZE {
        let row = RaceUiRect::new(
            RACE_RANK_FIRST_ROW_RECT.x,
            RACE_RANK_FIRST_ROW_RECT.y + slot as f32 * RACE_RANK_ROW_STRIDE,
            RACE_RANK_FIRST_ROW_RECT.width,
            RACE_RANK_FIRST_ROW_RECT.height,
        );
        parent
            .spawn((
                Button,
                RaceRankPresentationLocationRow(slot),
                RaceRankPresentationControl(RaceRankControl::Location(slot)),
                row.node(),
                stretch(assets.image(RACE_RANK_LOCATION_ROW_PATH)),
            ))
            .with_children(|row_parent| {
                row_parent.spawn((
                    RaceRankPresentationRowPart {
                        slot,
                        part: RaceRankRowPart::Selection,
                    },
                    RaceUiRect::new(0.0, 0.0, 460.0, 80.0).node(),
                    stretch(assets.image(RACE_RANK_LOCATION_SELECTED_PATH)),
                    Pickable::IGNORE,
                ));
                row_parent.spawn((
                    RaceRankPresentationRowPart {
                        slot,
                        part: RaceRankRowPart::Icon,
                    },
                    RaceUiRect::new(5.0, 5.0, 68.0, 68.0).node(),
                    ImageNode::default(),
                    Pickable::IGNORE,
                ));
                row_parent.spawn((
                    RaceRankPresentationRowPart {
                        slot,
                        part: RaceRankRowPart::Outline,
                    },
                    RaceUiRect::new(-3.0, -3.0, 466.0, 86.0).node(),
                    stretch(assets.image(RACE_RANK_SELECTION_OUTLINE_PATH)),
                    Pickable::IGNORE,
                ));
                row_parent.spawn((
                    RaceRankPresentationRowPart {
                        slot,
                        part: RaceRankRowPart::Point,
                    },
                    RaceUiRect::new(460.0, 30.0, 30.0, 22.0).node(),
                    stretch(assets.image(RACE_RANK_SELECTION_POINT_PATH)),
                    Pickable::IGNORE,
                ));
                row_text(
                    row_parent,
                    slot,
                    RaceRankRowPart::Name,
                    RaceUiRect::new(80.0, 17.0, 300.0, 20.0),
                    &assets.jeffe,
                    14.0,
                );
                row_text(
                    row_parent,
                    slot,
                    RaceRankRowPart::Area,
                    RaceUiRect::new(80.0, 42.0, 300.0, 20.0),
                    &assets.chalet,
                    12.0,
                );
            });
    }
    spawn_rank_control(
        parent,
        RaceRankControl::Previous,
        RACE_RANK_PREVIOUS_RECT,
        assets.image(RACE_RANK_PREVIOUS_PATH),
    );
    spawn_rank_control(
        parent,
        RaceRankControl::Next,
        RACE_RANK_NEXT_RECT,
        assets.image(RACE_RANK_NEXT_PATH),
    );
    parent.spawn((
        RACE_RANK_LEFT_ARROW_RECT.node(),
        stretch(assets.image(RACE_RANK_LEFT_ARROW_PATH)),
        Pickable::IGNORE,
    ));
    parent.spawn((
        RACE_RANK_RIGHT_ARROW_RECT.node(),
        stretch(assets.image(RACE_RANK_RIGHT_ARROW_PATH)),
        Pickable::IGNORE,
    ));
    rank_text(
        parent,
        RACE_RANK_PAGE_RECT,
        LocalizedText::new(RACE_RANK_PAGE_KEY, "{first} - {last} of {total} Locations")
            .with_arg("first", "")
            .with_arg("last", "")
            .with_arg("total", ""),
        &assets.chalet,
        12.0,
        [1.0, 1.0, 0.0, 1.0],
        Justify::Center,
        Some(RaceRankSelectedPart::Page),
    );
}

pub(super) fn spawn_race_rank_right(parent: &mut ChildSpawnerCommands, assets: &RaceRankPresentationAssets) {
    parent.spawn((
        RACE_RANK_MY_BLACK_RECT.node(),
        stretch(assets.image(RACE_RANK_BLACK_PATH)),
        Pickable::IGNORE,
    ));
    parent.spawn((
        RACE_RANK_TOP_BLACK_RECT.node(),
        stretch(assets.image(RACE_RANK_BLACK_PATH)),
        Pickable::IGNORE,
    ));
    rank_text(
        parent,
        RACE_RANK_RIGHT_LOCATION_RECT,
        LocalizedText::new(RACE_RANK_LOCATION_LABEL_KEY, "LOCATION:"),
        &assets.jeffe,
        14.0,
        [1.0, 1.0, 1.0, 1.0],
        Justify::Left,
        None,
    );
    rank_text(
        parent,
        RACE_RANK_RIGHT_NAME_RECT,
        race_rank_location_name_localized(""),
        &assets.jeffe,
        14.0,
        [1.0, 1.0, 0.0, 1.0],
        Justify::Left,
        Some(RaceRankSelectedPart::Location),
    );
    rank_text(
        parent,
        RACE_RANK_RIGHT_AREA_RECT,
        race_rank_area_name_localized(""),
        &assets.chalet,
        12.0,
        [1.0, 1.0, 1.0, 1.0],
        Justify::Left,
        Some(RaceRankSelectedPart::Area),
    );
    parent.spawn((
        RaceRankPresentationSelectedPart(RaceRankSelectedPart::BigImage),
        RACE_RANK_BIG_IMAGE_RECT.node(),
        ImageNode::default(),
        Pickable::IGNORE,
    ));
    for (rect, key, copy) in [
        (
            RACE_RANK_HEADER_RANK_RECT,
            RACE_RANK_HEADER_RANK_KEY,
            "RANK",
        ),
        (
            RACE_RANK_HEADER_PLAYER_RECT,
            RACE_RANK_HEADER_PLAYER_KEY,
            "PLAYER",
        ),
        (
            RACE_RANK_HEADER_SCORE_RECT,
            RACE_RANK_HEADER_TOP_SCORE_KEY,
            "TOP SCORE",
        ),
        (
            RaceUiRect::new(15.0, 288.0, 89.0, 20.0),
            RACE_RANK_HEADER_RANK_KEY,
            "RANK",
        ),
        (
            RaceUiRect::new(165.0, 288.0, 275.0, 20.0),
            RACE_RANK_HEADER_PLAYER_KEY,
            "PLAYER",
        ),
        (
            RaceUiRect::new(367.0, 288.0, 107.0, 20.0),
            RACE_RANK_HEADER_TOP_SCORE_KEY,
            "TOP SCORE",
        ),
    ] {
        rank_text(
            parent,
            rect,
            LocalizedText::new(key, copy),
            &assets.jeffe,
            12.0,
            [0.6, 0.85, 0.9, 1.0],
            Justify::Center,
            None,
        );
    }
    rank_text(
        parent,
        RaceUiRect::new(105.0, 225.0, 270.0, 20.0),
        race_rank_score_localized(RaceRankScoreColumn::Player, ""),
        &assets.chalet,
        12.0,
        [1.0, 1.0, 0.0, 1.0],
        Justify::Left,
        None,
    )
    .insert(RaceRankPresentationPersonalText(
        RaceRankScoreColumn::Player,
    ));
    rank_text(
        parent,
        RaceUiRect::new(375.0, 225.0, 100.0, 20.0),
        race_rank_score_localized(RaceRankScoreColumn::Score, ""),
        &assets.chalet,
        12.0,
        [1.0, 1.0, 0.0, 1.0],
        Justify::Left,
        None,
    )
    .insert(RaceRankPresentationPersonalText(RaceRankScoreColumn::Score));
    rank_text(
        parent,
        RACE_RANK_WARNING_RECT,
        LocalizedText::new(RACE_RANK_NO_SCORE_KEY, "No score registered yet."),
        &assets.chalet,
        12.0,
        [1.0, 1.0, 0.0, 1.0],
        Justify::Center,
        Some(RaceRankSelectedPart::NoScore),
    );
    for row in 0..RACE_RANK_TOP_COUNT {
        let y = 320.0 + row as f32 * 30.0;
        parent.spawn((
            RaceRankPresentationScoreRow(row),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(y),
                width: px(472),
                height: px(30),
                ..default()
            },
            Pickable::IGNORE,
        ));
        for (column, rect, justify) in [
            (
                RaceRankScoreColumn::Rank,
                RaceUiRect::new(35.0, y, 70.0, 20.0),
                Justify::Left,
            ),
            (
                RaceRankScoreColumn::Player,
                RaceUiRect::new(105.0, y, 270.0, 20.0),
                Justify::Left,
            ),
            (
                RaceRankScoreColumn::Score,
                RaceUiRect::new(375.0, y, 95.0, 20.0),
                Justify::Left,
            ),
        ] {
            rank_text(
                parent,
                rect,
                race_rank_score_localized(column, ""),
                &assets.chalet,
                12.0,
                [1.0, 1.0, 0.0, 1.0],
                justify,
                None,
            )
            .insert(RaceRankPresentationTopText { row, column });
        }
    }
    parent.spawn((
        RaceRankPresentationSelectedPart(RaceRankSelectedPart::Highlight),
        RACE_RANK_HIGHLIGHT_RECT.node(),
        stretch(assets.image(RACE_RANK_HIGHLIGHT_PATH)),
        Pickable::IGNORE,
        ZIndex(1),
    ));
    parent
        .spawn((RACE_RANK_TAB_GROUP_RECT.node(), Pickable::IGNORE))
        .with_children(|tabs| {
            tabs.spawn((
                RACE_RANK_TAB_BAR_RECT.node(),
                stretch(assets.image(RACE_RANK_TAB_BAR_PATH)),
                Pickable::IGNORE,
            ));
            tabs.spawn((
                RACE_RANK_TAB_BOX_RECT.node(),
                stretch(assets.image(RACE_RANK_TAB_BOX_PATH)),
                Pickable::IGNORE,
            ));
            tabs.spawn((
                RACE_RANK_MY_SKY_RECT.node(),
                stretch(assets.image(RACE_RANK_SKY_PATH)),
                Pickable::IGNORE,
            ));
            tabs.spawn((
                RACE_RANK_TOP_SKY_RECT.node(),
                stretch(assets.image(RACE_RANK_SKY_PATH)),
                Pickable::IGNORE,
            ));
            for (period, visual, hit) in [
                (
                    RaceRankPeriod::Today,
                    RACE_RANK_TODAY_VISUAL_RECT,
                    RACE_RANK_TODAY_HIT_RECT,
                ),
                (
                    RaceRankPeriod::Week,
                    RACE_RANK_WEEK_VISUAL_RECT,
                    RACE_RANK_WEEK_HIT_RECT,
                ),
                (
                    RaceRankPeriod::Month,
                    RACE_RANK_MONTH_VISUAL_RECT,
                    RACE_RANK_MONTH_HIT_RECT,
                ),
                (
                    RaceRankPeriod::AllTime,
                    RACE_RANK_ALL_VISUAL_RECT,
                    RACE_RANK_ALL_HIT_RECT,
                ),
            ] {
                tabs.spawn((
                    RaceRankPresentationTabVisual(period),
                    visual.node(),
                    stretch(assets.image(if period == RaceRankPeriod::Today {
                        RACE_RANK_TODAY_IDLE_PATH
                    } else {
                        RACE_RANK_WIDE_IDLE_PATH
                    })),
                    Pickable::IGNORE,
                ));
                tabs.spawn((
                    Button,
                    RaceRankPresentationControl(RaceRankControl::Period(period)),
                    hit.node(),
                ));
                rank_text(
                    tabs,
                    hit,
                    period.tab_localized(),
                    &assets.chalet,
                    12.0,
                    [1.0, 1.0, 1.0, 1.0],
                    Justify::Center,
                    None,
                );
            }
        });
    rank_text(
        parent,
        RACE_RANK_MY_BEST_RECT,
        RaceRankPeriod::Today.best_localized(),
        &assets.jeffe,
        14.0,
        [0.8, 1.0, 1.0, 1.0],
        Justify::Left,
        Some(RaceRankSelectedPart::BestCopy),
    );
    rank_text(
        parent,
        RACE_RANK_TOP_COPY_RECT,
        RaceRankPeriod::Today.top_localized(),
        &assets.jeffe,
        14.0,
        [0.8, 1.0, 1.0, 1.0],
        Justify::Left,
        Some(RaceRankSelectedPart::TopCopy),
    );
}

pub(super) fn spawn_rank_control(
    parent: &mut ChildSpawnerCommands,
    control: RaceRankControl,
    rect: RaceUiRect,
    image: Handle<Image>,
) {
    parent.spawn((
        Button,
        RaceRankPresentationControl(control),
        rect.node(),
        stretch(image),
    ));
}
