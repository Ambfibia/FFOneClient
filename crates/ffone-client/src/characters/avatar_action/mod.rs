//! Native scheduling for the local avatar's legacy actions and animation names.
//!
//! This module is based on the effective `b8c3` decompile of
//! `cnAvatarAttack`, `cnAvatarThirdPersonMove`, `cnAvatarAnimation`, and
//! `ConfigurableInput`. It deliberately emits gameplay intents instead of
//! constructing packets. It also never fabricates animation assets: exact
//! legacy clip names are paired with an optional native asset path, and an
//! absent mapping remains [`LegacyClipResolution::Unconnected`].
//!
//! [`LegacyAvatarActionState`] is the sole local-player animation state
//! machine. It is gender-neutral; renderer adapters resolve its semantic clips
//! against the selected male or female rig and only report real completion
//! events back to this owner.

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use crate::coordinates::LegacyUnityHeadingDegrees;
use crate::legacy_environment::LegacyAvatarEnvironmentState;
use crate::movement::{
    LegacyMovementSet, LegacyOrbitCamera, LegacyPlayerController, LegacyWorldColliderPending,
};
use crate::world::NativeWorldSet;

#[cfg(test)]
mod tests;

mod materials;
mod constants;
mod commands;
mod state;
mod types;
mod operations;
mod animation;
mod codec;
mod systems;
mod input;

pub use materials::{LEGACY_ANIMATION_BLEND_SECONDS, LEGACY_END_ANIMATION_BLEND_SECONDS};
pub use constants::{LEGACY_ATTACK_DELAY_SECONDS, LEGACY_ATTACK_ACCUMULATION_LIMIT};
pub use commands::{
    LegacyAvatarActionInput, LegacyAvatarActionContext, LegacyAvatarActionIntent,
    LegacyAvatarActionRequest, LegacyAvatarActionQueue, LegacyVisualCommand,
    LegacyVisualRequest, LegacyVisualRequestQueue, LegacyAvatarActionSet,
    LegacyAvatarActionPlugin
};
use commands::action_input_from_devices;
pub use state::{
    LegacyMoveMode, LegacyWeaponTargetMode, LegacyTargetSelection, LegacyLocomotionState,
    LegacyAvatarActionState
};
pub use types::{
    LegacyAvatarTraversalPresentation, LegacyVehiclePresentationFamily,
    LegacyAvatarPresentationContext, LegacyNanoTargetPolicy, LegacyTargetKind,
    LegacyTargetSample, LegacyTriggerSample, LegacyAvatarTargetFeed, LegacyFocusedTarget,
    LegacyAttackTarget, LegacyVisualCompletion, LegacyVisualCompletionQueue
};
use types::LegacyAvatarDeathPhase;
use operations::{
    cross_fade, legacy_presentation_transition_seconds, cross_fade_with_duration,
    schedule_primary_attack, ground_locomotion, water_locomotion, process_legacy_visual_completions
};
pub use operations::select_legacy_targets;
pub use animation::{
    LegacyVisualClip, LegacyClipResolution, LegacyAvatarClipBindings, LegacyAnimationLayer
};
pub use codec::{LegacyFrameSchedule, schedule_legacy_action_frame};
use systems::{advance_attack_timers, update_legacy_avatar_locomotion};
pub use input::read_legacy_avatar_action_input;
use input::resolve_legacy_avatar_actions;
