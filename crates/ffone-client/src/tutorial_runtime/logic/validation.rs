
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialLogicError {
    UnknownLegacyStage { chapter: u8, step: i16 },
    MovementChapterOutsideScope,
}
