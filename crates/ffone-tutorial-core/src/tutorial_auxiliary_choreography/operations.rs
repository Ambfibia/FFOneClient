use super::*;

#[must_use]
pub fn tutorial_auxiliary_definition(
    sequence: TutorialAuxiliarySequence,
) -> &'static TutorialAuxiliaryDefinition {
    TUTORIAL_AUXILIARY_DEFINITIONS
        .iter()
        .find(|definition| definition.sequence == sequence)
        .expect("every TutorialAuxiliarySequence must have an exact definition")
}
