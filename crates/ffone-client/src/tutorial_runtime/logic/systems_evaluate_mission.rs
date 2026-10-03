use super::*;

/// Evaluates the current stage in a [`TutorialProgress`] value.
pub fn evaluate_progress(
    progress: &TutorialProgress,
    observation: &TutorialObservation,
) -> Result<TutorialDecision, TutorialLogicError> {
    let stage = progress
        .stage()
        .ok_or(TutorialLogicError::UnknownLegacyStage {
            chapter: progress.chapter(),
            step: progress.step(),
        })?;
    evaluate_stage(stage, observation)
}

/// Evaluates exactly one unblocked reference `Chapter_03` ... `Chapter_07`
/// invocation.
pub fn evaluate_stage(
    stage: TutorialStage,
    observation: &TutorialObservation,
) -> Result<TutorialDecision, TutorialLogicError> {
    match stage {
        TutorialStage::Movement(_) => Err(TutorialLogicError::MovementChapterOutsideScope),
        TutorialStage::Combat(stage) => Ok(evaluate_combat(stage, observation)),
        TutorialStage::Minimap(stage) => Ok(evaluate_minimap(stage, observation)),
        TutorialStage::Mission(stage) => Ok(evaluate_mission(stage, observation)),
        TutorialStage::Infection(stage) => Ok(evaluate_infection(stage, observation)),
        TutorialStage::NanoPower(stage) => Ok(evaluate_nano_power(stage, observation)),
    }
}

pub(super) fn evaluate_combat(stage: CombatStage, observation: &TutorialObservation) -> TutorialDecision {
    let current = TutorialStage::Combat(stage);
    match stage {
        CombatStage::AwaitIntro => {
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Combat(CombatStage::CombatIntro),
                TutorialProgressReset::InitStep,
            );
            decision
                .intents
                .push(TutorialIntent::StartScene(TutorialScene::BasicCombatA));
            decision
        }
        CombatStage::CombatIntro => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Combat(CombatStage::TargetHostile),
                TutorialProgressReset::InitStep,
            );
            finish_scene(&mut decision, TutorialScene::BasicCombatA, observation);
            decision.intents.push(TutorialIntent::StartReminderTimer);
            decision
        }
        CombatStage::TargetHostile => {
            if observation.timer_exceeds(10) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.ui.hostile_target_selected {
                return TutorialDecision::transition(
                    current,
                    TutorialStage::Combat(CombatStage::FirstAttack),
                    TutorialProgressReset::InitStep,
                );
            }
            TutorialDecision::unchanged(current)
        }
        CombatStage::FirstAttack => {
            if !observation.event_is(TutorialEvent::DamageNpc, 0) {
                return TutorialDecision::transition(
                    current,
                    TutorialStage::Combat(CombatStage::KillThree),
                    TutorialProgressReset::InitStep,
                );
            }
            TutorialDecision::unchanged(current)
        }
        CombatStage::KillThree => {
            if observation.event_is(TutorialEvent::DeadNpc, 3) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Combat(CombatStage::CyberusIntro),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::DeleteNpc(1001),
                    TutorialIntent::DeleteNpc(1002),
                    TutorialIntent::DeleteNpc(1003),
                    TutorialIntent::StartScene(TutorialScene::BasicCombatB),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        CombatStage::CyberusIntro => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Combat(CombatStage::FightCyberus),
                TutorialProgressReset::InitStep,
            );
            finish_scene(&mut decision, TutorialScene::BasicCombatB, observation);
            // The final BasicCombatEvent_B tick has already released cinematic control.
            // Queue Cyber's opener in this handoff; FightCyberus keeps the paced retries.
            decision.intents.extend([
                TutorialIntent::AttackNpc(1005),
                TutorialIntent::StartReminderTimer,
            ]);
            decision
        }
        CombatStage::FightCyberus => {
            let mut decision = TutorialDecision::unchanged(current);
            if observation.timer_exceeds(1) {
                decision.intents.extend([
                    TutorialIntent::AttackNpc(1005),
                    TutorialIntent::StartReminderTimer,
                ]);
            }
            if observation.event_is(TutorialEvent::DamageUser, 1) {
                decision.next_stage = TutorialStage::Combat(CombatStage::PlayerWasHit);
                decision.reset = Some(TutorialProgressReset::DirectStepWrite);
            }
            decision
        }
        CombatStage::PlayerWasHit => {
            let mut decision = TutorialDecision::unchanged(current);
            if observation.timer_exceeds(1) {
                decision.intents.extend([
                    TutorialIntent::AttackNpc(1005),
                    TutorialIntent::StartReminderTimer,
                ]);
            }
            if observation.event_is(TutorialEvent::DeadNpc, 1) {
                decision.next_stage = TutorialStage::Combat(CombatStage::PlanetFusionOutro);
                decision.reset = Some(TutorialProgressReset::InitStep);
                decision
                    .intents
                    .push(TutorialIntent::StartScene(TutorialScene::BasicCombatC));
            }
            decision
        }
        CombatStage::PlanetFusionOutro => {
            if !observation.scene_completion.is_complete() {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Minimap(MinimapStage::AwaitSequence),
                TutorialProgressReset::InitChapter,
            );
            finish_scene(&mut decision, TutorialScene::BasicCombatC, observation);
            decision.intents.extend([
                TutorialIntent::DeleteNpc(1005),
                TutorialIntent::DeleteNpc(100),
                TutorialIntent::DeleteNpc(101),
                TutorialIntent::WarpPlayer(ClientPosition::centiunits(56_600, -10_100, 66_500)),
            ]);
            decision
        }
    }
}

pub(super) fn evaluate_minimap(stage: MinimapStage, observation: &TutorialObservation) -> TutorialDecision {
    let current = TutorialStage::Minimap(stage);
    match stage {
        MinimapStage::AwaitSequence => {
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Minimap(MinimapStage::MinimapSequence),
                TutorialProgressReset::InitStep,
            );
            decision.intents.extend([
                spawn(NUMBUH_TWO_ID, 2671, 65_100, 73_900, -8_200, Some(68)),
                TutorialIntent::StartDialogue(TutorialDialogue::Minimap),
            ]);
            decision
        }
        MinimapStage::MinimapSequence => {
            if observation.npcs.numbuh_two.within_or_damaged(50.0) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Minimap(MinimapStage::ApproachNumbuhTwo),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::StopDialogue(TutorialDialogue::Minimap),
                    TutorialIntent::SetWaypointToNpc(NUMBUH_TWO_ID),
                    TutorialIntent::StartReminderTimer,
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MinimapStage::ApproachNumbuhTwo => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.numbuh_two.within_or_damaged(25.0) {
                return TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::Approach),
                    TutorialProgressReset::InitChapter,
                );
            }
            TutorialDecision::unchanged(current)
        }
    }
}

pub(super) fn evaluate_mission(stage: MissionStage, observation: &TutorialObservation) -> TutorialDecision {
    let current = TutorialStage::Mission(stage);
    match stage {
        MissionStage::Approach => {
            if observation.npcs.numbuh_two.within_or_damaged(5.0) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::TargetAndTalk),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::PushInputFilter,
                    TutorialIntent::StartReminderTimer,
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::TargetAndTalk => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.extend([
                    TutorialIntent::PushInputFilter,
                    TutorialIntent::StartReminderTimer,
                ]);
                return decision;
            }
            if observation.npcs.numbuh_two.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::MissionMenu),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::LockUi,
                    TutorialIntent::ClearWaypoint,
                    TutorialIntent::StartDialogue(TutorialDialogue::TalkNumbuhTwoMission),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::MissionMenu => {
            if observation.ui.journal_mode == TutorialJournalMode::Allow {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::NumbuhTwoDelay),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::StopDialogue(TutorialDialogue::TalkNumbuhTwoMission),
                    TutorialIntent::ClearWaypoint,
                    TutorialIntent::LockUi,
                    TutorialIntent::StartDelayMillis(9_000),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::NumbuhTwoDelay => {
            if !observation.delay_completed {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Mission(MissionStage::AcceptMission),
                TutorialProgressReset::InitStep,
            );
            decision.intents.push(TutorialIntent::StartReminderTimer);
            decision
        }
        MissionStage::AcceptMission => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.event_is(TutorialEvent::TaskStart, 1) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::CloseAccept),
                    TutorialProgressReset::InitStep,
                );
                decision
                    .intents
                    .extend([TutorialIntent::UnlockUi, TutorialIntent::StartReminderTimer]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::CloseAccept => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision
                    .intents
                    .extend([TutorialIntent::UnlockUi, TutorialIntent::StartReminderTimer]);
                return decision;
            }
            if !observation.npcs.numbuh_two.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::ObjectiveCombat),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend(objective_combat_entry_intents());
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::ObjectiveCombat => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.extend(objective_combat_entry_intents());
                return decision;
            }

            let mut decision = TutorialDecision::unchanged(current);
            if observation.timer_exceeds(1) {
                if observation
                    .npcs
                    .mission_target
                    .within_or_damaged(MISSION_TARGET_ENGAGE_RADIUS)
                {
                    decision.intents.push(TutorialIntent::AttackNpc(1100));
                }
                decision.intents.push(TutorialIntent::StartReminderTimer);
            }
            if observation.event_is(TutorialEvent::TaskStart, 1) {
                decision.next_stage = TutorialStage::Mission(MissionStage::NumbuhTwoResponse);
                decision.reset = Some(TutorialProgressReset::InitStep);
                decision.intents.extend([
                    TutorialIntent::StartDelayMillis(3_500),
                    TutorialIntent::StopDialogue(TutorialDialogue::TalkNumbuhTwoMission),
                ]);
            }
            decision
        }
        MissionStage::NumbuhTwoResponse => {
            if !observation.delay_completed {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Mission(MissionStage::OpenMenu),
                TutorialProgressReset::InitStep,
            );
            decision.intents.push(TutorialIntent::PushInputFilter);
            decision
        }
        MissionStage::OpenMenu => {
            if observation.ui.nanocom_main_menu_visible {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::SelectJournal),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::DeleteNpc(1100));
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::SelectJournal => {
            if observation.timer_exceeds(25) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::OpenMenu),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.ui.journal_mode == TutorialJournalMode::Other {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::JournalIntro),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::LockUi,
                    TutorialIntent::StartDelayMillis(4_000),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::JournalIntro => journal_delay_transition(
            current,
            TutorialStage::Mission(MissionStage::JournalMissionTab),
            2_500,
            observation,
        ),
        MissionStage::JournalMissionTab => journal_delay_transition(
            current,
            TutorialStage::Mission(MissionStage::JournalObjective),
            2_500,
            observation,
        ),
        MissionStage::JournalObjective => journal_delay_transition(
            current,
            TutorialStage::Mission(MissionStage::JournalDetail),
            4_200,
            observation,
        ),
        MissionStage::JournalDetail => {
            if !observation.delay_completed
                || observation.ui.journal_mode != TutorialJournalMode::Other
            {
                return TutorialDecision::unchanged(current);
            }
            let mut decision = TutorialDecision::transition(
                current,
                TutorialStage::Mission(MissionStage::CloseJournal),
                TutorialProgressReset::InitStep,
            );
            decision.intents.extend([
                TutorialIntent::PopInputFilter,
                TutorialIntent::UnlockUi,
                TutorialIntent::StartReminderTimer,
            ]);
            decision
        }
        MissionStage::CloseJournal => {
            if observation.timer_exceeds(12) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.extend([
                    TutorialIntent::PopInputFilter,
                    TutorialIntent::UnlockUi,
                    TutorialIntent::StartReminderTimer,
                ]);
                return decision;
            }
            if observation.ui.journal_mode != TutorialJournalMode::Other {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::ReturnToNumbuhTwo),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::ReturnToNumbuhTwo => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.numbuh_two.within_or_damaged(15.0) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::NearNumbuhTwo),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::NearNumbuhTwo => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision.intents.push(TutorialIntent::StartReminderTimer);
                return decision;
            }
            if observation.npcs.numbuh_two.interacting {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::RewardMenu),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::LockUi,
                    TutorialIntent::StartDialogue(TutorialDialogue::TalkNumbuhTwoReward),
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::RewardMenu => {
            if observation.ui.journal_mode == TutorialJournalMode::Reward {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::ClaimReward),
                    TutorialProgressReset::InitStep,
                );
                decision.intents.extend([
                    TutorialIntent::StopDialogue(TutorialDialogue::TalkNumbuhTwoReward),
                    TutorialIntent::LockUi,
                ]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::ClaimReward => {
            if observation.event_is(TutorialEvent::QuestEnd, 1) {
                let mut decision = TutorialDecision::transition(
                    current,
                    TutorialStage::Mission(MissionStage::CloseReward),
                    TutorialProgressReset::InitStep,
                );
                decision
                    .intents
                    .extend([TutorialIntent::UnlockUi, TutorialIntent::StartReminderTimer]);
                return decision;
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::CloseReward => {
            if observation.timer_exceeds(25) {
                let mut decision =
                    TutorialDecision::transition(current, current, TutorialProgressReset::InitStep);
                decision
                    .intents
                    .extend([TutorialIntent::UnlockUi, TutorialIntent::StartReminderTimer]);
                return decision;
            }
            if !observation.npcs.numbuh_two.interacting {
                return mission_handoff(current);
            }
            TutorialDecision::unchanged(current)
        }
        MissionStage::Handoff => {
            let mut decision = TutorialDecision::unchanged(current);
            decision
                .ambiguities
                .push(TutorialReferenceAmbiguity::TransientStageHasNoStandaloneBranch(current));
            decision
        }
    }
}
