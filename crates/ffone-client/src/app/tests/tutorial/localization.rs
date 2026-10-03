use super::*;

#[test]
fn tutorial_auxiliary_timing_is_independent_of_text_and_voice_language() {
    use super::super::super::tutorial_runtime::cancel_tutorial_step_sequence;
    use bevy::ecs::system::RunSystemOnce;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (text_locale, voice_locale, elapsed) in [
        ("en", "en", 6.1),
        ("ru", "en", 6.1),
        ("en", "ru", 6.1),
        ("ru", "ru", 6.1),
        ("ru", "ru", 0.1),
    ] {
        let (localization, language) = Localization::open(&root, text_locale).unwrap();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<AudioSource>()
            .insert_resource(localization)
            .insert_resource(language)
            .insert_resource(VoiceLanguage {
                requested: voice_locale.into(),
                effective: voice_locale.into(),
            })
            .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
            .insert_resource(
                TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap(),
            )
            .init_resource::<TutorialSession>()
            .init_resource::<TutorialVoiceSubtitleState>()
            .init_resource::<TutorialMissionRuntime>()
            .init_resource::<TutorialAuxiliaryPresentation>()
            .init_resource::<NanocomMessageUiModel>()
            .init_resource::<PendingTutorialActorEffects>()
            .init_resource::<TutorialActorCommandQueue>()
            .init_resource::<TutorialEffectRuntime>()
            .init_resource::<RuntimeStatus>();
        app.world_mut()
            .resource_mut::<TutorialMissionRuntime>()
            .auxiliary
            .start(TutorialAuxiliarySequence::BasicArrowKey);
        // A long frame crosses both voice cues. No sink is installed, so the
        // spawned AudioPlayers also exercise cancellation before audio starts.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(elapsed));
        app.world_mut()
            .run_system_once(drive_tutorial_auxiliary_choreography)
            .unwrap();
        let mut query = app
            .world_mut()
            .query_filtered::<&LocalizedVoice, With<TutorialVoiceAudio>>();
        let voices: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(voices.len(), 1, "{text_locale}/{voice_locale}");
        assert_eq!(
            voices[0],
            &LocalizedVoice::by_true_name(if elapsed < 6.0 {
                "Computress_Tut07"
            } else {
                "Computress_Tut08"
            })
        );
        assert_eq!(
            app.world()
                .resource::<TutorialAuxiliaryPresentation>()
                .picture
                .as_ref()
                .unwrap()
                .0,
            if elapsed < 6.0 {
                "tut_move"
            } else {
                "tut_move_w"
            }
        );
        app.world_mut()
            .run_system_once(
                |mut commands: Commands,
                 voices: Query<Entity, With<TutorialVoiceAudio>>,
                 mut mission: ResMut<TutorialMissionRuntime>,
                 mut subtitles: ResMut<TutorialVoiceSubtitleState>| {
                    cancel_tutorial_step_sequence(
                        Some(TutorialStage::Movement(MovementStage::MoveForward)),
                        Some(TutorialStage::Movement(MovementStage::MoveBackward)),
                        &mut mission,
                    );
                    stop_tutorial_voice(&mut commands, &voices);
                    subtitles.clear();
                },
            )
            .unwrap();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(100));
        app.world_mut()
            .run_system_once(drive_tutorial_auxiliary_choreography)
            .unwrap();
        assert!(
            app.world()
                .resource::<TutorialMissionRuntime>()
                .auxiliary
                .active()
                .is_none()
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<TutorialVoiceAudio>>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
