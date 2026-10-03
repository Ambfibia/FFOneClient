use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialSceneEventProvenance {
    pub row_index: usize,
    pub event: i32,
    pub declared_element_count: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialSceneEventDefinition {
    pub provenance: TutorialSceneEventProvenance,
    pub(super) texts: BTreeMap<i32, TutorialSceneTextDefinition>,
}

impl TutorialSceneEventDefinition {
    pub fn text(&self, line: i32) -> TutorialMissionContentResult<&TutorialSceneTextDefinition> {
        self.texts
            .get(&line)
            .ok_or(TutorialMissionContentError::UnknownSceneText {
                event: self.provenance.event,
                line,
            })
    }

    pub fn texts(&self) -> impl ExactSizeIterator<Item = &TutorialSceneTextDefinition> {
        self.texts.values()
    }
}
