use super::*;

pub(super) fn evaluate_infection(
    stage: InfectionStage,
    observation: &TutorialObservation,
) -> TutorialDecision {
    let current = TutorialStage::Infection(stage);
    match stage {
        InfectionStage::ApproachButtercup => {
            if observation.npcs.buttercup.within_or_damaged(10.0) {
                // Clean `TalkBCup_1` owns both the 27-second full-FM display
                // and its eventual reset. Proximity only advances the chapter
                // state; it must not cancel that auxiliary coroutine or clear
                // the 220/220 meter early.
                return TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::TalkButtercup),
                    TutorialProgressReset::InitStep,
                );
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::TalkButtercup => {
            if observation.npcs.buttercup.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::SelectMission),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::LockUi);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::SelectMission => {
            if observation.event_is(TutorialEvent::TaskStart, 1) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::ButtercupDelay),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::UnlockUi,
                    TutorialIntent::StartDelayMillis(6_300),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::ButtercupDelay => {
            if !observation.delay_completed {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Infection(InfectionStage::CloseMission),
                TutorialProgressReset::InitStep,
            );
            decision.intents.push(TutorialIntent::StartReminderTimer);
            decision
        }
        InfectionStage::CloseMission => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if !observation.npcs.buttercup.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::InfectedFlythrough),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    spawn(
                        TECH_SQUARE_ATTENDANT_ID,
                        2694,
                        59_600,
                        78_400,
                        -8_200,
                        Some(291),
                    ),
                    TutorialIntent::StartScene(TutorialScene::InfectionA),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::InfectedFlythrough => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Infection(InfectionStage::TravelToGate),
                TutorialProgressReset::InitStep,
            );
            finish_scene(&mut decision, TutorialScene::InfectionA, observation);
            decision.intents.push(TutorialIntent::DeleteNpc(1012));
            decision
        }
        InfectionStage::TravelToGate => {
            if observation
                .npcs
                .tech_square_attendant
                .within_or_damaged(20.0)
            {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::SelectAttendant),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.tech_square_attendant.interacting {
                return enter_tech_square_warp_prompt(current);
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::SelectAttendant => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.tech_square_attendant.interacting {
                return enter_tech_square_warp_prompt(current);
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::WarpFromTechSquare => {
            if observation.timer_exceeds(25) {
                let mut decision = enter_tech_square_warp_prompt(current);
                decision.next_stage = current;
                return decision;
            }

            let mut decision = TutorialDecision::unchanged(current);
            if observation.event_is(TutorialEvent::NpcIconClose, 1) {
                decision.intents.push(TutorialIntent::HideTutorialPointer);
            }
            if observation.event_is(TutorialEvent::NpcWarp, 1) {
                decision.next_stage = TutorialStage::Infection(InfectionStage::OutsidePortal);
                decision.reset = Some(TutorialProgressReset::InitStep);
                decision.intents.extend([
                    spawn(1010, 2376, 59_500, 74_500, -9_200, None),
                    spawn(1011, 2377, 56_450, 72_400, -9_100, None),
                    spawn(FUSION_PORTAL_ID, 2695, 56_431, 72_416, -9_100, None),
                    TutorialIntent::DeleteNpc(NUMBUH_TWO_ID),
                    TutorialIntent::StartDialogue(TutorialDialogue::TalkDexterOutside),
                ]);
            }
            decision
        }
        InfectionStage::OutsidePortal => {
            if observation.npcs.fusion_portal.within_or_damaged(15.0) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::SelectTentacles),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::StopDialogue(TutorialDialogue::TalkDexterOutside),
                    TutorialIntent::StartReminderTimer,
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::SelectTentacles => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.fusion_portal.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::WarpIntoLair),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::WarpIntoLair => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }

            let mut decision = TutorialDecision::unchanged(current);
            if observation.event_is(TutorialEvent::NpcIconClose, 1) {
                decision.intents.push(TutorialIntent::HideTutorialPointer);
            }
            if observation.event_is(TutorialEvent::NpcWarp, 1) {
                decision.next_stage = TutorialStage::Infection(InfectionStage::TalkDexter);
                decision.reset = Some(TutorialProgressReset::InitStep);
                decision.intents.extend([
                    TutorialIntent::SetInstanceMap(true),
                    TutorialIntent::SetEpisode(0),
                    spawn(3001, 2378, 58_700, 98_400, -13_300, None),
                    spawn(LAIR_DEXTER_ID, 2673, 58_700, 98_400, -13_300, Some(262)),
                    TutorialIntent::StartDialogue(TutorialDialogue::TalkDexterInside),
                    TutorialIntent::SetWaypointToNpc(LAIR_DEXTER_ID),
                ]);
            }
            decision
        }
        InfectionStage::TalkDexter => {
            if observation.npcs.lair_dexter.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::SelectDexterMission),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::StopDialogue(TutorialDialogue::TalkDexterInside),
                    TutorialIntent::LockUi,
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::SelectDexterMission => {
            if observation.event_is(TutorialEvent::TaskStart, 1) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::DexterCutscene),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::UnlockUi,
                    TutorialIntent::ExitUi,
                    TutorialIntent::StartScene(TutorialScene::InfectionB),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::DexterCutscene => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            finish_dexter_cutscene(current, observation)
        }
        InfectionStage::ApproachFusion => {
            // Native actors retain their transform/damaged flag during the
            // death animation. Consume death before proximity: InitStep on
            // entry to FightFusion would otherwise erase the only death edge.
            if observation.event_value(TutorialEvent::DeadNpc) > 0 {
                return begin_nano_creation(current);
            }
            if observation.npcs.fusion_buttercup.within_or_damaged(20.0) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::FightFusion),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::FightFusion => {
            let mut decision = TutorialDecision::unchanged(current);
            if observation.timer_exceeds(1) {
                if observation.npcs.fusion_buttercup.within_or_damaged(10.0) {
                    decision
                        .intents
                        .push(TutorialIntent::AttackNpc(FUSION_BUTTERCUP_ID));
                }
                decision.intents.push(TutorialIntent::StartReminderTimer);
            }
            if observation.event_value(TutorialEvent::DeadNpc) > 0 {
                let creation = begin_nano_creation(current);
                decision.next_stage = creation.next_stage;
                decision.reset = creation.reset;
                decision.intents.extend(creation.intents);
            }
            decision
        }
        InfectionStage::NanoCreation => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Infection(InfectionStage::ExitPrompt),
                TutorialProgressReset::InitStep,
            );
            finish_scene(&mut decision, TutorialScene::InfectionC, observation);
            decision.intents.extend([
                TutorialIntent::EquipNano {
                    slot: 0,
                    nano_id: 1,
                    skill_id: 1,
                },
                spawn(LAIR_EXIT_ID, 2696, 55_800, 94_900, -13_300, Some(-132)),
                TutorialIntent::DeleteNpc(LAIR_DEXTER_ID),
                TutorialIntent::DeleteNpc(FUSION_BUTTERCUP_ID),
                TutorialIntent::SetFusionMatter(0),
                TutorialIntent::StartDialogue(TutorialDialogue::TalkDexterExit),
            ]);
            decision
        }
        InfectionStage::ExitPrompt => {
            if observation.npcs.lair_exit.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Infection(InfectionStage::WarpOut),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::StopDialogue(TutorialDialogue::TalkDexterExit),
                    TutorialIntent::ExitUi,
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        InfectionStage::WarpOut => {
            let mut decision = TutorialDecision::unchanged(current);
            if observation.event_is(TutorialEvent::NpcIconClose, 1) {
                decision.intents.push(TutorialIntent::HideTutorialPointer);
            }
            if observation.event_is(TutorialEvent::NpcWarp, 1) {
                decision.next_stage = TutorialStage::NanoPower(NanoPowerStage::AwaitCollapse);
                decision.reset = Some(TutorialProgressReset::InitChapter);
                decision.intents.extend([
                    TutorialIntent::SetInstanceMap(false),
                    TutorialIntent::StopLoopSound,
                ]);
            }
            decision
        }
    }
}

pub(super) fn evaluate_nano_power(
    stage: NanoPowerStage,
    observation: &TutorialObservation,
) -> TutorialDecision {
    let current = TutorialStage::NanoPower(stage);
    match stage {
        NanoPowerStage::AwaitCollapse => {
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::NanoPower(NanoPowerStage::CollapseCutscene),
                TutorialProgressReset::InitStep,
            );
            decision.intents.extend([
                spawn(
                    COLLAPSE_NUMBUH_TWO_ID,
                    2671,
                    90_700,
                    68_500,
                    1_100,
                    Some(180),
                ),
                spawn(5101, 2800, 90_800, 68_200, 1_100, Some(104)),
                TutorialIntent::ConfigureDemoMonster {
                    dont_kill: observation.locale == TutorialLocaleBranch::OriginalEnglish,
                },
                TutorialIntent::StartScene(TutorialScene::NanoPowerA),
            ]);
            decision
        }
        NanoPowerStage::CollapseCutscene => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::NanoPower(NanoPowerStage::CrossBridge),
                TutorialProgressReset::InitStep,
            );
            finish_scene(&mut decision, TutorialScene::NanoPowerA, observation);
            decision.intents.extend([
                TutorialIntent::DeleteNpc(2),
                TutorialIntent::StartReminderTimer,
            ]);
            decision
        }
        NanoPowerStage::CrossBridge => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.collapse_numbuh_two.within_or_damaged(28.0) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::NanoPower(NanoPowerStage::DemoMonsterCutscene),
                    TutorialProgressReset::InitStep,
                );
                decision
                    .intents
                    .push(TutorialIntent::StartScene(TutorialScene::NanoPowerA2));
                return decision;
            }
            let mut decision = TutorialDecision::unchanged(current);
            append_demo_attack_if_due(&mut decision, observation);
            decision
        }
        NanoPowerStage::DemoMonsterCutscene => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::NanoPower(NanoPowerStage::SummonNano),
                TutorialProgressReset::InitStep,
            );
            if observation.timer_exceeds(1) {
                decision.intents.extend([
                    TutorialIntent::AttackNpc(DEMO_MONSTER_ID),
                    TutorialIntent::StartReminderTimer,
                ]);
            }
            finish_scene(&mut decision, TutorialScene::NanoPowerA2, observation);
            let vertical_centiunits = match observation.scene_completion {
                SceneCompletion::Skipped => 1_200,
                SceneCompletion::Natural | SceneCompletion::Pending => 1_700,
            };
            decision.intents.push(spawn(
                DEMO_MONSTER_ID,
                2677,
                90_700,
                71_000,
                vertical_centiunits,
                Some(183),
            ));
            decision
        }
        NanoPowerStage::SummonNano => {
            let mut decision = TutorialDecision::unchanged(current);
            append_demo_attack_if_due(&mut decision, observation);
            if observation.locale == TutorialLocaleBranch::Localized
                && observation.event_value(TutorialEvent::DeadNpc) > 0
            {
                decision.next_stage = TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo);
                decision.reset = Some(TutorialProgressReset::InitStep);
                append_return_to_numbuh_two_intents(&mut decision);
            } else if observation.event_is(TutorialEvent::NanoActive, 1) {
                decision.next_stage = TutorialStage::NanoPower(NanoPowerStage::UseNanoPower);
                decision.reset = Some(TutorialProgressReset::InitStep);
                decision.intents.extend([
                    TutorialIntent::PopInputFilter,
                    TutorialIntent::StartReminderTimer,
                    TutorialIntent::StartDialogue(TutorialDialogue::NanoPowerReminder),
                ]);
            }
            decision
        }
        NanoPowerStage::UseNanoPower => {
            let mut decision = TutorialDecision::unchanged(current);
            append_demo_attack_if_due(&mut decision, observation);
            if observation.event_value(TutorialEvent::DeadNpc) > 0
                || observation.event_is(TutorialEvent::UseSkill, 1)
            {
                decision.next_stage = TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo);
                decision.reset = Some(TutorialProgressReset::InitStep);
                if observation.event_is(TutorialEvent::UseSkill, 1) {
                    decision.intents.push(TutorialIntent::StopDialogue(
                        TutorialDialogue::NanoPowerReminder,
                    ));
                }
                append_return_to_numbuh_two_intents(&mut decision);
            }
            decision
        }
        NanoPowerStage::ReturnToNumbuhTwo => {
            if observation.npcs.collapse_numbuh_two.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::NanoPower(NanoPowerStage::FinaleCutscene),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::ClearWaypoint,
                    TutorialIntent::ExitUi,
                    TutorialIntent::StartScene(TutorialScene::NanoPowerB),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        NanoPowerStage::FinaleCutscene => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::NanoPower(NanoPowerStage::Exit),
                TutorialProgressReset::InitStep,
            );
            finish_scene(&mut decision, TutorialScene::NanoPowerB, observation);
            decision.intents.extend([
                TutorialIntent::DeleteNpc(COLLAPSE_NUMBUH_TWO_ID),
                TutorialIntent::DeleteNpc(5101),
                TutorialIntent::StopLoopSound,
                TutorialIntent::ExitTutorial,
            ]);
            decision
        }
        NanoPowerStage::Exit => {
            let mut decision = TutorialDecision::unchanged(current);
            decision
                .ambiguities
                .push(TutorialReferenceAmbiguity::TerminalStageAlreadyExited(
                    current,
                ));
            decision
        }
    }
}
