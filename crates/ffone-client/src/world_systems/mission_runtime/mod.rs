//! Server-authoritative ordinary-world mission state for protocol 0104.
//!
//! This is deliberately separate from `TutorialMissionRuntime`: ordinary
//! mission ownership comes from `sPCLoadData2CL` and correlated FE replies.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;
use ffone_protocol::{
    ItemBase0104, PcKillQuestNpcsSuccess0104, PcLoadData0104, PcSetCurrentMissionId0104,
    PcTaskEndRequest0104, PcTaskEndSuccess0104, PcTaskFailure0104, PcTaskStartRequest0104,
    PcTaskStartSuccess0104, PcTaskStopFailure0104, PcTaskStopSuccess0104, RewardItemReply0104,
    WirePayload, packet,
};

use crate::{
    mission_ui::{MissionUiEntry, PendingMissionUiRequest},
    tutorial_mission_content::{
        TutorialMissionContent, TutorialMissionDefinition, TutorialMissionType,
        MissionMarkerSurface,
    },
};

#[cfg(test)]
mod tests;

mod input;
mod operations;
mod types;
mod systems;
mod codec;
mod state_world_mission_runtime_take_inventory_full_no;
mod state_world_mission_runtime_apply_event_inner;
mod commands;

use input::CLEAN_PC_LOAD_TIMER_INITIAL_MILLIS;
pub use operations::{mission_escort_npc_id, quest_item_count};
use operations::{repeat_flag_position, distance};
pub use types::{
    WorldActiveMissionTask, WorldMissionObjectiveProgress, WorldMissionCurrentObjective,
    WorldMissionUiResolution, WorldMissionCompletionKind, WorldMissionDialogueEdge,
    WorldMissionNearbyNpc, WorldMissionAutomaticReward, WorldMissionAutomaticActions,
    WorldMissionAcceptRejection, WorldMissionServerEvent0104
};
pub use systems::WorldMissionApplyOutcome;
pub use codec::decode_world_mission_event_0104;
pub use state_world_mission_runtime_take_inventory_full_no::WorldMissionRuntime;
use commands::task_end_request;
