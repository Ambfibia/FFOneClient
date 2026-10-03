use super::*;

// Advance the asset-ready boundary without an audio device or decoder.
pub(super) fn complete_pending_npc_voice_load(app: &mut App, npc: Entity) {
    let handle = app
        .world()
        .resource::<GameplayAudioRuntime>()
        .pending_npc_voices[&npc]
        .1
        .clone();
    app.world_mut()
        .resource_mut::<Assets<AudioSource>>()
        .insert(
            handle.id(),
            AudioSource {
                bytes: Vec::new().into(),
            },
        )
        .unwrap();
    app.update();
}

#[test]
fn npc_dialogue_voice_replaces_stops_and_follows_owner_in_both_locales() {
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
        // Keep loading deterministic while the test injects ready assets.
        .preregister_asset_loader::<bevy::audio::AudioLoader>(&["ogg"])
        .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
        .insert_resource(VoiceLanguage {
            requested: locale.into(),
            effective: locale.into(),
        })
        .init_resource::<RetrobutionAudioMix>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, drive_gameplay_audio);
        let npc = app
            .world_mut()
            .spawn(Transform::from_xyz(8.0, 2.0, -4.0))
            .id();
        app.update();
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::Greeting);
        app.update();
        complete_pending_npc_voice_load(&mut app, npc);
        let first = app
            .world()
            .resource::<GameplayAudioRuntime>()
            .active_npc_voices[&npc];
        assert!(app.world().get::<PlaybackSettings>(first).unwrap().spatial);
        assert_eq!(
            app.world()
                .get::<GameplayAudioChannel>(first)
                .unwrap()
                .category,
            NativeAudioCategory::Voice
        );
        assert_eq!(app.world().get::<ChildOf>(first).unwrap().parent(), npc);
        assert_eq!(
            app.world()
                .get::<GlobalTransform>(first)
                .unwrap()
                .translation(),
            Vec3::new(8.0, 2.0, -4.0)
        );
        let voice = app.world().get::<LocalizedVoice>(first).unwrap();
        let catalog = app.world().resource::<NativeAudioCatalog>();
        let asset = catalog
            .by_true_name(&voice.true_name)
            .into_iter()
            .find(|asset| asset.category == NativeAudioCategory::Voice)
            .unwrap();
        let handle = &app.world().get::<AudioPlayer>(first).unwrap().0;
        assert_eq!(
            app.world()
                .resource::<AssetServer>()
                .get_path(handle.id())
                .unwrap()
                .path(),
            Path::new(catalog.path_for_locale(asset, locale).unwrap())
        );

        app.world_mut()
            .get_mut::<Transform>(npc)
            .unwrap()
            .translation = Vec3::new(-9.0, 3.0, 6.0);
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::QuestGreeting);
        app.update();
        assert!(
            app.world().get_entity(first).is_ok(),
            "PlayVO retains the old line while loading"
        );
        complete_pending_npc_voice_load(&mut app, npc);
        let second = app
            .world()
            .resource::<GameplayAudioRuntime>()
            .active_npc_voices[&npc];
        assert_ne!(first, second);
        assert!(
            app.world().get_entity(first).is_err(),
            "old voice must stop even before it has an audio sink"
        );
        assert_eq!(
            app.world()
                .get::<GlobalTransform>(second)
                .unwrap()
                .translation(),
            Vec3::new(-9.0, 3.0, 6.0)
        );
        {
            let mut runtime = app.world_mut().resource_mut::<GameplayAudioRuntime>();
            runtime.queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::Farewell);
            runtime.queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::Greeting);
            assert!(
                runtime.npc_voice_changes[&npc]
                    .as_ref()
                    .unwrap()
                    .starts_with("Dexter_farewell0")
            );
        }
        app.update();
        assert!(app.world().get_entity(second).is_ok());
        complete_pending_npc_voice_load(&mut app, npc);
        let farewell = app
            .world()
            .resource::<GameplayAudioRuntime>()
            .active_npc_voices[&npc];
        assert!(app.world().get_entity(second).is_err());
        let voice = app.world().get::<LocalizedVoice>(farewell).unwrap();
        assert!(
            voice
                .true_name
                .to_ascii_lowercase()
                .starts_with("dexter_farewell0")
        );
        assert_eq!(app.world().get::<ChildOf>(farewell).unwrap().parent(), npc);
        assert!(
            app.world()
                .get::<PlaybackSettings>(farewell)
                .unwrap()
                .spatial
        );
        let catalog = app.world().resource::<NativeAudioCatalog>();
        let asset = catalog
            .by_true_name(&voice.true_name)
            .into_iter()
            .find(|asset| asset.category == NativeAudioCategory::Voice)
            .unwrap();
        let handle = &app.world().get::<AudioPlayer>(farewell).unwrap().0;
        assert_eq!(
            app.world()
                .resource::<AssetServer>()
                .get_path(handle.id())
                .unwrap()
                .path(),
            Path::new(catalog.path_for_locale(asset, locale).unwrap())
        );

        // An absent clip behaves like the original failed load: no replacement.
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_npc_voice(npc, "NoSuchVoiceOwner", LegacyNpcVoiceCue::Farewell);
        app.update();
        assert!(app.world().get_entity(farewell).is_ok());
        // A genuine later reopen is allowed to replace the farewell.
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::QuestGreeting);
        app.update();
        if app
            .world()
            .resource::<GameplayAudioRuntime>()
            .pending_npc_voices
            .contains_key(&npc)
        {
            complete_pending_npc_voice_load(&mut app, npc);
        }
        assert!(app.world().get_entity(farewell).is_err());
        let reopened = app
            .world()
            .resource::<GameplayAudioRuntime>()
            .active_npc_voices[&npc];
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::QuestAccepted);
        app.update();
        assert!(app.world().get_entity(reopened).is_ok());
        assert!(
            app.world()
                .resource::<GameplayAudioRuntime>()
                .pending_npc_voices
                .contains_key(&npc)
        );
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .stop_npc_dialogue_voice(npc);
        app.update();
        assert!(app.world().get_entity(reopened).is_err());
        assert!(
            app.world()
                .resource::<GameplayAudioRuntime>()
                .pending_npc_voices
                .is_empty()
        );
        assert!(
            app.world()
                .resource::<GameplayAudioRuntime>()
                .active_npc_voices
                .is_empty()
        );

        let mut runtime = app.world_mut().resource_mut::<GameplayAudioRuntime>();
        runtime.queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::Greeting);
        runtime.stop_npc_dialogue_voice(npc);
        runtime.queue_legacy_npc_voice(npc, "Dexter", LegacyNpcVoiceCue::Greeting);
        drop(runtime);
        app.update();
        assert!(
            app.world()
                .resource::<GameplayAudioRuntime>()
                .active_npc_voices
                .is_empty(),
            "explicit stop must cancel a queued asset load too"
        );
    }
}

#[test]
fn player_damage_selects_the_exact_gender_and_critical_voice_family() {
    let player = Entity::from_bits(42);
    let cases = [
        (
            PlayerRigGender::Male,
            false,
            &["M_Avatar_Hurt01", "M_Avatar_Hurt02"][..],
        ),
        (
            PlayerRigGender::Female,
            false,
            &["F_Avatar_Hurt01", "F_Avatar_Hurt02"][..],
        ),
        (
            PlayerRigGender::Male,
            true,
            &["M_Avatar_Critical01", "M_Avatar_Critical02"][..],
        ),
        (
            PlayerRigGender::Female,
            true,
            &["F_Avatar_Critical01", "F_Avatar_Critical02"][..],
        ),
    ];
    for (gender, critical, expected) in cases {
        let mut runtime = GameplayAudioRuntime::default();
        runtime.queue_player_damage(player, gender, critical);
        assert_eq!(runtime.scheduled.len(), 1);
        assert_eq!(runtime.scheduled[0].anchor, player);
        assert!(!runtime.scheduled[0].priority);
        assert!(matches!(
            runtime.scheduled[0].cue,
            SoundCue::Random(variants) if variants == expected
        ));
    }
}

#[test]
fn player_death_voice_is_priority_audio_with_the_exact_damage_family() {
    let player = Entity::from_bits(42);
    let mut runtime = GameplayAudioRuntime::default();

    runtime.queue_player_death(player, PlayerRigGender::Female, true);
    runtime.observe_player_death_pose(
        player,
        PlayerRigGender::Female,
        crate::avatar_action::LegacyVisualClip::Die,
    );

    assert_eq!(runtime.scheduled.len(), 2);
    assert_eq!(runtime.scheduled[0].anchor, player);
    assert!(runtime.scheduled[0].priority);
    assert!(matches!(
        runtime.scheduled[0].cue,
        SoundCue::Random(variants)
            if variants == &["F_Avatar_Critical01", "F_Avatar_Critical02"]
    ));
    assert!(runtime.scheduled[1].priority);
    assert_eq!(runtime.scheduled[1].remaining_seconds, 0.25);
    assert!(
        matches!(runtime.scheduled[1].cue, SoundCue::Random(variants)
        if variants == &["F_Avatar_DefeatLng01", "F_Avatar_DefeatLng02", "F_Avatar_DefeatLng03"])
    );
    runtime.observe_player_death_pose(
        player,
        PlayerRigGender::Female,
        crate::avatar_action::LegacyVisualClip::Die,
    );
    runtime.observe_player_death_pose(
        player,
        PlayerRigGender::Female,
        crate::avatar_action::LegacyVisualClip::Death,
    );
    assert_eq!(
        runtime.scheduled.len(),
        2,
        "dead frames cannot replay the voice"
    );
}

#[test]
fn death_and_vehicle_failure_resolve_production_audio_in_both_languages() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    for gender in ["F", "M"] {
        for take in 1..=3 {
            let name = format!("{gender}_Avatar_DefeatLng0{take}");
            let assets = catalog.by_true_name(&name);
            let audio = assets
                .iter()
                .find(|audio| audio.category == NativeAudioCategory::Voice)
                .unwrap();
            for locale in ["en", "ru"] {
                let path = catalog.path_for_locale(audio, locale).unwrap();
                assert!(root.join(path).is_file(), "{name} {locale}: {path}");
            }
        }
    }
    assert!(
        catalog
            .by_true_name("Action_Failure01")
            .iter()
            .any(|audio| audio.category == NativeAudioCategory::Sfx
                && root.join(&audio.path).is_file())
    );
}

#[test]
fn death_pose_audio_covers_non_combat_death_and_can_rearm_after_respawn() {
    use crate::avatar_action::LegacyVisualClip;
    let player = Entity::from_bits(42);
    let mut runtime = GameplayAudioRuntime::default();
    runtime.observe_player_death_pose(player, PlayerRigGender::Male, LegacyVisualClip::Die);
    assert_eq!(runtime.scheduled.len(), 1, "no damage packet is required");
    runtime.observe_player_death_pose(player, PlayerRigGender::Male, LegacyVisualClip::Death);
    assert_eq!(runtime.scheduled.len(), 1);
    runtime.observe_player_death_pose(player, PlayerRigGender::Male, LegacyVisualClip::Stand1);
    runtime.observe_player_death_pose(player, PlayerRigGender::Male, LegacyVisualClip::Die);
    assert_eq!(runtime.scheduled.len(), 2);
    assert!(
        matches!(runtime.scheduled[1].cue, SoundCue::Random(variants)
        if variants == &["M_Avatar_DefeatLng01", "M_Avatar_DefeatLng02", "M_Avatar_DefeatLng03"])
    );
}

#[test]
fn infection_tick_queues_shared_sfx_and_exact_gender_voice_together() {
    let player = Entity::from_bits(42);
    for (gender, expected) in [
        (
            PlayerRigGender::Male,
            &["M_Avatar_GooDmg01", "M_Avatar_GooDmg02"][..],
        ),
        (
            PlayerRigGender::Female,
            &["F_Avatar_GooDmg01", "F_Avatar_GooDmg02"][..],
        ),
    ] {
        let mut runtime = GameplayAudioRuntime::default();
        runtime.queue_player_infection_damage(player, gender);
        assert_eq!(runtime.scheduled.len(), 2);
        assert!(matches!(
            runtime.scheduled[0].cue,
            SoundCue::Exact("SFX_PoisonDamage")
        ));
        assert!(matches!(
            runtime.scheduled[1].cue,
            SoundCue::Random(variants) if variants == expected
        ));
        assert!(
            runtime
                .scheduled
                .iter()
                .all(|sound| sound.anchor == player && !sound.priority)
        );
    }
}

#[test]
fn every_gameplay_cue_resolves_to_one_published_native_audio_asset() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let mut names = HashSet::new();
    for cue in all_cues() {
        let variants: &[&str] = match cue {
            SoundCue::Exact(name) => &[name],
            SoundCue::Random(variants) => variants,
        };
        for true_name in variants {
            if !names.insert(*true_name) {
                continue;
            }
            let candidates = catalog.by_true_name(true_name);
            assert_eq!(
                candidates.len(),
                1,
                "{true_name:?} must resolve to exactly one native audio asset"
            );
            assert!(root.join(&candidates[0].path).is_file());
            if candidates[0].category == NativeAudioCategory::Voice {
                for locale in ["en", "ru"] {
                    assert!(
                        root.join(catalog.path_for_locale(candidates[0], locale).unwrap())
                            .is_file(),
                        "{true_name:?} must resolve for {locale} voice"
                    );
                }
            }
        }
    }
    for true_name in ["Task_Completed", "Mission_Completed"] {
        let candidates = catalog
            .by_true_name(true_name)
            .into_iter()
            .filter(|audio| audio.category == NativeAudioCategory::Sfx)
            .collect::<Vec<_>>();
        let [audio] = candidates.as_slice() else {
            panic!(
                "mission completion UI cue {true_name:?} must resolve to exactly one native SFX"
            );
        };
        assert!(root.join(&audio.path).is_file());
    }
    for true_name in [
        "Tab_Click01",
        "Click_WindowSlideOut",
        "Open_Screen",
        "Close_Screen",
        "Mission_Decline",
        "Mission_Accepted",
        "Abandon_Mission",
        "Yes_Button",
        "No_Button",
    ] {
        let candidates = catalog
            .by_true_name(true_name)
            .into_iter()
            .filter(|audio| audio.category == NativeAudioCategory::Sfx)
            .collect::<Vec<_>>();
        let [audio] = candidates.as_slice() else {
            panic!("gameplay UI cue {true_name:?} must resolve to exactly one native SFX");
        };
        assert!(root.join(&audio.path).is_file());
    }
}

#[test]
fn voice_playback_uses_one_way_fallback_and_silences_missing_english() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("data/tables")).unwrap();
    std::fs::write(
        dir.path().join("data/tables/xdt.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema":"ffone.table-set.v1", "tables":[{"name":"native_asset_routes","value":{
                "m_pAudioData":[{"logicalKey":"voice/test/hello01","trueName":"Test_Hello01",
                    "category":"voice","owner":"test","path":"test/hello01.ogg"},
                {"logicalKey":"voice/test/hello04","trueName":"Test_Hello04","category":"voice","owner":"test","path":"test/hello04.ogg"}],
                "m_pCharacterModelData":[]
            }}]
        }))
        .unwrap(),
    )
    .unwrap();
    for path in [
        "audio/voice/en/test/hello01.ogg",
        "audio/voice/ru/test/hello04.ogg",
    ] {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"OggSfixture").unwrap();
    }
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin {
            file_path: dir.path().to_string_lossy().into_owned(),
            ..default()
        },
    ))
    .init_asset::<AudioSource>()
    .insert_resource(NativeAudioCatalog::open(dir.path(), false).unwrap())
    .insert_resource(VoiceLanguage {
        requested: "ru".into(),
        effective: "ru".into(),
    })
    .init_resource::<RetrobutionAudioMix>()
    .init_resource::<GameplayAudioRuntime>()
    .add_systems(Update, drive_gameplay_audio);
    let owner = app.world_mut().spawn(Transform::default()).id();
    app.update();
    for (locale, name, expected) in [
        (
            "ru",
            "Test_Hello01",
            Some("audio/voice/en/test/hello01.ogg"),
        ),
        (
            "ru",
            "Test_Hello04",
            Some("audio/voice/ru/test/hello04.ogg"),
        ),
        (
            "en",
            "Test_Hello01",
            Some("audio/voice/en/test/hello01.ogg"),
        ),
        ("en", "Test_Hello04", None),
    ] {
        app.world_mut().resource_mut::<VoiceLanguage>().effective = locale.into();
        app.world_mut()
            .resource_mut::<GameplayAudioRuntime>()
            .queue_legacy_character_animation_sound(owner, &format!("{name}.wav"));
        app.update();
        let mut sounds = app
            .world_mut()
            .query::<(Entity, &AudioPlayer, &LocalizedVoice)>();
        if let Some(expected) = expected {
            let (entity, player, voice) = sounds.single(app.world()).unwrap();
            assert_eq!(voice.true_name, name);
            assert_eq!(
                app.world()
                    .resource::<AssetServer>()
                    .get_path(player.0.id())
                    .unwrap()
                    .path(),
                Path::new(expected)
            );
            app.world_mut().despawn(entity);
        } else {
            assert_eq!(sounds.iter(app.world()).count(), 0);
        }
    }
}

#[test]
fn player_emote_sounds_resolve_and_spawn_in_both_voice_languages() {
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
        .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
        .insert_resource(VoiceLanguage {
            requested: locale.into(),
            effective: locale.into(),
        })
        .init_resource::<RetrobutionAudioMix>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, drive_gameplay_audio);
        let owner = app.world_mut().spawn(Transform::default()).id();
        app.update();
        for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
            for code in 1..=24 {
                let clip = crate::tutorial_player_presentation::TutorialPlayerClip::from_avatar_emote_code(code).unwrap();
                let events = crate::player_emote::emote_events(gender, clip).unwrap();
                for &(_, payload) in events.sounds {
                    // Exercise every random take, not just the first lucky
                    // catalog hit. The runtime expansion uses this same range.
                    let payloads = if payload.contains("(RAND:1-3)") {
                        (1..=3)
                            .map(|i| payload.replace("(RAND:1-3)", &i.to_string()))
                            .collect::<Vec<_>>()
                    } else if payload.contains("(RAND:1-2)") {
                        (1..=2)
                            .map(|i| payload.replace("(RAND:1-2)", &i.to_string()))
                            .collect()
                    } else {
                        vec![payload.to_owned()]
                    };
                    for payload in payloads {
                        app.world_mut()
                            .resource_mut::<GameplayAudioRuntime>()
                            .queue_legacy_character_animation_sound(owner, &payload);
                        app.update();
                        let mut sounds =
                            app.world_mut()
                                .query::<(Entity, &AudioPlayer, Option<&LocalizedVoice>)>();
                        let (entity, player, voice) =
                            sounds.single(app.world()).unwrap_or_else(|e| {
                                panic!("{gender:?} {clip:?} {payload} {locale}: {e}")
                            });
                        let path = app
                            .world()
                            .resource::<AssetServer>()
                            .get_path(player.0.id())
                            .unwrap();
                        assert!(root.join(path.path()).is_file(), "missing {path}");
                        if let Some(voice) = voice {
                            let catalog = app.world().resource::<NativeAudioCatalog>();
                            let asset = catalog
                                .by_true_name(&voice.true_name)
                                .into_iter()
                                .find(|a| a.category == NativeAudioCategory::Voice)
                                .unwrap();
                            assert_eq!(
                                path.path(),
                                Path::new(catalog.path_for_locale(asset, locale).unwrap())
                            );
                        } else {
                            assert!(
                                payload.contains("_SFX_Dance"),
                                "avatar vocal must carry LocalizedVoice: {payload}"
                            );
                        }
                        app.world_mut().despawn(entity);
                    }
                }
            }
        }
    }
}

#[test]
fn actor_sound_events_follow_the_loaded_animation_clock_and_repeats() {
    assert_eq!(
        animation_event_crossings(0.0, 0, 0.3, 0, RepeatAnimation::Never, 0.25),
        1
    );
    assert_eq!(
        animation_event_crossings(0.1, 0, 0.1, 1, RepeatAnimation::Forever, 0.25),
        1
    );
    assert_eq!(
        animation_event_crossings(0.1, 0, 0.1, 3, RepeatAnimation::Forever, 0.25),
        3
    );
    assert_eq!(
        animation_event_crossings(0.3, 0, 0.1, 1, RepeatAnimation::Forever, 0.25),
        0
    );
}

#[test]
fn animation_sound_payloads_expand_all_inclusive_random_tokens_and_route_nano_voice() {
    let mut random = GameplayAudioRandom::with_seed(1);
    for _ in 0..64 {
        let expanded = expand_legacy_random_sound(
            "Btrcup_NanDance0(RAND:1-9)_(RAND:2-4).wav",
            &mut random,
        )
        .unwrap();
        let (take, suffix) = expanded
            .trim_start_matches("Btrcup_NanDance0")
            .split_once('_')
            .unwrap();
        assert!((1..=9).contains(&take.parse::<i32>().unwrap()));
        assert!((2..=4).contains(&suffix.parse::<i32>().unwrap()));
    }

    let anchor = Entity::from_bits(7);
    let mut runtime = GameplayAudioRuntime {
        random: GameplayAudioRandom::with_seed(9),
        ..default()
    };
    runtime.queue_legacy_nano_animation_sound(anchor, "Btrcup_NanSummon01_01.wav");
    runtime.queue_legacy_nano_animation_sound(anchor, "Nano Ability 06.wav");
    runtime.queue_legacy_character_animation_sound(anchor, "Eduardo_Hurt1_1.wav");
    assert!(runtime.animation_sounds[0].voice_route);
    assert!(!runtime.animation_sounds[1].voice_route);
    assert!(
        runtime.animation_sounds[2].voice_route,
        "NPC and mob animation cues must prefer localized voice when the catalog owns both voice and SFX copies"
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    assert!(
        semantic_numeric_family(&catalog, "Btrcup_NanDance09")
            .iter()
            .any(|asset| asset.category == NativeAudioCategory::Voice
                && asset.true_name == "Btrcup_NanDance01")
    );
    assert!(
        semantic_numeric_family(&catalog, "Eduardo_Hurt1_5")
            .iter()
            .any(|asset| asset.category == NativeAudioCategory::Voice
                && asset.true_name == "Eduardo_Hurt1_1")
    );

    for _ in 0..64 {
        runtime.queue_legacy_button_sound();
    }
    assert!(runtime.ui_sounds.iter().all(|sound| {
        matches!(
            sound.true_name.as_str(),
            "mouse_click01"
                | "mouse_click02"
                | "mouse_click03"
                | "mouse_click04"
                | "mouse_click05"
        ) && sound.clean_gain == 0.7
    }));

    runtime.ui_sounds.clear();
    runtime.queue_gameplay_ui_sound("Open_Screen");
    runtime.queue_gameplay_ui_sound("Tab_Click01");
    runtime.queue_gameplay_ui_sound("Task_Completed");
    runtime.queue_gameplay_ui_sound("Mission_Completed");
    assert_eq!(
        runtime
            .ui_sounds
            .iter()
            .map(|sound| (sound.true_name.as_str(), sound.clean_gain))
            .collect::<Vec<_>>(),
        vec![
            ("Open_Screen", 0.7),
            ("Tab_Click01", 0.7),
            ("Task_Completed", 0.7),
            ("Mission_Completed", 0.7),
        ]
    );

    runtime.ui_sounds.clear();
    runtime.queue_user_equip_mode_edge(true);
    runtime.queue_user_equip_mode_edge(false);
    assert_eq!(
        runtime
            .ui_sounds
            .iter()
            .map(|sound| (sound.true_name.as_str(), sound.clean_gain))
            .collect::<Vec<_>>(),
        vec![
            ("Open_Screen", 0.7),
            ("Open_Screen", 0.7),
            ("Close_Screen", 0.7),
            ("Close_Screen", 0.7),
        ]
    );
}
