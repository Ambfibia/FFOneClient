use super::*;

pub fn decode_world_mission_event_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<WorldMissionServerEvent0104>, String> {
    let malformed = |kind: &str, error: ffone_protocol::PayloadError| {
        format!("malformed protocol-0104 {kind}: {error}")
    };
    Ok(match packet_type {
        packet::P_FE2CL_REP_PC_TASK_START_SUCC => {
            Some(WorldMissionServerEvent0104::TaskStartSuccess(
                PcTaskStartSuccess0104::decode(payload)
                    .map_err(|error| malformed("task-start success", error))?,
            ))
        }
        packet::P_FE2CL_REP_PC_TASK_START_FAIL => {
            Some(WorldMissionServerEvent0104::TaskStartFailure(
                PcTaskFailure0104::decode(payload)
                    .map_err(|error| malformed("task-start failure", error))?,
            ))
        }
        packet::P_FE2CL_REP_PC_TASK_END_SUCC => Some(WorldMissionServerEvent0104::TaskEndSuccess(
            PcTaskEndSuccess0104::decode(payload)
                .map_err(|error| malformed("task-end success", error))?,
        )),
        packet::P_FE2CL_REP_PC_TASK_END_FAIL => Some(WorldMissionServerEvent0104::TaskEndFailure(
            PcTaskFailure0104::decode(payload)
                .map_err(|error| malformed("task-end failure", error))?,
        )),
        packet::P_FE2CL_REP_PC_TASK_STOP_SUCC => {
            Some(WorldMissionServerEvent0104::TaskStopSuccess(
                PcTaskStopSuccess0104::decode(payload)
                    .map_err(|error| malformed("task-stop success", error))?,
            ))
        }
        packet::P_FE2CL_REP_PC_TASK_STOP_FAIL => {
            Some(WorldMissionServerEvent0104::TaskStopFailure(
                PcTaskStopFailure0104::decode(payload)
                    .map_err(|error| malformed("task-stop failure", error))?,
            ))
        }
        packet::P_FE2CL_REP_PC_KILL_QUEST_NPCS_SUCC => {
            Some(WorldMissionServerEvent0104::KillQuestNpc(
                PcKillQuestNpcsSuccess0104::decode(payload)
                    .map_err(|error| malformed("kill-quest-NPC success", error))?,
            ))
        }
        packet::P_FE2CL_REP_REWARD_ITEM => Some(WorldMissionServerEvent0104::RewardItem(
            RewardItemReply0104::decode(payload)
                .map_err(|error| malformed("reward-item reply", error))?,
        )),
        packet::P_FE2CL_REP_PC_SET_CURRENT_MISSION_ID => {
            Some(WorldMissionServerEvent0104::SetCurrentMission(
                PcSetCurrentMissionId0104::decode(payload)
                    .map_err(|error| malformed("set-current-mission reply", error))?,
            ))
        }
        _ => None,
    })
}
