use crate::tutorial_choreography::{
    ChoreographyAction, LoopAction, NpcAction, TUTORIAL_SCENE_CHOREOGRAPHIES, TimedAction,
};
use crate::tutorial_choreography_runtime::*;

fn action_lines(events: &[ChoreographyPlaybackEvent]) -> Vec<u32> {
    events
        .iter()
        .filter_map(|event| match event {
            ChoreographyPlaybackEvent::Action { source_line, .. } => Some(*source_line),
            _ => None,
        })
        .collect()
}

#[test]
fn finished_scene_preserves_terminal_orbit_look_at_until_next_scene() {
    let mut presentation = TutorialChoreographyPresentation::default();
    presentation.begin_scene(TutorialScene::BasicCombatA);
    presentation.apply_camera_action(CameraAction::LookAt(PositionExpr::Client(
        ClientVec3::new(567.0, -98.0, 678.0),
    )));
    presentation.camera.resolved_look_at = Some(Vec3::new(-567.0, -98.0, 678.0));
    let revision = presentation.camera.look_at_revision;

    presentation.apply(ChoreographyPlaybackEvent::Finished {
        scene: TutorialScene::BasicCombatA,
        completion: ChoreographyCompletion::Natural,
    });

    assert_eq!(presentation.camera.mode, CameraMode::None);
    assert_eq!(
        presentation.camera.resolved_look_at,
        Some(Vec3::new(-567.0, -98.0, 678.0))
    );
    assert_eq!(presentation.camera.look_at_revision, revision);

    presentation.begin_scene(TutorialScene::BasicCombatB);
    assert!(presentation.camera.look_at.is_none());
    assert!(presentation.camera.resolved_look_at.is_none());
}

#[test]
fn authored_camera_assignments_replace_shake_and_freeze_current_focus() {
    let mut presentation = TutorialChoreographyPresentation::default();
    presentation.camera.shake_start_offset = Vec3::ONE;
    presentation.camera.shake_target_offset = Vec3::ONE;

    presentation
        .apply_camera_action(CameraAction::Start(PositionExpr::Client(ClientVec3::ZERO)));
    presentation
        .apply_camera_action(CameraAction::Target(PositionExpr::Client(ClientVec3::ZERO)));
    assert_eq!(presentation.camera.shake_start_offset, Vec3::ZERO);
    assert_eq!(presentation.camera.shake_target_offset, Vec3::ZERO);

    presentation.apply_camera_action(CameraAction::FreezeCurrentTarget);
    assert_eq!(presentation.camera.freeze_target_revision, 1);
}

#[test]
fn long_frame_preserves_due_timed_action_source_order() {
    let mut player = TutorialChoreographyPlayer::default();
    let mut events = player.start(TutorialScene::BasicCombatA);
    events.extend(player.tick(12.0, false));
    let lines = action_lines(&events);
    let expected = tutorial_scene_choreography(TutorialScene::BasicCombatA)
        .unwrap()
        .actions
        .iter()
        .map(|action| action.source_line)
        .collect::<Vec<_>>();
    assert_eq!(lines, expected);
    assert!(matches!(
        events.last(),
        Some(ChoreographyPlaybackEvent::Finished {
            completion: ChoreographyCompletion::Natural,
            ..
        })
    ));
}

#[test]
fn skip_runs_scene_then_common_cleanup_and_final_fade() {
    let choreography = tutorial_scene_choreography(TutorialScene::BasicMove).unwrap();
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::BasicMove);
    let events = player.tick(0.0, true);
    let actions = events
        .iter()
        .filter_map(|event| match event {
            ChoreographyPlaybackEvent::Action { origin, action, .. } => {
                Some((*origin, *action))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actions.len(),
        choreography.skip.scene_specific.len() + choreography.skip.common_cleanup.len()
    );
    assert!(
        actions[..choreography.skip.scene_specific.len()]
            .iter()
            .all(|(origin, _)| *origin == ChoreographyActionOrigin::SkipSceneSpecific)
    );
    assert!(
        actions[choreography.skip.scene_specific.len()..]
            .iter()
            .all(|(origin, _)| *origin == ChoreographyActionOrigin::SkipCommonCleanup)
    );
    assert!(matches!(
        events[events.len() - 2],
        ChoreographyPlaybackEvent::FinalSkipFade {
            fade: SkipFinalFade::GameEventFadeIn
        }
    ));
    assert!(matches!(
        events.last(),
        Some(ChoreographyPlaybackEvent::Finished {
            completion: ChoreographyCompletion::Skipped,
            ..
        })
    ));
}

#[test]
fn basic_move_expands_actor_sequences_instead_of_teleporting_once() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::BasicMove);
    // The first recovered MoveNpc calls are due at 22.15 seconds.
    let events = player.tick(22.15, false);
    let npc_delta_samples = events
        .iter()
        .filter(|event| {
            matches!(
                event,
                ChoreographyPlaybackEvent::SequenceSample {
                    sample: FrameSequenceSample::NpcDelta { .. },
                    ..
                }
            )
        })
        .count();
    assert!(npc_delta_samples >= 100);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            ChoreographyPlaybackEvent::Action {
                action: ChoreographyAction::Npc(NpcAction::Move { .. }),
                ..
            }
        )
    }));
}

#[test]
fn infection_c_quake_loop_survives_natural_completion_and_skip() {
    for skip in [false, true] {
        let mut player = TutorialChoreographyPlayer::default();
        let _ = player.start(TutorialScene::InfectionC);
        let events = if skip {
            player.tick(0.0, true)
        } else {
            player.tick(47.9, false)
        };
        assert!(events.iter().any(|event| {
            matches!(
                event,
                ChoreographyPlaybackEvent::Action {
                    action: ChoreographyAction::Loop(
                        LoopAction::Start("LairCollapse_Quake_LOOP")
                            | LoopAction::StartIfAbsent("LairCollapse_Quake_LOOP")
                    ),
                    ..
                }
            )
        }));
        assert!(player.is_loop_owned("LairCollapse_Quake_LOOP"));
    }
}

#[test]
fn runtime_skip_postconditions_match_every_loop_lifetime_contract() {
    for choreography in TUTORIAL_SCENE_CHOREOGRAPHIES {
        if choreography.loops.is_empty() {
            continue;
        }
        let mut player = TutorialChoreographyPlayer::default();
        let _ = player.start(choreography.scene);
        let events = player.skip();
        for lifetime in choreography.loops {
            assert!(events.iter().any(|event| {
                matches!(
                    event,
                    ChoreographyPlaybackEvent::Action {
                        origin: ChoreographyActionOrigin::SkipSceneSpecific,
                        action: ChoreographyAction::Loop(action),
                        ..
                    } if *action == lifetime.skip_action
                )
            }));
            let expected_owned = matches!(
                lifetime.skip_action,
                LoopAction::Start(_) | LoopAction::StartIfAbsent(_)
            );
            assert_eq!(
                player.is_loop_owned(lifetime.cue),
                expected_owned,
                "{:?} loop {:?} runtime skip state contradicts metadata",
                choreography.scene,
                lifetime.cue
            );
        }
    }
}

#[test]
fn basic_combat_a_blocks_after_the_second_es742_preload() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::BasicCombatA);

    let reached = player.tick_with_wait_resolution(6.0, false, |_| false);
    assert!((player.elapsed_seconds() - 6.0).abs() <= TIME_EPSILON);
    assert!(player.presentation_elapsed_seconds() < 6.0);
    assert!(reached.iter().any(|event| {
        matches!(
            event,
            ChoreographyPlaybackEvent::Action {
                source_line: 3062,
                action: ChoreographyAction::Effect(
                    crate::tutorial_choreography::EffectAction::Preload(742)
                ),
                ..
            }
        )
    }));
    assert!(reached.iter().any(|event| {
        matches!(
            event,
            ChoreographyPlaybackEvent::BlockingWaitReached {
                wait: BlockingWait::AssetPreload { effect_id: 742, .. }
            }
        )
    }));
    assert!(!action_lines(&reached).contains(&3070));

    assert!(
        player
            .tick_with_wait_resolution(30.0, false, |_| false)
            .is_empty()
    );
    assert!((player.elapsed_seconds() - 6.0).abs() <= TIME_EPSILON);

    let resumed = player.tick_with_wait_resolution(0.0, false, |_| true);
    assert!((player.presentation_elapsed_seconds() - 6.0).abs() <= TIME_EPSILON);
    assert!(action_lines(&resumed).contains(&3070));
    assert!(action_lines(&resumed).contains(&3071));
}

#[test]
fn nano_power_b_holds_same_timestamp_continuation_until_es741_resolves() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::NanoPowerB);

    let reached = player.tick_with_wait_resolution(13.25, false, |_| false);
    let reached_lines = action_lines(&reached);
    assert!(reached_lines.contains(&4697));
    assert!(reached_lines.contains(&4701));
    assert!(!reached_lines.contains(&4717));
    assert!(reached.iter().any(|event| {
        matches!(
            event,
            ChoreographyPlaybackEvent::BlockingWaitReached {
                wait: BlockingWait::EffectInstantiationRetry { effect_id: 741, .. }
            }
        )
    }));

    let resumed = player.tick_with_wait_resolution(0.0, false, |_| true);
    assert!(action_lines(&resumed).contains(&4717));
}

#[test]
fn nano_power_b_finishes_at_nominal_duration_when_both_initial_spawns_succeed() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::NanoPowerB);

    let first_wait = player.tick_with_wait_resolution(13.25, false, |_| true);
    assert!(!action_lines(&first_wait).contains(&4717));
    let first_continuation = player.tick_with_wait_resolution(0.0, false, |_| true);
    assert!(action_lines(&first_continuation).contains(&4717));

    let second_wait = player.tick_with_wait_resolution(3.4, false, |_| true);
    assert!(!action_lines(&second_wait).contains(&4844));
    let second_continuation = player.tick_with_wait_resolution(0.0, false, |_| true);
    assert!(action_lines(&second_continuation).contains(&4844));

    let before = player.tick_with_wait_resolution(29.499, false, |_| true);
    assert!(
        !before
            .iter()
            .any(|event| matches!(event, ChoreographyPlaybackEvent::Finished { .. }))
    );
    let finish = player.tick_with_wait_resolution(0.001, false, |_| true);
    assert!(matches!(
        finish.last(),
        Some(ChoreographyPlaybackEvent::Finished {
            scene: TutorialScene::NanoPowerB,
            completion: ChoreographyCompletion::Natural
        })
    ));
}

#[test]
fn nano_power_b_uses_one_shared_second_for_both_failed_retry_loops() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::NanoPowerB);

    let first_wait = player.tick_with_wait_resolution(13.25, false, |_| false);
    assert_eq!(
        first_wait
            .iter()
            .filter(|event| matches!(
                event,
                ChoreographyPlaybackEvent::BlockingWaitReached { .. }
            ))
            .count(),
        1
    );
    let mut retry_lines =
        action_lines(&player.tick_with_wait_resolution(0.0, false, |_| false));
    for _ in 0..10 {
        retry_lines.extend(action_lines(&player.tick_with_wait_resolution(
            0.1,
            false,
            |_| false,
        )));
    }
    assert_eq!(retry_lines.iter().filter(|&&line| line == 4711).count(), 10);

    let second_wait = player.tick_with_wait_resolution(3.4, false, |_| false);
    assert_eq!(
        second_wait
            .iter()
            .filter(|event| matches!(
                event,
                ChoreographyPlaybackEvent::BlockingWaitReached { .. }
            ))
            .count(),
        1
    );
    let second_continuation = player.tick_with_wait_resolution(0.0, false, |_| false);
    assert!(action_lines(&second_continuation).contains(&4844));
    assert!(!action_lines(&second_continuation).contains(&4841));

    let before = player.tick_with_wait_resolution(29.499, false, |_| false);
    assert!(
        !before
            .iter()
            .any(|event| matches!(event, ChoreographyPlaybackEvent::Finished { .. }))
    );
    let finish = player.tick_with_wait_resolution(0.001, false, |_| false);
    assert!(matches!(
        finish.last(),
        Some(ChoreographyPlaybackEvent::Finished {
            scene: TutorialScene::NanoPowerB,
            completion: ChoreographyCompletion::Natural
        })
    ));
}

#[test]
fn successful_finale_retry_still_yields_the_mandatory_tenth_of_a_second() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::NanoPowerB);
    let _ = player.tick_with_wait_resolution(13.25, false, |_| false);

    let retry = player.tick_with_wait_resolution(0.0, false, |_| false);
    assert_eq!(
        action_lines(&retry)
            .into_iter()
            .filter(|&line| line == 4711)
            .count(),
        1
    );
    assert!(!action_lines(&retry).contains(&4717));

    let early = player.tick_with_wait_resolution(0.099, false, |_| true);
    assert!(early.is_empty());
    assert!(!action_lines(&early).contains(&4717));

    let resumed = player.tick_with_wait_resolution(0.001, false, |_| true);
    assert!(action_lines(&resumed).contains(&4717));
}

#[test]
fn second_finale_loop_retries_the_original_es739_reference() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::NanoPowerB);
    let _ = player.tick_with_wait_resolution(13.25, false, |_| true);
    let _ = player.tick_with_wait_resolution(0.0, false, |_| true);
    let _ = player.tick_with_wait_resolution(3.4, false, |_| true);

    let retry = player.tick_with_wait_resolution(0.0, false, |wait| {
        matches!(
            wait,
            BlockingWait::EffectInstantiationRetry { effect_id: 741, .. }
        )
    });
    assert!(action_lines(&retry).contains(&4841));
    assert!(!action_lines(&retry).contains(&4844));
}

#[test]
fn implemented_native_actions_do_not_emit_speculative_issues() {
    let mut player = TutorialChoreographyPlayer::default();
    let _ = player.start(TutorialScene::BasicMove);
    let _ = player.tick(31.15, false);
    assert!(player.take_issues().is_empty());
}

#[test]
fn same_scene_start_is_idempotent() {
    let mut player = TutorialChoreographyPlayer::default();
    let first = player.start(TutorialScene::BasicMove);
    let duplicate = player.start(TutorialScene::BasicMove);
    assert!(!first.is_empty());
    assert!(duplicate.is_empty());
}

#[test]
fn timed_action_type_remains_copy_for_zero_allocation_playback() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<TimedAction>();
}
