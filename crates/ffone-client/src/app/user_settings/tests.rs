use super::*;
use ffone_client::localization::DEFAULT_LANGUAGE;

#[test]
fn text_locale_precedence_validates_overrides_and_preserves_saved_request() {
    let available = ["en", "ru"];
    assert_eq!(
        resolve_startup_text_locale(Some("RU_ru"), Some("en"), Some("en"), "en", available,),
        "ru-ru"
    );
    assert_eq!(
        resolve_startup_text_locale(
            Some("../../ru"),
            Some("de-DE"),
            Some("ru"),
            "en",
            available,
        ),
        "ru"
    );
    assert_eq!(
        resolve_startup_text_locale(None, Some("EN_us"), Some("ru"), "en", available),
        "en-us"
    );
    assert_eq!(
        resolve_startup_text_locale(None, None, Some("DE_de"), "en", available),
        "de-de"
    );
}

#[test]
fn first_run_prefers_russian_without_changing_saved_english_or_voice() {
    let available = ["en", "ru"];
    let fresh = UserSettings::default();
    assert_eq!(fresh.text_locale, "ru");
    assert_eq!(fresh.voice_locale, "en");
    assert_eq!(
        resolve_startup_text_locale(
            None,
            None,
            Some(&fresh.text_locale),
            DEFAULT_LANGUAGE,
            available,
        ),
        "ru"
    );
    assert_eq!(
        resolve_startup_text_locale(None, None, Some("en"), DEFAULT_LANGUAGE, available),
        "en"
    );
    assert_eq!(
        resolve_startup_text_locale(
            Some("ru"),
            Some("en"),
            Some("en"),
            DEFAULT_LANGUAGE,
            available,
        ),
        "ru"
    );
}

#[test]
fn startup_override_does_not_replace_saved_choice_until_user_selects_language() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = UserSettingsStore::new(&path);
    let saved = UserSettings::default();
    store.save(&saved).unwrap();
    let mut prepared = prepare_user_settings_at(&path);
    prepared.retain_saved_text_during_override("en");
    let mut runtime = saved.clone();
    runtime.text_locale = "en".to_owned();
    runtime.character_selection_music = false;
    assert_eq!(
        flush_user_settings_snapshot(&mut prepared.persistence, runtime.clone()).unwrap(),
        UserSettingsFlushOutcome::Saved
    );
    assert_eq!(store.load().settings.text_locale, "ru");
    runtime.text_locale = "ru".to_owned();
    assert_eq!(
        flush_user_settings_snapshot(&mut prepared.persistence, runtime.clone()).unwrap(),
        UserSettingsFlushOutcome::Unchanged
    );
    runtime.text_locale = "en".to_owned();
    assert_eq!(
        flush_user_settings_snapshot(&mut prepared.persistence, runtime).unwrap(),
        UserSettingsFlushOutcome::Saved
    );
    assert_eq!(store.load().settings.text_locale, "en");
}

#[test]
fn voice_locale_remains_independent_and_uses_text_then_catalog_fallbacks() {
    let text = Language {
        requested: "ru-RU".to_owned(),
        effective: "ru".to_owned(),
    };
    assert_eq!(
        resolve_startup_voice_language(Some("EN_us"), &text, ["en", "ru"]),
        VoiceLanguage {
            requested: "en-us".to_owned(),
            effective: "en".to_owned(),
        }
    );
    assert_eq!(
        resolve_startup_voice_language(Some("de"), &text, ["en", "ru"]),
        VoiceLanguage {
            requested: "de".to_owned(),
            effective: "ru".to_owned(),
        }
    );
    assert_eq!(
        resolve_startup_voice_language(None, &text, ["en"]),
        VoiceLanguage {
            requested: "ru-ru".to_owned(),
            effective: "en".to_owned(),
        }
    );
}

#[test]
fn complete_snapshot_includes_options_input_locales_and_selection_music() {
    let mut options = OptionProductionRuntime::default();
    options.options.sound.music.volume = 0.25;
    options.input.invert_y = true;
    let text = Language {
        requested: "ru".to_owned(),
        effective: "ru".to_owned(),
    };
    let voice = VoiceLanguage {
        requested: "en".to_owned(),
        effective: "en".to_owned(),
    };
    let character_selection = CharacterSelectionUiModel {
        music_enabled: false,
        ..default()
    };

    let snapshot =
        collect_user_settings_snapshot(&options, &text, &voice, &character_selection);
    assert_eq!(snapshot.options.sound.music.volume, 0.25);
    assert!(snapshot.input.invert_y);
    assert_eq!(snapshot.text_locale, "ru");
    assert_eq!(snapshot.voice_locale, "en");
    assert!(!snapshot.character_selection_music);
}

#[test]
fn loaded_snapshot_wins_over_live_window_and_camera_seed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = UserSettingsStore::new(&path);
    let mut saved = UserSettings::default();
    saved.options.graphics.width = 1_920;
    saved.options.graphics.height = 1_080;
    saved.options.graphics.glow = true;
    saved.input.camera_sensitivity = 7.0;
    store.save(&saved).unwrap();

    let prepared = prepare_user_settings_at(path);
    assert_eq!(prepared.origin, LoadOrigin::Loaded);
    let mut runtime = prepared.option_runtime();
    let live_window = Window {
        resolution: (800, 600).into(),
        ..default()
    };
    runtime.initialize_from_live_runtime(&live_window, false, Some(1.0));

    assert!(runtime.initialized_from_live_runtime);
    assert_eq!(runtime.options.graphics.width, 1_920);
    assert_eq!(runtime.options.graphics.height, 1_080);
    assert!(runtime.options.graphics.glow);
    assert_eq!(runtime.input.camera_sensitivity, 7.0);
}

#[test]
fn missing_document_is_not_written_until_the_live_snapshot_changes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let prepared = prepare_user_settings_at(&path);
    assert_eq!(prepared.origin, LoadOrigin::Missing);
    assert!(prepared.persistence.writes_enabled());
    assert!(!path.exists());

    let mut persistence = prepared.persistence;
    assert_eq!(
        flush_user_settings_snapshot(&mut persistence, prepared.settings.clone()).unwrap(),
        UserSettingsFlushOutcome::Unchanged
    );
    assert!(!path.exists());

    let mut changed = prepared.settings;
    changed.character_selection_music = false;
    assert_eq!(
        flush_user_settings_snapshot(&mut persistence, changed.clone()).unwrap(),
        UserSettingsFlushOutcome::Saved
    );
    assert_eq!(UserSettingsStore::new(path).load().settings, changed);
}

#[test]
fn disabled_writer_observes_changes_without_touching_disk() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let original = UserSettings::default();
    let mut persistence = UserSettingsPersistence::new(path.clone(), original.clone(), false);
    let mut changed = original;
    changed.character_selection_music = false;

    assert_eq!(
        flush_user_settings_snapshot(&mut persistence, changed.clone()).unwrap(),
        UserSettingsFlushOutcome::WritesDisabled
    );
    assert_eq!(persistence.last_observed(), &changed);
    assert!(!path.exists());
}
