use super::*;

#[test]
fn delayed_selection_reply_never_reverts_new_selection_or_blocks_completed_mission() {
    let content = production_content();
    let mut runtime = WorldMissionRuntime::default();
    for task_id in [451, 576] {
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                    task_id,
                    remaining_time: 0,
                }),
                content,
            )
            .unwrap();
    }
    let first = runtime.begin_select_task(451, content).unwrap().unwrap();
    assert!(runtime.begin_select_task(576, content).is_err());
    let desired = content.mission(576).unwrap().provenance.mission_id;
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 { task_id: 451 }),
            content,
        )
        .unwrap();
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::SetCurrentMission(first),
            content,
        )
        .unwrap();
    assert_eq!(runtime.selected_mission_id(), Some(desired));
    assert_eq!(
        runtime
            .begin_select_task(576, content)
            .unwrap()
            .unwrap()
            .mission_id,
        desired
    );
}

#[test]
fn nonterminal_success_keeps_selection_and_failure_starts_exact_failure_edge() {
    let content = production_content();
    let transition = content.mission(5_229).expect("primary task 5229");
    assert!(!content.is_final_serialized_task(5_229).unwrap());
    let other = content.mission(500).expect("primary task 500");
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(other), active_task(transition)],
        selected_mission_id: Some(transition.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };
    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 {
                task_id: transition.provenance.task_id,
            }),
            content,
        )
        .unwrap();
    assert_eq!(
        runtime.selected_mission_id(),
        Some(transition.provenance.mission_id)
    );
    assert_eq!(
        outcome.follow_up_start.map(|request| request.task_id),
        Some(5_230)
    );

    let failed = content
        .mission(500)
        .expect("primary failure-chain task 500");
    assert_eq!(failed.provenance.failure_outgoing_task_id, 27);
    let pending = PendingMissionUiRequest::QuestEnd {
        task_id: failed.provenance.task_id,
        npc_id: 0,
        box1_choice: 0,
        box2_choice: 0,
    };
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(failed)],
        selected_mission_id: Some(failed.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };
    runtime.record_request(pending).unwrap();
    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndFailure(PcTaskFailure0104 {
                task_id: failed.provenance.task_id,
                error_code: 1,
            }),
            content,
        )
        .unwrap();
    assert!(runtime.active_tasks().is_empty());
    assert_eq!(
        outcome.follow_up_start.map(|request| request.task_id),
        Some(27)
    );
    assert_eq!(
        runtime.pending_request(),
        Some(PendingMissionUiRequest::TaskStart {
            task_id: 27,
            npc_id: 0,
        })
    );
    assert_eq!(
        runtime.selected_mission_id(),
        Some(failed.provenance.mission_id)
    );
}

#[test]
fn reward_item_event_observes_post_state_before_auto_end_or_reward_ui() {
    let content = production_content();
    let definition = content
        .missions()
        .find(|definition| {
            definition.provenance.terminator_npc_type <= 0
                && definition
                    .provenance
                    .completion_item_ids
                    .iter()
                    .zip(definition.provenance.completion_item_counts)
                    .any(|(item_id, count)| *item_id > 0 && count > 0)
        })
        .expect("published catalog has a no-terminator quest-item task");
    let mut inventory = Vec::new();
    for index in 0..3 {
        let item_id = definition.provenance.completion_item_ids[index];
        let count = definition.provenance.completion_item_counts[index];
        if item_id > 0 && count > 0 {
            inventory.push(ItemBase0104 {
                item_type: 8,
                item_id: i16::try_from(item_id).unwrap(),
                option: count,
                time_limit: 0,
            });
        }
    }
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(definition)],
        ..WorldMissionRuntime::default()
    };
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::RewardItem(RewardItemReply0104 {
                candy: 0,
                fusion_matter: 0,
                nano_battery: 0,
                weapon_battery: 0,
                pack_padding: [0; 3],
                fatigue: 0,
                fatigue_level: 0,
                npc_type_id: 0,
                task_id: definition.provenance.task_id,
                items: Vec::new(),
            }),
            content,
        )
        .unwrap();
    let actions = runtime
        .collect_automatic_end_requests(&inventory, None, &[], content)
        .unwrap();
    if definition.provenance.reward_id > 0 {
        assert_eq!(
            actions.reward.map(|reward| reward.task_id),
            Some(definition.provenance.task_id)
        );
        assert!(actions.end_requests.is_empty());
    } else {
        assert_eq!(actions.end_requests.len(), 1);
        assert_eq!(
            actions.end_requests[0].task_id,
            definition.provenance.task_id
        );
    }
}

#[test]
fn type_two_proximity_uses_live_terminator_runtime_id() {
    let content = production_content();
    let definition = content
        .missions()
        .find(|definition| {
            definition.provenance.task_type == 2
                && definition.provenance.terminator_npc_type > 0
                && definition.provenance.reward_id <= 0
                && definition.provenance.completion_check_timer <= 0
                && definition
                    .provenance
                    .completion_enemy_counts
                    .iter()
                    .all(|count| *count <= 0)
                && definition
                    .provenance
                    .completion_item_counts
                    .iter()
                    .all(|count| *count <= 0)
        })
        .expect("published catalog has an immediately complete type-2 task");
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(definition)],
        ..WorldMissionRuntime::default()
    };
    let nearby = [WorldMissionNearbyNpc {
        npc_type: definition.provenance.terminator_npc_type,
        npc_id: 7_777,
        position: [1.0, 0.0, 0.0],
        sight_range_server_units: 500,
    }];
    let actions = runtime
        .collect_automatic_end_requests(&[], Some([0.0; 3]), &nearby, content)
        .unwrap();
    assert_eq!(actions.end_requests.len(), 1);
    assert_eq!(actions.end_requests[0].npc_id, 7_777);
}
