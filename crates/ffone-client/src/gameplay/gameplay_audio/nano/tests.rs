use super::*;
use std::path::Path;

fn audio_app(locale: &str) -> App {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin {
            file_path: root.to_string_lossy().into_owned(),
            ..default()
        },
    ))
    .init_asset::<AudioSource>()
    .preregister_asset_loader::<bevy::audio::AudioLoader>(&["ogg"])
    .insert_resource(NativeAudioCatalog::open(root, false).unwrap())
    .insert_resource(VoiceLanguage {
        requested: locale.into(),
        effective: locale.into(),
    })
    .init_resource::<RetrobutionAudioMix>()
    .init_resource::<GameplayAudioRuntime>()
    .add_systems(Update, drive_gameplay_audio);
    app
}

fn finish_load(app: &mut App) {
    let handles: Vec<_> = app
        .world()
        .resource::<GameplayAudioRuntime>()
        .nano
        .pending
        .iter()
        .filter_map(|voice| voice.handle.clone())
        .collect();
    assert!(!handles.is_empty());
    for handle in handles {
        app.world_mut()
            .resource_mut::<Assets<AudioSource>>()
            .insert(
                handle.id(),
                AudioSource {
                    bytes: Vec::new().into(),
                },
            )
            .unwrap();
    }
    app.update();
}

#[test]
fn repaired_nano_model_events_spawn_localized_summon_and_all_skill_voices() {
    use crate::network_world_runtime::parse_network_npc_animation_sound_events;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for locale in ["en", "ru"] {
        let mut app = audio_app(locale);
        let nano = app.world_mut().spawn(Transform::default()).id();
        app.update();
        for (owner, prefix, time, skills) in [
            (
                "nano_flapjack",
                "Flapjack",
                0.25,
                ["LGnat", "ALullabye", "FBurn"],
            ),
            ("nano_chowder", "Chowder", 0.1, ["STake", "INYB", "BMilk"]),
            (
                "nano_zaksaturday",
                "Zak",
                0.1,
                ["CAssault", "Claw", "MHunter"],
            ),
            (
                "nano_jackolantern",
                "Jack_O_Lantern",
                0.01,
                ["TT", "LL", "DLYH"],
            ),
        ] {
            let bytes =
                std::fs::read(root.join(format!("characters/nanos/{owner}/{owner}.glb")))
                    .unwrap();
            let events = parse_network_npc_animation_sound_events(&bytes).unwrap();
            for (clip, family) in
                std::iter::once(("call".to_owned(), format!("{prefix}_NanSummon"))).chain(
                    skills.into_iter().enumerate().map(|(slot, cue)| {
                        (
                            format!("skill{}", slot + 1),
                            format!("{prefix}_NanPwr{cue}"),
                        )
                    }),
                )
            {
                let voices: Vec<_> = events
                    .iter()
                    .filter(|e| e.clip == clip && e.payload.starts_with(&family))
                    .collect();
                assert_eq!(
                    voices.len(),
                    1,
                    "{owner}/{clip} must request its own voice once"
                );
                assert!((voices[0].time - time).abs() < 0.000_001);
                for take in 1..=3 {
                    let catalog = app.world().resource::<NativeAudioCatalog>();
                    let name = format!("{family}{take:02}");
                    let candidates = catalog.by_true_name(&name);
                    let audio = candidates
                        .iter()
                        .find(|a| a.category == NativeAudioCategory::Voice && a.owner == owner)
                        .unwrap();
                    assert!(
                        root.join(catalog.path_for_locale(audio, locale).unwrap())
                            .is_file()
                    );
                }
                app.world_mut()
                    .resource_mut::<GameplayAudioRuntime>()
                    .queue_legacy_nano_animation_sound(nano, &voices[0].payload);
                app.update();
                finish_load(&mut app);
                let current =
                    app.world().resource::<GameplayAudioRuntime>().nano.current[&nano];
                let voice = app.world().get::<LocalizedVoice>(current).unwrap();
                assert!(
                    voice.true_name.starts_with(&family),
                    "{owner}/{clip}: {}",
                    voice.true_name
                );
                let player = app.world().get::<AudioPlayer>(current).unwrap();
                let catalog = app.world().resource::<NativeAudioCatalog>();
                let candidates = catalog.by_true_name(&voice.true_name);
                let audio = candidates.iter().find(|a| a.owner == owner).unwrap();
                assert_eq!(
                    app.world()
                        .resource::<AssetServer>()
                        .get_path(player.0.id())
                        .unwrap()
                        .path(),
                    Path::new(catalog.path_for_locale(audio, locale).unwrap())
                );
                app.world_mut().despawn(current);
            }
        }
    }
}

#[test]
fn every_installed_nano_voice_event_reaches_localized_playback() {
    use crate::network_world_runtime::parse_network_npc_animation_sound_events;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&root).unwrap();
    let portraits =
        crate::gameplay_nano_portraits::GameplayNanoPortraitCatalog::open(&locator).unwrap();
    for locale in ["en", "ru"] {
        let mut app = audio_app(locale);
        let nano = app.world_mut().spawn(Transform::default()).id();
        app.update();
        let mut checked = 0;
        for id in 1..=i16::MAX {
            let Some(path) = portraits.model_path(id) else {
                continue;
            };
            let events = parse_network_npc_animation_sound_events(
                &std::fs::read(root.join(path)).unwrap(),
            )
            .unwrap();
            // These accepted models have incomplete or non-vocal source clips.
            let incomplete_source = [
                "nano_ben/",
                "nano_cheese/",
                "nano_ghostfreak/",
                "nano_upgrade/",
                "nano_holonano/",
            ];
            if !incomplete_source.iter().any(|owner| path.contains(owner)) {
                for clip in ["call", "skill1", "skill2", "skill3"] {
                    assert!(
                        events
                            .iter()
                            .any(|event| event.clip.eq_ignore_ascii_case(clip)),
                        "Nano {id} is missing {clip} audio events in {path}"
                    );
                }
            }
            for event in events.iter().filter(|e| {
                matches!(
                    e.clip.to_ascii_lowercase().as_str(),
                    "call" | "skill1" | "skill2" | "skill3"
                )
            }) {
                // These models contain source placeholders without a recorded voice.
                if path.contains("nano_holonano/")
                    || event.payload.starts_with("Titan_NanPwrBRockets")
                {
                    continue;
                }
                let payload = event.payload.to_ascii_lowercase();
                if !["_nan", "_summon0", "_pwr", "computress_skill"]
                    .iter()
                    .any(|marker| payload.contains(marker))
                {
                    continue;
                }
                app.world_mut()
                    .resource_mut::<GameplayAudioRuntime>()
                    .queue_legacy_nano_animation_sound(nano, &event.payload);
                app.update();
                if !app
                    .world()
                    .resource::<GameplayAudioRuntime>()
                    .nano
                    .pending
                    .is_empty()
                {
                    finish_load(&mut app);
                }
                assert!(
                    app.world()
                        .resource::<GameplayAudioRuntime>()
                        .nano
                        .current
                        .contains_key(&nano),
                    "Nano {id} {locale} {}: {} did not play",
                    event.clip,
                    event.payload
                );
                let current =
                    app.world().resource::<GameplayAudioRuntime>().nano.current[&nano];
                let voice = app.world().get::<LocalizedVoice>(current).unwrap();
                let catalog = app.world().resource::<NativeAudioCatalog>();
                let candidates = catalog.by_true_name(&voice.true_name);
                let audio = candidates
                    .iter()
                    .find(|a| a.category == NativeAudioCategory::Voice)
                    .unwrap();
                assert!(
                    root.join(catalog.path_for_locale(audio, locale).unwrap())
                        .is_file(),
                    "Nano {id} {locale}: {}",
                    voice.true_name
                );
                app.world_mut().despawn(current);
                checked += 1;
            }
        }
        assert!(
            checked >= 230,
            "only {checked} production call/power voice events checked in {locale}"
        );
    }
}

#[test]
fn candy_buccaneer_attack_resolves_to_combat_audio_in_both_languages() {
    for locale in ["en", "ru"] {
        let mut app = audio_app(locale);
        let pirate = app.world_mut().spawn(Transform::default()).id();
        app.update();
        let catalog = app.world().resource::<NativeAudioCatalog>();
        let candidates = catalog.by_true_name("Pirate_Melee1");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].logical_key, "sfx/creatures/pirate/melee1");
        assert_eq!(candidates[0].category, NativeAudioCategory::Sfx);
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_character_animation_sound(pirate, "Pirate_Melee1.wav");
        app.update();
        let mut sounds = app
            .world_mut()
            .query::<(&AudioPlayer, Option<&LocalizedVoice>)>();
        let (player, voice) = sounds.single(app.world()).unwrap();
        assert!(voice.is_none());
        assert_eq!(
            app.world()
                .resource::<AssetServer>()
                .get_path(player.0.id())
                .unwrap()
                .path(),
            Path::new("audio/sfx/creatures/pirate/melee1.ogg")
        );
    }
}

#[test]
fn published_nano_dismissal_takes_resolve_in_both_voice_languages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let registry: serde_json::Value =
        ffone_client_foundation::asset_tables::character_models(&root).unwrap();
    let mut checked = 0;
    let mut missing = Vec::new();
    for model in registry["models"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|model| model["category"] == "nano")
    {
        let owner = Path::new(model["glb"].as_str().unwrap())
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(model["id"].as_str().unwrap(), format!("nano/{owner}"));
        if dismissal_asset(&catalog, owner, 1).is_none() {
            missing.push(owner.to_owned());
            continue;
        }
        for take in 1..=3 {
            let audio = dismissal_asset(&catalog, owner, take)
                .unwrap_or_else(|| panic!("{owner} has no farewell take {take}"));
            for locale in ["en", "ru"] {
                let path = catalog.absolute_path_for_locale(audio, locale).unwrap();
                let bytes = std::fs::read(path).unwrap();
                assert_eq!(&bytes[..4], b"OggS");
            }
        }
        checked += 1;
    }
    assert!(checked >= 60, "must cover the production roster");
    assert_eq!(
        missing,
        [
            "nano_alienx",
            "nano_ghostfreak",
            "nano_holonano",
            "nano_runty",
            "nano_upgrade"
        ]
    );
    assert!(dismissal_asset(&catalog, "nano_missing", 1).is_none());
}

#[test]
fn nano_dismissal_waits_for_voice_and_survives_model_removal_in_en_and_ru() {
    for locale in ["en", "ru"] {
        let mut app = audio_app(locale);
        let owner = app
            .world_mut()
            .spawn(Transform::from_xyz(1.0, 2.0, 3.0))
            .id();
        let nano = app
            .world_mut()
            .spawn(Transform::from_xyz(2.0, 3.0, 4.0))
            .id();
        app.update();
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_nano_animation_sound(nano, "Btrcup_NanSummon01_01.wav");
        app.update();
        finish_load(&mut app);
        let current = app.world().resource::<GameplayAudioRuntime>().nano.current[&nano];
        assert!(app.world().get::<ChildOf>(current).is_none());
        assert!(app.world().get::<LocalizedVoice>(current).is_some());

        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_nano_animation_sound(nano, "Btrcup_NanPwrMF01_01.wav");
        app.update();
        let pending = &app.world().resource::<GameplayAudioRuntime>().nano.pending;
        assert_eq!(pending.len(), 1);
        assert!(pending[0].handle.is_none(), "VOLoad waits before loading");

        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_nano_dismissal(nano, owner, "nano_buttercup");
        app.world_mut().despawn(nano);
        app.update();
        assert!(
            app.world().get_entity(current).is_ok(),
            "Hide must not cut off currentVO"
        );
        let pending = &app.world().resource::<GameplayAudioRuntime>().nano.pending;
        assert_eq!(
            pending.len(),
            1,
            "the removed model's queued skill is cancelled"
        );
        assert_eq!(pending[0].anchor, owner);
        assert!(pending[0].handle.is_none());

        // Deterministic playback completion without depending on a device.
        app.world_mut().despawn(current);
        app.world_mut()
            .get_mut::<Transform>(owner)
            .unwrap()
            .translation = Vec3::new(5.0, 6.0, 7.0);
        app.update();
        app.update();
        finish_load(&mut app);
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &AudioPlayer, &LocalizedVoice, &Transform)>();
        let voices: Vec<_> = query.iter(world).collect();
        assert_eq!(voices.len(), 1);
        let (farewell, player, _, transform) = voices[0];
        assert_eq!(transform.translation, Vec3::new(5.0, 6.0, 7.0));
        let path = world
            .resource::<AssetServer>()
            .get_path(player.0.id())
            .unwrap();
        let catalog = world.resource::<NativeAudioCatalog>();
        let audio = catalog.by_path(path.path().to_str().unwrap()).unwrap();
        assert!(audio.true_name.contains("_NanDismiss0"));
        assert_eq!(
            path.path().to_str().unwrap(),
            catalog.path_for_locale(audio, locale).unwrap()
        );
        assert!(world.get::<ChildOf>(farewell).is_none());
        world.despawn(owner);
        app.update();
        assert!(
            app.world().get_entity(farewell).is_ok(),
            "a started one-shot finishes independently"
        );
        assert!(
            app.world()
                .resource::<GameplayAudioRuntime>()
                .nano
                .pending
                .is_empty()
        );
    }
}

#[test]
fn nano_farewell_pending_load_is_cancelled_when_avatar_leaves() {
    let mut app = audio_app("en");
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let nano = app.world_mut().spawn(Transform::IDENTITY).id();
    app.update();
    app.world_mut()
        .resource_mut::<GameplayAudioRuntime>()
        .queue_nano_dismissal(nano, owner, "nano_belladonna");
    app.world_mut().despawn(nano);
    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .nano
            .pending
            .len(),
        1
    );
    app.world_mut().despawn(owner);
    app.update();
    assert!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .nano
            .pending
            .is_empty()
    );
}
