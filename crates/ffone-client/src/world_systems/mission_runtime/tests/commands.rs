use super::*;

#[test]
fn login_instance_failure_restarts_nano_chain_without_pending_end_request() {
    let content = production_content();
    let failed = content.mission(1_142).expect("Nano lair completion task");
    let restart_id = failed.provenance.failure_outgoing_task_id;
    assert_eq!(restart_id, 1_140);
    for error_code in [1, 11, 12] {
        let mut load = PcLoadData0104::zeroed();
        write_running_quest(
            &mut load,
            4,
            ffone_protocol::RunningQuest0104 {
                task_id: failed.provenance.task_id,
                kill_npc_ids: failed.provenance.completion_enemy_ids,
                remaining_kill_counts: [0; 3],
                needed_item_ids: failed.provenance.completion_item_ids,
                needed_item_counts: failed.provenance.completion_item_counts,
            },
        );
        let mut runtime = WorldMissionRuntime::default();
        runtime.seed(&load, content).unwrap();
        let failure = WorldMissionServerEvent0104::TaskEndFailure(PcTaskFailure0104 {
            task_id: failed.provenance.task_id,
            error_code,
        });
        let outcome = runtime.apply_event(&failure, content).unwrap();
        assert_eq!(outcome.follow_up_start.unwrap().task_id, restart_id);
        assert_eq!(outcome.ui_resolution, None);
        // A duplicate must not send a second START or consume the pending restart.
        assert!(runtime.apply_event(&failure, content).is_err());
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskStopSuccess(PcTaskStopSuccess0104 {
                    task_id: failed.provenance.task_id,
                }),
                content,
            )
            .unwrap();
        assert!(
            runtime
                .pending_requests
                .contains(&PendingMissionUiRequest::TaskStart {
                    task_id: restart_id,
                    npc_id: 0,
                })
        );
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                    task_id: restart_id,
                    remaining_time: 0,
                }),
                content,
            )
            .unwrap();
        assert_eq!(runtime.active_task_ids(), vec![restart_id]);
        assert_eq!(
            runtime.selected_mission_id(),
            Some(failed.provenance.mission_id)
        );
    }
}

#[test]
fn automatic_reward_intent_is_durable_until_exact_acknowledgement() {
    let content = production_content();
    let definition = content
        .mission(1_270)
        .expect("primary automatic kill reward task");
    assert!(definition.provenance.reward_id > 0);
    assert!(definition.provenance.terminator_npc_type <= 0);
    let mut active = active_task(definition);
    active.remaining_enemy_counts = [0; 3];
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active],
        kill_progress_changed: true,
        ..WorldMissionRuntime::default()
    };

    let reward = runtime
        .collect_automatic_end_requests(&[], None, &[], content)
        .unwrap()
        .reward
        .expect("completed reward task must open Reward UI");
    assert_eq!(reward.task_id, definition.provenance.task_id);
    assert_eq!(runtime.pending_automatic_reward(), Some(reward));
    assert_eq!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .reward,
        Some(reward),
        "another modal frame must not consume the one-shot kill edge"
    );
    assert!(
        !runtime.acknowledge_automatic_reward(WorldMissionAutomaticReward {
            task_id: -1,
            npc_id: 0,
            escort_npc_id: 0,
        })
    );
    assert_eq!(runtime.pending_automatic_reward(), Some(reward));
    assert!(runtime.acknowledge_automatic_reward(reward));
    assert_eq!(runtime.pending_automatic_reward(), None);
    assert_eq!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .reward,
        None
    );
}
