use super::*;

pub(super) fn advance_race_rank_slide(time: Res<Time>, mut model: ResMut<RaceRankModel>) {
    model.advance_slide(time.delta_secs());
}

pub(super) fn sync_race_rank_rows(
    model: Res<RaceRankModel>,
    catalog: Res<RaceRankCatalog>,
    assets: Res<RaceRankPresentationAssets>,
    mut rows: Query<
        (&RaceRankPresentationLocationRow, &mut Node, &mut Pickable),
        Without<RaceRankPresentationRowPart>,
    >,
    mut parts: Query<
        (
            &RaceRankPresentationRowPart,
            &mut Node,
            Option<&mut LocalizedText>,
            Option<&mut ImageNode>,
        ),
        Without<RaceRankPresentationLocationRow>,
    >,
) {
    let enabled = model.phase() == RaceRankPhase::Browsing;
    for (row, mut node, mut pickable) in &mut rows {
        let index = model.current_page() * RACE_RANK_PAGE_SIZE + row.0;
        let exists = index < catalog.locations().len();
        node.display = if exists { Display::Flex } else { Display::None };
        *pickable = if exists && enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
    for (part, mut node, localized, image) in &mut parts {
        let index = model.current_page() * RACE_RANK_PAGE_SIZE + part.slot;
        let location = catalog.locations().get(index);
        let selected = model.selected_index() == Some(index);
        node.display = if location.is_some()
            && match part.part {
                RaceRankRowPart::Selection | RaceRankRowPart::Outline | RaceRankRowPart::Point => {
                    selected
                }
                _ => true,
            } {
            Display::Flex
        } else {
            Display::None
        };
        if let (Some(location), Some(mut component)) = (location, localized)
            && matches!(part.part, RaceRankRowPart::Name | RaceRankRowPart::Area)
        {
            let localized = race_rank_row_localized(
                part.part,
                match part.part {
                    RaceRankRowPart::Name => &location.name,
                    RaceRankRowPart::Area => &location.area_name,
                    RaceRankRowPart::Icon
                    | RaceRankRowPart::Selection
                    | RaceRankRowPart::Outline
                    | RaceRankRowPart::Point => unreachable!("non-text row part"),
                },
            );
            *component = localized;
        }
        if let (Some(location), Some(mut image)) = (location, image) {
            if part.part == RaceRankRowPart::Icon {
                image.image = assets.image(&location.small_image);
                image.image_mode = NodeImageMode::Stretch;
            }
        }
    }
}

pub(super) fn sync_race_rank_scores(
    model: Res<RaceRankModel>,
    mut personal: Query<
        (&RaceRankPresentationPersonalText, &mut LocalizedText),
        Without<RaceRankPresentationTopText>,
    >,
    mut top: Query<
        (
            &RaceRankPresentationTopText,
            &mut LocalizedText,
            &mut TextColor,
        ),
        Without<RaceRankPresentationPersonalText>,
    >,
) {
    let period = model.period() as usize;
    let highlighted = model.highlighted_top_index();
    for (column, mut component) in &mut personal {
        let value = model.scores().personal[period]
            .as_ref()
            .map_or_else(String::new, |score| match column.0 {
                RaceRankScoreColumn::Rank => score.rank.to_string(),
                RaceRankScoreColumn::Player => score.player.clone(),
                RaceRankScoreColumn::Score => score.score.to_string(),
            });
        *component = race_rank_score_localized(column.0, &value);
    }
    let visible_top_count = model.scores().top[period]
        .iter()
        .position(Option::is_none)
        .unwrap_or(RACE_RANK_TOP_COUNT);
    for (role, mut component, mut text_color) in &mut top {
        // Clean redraws the highlighted row with FusionFallRank's
        // `righthighlabel` style over the yellow `rankhighlight` texture.
        // Its serialized normal color is exact dark blue (0, 0, 0.14919356).
        *text_color = TextColor(if highlighted == Some(role.row) {
            Color::srgb(0.0, 0.0, 0.149_193_56)
        } else {
            Color::srgb(1.0, 1.0, 0.0)
        });
        let value = if role.row < visible_top_count {
            model.scores().top[period][role.row]
                .as_ref()
                .map_or_else(String::new, |score| match role.column {
                    RaceRankScoreColumn::Rank => score.rank.to_string(),
                    RaceRankScoreColumn::Player => score.player.clone(),
                    RaceRankScoreColumn::Score => score.score.to_string(),
                })
        } else {
            String::new()
        };
        *component = race_rank_score_localized(role.column, &value);
    }
}

pub(super) fn sync_race_rank_tabs(
    model: Res<RaceRankModel>,
    assets: Res<RaceRankPresentationAssets>,
    controls: Query<(&RaceRankPresentationControl, &Interaction)>,
    mut visuals: Query<(&RaceRankPresentationTabVisual, &mut ImageNode)>,
) {
    for (visual, mut image) in &mut visuals {
        let hovered = controls.iter().any(|(control, interaction)| {
            control.0 == RaceRankControl::Period(visual.0)
                && matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        });
        image.image = assets.image(if visual.0 == RaceRankPeriod::Today {
            if model.period() == visual.0 {
                RACE_RANK_TODAY_ACTIVE_PATH
            } else if hovered {
                RACE_RANK_TODAY_HOVER_PATH
            } else {
                RACE_RANK_TODAY_IDLE_PATH
            }
        } else if model.period() == visual.0 {
            RACE_RANK_WIDE_ACTIVE_PATH
        } else if hovered {
            RACE_RANK_WIDE_HOVER_PATH
        } else {
            RACE_RANK_WIDE_IDLE_PATH
        });
    }
}

pub(super) fn sync_race_rank_control_visuals(
    model: Res<RaceRankModel>,
    input: Res<RaceRankPresentationInput>,
    assets: Res<RaceRankPresentationAssets>,
    mut controls: Query<(
        &RaceRankPresentationControl,
        &Interaction,
        &mut ImageNode,
        &mut Pickable,
    )>,
) {
    let enabled = model.controls_enabled(input.system_popup_active);
    for (control, interaction, mut image, mut pickable) in &mut controls {
        *pickable = if enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
        let hovered =
            enabled && matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        let path = match control.0 {
            RaceRankControl::Previous => Some(if hovered {
                RACE_RANK_PREVIOUS_HOVER_PATH
            } else {
                RACE_RANK_PREVIOUS_PATH
            }),
            RaceRankControl::Next => Some(if hovered {
                RACE_RANK_NEXT_HOVER_PATH
            } else {
                RACE_RANK_NEXT_PATH
            }),
            RaceRankControl::Close => Some(if hovered {
                RACE_RANK_CLOSE_HOVER_PATH
            } else {
                RACE_RANK_CLOSE_PATH
            }),
            RaceRankControl::HelpIgnored => Some(if hovered {
                RACE_RANK_HELP_HOVER_PATH
            } else {
                RACE_RANK_HELP_PATH
            }),
            RaceRankControl::Location(_) | RaceRankControl::Period(_) => None,
        };
        if let Some(path) = path {
            image.image = assets.image(path);
        }
    }
}
