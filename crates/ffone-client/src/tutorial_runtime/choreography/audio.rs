use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialAudioTarget {
    None,
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialSoundAction {
    UiOneShot {
        cue: &'static str,
    },
    WorldOneShot {
        cue: &'static str,
        position: ClientVec3,
    },
    Ambient {
        cue: &'static str,
    },
}
