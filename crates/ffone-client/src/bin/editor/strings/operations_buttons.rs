use super::*;

pub(super) fn stamps(root: &Path) -> Option<[FileStamp; 2]> {
    let stamp = |locale| {
        let m = fs::metadata(root.join(format!("localization/{locale}.json"))).ok()?;
        Some((m.len(), m.modified().ok()))
    };
    Some([stamp("en")?, stamp("ru")?])
}

pub(super) fn placeholders(text: &str) -> BTreeSet<String> {
    text.split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}'))
        .map(|(key, _)| key.to_owned())
        .collect()
}

/// Merge only edited keys. Other writers' changes survive; divergent edits require resolution.
pub(super) fn merge(
    base: &BTreeMap<String, String>,
    draft: &BTreeMap<String, String>,
    disk: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut result = disk.clone();
    for (key, value) in draft {
        if base.get(key) == Some(value) {
            continue;
        }
        if disk.get(key) != base.get(key) && disk.get(key) != Some(value) {
            return Err(format!("Conflict: {key}"));
        }
        result.insert(key.clone(), value.clone());
    }
    Ok(result)
}

pub(super) fn section_header(
    state: Res<EditorState>,
    mut titles: Query<&mut LocalizedText, With<EditorSectionTitle>>,
    mut buttons: Query<(&EditorAction, &mut Node)>,
) {
    if !state.is_changed() {
        return;
    }
    for mut title in &mut titles {
        let next = if let Some(three_d) = state.world_open {
            LocalizedText::new(if three_d { "ui.editor.world.3d" } else { "ui.editor.world.2d" }, if three_d { "World 3D" } else { "World 2D" })
        } else if state.missions_open && state.xdt_open {
            LocalizedText::new("ui.editor.missions.tab", "Missions")
        } else if state.xdt_open {
            LocalizedText::new("ui.editor.xdt.tab", "XDT tables")
        } else if state.strings_open {
            LocalizedText::new("ui.editor.strings.tab", "Strings")
        } else {
            LocalizedText::new("ui.editor.title", "XDT CHARACTER LAB")
        };
        if *title != next {
            *title = next;
        }
    }
    for (action, mut node) in &mut buttons {
        if *action == EditorAction::ResetCamera {
            node.display = if state.strings_open || state.xdt_open || state.world_open.is_some() {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
}

pub(super) fn body_visibility(state: Res<EditorState>, mut bodies: Query<&mut Node, With<ModelEditorBody>>) {
    for mut node in &mut bodies {
        let display = if state.strings_open || state.xdt_open || state.world_open.is_some() {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
    }
}

pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: Action,
    key: &str,
    fallback: &str,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                border_radius: BorderRadius::all(px(4)),
                min_height: px(32),
                padding: UiRect::all(px(7)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.045, 0.19, 0.23)),
        ))
        .with_children(|p| {
            p.spawn(editor_text(fonts, key, fallback, 15., Color::WHITE, false));
        });
}

pub(super) fn value(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts, text: String, size: f32) {
    let mut bundle = editor_text(
        fonts,
        "ui.editor.strings.value",
        "{value}",
        size,
        Color::srgb(0.83, 0.94, 0.96),
        false,
    );
    bundle.localized = bundle.localized.with_arg("value", text);
    parent.spawn(bundle);
}

pub(super) fn buttons(
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    mut editor: ResMut<StringEditor>,
    cells: Query<(
        &StringCell,
        &RelativeCursorPosition,
        &ComputedNode,
        &bevy::text::ComputedTextBlock,
        &Text,
    )>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    localization: Res<Localization>,
    language: Res<Language>,
) {
    if editor.file_job.is_some() || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        editor.discard_confirm = matches!(action, Action::Discard) && !editor.discard_confirm;
        match action {
            Action::ExportRows | Action::ImportRows => {
                exchange::start_dialog(
                    &mut editor,
                    matches!(action, Action::ImportRows),
                    &localization,
                    &language,
                );
            }
            Action::SelectRow(key, toggle) => {
                let control =
                    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
                let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
                editor.select_row(key, control || *toggle, shift);
            }
            Action::Save => {
                if let Err(e) = editor.sync(true) {
                    editor.error(e);
                }
            }
            Action::Sync => match editor.sync(false) {
                Ok(()) => {
                    editor.status = if editor.dirty() {
                        LocalizedText::new("ui.editor.strings.dirty", "Unsaved changes")
                    } else {
                        LocalizedText::new("ui.editor.strings.ready", "Ready")
                    }
                }
                Err(e) => editor.error(e),
            },
            Action::ContextMode(mode) => {editor.context_mode=*mode;editor.filter_dirty=true;editor.offset=0.;editor.revision+=1;},
            Action::Discard => {
                if !editor.discard_confirm {
                    // Load first: an unreadable file must never destroy a draft.
                    match (
                        read_bundle(&editor.root, "en"),
                        read_bundle(&editor.root, "ru"),
                    ) {
                        (Ok(en), Ok(ru)) => {
                            editor.en_base = en.entries.clone();
                            editor.en = en.entries;
                            editor.base = ru.entries.clone();
                            editor.draft = ru.entries;
                            editor.undo.clear();
                            editor.redo.clear();
                            editor.changed_at = None;
                            editor.editing = false;
                            editor.filter_dirty = true;
                            editor.refresh_localization = true;
                            editor.status = LocalizedText::new("ui.editor.strings.ready", "Ready");
                        }
                        (Err(e), _) | (_, Err(e)) => editor.error(e),
                    }
                }
            }
            Action::Sort(column) => {
                editor.tools.sort = if editor.tools.sort.0 == *column {
                    (
                        *column,
                        if *column == 0 {
                            false
                        } else {
                            !editor.tools.sort.1
                        },
                    )
                } else {
                    (*column, false)
                };
                editor.filter_dirty = true;
                editor.offset = 0.;
            }
            Action::Replacement => {
                editor.tools.filter_focus = false;
                editor.search_focus = true;
                editor.tools.replacement_focus = true;
                editor.editing = false;
            }
            Action::Find(back) => editing_tools::find_next(&mut editor, *back),
            Action::ReplaceOne => editing_tools::replace_one(&mut editor),
            Action::ReplaceAll => editing_tools::replace_all(&mut editor),
            Action::CloseReplace => {
                editor.tools.replace_open = false;
                editor.tools.find_open = false;
                editor.tools.replacement_focus = false;
                editor.search_focus = false;
            }
            Action::Filter => {
                editor.tools.filter_focus = true;
                editor.search_focus = true;
                editor.editing = false;
            }
            Action::Search => {
                editor.tools.filter_focus = false;
                editor.tools.replacement_focus = false;
                editor.search_focus = true;
                editor.editing = false;
            }
            Action::FilterMode => {
                editor.tools.filter_mode = (editor.tools.filter_mode + 1) % 3;
                editor.filter_dirty = true;
                editor.offset = 0.;
                editor.tools.found = None;
            }
            Action::ReplaceCase => {
                editor.tools.case_sensitive = !editor.tools.case_sensitive;
                editor.tools.found = None;
            }
            Action::ReplaceWord => {
                editor.tools.whole_word = !editor.tools.whole_word;
                editor.tools.found = None;
            }
            Action::ReplaceLocale => {
                editor.tools.replace_english = !editor.tools.replace_english;
                editor.tools.found = None;
            }
            Action::Edit(key, russian) => {
                let ctrl =
                    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
                let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
                if ctrl || (shift && (!editor.editing || editor.selected != *key)) {
                    editor.select_row(key, ctrl, shift);
                    editor.revision += 1;
                    continue;
                }
                let source = if *russian {
                    &editor.draft[key]
                } else {
                    &editor.en[key]
                };
                let mut cursor = source.len();
                if let Some((_, relative, node, layout, _text)) = cells
                    .iter()
                    .find(|(cell, ..)| cell.0 == *key && cell.1 == *russian)
                {
                    if let Some(point) = relative
                        .normalized
                        .filter(|_| source.is_empty() || !layout.buffer().is_empty())
                    {
                        let point = (point + Vec2::splat(0.5)) * node.size();
                        cursor = parley::Cursor::from_point(layout.buffer(), point.x, point.y)
                            .index()
                            .min(source.len());
                        while !source.is_char_boundary(cursor) {
                            cursor -= 1;
                        }
                    }
                }
                let extend = editor.editing
                    && editor.selected == *key
                    && editor.russian == *russian
                    && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));
                editor.selected = key.clone();
                editor.russian = *russian;
                editor.editing = true;
                editor.search_focus = false;
                editor.cursor = cursor;
                if !extend {
                    editor.anchor = cursor;
                }
            }
        }
        editor.revision += 1;
    }
}

pub(super) fn keyboard(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<EditorState>,
    mut editor: ResMut<StringEditor>,
) {
    if !state.strings_open || editor.file_job.is_some() {
        events.clear();
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for event in events.read().filter(|e| e.state == ButtonState::Pressed) {
        // Document history belongs to the entire localization window, including
        // search fields and row menus after the edited cell has lost focus.
        if ctrl && matches!(event.key_code, KeyCode::KeyZ | KeyCode::KeyY) {
            editor.context_menu = None;
            editor.apply_history(event.key_code == KeyCode::KeyZ && !shift);
            continue;
        }
        if editor.context_menu.is_some() {
            if event.logical_key == Key::Escape {
                editor.context_menu = None;
                editor.revision += 1;
            }
            continue;
        }
        if !editor.search_focus && !editor.editing && ctrl && event.key_code == KeyCode::KeyA {
            editor.rows_selected.keys = editor.filtered.iter().cloned().collect();
            editor.revision += 1;
            continue;
        }
        if !editor.search_focus && !editor.editing && event.logical_key == Key::Escape {
            editor.rows_selected = default();
            editor.revision += 1;
            continue;
        }
        if ctrl && event.key_code == KeyCode::KeyS {
            if let Err(e) = editor.sync(true) {
                editor.error(e);
            }
            continue;
        }
        if ctrl && matches!(event.key_code, KeyCode::KeyF | KeyCode::KeyH) {
            editor.tools.filter_focus = false;
            editor.tools.find_open = true;
            if event.key_code == KeyCode::KeyH {
                editor.tools.replace_english = !editor.russian;
            }
            editor.search_focus = true;
            editor.editing = false;
            editor.tools.replacement_focus = false;
            editor.tools.replace_open = event.key_code == KeyCode::KeyH;
            editor.tools.search_cursor = editor.search.len();
            editor.tools.search_anchor = 0;
        } else if event.key_code == KeyCode::F3 {
            editing_tools::find_next(&mut editor, shift);
        } else if editor.search_focus {
            editing_tools::field_key(&mut editor, event, ctrl, shift);
        } else if event.logical_key == Key::Tab && editor.editing {
            editing_tools::tab_row(&mut editor, shift);
        } else if ctrl && event.logical_key == Key::Enter && editor.editing {
            editing_tools::next_row(&mut editor);
        } else if editor.editing {
            let Some(text) = editor.active().get(&editor.selected).cloned() else {
                continue;
            };
            if matches!(
                event.logical_key,
                Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End
            ) {
                editing_tools::move_cell(&mut editor, &event.logical_key, ctrl, shift);
                editor.revision += 1;
                continue;
            }
            if ctrl {
                match event.key_code {
                    KeyCode::KeyA => {
                        editor.anchor = 0;
                        editor.cursor = text.len();
                    }
                    KeyCode::KeyC | KeyCode::KeyX => {
                        let selected = &text
                            [editor.cursor.min(editor.anchor)..editor.cursor.max(editor.anchor)];
                        match arboard::Clipboard::new().and_then(|mut c| c.set_text(selected)) {
                            Ok(()) if event.key_code == KeyCode::KeyX => {
                                editor.replace_selection("")
                            }
                            Err(e) => editor.error(e.to_string()),
                            _ => {}
                        }
                    }
                    KeyCode::KeyV => match arboard::Clipboard::new().and_then(|mut c| c.get_text())
                    {
                        Ok(text) => editor.replace_selection(&text.replace("\r\n", "\n")),
                        Err(e) => editor.error(e.to_string()),
                    },

                    _ => {}
                }
            } else {
                match &event.logical_key {
                    Key::Escape => {
                        editor.editing = false;
                        editor.filter_dirty = true;
                    }
                    Key::Backspace => {
                        if editor.cursor == editor.anchor {
                            editor.anchor = text[..editor.cursor]
                                .char_indices()
                                .next_back()
                                .map_or(0, |(i, _)| i);
                        }
                        editor.replace_selection("");
                    }
                    Key::Delete => {
                        if editor.cursor == editor.anchor {
                            editor.anchor = text[editor.cursor..]
                                .chars()
                                .next()
                                .map_or(text.len(), |c| editor.cursor + c.len_utf8());
                        }
                        editor.replace_selection("");
                    }
                    Key::Enter => editor.replace_selection("\n"),
                    _ => {
                        if let Some(text) = &event.text {
                            let text: String = text.chars().filter(|c| !c.is_control()).collect();
                            if !text.is_empty() {
                                editor.replace_selection(&text);
                            }
                        }
                    }
                }
            }
        }
        editor.revision += 1;
    }
}

pub(super) fn persistence(
    mut state: ResMut<EditorState>,
    mut editor: ResMut<StringEditor>,
    mut close: MessageReader<bevy::window::WindowCloseRequested>,
    mut exit: MessageWriter<AppExit>,
    mut localization: ResMut<Localization>,
    language: Res<Language>,
    mut xdt: ResMut<super::super::xdt::XdtEditor>,
) {
    let closing = close.read().next().is_some();
    if closing && !xdt.save_on_close() {
        state.xdt_open = true;
        state.strings_open = false;
        return;
    }
    let due = editor
        .changed_at
        .is_some_and(|t| t.elapsed() > Duration::from_secs(1));
    if (closing && editor.dirty()) || due {
        match editor.sync(true) {
            Ok(()) => {}
            Err(e) => {
                editor.changed_at = None;
                editor.error(e);
                if closing {
                    state.strings_open = true;
                    state.xdt_open = false;
                    return;
                }
            }
        }
    }
    if closing {
        exit.write(AppExit::Success);
    }
    if editor.refresh_localization {
        if let Ok((updated, _)) = Localization::open(&editor.root, &language.requested) {
            *localization = updated;
        }
        editor.refresh_localization = false;
    }
    if state.strings_open && editor.last_poll.elapsed() > Duration::from_secs(2) {
        editor.last_poll = Instant::now();
        // A clean focused field can refresh; a draft is merged by the next save.
        if stamps(&editor.root) != editor.file_stamps && (!editor.editing || !editor.dirty()) {
            if let Err(e) = editor.sync(false) {
                editor.error(e);
            }
        }
    }
}

pub(super) fn measure_rows(
    mut editor: ResMut<StringEditor>,
    cells: Query<(&CellMetric, Ref<bevy::text::TextLayoutInfo>)>,
) {
    if !cells.iter().any(|(_, layout)| layout.is_changed()) {
        return;
    }
    let mut heights = BTreeMap::<&str, f32>::new();
    for (cell, layout) in &cells {
        if cell.revision != editor.revision || layout.scale_factor <= 0. {
            continue;
        }
        let height = (layout.size.y / layout.scale_factor + 24.).ceil().max(54.);
        heights
            .entry(&cell.key)
            .and_modify(|v| *v = v.max(height))
            .or_insert(height);
    }
    let mut changed = false;
    for (key, height) in heights {
        let Some(index) = editor.row_indices.get(key).copied() else {
            continue;
        };
        let old_end = editor.row_starts[index + 1];
        let previous = old_end - editor.row_starts[index];
        let delta = height - previous;
        if delta.abs() < 1. {
            continue;
        }
        if old_end <= editor.offset {
            editor.offset += delta;
        }
        for start in &mut editor.row_starts[index + 1..] {
            *start += delta;
        }
        changed = true;
    }
    if changed {
        editor.revision += 1;
    }
}

pub(super) fn estimated_height(en: &str, ru: &str, width: f32) -> f32 {
    // Conservative wrapping leaves room for Cyrillic and long unbroken words.
    let columns = ((width * 0.5 - 40.) / 11.).max(8.) as usize;
    let lines = |s: &str| {
        s.split('\n')
            .map(|line| line.chars().count().max(1).div_ceil(columns))
            .sum::<usize>()
    };
    (lines(en).max(lines(ru)) as f32 * 23. + 24.).max(54.)
}

pub(super) fn visible_range(starts: &[f32], offset: f32, height: f32) -> std::ops::Range<usize> {
    let count = starts.len().saturating_sub(1);
    let first = starts
        .partition_point(|y| *y <= offset)
        .saturating_sub(2)
        .min(count);
    let end = (starts.partition_point(|y| *y < offset + height) + 1).min(count);
    first..end
}

pub(super) fn scroll_table(
    mut wheels: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut editor: ResMut<StringEditor>,
    viewport: Query<(&ComputedNode, &RelativeCursorPosition), With<TableViewport>>,
    track: Query<&RelativeCursorPosition, With<TableTrack>>,
) {
    let amount: f32 = wheels
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line => e.y * 60.,
            MouseScrollUnit::Pixel => e.y,
        })
        .sum();
    let Ok((computed, cursor)) = viewport.single() else {
        return;
    };
    let size = computed.size() * computed.inverse_scale_factor();
    if size.min_element() <= 0. {
        return;
    }
    if editor.viewport != size {
        editor.metrics_dirty |= (editor.viewport.x - size.x).abs() > 1.;
        editor.viewport = size;
        editor.revision += 1;
    }
    let max = (editor.row_starts.last().copied().unwrap_or(0.) - size.y).max(0.);
    let mut offset = editor.offset;
    if cursor.cursor_over() {
        offset -= amount;
    }
    if let Ok(track) = track.single() {
        if mouse.pressed(MouseButton::Left) && track.cursor_over() {
            if let Some(position) = track.normalized {
                offset = (position.y + 0.5).clamp(0., 1.) * max;
            }
        }
    }
    offset = offset.clamp(0., max);
    if offset != editor.offset {
        editor.context_menu = None;
        editor.offset = offset;
        editor.revision += 1;
    }
}
