use super::*;

pub(super) fn task_end_request(task_id: i32, npc_id: i32, escort_npc_id: i32) -> PcTaskEndRequest0104 {
    PcTaskEndRequest0104 {
        task_id,
        npc_id,
        reward_box_1: 0,
        reward_box_2: 0,
        pack_padding: [0; 2],
        escort_npc_id,
    }
}
