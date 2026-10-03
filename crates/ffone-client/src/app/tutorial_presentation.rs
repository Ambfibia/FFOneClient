//! Gameplay HUD and tutorial overlay projection from authoritative runtime state.

use crate::app::*;

#[cfg(test)]
mod location_notice_tests;

mod types;
mod systems;
mod projects;
mod operations;
mod input;
mod localization;

pub(super) use types::{TutorialPresentationPlugin, TutorialPresentationSet};
use types::{GameplayHudOptionInputs, GameplayHudMissionInputs};
use systems::{sync_world_location_notice, sync_gameplay_hud, sync_tutorial_overlay};
pub(super) use projects::project_nano_skill_cooldown;
pub(super) use operations::{tutorial_ui_for_stage, tutorial_auxiliary_presentation_matches};
use operations::{tutorial_illustration_cue, tutorial_arrow_cue};
pub(super) use input::resolve_tutorial_auxiliary_text;
pub(super) use localization::localized_tutorial_auxiliary_text;
