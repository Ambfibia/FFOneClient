//! Pure transition logic for the Retrobution tutorial chapters 03 through 07.
//!
//! This module is deliberately independent from Bevy.  The caller observes the
//! world/UI once, evaluates one legacy tutorial update, applies the returned
//! intents, and commits the returned stage with the requested reset semantics.
//!
//! Two legacy details are intentionally visible in the API:
//!
//! - `CheckNpcDistance` succeeds when the NPC is damaged even when it is outside
//!   the requested radius.
//! - a skipped scene can have different postconditions from a naturally
//!   completed scene (`NanoPowerA2` is the concrete case in the reference).

use crate::tutorial::{
    CombatStage, InfectionStage, MinimapStage, MissionStage, NanoPowerStage, TUTORIAL_EVENT_COUNT,
    TutorialEvent, TutorialProgress, TutorialScene, TutorialStage,
};

#[cfg(test)]
mod tests;

mod constants;
mod localization;
mod types;
mod state;
mod commands;
mod validation;
mod systems_evaluate_mission;
mod systems_evaluate_infection;
mod operations;

pub use constants::{
    NUMBUH_TWO_ID, BUTTERCUP_ID, TECH_SQUARE_ATTENDANT_ID, FUSION_PORTAL_ID, LAIR_DEXTER_ID,
    FUSION_BUTTERCUP_ID, LAIR_EXIT_ID, DEMO_MONSTER_ID, COLLAPSE_NUMBUH_TWO_ID,
    MISSION_TARGET_ENGAGE_RADIUS
};
use constants::ALL_TUTORIAL_EVENTS;
pub use localization::TutorialLocaleBranch;
pub use types::{
    SceneCompletion, ObservedNpc, TutorialNpcObservation, TutorialUiObservation,
    TutorialObservation, LegacySpawnPosition, ClientPosition, TutorialNpcSpawn,
    TutorialDialogue, TutorialProgressReset, TutorialReferenceAmbiguity, TutorialDecision
};
pub use state::TutorialJournalMode;
pub use commands::TutorialIntent;
pub use validation::TutorialLogicError;
pub use systems_evaluate_mission::{evaluate_progress, evaluate_stage};
use systems_evaluate_infection::{evaluate_infection, evaluate_nano_power};
use operations::{
    finish_scene, spawn, objective_combat_entry_intents, journal_delay_transition,
    mission_handoff, enter_tech_square_warp_prompt, finish_dexter_cutscene,
    begin_nano_creation, append_demo_attack_if_due, append_return_to_numbuh_two_intents
};
