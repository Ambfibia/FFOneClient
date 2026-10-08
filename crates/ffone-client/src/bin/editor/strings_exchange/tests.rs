use super::*;
fn rows() -> Vec<ExchangeRow> {
    vec![
        ExchangeRow {
            key: "a".into(),
            en: "Hello, \"{name}\"!\nSecond line".into(),
            ru: "Привет, \"{name}\"!\nВторая строка".into(),
        },
        ExchangeRow {
            key: "b".into(),
            en: "Bye".into(),
            ru: "Пока".into(),
        },
    ]
}
fn maps(rows: &[ExchangeRow]) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    (
        rows.iter().map(|r| (r.key.clone(), r.en.clone())).collect(),
        rows.iter().map(|r| (r.key.clone(), r.ru.clone())).collect(),
    )
}
#[test]
fn csv_roundtrips_unicode_quotes_commas_newlines_and_bom() {
    let rows = rows();
    let bytes = encode(&rows).unwrap();
    assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert_eq!(decode(&bytes).unwrap(), rows);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("batch.csv");
    write_csv(&path, &rows).unwrap();
    assert_eq!(decode(&fs::read(path).unwrap()).unwrap(), rows);
}
#[test]
fn reordered_import_matches_keys_and_rejects_wrong_selection_or_source() {
    let original = rows();
    let (en, ru) = maps(&original);
    let mut translated = original.clone();
    translated.reverse();
    translated[0].ru = "До свидания".into();
    let plan = plan_import(translated.clone(), &original, &en, &ru).unwrap();
    assert_eq!(plan[1], ("b".into(), "До свидания".into()));
    assert!(plan_import(translated.clone(), &original[..1], &en, &ru).is_err());
    translated[0].en = "Changed".into();
    assert!(plan_import(translated, &original, &en, &ru).is_err());
}
#[test]
fn invalid_batch_never_changes_any_draft_and_import_is_one_undo_step() {
    let dir = tempfile::tempdir().unwrap();
    let mut editor = StringEditor::open(dir.path().into());
    let original = rows();
    let (en, ru) = maps(&original);
    editor.en = en;
    editor.base = ru.clone();
    editor.draft = ru.clone();
    let mut translated = original.clone();
    translated[0].ru = "Missing placeholder".into();
    translated[1].ru = "Пока-пока".into();
    assert!(plan_import(translated.clone(), &original, &editor.en, &editor.draft).is_err());
    assert_eq!(editor.draft, ru);
    translated[0].ru = "Здравствуйте, {name}".into();
    let plan = plan_import(translated, &original, &editor.en, &editor.draft).unwrap();
    apply_import(&mut editor, plan);
    assert_eq!(editor.undo.len(), 1);
    assert_eq!(editor.draft["b"], "Пока-пока");
    editor.apply_history(true);
    assert_eq!(editor.draft, ru);
    editor.apply_history(false);
    assert_eq!(editor.draft["b"], "Пока-пока");
}
#[test]
fn a_hundred_translations_roundtrip_as_one_batch_without_touching_other_rows() {
    let original: Vec<_> = (0..100)
        .map(|i| ExchangeRow {
            key: format!("line.{i:03}"),
            en: format!("Line {i}"),
            ru: format!("Строка {i}"),
        })
        .collect();
    let (mut en, mut draft) = maps(&original);
    en.insert("outside".into(), "Do not change".into());
    draft.insert("outside".into(), "Не изменять".into());
    let mut imported = decode(&encode(&original).unwrap()).unwrap();
    imported.reverse();
    for row in &mut imported {
        row.ru = format!("Перевод {}", row.key);
    }
    let plan = plan_import(imported, &original, &en, &draft).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut editor = StringEditor::open(dir.path().into());
    editor.en = en;
    editor.base = draft.clone();
    editor.draft = draft.clone();
    apply_import(&mut editor, plan);
    assert_eq!(editor.draft["outside"], "Не изменять");
    assert_eq!(editor.draft["line.099"], "Перевод line.099");
    assert_eq!(editor.undo.len(), 1);
    editor.apply_history(true);
    assert_eq!(editor.draft, draft);
    let swapped = b"ru,key,en\nRussian,a,English\n";
    assert_eq!(decode(swapped).unwrap()[0].key, "a");
}
#[test]
fn malformed_duplicates_and_missing_rows_are_rejected() {
    assert!(decode(b"en,ru\nHello,World\n").is_err());
    assert!(decode(b"key,en,ru\na,Hello,World,extra\n").is_err());
    let original = rows();
    let (en, ru) = maps(&original);
    let mut duplicate = original.clone();
    duplicate.push(original[0].clone());
    assert!(plan_import(duplicate, &original, &en, &ru).is_err());
    assert!(plan_import(original[..1].to_vec(), &original, &en, &ru).is_err());
    let mut changed = ru.clone();
    changed.insert("a".into(), "New draft".into());
    assert!(plan_import(original.clone(), &original, &en, &changed).is_err());
}
#[test]
fn right_click_keeps_batch_and_creates_only_the_two_context_actions() {
    let dir = tempfile::tempdir().unwrap();
    let mut editor = StringEditor::open(dir.path().into());
    let (en, ru) = maps(&rows());
    editor.en = en;
    editor.base = ru.clone();
    editor.draft = ru;
    editor.rows_selected.keys = BTreeSet::from(["a".into(), "b".into()]);
    let state = EditorState {
        reveal_selection: false,
        kind: CatalogKind::Npc,
        selected: 0,
        search: String::new(),
        search_focused: false,
        strings_open: true,
        xdt_open: false,
        missions_open: false,
        world_open: None,
        viewer_tabs: BTreeMap::new(),
        details_open: false,
        npc_inspector: NpcInspectorTab::Details,
        equipment_female: false,
        equipment_category: None,
        animation_page: 0,
        clip_index: 0,
        pose_mode: EditorPoseMode::Default,
        paused: true,
        looping: true,
        speed: 1.,
        turntable: false,
        playback_revision: 0,
    };
    let mut app = App::new();
    app.insert_resource(editor)
        .insert_resource(state)
        .init_resource::<ButtonInput<MouseButton>>()
        .insert_resource(EditorFonts {
            body: default(),
            display: default(),
            button: default(),
            button_hover: default(),
            panel: default(),
            textfield: default(),
        })
        .add_systems(Update, (context_input, draw).chain());
    let mut window = Window::default();
    window.set_cursor_position(Some(Vec2::new(200., 200.)));
    app.world_mut().spawn(window);
    app.world_mut().spawn((
        RowHit("a".into()),
        RelativeCursorPosition {
            cursor_over: true,
            normalized: Some(Vec2::ZERO),
        },
    ));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    assert_eq!(
        app.world()
            .resource::<StringEditor>()
            .rows_selected
            .keys
            .len(),
        2
    );
    assert!(
        app.world()
            .resource::<StringEditor>()
            .context_menu
            .is_some()
    );
    let world = app.world_mut();
    assert_eq!(
        world
            .query::<&Action>()
            .iter(world)
            .filter(|a| matches!(a, Action::ExportRows | Action::ImportRows))
            .count(),
        2
    );
    world.resource_mut::<ButtonInput<MouseButton>>().clear();
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(
        app.world()
            .resource::<StringEditor>()
            .context_menu
            .is_none()
    );
}
#[test]
fn shift_selects_one_hundred_across_virtual_pages_and_ctrl_toggles() {
    let keys: Vec<_> = (0..1000).map(|n| format!("row.{n:04}")).collect();
    let mut selection = RowSelection::default();
    selection.select(&keys, &keys[20], false, false);
    selection.select(&keys, &keys[119], false, true);
    assert_eq!(selection.keys.len(), 100);
    selection.select(&keys, &keys[70], true, false);
    assert_eq!(selection.keys.len(), 99);
    selection.select(&keys, &keys[400], true, false);
    assert_eq!(selection.keys.len(), 100);
}
