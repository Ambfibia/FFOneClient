use super::*;

#[test]
fn escort_requests_resolve_live_owners_for_every_published_escort_task() {
    let content = production_content();
    let mut count = 0;
    for task in content.missions().filter(|task| task.provenance.escort_def_npc_type > 0) {
        count += 1;
        let task_id = task.provenance.task_id;
        let npc_type = task.provenance.escort_def_npc_type;
        assert_eq!(mission_escort_npc_id(task_id, content, [(npc_type, 900_001)]), 900_001);
        assert_eq!(mission_escort_npc_id(task_id, content, [(npc_type + 1, 900_001)]), 0);
        assert_eq!(mission_escort_npc_id(task_id, content, []), 0);
    }
    assert!(count > 1);
    assert_eq!(mission_escort_npc_id(576, content, [(1007, 42), (1011, 73)]), 42);
    assert_eq!(mission_escort_npc_id(5211, content, [(2567, 42), (1588, 73)]), 42);
    assert_eq!(mission_escort_npc_id(-1, content, [(1007, 42)]), 0);
}

#[test]
fn pc_load_seed_uses_clean_bit_mapping_slots_and_current_mission() {
    let content = production_content();
    let definition = content
        .missions()
        .find(|definition| {
            definition.mission_type == TutorialMissionType::World
                && (1..=1024).contains(&definition.provenance.mission_id)
        })
        .expect("published catalog has a normal mission in clean flag capacity");
    let mission_zero_based = definition.provenance.mission_id - 1;
    let mission_word = usize::try_from(mission_zero_based / 64).unwrap();
    let mission_bit = u32::try_from(mission_zero_based % 64).unwrap();

    let mut load = PcLoadData0104::zeroed();
    write_i64_at(
        &mut load,
        PcLoadData0104::QUEST_FLAGS_OFFSET + mission_word * 8,
        1_i64 << mission_bit,
    );
    // The wire owns 32 words, but clean SetMissionAndTaskFlags consumes 16.
    write_i64_at(&mut load, PcLoadData0104::QUEST_FLAGS_OFFSET + 16 * 8, 1);
    let running = ffone_protocol::RunningQuest0104 {
        task_id: definition.provenance.task_id,
        kill_npc_ids: definition.provenance.completion_enemy_ids,
        remaining_kill_counts: definition.provenance.completion_enemy_counts,
        needed_item_ids: definition.provenance.completion_item_ids,
        needed_item_counts: definition.provenance.completion_item_counts,
    };
    load.as_bytes_mut()[PcLoadData0104::RUNNING_QUESTS_OFFSET
        ..PcLoadData0104::RUNNING_QUESTS_OFFSET + ffone_protocol::RunningQuest0104::SIZE]
        .copy_from_slice(&running.encode());
    write_i32_at(
        &mut load,
        PcLoadData0104::CURRENT_MISSION_ID_OFFSET,
        definition.provenance.mission_id,
    );

    let mut runtime = WorldMissionRuntime::default();
    runtime.seed(&load, content).unwrap();
    assert!(
        runtime
            .completed_mission_ids()
            .contains(&definition.provenance.mission_id)
    );
    assert_eq!(
        runtime.completed_task_ids().collect::<Vec<_>>(),
        vec![
            content
                .final_serialized_task_for_mission_id(definition.provenance.mission_id)
                .unwrap()
                .provenance
                .task_id
        ]
    );
    assert!(!runtime.completed_mission_ids().contains(&1025));
    assert_eq!(
        runtime.active_task_ids(),
        vec![definition.provenance.task_id]
    );
    assert_eq!(
        runtime.selected_mission_id(),
        Some(definition.provenance.mission_id)
    );
}

#[test]
fn pc_load_seed_remaps_kills_by_npc_id_and_restores_clean_expired_timer() {
    let content = production_content();
    let kill_definition = content
        .missions()
        .find(|definition| {
            definition.mission_type == TutorialMissionType::World
                && definition
                    .provenance
                    .completion_enemy_ids
                    .iter()
                    .filter(|npc_id| **npc_id > 0)
                    .count()
                    == 1
        })
        .expect("published catalog has a one-enemy world task");
    let definition_index = kill_definition
        .provenance
        .completion_enemy_ids
        .iter()
        .position(|npc_id| *npc_id > 0)
        .unwrap();
    let wire_index = (definition_index + 1) % 3;
    let mut wire_ids = [0; 3];
    let mut wire_counts = [0; 3];
    wire_ids[wire_index] = kill_definition.provenance.completion_enemy_ids[definition_index];
    wire_counts[wire_index] = 37;
    let mut load = PcLoadData0104::zeroed();
    write_running_quest(
        &mut load,
        0,
        ffone_protocol::RunningQuest0104 {
            task_id: kill_definition.provenance.task_id,
            kill_npc_ids: wire_ids,
            remaining_kill_counts: wire_counts,
            needed_item_ids: [0; 3],
            needed_item_counts: [0; 3],
        },
    );
    let mut runtime = WorldMissionRuntime::default();
    runtime.seed(&load, content).unwrap();
    let active = runtime.active_tasks().first().unwrap();
    assert_eq!(
        active.remaining_enemy_ids,
        kill_definition.provenance.completion_enemy_ids
    );
    assert_eq!(active.remaining_enemy_counts[definition_index], 37);

    let timer_definition = content
        .missions()
        .find(|definition| definition.provenance.grant_timer > 0)
        .expect("published catalog has a grant-timer task");
    let mut load = PcLoadData0104::zeroed();
    write_running_quest(
        &mut load,
        0,
        ffone_protocol::RunningQuest0104 {
            task_id: timer_definition.provenance.task_id,
            kill_npc_ids: timer_definition.provenance.completion_enemy_ids,
            remaining_kill_counts: timer_definition.provenance.completion_enemy_counts,
            needed_item_ids: timer_definition.provenance.completion_item_ids,
            needed_item_counts: timer_definition.provenance.completion_item_counts,
        },
    );
    let mut runtime = WorldMissionRuntime::default();
    runtime.seed(&load, content).unwrap();
    assert_eq!(
        runtime.active_tasks()[0].remaining_time_millis,
        Some(CLEAN_PC_LOAD_TIMER_INITIAL_MILLIS)
    );
    let actions = runtime
        .collect_automatic_end_requests(&[], None, &[], content)
        .unwrap();
    assert_eq!(
        actions
            .end_requests
            .iter()
            .map(|request| request.task_id)
            .collect::<Vec<_>>(),
        vec![timer_definition.provenance.task_id]
    );
}

#[test]
fn computress_completion_row_is_restored_from_pc_load_quest_inventory() {
    use crate::{
        gameplay_ui::{GameplayUiAction, GameplayUiOutbox},
        mission_ui::{MissionJournalUi, MissionUiModel, NpcInteractionUi},
    };

    let content = production_content();
    let definition = content.mission(451).expect("primary Computress task 451");
    assert_eq!(definition.provenance.terminator_npc_type, 2_555);
    assert_eq!(definition.provenance.completion_item_ids[0], 129);
    assert_eq!(definition.provenance.completion_item_counts[0], 1);
    let reward = content.reward(106).expect("primary reward 106");
    assert_eq!((reward.box1_choice, reward.box2_choice), (0, 0));
    assert_eq!(
        reward.primary_items[0],
        crate::tutorial_mission_content::TutorialRewardItem {
            item_type: 5,
            item_id: 24,
        }
    );

    let mut load = PcLoadData0104::zeroed();
    write_running_quest(
        &mut load,
        0,
        ffone_protocol::RunningQuest0104 {
            task_id: definition.provenance.task_id,
            kill_npc_ids: [0; 3],
            remaining_kill_counts: [0; 3],
            needed_item_ids: [0; 3],
            needed_item_counts: [0; 3],
        },
    );
    let quest_item_offset = PcLoadData0104::QUEST_INVENTORY_OFFSET;
    load.as_bytes_mut()[quest_item_offset..quest_item_offset + 2]
        .copy_from_slice(&8_i16.to_le_bytes());
    load.as_bytes_mut()[quest_item_offset + 2..quest_item_offset + 4]
        .copy_from_slice(&129_i16.to_le_bytes());
    write_i32_at(&mut load, quest_item_offset + 4, 1);

    let mut runtime = WorldMissionRuntime::default();
    runtime.seed(&load, content).unwrap();
    let (_, completed) = runtime
        .npc_entries(
            2_555,
            1,
            "Tech Square",
            36,
            0,
            &BTreeSet::new(),
            &load.quest_inventory(),
            content,
        )
        .unwrap();

    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].task_id, 451);
    assert_eq!(completed[0].rewards.cash, 30);
    assert_eq!(completed[0].rewards.fusion_matter, 55);

    let mut ui = MissionUiModel::default();
    ui.show_npc_interaction(NpcInteractionUi {
        npc_id: 1,
        npc_type: 2_555,
        name: "Computress".to_owned(),
        completed_missions: completed,
        ..NpcInteractionUi::default()
    });
    let mut outbox = GameplayUiOutbox::default();
    assert!(ui.select_npc_mission(0, &mut outbox));
    assert!(matches!(
        ui.journal,
        MissionJournalUi::Reward { ref mission, .. } if mission.task_id == 451
    ));
    assert!(ui.complete_mission(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenMissionReward {
                task_id: 451,
                npc_id: 1,
            },
            GameplayUiAction::QuestEnd {
                task_id: 451,
                npc_id: 1,
                box1_choice: 0,
                box2_choice: 0,
            },
        ]
    );
}
