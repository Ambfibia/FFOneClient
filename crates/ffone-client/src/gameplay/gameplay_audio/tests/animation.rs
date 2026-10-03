use super::*;

#[test]
fn swimming_sounds_cover_both_rigs_and_primary_animation_cycles() {
    for (gender, forward, turn, forward_duration) in [
        (
            PlayerRigGender::Male,
            "M_Avatar_SwimForward",
            "M_Avatar_SwimTurn",
            0.816_666_7,
        ),
        (
            PlayerRigGender::Female,
            "F_Avatar_SwimForward",
            "F_Avatar_SwimTurn",
            0.950_000_05,
        ),
    ] {
        for (state, expected, duration) in [
            (LegacyLocomotionState::Swim, forward, forward_duration),
            (
                LegacyLocomotionState::SwimBack,
                "Avatar_SwimBack",
                0.816_666_7,
            ),
            (
                LegacyLocomotionState::SwimIdle,
                "Avatar_SwimIdle",
                0.816_666_7,
            ),
            (LegacyLocomotionState::SwimLeft, turn, 0.75),
            (LegacyLocomotionState::SwimRight, turn, 0.75),
        ] {
            assert!(matches!(player_locomotion_cue(gender, state),
                Some(SoundCue::Exact(name)) if name == expected));
            assert_eq!(
                player_locomotion_loop_duration(gender, state),
                Some(duration)
            );
        }
    }
}

#[test]
fn traversal_animation_sounds_follow_primary_event_time_and_end_cycles() {
    assert_eq!(
        player_locomotion_loop_duration(PlayerRigGender::Male, LegacyLocomotionState::Slide),
        Some(1.483_333_3)
    );
    assert_eq!(
        player_locomotion_loop_duration(PlayerRigGender::Male, LegacyLocomotionState::RopeDown),
        Some(0.483_333_35)
    );
    assert!(
        player_locomotion_loop_duration(PlayerRigGender::Male, LegacyLocomotionState::RopeDrop)
            .is_none()
    );

    let mut cursor = PlayerLocomotionAudioCursor {
        locomotion: LegacyLocomotionState::RopeDown,
        elapsed_seconds: 0.0,
        next_loop_event_seconds: Some(0.25),
    };
    cursor.elapsed_seconds = 0.249;
    assert!(cursor.elapsed_seconds < cursor.next_loop_event_seconds.unwrap());
    cursor.elapsed_seconds = 0.25;
    assert!(cursor.elapsed_seconds >= cursor.next_loop_event_seconds.unwrap());
}

#[test]
fn unarmed_animation_events_keep_primary_gender_cues_and_layer_timing() {
    let player = Entity::from_bits(1);
    let mut runtime = GameplayAudioRuntime::default();
    runtime.queue_player_unarmed_attack(player, PlayerRigGender::Male, true);
    runtime.queue_player_unarmed_attack(player, PlayerRigGender::Female, false);

    let observed = runtime
        .scheduled
        .iter()
        .map(|sound| {
            let SoundCue::Exact(name) = sound.cue else {
                panic!("unarmed animation cues are exact")
            };
            (sound.remaining_seconds, name)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        vec![
            (0.25, "M_Avatar_Attack1"),
            (0.25, "M_Avatar_Attack1upper"),
            (0.25, "PlyrAvtF_Attack1upper"),
        ]
    );
}
