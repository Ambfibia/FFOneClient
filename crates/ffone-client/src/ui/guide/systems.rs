use super::*;

#[allow(clippy::type_complexity)]
pub(super) fn sync_guide_ui_confirmation(
    model: Res<GuideUiModel>,
    assets: Res<GuideUiAssets>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    mut elements: Query<
        (
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut LocalizedText>,
            Option<&GuideUiConfirmationArt>,
            Option<&GuideUiConfirmationTitle>,
            Option<&GuideUiConfirmationBody>,
        ),
        Or<(
            With<GuideUiConfirmationArt>,
            With<GuideUiConfirmationTitle>,
            With<GuideUiConfirmationBody>,
        )>,
    >,
) {
    let confirmation = guide_confirmation_view(&model);
    for (mut node, mut image, mut localized, art, title, body) in &mut elements {
        if art.is_some() {
            if let Some(confirmation) = confirmation.as_ref() {
                apply_rect(&mut node, confirmation.art_rect);
                if let Some(image) = image.as_deref_mut() {
                    image.image = assets.confirmation_art[confirmation.mentor.slot()].clone();
                }
            }
        } else if title.is_some() {
            if let (Some(_), Some(localized)) = (confirmation.as_ref(), localized.as_deref_mut()) {
                set_localized_text(
                    localized,
                    guide_confirmation_title_localized_text(model.purpose),
                );
            }
        } else if body.is_some()
            && let (Some(confirmation), Some(localized)) =
                (confirmation.as_ref(), localized.as_deref_mut())
        {
            let mentor_name = guide_localized_mentor_name(
                confirmation.mentor,
                localization.as_deref(),
                language.as_deref(),
            );
            set_localized_text(
                localized,
                guide_confirmation_body_localized_text_with_name(model.purpose, mentor_name),
            );
        }
    }
}
