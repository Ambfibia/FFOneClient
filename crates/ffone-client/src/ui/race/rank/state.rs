use super::*;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationSelectedPart(pub(super) RaceRankSelectedPart);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RaceRankSelectedPart {
    BigImage,
    Location,
    Area,
    NoScore,
    Highlight,
    BestCopy,
    TopCopy,
    Page,
    NpcName,
}

pub(super) fn sync_race_rank_selected(
    model: Res<RaceRankModel>,
    catalog: Res<RaceRankCatalog>,
    assets: Res<RaceRankPresentationAssets>,
    mut selected: Query<(
        &RaceRankPresentationSelectedPart,
        &mut Node,
        Option<&mut LocalizedText>,
        Option<&mut ImageNode>,
    )>,
) {
    let location = model.selected_location(&catalog);
    let period = model.period();
    let has_personal = model.scores().personal[period as usize].is_some();
    for (part, mut node, localized, image) in &mut selected {
        match part.0 {
            RaceRankSelectedPart::BigImage => {
                node.display = if location.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                if let (Some(location), Some(mut image)) = (location, image) {
                    image.image = assets.image(&location.big_image);
                    image.image_mode = NodeImageMode::Stretch;
                }
            }
            RaceRankSelectedPart::Highlight => {
                if let Some(index) = model.highlighted_top_index() {
                    node.display = Display::Flex;
                    node.top = px(RACE_RANK_HIGHLIGHT_RECT.y + index as f32 * 30.0);
                } else {
                    node.display = Display::None;
                }
            }
            _ => {
                node.display = if part.0 == RaceRankSelectedPart::NoScore
                    && !(location.is_some() && !has_personal)
                {
                    Display::None
                } else {
                    Display::Flex
                };
                if let Some(mut component) = localized {
                    let localized = match part.0 {
                        RaceRankSelectedPart::Location => race_rank_location_name_localized(
                            location.map_or("", |value| value.name.as_str()),
                        ),
                        RaceRankSelectedPart::Area => race_rank_area_name_localized(
                            location.map_or("", |value| value.area_name.as_str()),
                        ),
                        RaceRankSelectedPart::NoScore => {
                            LocalizedText::new(RACE_RANK_NO_SCORE_KEY, "No score registered yet.")
                        }
                        RaceRankSelectedPart::BestCopy => period.best_localized(),
                        RaceRankSelectedPart::TopCopy => period.top_localized(),
                        RaceRankSelectedPart::Page => {
                            model.page_localized(catalog.locations().len())
                        }
                        RaceRankSelectedPart::NpcName => {
                            race_rank_npc_name_localized(model.npc_name())
                        }
                        RaceRankSelectedPart::BigImage | RaceRankSelectedPart::Highlight => {
                            unreachable!("image-only selected part")
                        }
                    };
                    *component = localized;
                }
            }
        }
    }
}
