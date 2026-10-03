use super::*;
fn map(value: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("key".into(), value.into())])
}
#[test]
fn virtual_table_stays_bounded_across_sixty_five_thousand_rows() {
    let starts: Vec<f32> = (0..=65_869).map(|i| i as f32 * 54.).collect();
    for offset in [0., 100., 50_000., 1_500_000., starts[65_869] - 600.] {
        let range = visible_range(&starts, offset, 600.);
        assert!(range.len() <= 15);
        assert!(starts[range.start] <= offset);
        assert!(starts[range.end] >= offset + 600.);
    }
    assert!(visible_range(&[0.], 0., 600.).is_empty());
}
#[test]
fn table_wheel_and_scrollbar_reach_the_end_without_pages() {
    let dir = fixture();
    let mut editor = StringEditor::open(dir.path().into());
    editor.row_starts = (0..=65_869).map(|i| i as f32 * 54.).collect();
    let maximum = editor.row_starts.last().unwrap() - 600.;
    let mut app = App::new();
    app.insert_resource(editor)
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<MouseWheel>()
        .add_systems(Update, scroll_table);
    app.world_mut().spawn((
        TableViewport,
        ComputedNode {
            size: Vec2::new(1100., 600.),
            inverse_scale_factor: 1.,
            ..default()
        },
        RelativeCursorPosition {
            cursor_over: true,
            normalized: Some(Vec2::ZERO),
        },
    ));
    let track = app
        .world_mut()
        .spawn((TableTrack, RelativeCursorPosition::default()))
        .id();
    app.update();
    app.world_mut().write_message(MouseWheel {
        phase: bevy::input::touch::TouchPhase::Moved,
        unit: MouseScrollUnit::Line,
        x: 0.,
        y: -2.,
        window: Entity::PLACEHOLDER,
    });
    app.update();
    assert_eq!(app.world().resource::<StringEditor>().offset, 120.);
    *app.world_mut()
        .get_mut::<RelativeCursorPosition>(track)
        .unwrap() = RelativeCursorPosition {
        cursor_over: true,
        normalized: Some(Vec2::new(0., 0.5)),
    };
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(app.world().resource::<StringEditor>().offset, maximum);
}
#[test]
fn merge_preserves_external_changes_and_rejects_conflicts() {
    assert_eq!(
        merge(&map("a"), &map("a"), &map("external")).unwrap(),
        map("external")
    );
    assert_eq!(
        merge(&map("a"), &map("draft"), &map("a")).unwrap(),
        map("draft")
    );
    assert!(merge(&map("a"), &map("draft"), &map("external")).is_err());
    assert_eq!(
        merge(&map("a"), &map("draft"), &map("draft")).unwrap(),
        map("draft")
    );
}
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("localization")).unwrap();
    for locale in ["en", "ru"] {
        fs::write(dir.path().join(format!("localization/{locale}.json")), serde_json::to_vec(&serde_json::json!({"schema":"ffone.text-bundle.v1", "locale":locale, "custom":42, "entries":{"key":"Hello {name}"}})).unwrap()).unwrap();
    }
    dir
}
#[test]
fn atomic_save_roundtrips_unicode_and_preserves_metadata_and_english() {
    let dir = fixture();
    let en = fs::read(dir.path().join("localization/en.json")).unwrap();
    let mut editor = StringEditor::open(dir.path().into());
    editor
        .draft
        .insert("key".into(), "Привет, {name}!\nВторая строка".into());
    editor.sync(true).unwrap();
    let ru = read_bundle(dir.path(), "ru").unwrap();
    assert_eq!(ru.entries, editor.draft);
    assert_eq!(ru.document["custom"], 42);
    assert_eq!(
        fs::read(dir.path().join("localization/en.json")).unwrap(),
        en
    );
}
#[test]
fn invalid_placeholders_or_external_conflicts_leave_file_and_draft_intact() {
    let dir = fixture();
    let mut editor = StringEditor::open(dir.path().into());
    editor.draft.insert("key".into(), "Привет".into());
    let before = read_bundle(dir.path(), "ru").unwrap().bytes;
    assert!(editor.sync(true).is_err());
    assert_eq!(read_bundle(dir.path(), "ru").unwrap().bytes, before);
    assert_eq!(editor.draft["key"], "Привет");
    editor.draft.insert("key".into(), "Привет {name}".into());
    fs::write(
        dir.path().join("localization/ru.json"),
        String::from_utf8(before)
            .unwrap()
            .replace("Hello", "External"),
    )
    .unwrap();
    assert!(editor.sync(true).is_err());
    assert!(editor.sync(false).is_err());
    assert_eq!(editor.draft["key"], "Привет {name}");
}
#[test]
fn new_english_keys_sync_without_overwriting_existing_translation() {
    let dir = fixture();
    let mut editor = StringEditor::open(dir.path().into());
    editor.draft.insert("key".into(), "Привет {name}".into());
    let mut en = read_bundle(dir.path(), "en").unwrap().document;
    en["entries"]["new.key"] = Value::String("New {value}".into());
    fs::write(
        dir.path().join("localization/en.json"),
        serde_json::to_vec(&en).unwrap(),
    )
    .unwrap();
    editor.sync(false).unwrap();
    assert_eq!(editor.draft["new.key"], "New {value}");
    assert_eq!(editor.draft["key"], "Привет {name}");
    assert!(editor.dirty());
    editor.sync(true).unwrap();
    validate(
        &read_bundle(dir.path(), "en").unwrap().entries,
        &read_bundle(dir.path(), "ru").unwrap().entries,
    )
    .unwrap();
}
#[test]
fn unrelated_external_translation_survives_save() {
    let dir = fixture();
    for locale in ["en", "ru"] {
        let mut bundle = read_bundle(dir.path(), locale).unwrap().document;
        bundle["entries"]["other"] = Value::String("Other".into());
        fs::write(
            dir.path().join(format!("localization/{locale}.json")),
            serde_json::to_vec(&bundle).unwrap(),
        )
        .unwrap();
    }
    let mut editor = StringEditor::open(dir.path().into());
    editor.draft.insert("key".into(), "Привет {name}".into());
    let mut disk = read_bundle(dir.path(), "ru").unwrap().document;
    disk["entries"]["other"] = Value::String("Внешняя правка".into());
    fs::write(
        dir.path().join("localization/ru.json"),
        serde_json::to_vec(&disk).unwrap(),
    )
    .unwrap();
    editor.sync(true).unwrap();
    let ru = read_bundle(dir.path(), "ru").unwrap();
    assert_eq!(ru.entries["other"], "Внешняя правка");
    assert_eq!(ru.entries["key"], "Привет {name}");
}
#[test]
fn selection_edits_do_not_split_cyrillic() {
    let dir = fixture();
    let mut editor = StringEditor::open(dir.path().into());
    editor.draft.insert("key".into(), "Привет".into());
    editor.cursor = 4;
    editor.anchor = 0;
    editor.replace_selection("Да");
    assert_eq!(editor.draft["key"], "Даивет");
    assert_eq!(editor.cursor, 4);
}
