use super::*;

pub(in super::super) fn localized_tutorial_auxiliary_text(
    text: TutorialText,
    content: &TutorialMissionContent,
) -> Option<LocalizedText> {
    match text {
        TutorialText::SceneText { group, index } => {
            let event = i32::from(group);
            let line = i32::from(index);
            let fallback = content.scene_text(event, line).ok()?;
            Some(localized_tutorial_scene_text(event, line, fallback))
        }
        TutorialText::Localized(source) => Some(localized_tutorial_instruction(source)),
        TutorialText::Literal(source) => Some(localized_tutorial_literal(source)),
    }
}
