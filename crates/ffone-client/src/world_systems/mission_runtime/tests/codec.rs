use super::*;

#[test]
fn pc_load_restores_nano_mission_and_progress_in_every_wire_slot() {
    let content = production_content();
    let definition = content
        .missions()
        .find(|definition| {
            definition.mission_type == TutorialMissionType::Nano
                && definition
                    .provenance
                    .completion_enemy_ids
                    .iter()
                    .any(|id| *id > 0)
        })
        .expect("published catalog has a Nano combat task");
    let running = ffone_protocol::RunningQuest0104 {
        task_id: definition.provenance.task_id,
        kill_npc_ids: definition.provenance.completion_enemy_ids,
        remaining_kill_counts: [0; 3],
        needed_item_ids: definition.provenance.completion_item_ids,
        needed_item_counts: definition.provenance.completion_item_counts,
    };
    for slot in 0..PcLoadData0104::RUNNING_QUEST_COUNT {
        let mut load = PcLoadData0104::zeroed();
        write_running_quest(&mut load, slot, running);
        write_i32_at(
            &mut load,
            PcLoadData0104::CURRENT_MISSION_ID_OFFSET,
            definition.provenance.mission_id,
        );
        let mut runtime = WorldMissionRuntime::default();
        runtime.seed(&load, content).unwrap();
        assert_eq!(
            runtime.active_task_ids(),
            vec![running.task_id],
            "slot {slot}"
        );
        assert_eq!(runtime.active_tasks[0].remaining_enemy_counts, [0; 3]);
        assert_eq!(
            runtime.selected_mission_id(),
            Some(definition.provenance.mission_id)
        );
        // A second login replaces the session without losing or duplicating the task.
        runtime.seed(&load, content).unwrap();
        assert_eq!(runtime.active_task_ids(), vec![running.task_id]);
    }
}
