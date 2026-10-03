use super::*;

#[test]
fn swimming_and_stun_cues_spawn_audio_after_delay_with_localized_voice_routes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for locale in ["en", "ru"] {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            bevy::asset::AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            },
        ))
        .init_asset::<AudioSource>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(125),
        ))
        .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
        .insert_resource(VoiceLanguage {
            requested: locale.into(),
            effective: locale.into(),
        })
        .init_resource::<RetrobutionAudioMix>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, drive_gameplay_audio);
        let player = app.world_mut().spawn(Transform::default()).id();
        app.update();
        for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
            for state in [
                LegacyLocomotionState::Stun,
                LegacyLocomotionState::Swim,
                LegacyLocomotionState::SwimBack,
                LegacyLocomotionState::SwimIdle,
                LegacyLocomotionState::SwimLeft,
                LegacyLocomotionState::SwimRight,
            ] {
                let cue = player_locomotion_cue(gender, state).unwrap();
                let names = match cue {
                    SoundCue::Exact(name) => vec![name],
                    SoundCue::Random(variants) => variants.to_vec(),
                    _ => panic!("named avatar cue"),
                };
                app.world_mut()
                    .resource_mut::<GameplayAudioRuntime>()
                    .schedule(player, 0.25, cue);
                app.update();
                assert_eq!(
                    app.world_mut()
                        .query::<&AudioPlayer>()
                        .iter(app.world())
                        .count(),
                    0
                );
                app.update();
                let mut sounds = app
                    .world_mut()
                    .query::<(Entity, &AudioPlayer, Option<&LocalizedVoice>)>();
                let (entity, audio_player, voice) = sounds.single(app.world()).unwrap();
                let catalog = app.world().resource::<NativeAudioCatalog>();
                let name = voice.map_or(names[0], |voice| voice.true_name.as_str());
                assert!(names.contains(&name));
                let asset = catalog.by_true_name(name)[0];
                let expected_path = if asset.category == NativeAudioCategory::Voice {
                    assert_eq!(voice.unwrap().true_name, name);
                    catalog.path_for_locale(asset, locale).unwrap()
                } else {
                    assert!(voice.is_none());
                    asset.path.as_str()
                };
                assert_eq!(
                    app.world()
                        .resource::<AssetServer>()
                        .get_path(audio_player.0.id())
                        .unwrap()
                        .path(),
                    Path::new(expected_path)
                );
                app.world_mut().despawn(entity);
            }
        }
    }
}
