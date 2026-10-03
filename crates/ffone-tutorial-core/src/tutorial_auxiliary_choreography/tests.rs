#[test]
fn localized_voice_waits_at_each_boundary_and_resumes_without_losing_lines() {
    use super::*;
    for sequence in TutorialAuxiliarySequence::ALL {
        let mut player = TutorialAuxiliaryPlayer::default();
        player.start(sequence);
        let Some(first) = player.next_voice_seconds() else {
            continue;
        };
        assert!(player.tick_localized(1000.0, true).unwrap().is_empty());
        assert_eq!(player.elapsed_seconds(), 0.0);
        let first_batch = player.tick_localized(1000.0, false).unwrap();
        assert_eq!(player.elapsed_seconds(), first);
        assert_eq!(
            first_batch
                .iter()
                .filter(|event| matches!(event.action, TutorialAuxiliaryAction::Voice { .. }))
                .count(),
            1
        );
        if let Some(next) = player.next_voice_seconds() {
            assert!(next > first);
            assert!(player.tick_localized(1000.0, true).unwrap().is_empty());
            assert_eq!(player.elapsed_seconds(), first);
            let next_batch = player.tick_localized(1000.0, false).unwrap();
            assert_eq!(player.elapsed_seconds(), next);
            assert_eq!(
                next_batch
                    .iter()
                    .filter(|event| matches!(
                        event.action,
                        TutorialAuxiliaryAction::Voice { .. }
                    ))
                    .count(),
                1
            );
        }
        player.stop();
        assert!(player.tick_localized(1000.0, true).unwrap().is_empty());
    }
}
use super::*;
use std::collections::HashSet;

#[test]
fn all_nine_auxiliary_coroutines_are_unique_and_mapped() {
    assert_eq!(TutorialAuxiliarySequence::ALL.len(), 9);
    assert_eq!(TUTORIAL_AUXILIARY_DEFINITIONS.len(), 9);
    let definitions = TUTORIAL_AUXILIARY_DEFINITIONS
        .iter()
        .map(|definition| definition.sequence)
        .collect::<HashSet<_>>();
    assert_eq!(definitions.len(), 9);
    for sequence in TutorialAuxiliarySequence::ALL {
        assert_eq!(tutorial_auxiliary_definition(sequence).sequence, sequence);
    }
}

#[test]
fn exact_screen_expressions_preserve_legacy_integer_math() {
    assert_eq!(MOVE_HINT_POSITION.evaluate(1280, 720), [80, 390]);
    assert_eq!(TALK_CURSOR_POSITION.evaluate(1280, 720), [540, 180]);
    assert_eq!(MISSION_CURSOR_POSITION.evaluate(1280, 720), [714, 319]);
}

#[test]
fn minimap_large_tick_emits_every_action_and_reminder_cycle() {
    let mut player = TutorialAuxiliaryPlayer::default();
    player.start(TutorialAuxiliarySequence::MinimapEvent);
    let emitted = player.tick(66.0).unwrap();
    assert!(emitted.iter().any(|emission| {
        matches!(
            emission.action,
            TutorialAuxiliaryAction::SpawnEffectRelativeToActor {
                runtime_id: 1005,
                offset: [0.0, -1.0, 0.0],
                effect_id: 668,
            }
        )
    }));
    let repeat_cycles = emitted
        .iter()
        .filter_map(|emission| match emission.origin {
            TutorialAuxiliaryEmissionOrigin::Repeat { cycle, .. } => Some(cycle),
            TutorialAuxiliaryEmissionOrigin::OneShot { .. } => None,
        })
        .collect::<HashSet<_>>();
    assert_eq!(repeat_cycles, HashSet::from([0, 1, 2]));
    assert_eq!(
        emitted
            .iter()
            .filter(|emission| matches!(
                emission.action,
                TutorialAuxiliaryAction::Voice {
                    clip: "Computress_Tut22",
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        player.active(),
        Some(TutorialAuxiliarySequence::MinimapEvent)
    );
}

#[test]
fn talking_to_numbuh_two_immediately_clears_the_standalone_world_marker() {
    let mut player = TutorialAuxiliaryPlayer::default();
    player.start(TutorialAuxiliarySequence::TalkNumTwo1);

    let opening = player.tick(0.0).unwrap();

    assert!(opening.iter().any(|emission| {
        emission.scheduled_at_seconds == 0.0
            && emission.action == TutorialAuxiliaryAction::ClearEffects
    }));
    assert!(
        !tutorial_auxiliary_definition(TutorialAuxiliarySequence::TalkNumTwo1)
            .actions
            .iter()
            .any(|timed| {
                timed.at_seconds > 0.0 && timed.action == TutorialAuxiliaryAction::ClearEffects
            })
    );
}

#[test]
fn finite_sequence_emits_time_zero_and_completion_actions_once() {
    let mut player = TutorialAuxiliaryPlayer::default();
    player.start(TutorialAuxiliarySequence::TalkDexter3);
    let opening = player.tick(0.0).unwrap();
    assert_eq!(opening.len(), 2);
    let ending = player.tick(3.0).unwrap();
    assert_eq!(ending.len(), 3);
    assert_eq!(player.active(), None);
    assert!(player.tick(30.0).unwrap().is_empty());
}

#[test]
fn buttercup_sequence_keeps_fusion_matter_full_until_second_twenty_seven() {
    let mut player = TutorialAuxiliaryPlayer::default();
    player.start(TutorialAuxiliarySequence::TalkButtercup1);

    let before_completion = player.tick(26.99).unwrap();
    assert!(!before_completion.iter().any(|emission| matches!(
        emission.action,
        TutorialAuxiliaryAction::SetFusionMatter { value: 0 }
    )));

    let completion = player.tick(0.02).unwrap();
    assert_eq!(
        completion
            .iter()
            .filter(|emission| matches!(
                emission.action,
                TutorialAuxiliaryAction::SetFusionMatter { value: 0 }
            ))
            .count(),
        1
    );
    assert_eq!(player.active(), None);
}

#[test]
fn contracts_cover_all_twenty_one_legacy_coroutines() {
    // Ten large definitions live in tutorial_choreography, nine exact
    // auxiliary definitions live here, plus these two generic/init
    // coroutine contracts.
    assert_eq!(10 + TUTORIAL_AUXILIARY_DEFINITIONS.len() + 2, 21);
    assert_eq!(TUTORIAL_AUDIO_CROSSFADE.steps_per_fade, 10);
    assert_eq!(TUTORIAL_AUDIO_CROSSFADE.seconds_per_step, 0.05);
    assert_eq!(
        TUTORIAL_INITIALIZATION.dome_route,
        "Mob/etc_domeglass_04.kfm"
    );
}
