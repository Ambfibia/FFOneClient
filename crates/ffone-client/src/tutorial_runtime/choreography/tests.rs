use std::collections::{BTreeMap, BTreeSet};

use crate::tutorial_choreography::*;
use crate::tutorial_presenter::tutorial_scene_presentation;

#[test]
fn all_ten_finite_coroutines_have_ordered_reference_timelines() {
    assert_eq!(TUTORIAL_SCENE_CHOREOGRAPHIES.len(), 10);
    for scene in TUTORIAL_SCENE_CHOREOGRAPHIES {
        let mut previous = 0.0;
        for action in scene.actions {
            assert!(
                action.at_seconds >= previous,
                "{} is unordered at source line {}",
                scene.coroutine,
                action.source_line
            );
            assert!(
                action.at_seconds <= scene.deterministic_duration_seconds,
                "{} action at {} exceeds deterministic duration {}",
                scene.coroutine,
                action.at_seconds,
                scene.deterministic_duration_seconds
            );
            previous = action.at_seconds;
        }
    }
}

#[test]
fn reference_completion_durations_match_the_presenter_contract() {
    for choreography in TUTORIAL_SCENE_CHOREOGRAPHIES {
        let presentation = tutorial_scene_presentation(choreography.presenter_audio_scene)
            .expect("finite choreography must have presenter audio");
        let expected = choreography
            .maximum_duration_seconds
            .unwrap_or(choreography.deterministic_duration_seconds);
        assert_eq!(
            presentation.duration_seconds, expected,
            "{:?} presenter duration drifted from the recovered coroutine",
            choreography.scene
        );
    }
}

#[test]
fn basic_move_skip_keeps_followers_and_deletes_both_reference_ranges() {
    let skip = tutorial_scene_choreography(TutorialScene::BasicMove)
        .unwrap()
        .skip
        .scene_specific;
    let mut spawned = BTreeSet::new();
    let mut deleted = BTreeSet::new();
    let mut final_pose = BTreeMap::new();
    for sourced in skip {
        match sourced.action {
            ChoreographyAction::Npc(NpcAction::Spawn(spawn)) => {
                spawned.insert(spawn.id);
            }
            ChoreographyAction::Npc(NpcAction::SpawnBatch(spawns)) => {
                spawned.extend(spawns.iter().map(|spawn| spawn.id));
            }
            ChoreographyAction::Npc(NpcAction::Delete(id)) => {
                deleted.insert(id);
            }
            ChoreographyAction::Npc(NpcAction::DeleteInclusive { first, last }) => {
                deleted.extend(first..=last);
            }
            ChoreographyAction::Npc(NpcAction::Animation { id, clip, once }) => {
                final_pose.insert(id, (clip, once));
            }
            _ => {}
        }
    }
    assert!(spawned.contains(&100));
    assert!(spawned.contains(&101));
    assert!(!deleted.contains(&100));
    assert!(!deleted.contains(&101));
    assert!((200..=205).all(|id| deleted.contains(&id)));
    assert!((304..=330).all(|id| deleted.contains(&id)));
    assert_eq!(final_pose.get(&100), Some(&("ready", false)));
    assert_eq!(final_pose.get(&101), Some(&("ready", false)));
    assert!(!skip.iter().any(|sourced| {
        matches!(
            sourced.action,
            ChoreographyAction::Npc(NpcAction::Move { id: 100 | 101, .. })
        )
    }));
}

#[test]
fn ben_and_numbuh_five_face_the_player_at_every_recovered_dialogue_beat() {
    let has_face = |scene: TutorialScene, at_seconds: f32, id: i32| {
        tutorial_scene_choreography(scene)
            .unwrap()
            .actions
            .iter()
            .any(|timed| {
                timed.at_seconds == at_seconds
                    && timed.action
                        == ChoreographyAction::Npc(NpcAction::Command {
                            id,
                            command: NpcCommand::SetAngleTo(EntityRef::Player),
                        })
            })
    };

    assert!(has_face(TutorialScene::BasicMove, 23.15, 100));
    assert!(has_face(TutorialScene::BasicMove, 23.15, 101));
    assert!(has_face(TutorialScene::BasicCombatB, 6.8, 100));
    assert!(has_face(TutorialScene::BasicCombatB, 6.8, 101));
    assert!(has_face(TutorialScene::BasicCombatB, 8.8, 101));
    assert!(has_face(TutorialScene::BasicCombatC, 4.0, 100));
    assert!(has_face(TutorialScene::BasicCombatC, 4.0, 101));
}

#[test]
fn basic_move_delegates_clip_events_and_preserves_explicit_follower_run_clips() {
    let actions = tutorial_scene_choreography(TutorialScene::BasicMove)
        .unwrap()
        .actions;
    let has_run = |id: i32| {
        actions.iter().any(|timed| {
            timed.at_seconds == 28.15
                && timed.action
                    == ChoreographyAction::Npc(NpcAction::Animation {
                        id,
                        clip: "run",
                        once: false,
                    })
        })
    };

    assert!(!actions.iter().any(|timed| matches!(
        timed.action,
        ChoreographyAction::Effect(EffectAction::AttachBone(attachment))
            if matches!(attachment.actor_id, 201 | 203 | 204 | 205)
                && matches!(attachment.effect_id, 38 | 734 | 736 | 751)
    )));
    assert!(actions.iter().any(|timed| {
        (timed.at_seconds - 11.65).abs() <= f32::EPSILON
            && timed.action
                == ChoreographyAction::Npc(NpcAction::Animation {
                    id: 205,
                    clip: "melee1",
                    once: false,
                })
    }));
    assert!(has_run(100));
    assert!(has_run(101));

    let ben_pose_at = |at_seconds: f32, clip: &'static str| {
        actions.iter().any(|timed| {
            timed.at_seconds == at_seconds
                && timed.action
                    == ChoreographyAction::Npc(NpcAction::Animation {
                        id: 101,
                        clip,
                        once: false,
                    })
        })
    };
    assert!(ben_pose_at(30.15, "jump"));
    assert!(ben_pose_at(30.15, "ready"));
    assert!(!ben_pose_at(31.15, "ready"));
}

#[test]
fn basic_combat_a_delegates_numbuh_five_melee_projectile_event() {
    let actions = tutorial_scene_choreography(TutorialScene::BasicCombatA)
        .unwrap()
        .actions;
    assert!(!actions.iter().any(|timed| matches!(
        timed.action,
        ChoreographyAction::Effect(EffectAction::AttachBone(attachment))
            if attachment.effect_id == 734 && attachment.actor_id == 100
    )));
}

#[test]
fn buttercup_flyby_freezes_the_live_camera_used_to_place_her() {
    let actions = tutorial_scene_choreography(TutorialScene::BasicMove)
        .unwrap()
        .actions;
    assert!(actions.iter().any(|timed| {
        timed.at_seconds == 8.15
            && timed.action
                == ChoreographyAction::Camera(CameraAction::Start(PositionExpr::Entity(
                    EntityRef::Camera,
                )))
    }));
    assert!(actions.iter().any(|timed| {
        timed.at_seconds == 8.15
            && timed.action == ChoreographyAction::Camera(CameraAction::FreezeCurrentTarget)
    }));
}

#[test]
fn loop_lifetimes_and_skip_postconditions_are_explicit() {
    let infection = tutorial_scene_choreography(TutorialScene::InfectionC).unwrap();
    assert_eq!(infection.loops.len(), 1);
    assert!(infection.loops[0].persists_after_scene);
    assert_eq!(infection.loops[0].natural_stop_at_seconds, None);
    assert_eq!(
        infection.loops[0].skip_action,
        LoopAction::StartIfAbsent("LairCollapse_Quake_LOOP")
    );

    for scene in [TutorialScene::NanoPowerA, TutorialScene::NanoPowerB] {
        let choreography = tutorial_scene_choreography(scene).unwrap();
        assert!(
            choreography.loops.iter().all(|loop_| {
                loop_.natural_stop_at_seconds.is_some()
                    && loop_.skip_action == LoopAction::StopIfPresent
                    && !loop_.persists_after_scene
            }),
            "{scene:?} must stop every scene-owned loop naturally and on skip"
        );
    }
}

#[test]
fn every_loop_lifetime_matches_its_explicit_timeline_and_skip_action() {
    for choreography in TUTORIAL_SCENE_CHOREOGRAPHIES {
        for lifetime in choreography.loops {
            assert!(
                choreography.actions.iter().any(|timed| {
                    (timed.at_seconds - lifetime.starts_at_seconds).abs() <= f32::EPSILON
                        && matches!(
                            timed.action,
                            ChoreographyAction::Loop(
                                LoopAction::Start(cue) | LoopAction::StartIfAbsent(cue)
                            ) if cue == lifetime.cue
                        )
                }),
                "{:?} loop {:?} metadata has no exact timeline start",
                choreography.scene,
                lifetime.cue
            );
            if let Some(stop_seconds) = lifetime.natural_stop_at_seconds {
                assert!(
                    choreography.actions.iter().any(|timed| {
                        (timed.at_seconds - stop_seconds).abs() <= f32::EPSILON
                            && timed.action
                                == ChoreographyAction::Loop(LoopAction::StopIfPresent)
                    }),
                    "{:?} loop {:?} metadata has no exact timeline stop",
                    choreography.scene,
                    lifetime.cue
                );
            }
            assert!(
                choreography
                    .skip
                    .scene_specific
                    .iter()
                    .chain(choreography.skip.common_cleanup.iter())
                    .any(|sourced| {
                        sourced.action == ChoreographyAction::Loop(lifetime.skip_action)
                    }),
                "{:?} loop {:?} metadata has no matching explicit skip action",
                choreography.scene,
                lifetime.cue
            );
        }
    }
}

#[test]
fn non_deterministic_waits_never_hide_inside_the_timeline() {
    let combat = tutorial_scene_choreography(TutorialScene::BasicCombatA).unwrap();
    assert!(matches!(
        combat.blocking_waits,
        [BlockingWait::AssetPreload {
            reached_at_seconds: 6.0,
            continuation_source_line: 3070,
            effect_id: 742,
            ..
        }]
    ));
    assert_eq!(
        combat
            .actions
            .iter()
            .filter(|action| {
                action.action == ChoreographyAction::Effect(EffectAction::Preload(742))
            })
            .count(),
        2
    );

    let finale = tutorial_scene_choreography(TutorialScene::NanoPowerB).unwrap();
    assert_eq!(finale.deterministic_duration_seconds, 46.15);
    assert_eq!(finale.maximum_duration_seconds, Some(47.15));
    assert!(matches!(
        finale.blocking_waits,
        [
            BlockingWait::EffectInstantiationRetry {
                reached_at_seconds: 13.25,
                effect_id: 741,
                shared_attempt_counter: "num",
                ..
            },
            BlockingWait::EffectInstantiationRetry {
                reached_at_seconds: 16.65,
                effect_id: 739,
                shared_attempt_counter: "num",
                ..
            }
        ]
    ));
}

#[test]
fn every_former_external_event_is_now_a_typed_action() {
    let combat = tutorial_scene_choreography(TutorialScene::BasicCombatB).unwrap();
    assert!(combat.actions.iter().any(|action| {
        action.source_line == 3253
            && action.action
                == ChoreographyAction::Picture(TutorialPictureAction::ShowCursor {
                    position: TutorialScreenPoint::new(
                        TutorialScreenAxis::WidthMinus(120),
                        TutorialScreenAxis::Pixels(50),
                    ),
                    resource: "tut_right",
                    direction: TutorialCursorDirection::Right,
                    legacy_pivot: 0,
                })
    }));
    assert!(combat.actions.iter().any(|action| {
        action.source_line == 3256
            && action.action == ChoreographyAction::Picture(TutorialPictureAction::HideAll)
    }));
    assert!(combat.actions.iter().any(|action| {
        action.source_line == 3284
            && action.action
                == ChoreographyAction::Sound(TutorialSoundAction::WorldOneShot {
                    cue: "Cyberus_Landing",
                    position: ClientVec3::new(567.0, -102.0, 658.0),
                })
    }));

    let infection_b = tutorial_scene_choreography(TutorialScene::InfectionB).unwrap();
    assert!(infection_b.actions.iter().any(|action| {
        action.source_line == 3982
            && action.action
                == ChoreographyAction::Sound(TutorialSoundAction::UiOneShot {
                    cue: "FusionButtercup_Stand1",
                })
    }));

    let infection_c = tutorial_scene_choreography(TutorialScene::InfectionC).unwrap();
    assert!(infection_c.actions.iter().any(|action| {
        action.source_line == 4138
            && action.action
                == ChoreographyAction::Picture(TutorialPictureAction::ShowCursor {
                    position: TutorialScreenPoint::new(
                        TutorialScreenAxis::WidthMinus(240),
                        TutorialScreenAxis::HeightMinus(180),
                    ),
                    resource: "tut_down",
                    direction: TutorialCursorDirection::Down,
                    legacy_pivot: 2,
                })
    }));

    let finale = tutorial_scene_choreography(TutorialScene::NanoPowerB).unwrap();
    assert!(finale.actions.iter().any(|action| {
        action.source_line == 4627
            && action.action
                == ChoreographyAction::Sound(TutorialSoundAction::Ambient { cue: "none" })
    }));
}

#[test]
fn combat_camera_tables_keep_live_temp_position_and_cyberus_shake_edges() {
    for (scene, source_line) in [
        (TutorialScene::BasicCombatA, 3055),
        (TutorialScene::BasicCombatB, 3232),
        (TutorialScene::BasicCombatC, 3358),
    ] {
        let choreography = tutorial_scene_choreography(scene).unwrap();
        assert!(choreography.actions.iter().any(|action| {
            action.source_line == source_line
                && action.action
                    == ChoreographyAction::Camera(CameraAction::StoredStart(
                        PositionExpr::Entity(EntityRef::Camera),
                    ))
        }));
    }

    let combat_b = tutorial_scene_choreography(TutorialScene::BasicCombatB).unwrap();
    for expected in [
        ChoreographyAction::Camera(CameraAction::FreezeCurrentTarget),
        ChoreographyAction::Camera(CameraAction::ForwardInterpolation(false)),
        ChoreographyAction::Camera(CameraAction::TargetRotation(RotationExpr::Euler(
            ClientVec3::new(10.0, 180.0, 0.0),
        ))),
        ChoreographyAction::Camera(CameraAction::ForwardInterpolation(true)),
    ] {
        assert!(
            combat_b
                .actions
                .iter()
                .any(|action| action.action == expected)
        );
    }
}

#[test]
fn camera_coroutine_locals_are_explicit_captures() {
    let infection_b = tutorial_scene_choreography(TutorialScene::InfectionB).unwrap();
    assert!(infection_b.actions.iter().any(|action| {
        action.at_seconds == 17.1
            && action.source_line == 3967
            && action.action
                == ChoreographyAction::Camera(CameraAction::CaptureTransform(
                    CameraCaptureSlot::InfectionReturnStart,
                ))
    }));
    assert!(infection_b.actions.iter().any(|action| {
        action.source_line == 3985
            && matches!(
                action.action,
                ChoreographyAction::Camera(CameraAction::Start(PositionExpr::OrientedOffset {
                    origin: PositionAnchor::CapturedCamera(
                        CameraCaptureSlot::InfectionReturnStart
                    ),
                    world_offset: ClientVec3::ZERO,
                    orientation: OrientationBasis::Euler(ClientVec3::ZERO),
                    local_offset: ClientVec3::ZERO,
                }))
            )
    }));

    let infection_c = tutorial_scene_choreography(TutorialScene::InfectionC).unwrap();
    assert!(infection_c.actions.iter().any(|action| {
        action.source_line == 4028
            && action.action
                == ChoreographyAction::Camera(CameraAction::CaptureTransform(
                    CameraCaptureSlot::InfectionEntry,
                ))
    }));
    assert!(infection_c.actions.iter().any(|action| {
        action.source_line == 4033
            && action.action
                == ChoreographyAction::Camera(CameraAction::Start(oriented(
                    PositionAnchor::Entity(EntityRef::Npc(4000)),
                    ClientVec3::new(0.0, 2.0, 0.0),
                    OrientationBasis::CapturedCameraYaw(CameraCaptureSlot::InfectionEntry),
                    ClientVec3::new(0.0, 0.0, -6.0),
                )))
    }));
}

#[test]
fn screen_formulas_evaluate_against_the_live_viewport() {
    assert_eq!(
        TutorialScreenPoint::new(
            TutorialScreenAxis::WidthMinus(120),
            TutorialScreenAxis::Pixels(50)
        )
        .evaluate(1920, 1080),
        [1800, 50]
    );
    assert_eq!(
        TutorialScreenPoint::new(
            TutorialScreenAxis::WidthMinus(240),
            TutorialScreenAxis::HeightMinus(180)
        )
        .evaluate(1920, 1080),
        [1680, 900]
    );
}
