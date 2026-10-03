use super::*;

#[test]
fn every_reference_stage_round_trips_its_raw_legacy_pair() {
    let pairs = [
        (0, 0),
        (0, 11),
        (1, 8),
        (2, 5),
        (3, 100),
        (3, 102),
        (4, 109),
        (4, 200),
        (4, 201),
        (4, 210),
        (5, 9),
    ];
    for pair in pairs {
        assert_eq!(
            TutorialStage::from_legacy(pair.0, pair.1)
                .expect("reference pair must be classified")
                .legacy(),
            pair
        );
    }
    assert_eq!(TutorialStage::from_legacy(6, 0), None);
    assert_eq!(TutorialStage::from_legacy(0, 12), None);
}

#[test]
fn init_step_and_chapter_preserve_cntutorialscript_reset_rules() {
    let mut progress = TutorialProgress::default();
    progress.receive_event(TutorialEvent::DeadNpc, 0);
    progress.receive_event(TutorialEvent::DeadNpc, 0);
    progress.set_timer_seconds(26);
    progress.init_step(4);
    assert_eq!(
        progress.stage(),
        Some(TutorialStage::Movement(MovementStage::LookLeft))
    );
    assert_eq!(progress.event_value(TutorialEvent::DeadNpc), 0);
    assert_eq!(progress.timer_seconds(), 0);

    progress.receive_event(TutorialEvent::TaskStart, 0);
    progress.init_chapter(3).unwrap();
    assert_eq!(
        progress.stage(),
        Some(TutorialStage::Mission(MissionStage::Approach))
    );
    assert_eq!(progress.event_value(TutorialEvent::TaskStart), 0);
    assert_eq!(progress.init_chapter(6), Err(6));
}

#[test]
fn receive_event_matches_movement_death_and_boolean_storage() {
    let mut progress = TutorialProgress::default();
    progress.receive_event(TutorialEvent::Move, 7);
    progress.receive_event(TutorialEvent::DeadNpc, 99);
    progress.receive_event(TutorialEvent::DeadNpc, 99);
    progress.receive_event(TutorialEvent::NpcWarp, 0);
    assert_eq!(progress.event_value(TutorialEvent::Move), 7);
    assert_eq!(progress.event_value(TutorialEvent::DeadNpc), 2);
    assert_eq!(progress.event_value(TutorialEvent::NpcWarp), 1);
}

#[test]
fn clear_event_flags_does_not_rewind_stage_or_timer() {
    let mut progress = TutorialProgress::default();
    progress.init_chapter(4).unwrap();
    progress.write_step_preserving_state(2);
    progress.set_timer_seconds(17);
    progress.receive_event(TutorialEvent::TaskStart, 1);

    progress.clear_event_flags();

    assert_eq!(progress.chapter(), 4);
    assert_eq!(progress.step(), 2);
    assert_eq!(progress.timer_seconds(), 17);
    assert_eq!(progress.event_value(TutorialEvent::TaskStart), 0);
}

#[test]
fn direct_step_write_preserves_events_and_timer() {
    let mut progress = TutorialProgress::default();
    progress.init_chapter(1).unwrap();
    progress.init_step(6);
    progress.receive_event(TutorialEvent::DamageUser, 1);
    progress.set_timer_seconds(2);

    progress.write_step_preserving_state(7);

    assert_eq!(
        progress.stage(),
        Some(TutorialStage::Combat(CombatStage::PlayerWasHit))
    );
    assert_eq!(progress.event_value(TutorialEvent::DamageUser), 1);
    assert_eq!(progress.timer_seconds(), 2);
}

#[test]
fn movement_metadata_replays_the_reference_input_gate_sequence() {
    let stages = [
        MovementStage::AwaitIntro,
        MovementStage::IntroCutscene,
        MovementStage::PostIntro,
        MovementStage::LookRight,
        MovementStage::LookLeft,
        MovementStage::LookUp,
        MovementStage::LookDown,
        MovementStage::MoveForward,
        MovementStage::MoveBackward,
        MovementStage::MoveAndSteer,
        MovementStage::ReachLedge,
        MovementStage::JumpAndLand,
    ];
    let expected = [
        TutorialInputPermissions::NONE,
        TutorialInputPermissions::NONE,
        TutorialInputPermissions::NONE,
        LOOK_INPUT,
        LOOK_INPUT,
        LOOK_INPUT,
        LOOK_INPUT,
        FORWARD_INPUT,
        BACKWARD_INPUT,
        STEER_INPUT,
        WALK_INPUT,
        MOVE_INPUT,
    ];

    for (stage, input) in stages.into_iter().zip(expected) {
        let metadata = TutorialStage::Movement(stage).metadata();
        assert_eq!(metadata.input, input, "wrong input gate for {stage:?}");
        assert!(!metadata.input.allows(TutorialInputLock::Attack));
        assert!(!metadata.input.allows(TutorialInputLock::Menu));
    }

    let intro = TutorialStage::Movement(MovementStage::IntroCutscene).metadata();
    assert_eq!(intro.scene, TutorialScene::BasicMove);
    let jump = TutorialStage::Movement(MovementStage::JumpAndLand).metadata();
    assert_eq!(
        jump.instruction,
        Some("Jump onto the ledge by pressing the Space Bar.")
    );
    assert_eq!(jump.entry_voice_cue, Some("Computress_Tut12"));
    assert!(jump.input.allows(TutorialInputLock::Front));
    assert!(jump.input.allows(TutorialInputLock::Jump));
    assert!(jump.input.allows(TutorialInputLock::Mouse));
}

#[test]
fn later_stage_metadata_preserves_ui_scene_and_nano_gates() {
    let combat_intro = TutorialStage::Combat(CombatStage::CyberusIntro).metadata();
    assert_eq!(combat_intro.scene, TutorialScene::BasicCombatB);
    assert_eq!(combat_intro.input, TutorialInputPermissions::NONE);

    let mission_menu = TutorialStage::Mission(MissionStage::OpenMenu).metadata();
    assert!(mission_menu.input.allows(TutorialInputLock::Menu));
    assert!(mission_menu.input.allows(TutorialInputLock::Journal));
    assert!(!mission_menu.input.allows(TutorialInputLock::Front));
    assert!(!mission_menu.input.allows(TutorialInputLock::Attack));

    let close_reward = TutorialStage::Mission(MissionStage::CloseReward).metadata();
    assert_eq!(
        close_reward.instruction,
        Some("Click the \"Close\" button.")
    );
    assert!(close_reward.input.allows(TutorialInputLock::Front));
    assert!(close_reward.input.allows(TutorialInputLock::Attack));
    assert!(close_reward.input.allows(TutorialInputLock::Journal));

    let dexter_scene = TutorialStage::Infection(InfectionStage::DexterCutscene).metadata();
    assert_eq!(dexter_scene.scene, TutorialScene::InfectionB);
    assert_eq!(dexter_scene.input, TutorialInputPermissions::NONE);

    let summon = TutorialStage::NanoPower(NanoPowerStage::SummonNano).metadata();
    assert!(summon.input.allows(TutorialInputLock::NanoActive));
    assert!(!summon.input.allows(TutorialInputLock::Attack));
    assert!(!summon.input.allows(TutorialInputLock::NanoPower));

    let use_power = TutorialStage::NanoPower(NanoPowerStage::UseNanoPower).metadata();
    assert_eq!(use_power.entry_voice_cue, Some("Computress_Tut60"));
    assert!(use_power.input.allows(TutorialInputLock::Front));
    assert!(use_power.input.allows(TutorialInputLock::Attack));
    assert!(use_power.input.allows(TutorialInputLock::NanoPower));
    assert!(use_power.input.allows(TutorialInputLock::NanoActive));
}
