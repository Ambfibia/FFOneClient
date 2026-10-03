//! Native local state for the non-presentation tutorial intents.
//!
//! The original `cntutorialscript` stores one live `bInputLock[19]` array and
//! one `bInputLockTemp[19]` snapshot. Stable stage masks remain authored in
//! [`TutorialStage::metadata`]; this resource rebinds the live array when a
//! stage changes, then applies the reference's transient Push/Pop and LockUI
//! writes literally.

use bevy::prelude::Resource;

use crate::{
    tutorial::{TutorialInputLock, TutorialStage},
    tutorial_logic::TutorialIntent,
};

pub const TUTORIAL_INPUT_LOCK_COUNT: usize = 19;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialNativeIntentApplication {
    Applied,
    HideTutorialPointer,
    Unhandled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialInputFilterRuntime {
    bound_stage: Option<TutorialStage>,
    live_locks: [bool; TUTORIAL_INPUT_LOCK_COUNT],
    saved_locks: [bool; TUTORIAL_INPUT_LOCK_COUNT],
}

impl Default for TutorialInputFilterRuntime {
    fn default() -> Self {
        Self {
            bound_stage: None,
            // Unknown tutorial states must not accidentally expose input.
            live_locks: [true; TUTORIAL_INPUT_LOCK_COUNT],
            // C# `new bool[19]` initializes the single temp snapshot to false.
            saved_locks: [false; TUTORIAL_INPUT_LOCK_COUNT],
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct TutorialNativeMechanics {
    fatigue_level: i32,
    instance_map: bool,
    episode: i32,
    tutorial_pointer_visible: bool,
    input: TutorialInputFilterRuntime,
}

impl TutorialNativeMechanics {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn fatigue_level(&self) -> i32 {
        self.fatigue_level
    }

    pub fn instance_map(&self) -> bool {
        self.instance_map
    }

    pub fn episode(&self) -> i32 {
        self.episode
    }

    pub fn tutorial_pointer_visible(&self) -> bool {
        self.tutorial_pointer_visible
    }

    pub fn mark_tutorial_pointer_visible(&mut self) {
        self.tutorial_pointer_visible = true;
    }

    pub fn bound_stage(&self) -> Option<TutorialStage> {
        self.input.bound_stage
    }

    pub fn live_input_locks(&self) -> &[bool; TUTORIAL_INPUT_LOCK_COUNT] {
        &self.input.live_locks
    }

    pub fn saved_input_locks(&self) -> &[bool; TUTORIAL_INPUT_LOCK_COUNT] {
        &self.input.saved_locks
    }

    pub fn is_locked(&self, input: TutorialInputLock) -> bool {
        self.input.live_locks[input.index()]
    }

    /// Rebinds only when the stable stage changes. The single legacy snapshot
    /// deliberately survives stage changes until a matching Pop restores it.
    pub fn sync_stable_stage(&mut self, stage: TutorialStage) {
        if self.input.bound_stage == Some(stage) {
            return;
        }
        self.input.bound_stage = Some(stage);
        self.input.live_locks = stable_stage_locks(stage);
    }

    pub fn fail_closed_input(&mut self) {
        self.input.bound_stage = None;
        self.input.live_locks = [true; TUTORIAL_INPUT_LOCK_COUNT];
    }

    pub fn apply_intent(&mut self, intent: TutorialIntent) -> TutorialNativeIntentApplication {
        match intent {
            TutorialIntent::SetFatigueLevel(value) => {
                self.fatigue_level = value;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::SetInstanceMap(value) => {
                self.instance_map = value;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::SetEpisode(value) => {
                self.episode = value;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::LockUi => {
                self.input.live_locks[TutorialInputLock::ModeChange.index()] = true;
                self.input.live_locks[TutorialInputLock::Menu.index()] = true;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::UnlockUi => {
                self.input.live_locks[TutorialInputLock::ModeChange.index()] = false;
                self.input.live_locks[TutorialInputLock::Menu.index()] = false;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::PushInputFilter => {
                self.input.saved_locks = self.input.live_locks;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::PopInputFilter => {
                self.input.live_locks = self.input.saved_locks;
                TutorialNativeIntentApplication::Applied
            }
            TutorialIntent::HideTutorialPointer => {
                self.tutorial_pointer_visible = false;
                TutorialNativeIntentApplication::HideTutorialPointer
            }
            _ => TutorialNativeIntentApplication::Unhandled,
        }
    }
}

fn stable_stage_locks(stage: TutorialStage) -> [bool; TUTORIAL_INPUT_LOCK_COUNT] {
    let allowed_bits = stage.metadata().input.allowed_bits();
    std::array::from_fn(|index| allowed_bits & (1_u32 << index) == 0)
}

#[cfg(test)]
mod tests;
