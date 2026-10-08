use super::*;
fn editor() -> StringEditor {
    let mut e = StringEditor::open(PathBuf::from("target/nonexistent-string-test"));
    e.en = BTreeMap::from([
        ("a".into(), "Zebra {name}".into()),
        ("b".into(), "Apple".into()),
        ("c".into(), "Berry".into()),
    ]);
    e.draft = BTreeMap::from([
        ("a".into(), "Привет {name}".into()),
        ("b".into(), "Привет мир".into()),
        ("c".into(), "Мир".into()),
    ]);
    e.base = e.draft.clone();
    e.filtered = e.keys();
    e
}
fn key(logical_key: Key, key_code: KeyCode, text: Option<&str>) -> KeyboardInput {
    KeyboardInput {
        logical_key,
        key_code,
        text: text.map(Into::into),
        state: ButtonState::Pressed,
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}
#[test]
fn shortcuts_open_fields_and_advance_editing_with_a_one_pixel_caret() {
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
    app.insert_resource(editor())
        .insert_resource(state)
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(EditorFonts {
            body: default(),
            display: default(),
            button: default(),
            button_hover: default(),
            panel: default(),
            textfield: default(),
        })
        .add_message::<KeyboardInput>()
        .add_systems(Update, (keyboard, draw).chain());
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ControlLeft);
    app.world_mut()
        .write_message(key(Key::Character("h".into()), KeyCode::KeyH, None));
    app.update();
    assert!(app.world().resource::<StringEditor>().tools.replace_open);
    assert!(app.world().resource::<StringEditor>().search_focus);
    let world = app.world_mut();
    assert_eq!(
        world
            .query::<&Action>()
            .iter(world)
            .filter(|a| matches!(a, Action::ReplaceAll))
            .count(),
        1
    );
    assert!(
        world
            .query_filtered::<&Node, With<Caret>>()
            .iter(world)
            .all(|node| node.width == px(1))
    );
    world.write_message(key(Key::Character("f".into()), KeyCode::KeyF, None));
    app.update();
    assert!(!app.world().resource::<StringEditor>().tools.replace_open);
    {
        let mut e = app.world_mut().resource_mut::<StringEditor>();
        e.search_focus = false;
        e.editing = true;
        e.selected = "a".into();
        e.cursor = e.draft["a"].len();
        e.anchor = e.cursor;
    }
    app.world_mut()
        .write_message(key(Key::Enter, KeyCode::Enter, None));
    app.update();
    let e = app.world().resource::<StringEditor>();
    assert_eq!(e.selected, "b");
    assert!(e.editing);
    assert_eq!(e.draft["a"], "Привет {name}");

    // Edits in different rows remain undoable while focus is in search.
    {
        let mut e = app.world_mut().resource_mut::<StringEditor>();
        e.selected = "b".into();
        e.cursor = e.draft["b"].len();
        e.anchor = 0;
        e.replace_selection("Правка");
        e.selected = "c".into();
        e.cursor = e.draft["c"].len();
        e.anchor = 0;
        e.replace_selection("Другая правка");
        e.editing = false;
        e.search_focus = true;
        e.search = "unchanged search".into();
    }
    for expected in ["c", "b"] {
        app.world_mut().write_message(key(Key::Character("z".into()), KeyCode::KeyZ, None));
        app.update();
        let e = app.world().resource::<StringEditor>();
        assert_eq!(e.selected, expected);
        assert_eq!(e.draft[expected], e.base[expected]);
        assert_eq!(e.search, "unchanged search");
        assert!(!e.editing);
    }
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft);
    app.world_mut().write_message(key(Key::Character("z".into()), KeyCode::KeyZ, None));
    app.update();
    assert_eq!(app.world().resource::<StringEditor>().draft["b"], "Правка");
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ShiftLeft);
    app.world_mut().write_message(key(Key::Character("y".into()), KeyCode::KeyY, None));
    app.update();
    assert_eq!(app.world().resource::<StringEditor>().draft["c"], "Другая правка");
}
#[test]
fn document_history_keeps_more_than_one_hundred_edits() {
    let mut e = editor();
    e.selected = "b".into();
    let original = e.draft["b"].clone();
    for i in 0..150 {
        e.anchor = 0;
        e.cursor = e.draft["b"].len();
        e.replace_selection(&format!("Изменение {i}"));
    }
    for _ in 0..150 { e.apply_history(true); }
    assert_eq!(e.draft["b"], original);
    for _ in 0..150 { e.apply_history(false); }
    assert_eq!(e.draft["b"], "Изменение 149");
    e.apply_history(true);
    e.replace_selection("новое действие");
    assert!(e.redo.is_empty());
}
#[test]
fn search_navigation_selection_and_unicode_insertion() {
    let mut e = editor();
    e.search = "Привет мир".into();
    e.tools.search_cursor = e.search.len();
    e.tools.search_anchor = e.search.len();
    field_key(
        &mut e,
        &key(Key::ArrowLeft, KeyCode::ArrowLeft, None),
        true,
        false,
    );
    assert_eq!(e.tools.search_cursor, "Привет ".len());
    field_key(&mut e, &key(Key::Home, KeyCode::Home, None), false, true);
    field_key(
        &mut e,
        &key(Key::Character("Я ".into()), KeyCode::KeyZ, Some("Я ")),
        false,
        false,
    );
    assert_eq!(e.search, "Я мир");
    field_key(
        &mut e,
        &key(Key::Character("a".into()), KeyCode::KeyA, None),
        true,
        false,
    );
    assert_eq!(e.search, "Я мир"); // Ctrl+A selects, never deletes.
    assert_eq!(
        (e.tools.search_anchor, e.tools.search_cursor),
        (0, e.search.len())
    );
    field_key(
        &mut e,
        &key(Key::Delete, KeyCode::Delete, None),
        false,
        false,
    );
    assert!(e.search.is_empty());
}
#[test]
fn replacement_field_is_independent_of_search() {
    let mut e = editor();
    e.search = "Привет".into();
    e.tools.replace_open = true;
    e.tools.replacement_focus = true;
    field_key(
        &mut e,
        &key(
            Key::Character("Здравствуйте".into()),
            KeyCode::KeyP,
            Some("Здравствуйте"),
        ),
        false,
        false,
    );
    assert_eq!(e.tools.replacement, "Здравствуйте");
    assert_eq!(e.search, "Привет");
    field_key(&mut e, &key(Key::Tab, KeyCode::Tab, None), false, false);
    assert!(!e.tools.replacement_focus);
}
#[test]
fn replace_batch_preserves_english_and_undoes_together() {
    let mut e = editor();
    let en = e.en.clone();
    let original = e.draft.clone();
    e.search = "Привет".into();
    e.tools.replacement = "Здравствуйте".into();
    replace_all(&mut e);
    assert_eq!(e.draft["a"], "Здравствуйте {name}");
    assert_eq!(e.draft["b"], "Здравствуйте мир");
    assert_eq!(e.draft["c"], "Мир");
    assert_eq!(e.en, en);
    assert_eq!(e.undo.len(), 1);
    e.apply_history(true);
    assert_eq!(e.draft, original);
    e.apply_history(false);
    assert_eq!(e.draft["a"], "Здравствуйте {name}");
    let before = e.draft.clone();
    e.search = "{name}".into();
    e.tools.replacement.clear();
    replace_all(&mut e);
    assert_eq!(e.draft, before);
}
#[test]
fn sorting_and_next_row_follow_display_order() {
    let mut e = editor();
    e.tools.sort = (1, false);
    sort_rows(&mut e);
    assert_eq!(e.filtered, vec!["b", "c", "a"]);
    e.selected = "c".into();
    e.row_starts = vec![0., 54., 108., 162.];
    next_row(&mut e);
    assert_eq!(e.selected, "a");
    assert_eq!(e.offset, 108.);
    next_row(&mut e);
    assert_eq!(e.selected, "a");
    e.tools.sort = (1, true);
    sort_rows(&mut e);
    assert_eq!(e.filtered, vec!["a", "c", "b"]);
}
#[test]
fn shift_range_uses_sorted_order() {
    let mut e = editor();
    e.tools.sort = (1, false);
    sort_rows(&mut e);
    e.select_row("c", false, false);
    e.select_row("a", false, true);
    assert_eq!(
        e.rows_selected.keys,
        BTreeSet::from(["a".into(), "c".into()])
    );
}
