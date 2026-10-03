use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningContentError {
    InvalidNanoId(i16),
    InvalidNanoStyle(u8),
    EmptyNanoName,
    InvalidTuneId { index: usize, tune_id: i16 },
    InvalidSkillId { index: usize, skill_id: i16 },
    EmptyIconPath(usize),
}

impl fmt::Display for NanoFreeTuningContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid NanoFreeTuning content: {self:?}")
    }
}

impl Error for NanoFreeTuningContentError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningTransitionError {
    AlreadyOpen,
    Closed,
    InvalidDelta,
    InvalidContent(NanoFreeTuningContentError),
    WrongPhase {
        expected: NanoFreeTuningPhase,
        actual: NanoFreeTuningPhase,
    },
    InvalidPowerIndex(usize),
    MissingSelection,
    ControlsDisabled,
    RequestAlreadyPending,
    InvalidAnimationDuration,
    IdleRollNotRequested,
    InvalidIdleRoll(i32),
    Protocol(NanoFreeTuningProtocolFault),
}

impl fmt::Display for NanoFreeTuningTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "NanoFreeTuning transition rejected: {self:?}")
    }
}

impl Error for NanoFreeTuningTransitionError {}
