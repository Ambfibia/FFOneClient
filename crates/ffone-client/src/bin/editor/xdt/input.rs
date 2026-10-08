use super::*;
use bevy::text::ComputedTextBlock;
use parley::Cursor as TextCursor;

pub(super) fn clamp_viewport(
    state: Res<EditorState>,
    mut editor: ResMut<XdtEditor>,
    window: Single<&Window>,
) {
    if !state.xdt_open {
        return;
    }
    let end = editor
        .filtered
        .len()
        .saturating_sub(presentation::visible_rows(
            window.height(),
            editor.advanced,
            editor.mission_table(),
        ));
    if editor.offset > end {
        editor.offset = end;
        editor.revision += 1;
    }
}

pub(super) fn buttons(
    mut state: ResMut<EditorState>,
    mouse:Res<ButtonInput<MouseButton>>,
    mut editor: ResMut<XdtEditor>,
    mut appearance: ResMut<super::super::hnpc::HnpcEditor>,
    catalog: Res<EditorCatalog>,
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    texts: Query<
        (
            &Text,
            &ComputedNode,
            &RelativeCursorPosition,
            &ComputedTextBlock,
        ),
        With<EditText>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    localization: Res<Localization>,
    language: Res<Language>,
) {
    exchange::poll(&mut editor);
    if state.npc_editing() && appearance.active(){return;}
    if !state.xdt_open && !state.npc_editing() {
        return;
    }
    // Rebuilding a pressed button must not fire the same action again while held.
    if !mouse.just_pressed(MouseButton::Left){return;}
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let action = action.clone();
        if state.npc_editing(){state.search_focused=false;}
        if let Action::Field(focus) = &action {
            if editor.focus.as_ref() == Some(focus) {
                if let Some((text, node, cursor, block)) =
                    texts.iter().find(|(_, _, cursor, _)| cursor.cursor_over())
                {
                    if let Some(point) = cursor.normalized {
                        let point = point * node.size();
                        editor.cursor = TextCursor::from_point(block.buffer(), point.x, point.y)
                            .index()
                            .min(text.0.len());
                        if !keys.pressed(KeyCode::ShiftLeft) && !keys.pressed(KeyCode::ShiftRight) {
                            editor.anchor = editor.cursor;
                        }
                        editor.revision += 1;
                    }
                }
                continue;
            }
        }
        if matches!(action, Action::Cancel) {
            if editor.workspace.quick.is_some() { editor.cancel_quick(); continue; }
            editor.array_slot = None;
            editor.focus = None;
            editor.draft = None;
            editor.picker_field = None;
            editor.row = editor.row.filter(|i| *i < editor.rows().len());
            editor.status.clear();
            editor.revision += 1;
            continue;
        }
        if matches!(action, Action::CancelEdit) {
            editor.array_slot = None;
            editor.focus = None;
            editor.picker_field = None;
            editor.status.clear();
            editor.revision += 1;
            continue;
        }
        if editor.draft.is_some()
            && !matches!(
                action,
                Action::Field(Focus::Draft(_) | Focus::ReferenceSearch | Focus::NewName | Focus::Locale(_))
                    | Action::Required(_)
                    | Action::PickReference(..)
                    | Action::Choice(..)
                    | Action::MobLevels | Action::MobLevel(_) | Action::MobStats(_)
                    | Action::Create
                    | Action::Apply
                    | Action::Advanced
                    | Action::OwnName(_)
                    | Action::ExistingName
                    | Action::Block(..)
                    | Action::ArrayCell(..)
                    | Action::ClearReference(_)
                    | Action::Appearance
                    | Action::Mission(mission_workspace::Command::QuickCreate(_))
                    | Action::Mission(mission_workspace::Command::QuickPlaceholder(_))
                    | Action::Mission(mission_workspace::Command::EventLocale(..))
                    | Action::Mission(mission_workspace::Command::NpcTemplate(_))
                    | Action::EditSharedText(_)
                    | Action::CopyText(_)
            )
        {
            editor.status = "Finish or cancel the new record first".into();
            editor.revision += 1;
            continue;
        }
        if !matches!(
            action,
            Action::PickReference(..)
                | Action::Choice(..)
                | Action::Field(Focus::ReferenceSearch)
                | Action::ClearReference(_)
        ) && !editor.apply()
        {
            continue;
        }
        if !matches!(action, Action::Delete) {
            editor.delete_confirm = false;
        }
        if !matches!(action, Action::Reload) {
            editor.reload_confirm = false;
        }
        let result: Result<(), String> = (|| {
            match action {
                Action::Mission(command) => editor.mission_command(command, &localization, &language)?,
                Action::Table(i) => editor.select_table(i),
                Action::Row(i) => {
                    editor.row = Some(i);
                    editor.picker_field = None;
                    editor.rebuild_links();
                }
                Action::Cell(i, field) => {
                    editor.relations_open = false;
                    editor.row = Some(i);
                    editor.rebuild_links();
                    if editor
                        .rows()
                        .get(i)
                        .and_then(|row| row.get(&field))
                        .is_some()
                    {
                        editor.begin(Focus::Cell(field));
                    }
                }
                Action::Block(key, open) => {
                    let context = (editor.table, editor.row, editor.draft.is_some(), key);
                    editor.block_overrides.insert(context, open);
                }
                Action::EditSharedText(field) => {
                    if editor.mission_workspace_table().is_some(){editor.start_mission_text_edit(&field,true)?;}
                    else{editor.edit_shared_text(&field)?;}
                }
                Action::CopyText(field) => {
                    if editor.mission_workspace_table().is_some(){editor.start_mission_text_edit(&field,false)?;}
                    else{editor.copy_text(&field)?;}
                }
                Action::ArrayCell(field, slot) => editor.begin_element(field, slot)?,
                Action::ClearReference(field) => editor.clear_reference(&field)?,
                Action::Field(focus) => editor.begin(focus),
                Action::MobLevels => {editor.workspace.mob_level_picker=!editor.workspace.mob_level_picker;editor.workspace.mob_level=None;editor.revision+=1;},
                Action::MobLevel(level) => {editor.workspace.mob_level=Some(level);editor.revision+=1;},
                Action::MobStats(row) => editor.copy_mob_stats(row)?,
                Action::Sort(field) => {
                    editor.toggle_sort(field);
                }
                Action::Save => editor.save()?,
                Action::Rewrite => editor.rewrite()?,
                Action::Undo => editor.undo(false),
                Action::Redo => editor.undo(true),
                Action::Add | Action::Duplicate => {
                    editor.start_draft(matches!(action, Action::Duplicate))?;
                }
                Action::AddPlaceholder => editor.start_placeholder_draft()?,
                Action::AddTask => {
                    if !editor.tables[editor.table]
                        .label
                        .ends_with("/m_pMissionTable/m_pMissionData")
                    {
                        return Err("Select a mission task".into());
                    }
                    editor.mission_command(mission_workspace::Command::NewStage, &localization, &language)?;
                }
                Action::MissionStages => {
                    editor.mission_filter = if editor.mission_filter.is_some() {
                        None
                    } else {
                        Some(
                            editor
                                .row
                                .and_then(|r| editor.rows().get(r))
                                .and_then(|r| r.get("m_iHMissionID"))
                                .and_then(Value::as_i64)
                                .ok_or("Select a mission task")?,
                        )
                    };
                    editor.offset = 0;
                    editor.refresh();
                }
                Action::Delete => {
                    editor.graph_open = false;
                    let i = editor.row.ok_or("Select a row")?;
                    if !editor.delete_confirm {
                        editor.delete_confirm = true;
                    } else {
                        let mut rows = editor.rows().to_vec();
                        if i >= rows.len() {
                            return Err("Row no longer exists".into());
                        }
                        rows.remove(i);
                        editor.change(rows)?;
                        editor.row = None;
                        editor.delete_confirm = false;
                    }
                }
                Action::Raw => {
                    if editor.row.is_some() {
                        editor.begin(Focus::Row);
                    }
                }
                Action::Apply => {
                    editor.picker_field = None;
                }
                Action::Cancel | Action::CancelEdit => {}
                Action::Advanced => {
                    editor.advanced = !editor.advanced;
                    editor.column_search.clear();
                }
                Action::AllTables => {
                    editor.all_tables = !editor.all_tables;
                    editor.table_offset = 0;
                }
                Action::Graph => {editor.graph_open = !editor.graph_open;if editor.mission_table(){editor.workspace.enabled=true;}}
                Action::GraphAll => editor.graph_all = !editor.graph_all,
                Action::GraphMode(stages) => {
                    editor.graph_stages = stages;
                    editor.workspace.pending_link = None;
                    editor.workspace.selected_edge = None;
                    editor.workspace.view_request = Some(true);
                }
                Action::GraphZoom(delta) => editor.graph_zoom = (editor.graph_zoom + delta).clamp(0.5, 1.5),
                Action::GraphNode(row) => {
                    editor.row = Some(row);
                    editor.picker_field = None;
                    editor.rebuild_links();
                }
                Action::GraphAdd(field) => editor.graph_add(&field)?,
                Action::GraphRemove(row, field, slot) => editor.graph_remove(row, &field, slot)?,
                Action::Appearance | Action::AddHnpc => {
                    if matches!(action,Action::AddHnpc) { editor.start_draft(false)?; }
                    let row = editor.hnpc_value().ok_or("Select an NPC")?;
                    appearance.open(&editor.path, row, &catalog)?;
                    state.xdt_open = false;
                    state.strings_open = false;
                }
                Action::OwnName(field) => {
                    editor.own_name(field)?;
                    editor.begin(Focus::NewName);
                }
                Action::ExistingName => editor.existing_name()?,
                Action::Required(field) => {
                    let fixed = schema::required(&editor.tables[editor.table].label)
                        .contains(&field.as_str());
                    if fixed {
                        editor.status = "This parameter is required by the native client".into();
                    }
                    if let Some(draft) = &mut editor.draft {
                        if !fixed && !draft.required.remove(&field) {
                            draft.required.insert(field);
                        }
                    }
                }
                Action::Create => editor.create_draft()?,
                Action::PickReference(field, t, r) => editor.pick_reference(field, t, r)?,
                Action::Page(down) => {
                    let size = presentation::visible_rows(
                        window.height(),
                        editor.advanced,
                        editor.mission_table(),
                    );
                    editor.offset = if down {
                        (editor.offset + size).min(editor.filtered.len().saturating_sub(size))
                    } else {
                        editor.offset.saturating_sub(size)
                    };
                }
                Action::Relations(open) => {
                    editor.relations_open = open;
                    editor.picker_field = None;
                }
                Action::Choice(field, value) => {
                    if field==mission_visibility::FIELD {return editor.set_mission_visibility(value);}
                    let slot=editor.array_slot;
                    let focus = if editor.draft.is_some() {
                        Focus::Draft(field.clone())
                    } else {
                        Focus::Cell(field.clone())
                    };
                    if let Some(slot)=slot{editor.begin_element(field,slot)?;}else{editor.begin(focus);}
                    editor.edit = value.to_string();
                    if editor.apply() {
                        editor.picker_field = None;
                    }
                }
                Action::Back => {
                    if let Some((t, r)) = editor.history.pop() {
                        editor.select_table(t);
                        editor.row = Some(r);
                        editor.offset = editor.filtered.iter().position(|i| *i == r).unwrap_or(0);
                        editor.rebuild_links();
                    }
                }
                Action::Link(t, r) => editor.navigate(t, r),
                Action::Reload => {
                    if (editor.dirty() || editor.unpublished()) && !editor.reload_confirm {
                        editor.reload_confirm = true;
                    } else {
                        editor.reload_game()?;
                    }
                }
                Action::Export => exchange::start(&mut editor, false)?,
                Action::Import => exchange::start(&mut editor, true)?,
            }
            Ok(())
        })();
        if let Err(error) = result {
            editor.status = error;
        }
        editor.revision += 1;
    }
}
pub(super) fn keyboard(
    state: Res<EditorState>,
    appearance:Res<hnpc::HnpcEditor>,
    keys: Res<ButtonInput<KeyCode>>,
    mut events: MessageReader<KeyboardInput>,
    mut editor: ResMut<XdtEditor>,
    texts: Query<&ComputedTextBlock, With<ActiveText>>,
    localization: Res<Localization>,
    language: Res<Language>,
) {
    if (!state.xdt_open && !state.npc_editing()) || (state.npc_editing() && (appearance.active() || state.search_focused)) {
        events.clear();
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for event in events.read().filter(|e| e.state == ButtonState::Pressed) {
        if ctrl && matches!(event.key_code, KeyCode::KeyS | KeyCode::KeyF) {
            if editor.draft.is_some() {
                editor.status = "Finish or cancel the new record first".into();
                editor.revision += 1;
                continue;
            }
            if editor.apply() {
                if event.key_code == KeyCode::KeyS {
                    if let Err(error) = editor.save() {
                        editor.status = error;
                    }
                } else {
                    editor.begin(Focus::Search);
                }
            }
        } else if ctrl && matches!(event.key_code, KeyCode::KeyZ | KeyCode::KeyY) {
            let redo = event.key_code == KeyCode::KeyY || shift;
            if editor.focus.is_none() {
                if editor.draft.is_none() {
                    editor.undo(redo);
                }
            } else {
                let previous = if redo {
                    editor.text_redo.pop()
                } else {
                    editor.text_undo.pop()
                };
                if let Some((text, cursor, anchor)) = previous {
                    let current = (editor.edit.clone(), editor.cursor, editor.anchor);
                    if redo {
                        editor.text_undo.push(current);
                    } else {
                        editor.text_redo.push(current);
                    }
                    editor.edit = text;
                    editor.cursor = cursor;
                    editor.anchor = anchor;
                    match editor.focus {
                        Some(Focus::Tables) => editor.table_search = editor.edit.clone(),
                        Some(Focus::Search) => {
                            editor.search = editor.edit.clone();
                            editor.refresh();
                        }
                        Some(Focus::Columns) => editor.column_search = editor.edit.clone(),
                        Some(Focus::ReferenceSearch) => {
                            editor.reference_search = editor.edit.clone()
                        }
                        _ => {}
                    }
                }
            }
        } else if event.logical_key == Key::Escape {
            if editor.workspace.context.is_some() || editor.workspace.help.is_some() || editor.workspace.pending_link.is_some() {
                editor.dismiss_mission_context();
                continue;
            }
            if editor.workspace.quick.is_some() && editor.picker_field.is_none() { editor.cancel_quick(); continue; }
            editor.array_slot = None;
            editor.focus = None;
            if editor.picker_field.is_none() {
                editor.draft = None;
            }
            editor.picker_field = None;
            editor.delete_confirm = false;
            editor.reload_confirm = false;
            editor.row = editor.row.filter(|i| *i < editor.rows().len());
        } else if event.logical_key == Key::Enter && ctrl {
            if editor.draft.is_some() {
                if let Err(error) = editor.create_draft() {
                    editor.status = error;
                }
            } else if editor.apply() {
                editor.picker_field = None;
            }
        } else if event.logical_key == Key::Tab {
            if let Some(Focus::Locale(index))=editor.focus {
                if editor.apply(){editor.begin(Focus::Locale(1-index));}
            } else if editor.focus == Some(Focus::NewName) {
                if editor.apply() {
                    if let Some(field) = schema::identity(&editor.tables[editor.table].label) {
                        editor.begin(Focus::Draft(field.into()));
                    }
                }
            } else if matches!(editor.focus, Some(Focus::Row)) {
                editor.replace("  ");
            } else if matches!(editor.focus, Some(Focus::Cell(_))) {
                editor.tab_cell(shift, Some((&localization, &language)));
            } else if let Some(Focus::Draft(current)) = editor.focus.clone() {
                if editor.apply() {
                    let fields = editor
                        .draft
                        .as_ref()
                        .map(|d| editor.authoring_fields(&d.value))
                        .unwrap_or_default();
                    if let Some(i) = fields.iter().position(|f| *f == current) {
                        let next = if shift {
                            i.saturating_sub(1)
                        } else {
                            (i + 1).min(fields.len() - 1)
                        };
                        editor.begin(Focus::Draft(fields[next].clone()));
                    }
                }
            } else {
                if editor.apply() {
                    if editor.draft.is_some() {
                        if editor.draft.as_ref().is_some_and(|d| d.name.is_some()) {
                            editor.begin(Focus::NewName);
                        }
                    } else {
                        editor.begin(if shift { Focus::Tables } else { Focus::Columns });
                    }
                }
            }
        } else if editor.focus.is_some() {
            if ctrl {
                match event.key_code {
                    KeyCode::KeyA => {
                        editor.anchor = 0;
                        editor.cursor = editor.edit.len();
                    }
                    KeyCode::KeyC | KeyCode::KeyX => {
                        let text = editor.edit
                            [editor.cursor.min(editor.anchor)..editor.cursor.max(editor.anchor)]
                            .to_owned();
                        match arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
                            Ok(()) if event.key_code == KeyCode::KeyX => editor.replace(""),
                            Err(e) => editor.status = e.to_string(),
                            _ => {}
                        }
                    }
                    KeyCode::KeyV => match arboard::Clipboard::new().and_then(|mut c| c.get_text())
                    {
                        Ok(text) => editor.replace(&text),
                        Err(e) => editor.status = e.to_string(),
                    },
                    _ => {}
                }
            } else {
                match event.logical_key {
                    Key::ArrowLeft
                    | Key::ArrowRight
                    | Key::ArrowUp
                    | Key::ArrowDown
                    | Key::Home
                    | Key::End => {
                        editor.cursor = match event.logical_key {
                            Key::ArrowLeft => editor.edit[..editor.cursor]
                                .char_indices()
                                .next_back()
                                .map_or(0, |(i, _)| i),
                            Key::ArrowRight => editor.edit[editor.cursor..]
                                .chars()
                                .next()
                                .map_or(editor.cursor, |c| editor.cursor + c.len_utf8()),
                            Key::ArrowUp | Key::ArrowDown => texts
                                .iter()
                                .next()
                                .map(|block| {
                                    let bounds = TextCursor::from_byte_index(
                                        block.buffer(),
                                        editor.cursor,
                                        parley::Affinity::Downstream,
                                    )
                                    .geometry(block.buffer(), 1.);
                                    let direction = if event.logical_key == Key::ArrowUp {
                                        -1.
                                    } else {
                                        1.
                                    };
                                    TextCursor::from_point(
                                        block.buffer(),
                                        bounds.x0 as f32,
                                        ((bounds.y0 + bounds.y1) * 0.5
                                            + direction * (bounds.y1 - bounds.y0))
                                            as f32,
                                    )
                                    .index()
                                    .min(editor.edit.len())
                                })
                                .unwrap_or(editor.cursor),
                            Key::Home => editor.edit[..editor.cursor]
                                .rfind('\n')
                                .map_or(0, |i| i + 1),
                            _ => editor.edit[editor.cursor..]
                                .find('\n')
                                .map_or(editor.edit.len(), |i| editor.cursor + i),
                        };
                        if !shift {
                            editor.anchor = editor.cursor;
                        }
                    }
                    Key::Backspace | Key::Delete => {
                        if editor.cursor == editor.anchor {
                            editor.anchor = if event.logical_key == Key::Backspace {
                                editor.edit[..editor.cursor]
                                    .char_indices()
                                    .next_back()
                                    .map_or(0, |(i, _)| i)
                            } else {
                                editor.edit[editor.cursor..]
                                    .chars()
                                    .next()
                                    .map_or(editor.cursor, |c| editor.cursor + c.len_utf8())
                            };
                        }
                        editor.replace("");
                    }
                    Key::Enter => {
                        if matches!(
                            editor.focus,
                            Some(Focus::Row | Focus::Cell(_) | Focus::Draft(_))
                        ) {
                            editor.replace("\n");
                        } else {
                            editor.apply();
                        }
                    }
                    _ => {
                        if let Some(text) = &event.text {
                            let text: String = text.chars().filter(|c| !c.is_control()).collect();
                            if !text.is_empty() {
                                editor.replace(&text);
                            }
                        }
                    }
                }
            }
        } else if editor.draft.is_none()
            && matches!(event.logical_key, Key::ArrowUp | Key::ArrowDown)
        {
            let position = editor
                .row
                .and_then(|r| editor.filtered.iter().position(|i| *i == r))
                .unwrap_or(0);
            let position = if event.logical_key == Key::ArrowDown {
                (position + 1).min(editor.filtered.len().saturating_sub(1))
            } else {
                position.saturating_sub(1)
            };
            editor.row = editor.filtered.get(position).copied();
            editor.offset = position;
            editor.rebuild_links();
        }
        editor.revision += 1;
    }
}

pub(super) fn field_scroll_key(editor: &XdtEditor, focus: &Focus) -> String {
    format!("{}:{:?}:{}:{focus:?}", editor.table, editor.row, editor.workspace.creation.len())
}
#[derive(Component)]
pub(super) struct FieldScrollKey(pub String);

pub(super) fn remember_field_scroll(
    mut editor: ResMut<XdtEditor>,
    fields: Query<(&FieldScrollKey, &ScrollPosition)>,
) {
    for (key, scroll) in &fields {
        editor.workspace.field_scrolls.insert(key.0.clone(), scroll.0);
    }
}

pub(super) fn reveal_caret(
    editor: Res<XdtEditor>,
    texts: Query<(&Text, &ChildOf, &bevy::text::TextLayoutInfo, &ComputedTextBlock), With<ActiveText>>,
    mut containers: Query<(&ComputedNode, &mut ScrollPosition)>,
    mut revision: Local<u64>,
) {
    if *revision == editor.revision {
        return;
    }
    let mut ready = false;
    for (text, parent, layout, block) in &texts {
        if block.buffer().is_empty() {
            continue;
        }
        // A rebuilt UI can have an empty or stale layout for a frame. Retry
        // until Parley has shaped this exact text before consuming the revision.
        if block.buffer().lines().last().map(|line| line.text_range().end) != Some(text.0.len()) {
            continue;
        }
        let Ok((node, mut scroll)) = containers.get_mut(parent.parent()) else {
            continue;
        };
        let bounds = TextCursor::from_byte_index(
            block.buffer(),
            editor.cursor,
            parley::Affinity::Downstream,
        )
        .geometry(block.buffer(), 1.);
        let scale = layout.scale_factor.max(0.01);
        let point = Vec2::new(bounds.x0 as f32 / scale, bounds.y0 as f32 / scale);
        let size = node.size() * node.inverse_scale_factor() - Vec2::splat(20.);
        let bottom = bounds.y1 as f32 / scale;
        scroll.0.x = if point.x < scroll.0.x {
            point.x
        } else {
            scroll.0.x.max((point.x - size.x).max(0.))
        };
        scroll.0.y = if point.y < scroll.0.y {
            point.y
        } else {
            scroll.0.y.max((bottom - size.y).max(0.))
        };
        ready = true;
    }
    if ready { *revision = editor.revision; }
}
pub(super) fn scroll(
    state: Res<EditorState>,
    mut editor: ResMut<XdtEditor>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheels: MessageReader<MouseWheel>,
    mut regions: Query<(
        &ScrollRegion,
        &ComputedNode,
        &UiGlobalTransform,
        Option<&mut ScrollPosition>,
    )>,
    window: Single<&Window>,
) {
    if !state.xdt_open && !state.npc_editing() {
        wheels.clear();
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        wheels.clear();
        return;
    };
    let scale = window.resolution.scale_factor();
    for wheel in wheels.read() {
        let priority = |r: u8| {
            if r == 3 {
                3
            } else if r == 4 {
                2
            } else {
                1
            }
        };
        let best = regions
            .iter()
            .filter(|(_, node, transform, _)| {
                Rect::from_center_size(transform.translation.into(), node.size())
                    .contains(cursor * scale)
            })
            .map(|(r, _, _, _)| priority(r.0))
            .max()
            .unwrap_or(0);
        let step = match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 3.,
            MouseScrollUnit::Pixel => wheel.y / 30.,
        };
        for (region, node, transform, scroll) in &mut regions {
            if priority(region.0) != best {
                continue;
            }
            let center: Vec2 = transform.translation.into();
            if !(Rect::from_center_size(center, node.size()).contains(cursor * scale)) {
                continue;
            }
            if region.0 == 1 {
                if keys.pressed(KeyCode::ShiftLeft)
                    || keys.pressed(KeyCode::ShiftRight)
                    || wheel.x != 0.
                {
                    let delta = if wheel.x != 0. { wheel.x } else { step };
                    editor.column_offset = (editor.column_offset as f32 - delta).max(0.) as usize;
                    editor.column_offset = editor
                        .column_offset
                        .min(editor.columns.len().saturating_sub(1));
                } else {
                    editor.offset = (editor.offset as f32 - step).max(0.) as usize;
                    editor.offset = editor.offset.min(editor.filtered.len().saturating_sub(1));
                }
                editor.revision += 1;
            } else if region.0 == 4 {
                editor.reference_offset = (editor.reference_offset as f32 - step).max(0.) as usize;
                if let Some(field) = &editor.picker_field {
                    if let Some((t, _)) = editor.reference_target(field) {
                        let total = editor.reference_candidates(t,field).len();
                        editor.reference_offset =
                            editor.reference_offset.min(total.saturating_sub(3));
                    }
                }
                editor.revision += 1;
            } else if region.0 == 0 {
                let total = editor.tables.len();
                editor.table_offset = (editor.table_offset as f32 - step).max(0.) as usize;
                editor.table_offset = editor.table_offset.min(total.saturating_sub(1));
                editor.revision += 1;
            } else if let Some(mut scroll) = scroll {
                let maximum = (node.content_size() - node.size()).max(Vec2::ZERO) * node.inverse_scale_factor();
                if region.0 == 5 {
                    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                        scroll.0.x = (scroll.0.x - step * 30.).clamp(0., maximum.x);
                    } else {
                        scroll.0.y = (scroll.0.y - step * 30.).clamp(0., maximum.y);
                        scroll.0.x = (scroll.0.x - wheel.x * 30.).clamp(0., maximum.x);
                    }
                } else { scroll.0.y = (scroll.0.y - step * 30.).clamp(0., maximum.y); }
            }
        }
    }
}
pub(super) fn drag(
    state: Res<EditorState>,
    mut editor: ResMut<XdtEditor>,
    mouse: Res<ButtonInput<MouseButton>>,
    texts: Query<
        (
            &Text,
            &ComputedNode,
            &RelativeCursorPosition,
            &ComputedTextBlock,
        ),
        With<ActiveText>,
    >,
    tracks: Query<(&Track, &RelativeCursorPosition)>,
    window: Single<&Window>,
    mut previous: Local<Option<Vec2>>,
) {
    let cursor = window.cursor_position();
    let moved = *previous != cursor;
    *previous = cursor;
    if (!state.xdt_open && !state.npc_editing()) || !mouse.pressed(MouseButton::Left) {
        return;
    }
    for (track, relative) in &tracks {
        if relative.cursor_over() && (moved || mouse.just_pressed(MouseButton::Left)) {
            if let Some(point) = relative.normalized {
                if track.0 == 1 {
                    editor.offset = ((point.y.clamp(0., 1.) * editor.filtered.len() as f32)
                        as usize)
                        .min(editor.filtered.len().saturating_sub(1));
                    editor.revision += 1;
                    return;
                }
            }
        }
    }
    if !moved || mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (text, node, relative, block) in &texts {
        if !relative.cursor_over() {
            continue;
        }
        if let Some(point) = relative.normalized {
            let point = point * node.size();
            editor.cursor = TextCursor::from_point(block.buffer(), point.x, point.y)
                .index()
                .min(text.0.len());
            editor.revision += 1;
        }
    }
}
