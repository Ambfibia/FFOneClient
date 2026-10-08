use super::*;
use bevy::text::ComputedTextBlock;
use parley::{Affinity, Cursor as TextCursor, Selection as TextSelection};

#[derive(Default)]
pub(super) struct Tools {
    pub filter_mode: u8,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub replace_english: bool,
    pub filter: String,
    pub filter_cursor: usize,
    pub filter_anchor: usize,
    pub filter_focus: bool,
    pub find_open: bool,
    pub pending_find: bool,
    pub found: Option<(String, bool, usize, usize)>,
    pub search_cursor: usize,
    pub search_anchor: usize,
    pub replacement: String,
    pub replacement_cursor: usize,
    pub replacement_anchor: usize,
    pub replacement_focus: bool,
    pub replace_open: bool,
    pub sort: (u8, bool),
}
#[derive(Component)]
pub(super) struct Field(u8);
#[derive(Component)]
pub(super) struct Caret(usize);
#[derive(Component)]
pub(super) struct Selection(usize, usize);

pub(in super::super) fn caret(p: &mut ChildSpawnerCommands, index: usize) {
    p.spawn((
        Caret(index),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: px(1),
            height: px(18),
            ..default()
        },
        BackgroundColor(Color::srgb(0.8, 1., 1.)),
    ));
}
pub(in super::super) fn highlight(p: &mut ChildSpawnerCommands, start: usize, end: usize) {
    p.spawn((CellSelection(start, end), Node {
        position_type: PositionType::Absolute, ..default()
    }));
}
pub(super) fn carets(
    mut nodes: Query<(&Caret, &ChildOf, &mut Node, &mut Visibility)>,
    texts: Query<(&Text, &bevy::text::TextLayoutInfo, &ComputedTextBlock)>,
) {
    for (caret, parent, mut node, mut visibility) in &mut nodes {
        let Ok((text, layout, block)) = texts.get(parent.parent()) else {
            continue;
        };
        if !text.0.is_empty() && block.buffer().is_empty() {
            continue;
        }
        if *visibility != Visibility::Inherited {
            *visibility = Visibility::Inherited;
        }
        let bounds = TextCursor::from_byte_index(
            block.buffer(),
            caret.0.min(text.0.len()),
            Affinity::Downstream,
        )
        .geometry(block.buffer(), 1.0);
        let point = Vec2::new(bounds.x0 as f32, bounds.y0 as f32);
        let scale = layout.scale_factor.max(0.01);
        let left = px(point.x / scale);
        let top = px(point.y / scale);
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
    }
}
pub(super) fn field(
    p: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    editor: &StringEditor,
    mode: u8,
) {
    let replacement = mode == 1;
    let active = editor.search_focus
        && if mode == 2 {
            editor.tools.filter_focus
        } else {
            !editor.tools.filter_focus && editor.tools.replacement_focus == replacement
        };
    let (text, cursor, anchor) = if mode == 2 {
        (
            &editor.tools.filter,
            editor.tools.filter_cursor,
            editor.tools.filter_anchor,
        )
    } else if replacement {
        (
            &editor.tools.replacement,
            editor.tools.replacement_cursor,
            editor.tools.replacement_anchor,
        )
    } else {
        (
            &editor.search,
            editor.tools.search_cursor,
            editor.tools.search_anchor,
        )
    };
    p.spawn((
        Button,
        if mode == 2 {
            Action::Filter
        } else if replacement {
            Action::Replacement
        } else {
            Action::Search
        },
        Node {
            flex_grow: 1.,
            min_width: px(0),
            height: px(38),
            padding: UiRect::all(px(9)),
            overflow: Overflow::scroll_x(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.09, 0.12)),
    ))
    .with_children(|p| {
        let mut bundle = editor_text(
            fonts,
            "ui.editor.strings.value",
            "{value}",
            16.,
            Color::srgb(0.83, 0.94, 0.96),
            false,
        );
        bundle.localized = bundle.localized.with_arg("value", text.clone());
        if text.is_empty() && !active {
            bundle.localized = if mode == 2 {
                LocalizedText::new("ui.editor.strings.filter", "Filter rows…")
            } else if replacement {
                LocalizedText::new("ui.editor.strings.replacement", "Replace with…")
            } else {
                LocalizedText::new("ui.editor.strings.search_simple", "Search text…")
            };
        }
        let mut entity = p.spawn((
            bundle,
            Field(mode),
            RelativeCursorPosition::default(),
            Node {
                min_width: px(20),
                min_height: px(20),
                flex_shrink: 0.,
                ..default()
            },
        ));
        entity.insert(TextLayout::default().with_linebreak(bevy::text::LineBreak::NoWrap));
        if active {
            entity.with_children(|p| caret(p, cursor));
        }
        if active && cursor != anchor {
            entity.with_children(|p| {
                p.spawn((
                    Selection(cursor.min(anchor), cursor.max(anchor)),
                    Node {
                        position_type: PositionType::Absolute,
                        height: px(20),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.65, 0.8, 0.3)),
                ));
            });
        }
    });
}
fn hit(text: &str, block: &ComputedTextBlock, point: Vec2) -> usize {
    TextCursor::from_point(block.buffer(), point.x, point.y)
        .index()
        .min(text.len())
}
pub(super) fn field_click(
    windows: Query<&Window>,
    mut pointer: Local<Option<Vec2>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    actions: Query<(&Interaction, &Action), Changed<Interaction>>,
    fields: Query<(
        &Field,
        &RelativeCursorPosition,
        &ComputedNode,
        &ComputedTextBlock,
        &Text,
    )>,
    mut editor: ResMut<StringEditor>,
) {
    let point = windows.iter().find_map(Window::cursor_position);
    let moved = pointer_moved(&mut pointer, point);
    if !mouse.just_pressed(MouseButton::Left) && !moved {
        return;
    }
    if !mouse.pressed(MouseButton::Left) || editor.file_job.is_some() {
        return;
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for (interaction, action) in &actions {
        if !mouse.just_pressed(MouseButton::Left) {
            continue;
        }
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            Action::Filter => {
                editor.tools.filter_cursor = editor.tools.filter.len();
                if !shift {
                    editor.tools.filter_anchor = editor.tools.filter_cursor;
                }
            }
            Action::Search => {
                editor.tools.search_cursor = editor.search.len();
                if !shift {
                    editor.tools.search_anchor = editor.tools.search_cursor;
                }
            }
            Action::Replacement => {
                editor.tools.replacement_cursor = editor.tools.replacement.len();
                if !shift {
                    editor.tools.replacement_anchor = editor.tools.replacement_cursor;
                }
            }
            _ => {}
        }
    }
    for (field, relative, node, layout, text) in &fields {
        if !relative.cursor_over()
            || (!mouse.just_pressed(MouseButton::Left)
                && (!editor.search_focus
                    || if field.0 == 2 {
                        !editor.tools.filter_focus
                    } else {
                        editor.tools.filter_focus
                            || editor.tools.replacement_focus != (field.0 == 1)
                    }))
        {
            continue;
        }
        if !text.0.is_empty() && layout.buffer().is_empty() {
            continue;
        }
        let index = hit(
            &text.0,
            layout,
            (relative.normalized.unwrap_or_default() + Vec2::splat(0.5)) * node.size(),
        );
        let shift = !mouse.just_pressed(MouseButton::Left)
            || keys.pressed(KeyCode::ShiftLeft)
            || keys.pressed(KeyCode::ShiftRight);
        let previous = if field.0 == 2 {
            editor.tools.filter_cursor
        } else if field.0 == 1 {
            editor.tools.replacement_cursor
        } else {
            editor.tools.search_cursor
        };
        if !mouse.just_pressed(MouseButton::Left) && index == previous {
            continue;
        }
        if field.0 == 2 {
            editor.tools.filter_cursor = index.min(editor.tools.filter.len());
            if !shift {
                editor.tools.filter_anchor = editor.tools.filter_cursor;
            }
        } else if field.0 == 1 {
            editor.tools.replacement_cursor = index.min(editor.tools.replacement.len());
            if !shift {
                editor.tools.replacement_anchor = editor.tools.replacement_cursor;
            }
        } else {
            editor.tools.search_cursor = index.min(editor.search.len());
            if !shift {
                editor.tools.search_anchor = editor.tools.search_cursor;
            }
        }
        editor.revision += 1;
    }
}
fn step(text: &str, cursor: usize, right: bool, word: bool) -> usize {
    if right {
        let mut at = cursor;
        let mut seen = false;
        for c in text[cursor..].chars() {
            if word && seen && !c.is_whitespace() {
                break;
            }
            at += c.len_utf8();
            if !word {
                break;
            }
            if c.is_whitespace() {
                seen = true;
            }
        }
        at
    } else {
        let mut at = cursor;
        let mut seen = false;
        for (i, c) in text[..cursor].char_indices().rev() {
            if word && seen && c.is_whitespace() {
                break;
            }
            at = i;
            if !word {
                break;
            }
            if !c.is_whitespace() {
                seen = true;
            }
        }
        at
    }
}
fn insert(text: &mut String, cursor: &mut usize, anchor: &mut usize, value: &str) {
    let start = (*cursor).min(*anchor);
    text.replace_range(start..(*cursor).max(*anchor), value);
    *cursor = start + value.len();
    *anchor = *cursor;
}
pub(super) fn field_key(editor: &mut StringEditor, event: &KeyboardInput, ctrl: bool, shift: bool) {
    if matches!(event.logical_key, Key::Escape | Key::Enter) {
        if event.logical_key == Key::Escape {
            editor.search_focus = false;
        } else if editor.tools.filter_focus {
            editor.search_focus = false;
        } else {
            find_next(editor, shift);
        }
        return;
    }
    if event.logical_key == Key::Tab {
        if editor.tools.replace_open && !editor.tools.filter_focus {
            editor.tools.replacement_focus = !editor.tools.replacement_focus;
        } else {
            editor.search_focus = false;
        }
        return;
    }
    let (text, cursor, anchor) = if editor.tools.filter_focus {
        (
            &mut editor.tools.filter,
            &mut editor.tools.filter_cursor,
            &mut editor.tools.filter_anchor,
        )
    } else if editor.tools.replacement_focus {
        (
            &mut editor.tools.replacement,
            &mut editor.tools.replacement_cursor,
            &mut editor.tools.replacement_anchor,
        )
    } else {
        (
            &mut editor.search,
            &mut editor.tools.search_cursor,
            &mut editor.tools.search_anchor,
        )
    };
    let before = text.clone();
    match &event.logical_key {
        Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End => {
            *cursor = match event.logical_key {
                Key::Home => 0,
                Key::End => text.len(),
                Key::ArrowLeft if !shift && !ctrl && cursor != anchor => (*cursor).min(*anchor),
                Key::ArrowRight if !shift && !ctrl && cursor != anchor => (*cursor).max(*anchor),
                _ => step(text, *cursor, event.logical_key == Key::ArrowRight, ctrl),
            };
            if !shift {
                *anchor = *cursor;
            }
        }
        Key::Backspace | Key::Delete => {
            if cursor == anchor {
                *anchor = step(text, *cursor, event.logical_key == Key::Delete, ctrl);
            }
            insert(text, cursor, anchor, "");
        }
        _ if ctrl && event.key_code == KeyCode::KeyA => {
            *anchor = 0;
            *cursor = text.len();
        }
        _ if ctrl && matches!(event.key_code, KeyCode::KeyC | KeyCode::KeyX) => {
            if cursor != anchor {
                if arboard::Clipboard::new()
                    .and_then(|mut c| {
                        c.set_text(&text[(*cursor).min(*anchor)..(*cursor).max(*anchor)])
                    })
                    .is_ok()
                    && event.key_code == KeyCode::KeyX
                {
                    insert(text, cursor, anchor, "");
                }
            }
        }
        _ if ctrl && event.key_code == KeyCode::KeyV => {
            if let Ok(value) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                insert(
                    text,
                    cursor,
                    anchor,
                    &value
                        .chars()
                        .filter(|c| !c.is_control())
                        .collect::<String>(),
                );
            }
        }
        _ if !ctrl => {
            if let Some(value) = &event.text {
                insert(
                    text,
                    cursor,
                    anchor,
                    &value
                        .chars()
                        .filter(|c| !c.is_control())
                        .collect::<String>(),
                );
            }
        }
        _ => {}
    }
    if *text != before && editor.tools.filter_focus {
        editor.filter_dirty = true;
        editor.offset = 0.;
        editor.tools.found = None;
    } else if *text != before && !editor.tools.replacement_focus {
        editor.tools.found = None;
    }
}
pub(super) fn sort_rows(editor: &mut StringEditor) {
    let (column, descending) = editor.tools.sort;
    if column == 0 {
        if descending {
            editor.filtered.reverse();
        }
        return;
    }
    let source = if column == 1 {
        &editor.en
    } else {
        &editor.draft
    };
    editor
        .filtered
        .sort_by_cached_key(|key| (source[key].to_lowercase(), key.clone()));
    if descending {
        editor.filtered.reverse();
    }
}
pub(super) fn next_row(editor: &mut StringEditor) {
    if let Some(index) = editor
        .filtered
        .iter()
        .position(|key| key == &editor.selected)
    {
        if let Some(key) = editor.filtered.get(index + 1).cloned() {
            editor.selected = key;
            editor.cursor = editor.active()[&editor.selected].len();
            editor.anchor = editor.cursor;
            if let Some(top) = editor.row_starts.get(index + 1) {
                editor.offset = *top;
            }
        }
    }
}
pub(super) fn replace_all(editor: &mut StringEditor) {
    if editor.search.is_empty() {
        return;
    }
    let russian = !editor.tools.replace_english;
    let source = if russian { &editor.draft } else { &editor.en };
    let counterpart = if russian { &editor.en } else { &editor.draft };
    let mut plan = Vec::new();
    // Snapshot the current filter before any mutation; hidden rows never enter this batch.
    for key in editor.keys() {
        let text = &source[&key];
        let mut changed = text.clone();
        for (start, end) in replacement_matches(text, &editor.search, &editor.tools)
            .into_iter()
            .rev()
        {
            changed.replace_range(start..end, &editor.tools.replacement);
        }
        if changed == *text {
            continue;
        }
        if placeholders(&changed) != placeholders(&counterpart[&key]) {
            editor.status = LocalizedText::new(
                "ui.editor.strings.exchange_placeholder",
                "Template parameters do not match: {key}",
            )
            .with_arg("key", key);
            return;
        }
        plan.push((key, changed));
    }
    let count = plan.len();
    exchange::apply_batch(editor, russian, plan);
    editor.search_focus = false;
    editor.status = LocalizedText::new("ui.editor.strings.replaced", "Replaced in {count} rows")
        .with_arg("count", count.to_string());
}
fn preferences_path(root: &Path) -> PathBuf {
    root.join("../../target/editor/preferences.json")
}
pub(crate) fn restore_tab(root: &Path, state: &mut EditorState, catalog: &EditorCatalog) {
    let Some(tab) = read_tab(root) else { return };
    state.strings_open = tab == "strings";
    state.xdt_open = tab == "xdt";
    state.missions_open = tab == "missions";
    state.world_open = match tab.as_str() { "world2d" => Some(false), "world3d" => Some(true), _ => None };
    let kind = match tab.as_str() {
        "nano" => CatalogKind::Nano,
        "equipment" => CatalogKind::Equipment,
        _ => CatalogKind::Npc,
    };
    state.kind = kind;
    if let Some(index) = catalog.entries.iter().position(|entry| entry.kind == kind) {
        state.selected = index;
        state.choose_default_clip(catalog);
    }
}
pub(super) fn remember_tab(
    state: Res<EditorState>,
    editor: Res<StringEditor>,
    mut last: Local<String>,
) {
    let tab = if let Some(three_d) = state.world_open {
        if three_d { "world3d" } else { "world2d" }
    } else if state.missions_open && state.xdt_open {
        "missions"
    } else if state.xdt_open {
        "xdt"
    } else if state.strings_open {
        "strings"
    } else {
        match state.kind {
            CatalogKind::Npc => "npc",
            CatalogKind::Nano => "nano",
            CatalogKind::Equipment => "equipment",
        }
    };
    if *last == tab {
        return;
    }
    let result = write_tab(&editor.root, tab);
    if let Err(error) = result {
        warn!("Cannot save editor tab: {error}");
    }
    *last = tab.to_owned();
}

pub(super) fn field_layout(
    carets: Query<(&Caret, &ChildOf)>,
    texts: Query<(&ChildOf, &bevy::text::TextLayoutInfo, &ComputedTextBlock), With<Field>>,
    mut scrolls: Query<(&ComputedNode, &mut ScrollPosition)>,
    mut selections: Query<(&Selection, &ChildOf, &mut Node)>,
) {
    let edge = |layout: &bevy::text::TextLayoutInfo, block: &ComputedTextBlock, index: usize| {
        TextCursor::from_byte_index(block.buffer(), index, Affinity::Downstream)
            .geometry(block.buffer(), 1.0)
            .x0 as f32
            / layout.scale_factor.max(0.01)
    };
    for (caret, parent) in &carets {
        let Ok((container, layout, block)) = texts.get(parent.parent()) else {
            continue;
        };
        let Ok((node, mut scroll)) = scrolls.get_mut(container.parent()) else {
            continue;
        };
        let x = edge(layout, block, caret.0);
        let width = node.size().x * node.inverse_scale_factor() - 20.;
        let next = if x < scroll.0.x {
            x
        } else if x > scroll.0.x + width {
            (x - width).max(0.)
        } else {
            scroll.0.x
        };
        if scroll.0.x != next {
            scroll.0.x = next;
        }
    }
    for (selection, parent, mut node) in &mut selections {
        let Ok((_, layout, block)) = texts.get(parent.parent()) else {
            continue;
        };
        let left = px(edge(layout, block, selection.0));
        let width =
            px((edge(layout, block, selection.1) - edge(layout, block, selection.0)).max(0.));
        if node.left != left {
            node.left = left;
        }
        if node.width != width {
            node.width = width;
        }
    }
}

#[cfg(test)]
#[path = "strings_tools/tests.rs"]
mod tests;

fn read_tab(root: &Path) -> Option<String> {
    let value: Value = serde_json::from_slice(&fs::read(preferences_path(root)).ok()?).ok()?;
    let tab = value["tab"].as_str()?;
    matches!(tab, "xdt" | "strings" | "npc" | "nano" | "equipment").then(|| tab.to_owned())
}
fn write_tab(root: &Path, tab: &str) -> std::io::Result<()> {
    let path = preferences_path(root);
    fs::create_dir_all(path.parent().unwrap())?;
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    temp.write_all(serde_json::json!({"tab":tab}).to_string().as_bytes())?;
    temp.persist(&path).map_err(|e| e.error)?;
    Ok(())
}
#[cfg(test)]
#[path = "strings_tools/preference_tests.rs"]
mod preference_tests;

pub(super) fn tab_row(editor: &mut StringEditor, back: bool) {
    let Some(index) = editor.filtered.iter().position(|k| k == &editor.selected) else {
        return;
    };
    let next = if back {
        index.saturating_sub(1)
    } else {
        (index + 1).min(editor.filtered.len().saturating_sub(1))
    };
    editor.selected = editor.filtered[next].clone();
    editor.cursor = editor.active()[&editor.selected].len();
    editor.anchor = 0;
    reveal(editor, next);
}
fn reveal(editor: &mut StringEditor, index: usize) {
    if let (Some(top), Some(bottom)) = (
        editor.row_starts.get(index),
        editor.row_starts.get(index + 1),
    ) {
        let height = editor.viewport.y.max(400.);
        if *top < editor.offset {
            editor.offset = *top;
        } else if *bottom > editor.offset + height {
            editor.offset = (*bottom - height).max(0.);
        }
    }
}
fn matches(text: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return vec![];
    }
    let mut folded = String::new();
    let mut boundaries = Vec::new();
    for (i, c) in text.char_indices() {
        let lower = c.to_lowercase().collect::<String>();
        boundaries.extend(std::iter::repeat_n((i, i + c.len_utf8()), lower.len()));
        folded.push_str(&lower);
    }
    folded
        .match_indices(&query.to_lowercase())
        .map(|(i, m)| (boundaries[i].0, boundaries[i + m.len() - 1].1))
        .collect()
}
pub(super) fn find_next(editor: &mut StringEditor, back: bool) {
    if editor.search.is_empty() {
        return;
    }
    let mut hits = Vec::new();
    for key in &editor.filtered {
        for russian in [false, true] {
            if editor.tools.replace_open && russian == editor.tools.replace_english {
                continue;
            }
            let text = if russian {
                &editor.draft[key]
            } else {
                &editor.en[key]
            };
            let ranges = if editor.tools.replace_open {
                replacement_matches(text, &editor.search, &editor.tools)
            } else {
                matches(text, &editor.search)
            };
            for (start, end) in ranges {
                hits.push((key.clone(), russian, start, end));
            }
        }
    }
    if hits.is_empty() {
        editor.tools.found = None;
        editor.status = LocalizedText::new("ui.editor.strings.no_match", "No matches");
        return;
    }
    let index = editor
        .tools
        .found
        .as_ref()
        .and_then(|hit| hits.iter().position(|h| h == hit))
        .map(|i| {
            if back {
                (i + hits.len() - 1) % hits.len()
            } else {
                (i + 1) % hits.len()
            }
        })
        .unwrap_or_else(|| {
            let row = editor
                .filtered
                .iter()
                .position(|k| k == &editor.selected)
                .unwrap_or(0);
            hits.iter()
                .position(|h| editor.row_indices.get(&h.0).copied().unwrap_or(0) >= row)
                .unwrap_or(0)
        });
    let hit = hits[index].clone();
    editor.selected = hit.0.clone();
    editor.editing = true;
    editor.russian = hit.1;
    editor.anchor = hit.2;
    editor.cursor = hit.3;
    if let Some(row) = editor.filtered.iter().position(|k| k == &hit.0) {
        reveal(editor, row);
    }
    editor.tools.found = Some(hit);
    editor.status = LocalizedText::new("ui.editor.strings.match_found", "Match {index} of {count}")
        .with_arg("index", (index + 1).to_string())
        .with_arg("count", hits.len().to_string());
}
pub(super) fn replace_one(editor: &mut StringEditor) {
    if let Some((key, russian, start, end)) = editor.tools.found.clone() {
        if russian != editor.tools.replace_english && editor.keys().contains(&key) {
            let source = if russian { &editor.draft } else { &editor.en };
            let counterpart = if russian { &editor.en } else { &editor.draft };
            if let Some(text) = source.get(&key) {
                if replacement_matches(text, &editor.search, &editor.tools).contains(&(start, end))
                {
                    let mut next = text.clone();
                    next.replace_range(start..end, &editor.tools.replacement);
                    if placeholders(&next) != placeholders(&counterpart[&key]) {
                        editor.status = LocalizedText::new(
                            "ui.editor.strings.exchange_placeholder",
                            "Template parameters do not match: {key}",
                        )
                        .with_arg("key", key);
                        return;
                    }
                    exchange::apply_batch(editor, russian, vec![(key, next)]);
                    editor.search_focus = false;
                }
            }
        }
    }
    find_next(editor, false);
}
#[derive(Component)]
pub(super) struct CellSelection(usize, usize);
pub(super) fn cell_selection(p: &mut ChildSpawnerCommands, cursor: usize, anchor: usize) {
    p.spawn((
        CellSelection(cursor.min(anchor), cursor.max(anchor)),
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            ..default()
        },
    ));
}
pub(super) fn selection_layout(
    mut commands: Commands,
    selection: Query<(Entity, &ChildOf, &CellSelection)>,
    texts: Query<(&Text, Ref<bevy::text::TextLayoutInfo>, &ComputedTextBlock)>,
    children: Query<&Children>,
) {
    for (entity, parent, range) in &selection {
        let Ok((text, layout, block)) = texts.get(parent.parent()) else {
            continue;
        };
        if !layout.is_changed() {
            continue;
        }
        if let Ok(children) = children.get(entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        let scale = layout.scale_factor.max(0.01);
        let selection = TextSelection::from_byte_index(
            block.buffer(),
            range.0.min(text.0.len()),
            Affinity::Downstream,
        )
        .extend(TextCursor::from_byte_index(
            block.buffer(),
            range.1.min(text.0.len()),
            Affinity::Downstream,
        ));
        let lines = selection.geometry(block.buffer());
        commands.entity(entity).with_children(|p| {
            for (bounds, _) in lines {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(bounds.x0 as f32 / scale),
                        top: px(bounds.y0 as f32 / scale),
                        width: px((bounds.x1 - bounds.x0) as f32 / scale),
                        height: px((bounds.y1 - bounds.y0) as f32 / scale),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.6, 0.85, 0.4)),
                ));
            }
        });
    }
}
pub(super) fn drag_cell(
    windows: Query<&Window>,
    mut pointer: Local<Option<Vec2>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut editor: ResMut<StringEditor>,
    cells: Query<(
        &StringCell,
        &RelativeCursorPosition,
        &ComputedNode,
        &ComputedTextBlock,
        &Text,
    )>,
) {
    let point = windows.iter().find_map(Window::cursor_position);
    if !pointer_moved(&mut pointer, point) {
        return;
    }
    if !mouse.pressed(MouseButton::Left)
        || mouse.just_pressed(MouseButton::Left)
        || !editor.editing
        || editor.search_focus
        || editor.file_job.is_some()
    {
        return;
    }
    for (cell, relative, node, layout, text) in &cells {
        if cell.0 != editor.selected || cell.1 != editor.russian || !relative.cursor_over() {
            continue;
        }
        let point = (relative.normalized.unwrap_or_default() + Vec2::splat(0.5)) * node.size();
        if !text.0.is_empty() && layout.buffer().is_empty() {
            continue;
        }
        let index = hit(&text.0, layout, point);
        if index != editor.cursor {
            editor.cursor = index.min(text.0.len());
            editor.revision += 1;
        }
    }
}

#[cfg(test)]
#[path = "strings_tools/find_tests.rs"]
mod find_tests;

fn pointer_moved(previous: &mut Option<Vec2>, point: Option<Vec2>) -> bool {
    let moved = point.is_some() && *previous != point;
    *previous = point;
    moved
}
fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || ('\u{0300}'..='\u{036f}').contains(&c)
}
fn whole(text: &str, start: usize, end: usize) -> bool {
    !text[..start].chars().next_back().is_some_and(word_char)
        && !text[end..].chars().next().is_some_and(word_char)
}
fn replacement_matches(text: &str, query: &str, tools: &Tools) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return vec![];
    }
    let ranges = if tools.case_sensitive {
        text.match_indices(query)
            .map(|(i, m)| (i, i + m.len()))
            .collect()
    } else {
        matches(text, query)
    };
    ranges
        .into_iter()
        .filter(|(start, end)| !tools.whole_word || whole(text, *start, *end))
        .collect()
}
impl Tools {
    pub(super) fn filter_matches(&self, text: &str) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        match self.filter_mode {
            2 => text.trim().to_lowercase() == self.filter.trim().to_lowercase(),
            1 => matches(text, &self.filter)
                .iter()
                .any(|(start, end)| whole(text, *start, *end)),
            _ => text.to_lowercase().contains(&self.filter.to_lowercase()),
        }
    }
    pub(super) fn filter_label(&self) -> (&'static str, &'static str) {
        match self.filter_mode {
            2 => ("ui.editor.strings.filter_exact", "Exclusive"),
            1 => ("ui.editor.strings.filter_word", "Strict"),
            _ => ("ui.editor.strings.filter_normal", "Normal"),
        }
    }
}
pub(super) fn toggle(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: Action,
    key: &str,
    label: &str,
    enabled: bool,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                border_radius: BorderRadius::all(px(4)),
                min_height: px(32),
                padding: UiRect::all(px(7)),
                column_gap: px(7),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.045, 0.19, 0.23)),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(12),
                    height: px(12),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.5, 0.8, 0.85)),
                BackgroundColor(if enabled {
                    Color::srgb(0.3, 0.75, 0.82)
                } else {
                    Color::NONE
                }),
            ));
            p.spawn(editor_text(fonts, key, label, 15., Color::WHITE, false));
        });
}

#[cfg(test)]
#[path = "strings_tools/option_tests.rs"]
mod option_tests;

pub(super) fn move_cell(editor: &mut StringEditor, key: &Key, ctrl: bool, shift: bool) {
    let text = editor
        .active()
        .get(&editor.selected)
        .map(String::as_str)
        .unwrap_or("");
    let next = match key {
        Key::Home => 0,
        Key::End => text.len(),
        Key::ArrowLeft if !shift && !ctrl && editor.cursor != editor.anchor => {
            editor.cursor.min(editor.anchor)
        }
        Key::ArrowRight if !shift && !ctrl && editor.cursor != editor.anchor => {
            editor.cursor.max(editor.anchor)
        }
        _ => step(text, editor.cursor, *key == Key::ArrowRight, ctrl),
    };
    editor.cursor = next;
    if !shift {
        editor.anchor = next;
    }
}
