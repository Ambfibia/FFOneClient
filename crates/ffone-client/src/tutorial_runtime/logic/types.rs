use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SceneCompletion {
    #[default]
    Pending,
    Natural,
    Skipped,
}

impl SceneCompletion {
    pub const fn is_complete(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObservedNpc {
    /// Full 3D client-space distance. `None` means the NPC does not exist.
    pub distance: Option<f32>,
    /// The reference `CheckNpcDistance` returns true for a damaged NPC at any
    /// distance.
    pub damaged: bool,
    /// Whether NPC icon mode currently targets this exact NPC ID.
    pub interacting: bool,
}

impl ObservedNpc {
    pub fn within_or_damaged(self, radius: f32) -> bool {
        self.damaged || self.distance.is_some_and(|distance| distance < radius)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TutorialNpcObservation {
    pub numbuh_two: ObservedNpc,
    pub mission_target: ObservedNpc,
    pub buttercup: ObservedNpc,
    pub tech_square_attendant: ObservedNpc,
    pub fusion_portal: ObservedNpc,
    pub lair_dexter: ObservedNpc,
    pub fusion_buttercup: ObservedNpc,
    pub lair_exit: ObservedNpc,
    pub collapse_numbuh_two: ObservedNpc,
    pub demo_monster: ObservedNpc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TutorialUiObservation {
    /// Hostile monster info is visible and has a renderable combat icon:
    /// `CheckUI(8)`.
    pub hostile_target_selected: bool,
    /// The Nanocom main menu is visible: `CheckUI(2)`.
    pub nanocom_main_menu_visible: bool,
    pub journal_mode: TutorialJournalMode,
    /// Whether the general NPC-icon game mode is active. Target identity is
    /// represented by each [`ObservedNpc::interacting`] field.
    pub npc_icon_mode_visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TutorialObservation {
    /// Exact `pEventFlag[0..34]` snapshot.
    pub event_flags: [i32; TUTORIAL_EVENT_COUNT],
    pub npcs: TutorialNpcObservation,
    pub ui: TutorialUiObservation,
    /// Integer `pFlag[6]`, populated by the reference timer using truncation.
    pub timer_seconds: u32,
    pub locale: TutorialLocaleBranch,
    /// `true` only after the blocking `Delay(...)` has released Update.
    pub delay_completed: bool,
    pub scene_completion: SceneCompletion,
}

impl Default for TutorialObservation {
    fn default() -> Self {
        Self {
            event_flags: [0; TUTORIAL_EVENT_COUNT],
            npcs: TutorialNpcObservation::default(),
            ui: TutorialUiObservation::default(),
            timer_seconds: 0,
            locale: TutorialLocaleBranch::default(),
            delay_completed: false,
            scene_completion: SceneCompletion::default(),
        }
    }
}

impl TutorialObservation {
    /// Copies the event and timer state owned by [`TutorialProgress`] while
    /// preserving the caller-provided NPC/UI/presentation observations.
    pub fn with_progress(mut self, progress: &TutorialProgress) -> Self {
        for event in ALL_TUTORIAL_EVENTS {
            self.event_flags[event.index()] = progress.event_value(event);
        }
        self.timer_seconds = progress.timer_seconds();
        self
    }

    pub const fn event_value(&self, event: TutorialEvent) -> i32 {
        self.event_flags[event.index()]
    }

    pub const fn event_is(&self, event: TutorialEvent, value: i32) -> bool {
        self.event_value(event) == value
    }

    pub const fn timer_exceeds(&self, seconds: u32) -> bool {
        self.timer_seconds > seconds
    }
}

/// Position in the argument order used by `cntutorialscript.AddNpc`: server X,
/// server Y (horizontal client Z), server Z (vertical client Y), in centiunits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacySpawnPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl LegacySpawnPosition {
    pub const fn centiunits(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Unity client-space position in centiunits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ClientPosition {
    pub const fn centiunits(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialNpcSpawn {
    pub id: i32,
    pub npc_type: i32,
    pub position: LegacySpawnPosition,
    /// Angle passed to `AngleNpc`, before that helper's internal `+180`.
    pub angle: Option<i16>,
}

impl TutorialNpcSpawn {
    pub const fn new(
        id: i32,
        npc_type: i32,
        position: LegacySpawnPosition,
        angle: Option<i16>,
    ) -> Self {
        Self {
            id,
            npc_type,
            position,
            angle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialDialogue {
    Minimap,
    TalkNumbuhTwoMission,
    TalkNumbuhTwoReward,
    TalkButtercup,
    TalkDexterOutside,
    TalkDexterInside,
    TalkDexterExit,
    NanoPowerReminder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialProgressReset {
    /// Exact `InitStep(next_step)`.
    InitStep,
    /// Direct `pFlag[1] = next_step`; event flags and timer are preserved.
    DirectStepWrite,
    /// `InitStep(0)` followed by `InitChapter(next_chapter)`.
    InitChapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialReferenceAmbiguity {
    /// The reference writes this step and immediately writes a new chapter in
    /// the same branch. If externally restored at this step, it has no handler.
    TransientStageHasNoStandaloneBranch(TutorialStage),
    /// `ExitTutorial` normally disables the script immediately. Restoring the
    /// terminal step alone has no additional reference transition.
    TerminalStageAlreadyExited(TutorialStage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialDecision {
    pub current_stage: TutorialStage,
    pub next_stage: TutorialStage,
    pub reset: Option<TutorialProgressReset>,
    pub intents: Vec<TutorialIntent>,
    pub ambiguities: Vec<TutorialReferenceAmbiguity>,
}

impl TutorialDecision {
    pub(super) fn unchanged(stage: TutorialStage) -> Self {
        Self {
            current_stage: stage,
            next_stage: stage,
            reset: None,
            intents: Vec::new(),
            ambiguities: Vec::new(),
        }
    }

    pub(super) fn transition(
        current_stage: TutorialStage,
        next_stage: TutorialStage,
        reset: TutorialProgressReset,
    ) -> Self {
        Self {
            current_stage,
            next_stage,
            reset: Some(reset),
            intents: Vec::new(),
            ambiguities: Vec::new(),
        }
    }

    pub const fn transitioned(&self) -> bool {
        self.reset.is_some()
    }
}
