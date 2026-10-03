use super::*;

#[test]
fn larry_3000_and_incomplete_comm_out_families_resolve_native_voice() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    // Standalone UI harnesses intentionally install a tiny synthetic
    // Computress-only catalog. The production-catalog assertion belongs
    // to the real library test which owns Larry's native files.
    if catalog.by_true_name("Larry_CommOut01").is_empty() {
        return;
    }
    for take in 1..=3 {
        let requested = format!("larry_CommOut0{take}");
        for locale in ["en", "ru"] {
            let (true_name, path) =
                resolve_nanocom_voice(&catalog, locale, &requested).unwrap();
            assert_eq!(true_name, format!("Larry_CommOut0{take}"));
            assert!(root.join(path).is_file());
        }
    }

    // The native primary catalog owns only takes 01 and 02 for this
    // CommOut family. A request which hashes to take 03 must still speak
    // one of that same owner's published lines.
    let (true_name, path) =
        resolve_nanocom_voice(&catalog, "ru", "F_KNDOp3_CommOut03").unwrap();
    assert_eq!(true_name, "F_KNDOp3_CommOut01");
    assert!(root.join(path).is_file());
}

#[test]
fn suspended_message_defers_audio_entities_until_presentation_resumes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    // Numbuh Two's first CommOut currently has only EN, so preserve the
    // catalog fallback too. Larry supplies distinct EN and RU payloads.
    for (true_name, locale, resolved_locale) in [
        ("NumTwo_CommOut01", "en", "en"),
        ("NumTwo_CommOut01", "ru", "en"),
        ("Larry_CommOut01", "en", "en"),
        ("Larry_CommOut01", "ru", "ru"),
    ] {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            },
        ))
        .init_asset::<AudioSource>()
        .preregister_asset_loader::<bevy::audio::AudioLoader>(&["ogg"])
        .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
        .insert_resource(VoiceLanguage {
            requested: locale.into(),
            effective: locale.into(),
        })
        .init_resource::<NanocomMessageUiModel>()
        .add_systems(Update, play_nanocom_message_sounds);
        {
            let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
            let mut request = NanocomMessageRequest::type_9(9, "NPC", "Come in, cadet.", "");
            request.voice_true_name = Some(true_name.into());
            model.enqueue(request);
            model.set_scene_event_active(true);
        }
        app.update();
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            0
        );
        app.world_mut()
            .resource_mut::<NanocomMessageUiModel>()
            .set_scene_event_active(false);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            2
        );
        let world = app.world_mut();
        let mut voices = world.query::<(&LocalizedVoice, &AudioPlayer)>();
        let (voice, player) = voices.single(world).unwrap();
        assert_eq!(voice.true_name, true_name);
        let path = world
            .resource::<AssetServer>()
            .get_path(player.0.id())
            .unwrap();
        let (_, expected) = resolve_nanocom_voice(
            world.resource::<NativeAudioCatalog>(),
            locale,
            &voice.true_name,
        )
        .unwrap();
        assert!(expected.contains(&format!("/voice/{resolved_locale}/")));
        assert_eq!(path.path(), Path::new(&expected));
        // A dialogue can open after the voice was spawned, including
        // while decoding. Keep the same entities paused, then resume.
        app.world_mut()
            .resource_mut::<NanocomMessageUiModel>()
            .set_scene_event_active(true);
        app.update();
        assert!(
            app.world_mut()
                .query_filtered::<&PlaybackSettings, With<NanocomMessageAudio>>()
                .iter(app.world())
                .all(|settings| settings.paused)
        );
        app.world_mut()
            .resource_mut::<NanocomMessageUiModel>()
            .set_scene_event_active(false);
        app.update();
        assert!(
            app.world_mut()
                .query_filtered::<&PlaybackSettings, With<NanocomMessageAudio>>()
                .iter(app.world())
                .all(|settings| !settings.paused)
        );
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            2,
            "resuming the same head must not replay its sounds every frame"
        );
    }
}

#[test]
fn scene_event_freezes_reveal_lifetime_removal_and_audio() {
    let mut model = NanocomMessageUiModel::default();
    model.enqueue(buddy(8));
    model.pop_sound();
    model.set_scene_event_active(true);
    let before = model.active().unwrap().remaining_seconds;

    assert_eq!(
        model.choose(NanocomMessageChoice::Decline),
        None,
        "hidden scene-event UI is not actionable"
    );
    assert_eq!(model.tick(100.0), None);
    assert_eq!(model.active().unwrap().remaining_seconds, before);
    assert_eq!(model.reveal_parameter(), 1.0);
    assert_eq!(model.pop_sound(), None);
    assert!(!model.compact_visible());
    assert!(!model.expanded_visible());
}
