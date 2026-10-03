use super::*;

pub(super) fn finish_scene(
    decision: &mut TutorialDecision,
    scene: TutorialScene,
    observation: &TutorialObservation,
) {
    decision.intents.push(TutorialIntent::FinishScene {
        scene,
        completion: observation.scene_completion,
    });
}

pub(super) fn spawn(id: i32, npc_type: i32, x: i32, y: i32, z: i32, angle: Option<i16>) -> TutorialIntent {
    TutorialIntent::SpawnNpc(TutorialNpcSpawn::new(
        id,
        npc_type,
        LegacySpawnPosition::centiunits(x, y, z),
        angle,
    ))
}

pub(super) fn objective_combat_entry_intents() -> [TutorialIntent; 6] {
    [
        TutorialIntent::PopInputFilter,
        TutorialIntent::UnlockUi,
        spawn(2000, 2374, 69_700, 79_400, -8_200, None),
        spawn(1100, 2676, 69_700, 79_400, -8_200, Some(90)),
        TutorialIntent::StartReminderTimer,
        TutorialIntent::HideTutorialPointer,
    ]
}

pub(super) fn journal_delay_transition(
    current: TutorialStage,
    next: TutorialStage,
    next_delay_millis: u32,
    observation: &TutorialObservation,
) -> TutorialDecision {
    if !observation.delay_completed || observation.ui.journal_mode != TutorialJournalMode::Other {
        return TutorialDecision::unchanged(current);
    }
    let mut decision = TutorialDecision::transition(current, next, TutorialProgressReset::InitStep);
    decision
        .intents
        .push(TutorialIntent::StartDelayMillis(next_delay_millis));
    decision
}

pub(super) fn mission_handoff(current: TutorialStage) -> TutorialDecision {
    let mut decision = TutorialDecision::transition(
        current,
        TutorialStage::Infection(InfectionStage::ApproachButtercup),
        TutorialProgressReset::InitChapter,
    );
    decision.intents.extend([
        spawn(BUTTERCUP_ID, 2672, 62_400, 77_200, -8_200, Some(275)),
        spawn(1008, 2375, 59_600, 78_400, -8_200, None),
        TutorialIntent::SetFusionMatter(220),
        TutorialIntent::SetFatigueLevel(1),
        TutorialIntent::StartTask(2250),
        // Legacy Chapter_05 starts task 2250 before InitStep/InitChapter.
        // Native applies the chapter reset before intents, so clear the
        // resulting transient TaskStart flag here to preserve source order.
        TutorialIntent::ClearEventFlags,
        TutorialIntent::StartDialogue(TutorialDialogue::TalkButtercup),
    ]);
    decision
}

pub(super) fn enter_tech_square_warp_prompt(current: TutorialStage) -> TutorialDecision {
    let mut decision = TutorialDecision::transition(
        current,
        TutorialStage::Infection(InfectionStage::WarpFromTechSquare),
        TutorialProgressReset::InitStep,
    );
    decision.intents.extend([
        TutorialIntent::StartReminderTimer,
        TutorialIntent::HideTutorialPointer,
    ]);
    decision
}

pub(super) fn finish_dexter_cutscene(
    current: TutorialStage,
    observation: &TutorialObservation,
) -> TutorialDecision {
    let next = if observation.event_value(TutorialEvent::DeadNpc) > 0 {
        TutorialStage::Infection(InfectionStage::NanoCreation)
    } else if observation.npcs.fusion_buttercup.within_or_damaged(20.0) {
        TutorialStage::Infection(InfectionStage::FightFusion)
    } else {
        TutorialStage::Infection(InfectionStage::ApproachFusion)
    };

    let mut decision = TutorialDecision::transition(current, next, TutorialProgressReset::InitStep);
    finish_scene(&mut decision, TutorialScene::InfectionB, observation);
    decision.intents.extend([
        TutorialIntent::DeleteNpc(LAIR_DEXTER_ID),
        spawn(
            FUSION_BUTTERCUP_ID,
            2678,
            56_300,
            96_700,
            -13_300,
            Some(266),
        ),
        spawn(LAIR_DEXTER_ID, 2902, 56_200, 96_700, -13_300, None),
        TutorialIntent::FaceNpc {
            id: LAIR_DEXTER_ID,
            target: FUSION_BUTTERCUP_ID,
        },
        // Primary `Infection_Event_B` leaves combat Dexter in the looping
        // `melee1event` pose after replacing the dialogue NPC. The scene
        // finisher respawns that actor to make normal and skipped completion
        // converge, so it must also restore the legacy combat pose.
        TutorialIntent::PlayNpcAnimation {
            id: LAIR_DEXTER_ID,
            clip: "melee1event",
            once: false,
        },
        TutorialIntent::FaceNpc {
            id: FUSION_BUTTERCUP_ID,
            target: LAIR_DEXTER_ID,
        },
        TutorialIntent::PlayNpcAnimation {
            id: FUSION_BUTTERCUP_ID,
            clip: "melee1",
            once: false,
        },
        TutorialIntent::WarpPlayer(ClientPosition::centiunits(58_840, -13_350, 98_380)),
    ]);

    if matches!(next, TutorialStage::Infection(InfectionStage::FightFusion)) {
        decision.intents.push(TutorialIntent::StartReminderTimer);
    } else if matches!(next, TutorialStage::Infection(InfectionStage::NanoCreation)) {
        decision.intents.extend([
            TutorialIntent::EndTask(2254),
            TutorialIntent::StartScene(TutorialScene::InfectionC),
        ]);
    }
    decision
}

pub(super) fn begin_nano_creation(current: TutorialStage) -> TutorialDecision {
    let mut decision = TutorialDecision::transition(
        current,
        TutorialStage::Infection(InfectionStage::NanoCreation),
        TutorialProgressReset::InitStep,
    );
    decision.intents.extend([
        TutorialIntent::EndTask(2254),
        TutorialIntent::StartScene(TutorialScene::InfectionC),
    ]);
    decision
}

pub(super) fn append_demo_attack_if_due(decision: &mut TutorialDecision, observation: &TutorialObservation) {
    if observation.timer_exceeds(1) {
        decision.intents.extend([
            TutorialIntent::AttackNpc(DEMO_MONSTER_ID),
            TutorialIntent::StartReminderTimer,
        ]);
    }
}

pub(super) fn append_return_to_numbuh_two_intents(decision: &mut TutorialDecision) {
    decision.intents.extend([
        TutorialIntent::SetWaypointToNpc(COLLAPSE_NUMBUH_TWO_ID),
        TutorialIntent::StopDialogue(TutorialDialogue::NanoPowerReminder),
    ]);
}
