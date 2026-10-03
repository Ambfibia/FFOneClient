use super::*;

pub(super) fn production_content() -> &'static TutorialMissionContent {
    static CONTENT: OnceLock<TutorialMissionContent> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let assets = AssetLocator::open(root).expect("open published game assets");
        TutorialMissionContent::open(&assets).expect("open published TableData mission catalog")
    })
}

pub(super) fn active_task(definition: &TutorialMissionDefinition) -> WorldActiveMissionTask {
    WorldActiveMissionTask {
        task_id: definition.provenance.task_id,
        remaining_enemy_ids: definition.provenance.completion_enemy_ids,
        remaining_enemy_counts: definition.provenance.completion_enemy_counts,
        remaining_time_millis: None,
    }
}

#[test]
fn nano_mission_completion_cue_requires_one_authoritative_end() {
    let content = production_content();
    let definition = content
        .missions()
        .find(|definition| {
            definition.mission_type == TutorialMissionType::Nano
                && definition.provenance.outgoing_task_id == 0
        })
        .expect("published catalog has a terminal Nano task");
    let task_id = definition.provenance.task_id;
    let mut runtime = WorldMissionRuntime::default();
    let start = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id,
                remaining_time: definition.provenance.grant_timer,
            }),
            content,
        )
        .unwrap();
    assert_eq!(start.completion_kind, None);

    let end = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 { task_id }),
            content,
        )
        .unwrap();
    assert_eq!(
        end.completion_kind,
        Some(WorldMissionCompletionKind::Mission)
    );
    assert_eq!(
        end.completion_kind.unwrap().legacy_audio_true_name(),
        "Mission_Completed"
    );
    assert!(
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 { task_id }),
                content,
            )
            .is_err()
    );
}

#[test]
fn unsolicited_start_between_task_end_and_follow_up_keeps_tracked_chain() {
    let content = production_content();
    let mut runtime = WorldMissionRuntime::default();
    let task_id = 2253;
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id,
                remaining_time: 0,
            }),
            content,
        )
        .unwrap();
    let selected = runtime.selected_mission_id();
    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 { task_id }),
            content,
        )
        .unwrap();
    assert_eq!(
        outcome.completion_kind,
        Some(WorldMissionCompletionKind::Task)
    );
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: 576,
                remaining_time: 0,
            }),
            content,
        )
        .unwrap();
    assert_eq!(runtime.selected_mission_id(), selected);
    let follow_up = outcome.follow_up_start.unwrap();
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: follow_up.task_id,
                remaining_time: 0,
            }),
            content,
        )
        .unwrap();
    assert_eq!(runtime.selected_task_id(content), Some(follow_up.task_id));
    assert_eq!(
        runtime.take_dialogue_edges(),
        vec![
            (task_id, WorldMissionDialogueEdge::Start),
            (task_id, WorldMissionDialogueEdge::Success),
            (576, WorldMissionDialogueEdge::Start),
            (follow_up.task_id, WorldMissionDialogueEdge::Start),
        ]
    );
    assert!(runtime.take_dialogue_edges().is_empty());
}

#[test]
fn every_published_objective_preserves_clean_timer_enemy_and_item_slots() {
    let content = production_content();
    for definition in content.missions() {
        let mut remaining_enemy_counts = definition.provenance.completion_enemy_counts;
        for remaining in &mut remaining_enemy_counts {
            if *remaining > 0 {
                *remaining -= 1;
            }
        }
        let remaining_time_millis = (definition.provenance.grant_timer > 0).then_some(42_000);
        let runtime = WorldMissionRuntime {
            active_tasks: vec![WorldActiveMissionTask {
                task_id: definition.provenance.task_id,
                remaining_enemy_ids: definition.provenance.completion_enemy_ids,
                remaining_enemy_counts,
                remaining_time_millis,
            }],
            selected_mission_id: Some(definition.provenance.mission_id),
            ..WorldMissionRuntime::default()
        };
        let inventory = definition
            .provenance
            .completion_item_ids
            .into_iter()
            .zip(definition.provenance.completion_item_counts)
            .filter(|(item_id, needed)| *item_id > 0 && *needed > 0)
            .map(|(item_id, needed)| ItemBase0104 {
                item_type: 8,
                item_id: i16::try_from(item_id).expect("quest-item ID fits protocol i16"),
                option: needed - 1,
                time_limit: 0,
            })
            .collect::<Vec<_>>();
        let objective = runtime
            .current_objective(&inventory, content)
            .unwrap_or_else(|error| {
                panic!(
                    "task {} current-objective projection failed: {error}",
                    definition.provenance.task_id
                )
            })
            .expect("one selected active task projects an objective");

        assert_eq!(objective.task_id, definition.provenance.task_id);
        assert_eq!(objective.title, definition.title);
        assert_eq!(objective.objective, definition.objective);
        assert_eq!(
            objective.remaining_time_seconds,
            remaining_time_millis.map(|remaining| remaining / 1_000)
        );
        let expected_enemies = definition
            .provenance
            .completion_enemy_ids
            .into_iter()
            .zip(definition.provenance.completion_enemy_counts)
            .filter(|(npc_type, needed)| *npc_type > 0 && *needed > 0)
            .collect::<Vec<_>>();
        assert_eq!(objective.enemies.len(), expected_enemies.len());
        for (actual, (npc_type, needed)) in objective.enemies.iter().zip(expected_enemies) {
            assert_eq!(actual.content_id, npc_type);
            assert_eq!(actual.needed, needed);
            assert_eq!(actual.complete, 1);
            assert_eq!(
                actual.name,
                content.gameplay_npc(npc_type).unwrap().name,
                "task {} enemy slot must use m_iNpcName",
                definition.provenance.task_id
            );
        }
        let expected_items = definition
            .provenance
            .completion_item_ids
            .into_iter()
            .zip(definition.provenance.completion_item_counts)
            .filter(|(item_id, needed)| *item_id > 0 && *needed > 0)
            .collect::<Vec<_>>();
        assert_eq!(objective.quest_items.len(), expected_items.len());
        for (actual, (item_id, needed)) in objective.quest_items.iter().zip(expected_items) {
            assert_eq!(actual.content_id, item_id);
            assert_eq!(actual.needed, needed);
            assert_eq!(
                actual.complete,
                quest_item_count(&inventory, item_id).min(needed)
            );
            assert_eq!(
                actual.name,
                content.quest_item_name(item_id).unwrap(),
                "task {} item slot must use m_iItemName",
                definition.provenance.task_id
            );
        }
    }
}

#[test]
fn unsolicited_nonterminal_end_rejection_preserves_active_nano_task() {
    let content = production_content();
    let definition = content.mission(1_142).unwrap();
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(definition)],
        ..WorldMissionRuntime::default()
    };
    assert!(
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskEndFailure(PcTaskFailure0104 {
                    task_id: 1_142,
                    error_code: 13,
                }),
                content
            )
            .is_err()
    );
    assert_eq!(runtime.active_task_ids(), vec![1_142]);
    assert!(runtime.pending_requests.is_empty());
}

#[test]
fn repeat_flags_use_clean_eight_bits_per_i64_group() {
    let content = production_content();
    // The current published catalog has no positive repeat rows, so use
    // a synthetic wire flag while retaining the production seed path.
    let flag = 17;
    let (word, bit) = repeat_flag_position(flag).unwrap();
    let mut load = PcLoadData0104::zeroed();
    write_i64_at(
        &mut load,
        PcLoadData0104::REPEAT_QUEST_FLAGS_OFFSET + word * 8,
        1_i64 << bit,
    );
    let mut runtime = WorldMissionRuntime::default();
    runtime.seed(&load, content).unwrap();
    assert!(runtime.repeat_completed(flag));
}

/// Task 576 is the Eduardo escort that terminates Nano mission 516 inside
/// instance 11. It must be offered and acceptable, or Nano 4 — and with it
/// every later level and Fusion Matter cap — can never be reached.
#[test]
fn the_eduardo_escort_that_terminates_a_nano_chain_is_offered_and_acceptable() {
    let content = production_content();
    let runtime = WorldMissionRuntime::default();
    let escort = content
        .mission(576)
        .expect("published catalog has the Eduardo escort task");
    assert_eq!(escort.provenance.task_type, 6);
    assert_eq!(escort.provenance.required_instance_id, 11);
    assert_eq!(escort.provenance.escort_def_npc_type, 1_007);
    assert_eq!(escort.provenance.terminator_npc_type, 1_011);
    assert_eq!(runtime.check_accept_task(576, content), Ok(()));

    let owned_nanos = BTreeSet::from([1, 2, 3]);
    assert!(runtime.can_start_task(escort, 100, 0, &owned_nanos, &[], content));
}

#[test]
fn published_defeat_tasks_are_available_acceptable_and_start_authoritatively() {
    let content = production_content();
    let representatives = [
        (
            1,
            "Dee Dee's Dilemmas (Part 1 of 4)",
            3,
            701,
            [97, 0, 0],
            [5, 0, 0],
            0,
            2,
            0,
        ),
        (
            48,
            "Dee Dee's Dilemmas (Part 4 of 4)",
            6,
            0,
            [293, 0, 0],
            [6, 0, 0],
            0,
            49,
            0,
        ),
        (
            501,
            "Master of Dishaster",
            7,
            0,
            [274, 0, 0],
            [8, 0, 0],
            180,
            677,
            7,
        ),
    ];

    for (
        task_id,
        title,
        mission_id,
        start_npc_type,
        enemy_ids,
        enemy_counts,
        grant_timer,
        outgoing_task_id,
        failure_outgoing_task_id,
    ) in representatives
    {
        let definition = content.mission(task_id).expect("published defeat task");
        assert_eq!(definition.title.as_str(), title);
        assert_eq!(definition.mission_type, TutorialMissionType::World);
        assert_eq!(definition.provenance.mission_id, mission_id);
        assert_eq!(definition.provenance.task_type, 5);
        assert_eq!(definition.provenance.required_instance_id, 0);
        assert_eq!(definition.provenance.start_npc_type, start_npc_type);
        assert_eq!(definition.provenance.completion_enemy_ids, enemy_ids);
        assert_eq!(definition.provenance.completion_enemy_counts, enemy_counts);
        assert_eq!(definition.provenance.terminator_npc_type, 0);
        assert_eq!(definition.provenance.reward_id, 0);
        assert_eq!(definition.provenance.grant_timer, grant_timer);
        assert_eq!(definition.provenance.outgoing_task_id, outgoing_task_id);
        assert_eq!(
            definition.provenance.failure_outgoing_task_id,
            failure_outgoing_task_id
        );

        let mut runtime = WorldMissionRuntime::default();
        for required_mission in definition
            .provenance
            .required_missions
            .iter()
            .copied()
            .filter(|mission_id| *mission_id > 0)
        {
            runtime.completed_mission_ids.insert(required_mission);
        }
        let owned_nanos = BTreeSet::new();
        assert!(runtime.can_start_task(definition, 100, 0, &owned_nanos, &[], content,));
        assert_eq!(runtime.check_accept_task(task_id, content), Ok(()));

        if start_npc_type > 0 {
            let (offers, _) = runtime
                .npc_entries(
                    start_npc_type,
                    77,
                    "published defeat fixture",
                    100,
                    0,
                    &owned_nanos,
                    &[],
                    content,
                )
                .unwrap();
            assert!(offers.iter().any(|entry| entry.task_id == task_id));
            assert!(
                runtime
                    .npc_has_available_or_completable_mission(
                        start_npc_type,
                        100,
                        0,
                        &owned_nanos,
                        &[],
                        content,
                    )
                    .0
            );
        }

        runtime
            .record_request(PendingMissionUiRequest::TaskStart {
                task_id,
                npc_id: start_npc_type,
            })
            .unwrap();
        let outcome = runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                    task_id,
                    remaining_time: grant_timer,
                }),
                content,
            )
            .unwrap();
        assert_eq!(
            outcome.ui_resolution,
            Some(WorldMissionUiResolution::TaskStartAccepted(task_id))
        );
        assert_eq!(runtime.active_task_ids(), vec![task_id]);
        assert_eq!(runtime.active_tasks()[0].remaining_enemy_ids, enemy_ids);
        assert_eq!(
            runtime.active_tasks()[0].remaining_enemy_counts,
            enemy_counts
        );
        assert_eq!(
            runtime.active_tasks()[0].remaining_time_millis,
            (grant_timer > 0).then_some(i64::from(grant_timer) * 1_000)
        );
    }
}

#[test]
fn published_defeat_task_one_decrements_kills_and_sends_exact_automatic_end() {
    let content = production_content();
    let definition = content.mission(1).expect("published defeat task 1");
    let mut runtime = WorldMissionRuntime::default();
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: 1,
                remaining_time: 0,
            }),
            content,
        )
        .unwrap();

    for remaining in [4, 3, 2, 1] {
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::KillQuestNpc(PcKillQuestNpcsSuccess0104 {
                    npc_type_id: 97,
                }),
                content,
            )
            .unwrap();
        assert_eq!(
            runtime.active_tasks()[0].remaining_enemy_counts,
            [remaining, 0, 0]
        );
        assert!(
            runtime
                .collect_automatic_end_requests(&[], None, &[], content)
                .unwrap()
                .end_requests
                .is_empty()
        );
    }

    runtime
        .apply_event(
            &WorldMissionServerEvent0104::KillQuestNpc(PcKillQuestNpcsSuccess0104 {
                npc_type_id: 97,
            }),
            content,
        )
        .unwrap();
    assert_eq!(runtime.active_tasks()[0].remaining_enemy_counts, [0; 3]);
    let actions = runtime
        .collect_automatic_end_requests(&[], None, &[], content)
        .unwrap();
    assert_eq!(
        actions.end_requests,
        vec![PcTaskEndRequest0104 {
            task_id: 1,
            npc_id: 0,
            reward_box_1: 0,
            reward_box_2: 0,
            pack_padding: [0; 2],
            escort_npc_id: 0,
        }]
    );
    assert!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .end_requests
            .is_empty()
    );

    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 { task_id: 1 }),
            content,
        )
        .unwrap();
    assert_eq!(outcome.ui_resolution, None);
    assert_eq!(
        outcome.completion_kind,
        Some(WorldMissionCompletionKind::Task)
    );
    assert_eq!(
        outcome.completion_kind.unwrap().legacy_audio_true_name(),
        "Task_Completed"
    );
    assert_eq!(
        outcome.follow_up_start,
        Some(PcTaskStartRequest0104 {
            task_id: definition.provenance.outgoing_task_id,
            npc_id: 0,
            escort_npc_id: 0,
        })
    );
    assert!(runtime.active_tasks().is_empty());
    assert_eq!(
        runtime.pending_request(),
        Some(PendingMissionUiRequest::TaskStart {
            task_id: 2,
            npc_id: 0,
        })
    );
    assert_eq!(runtime.pending_automatic_reward(), None);
    assert!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .end_requests
            .is_empty()
    );
}

#[test]
fn published_defeat_task_501_timer_failure_and_cleanup_follow_clean_edges() {
    let content = production_content();
    let definition = content.mission(501).expect("published defeat task 501");
    assert_eq!(definition.provenance.grant_timer, 180);
    assert_eq!(definition.provenance.failure_outgoing_task_id, 7);

    let mut runtime = WorldMissionRuntime::default();
    runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: 501,
                remaining_time: 180,
            }),
            content,
        )
        .unwrap();
    runtime.tick(179.0);
    assert!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .end_requests
            .is_empty()
    );

    runtime.tick(1.0);
    let actions = runtime
        .collect_automatic_end_requests(&[], None, &[], content)
        .unwrap();
    assert_eq!(
        actions.end_requests,
        vec![PcTaskEndRequest0104 {
            task_id: 501,
            npc_id: 0,
            reward_box_1: 0,
            reward_box_2: 0,
            pack_padding: [0; 2],
            escort_npc_id: 0,
        }]
    );

    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndFailure(PcTaskFailure0104 {
                task_id: 501,
                error_code: 1,
            }),
            content,
        )
        .unwrap();
    assert_eq!(outcome.ui_resolution, None);
    assert_eq!(
        outcome.follow_up_start,
        Some(PcTaskStartRequest0104 {
            task_id: 7,
            npc_id: 0,
            escort_npc_id: 0,
        })
    );
    assert!(runtime.active_tasks().is_empty());
    assert_eq!(
        runtime.pending_request(),
        Some(PendingMissionUiRequest::TaskStart {
            task_id: 7,
            npc_id: 0,
        })
    );
    assert_eq!(runtime.pending_automatic_reward(), None);
    assert!(
        runtime
            .collect_automatic_end_requests(&[], None, &[], content)
            .unwrap()
            .end_requests
            .is_empty()
    );
}

#[test]
fn batched_world_map_availability_matches_the_single_npc_reducer() {
    let content = production_content();
    let active_definition = content.mission(2_254).expect("primary tutorial hunt task");
    let runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(active_definition)],
        ..WorldMissionRuntime::default()
    };
    let owned_nanos = BTreeSet::new();
    let batch = runtime.mission_availability_by_npc(100, 0, &owned_nanos, &[], content);
    let npc_types = content
        .missions()
        .flat_map(|definition| {
            [
                definition.provenance.start_npc_type,
                definition.provenance.terminator_npc_type,
            ]
        })
        .filter(|npc_type| *npc_type > 0)
        .collect::<BTreeSet<_>>();

    for npc_type in npc_types {
        assert_eq!(
            batch.get(&npc_type).copied().unwrap_or_default(),
            runtime.npc_has_available_or_completable_mission(
                npc_type,
                100,
                0,
                &owned_nanos,
                &[],
                content,
            ),
            "NPC type {npc_type}",
        );
    }
}

#[test]
fn unsolicited_known_start_and_active_end_are_authoritative() {
    let content = production_content();
    let definition = content
        .missions()
        .find(|definition| definition.provenance.outgoing_task_id == 0)
        .expect("published catalog has a terminal task");
    let mut runtime = WorldMissionRuntime::default();
    let start = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: definition.provenance.task_id,
                remaining_time: definition.provenance.grant_timer,
            }),
            content,
        )
        .unwrap();
    assert_eq!(start.ui_resolution, None);
    assert_eq!(
        runtime.active_task_ids(),
        vec![definition.provenance.task_id]
    );

    let end = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskEndSuccess(PcTaskEndSuccess0104 {
                task_id: definition.provenance.task_id,
            }),
            content,
        )
        .unwrap();
    assert_eq!(end.ui_resolution, None);
    assert_eq!(
        end.completion_kind,
        Some(WorldMissionCompletionKind::Mission)
    );
    assert_eq!(
        end.completion_kind.unwrap().legacy_audio_true_name(),
        "Mission_Completed"
    );
    assert!(runtime.active_tasks().is_empty());
    assert!(
        runtime
            .completed_mission_ids()
            .contains(&definition.provenance.mission_id)
    );
}

#[test]
fn task_stop_success_is_authoritative_and_only_resolves_matching_ui() {
    let content = production_content();
    let stopped = content.mission(500).expect("primary task 500");
    let remaining = content
        .missions()
        .find(|definition| definition.provenance.mission_id != stopped.provenance.mission_id)
        .expect("published catalog has another mission");
    let pending = PendingMissionUiRequest::TaskStop {
        task_id: stopped.provenance.task_id,
    };
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(remaining), active_task(stopped)],
        completed_mission_ids: BTreeSet::from([stopped.provenance.mission_id]),
        selected_mission_id: Some(stopped.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };
    runtime.record_request(pending).unwrap();

    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStopSuccess(PcTaskStopSuccess0104 {
                task_id: stopped.provenance.task_id,
            }),
            content,
        )
        .unwrap();

    assert_eq!(
        outcome.ui_resolution,
        Some(WorldMissionUiResolution::TaskStopAccepted(
            stopped.provenance.task_id
        ))
    );
    assert_eq!(
        runtime.active_task_ids(),
        vec![remaining.provenance.task_id]
    );
    assert_eq!(
        runtime.selected_mission_id(),
        Some(remaining.provenance.mission_id)
    );
    assert!(
        !runtime
            .completed_mission_ids()
            .contains(&stopped.provenance.mission_id)
    );
    assert_eq!(runtime.pending_request(), None);

    let mut proactive = WorldMissionRuntime {
        active_tasks: vec![active_task(stopped)],
        selected_mission_id: Some(stopped.provenance.mission_id),
        ..WorldMissionRuntime::default()
    };
    let outcome = proactive
        .apply_event(
            &WorldMissionServerEvent0104::TaskStopSuccess(PcTaskStopSuccess0104 {
                task_id: stopped.provenance.task_id,
            }),
            content,
        )
        .unwrap();
    assert_eq!(outcome.ui_resolution, None);
    assert!(proactive.active_tasks().is_empty());
    assert_eq!(proactive.selected_mission_id(), None);
}

#[test]
fn task_stop_failure_rejects_only_the_sole_pending_stop() {
    let content = production_content();
    let stopped = content.mission(500).expect("primary task 500");
    let pending = PendingMissionUiRequest::TaskStop {
        task_id: stopped.provenance.task_id,
    };
    let mut runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(stopped)],
        ..WorldMissionRuntime::default()
    };
    runtime.record_request(pending).unwrap();
    let outcome = runtime
        .apply_event(
            &WorldMissionServerEvent0104::TaskStopFailure(PcTaskStopFailure0104 {
                error_code: 1,
            }),
            content,
        )
        .unwrap();
    assert_eq!(
        outcome.ui_resolution,
        Some(WorldMissionUiResolution::RequestRejected(
            stopped.provenance.task_id
        ))
    );
    assert_eq!(runtime.active_task_ids(), vec![stopped.provenance.task_id]);
    assert_eq!(runtime.pending_request(), None);

    let mut ambiguous = WorldMissionRuntime::default();
    ambiguous.record_request(pending).unwrap();
    ambiguous
        .record_request(PendingMissionUiRequest::TaskStop {
            task_id: stopped.provenance.task_id + 1,
        })
        .unwrap();
    let outcome = ambiguous
        .apply_event(
            &WorldMissionServerEvent0104::TaskStopFailure(PcTaskStopFailure0104 {
                error_code: 2,
            }),
            content,
        )
        .unwrap();
    assert_eq!(outcome.ui_resolution, None);
    assert_eq!(ambiguous.pending_request(), Some(pending));
}
