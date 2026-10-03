use super::*;

pub(in super::super) fn resolve_tutorial_auxiliary_text(
    text: TutorialText,
    content: &TutorialMissionContent,
    localization: &Localization,
    language: &Language,
) -> Option<String> {
    let localized = localized_tutorial_auxiliary_text(text, content)?;
    Some(localization.text(language, &localized))
}
