use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldMissionApplyOutcome {
    pub ui_resolution: Option<WorldMissionUiResolution>,
    pub completion_kind: Option<WorldMissionCompletionKind>,
    pub follow_up_start: Option<PcTaskStartRequest0104>,
}
