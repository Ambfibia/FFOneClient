use crate::tutorial_presenter::*;

#[test]
fn every_finite_reference_scene_has_an_ordered_bounded_timeline() {
    assert_eq!(TUTORIAL_SCENE_PRESENTATIONS.len(), 10);
    for presentation in TUTORIAL_SCENE_PRESENTATIONS {
        let mut previous = 0.0;
        for cue in presentation.audio {
            assert!(
                cue.seconds >= previous,
                "{:?} is unordered",
                presentation.scene
            );
            assert!(
                cue.seconds <= presentation.duration_seconds,
                "{:?} cue {:?} exceeds scene duration",
                presentation.scene,
                cue
            );
            assert!(
                cue.kind == TutorialAudioCueKind::StopLoops || !cue.cue.is_empty(),
                "{:?} has an empty playable cue",
                presentation.scene
            );
            previous = cue.seconds;
        }
    }
}

#[test]
fn lookup_excludes_non_coroutine_and_none_scenes() {
    assert!(tutorial_scene_presentation(TutorialScene::None).is_none());
    assert!(tutorial_scene_presentation(TutorialScene::InfectionD).is_none());
    assert_eq!(
        tutorial_scene_presentation(TutorialScene::BasicMove)
            .unwrap()
            .duration_seconds,
        31.15
    );
}

#[test]
fn tutorial_location_typewriter_keeps_its_original_keyboard_sound() {
    let basic_move = tutorial_scene_presentation(TutorialScene::BasicMove).unwrap();
    assert!(basic_move.audio.iter().any(|cue| {
        cue.seconds == 2.0
            && cue.cue == "TechSquareText_Typed"
            && cue.kind == TutorialAudioCueKind::OneShot
    }));
}

#[test]
fn buttercup_flyby_uses_the_exact_recovered_voice_name() {
    let basic_move = tutorial_scene_presentation(TutorialScene::BasicMove).unwrap();
    assert!(basic_move.audio.iter().any(|cue| {
        cue.seconds == 8.15
            && cue.cue == "Btrcup_Tut01"
            && cue.kind == TutorialAudioCueKind::Voice
    }));
}

#[test]
fn looping_audio_is_explicitly_stopped_before_scene_cleanup() {
    for scene in [TutorialScene::NanoPowerA, TutorialScene::NanoPowerB] {
        let presentation = tutorial_scene_presentation(scene).unwrap();
        assert!(
            presentation
                .audio
                .iter()
                .any(|cue| cue.kind == TutorialAudioCueKind::StartLoop)
        );
        assert!(
            presentation
                .audio
                .iter()
                .any(|cue| cue.kind == TutorialAudioCueKind::StopLoops)
        );
    }
    let infection = tutorial_scene_presentation(TutorialScene::InfectionC).unwrap();
    assert!(
        infection
            .audio
            .iter()
            .any(|cue| cue.kind == TutorialAudioCueKind::StartLoop)
    );
    assert!(
        !infection
            .audio
            .iter()
            .any(|cue| cue.kind == TutorialAudioCueKind::StopLoops),
        "the collapse loop survives InfectionC and is stopped by the later warp transition"
    );
}

#[test]
fn duplicated_or_nonshared_tutorial_audio_uses_container_proven_routes() {
    let routes = TUTORIAL_SCENE_PRESENTATIONS
        .iter()
        .flat_map(|scene| scene.audio)
        .filter_map(|cue| cue.semantic_path.map(|path| (cue.cue, path)))
        .collect::<Vec<_>>();
    assert_eq!(
        routes,
        vec![
            ("Ben_Tut01", "audio/voice/en/ben/tut01.ogg"),
            ("Ben_Tut02", "audio/voice/en/ben/tut02.ogg"),
            ("Ben_Tut03", "audio/voice/en/ben/tut03.ogg"),
            ("Ben_Tut04", "audio/voice/en/ben/tut04.ogg"),
            (
                "Cyberus_Landing",
                "audio/sfx/world_events/cyberus_landing.ogg"
            ),
            ("Ben_Tut06", "audio/voice/en/ben/tut06.ogg"),
            ("Ben_Tut08", "audio/voice/en/ben/tut08.ogg"),
            ("Ben_Tut09", "audio/voice/en/ben/tut09.ogg"),
            ("Ben_Tut10", "audio/voice/en/ben/tut10.ogg"),
            (
                "FusionButtercup_Stand1",
                "audio/voice/en/fusionbuttercup/stand1.ogg"
            ),
            (
                "Btrcup_NanSummon01_01",
                "audio/voice/en/nano_buttercup/nansummon01_01.ogg"
            ),
            ("HologramOff", "audio/sfx/environment/hologramoff.ogg"),
        ]
    );
}
