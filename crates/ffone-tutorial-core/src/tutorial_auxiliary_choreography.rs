//! Exact engine-independent data for the eleven tutorial coroutines
//! which sit around the ten large cinematic coroutines.
//!
//! Source of truth:
//! `work/ilspy-retrobution-csharp/cntutorialscript.cs` from
//! `builds/retrobution-20260613`.
//!
//! Keeping these routines typed matters: the legacy client interleaves them
//! with chapter state, mission UI and the large cutscenes. Treating them as
//! prose or auto-completing their prompts silently drops voice lines, cursor
//! hints, waypoint state and two intentional 25-second reminder loops.



#[cfg(test)]
mod tests;

mod types;
mod commands;
mod constants;
mod operations;
mod audio;

pub use types::{
    TutorialSourceSpan, TutorialAuxiliarySequence, TutorialText, TutorialSubtitleChannel,
    TutorialCursorDirection, TutorialScreenAxis, TutorialScreenPoint, TutorialScreenPivot,
    TutorialWaypointTarget, TutorialBooleanFlag, TutorialRepeatBlock,
    TutorialAuxiliaryDefinition, TutorialInitialNpc, TutorialInitializationContract,
    TutorialAuxiliaryEmissionOrigin, TutorialAuxiliaryEmission, TutorialAuxiliaryPlayer
};
pub use commands::{TutorialAuxiliaryAction, TimedTutorialAuxiliaryAction};
pub use constants::{TUTORIAL_AUXILIARY_DEFINITIONS, TUTORIAL_INITIALIZATION};
#[cfg(test)]
use constants::{MOVE_HINT_POSITION, TALK_CURSOR_POSITION, MISSION_CURSOR_POSITION};
pub use operations::tutorial_auxiliary_definition;
pub use audio::{TutorialAudioCrossfadeContract, TUTORIAL_AUDIO_CROSSFADE};
