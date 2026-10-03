//! Reference choreography for every finite Retrobution tutorial coroutine.
//!
//! Source of truth:
//! `work/ilspy-retrobution-csharp/cntutorialscript.cs`.
//!
//! The data is deliberately renderer- and ECS-independent. `at_seconds` is the
//! cumulative sum of `WaitForSeconds` and fixed-count loop waits preceding an
//! action. Asset completion and bounded effect-instantiation retry waits are
//! represented separately, so a runtime cannot accidentally advance through a
//! reference blocking point by treating it as a zero-duration action.

use crate::tutorial::TutorialScene;

#[cfg(test)]
mod tests;

mod types;
mod state;
mod localization;
mod animation;
mod commands;
mod codec;
mod audio;
mod operations;
mod constants_basic_move_spawns;
mod constants_basic_move_actions;
mod constants_basic_combat_a_actions;
mod constants_basic_combat_b_actions;
mod constants_basic_combat_c_actions;
mod constants_infection_b_actions;
mod constants_infection_c_actions;
mod constants_nano_power_b_actions;
mod constants_tutorial_scene_choreographies;

pub use types::{
    SourceSpan, ClientVec3, LegacyServerPosition, EntityRef, CameraCaptureSlot,
    PositionAnchor, OrientationBasis, PositionExpr, RotationExpr, FadeChannel, FadeSequence,
    NpcSpawn, EffectSpawn, CameraShakeWave, TutorialCursorDirection, TutorialScreenAxis,
    TutorialScreenPoint, BlockingWait, SkipFinalFade, SkipContract, LoopLifetime,
    TutorialSceneChoreography
};
pub use state::CameraMode;
pub use localization::EffectLocaleGate;
pub use animation::EffectBoneAttachment;
pub use commands::{
    NpcCommand, NpcAction, PlayerAction, CameraAction, EffectAction, HudAction, LoopAction,
    NanoAction, EquipmentAction, ProgressAction, ProjectileAction, PanAction,
    TutorialPictureAction, UnresolvedAction, ChoreographyAction, TimedAction, SourcedAction
};
pub use codec::FrameSequence;
pub use audio::{SpatialAudioTarget, TutorialSoundAction};
use operations::{at, oriented, skip, spawn, effect, skip_contract};
pub use operations::tutorial_scene_choreography;
use constants_basic_move_spawns::{
    FADE_SCREEN_IN, FADE_SCREEN_OUT, FADE_OVERLAY_OUT, FADE_OVERLAY_IN, SHAKE_START_WAVE,
    SHAKE_TARGET_WAVE, BASIC_MOVE_SPAWNS, BASIC_MOVE_EXPLOSIONS
};
use constants_basic_move_actions::BASIC_MOVE_ACTIONS;
use constants_basic_combat_a_actions::{BASIC_COMBAT_A_ACTIONS, BASIC_COMBAT_A_WAITS};
use constants_basic_combat_b_actions::BASIC_COMBAT_B_ACTIONS;
use constants_basic_combat_c_actions::{BASIC_COMBAT_C_ACTIONS, INFECTION_A_ACTIONS};
use constants_infection_b_actions::INFECTION_B_ACTIONS;
use constants_infection_c_actions::{INFECTION_C_ACTIONS, NANO_POWER_A_ACTIONS};
use constants_nano_power_b_actions::{NANO_POWER_A2_ACTIONS, NANO_POWER_B_ACTIONS};
use constants_tutorial_scene_choreographies::COMMON_SKIP_CLEANUP;
pub use constants_tutorial_scene_choreographies::TUTORIAL_SCENE_CHOREOGRAPHIES;
