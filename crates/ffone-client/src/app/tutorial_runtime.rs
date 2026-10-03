use crate::app::*;

mod localization;
mod state;
mod operations;
mod systems;
mod input;
mod types;
mod commands;

use localization::retain_localized_tutorial_subtitle;
pub(super) use state::TutorialRuntimePlugin;
pub(super) use operations::{
    drive_tutorial_auxiliary_choreography, tutorial_actor_effect_world_position,
    tutorial_movement_stage_effects, tutorial_movement_stage_camera_target, equip_tutorial_nano,
    drive_local_tutorial
};
#[cfg(test)]
pub(super) use operations::{
    tutorial_movement_timeout_restarts, ensure_tutorial_nano_loadout,
    reconcile_tutorial_nano_activation, cancel_tutorial_step_sequence
};
pub(super) use systems::{
    apply_tutorial_auxiliary_emission, apply_tutorial_movement_stage_effects,
    apply_tutorial_movement_stage_camera, apply_tutorial_decision, apply_pending_tutorial_exit
};
#[cfg(test)]
pub(super) use systems::apply_tutorial_input_stage_transition;
pub(super) use input::resolve_pending_tutorial_actor_effects;
pub(super) use types::TutorialMovementStageEffect;
pub(super) use commands::{
    TutorialNanoShortcutAction, tutorial_nano_shortcut_action, apply_tutorial_native_intent
};
