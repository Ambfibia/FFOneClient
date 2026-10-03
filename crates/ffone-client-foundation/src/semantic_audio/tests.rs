use super::*;

fn fixture(rows: serde_json::Value, files: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("data/tables")).unwrap();
    fs::write(
        dir.path().join("data/tables/xdt.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema":"ffone.table-set.v1", "tables":[{"name":"native_asset_routes","value":{
                "m_pAudioData":rows,"m_pCharacterModelData":[]
            }}]
        }))
        .unwrap(),
    )
    .unwrap();
    for file in files {
        let path = dir.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"OggSfixture").unwrap();
    }
    dir
}

fn line(name: &str, path: &str) -> serde_json::Value {
    serde_json::json!({"logicalKey":format!("voice/{path}"), "trueName":name,
        "category":"voice","owner":"test","path":path})
}

#[test]
fn fallback_is_same_take_ru_then_en_and_never_en_then_ru() {
    let dir = fixture(
        serde_json::json!([
            line("Test_Hello01", "test/hello01.ogg"),
            line("Test_Hello02", "test/hello02.ogg"),
            line("Test_Hello04", "test/hello04.ogg"),
            line("Test_Hello07", "test/hello07.ogg")
        ]),
        &[
            "audio/voice/en/test/hello01.ogg",
            "audio/voice/ru/test/hello01.ogg",
            "audio/voice/en/test/hello02.ogg",
            "audio/voice/ru/test/hello04.ogg",
            "audio/voice/ru/test/hello99.ogg",
        ],
    );
    let index = NativeAudioCatalog::open(dir.path(), false).unwrap();
    let one = index.by_true_name("Test_Hello01")[0];
    let two = index.by_true_name("Test_Hello02")[0];
    let four = index.by_true_name("Test_Hello04")[0];
    let seven = index.by_true_name("Test_Hello07")[0];
    assert_eq!(
        index.path_for_locale(one, "ru"),
        Some("audio/voice/ru/test/hello01.ogg")
    );
    assert_eq!(
        index.path_for_locale(one, "en"),
        Some("audio/voice/en/test/hello01.ogg")
    );
    assert_eq!(
        index.path_for_locale(two, "ru"),
        Some("audio/voice/en/test/hello02.ogg")
    );
    assert_eq!(
        index.path_for_locale(four, "ru"),
        Some("audio/voice/ru/test/hello04.ogg")
    );
    assert_eq!(index.path_for_locale(four, "en"), None);
    assert_eq!(index.by_path("audio/voice/en/test/hello04.ogg"), Some(four));
    assert_eq!(index.path_for_locale(seven, "ru"), None);
    assert_eq!(index.path_for_locale(seven, "en"), None);
    assert!(index.by_true_name("Test_Hello99").is_empty());
    let mut selected = BTreeSet::new();
    for n in 0..12 {
        let en = index.choose_voice_family("Test_Hello01", "en", n).unwrap();
        let ru = index.choose_voice_family("Test_Hello01", "ru", n).unwrap();
        assert_eq!(en.true_name, ru.true_name);
        selected.insert(en.true_name.as_str());
    }
    assert_eq!(
        selected,
        BTreeSet::from([
            "Test_Hello01",
            "Test_Hello02",
            "Test_Hello04",
            "Test_Hello07"
        ])
    );
}

#[test]
fn english_remains_selectable_when_every_file_is_russian_only() {
    let dir = fixture(
        serde_json::json!([line("Test_Line", "test/line.ogg")]),
        &["audio/voice/ru/test/line.ogg"],
    );
    let index = NativeAudioCatalog::open(dir.path(), false).unwrap();
    assert!(index.voice_locales().any(|locale| locale == "en"));
    assert_eq!(
        index.path_for_locale(index.by_true_name("Test_Line")[0], "en"),
        None
    );
}

#[test]
fn newly_added_language_is_discovered_by_relative_path() {
    let dir = fixture(
        serde_json::json!([line("Test_Line", "test/line.ogg")]),
        &[
            "audio/voice/en/test/line.ogg",
            "audio/voice/ru/test/line.ogg",
            "audio/voice/de/test/line.ogg",
        ],
    );
    let index = NativeAudioCatalog::open(dir.path(), false).unwrap();
    let audio = index.by_true_name("Test_Line")[0];
    assert_eq!(
        index.path_for_locale(audio, "de"),
        Some("audio/voice/de/test/line.ogg")
    );
    assert_eq!(
        index.path_for_locale(audio, "fr"),
        Some("audio/voice/en/test/line.ogg")
    );
}

#[test]
fn unsafe_table_path_is_rejected() {
    let dir = fixture(serde_json::json!([line("Test_Line", "../line.ogg")]), &[]);
    assert!(NativeAudioCatalog::open(dir.path(), false).is_err());
}

#[test]
fn production_tables_resolve_all_audio_and_character_files_without_inventories() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    assert!(!root.join("_runtime/audio.json").exists());
    assert!(!root.join("_runtime/characters.json").exists());
    let index = NativeAudioCatalog::open(&root, false).unwrap();
    assert!(index.assets().len() >= 10_200);
    for asset in index.assets() {
        if asset.category != NativeAudioCategory::Voice {
            assert!(index.absolute_path(asset).is_file(), "{}", asset.path);
        }
        for variant in asset.locale_variants.values() {
            assert!(root.join(&variant.path).is_file());
        }
    }
    let female = index.by_true_name("F_Avatar_Hello01")[0];
    assert_eq!(
        index.path_for_locale(female, "ru"),
        Some("audio/voice/ru/f_avatar/avatar_hello01.ogg")
    );
    let models = crate::asset_tables::character_models(&root).unwrap();
    for model in models["models"].as_array().unwrap() {
        assert!(root.join(model["glb"].as_str().unwrap()).is_file());
        assert!(model.get("glbBlake3").is_none());
    }
}
