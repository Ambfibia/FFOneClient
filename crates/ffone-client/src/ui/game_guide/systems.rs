use super::*;

pub(super) fn sync_game_guide_labels(
    model: Res<GameGuideUiModel>,
    catalog: Res<GameGuideCatalog>,
    mut main_labels: Query<(&GameGuideMainLabel, &mut LocalizedText, &mut Node)>,
    mut sub_labels: Query<
        (&GameGuideSubLabel, &mut LocalizedText, &mut Node),
        Without<GameGuideMainLabel>,
    >,
    mut buttons: Query<
        (&GameGuideControl, &mut Node),
        (Without<GameGuideMainLabel>, Without<GameGuideSubLabel>),
    >,
    mut heading: Query<
        &mut LocalizedText,
        (
            With<GameGuideSelectedHeading>,
            Without<GameGuideMainLabel>,
            Without<GameGuideSubLabel>,
        ),
    >,
) {
    if !model.visible {
        return;
    }
    for (control, mut node) in &mut buttons {
        let visible = match control {
            GameGuideControl::Main(index) => catalog.topics.get(*index).is_some(),
            GameGuideControl::Sub(index) => catalog
                .topics
                .get(model.selected_main_topic)
                .is_some_and(|range| {
                    range
                        .start
                        .checked_add(*index)
                        .is_some_and(|id| id <= range.end)
                }),
            _ => continue,
        };
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (label, mut localized, mut node) in &mut main_labels {
        let Some(range) = catalog.topics.get(label.0) else {
            node.display = Display::None;
            continue;
        };
        node.display = Display::Flex;
        let fallback = catalog
            .help_names
            .get(range.start)
            .cloned()
            .unwrap_or_default();
        *localized = LocalizedText::new(
            format!(
                "content.tabledata.help.help_string.{}.str_name",
                range.start
            ),
            fallback,
        );
    }
    let Some(selected_range) = catalog.topics.get(model.selected_main_topic) else {
        return;
    };
    if let Ok(mut localized) = heading.single_mut() {
        let fallback = catalog
            .help_names
            .get(selected_range.start)
            .cloned()
            .unwrap_or_default();
        *localized = LocalizedText::new(
            format!(
                "content.tabledata.help.help_string.{}.str_name",
                selected_range.start
            ),
            fallback,
        );
    }
    for (label, mut localized, mut node) in &mut sub_labels {
        let string_id = selected_range.start + label.0;
        if string_id > selected_range.end {
            node.display = Display::None;
            continue;
        }
        node.display = Display::Flex;
        let fallback = catalog
            .help_comments
            .get(string_id)
            .cloned()
            .unwrap_or_default();
        *localized = LocalizedText::new(
            format!("content.tabledata.help.help_string.{string_id}.str_comment"),
            fallback,
        );
    }
}
