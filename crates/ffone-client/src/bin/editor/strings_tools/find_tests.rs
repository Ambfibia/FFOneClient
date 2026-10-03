use super::*;
fn editor() -> StringEditor {
    let mut e = StringEditor::open(PathBuf::from("target/no-search-fixture"));
    e.en = BTreeMap::from([
        ("a".into(), "Hello".into()),
        ("b".into(), "Related reply".into()),
        ("c".into(), "Hello again".into()),
    ]);
    e.draft = BTreeMap::from([
        ("a".into(), "Привет".into()),
        ("b".into(), "Соседняя реплика".into()),
        ("c".into(), "ПРИВЕТ снова".into()),
    ]);
    e.base = e.draft.clone();
    e.filtered = e.keys();
    e.row_indices = e
        .filtered
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, k)| (k, i))
        .collect();
    e.row_starts = vec![0., 54., 108., 162.];
    e.selected = "a".into();
    e
}
#[test]
fn filter_and_find_have_independent_queries() {
    let mut e = editor();
    e.tools.filter = "снова".into();
    e.search = "привет".into();
    assert_eq!(e.keys(), vec!["c"]);
    e.filtered = e.keys();
    find_next(&mut e, false);
    assert_eq!(e.selected, "c");
    assert_eq!(e.tools.filter, "снова");
    e.tools.filter.clear();
    assert_eq!(e.keys().len(), 3);
    assert_eq!(e.search, "привет");
    e.tools.filter = "RELATED".into();
    assert_eq!(e.keys(), vec!["b"]);
}
#[test]
fn search_never_filters_and_cycles_matches_with_utf8_offsets() {
    let mut e = editor();
    e.search = "привет".into();
    let rows = e.keys();
    assert_eq!(rows.len(), 3);
    find_next(&mut e, false);
    assert_eq!(e.selected, "a");
    assert_eq!((e.anchor, e.cursor), (0, "Привет".len()));
    find_next(&mut e, false);
    assert_eq!(e.selected, "c");
    find_next(&mut e, false);
    assert_eq!(e.selected, "a");
    find_next(&mut e, true);
    assert_eq!(e.selected, "c");
    assert_eq!(e.filtered, rows);
    e.search = "absent".into();
    find_next(&mut e, false);
    assert_eq!(e.keys(), rows);
    assert_eq!(e.status.key, "ui.editor.strings.no_match");
}
#[test]
fn tab_selects_entire_next_translation_for_paste() {
    let mut e = editor();
    tab_row(&mut e, false);
    assert_eq!(e.selected, "b");
    assert_eq!(e.anchor, 0);
    assert_eq!(e.cursor, e.draft["b"].len());
    e.replace_selection("Вставлено");
    assert_eq!(e.draft["b"], "Вставлено");
    tab_row(&mut e, true);
    assert_eq!(e.selected, "a");
    assert_eq!((e.anchor, e.cursor), (0, "Привет".len()));
}
#[test]
fn replacement_uses_same_case_insensitive_matches_and_one_undo() {
    let mut e = editor();
    e.search = "привет".into();
    e.tools.replacement = "Здравствуйте".into();
    let before = e.draft.clone();
    replace_all(&mut e);
    assert_eq!(e.draft["a"], "Здравствуйте");
    assert_eq!(e.draft["c"], "Здравствуйте снова");
    e.apply_history(true);
    assert_eq!(e.draft, before);
    e.tools.found = None;
    find_next(&mut e, false);
    replace_one(&mut e);
    assert_eq!(e.draft["a"], "Здравствуйте");
    assert_eq!(e.draft["c"], "ПРИВЕТ снова");
}
