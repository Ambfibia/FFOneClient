use super::*;

pub(super) fn section<'a>(body: &'a str, name: &'static str) -> Result<&'a str, RaceRankParseError> {
    let open = format!("<{name}");
    let close = format!("</{name}");
    let start = body
        .find(&open)
        .ok_or(RaceRankParseError::MissingSection(name))?;
    let content_start = body[start..]
        .find('>')
        .map(|offset| start + offset + 1)
        .ok_or(RaceRankParseError::MissingSection(name))?;
    let content_end = body[content_start..]
        .find(&close)
        .map(|offset| content_start + offset)
        .ok_or(RaceRankParseError::MissingSection(name))?;
    Ok(&body[content_start..content_end])
}

pub(super) fn score_tags(section: &str) -> Result<Vec<RaceRankScore>, RaceRankParseError> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = find_score_open(&section[cursor..]) {
        let start = cursor + relative;
        let end = section[start..]
            .find('>')
            .map(|offset| start + offset + 1)
            .ok_or(RaceRankParseError::UnterminatedScore)?;
        result.push(parse_score_tag(&section[start..end])?);
        cursor = end;
    }
    Ok(result)
}

pub(super) fn ordered_attribute<'a>(
    tag: &'a str,
    name: &'static str,
    previous: &'static str,
    cursor: &mut usize,
) -> Result<&'a str, RaceRankParseError> {
    let pattern = format!("{name}=\"");
    let Some(relative) = tag[*cursor..].find(&pattern) else {
        if tag[..(*cursor).min(tag.len())].contains(&pattern) {
            return Err(RaceRankParseError::WrongAttributeOrder {
                previous,
                current: name,
            });
        }
        return Err(RaceRankParseError::MissingAttribute(name));
    };
    let start = *cursor + relative + pattern.len();
    let end = tag[start..]
        .find('"')
        .map(|offset| start + offset)
        .ok_or(RaceRankParseError::MissingAttribute(name))?;
    *cursor = end + 1;
    Ok(&tag[start..end])
}

pub(super) fn row_text(
    parent: &mut ChildSpawnerCommands,
    slot: usize,
    part: RaceRankRowPart,
    rect: RaceUiRect,
    font: &Handle<Font>,
    size: f32,
) {
    let localized = match part {
        RaceRankRowPart::Name => {
            LocalizedText::new(RACE_RANK_ROW_NAME_KEY, "{location}").with_arg("location", "")
        }
        RaceRankRowPart::Area => {
            LocalizedText::new(RACE_RANK_ROW_AREA_KEY, "{area}").with_arg("area", "")
        }
        RaceRankRowPart::Icon
        | RaceRankRowPart::Selection
        | RaceRankRowPart::Outline
        | RaceRankRowPart::Point => unreachable!("only text row parts reach row_text"),
    };
    parent.spawn((
        RaceRankPresentationRowPart { slot, part },
        Text::new(localized_fallback(&localized)),
        localized,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (size).into(),
                ..default()
            },
            LineHeight::Px(size),
        ),
        TextColor(Color::WHITE),
        TextLayout::new(Justify::Left, LineBreak::NoWrap),
        rect.node(),
        Pickable::IGNORE,
        ZIndex(2),
    ));
}

pub(super) fn rank_text<'a>(
    parent: &'a mut ChildSpawnerCommands,
    rect: RaceUiRect,
    localized: LocalizedText,
    font: &Handle<Font>,
    size: f32,
    tint: [f32; 4],
    justify: Justify,
    selected_part: Option<RaceRankSelectedPart>,
) -> EntityCommands<'a> {
    let mut entity = parent.spawn((
        Text::new(localized_fallback(&localized)),
        localized,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (size).into(),
                ..default()
            },
            LineHeight::Px(size),
        ),
        TextColor(color(tint)),
        TextLayout::new(justify, LineBreak::NoWrap),
        rect.node(),
        Pickable::IGNORE,
        ZIndex(2),
    ));
    if let Some(part) = selected_part {
        entity.insert(RaceRankPresentationSelectedPart(part));
    }
    entity
}

pub(super) fn queue_race_rank_controls(
    model: Res<RaceRankModel>,
    input: Res<RaceRankPresentationInput>,
    controls: Query<(&Interaction, &RaceRankPresentationControl), Changed<Interaction>>,
    mut outbox: ResMut<RaceRankUiCommandOutbox>,
) {
    if !model.controls_enabled(input.system_popup_active) {
        return;
    }
    for (interaction, control) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let command = match control.0 {
            RaceRankControl::Location(slot) => RaceRankUiCommand::SelectLocationSlot(slot + 1),
            RaceRankControl::Previous => RaceRankUiCommand::PreviousPage,
            RaceRankControl::Next => RaceRankUiCommand::NextPage,
            RaceRankControl::Period(period) => RaceRankUiCommand::SelectPeriod(period),
            RaceRankControl::Close => RaceRankUiCommand::Close,
            // Exact clean behavior: GUI.Button return is ignored.
            RaceRankControl::HelpIgnored => continue,
        };
        outbox.push(command);
    }
}
