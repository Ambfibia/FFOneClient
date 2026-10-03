use super::*;

#[test]
fn authoritative_start_replaces_the_active_task_from_the_same_mission() {
    let content = production_content();
    let previous = content.mission(5_229).expect("primary task 5229");
    let replacement = content.mission(5_230).expect("primary task 5230");
    assert_eq!(
        previous.provenance.mission_id,
        replacement.provenance.mission_id
    );
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(previous)],
        selected_mission_id: Some(previous.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };

    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: replacement.provenance.task_id,
                remaining_time: replacement.provenance.grant_timer,
            }),
            content,
        )
        .unwrap();

    assert_eq!(
        runtime.active_task_ids(),
        vec![replacement.provenance.task_id]
    );
    assert_eq!(
        runtime.selected_mission_id(),
        Some(replacement.provenance.mission_id)
    );
}

#[test]
fn end_success_ignores_dangling_outgoing_but_preserves_final_row_semantics() {
    let content = production_content();
    let dangling = content.mission(1_197).expect("primary dangling task 1197");
    assert_eq!(dangling.provenance.outgoing_task_id, 1_200);
    assert!(content.mission(1_200).is_err());
    assert!(!content.is_final_serialized_task(1_197).unwrap());
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(dangling)],
        selected_mission_id: Some(dangling.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };
    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 {
                task_id: dangling.provenance.task_id,
            }),
            content,
        )
        .unwrap();
    assert!(runtime.active_tasks().is_empty());
    assert_eq!(outcome.follow_up_start, None);
    assert!(
        !runtime
            .completed_mission_ids()
            .contains(&dangling.provenance.mission_id)
    );
    assert_eq!(
        runtime.selected_mission_id(),
        Some(dangling.provenance.mission_id),
        "clean leaves a nonterminal dangling mission selected"
    );

    let final_with_outgoing = content.mission(5_233).expect("primary task 5233");
    assert!(content.is_final_serialized_task(5_233).unwrap());
    assert_eq!(final_with_outgoing.provenance.outgoing_task_id, 5_229);
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(final_with_outgoing)],
        selected_mission_id: Some(final_with_outgoing.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };
    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 {
                task_id: final_with_outgoing.provenance.task_id,
            }),
            content,
        )
        .unwrap();
    assert!(
        runtime
            .completed_mission_ids()
            .contains(&final_with_outgoing.provenance.mission_id)
    );
    assert_eq!(
        outcome.follow_up_start.map(|request| request.task_id),
        Some(5_229)
    );
    assert_eq!(runtime.selected_mission_id(), None);
}

#[test]
fn kill_and_timer_auto_end_edges_register_once() {
    let content = production_content();
    let kill_definition = content
        .missions()
        .find(|definition| {
            definition.provenance.terminator_npc_type <= 0
                && definition.provenance.reward_id <= 0
                && definition
                    .provenance
                    .completion_enemy_counts
                    .iter()
                    .sum::<i32>()
                    > 0
        })
        .expect("published catalog has a rewardless no-terminator kill task");
    let kill_index = kill_definition
        .provenance
        .completion_enemy_ids
        .iter()
        .position(|npc_type| *npc_type > 0)
        .unwrap();
    let mut kill_active = active_task(kill_definition);
    kill_active.remaining_enemy_counts = [0; 3];
    kill_active.remaining_enemy_counts[kill_index] = 1;
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![kill_active],
        ..WorldMissionRuntime::default()
    };
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::KillQuestNpc(PcKillQuestNpcsSuccess0104 {
                npc_type_id: kill_definition.provenance.completion_enemy_ids[kill_index],
            }),
            content,
        )
        .unwrap();
    let actions = runtime
        .collect_automatic_end_requests(&[], None, &[], content)
        .unwrap();
    assert_eq!(actions.end_requests.len(), 1);
    assert_eq!(
        actions.end_requests[0].task_id,
        kill_definition.provenance.task_id
    );
    assert!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .end_requests
            .is_empty()
    );

    let timer_definition = content
        .missions()
        .find(|definition| definition.provenance.grant_timer > 0)
        .expect("published catalog has a grant-timer task");
    let mut timer_runtime = WorldMissionRuntime::default();
    timer_runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: timer_definition.provenance.task_id,
                remaining_time: 1,
            }),
            content,
        )
        .unwrap();
    timer_runtime.tick(1.0);
    let actions = timer_runtime
        .collect_automatic_end_requests(&[], None, &[], content)
        .unwrap();
    assert_eq!(actions.end_requests.len(), 1);
    assert_eq!(actions.end_requests[0].npc_id, 0);
}
