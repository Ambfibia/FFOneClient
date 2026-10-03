use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialSourceSpan {
    pub file: &'static str,
    pub start_line: u32,
    pub end_line: u32,
}

impl TutorialSourceSpan {
    pub const fn new(start_line: u32, end_line: u32) -> Self {
        Self {
            file: "cntutorialscript.cs",
            start_line,
            end_line,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TutorialAuxiliarySequence {
    BasicArrowKey,
    MinimapEvent,
    TalkNumTwo1,
    TalkNumTwo2,
    TalkButtercup1,
    TalkDexter1,
    TalkDexter2,
    TalkDexter3,
    NanoPowerTalk1,
}

impl TutorialAuxiliarySequence {
    pub const ALL: [Self; 9] = [
        Self::BasicArrowKey,
        Self::MinimapEvent,
        Self::TalkNumTwo1,
        Self::TalkNumTwo2,
        Self::TalkButtercup1,
        Self::TalkDexter1,
        Self::TalkDexter2,
        Self::TalkDexter3,
        Self::NanoPowerTalk1,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialText {
    /// `TextManager.GetSceneText(group, index)`.
    SceneText { group: u16, index: u16 },
    /// `TextManager.GetStr(text)`.
    Localized(&'static str),
    /// A literal passed directly by the original script.
    Literal(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialSubtitleChannel {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialCursorDirection {
    Right,
}

/// An exact integer screen-coordinate expression used by the legacy OnGUI
/// calls. It is evaluated only after the current viewport size is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialScreenAxis {
    Pixels(i32),
    WidthMinus(i32),
    HeightMinus(i32),
    WidthFractionMinus {
        numerator: i32,
        denominator: i32,
        subtract: i32,
    },
    HeightFractionMinus {
        numerator: i32,
        denominator: i32,
        subtract: i32,
    },
}

impl TutorialScreenAxis {
    #[must_use]
    pub fn evaluate(self, width: i32, height: i32) -> i32 {
        match self {
            Self::Pixels(value) => value,
            Self::WidthMinus(value) => width - value,
            Self::HeightMinus(value) => height - value,
            Self::WidthFractionMinus {
                numerator,
                denominator,
                subtract,
            } => width * numerator / denominator - subtract,
            Self::HeightFractionMinus {
                numerator,
                denominator,
                subtract,
            } => height * numerator / denominator - subtract,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialScreenPoint {
    pub x: TutorialScreenAxis,
    pub y: TutorialScreenAxis,
}

impl TutorialScreenPoint {
    pub const fn new(x: TutorialScreenAxis, y: TutorialScreenAxis) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn evaluate(self, width: i32, height: i32) -> [i32; 2] {
        [
            self.x.evaluate(width, height),
            self.y.evaluate(width, height),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialScreenPivot {
    /// Numeric value of the old `ScreenPivot` enum.
    Legacy(u8),
    Point(TutorialScreenPoint),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialWaypointTarget {
    DisabledAtZero,
    ActorPosition { runtime_id: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialBooleanFlag {
    MyPointEvent,
    WayPointEvent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialRepeatBlock {
    pub starts_at_seconds: f32,
    pub every_seconds: f32,
    pub actions: &'static [TutorialAuxiliaryAction],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialAuxiliaryDefinition {
    pub sequence: TutorialAuxiliarySequence,
    pub source: TutorialSourceSpan,
    pub actions: &'static [TimedTutorialAuxiliaryAction],
    /// Legacy coroutines without an infinite loop complete immediately after
    /// their last timed action.
    pub completes_at_seconds: Option<f32>,
    pub repeating: Option<TutorialRepeatBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialInitialNpc {
    pub legacy_server_position: [i32; 3],
    pub npc_type: i32,
    pub runtime_id: i32,
    pub legacy_angle_degrees: i32,
}

/// Exact production-side initialization contract from `InitStartChapter`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialInitializationContract {
    pub source: TutorialSourceSpan,
    pub required_web_bundles: &'static [&'static str],
    pub force_view_menu: bool,
    pub ambient_loop: &'static str,
    pub initial_music: &'static str,
    pub new_nano_id: i32,
    pub new_nano_offscreen_position: [f32; 3],
    pub dome_route: &'static str,
    pub dome_offscreen_position: [f32; 3],
    pub initial_npc: TutorialInitialNpc,
    pub production_locks_all_input: bool,
    pub production_fade_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialAuxiliaryEmissionOrigin {
    OneShot { action_index: usize },
    Repeat { cycle: u64, action_index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialAuxiliaryEmission {
    pub sequence: TutorialAuxiliarySequence,
    pub scheduled_at_seconds: f32,
    pub origin: TutorialAuxiliaryEmissionOrigin,
    pub action: TutorialAuxiliaryAction,
}

/// Small deterministic player. A zero-second tick after `start` emits all
/// actions scheduled at time zero. Large deltas never skip an action or a
/// reminder cycle.
#[derive(Debug, Clone, Default)]
pub struct TutorialAuxiliaryPlayer {
    pub(super) active: Option<TutorialAuxiliarySequence>,
    pub(super) elapsed_seconds: f32,
    pub(super) next_action: usize,
    pub(super) next_repeat_cycle: u64,
}

impl TutorialAuxiliaryPlayer {
    #[must_use]
    pub fn active(&self) -> Option<TutorialAuxiliarySequence> {
        self.active
    }

    #[must_use]
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    /// The next action which would replace a playing voice, including reminders.
    pub fn next_voice_seconds(&self) -> Option<f32> {
        let definition = tutorial_auxiliary_definition(self.active?);
        let once = definition.actions[self.next_action..]
            .iter()
            .find(|timed| matches!(timed.action, TutorialAuxiliaryAction::Voice { .. }))
            .map(|timed| timed.at_seconds);
        let repeat = definition
            .repeating
            .filter(|repeat| {
                repeat
                    .actions
                    .iter()
                    .any(|action| matches!(action, TutorialAuxiliaryAction::Voice { .. }))
            })
            .map(|repeat| {
                repeat.starts_at_seconds + repeat.every_seconds * self.next_repeat_cycle as f32
            });
        once.into_iter().chain(repeat).min_by(f32::total_cmp)
    }

    pub fn start(&mut self, sequence: TutorialAuxiliarySequence) {
        self.active = Some(sequence);
        self.elapsed_seconds = 0.0;
        self.next_action = 0;
        self.next_repeat_cycle = 0;
    }

    pub fn stop(&mut self) {
        self.active = None;
        self.elapsed_seconds = 0.0;
        self.next_action = 0;
        self.next_repeat_cycle = 0;
    }

    /// Localized playback waits only where another voice would replace it.
    /// Limit large frames to one voice boundary so deferred audio spawns have
    /// a frame to become visible before deciding whether another line can play.
    pub fn tick_localized(
        &mut self,
        delta_seconds: f32,
        voice_busy: bool,
    ) -> Result<Vec<TutorialAuxiliaryEmission>, &'static str> {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err("tutorial auxiliary delta must be finite and non-negative");
        }
        let delta = if let Some(next) = self.next_voice_seconds() {
            if voice_busy && next <= self.elapsed_seconds + delta_seconds {
                return Ok(Vec::new());
            }
            delta_seconds.min((next - self.elapsed_seconds).max(0.0))
        } else {
            delta_seconds
        };
        self.tick(delta)
    }

    pub fn tick(
        &mut self,
        delta_seconds: f32,
    ) -> Result<Vec<TutorialAuxiliaryEmission>, &'static str> {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err("tutorial auxiliary delta must be finite and non-negative");
        }
        let Some(sequence) = self.active else {
            return Ok(Vec::new());
        };
        let definition = tutorial_auxiliary_definition(sequence);
        let end = self.elapsed_seconds + delta_seconds;
        let mut emitted = Vec::new();

        while let Some(timed) = definition.actions.get(self.next_action) {
            if timed.at_seconds > end {
                break;
            }
            emitted.push(TutorialAuxiliaryEmission {
                sequence,
                scheduled_at_seconds: timed.at_seconds,
                origin: TutorialAuxiliaryEmissionOrigin::OneShot {
                    action_index: self.next_action,
                },
                action: timed.action,
            });
            self.next_action += 1;
        }

        if let Some(repeating) = definition.repeating {
            loop {
                let scheduled = repeating.starts_at_seconds
                    + repeating.every_seconds * self.next_repeat_cycle as f32;
                if scheduled > end {
                    break;
                }
                for (action_index, action) in repeating.actions.iter().copied().enumerate() {
                    emitted.push(TutorialAuxiliaryEmission {
                        sequence,
                        scheduled_at_seconds: scheduled,
                        origin: TutorialAuxiliaryEmissionOrigin::Repeat {
                            cycle: self.next_repeat_cycle,
                            action_index,
                        },
                        action,
                    });
                }
                self.next_repeat_cycle += 1;
            }
        }

        self.elapsed_seconds = end;
        if definition
            .completes_at_seconds
            .is_some_and(|completion| end >= completion)
        {
            self.active = None;
        }
        Ok(emitted)
    }
}
