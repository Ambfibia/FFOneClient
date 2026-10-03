use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) enum TutorialMovementStageEffect {
    Preload {
        source_line: u32,
    },
    Clear {
        source_line: u32,
    },
    Marker {
        unity_position: Vec3,
        source_line: u32,
    },
}
