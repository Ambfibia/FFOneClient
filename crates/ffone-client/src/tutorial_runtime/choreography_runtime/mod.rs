//! Deterministic runtime for the recovered tutorial choreography.
//!
//! The reference tables in [`crate::tutorial_choreography`] deliberately do
//! not depend on Bevy. This module owns the live scene cursor, expands the
//! fixed-frame legacy loops, applies skip contracts in source order, and keeps
//! presentation state which the native client can resolve against live
//! entities.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, VecDeque},
};

use bevy::prelude::*;

use crate::{
    tutorial::TutorialScene,
    tutorial_choreography::{
        BlockingWait, CameraAction, CameraMode, ChoreographyAction, ClientVec3, EquipmentAction,
        FadeChannel, FrameSequence, HudAction, LoopAction, NanoAction, PanAction, PlayerAction,
        PositionExpr, ProjectileAction, RotationExpr, SkipFinalFade, SpatialAudioTarget,
        TutorialSceneChoreography, tutorial_scene_choreography,
    },
    tutorial_choreography_formula::ChoreographyFormulaError,
    tutorial_cinematic_title::TUTORIAL_CINEMATIC_BAR_HEIGHT_FRACTION,
};

#[cfg(test)]
mod tests;

mod constants;
mod commands;
mod types_tutorial_choreography_player;
mod types_tutorial_choreography_presentation;
mod codec;
mod animation;
mod operations;
mod state;
mod entities;
mod systems;

use constants::TIME_EPSILON;
pub use constants::TUTORIAL_PAN_PATHS;
pub use commands::{ChoreographyActionOrigin, ChoreographyPlaybackEvent};
use commands::{ScheduledEvent, unavailable_action_issue};
pub use types_tutorial_choreography_player::{
    ChoreographyCompletion, TutorialChoreographyIssue, TutorialChoreographyIssueQueue,
    TutorialLoopOwner, TutorialChoreographyPlayer, TutorialCameraPresentation,
    TutorialChoreographyPresentation
};
use types_tutorial_choreography_player::ActivePlayback;
pub use types_tutorial_choreography_presentation::TutorialChoreographyOverlay;
use types_tutorial_choreography_presentation::TutorialPanAssets;
pub use codec::FrameSequenceSample;
use codec::{sequence_frame_count, sequence_frame_delay, TutorialPanFrame};
pub use animation::RigAnimationTarget;
use operations::{
    blocking_wait_start, blocking_wait_continuation_source_line, next_blocking_wait,
    sample_sequence, client_vec3
};
pub use state::TutorialChoreographyRuntimePlugin;
use entities::spawn_tutorial_choreography_overlay;
use systems::{sync_tutorial_choreography_overlay, sync_tutorial_pan_strip};
