use crate::tutorial_logic::*;

fn stage_decision(stage: TutorialStage, observation: TutorialObservation) -> TutorialDecision {
    evaluate_stage(stage, &observation).expect("later tutorial stage should evaluate")
}

fn event(
    mut observation: TutorialObservation,
    event: TutorialEvent,
    value: i32,
) -> TutorialObservation {
    observation.event_flags[event.index()] = value;
    observation
}

fn completed(mut observation: TutorialObservation) -> TutorialObservation {
    observation.scene_completion = SceneCompletion::Natural;
    observation
}

#[test]
fn every_chapter_03_through_07_legacy_stage_has_an_explicit_evaluator_arm() {
    let stages = [
        TutorialStage::Combat(CombatStage::AwaitIntro),
        TutorialStage::Combat(CombatStage::CombatIntro),
        TutorialStage::Combat(CombatStage::TargetHostile),
        TutorialStage::Combat(CombatStage::FirstAttack),
        TutorialStage::Combat(CombatStage::KillThree),
        TutorialStage::Combat(CombatStage::CyberusIntro),
        TutorialStage::Combat(CombatStage::FightCyberus),
        TutorialStage::Combat(CombatStage::PlayerWasHit),
        TutorialStage::Combat(CombatStage::PlanetFusionOutro),
        TutorialStage::Minimap(MinimapStage::AwaitSequence),
        TutorialStage::Minimap(MinimapStage::MinimapSequence),
        TutorialStage::Minimap(MinimapStage::ApproachNumbuhTwo),
        TutorialStage::Mission(MissionStage::Approach),
        TutorialStage::Mission(MissionStage::TargetAndTalk),
        TutorialStage::Mission(MissionStage::MissionMenu),
        TutorialStage::Mission(MissionStage::ObjectiveCombat),
        TutorialStage::Mission(MissionStage::NumbuhTwoResponse),
        TutorialStage::Mission(MissionStage::OpenMenu),
        TutorialStage::Mission(MissionStage::SelectJournal),
        TutorialStage::Mission(MissionStage::JournalIntro),
        TutorialStage::Mission(MissionStage::JournalMissionTab),
        TutorialStage::Mission(MissionStage::JournalObjective),
        TutorialStage::Mission(MissionStage::JournalDetail),
        TutorialStage::Mission(MissionStage::CloseJournal),
        TutorialStage::Mission(MissionStage::ReturnToNumbuhTwo),
        TutorialStage::Mission(MissionStage::NearNumbuhTwo),
        TutorialStage::Mission(MissionStage::RewardMenu),
        TutorialStage::Mission(MissionStage::ClaimReward),
        TutorialStage::Mission(MissionStage::CloseReward),
        TutorialStage::Mission(MissionStage::Handoff),
        TutorialStage::Mission(MissionStage::NumbuhTwoDelay),
        TutorialStage::Mission(MissionStage::AcceptMission),
        TutorialStage::Mission(MissionStage::CloseAccept),
        TutorialStage::Infection(InfectionStage::ApproachButtercup),
        TutorialStage::Infection(InfectionStage::TalkButtercup),
        TutorialStage::Infection(InfectionStage::SelectMission),
        TutorialStage::Infection(InfectionStage::InfectedFlythrough),
        TutorialStage::Infection(InfectionStage::TravelToGate),
        TutorialStage::Infection(InfectionStage::SelectAttendant),
        TutorialStage::Infection(InfectionStage::WarpFromTechSquare),
        TutorialStage::Infection(InfectionStage::OutsidePortal),
        TutorialStage::Infection(InfectionStage::TalkDexter),
        TutorialStage::Infection(InfectionStage::ApproachFusion),
        TutorialStage::Infection(InfectionStage::FightFusion),
        TutorialStage::Infection(InfectionStage::NanoCreation),
        TutorialStage::Infection(InfectionStage::ExitPrompt),
        TutorialStage::Infection(InfectionStage::WarpOut),
        TutorialStage::Infection(InfectionStage::ButtercupDelay),
        TutorialStage::Infection(InfectionStage::CloseMission),
        TutorialStage::Infection(InfectionStage::SelectDexterMission),
        TutorialStage::Infection(InfectionStage::SelectTentacles),
        TutorialStage::Infection(InfectionStage::WarpIntoLair),
        TutorialStage::Infection(InfectionStage::DexterCutscene),
        TutorialStage::NanoPower(NanoPowerStage::AwaitCollapse),
        TutorialStage::NanoPower(NanoPowerStage::CollapseCutscene),
        TutorialStage::NanoPower(NanoPowerStage::CrossBridge),
        TutorialStage::NanoPower(NanoPowerStage::DemoMonsterCutscene),
        TutorialStage::NanoPower(NanoPowerStage::SummonNano),
        TutorialStage::NanoPower(NanoPowerStage::UseNanoPower),
        TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo),
        TutorialStage::NanoPower(NanoPowerStage::FinaleCutscene),
        TutorialStage::NanoPower(NanoPowerStage::Exit),
    ];

    for stage in stages {
        let decision = stage_decision(stage, TutorialObservation::default());
        assert_eq!(decision.current_stage, stage);
    }
}

#[test]
fn combat_preserves_timeout_priority_direct_step_write_and_scene_handoffs() {
    let target = TutorialStage::Combat(CombatStage::TargetHostile);
    let observation = TutorialObservation {
        timer_seconds: 11,
        ui: TutorialUiObservation {
            hostile_target_selected: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let decision = stage_decision(target, observation);
    assert_eq!(decision.next_stage, target, "timeout wins over target");
    assert_eq!(decision.reset, Some(TutorialProgressReset::InitStep));

    let fight = TutorialStage::Combat(CombatStage::FightCyberus);
    let observation = event(
        TutorialObservation {
            timer_seconds: 2,
            ..Default::default()
        },
        TutorialEvent::DamageUser,
        1,
    );
    let decision = stage_decision(fight, observation);
    assert_eq!(
        decision.next_stage,
        TutorialStage::Combat(CombatStage::PlayerWasHit)
    );
    assert_eq!(decision.reset, Some(TutorialProgressReset::DirectStepWrite));
    assert!(decision.intents.contains(&TutorialIntent::AttackNpc(1005)));

    let cyberus_intro = TutorialStage::Combat(CombatStage::CyberusIntro);
    assert!(
        !stage_decision(cyberus_intro, TutorialObservation::default())
            .intents
            .contains(&TutorialIntent::AttackNpc(1005)),
        "Cyber must remain passive while BasicCombatEvent_B still owns the scene"
    );
    for scene_completion in [SceneCompletion::Natural, SceneCompletion::Skipped] {
        let decision = stage_decision(
            cyberus_intro,
            TutorialObservation {
                scene_completion,
                ..Default::default()
            },
        );
        assert_eq!(
            decision.next_stage,
            TutorialStage::Combat(CombatStage::FightCyberus)
        );
        assert!(
            decision.intents.contains(&TutorialIntent::AttackNpc(1005)),
            "Cyber must attack on the same handoff that releases the cutscene"
        );
        assert!(
            decision
                .intents
                .contains(&TutorialIntent::StartReminderTimer),
            "the immediate attack must still restart the repeating attack interval"
        );
    }

    let outro = TutorialStage::Combat(CombatStage::PlanetFusionOutro);
    assert!(!stage_decision(outro, TutorialObservation::default()).transitioned());
    let decision = stage_decision(outro, completed(TutorialObservation::default()));
    assert_eq!(
        decision.next_stage,
        TutorialStage::Minimap(MinimapStage::AwaitSequence)
    );
    assert_eq!(decision.reset, Some(TutorialProgressReset::InitChapter));
    assert!(decision.intents.contains(&TutorialIntent::FinishScene {
        scene: TutorialScene::BasicCombatC,
        completion: SceneCompletion::Natural,
    }));
}

#[test]
fn minimap_uses_damaged_as_near_and_strict_timeout_precedes_arrival() {
    let sequence = TutorialStage::Minimap(MinimapStage::MinimapSequence);
    let mut observation = TutorialObservation::default();
    observation.npcs.numbuh_two.distance = Some(500.0);
    observation.npcs.numbuh_two.damaged = true;
    assert_eq!(
        stage_decision(sequence, observation).next_stage,
        TutorialStage::Minimap(MinimapStage::ApproachNumbuhTwo)
    );

    let approach = TutorialStage::Minimap(MinimapStage::ApproachNumbuhTwo);
    let mut observation = TutorialObservation {
        timer_seconds: 26,
        ..Default::default()
    };
    observation.npcs.numbuh_two.distance = Some(1.0);
    let decision = stage_decision(approach, observation);
    assert_eq!(decision.next_stage, approach);
    assert_eq!(decision.reset, Some(TutorialProgressReset::InitStep));
}

#[test]
fn mission_covers_100_101_102_ui_delays_and_transient_19() {
    let delay = TutorialStage::Mission(MissionStage::NumbuhTwoDelay);
    assert!(!stage_decision(delay, TutorialObservation::default()).transitioned());
    let decision = stage_decision(
        delay,
        TutorialObservation {
            delay_completed: true,
            ..Default::default()
        },
    );
    assert_eq!(
        decision.next_stage,
        TutorialStage::Mission(MissionStage::AcceptMission)
    );

    let accept = TutorialStage::Mission(MissionStage::AcceptMission);
    let observation = event(
        TutorialObservation {
            timer_seconds: 26,
            ..Default::default()
        },
        TutorialEvent::TaskStart,
        1,
    );
    assert_eq!(
        stage_decision(accept, observation).next_stage,
        accept,
        "step 101 timeout branch executes before task acceptance"
    );

    let close = TutorialStage::Mission(MissionStage::CloseAccept);
    let decision = stage_decision(close, TutorialObservation::default());
    assert_eq!(
        decision.next_stage,
        TutorialStage::Mission(MissionStage::ObjectiveCombat)
    );
    assert!(decision.intents.iter().any(|intent| matches!(
        intent,
        TutorialIntent::SpawnNpc(TutorialNpcSpawn { id: 2000, .. })
    )));

    let journal = TutorialStage::Mission(MissionStage::JournalIntro);
    let observation = TutorialObservation {
        delay_completed: true,
        ui: TutorialUiObservation {
            journal_mode: TutorialJournalMode::Other,
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        stage_decision(journal, observation).next_stage,
        TutorialStage::Mission(MissionStage::JournalMissionTab)
    );

    let handoff = TutorialStage::Mission(MissionStage::Handoff);
    let decision = stage_decision(handoff, TutorialObservation::default());
    assert_eq!(decision.next_stage, handoff);
    assert_eq!(
        decision.ambiguities,
        vec![TutorialReferenceAmbiguity::TransientStageHasNoStandaloneBranch(handoff)]
    );
}

#[test]
fn mission_journal_walkthrough_reaches_return_to_numbuh_two() {
    let response = TutorialStage::Mission(MissionStage::NumbuhTwoResponse);
    let open_menu = stage_decision(
        response,
        TutorialObservation {
            delay_completed: true,
            ..Default::default()
        },
    );
    assert_eq!(
        open_menu.next_stage,
        TutorialStage::Mission(MissionStage::OpenMenu),
    );
    assert!(open_menu.intents.contains(&TutorialIntent::PushInputFilter));

    let select_journal = stage_decision(
        open_menu.next_stage,
        TutorialObservation {
            ui: TutorialUiObservation {
                nanocom_main_menu_visible: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    assert_eq!(
        select_journal.next_stage,
        TutorialStage::Mission(MissionStage::SelectJournal),
    );

    let journal_open = TutorialObservation {
        delay_completed: true,
        ui: TutorialUiObservation {
            journal_mode: TutorialJournalMode::Other,
            ..Default::default()
        },
        ..Default::default()
    };
    let intro = stage_decision(select_journal.next_stage, journal_open.clone());
    assert_eq!(
        intro.next_stage,
        TutorialStage::Mission(MissionStage::JournalIntro),
    );
    assert!(intro.intents.contains(&TutorialIntent::LockUi));
    assert!(
        intro
            .intents
            .contains(&TutorialIntent::StartDelayMillis(4_000))
    );

    let mission_tab = stage_decision(intro.next_stage, journal_open.clone());
    assert_eq!(
        mission_tab.next_stage,
        TutorialStage::Mission(MissionStage::JournalMissionTab),
    );
    assert!(
        mission_tab
            .intents
            .contains(&TutorialIntent::StartDelayMillis(2_500))
    );

    let objective = stage_decision(mission_tab.next_stage, journal_open.clone());
    assert_eq!(
        objective.next_stage,
        TutorialStage::Mission(MissionStage::JournalObjective),
    );
    assert!(
        objective
            .intents
            .contains(&TutorialIntent::StartDelayMillis(2_500))
    );

    let detail = stage_decision(objective.next_stage, journal_open.clone());
    assert_eq!(
        detail.next_stage,
        TutorialStage::Mission(MissionStage::JournalDetail),
    );
    assert!(
        detail
            .intents
            .contains(&TutorialIntent::StartDelayMillis(4_200))
    );

    let close = stage_decision(detail.next_stage, journal_open);
    assert_eq!(
        close.next_stage,
        TutorialStage::Mission(MissionStage::CloseJournal),
    );
    assert!(close.intents.contains(&TutorialIntent::PopInputFilter));
    assert!(close.intents.contains(&TutorialIntent::UnlockUi));

    let return_to_numbuh_two = stage_decision(close.next_stage, TutorialObservation::default());
    assert_eq!(
        return_to_numbuh_two.next_stage,
        TutorialStage::Mission(MissionStage::ReturnToNumbuhTwo),
    );
}

#[test]
fn mission_journal_walkthrough_fails_closed_until_each_real_ui_observation() {
    let open_menu = TutorialStage::Mission(MissionStage::OpenMenu);
    assert!(!stage_decision(open_menu, TutorialObservation::default()).transitioned());

    let select_journal = TutorialStage::Mission(MissionStage::SelectJournal);
    assert!(!stage_decision(select_journal, TutorialObservation::default()).transitioned());
    let timeout = stage_decision(
        select_journal,
        TutorialObservation {
            timer_seconds: 26,
            ui: TutorialUiObservation {
                journal_mode: TutorialJournalMode::Other,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    assert_eq!(timeout.next_stage, open_menu);

    for stage in [
        MissionStage::JournalIntro,
        MissionStage::JournalMissionTab,
        MissionStage::JournalObjective,
        MissionStage::JournalDetail,
    ] {
        let stage = TutorialStage::Mission(stage);
        assert!(
            !stage_decision(
                stage,
                TutorialObservation {
                    ui: TutorialUiObservation {
                        journal_mode: TutorialJournalMode::Other,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .transitioned(),
            "{stage:?} must wait for its exact Retrobution delay"
        );
        assert!(
            !stage_decision(
                stage,
                TutorialObservation {
                    delay_completed: true,
                    ..Default::default()
                },
            )
            .transitioned(),
            "{stage:?} must not advance after the real Journal closes"
        );
    }

    let close_journal = TutorialStage::Mission(MissionStage::CloseJournal);
    assert!(
        !stage_decision(
            close_journal,
            TutorialObservation {
                ui: TutorialUiObservation {
                    journal_mode: TutorialJournalMode::Other,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .transitioned()
    );
}

#[test]
fn mission_accept_close_chain_spawns_exact_scamper_and_oil_ogre() {
    let accept = TutorialStage::Mission(MissionStage::AcceptMission);
    let mut accepted_observation =
        event(TutorialObservation::default(), TutorialEvent::TaskStart, 1);
    accepted_observation.npcs.numbuh_two = ObservedNpc {
        distance: Some(1.0),
        interacting: true,
        ..Default::default()
    };
    let accepted = stage_decision(accept, accepted_observation);
    assert_eq!(
        accepted.next_stage,
        TutorialStage::Mission(MissionStage::CloseAccept)
    );
    assert_eq!(accepted.reset, Some(TutorialProgressReset::InitStep));

    let mut close_observation = event(
        TutorialObservation::default(),
        TutorialEvent::NpcIconClose,
        1,
    );
    close_observation.npcs.numbuh_two = ObservedNpc {
        distance: Some(1.0),
        interacting: true,
        ..Default::default()
    };
    assert_eq!(
        stage_decision(accepted.next_stage, close_observation.clone()).next_stage,
        TutorialStage::Mission(MissionStage::CloseAccept),
        "the close event waits for the queued interaction clear to reach the NPC snapshot"
    );

    close_observation.npcs.numbuh_two.interacting = false;
    let objective = stage_decision(accepted.next_stage, close_observation);
    assert_eq!(
        objective.next_stage,
        TutorialStage::Mission(MissionStage::ObjectiveCombat)
    );
    assert_eq!(objective.reset, Some(TutorialProgressReset::InitStep));
    let spawns = objective
        .intents
        .iter()
        .filter_map(|intent| match intent {
            TutorialIntent::SpawnNpc(spawn) => Some((spawn.id, spawn.npc_type)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(spawns, vec![(2000, 2374), (1100, 2676)]);
}

#[test]
fn mission_reward_handoff_emits_exact_task_and_spawn_intents() {
    let close = TutorialStage::Mission(MissionStage::CloseReward);
    let decision = stage_decision(close, TutorialObservation::default());
    assert_eq!(
        decision.next_stage,
        TutorialStage::Infection(InfectionStage::ApproachButtercup)
    );
    assert_eq!(decision.reset, Some(TutorialProgressReset::InitChapter));
    let start_task = decision
        .intents
        .iter()
        .position(|intent| *intent == TutorialIntent::StartTask(2250))
        .expect("task 2250 must start during the mission handoff");
    assert_eq!(
        decision.intents.get(start_task + 1),
        Some(&TutorialIntent::ClearEventFlags),
        "legacy InitChapter clears TaskStart after StartTask(2250)"
    );
    assert!(
        decision
            .intents
            .contains(&TutorialIntent::SetFusionMatter(220))
    );
    assert!(decision.intents.contains(&TutorialIntent::StartDialogue(
        TutorialDialogue::TalkButtercup
    )));
    assert!(decision.intents.iter().any(|intent| matches!(
        intent,
        TutorialIntent::SpawnNpc(TutorialNpcSpawn {
            id: BUTTERCUP_ID,
            npc_type: 2672,
            ..
        })
    )));
}

#[test]
fn approaching_buttercup_does_not_clear_full_fusion_matter_early() {
    let stage = TutorialStage::Infection(InfectionStage::ApproachButtercup);
    let mut observation = TutorialObservation::default();
    observation.npcs.buttercup.distance = Some(0.0);
    let decision = stage_decision(stage, observation);

    assert_eq!(
        decision.next_stage,
        TutorialStage::Infection(InfectionStage::TalkButtercup)
    );
    assert!(!decision.intents.iter().any(|intent| matches!(
        intent,
        TutorialIntent::SetFusionMatter(0)
            | TutorialIntent::StopDialogue(TutorialDialogue::TalkButtercup)
    )));
}

#[test]
fn infection_covers_101_102_200_201_109_and_warp_events() {
    let delay = TutorialStage::Infection(InfectionStage::ButtercupDelay);
    assert!(!stage_decision(delay, TutorialObservation::default()).transitioned());
    assert_eq!(
        stage_decision(
            delay,
            TutorialObservation {
                delay_completed: true,
                ..Default::default()
            }
        )
        .next_stage,
        TutorialStage::Infection(InfectionStage::CloseMission)
    );

    let close = TutorialStage::Infection(InfectionStage::CloseMission);
    assert_eq!(
        stage_decision(close, TutorialObservation::default()).next_stage,
        TutorialStage::Infection(InfectionStage::InfectedFlythrough)
    );

    let tentacles = TutorialStage::Infection(InfectionStage::SelectTentacles);
    let mut observation = TutorialObservation::default();
    observation.npcs.fusion_portal.interacting = true;
    assert_eq!(
        stage_decision(tentacles, observation).next_stage,
        TutorialStage::Infection(InfectionStage::WarpIntoLair)
    );

    let warp = TutorialStage::Infection(InfectionStage::WarpIntoLair);
    let observation = event(TutorialObservation::default(), TutorialEvent::NpcWarp, 1);
    let decision = stage_decision(warp, observation);
    assert_eq!(
        decision.next_stage,
        TutorialStage::Infection(InfectionStage::TalkDexter)
    );
    assert!(
        decision
            .intents
            .contains(&TutorialIntent::SetInstanceMap(true))
    );

    let dexter = TutorialStage::Infection(InfectionStage::SelectDexterMission);
    let observation = event(TutorialObservation::default(), TutorialEvent::TaskStart, 1);
    assert_eq!(
        stage_decision(dexter, observation).next_stage,
        TutorialStage::Infection(InfectionStage::DexterCutscene)
    );
}

#[test]
fn tutorial_nano_boss_death_wins_over_distance_and_duplicate_notifications() {
    for stage in [
        InfectionStage::DexterCutscene,
        InfectionStage::ApproachFusion,
        InfectionStage::FightFusion,
    ] {
        for completion in [SceneCompletion::Natural, SceneCompletion::Skipped] {
            for deaths in [1, 2, 3] {
                for distance in [Some(0.0), Some(10.0), Some(100.0), None] {
                    let mut observation = event(
                        TutorialObservation {
                            scene_completion: completion,
                            ..Default::default()
                        },
                        TutorialEvent::DeadNpc,
                        deaths,
                    );
                    observation.npcs.fusion_buttercup = ObservedNpc {
                        distance,
                        damaged: true,
                        interacting: false,
                    };
                    let decision = stage_decision(TutorialStage::Infection(stage), observation);
                    assert_eq!(
                        decision.next_stage,
                        TutorialStage::Infection(InfectionStage::NanoCreation),
                        "{stage:?}, {completion:?}, deaths={deaths}, distance={distance:?}"
                    );
                    assert!(decision.intents.contains(&TutorialIntent::EndTask(2254)));
                    assert!(
                        decision
                            .intents
                            .contains(&TutorialIntent::StartScene(TutorialScene::InfectionC))
                    );
                    let grant = stage_decision(
                        decision.next_stage,
                        TutorialObservation {
                            scene_completion: completion,
                            ..Default::default()
                        },
                    );
                    assert!(grant.intents.contains(&TutorialIntent::EquipNano {
                        slot: 0,
                        nano_id: 1,
                        skill_id: 1
                    }));
                }
            }
        }
    }
}

#[test]
fn infection_210_scene_completion_replays_same_frame_fallthrough() {
    let scene = TutorialStage::Infection(InfectionStage::DexterCutscene);
    assert!(!stage_decision(scene, TutorialObservation::default()).transitioned());

    let mut observation = completed(TutorialObservation::default());
    observation.npcs.fusion_buttercup.distance = Some(10.0);
    let decision = stage_decision(scene, observation);
    assert_eq!(
        decision.next_stage,
        TutorialStage::Infection(InfectionStage::FightFusion)
    );
    assert!(
        decision
            .intents
            .contains(&TutorialIntent::StartReminderTimer)
    );
    assert!(
        decision
            .intents
            .contains(&TutorialIntent::PlayNpcAnimation {
                id: LAIR_DEXTER_ID,
                clip: "melee1event",
                once: false,
            })
    );
    assert!(decision.intents.contains(&TutorialIntent::FaceNpc {
        id: LAIR_DEXTER_ID,
        target: FUSION_BUTTERCUP_ID,
    }));
    assert!(
        decision
            .intents
            .contains(&TutorialIntent::PlayNpcAnimation {
                id: FUSION_BUTTERCUP_ID,
                clip: "melee1",
                once: false,
            })
    );

    let observation = event(
        completed(TutorialObservation::default()),
        TutorialEvent::DeadNpc,
        1,
    );
    let decision = stage_decision(scene, observation);
    assert_eq!(
        decision.next_stage,
        TutorialStage::Infection(InfectionStage::NanoCreation)
    );
    assert!(decision.intents.contains(&TutorialIntent::EndTask(2254)));
    assert!(
        decision
            .intents
            .contains(&TutorialIntent::StartScene(TutorialScene::InfectionC))
    );
}

#[test]
fn nano_locale_branch_and_all_scene_completions_are_explicit() {
    let await_stage = TutorialStage::NanoPower(NanoPowerStage::AwaitCollapse);
    let english = stage_decision(await_stage, TutorialObservation::default());
    assert!(
        english
            .intents
            .contains(&TutorialIntent::ConfigureDemoMonster { dont_kill: true })
    );
    let localized = stage_decision(
        await_stage,
        TutorialObservation {
            locale: TutorialLocaleBranch::Localized,
            ..Default::default()
        },
    );
    assert!(
        localized
            .intents
            .contains(&TutorialIntent::ConfigureDemoMonster { dont_kill: false })
    );

    let collapse = TutorialStage::NanoPower(NanoPowerStage::CollapseCutscene);
    assert_eq!(
        stage_decision(collapse, completed(TutorialObservation::default())).next_stage,
        TutorialStage::NanoPower(NanoPowerStage::CrossBridge)
    );

    let demo = TutorialStage::NanoPower(NanoPowerStage::DemoMonsterCutscene);
    let natural = stage_decision(demo, completed(TutorialObservation::default()));
    let skipped = stage_decision(
        demo,
        TutorialObservation {
            scene_completion: SceneCompletion::Skipped,
            ..Default::default()
        },
    );
    let natural_spawn = natural
        .intents
        .iter()
        .find_map(|intent| match intent {
            TutorialIntent::SpawnNpc(spawn) if spawn.id == DEMO_MONSTER_ID => Some(*spawn),
            _ => None,
        })
        .unwrap();
    let skipped_spawn = skipped
        .intents
        .iter()
        .find_map(|intent| match intent {
            TutorialIntent::SpawnNpc(spawn) if spawn.id == DEMO_MONSTER_ID => Some(*spawn),
            _ => None,
        })
        .unwrap();
    assert_eq!(natural_spawn.position.z, 1_700);
    assert_eq!(skipped_spawn.position.z, 1_200);

    let finale = TutorialStage::NanoPower(NanoPowerStage::FinaleCutscene);
    let decision = stage_decision(finale, completed(TutorialObservation::default()));
    assert_eq!(
        decision.next_stage,
        TutorialStage::NanoPower(NanoPowerStage::Exit)
    );
    assert!(decision.intents.contains(&TutorialIntent::ExitTutorial));
}

#[test]
fn localized_demo_death_can_skip_step_6_but_english_cannot() {
    let summon = TutorialStage::NanoPower(NanoPowerStage::SummonNano);
    let dead = event(
        TutorialObservation {
            locale: TutorialLocaleBranch::Localized,
            ..Default::default()
        },
        TutorialEvent::DeadNpc,
        1,
    );
    assert_eq!(
        stage_decision(summon, dead).next_stage,
        TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo)
    );

    let dead = event(TutorialObservation::default(), TutorialEvent::DeadNpc, 1);
    assert_eq!(stage_decision(summon, dead).next_stage, summon);
    // The localized branch must not enter the stun lesson after its
    // target died in the same frame as the Nano finished loading.
    for deaths in [1, 2] {
        let mut simultaneous = event(
            TutorialObservation {
                locale: TutorialLocaleBranch::Localized,
                ..Default::default()
            },
            TutorialEvent::DeadNpc,
            deaths,
        );
        simultaneous.event_flags[TutorialEvent::NanoActive.index()] = 1;
        assert_eq!(
            stage_decision(summon, simultaneous).next_stage,
            TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo)
        );
    }
}

#[test]
fn strict_timer_comparisons_do_not_fire_at_the_boundary() {
    let target = TutorialStage::Combat(CombatStage::TargetHostile);
    assert!(
        !stage_decision(
            target,
            TutorialObservation {
                timer_seconds: 10,
                ..Default::default()
            }
        )
        .transitioned()
    );

    let close_journal = TutorialStage::Mission(MissionStage::CloseJournal);
    let still_open = TutorialObservation {
        timer_seconds: 12,
        ui: TutorialUiObservation {
            journal_mode: TutorialJournalMode::Other,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(!stage_decision(close_journal, still_open).transitioned());
}

#[test]
fn observation_can_import_all_progress_owned_flags_and_timer() {
    let mut progress = TutorialProgress::default();
    progress.receive_event(TutorialEvent::DeadNpc, 0);
    progress.receive_event(TutorialEvent::NpcWarp, 0);
    progress.set_timer_seconds(26);

    let observation = TutorialObservation::default().with_progress(&progress);
    assert_eq!(observation.event_value(TutorialEvent::DeadNpc), 1);
    assert_eq!(observation.event_value(TutorialEvent::NpcWarp), 1);
    assert_eq!(observation.timer_seconds, 26);
}
