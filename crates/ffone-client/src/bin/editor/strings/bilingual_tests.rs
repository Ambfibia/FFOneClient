use super::*;
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("localization")).unwrap();
    for (locale, entries) in [
        (
            "en",
            serde_json::json!({"a":"Run {name}","b":"Run outside","c":"Running"}),
        ),
        (
            "ru",
            serde_json::json!({"a":"Бег {name}","b":"Бег снаружи","c":"Беготня"}),
        ),
    ] {
        fs::write(dir.path().join(format!("localization/{locale}.json")),serde_json::to_vec(&serde_json::json!({"schema":"ffone.text-bundle.v1","locale":locale,"custom":42,"entries":entries})).unwrap()).unwrap();
    }
    dir
}
#[test]
fn english_navigation_collapses_selection_and_tab_keeps_language() {
    let dir = fixture();
    let mut e = StringEditor::open(dir.path().into());
    e.russian = false;
    e.selected = "a".into();
    e.filtered = e.keys();
    e.cursor = e.en["a"].len();
    e.anchor = 0;
    editing_tools::move_cell(&mut e, &Key::ArrowLeft, false, false);
    assert_eq!((e.cursor, e.anchor), (0, 0));
    e.anchor = 0;
    e.cursor = e.en["a"].len();
    editing_tools::move_cell(&mut e, &Key::ArrowRight, false, false);
    assert_eq!(e.anchor, e.en["a"].len());
    editing_tools::tab_row(&mut e, false);
    assert!(!e.russian);
    assert_eq!(e.selected, "b");
    assert_eq!((e.anchor, e.cursor), (0, e.en["b"].len()));
}
#[test]
fn english_edits_save_without_changing_russian_and_keep_selection() {
    let dir = fixture();
    let mut e = StringEditor::open(dir.path().into());
    let ru = fs::read(dir.path().join("localization/ru.json")).unwrap();
    e.russian = false;
    e.selected = "a".into();
    e.cursor = 3;
    e.anchor = 0;
    e.replace_selection("Sprint");
    e.anchor = 0;
    let cursor = e.cursor;
    e.sync(true).unwrap();
    assert_eq!(
        read_bundle(dir.path(), "en").unwrap().entries["a"],
        "Sprint {name}"
    );
    assert_eq!(
        read_bundle(dir.path(), "en").unwrap().document["custom"],
        42
    );
    assert_eq!(
        fs::read(dir.path().join("localization/ru.json")).unwrap(),
        ru
    );
    assert_eq!((e.anchor, e.cursor), (0, cursor));
    assert!(!e.dirty());
    e.apply_history(true);
    assert!(!e.russian);
    assert_eq!(e.en["a"], "Run {name}");
    e.sync(true).unwrap();
    e.apply_history(false);
    e.sync(true).unwrap();
    assert_eq!(e.en["a"], "Sprint {name}");
}
#[test]
fn mixed_language_history_and_conflicts_do_not_overwrite_other_authors() {
    let dir = fixture();
    let mut e = StringEditor::open(dir.path().into());
    e.selected = "b".into();
    e.russian = false;
    e.cursor = e.en["b"].len();
    e.anchor = 0;
    e.replace_selection("English edit");
    e.russian = true;
    e.cursor = e.draft["b"].len();
    e.anchor = 0;
    e.replace_selection("Русская правка");
    e.apply_history(true);
    assert!(e.russian);
    assert_eq!(e.draft["b"], "Бег снаружи");
    e.apply_history(true);
    assert!(!e.russian);
    assert_eq!(e.en["b"], "Run outside");
    e.apply_history(false);
    let path = dir.path().join("localization/en.json");
    let mut doc = read_bundle(dir.path(), "en").unwrap().document;
    doc["entries"]["b"] = Value::String("External edit".into());
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    assert!(e.sync(true).is_err());
    assert_eq!(fs::read(path).unwrap(), before);
    assert_eq!(e.en["b"], "English edit");
}
#[test]
fn both_locales_validate_before_any_file_is_written() {
    let dir = fixture();
    let mut e = StringEditor::open(dir.path().into());
    let before = fs::read(dir.path().join("localization/en.json")).unwrap();
    e.en.insert("a".into(), "Run {person}".into());
    assert!(e.sync(true).is_err());
    assert_eq!(
        fs::read(dir.path().join("localization/en.json")).unwrap(),
        before
    );
    e.draft.insert("a".into(), "Бег {person}".into());
    e.sync(true).unwrap();
    validate(
        &read_bundle(dir.path(), "en").unwrap().entries,
        &read_bundle(dir.path(), "ru").unwrap().entries,
    )
    .unwrap();
}
#[test]
fn replacements_only_touch_current_filter_in_the_chosen_language() {
    let dir = fixture();
    let mut e = StringEditor::open(dir.path().into());
    e.tools.filter = "снаружи".into();
    e.search = "run".into();
    e.tools.replacement = "Walk".into();
    e.tools.replace_english = true;
    editing_tools::replace_all(&mut e);
    assert_eq!(e.en["a"], "Run {name}");
    assert_eq!(e.en["b"], "Walk outside");
    assert_eq!(e.en["c"], "Running");
    assert_eq!(e.undo.len(), 1);
    e.apply_history(true);
    assert_eq!(e.en["b"], "Run outside");
    e.search = "бег".into();
    e.tools.replacement = "Шаг".into();
    e.tools.replace_english = false;
    editing_tools::replace_all(&mut e);
    assert_eq!(e.draft["a"], "Бег {name}");
    assert_eq!(e.draft["b"], "Шаг снаружи");
    assert_eq!(e.draft["c"], "Беготня");
    e.tools.found = Some(("a".into(), true, 0, "Бег".len()));
    editing_tools::replace_one(&mut e);
    assert_eq!(e.draft["a"], "Бег {name}");
}
