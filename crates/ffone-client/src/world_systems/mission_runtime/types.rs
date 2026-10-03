use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldActiveMissionTask {
    pub task_id: i32,
    pub remaining_enemy_ids: [i32; 3],
    pub remaining_enemy_counts: [i32; 3],
    /// `None` means the task has no grant timer. Fresh starts use the server
    /// value; PC-load grant timers preserve clean's -1-second initialization.
    pub remaining_time_millis: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldMissionObjectiveProgress {
    pub content_id: i32,
    pub name: String,
    pub complete: i32,
    pub needed: i32,
}

/// Structured source data for `cnGUINanocom`'s selected current objective.
/// Localization must happen before the clean newline/progress composition is
/// rebuilt, otherwise translating the base objective discards its counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldMissionCurrentObjective {
    pub task_id: i32,
    pub title: String,
    pub objective: String,
    pub remaining_time_seconds: Option<i64>,
    pub enemies: Vec<WorldMissionObjectiveProgress>,
    pub quest_items: Vec<WorldMissionObjectiveProgress>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldMissionUiResolution {
    TaskStartAccepted(i32),
    QuestEndAccepted(i32),
    TaskStopAccepted(i32),
    RequestRejected(i32),
}

/// Clean `cnMissionManager.ProcessEndSucc` distinguishes an intermediate
/// task edge from the mission's terminal edge for presentation and audio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldMissionCompletionKind {
    Task,
    Mission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldMissionDialogueEdge {
    Start,
    Success,
    Complete,
    Failure,
}

impl WorldMissionCompletionKind {
    #[must_use]
    pub const fn legacy_audio_true_name(self) -> &'static str {
        match self {
            Self::Task => "Task_Completed",
            Self::Mission => "Mission_Completed",
        }
    }
}

/// One live normal-world NPC projected into the pure clean mission reducer.
/// Positions use native/Unity axes and `sight_range_server_units` retains the
/// exact centiunit TableData value consumed by `CoordUtil.Distance_SToC`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldMissionNearbyNpc {
    pub npc_type: i32,
    pub npc_id: i32,
    pub position: [f32; 3],
    pub sight_range_server_units: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldMissionAutomaticReward {
    pub task_id: i32,
    pub npc_id: i32,
    pub escort_npc_id: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldMissionAutomaticActions {
    pub end_requests: Vec<PcTaskEndRequest0104>,
    pub reward: Option<WorldMissionAutomaticReward>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldMissionAcceptRejection {
    UnknownTask {
        task_id: i32,
    },
    CategoryFull {
        mission_type: TutorialMissionType,
        capacity: usize,
    },
    UnsupportedLocation {
        task_id: i32,
        task_type: i32,
        required_instance_id: i32,
    },
    UnsupportedEscort {
        task_id: i32,
        escort_npc_type: i32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldMissionServerEvent0104 {
    TaskStartSuccess(PcTaskStartSuccess0104),
    TaskStartFailure(PcTaskFailure0104),
    TaskEndSuccess(PcTaskEndSuccess0104),
    TaskEndFailure(PcTaskFailure0104),
    TaskStopSuccess(PcTaskStopSuccess0104),
    TaskStopFailure(PcTaskStopFailure0104),
    KillQuestNpc(PcKillQuestNpcsSuccess0104),
    RewardItem(RewardItemReply0104),
    SetCurrentMission(PcSetCurrentMissionId0104),
}
