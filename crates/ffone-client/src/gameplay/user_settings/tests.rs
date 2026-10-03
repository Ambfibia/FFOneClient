use std::fs;

use serde_json::{Value, json};
use tempfile::tempdir;

use super::*;

#[test]
fn local_app_data_path_has_stable_product_and_file_names() {
    let path = settings_path_from_local_app_data(Path::new("C:/Users/test/AppData/Local"));
    assert_eq!(
        path,
        Path::new("C:/Users/test/AppData/Local")
            .join(USER_SETTINGS_DIRECTORY)
            .join(USER_SETTINGS_FILE)
    );
}

#[test]
fn missing_file_returns_writable_clean_defaults() {
    let directory = tempdir().unwrap();
    let outcome = UserSettingsStore::new(directory.path().join("settings.json")).load();

    assert_eq!(outcome.origin, LoadOrigin::Missing);
    assert_eq!(outcome.settings, UserSettings::default());
    assert!(outcome.warning.is_none());
    assert!(outcome.is_writable());
}

#[test]
fn round_trip_preserves_options_input_locales_and_selection_music() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("nested").join("settings.json");
    let store = UserSettingsStore::new(&path);
    let mut expected = UserSettings::default();
    expected.options.graphics.width = 1_920;
    expected.options.graphics.height = 1_080;
    expected.options.sound.effects.volume = 0.25;
    expected.input.camera_sensitivity = 8.0;
    expected.input.invert_y = true;
    expected.input.pad_invert_y = true;
    expected.input.pad_camera_sensitivity = 2.0;
    expected.text_locale = "ru".to_owned();
    expected.voice_locale = "en".to_owned();
    expected.character_selection_music = false;
    expected.window_maximized = true;

    store.save(&expected).unwrap();
    let outcome = store.load();

    assert_eq!(outcome.origin, LoadOrigin::Loaded);
    assert_eq!(outcome.settings, expected);
    assert!(outcome.warning.is_none());
    let persisted: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(persisted["schema"], USER_SETTINGS_SCHEMA);
}

#[test]
fn older_document_without_window_maximized_keeps_preferences() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let mut expected = UserSettings::default();
    expected.text_locale = "ru".to_owned();
    let mut document: Value =
        serde_json::from_slice(&document_bytes(&expected).unwrap()).unwrap();
    document.as_object_mut().unwrap().remove("window_maximized");
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();

    let outcome = UserSettingsStore::new(path).load();
    assert_eq!(outcome.origin, LoadOrigin::Loaded);
    assert_eq!(outcome.settings, expected);
    assert!(!outcome.settings.window_maximized);
    assert!(outcome.warning.is_none());
}

#[test]
fn consecutive_saves_replace_existing_file_without_artifacts() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = UserSettingsStore::new(&path);
    let mut first = UserSettings::default();
    first.text_locale = "ru".to_owned();
    let mut second = first.clone();
    second.text_locale = "en".to_owned();
    second.voice_locale = "ru".to_owned();
    second.options.sound.music.volume = 0.75;
    second.character_selection_music = false;

    store.save(&first).unwrap();
    store.save(&second).unwrap();

    let outcome = store.load();
    assert_eq!(outcome.origin, LoadOrigin::Loaded);
    assert_eq!(outcome.settings, second);
    assert!(outcome.warning.is_none());
    assert!(!backup_path(&path).exists());
    let temporary_file_remains = fs::read_dir(directory.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    });
    assert!(!temporary_file_remains);
}

#[test]
fn valid_fields_survive_partial_recovery_of_invalid_neighbors() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = UserSettingsStore::new(&path);
    store.save(&UserSettings::default()).unwrap();

    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document["options"]["graphics"]["width"] = json!("wide");
    document["options"]["graphics"]["height"] = json!(900);
    document["options"]["sound"]["music"]["volume"] = json!(99.0);
    document["options"]["sound"]["effects"]["volume"] = json!(0.25);
    document["input"]["camera_sensitivity"] = json!(0.0);
    document["input"]["invert_y"] = json!(true);
    document["text_locale"] = json!("RU_ru");
    document["voice_locale"] = json!("bad--locale");
    document["character_selection_music"] = json!("yes");
    fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    let outcome = store.load();

    assert_eq!(outcome.origin, LoadOrigin::Loaded);
    assert_eq!(
        outcome.settings.options.graphics.width,
        OptionSettings::default().graphics.width
    );
    assert_eq!(outcome.settings.options.graphics.height, 900);
    assert_eq!(
        outcome.settings.options.sound.music.volume,
        OptionSettings::default().sound.music.volume
    );
    assert_eq!(outcome.settings.options.sound.effects.volume, 0.25);
    assert_eq!(
        outcome.settings.input.camera_sensitivity,
        InputSettings::default().camera_sensitivity
    );
    assert!(outcome.settings.input.invert_y);
    assert_eq!(outcome.settings.text_locale, "ru-ru");
    assert_eq!(outcome.settings.voice_locale, "en");
    assert!(outcome.settings.character_selection_music);
    assert!(outcome.warning.is_some());
}

#[test]
fn malformed_input_mappings_do_not_discard_other_input_preferences() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = UserSettingsStore::new(&path);
    store.save(&UserSettings::default()).unwrap();

    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document["input"]["invert_y"] = json!(true);
    document["input"]["mappings"] = json!([{"action": "NotAnAction"}]);
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();

    let outcome = store.load();

    assert_eq!(outcome.origin, LoadOrigin::Loaded);
    assert!(outcome.settings.input.invert_y);
    assert_eq!(
        outcome.settings.input.mappings,
        InputSettings::default().mappings
    );
    assert!(outcome.warning.is_some());
}

#[test]
fn syntactically_invalid_document_returns_invalid_defaults() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(&path, b"{ definitely not json").unwrap();

    let outcome = UserSettingsStore::new(path).load();

    assert_eq!(outcome.origin, LoadOrigin::Invalid);
    assert_eq!(outcome.settings, UserSettings::default());
    assert!(outcome.warning.is_some());
    assert!(outcome.is_writable());
}

#[test]
fn unsupported_schema_is_read_only_and_is_not_overwritten() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let original = br#"{"schema":"ffone.user-settings.v2","future":true}"#;
    fs::write(&path, original).unwrap();
    let store = UserSettingsStore::new(&path);

    let outcome = store.load();
    assert_eq!(outcome.origin, LoadOrigin::Unsupported);
    assert!(!outcome.is_writable());

    let error = store.save(&UserSettings::default()).unwrap_err();
    assert!(matches!(
        error,
        SettingsSaveError::UnsupportedSchema { ref found }
            if found == "ffone.user-settings.v2"
    ));
    assert_eq!(fs::read(path).unwrap(), original);
}

#[test]
fn interrupted_windows_replacement_backup_can_be_loaded() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let backup = backup_path(&path);
    let mut settings = UserSettings::default();
    settings.text_locale = "ru".to_owned();
    fs::write(&backup, document_bytes(&settings).unwrap()).unwrap();

    let outcome = UserSettingsStore::new(path).load();

    assert_eq!(outcome.origin, LoadOrigin::Loaded);
    assert_eq!(outcome.settings.text_locale, "ru");
    assert!(outcome.warning.unwrap().contains("backup"));
}

#[test]
fn failed_install_rolls_the_previous_windows_file_back() {
    let directory = tempdir().unwrap();
    let target = directory.path().join("settings.json");
    let missing_temporary = directory.path().join("missing.tmp");
    fs::write(&target, b"previous").unwrap();

    let error = replace_with_backup_rollback(&missing_temporary, &target).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert_eq!(fs::read(&target).unwrap(), b"previous");
    assert!(!backup_path(&target).exists());
}
