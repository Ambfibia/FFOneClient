use super::view::{button, dynamic_button, field, label, property};
use super::*;

const WHITE: Color = Color::srgb(0.86, 0.94, 0.96);
const MUTED: Color = Color::srgb(0.57, 0.67, 0.73);
const ACCENT: Color = Color::srgb(0.45, 0.79, 0.74);

pub(super) fn visible_rows(height: f32, advanced: bool, mission: bool) -> usize {
    ((height
        - EDITOR_HEADER_HEIGHT
        - 260.
        - if advanced { 36. } else { 0. }
        - if mission { 76. } else { 0. })
        / 34.)
        .max(1.) as usize
}
#[derive(Default)]
pub(super) struct ScrollMemory {
    context: (
        usize,
        Option<usize>,
        bool,
        bool,
        bool,
        Option<String>,
        Option<usize>,
    ),
    position: Vec2,
}

fn stack() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        min_height: px(0),
        min_width: px(0),
        row_gap: px(8),
        ..default()
    }
}
fn row() -> Node {
    Node {
        min_height: px(32),
        flex_shrink: 0.,
        column_gap: px(6),
        align_items: AlignItems::Center,
        ..default()
    }
}
pub(super) fn tr(l: &Localization, lang: &Language, key: &str, en: &str) -> String {
    schema::text(l, lang, key, en)
}
fn action(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    a: Action,
    key: &str,
    en: &str,
    width: f32,
) {
    button(p, f, a, &format!("ui.editor.xdt.{key}"), en, false, width);
}
fn shown_value(e: &XdtEditor, r: usize, col: &str, l: &Localization, lang: &Language) -> String {
    let value = &e.rows()[r][col];
    if let Some(value) = schema::value_name(&e.tables[e.table].label, col, value, l, lang) {
        return value;
    }
    if !e.advanced {
        if let Some(values) = value.as_array() {
            let names: Vec<_> = e
                .references
                .iter()
                .filter(|reference| {
                    reference.table == e.table
                        && reference.row == r
                        && reference.field.starts_with(&format!("{col}["))
                })
                .filter_map(|reference| {
                    reference
                        .target_row
                        .map(|target| record_title(e, reference.target_table, target, l, lang))
                })
                .collect();
            if !names.is_empty() {
                return names.join(", ");
            }
            return format!(
                "{} / {} · {}",
                values.iter().filter(|v| !authoring::neutral(v)).count(),
                values.len(),
                tr(l, lang, "list_filled", "filled elements")
            );
        }
        if let Some(reference) = e
            .references
            .iter()
            .find(|v| v.table == e.table && v.row == r && v.field == col)
        {
            if let Some(target) = reference.target_row {
                return record_title(e, reference.target_table, target, l, lang);
            }
        }
        if value.as_i64() == Some(0)
            && (relations::optional_index(col)
                || e.reference_target(col).is_some_and(|(_, id)| id.is_some()))
        {
            return tr(l, lang, "not_assigned", "Not assigned");
        }
        if value.is_f64() {
            if let Some(n) = value
                .as_f64()
                .filter(|n| n.abs() >= 0.000001 && n.abs() < 1e12)
            {
                return format!("{n:.6}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_owned();
            }
        }
    }
    e.value_label(r, col)
}
fn record_title(e: &XdtEditor, t: usize, r: usize, l: &Localization, lang: &Language) -> String {
    let row = e
        .document
        .pointer(&e.tables[t].pointer)
        .and_then(|rows| rows.get(r));
    if let Some(number) = row.and_then(|v| v.get("m_iIconNumber")) {
        return format!(
            "{} {} · {} {}",
            tr(l, lang, "icon_label", "Icon"),
            display(number),
            tr(l, lang, "atlas_label", "atlas"),
            row.and_then(|v| v.get("m_iIconType"))
                .map(display)
                .unwrap_or_default()
        );
    }
    e.record_name(t, r)
}

pub(super) fn draw(
    mut commands: Commands,
    fonts: Option<Res<EditorFonts>>,
    state: Res<EditorState>,
    editor: Res<XdtEditor>,
    localization: Res<Localization>,
    language: Res<Language>,
    roots: Query<Entity, With<Root>>,
    window: Single<&Window>,
    mut shown: Local<(bool, u64, u32, u32)>,
    scrolls: Query<(&ScrollRegion, &ScrollPosition)>,
    mut memory: Local<ScrollMemory>,
    mut graph_memory: Local<mission_graph::GraphScroll>,
    skin:Res<mission_skin::MissionSkin>,
) {
    let next = (
        state.xdt_open,
        editor.revision,
        window.width() as u32,
        window.height() as u32,
    );
    if *shown == next && !language.is_changed() {
        return;
    }
    let Some(fonts) = fonts else { return };
    *shown = next;
    for root in &roots {
        commands.entity(root).despawn();
    }
    if !state.xdt_open {
        return;
    }
    let l = &localization;
    let lang = &language;
    let f = &fonts;
    let e = &editor;
    if e.mission_workspace_table().is_some() {
        let started=std::time::Instant::now();
        let scroll=scrolls.iter().find(|(r,_)|r.0==2).map(|(_,s)|s.0).unwrap_or_default();
        let catalog=scrolls.iter().find(|(r,_)|r.0==6).map(|(_,s)|s.0).unwrap_or_default();
        super::mission_view::draw(&mut commands,f,e,l,lang,&skin,window.width(),window.height(),scroll,catalog);
        if std::env::var_os("FFONE_MISSION_PROFILE").is_some(){eprintln!("mission full draw {:?}",started.elapsed());}
        return;
    }
    let Some(table) = e.tables.get(e.table) else {
        return;
    };
    let creating = e.draft.is_some();
    let graph = e.graph_open && e.mission_table() && !creating;
    let graph_context = Some((e.table, if e.graph_all {None}else{e.row}, e.graph_stages, e.graph_zoom.to_bits() ^ u32::from(e.graph_all)));
    let graph_position = if graph_memory.context == graph_context {
        scrolls.iter().find(|(r,_)| r.0 == 5).map(|(_,s)| s.0).unwrap_or(graph_memory.position)
    } else { Vec2::ZERO };
    graph_memory.context = graph_context;
    graph_memory.position = graph_position;
    let context = (
        e.table,
        e.row,
        e.advanced,
        creating,
        e.relations_open,
        e.picker_field.clone(),
        e.array_slot,
    );
    let position = if memory.context == context {
        scrolls
            .iter()
            .find(|(r, _)| r.0 == 2)
            .map(|(_, s)| s.0)
            .unwrap_or(memory.position)
    } else {
        Vec2::ZERO
    };
    memory.context = context;
    memory.position = position;
    let editing = matches!(e.focus, Some(Focus::Cell(_) | Focus::Row | Focus::Draft(_)))
        || e.picker_field.is_some();
    let count = visible_rows(window.height(), e.advanced, e.mission_table());
    let table_count = ((window.height() - EDITOR_HEADER_HEIGHT - 225.) / 38.).max(1.) as usize;
    let inspector_width = if window.width() < 1300. { 360. } else { 430. };
    commands.spawn((Root, Node {
        position_type: PositionType::Absolute, top: px(EDITOR_HEADER_HEIGHT), bottom: px(0), width: percent(100),
        padding: UiRect::all(px(12)), overflow: Overflow::clip(), ..stack()
    }, ZIndex(200), BackgroundColor(Color::srgb(0.035, 0.055, 0.075))))
    .with_children(|root| {
        root.spawn(row()).with_children(|bar| {
            if !creating {
            action(bar, f, Action::Save, "save", "Save  Ctrl+S", 124.);
            action(bar, f, Action::Rewrite, "mission.rewrite", "Rewrite", 120.);
            action(bar, f, Action::Undo, "undo", "Undo", 70.);
            action(bar, f, Action::Redo, "redo", "Redo", 70.);
            action(bar, f, Action::Add, if table.label.ends_with("/m_pMissionTable/m_pMissionData") { "new_mission" } else { "new_record" }, "+ New record", 150.);
            if table.label.ends_with("/m_pNpcTable/m_pNpcData") {
                action(bar, f, Action::AddPlaceholder, "new_placeholder", "Create placeholder", 170.);
            }
            if table.label.ends_with("/m_pMissionTable/m_pMissionData") {
                action(bar, f, Action::AddTask, "new_task", "+ Mission stage", 150.);
            }
            action(bar, f, Action::Duplicate, "duplicate", "Duplicate", 110.);
            action(bar, f, Action::Delete, if e.delete_confirm { "confirm_delete" } else { "delete" },
                if e.delete_confirm { "Confirm delete" } else { "Delete" }, 125.);
            }
            action(bar, f, Action::Advanced, if e.advanced { "basic" } else { "advanced" },
                if e.advanced { "Main fields" } else { "All fields" }, 160.);
        });
        mission_server::destination(root, f, e, l, lang);
        root.spawn(Node { flex_grow: 1., min_height: px(0), column_gap: px(12), overflow: Overflow::clip(), ..default() })
        .with_children(|body| {
            if !creating && !graph {
            body.spawn((Node { width: px(218), padding: UiRect::all(px(10)), border_radius: BorderRadius::all(px(8)), flex_shrink: 0., overflow: Overflow::clip(), ..stack() }, BackgroundColor(Color::srgb(0.06, 0.085, 0.11))))
            .with_children(|list| {
                label(list, f, tr(l, lang, "data", "Game data"), 19., WHITE);
                field(list, f, e, Focus::Tables, 34.);
                action(list, f, Action::AllTables, if e.all_tables { "main_tables" } else { "all_tables" },
                    if e.all_tables { "Main tables" } else { "All tables" }, 198.);
                list.spawn((ScrollRegion(0), Node { flex_grow: 1., overflow: Overflow::clip(), ..stack() }))
                .with_children(|tables| {
                    let needle = e.table_search.to_lowercase();
                    let mut order: Vec<_> = e.tables.iter().enumerate().collect();
                    order.sort_by_key(|(_, t)| {
                        let group = ["m_pNpcTable", "m_pNanoTable", "m_pMissionTable", "m_pWeaponItemTable", "m_pShirtsItemTable", "m_pPantsItemTable", "m_pShoesItemTable"]
                            .iter().position(|g| t.label.contains(&format!("/{g}/"))).unwrap_or(20);
                        let kind = if schema::identity(&t.label).is_some() { 0 } else if t.label.ends_with("StringData") { 1 } else { 2 };
                        (group, kind, t.label.clone())
                    });
                    for (i, t) in order.into_iter().filter(|(i,t)| (e.all_tables || !needle.is_empty() || *i == e.table || schema::identity(&t.label).is_some())
                        && (t.label.to_lowercase().contains(&needle) || schema::table_name(&t.label, l, lang).to_lowercase().contains(&needle)))
                        .skip(e.table_offset).take(table_count) {
                        dynamic_button(tables, f, Action::Table(i), schema::table_name(&t.label, l, lang), i == e.table, 198.);
                    }
                });
            });
            }
            body.spawn((Node { flex_grow: 1., padding: UiRect::all(px(12)), border_radius: BorderRadius::all(px(8)), overflow: Overflow::clip(), ..stack() }, BackgroundColor(Color::srgb(0.055, 0.08, 0.10)))).with_children(|center| {
                label(center, f, schema::table_name(&table.label, l, lang), 22., WHITE);
                if !creating && table.label.ends_with("/m_pMissionTable/m_pMissionData") {
                    action(center, f, Action::MissionStages, if e.mission_filter.is_some() { "all_missions" } else { "mission_stages" }, "Stages of selected mission", 0.);
                    action(center, f, Action::Graph, if graph { "graph.list" } else { "graph.open" }, "Mission graph", 0.);
                }
                if creating {
                    if e.hnpc_value().is_some() {action(center,f,Action::Appearance,"hnpc.appearance","HNPC appearance",0.);}
                    creation(center, f, e, l, lang, position);
                } else {
                    if !graph {center.spawn(row()).with_children(|search| {
                        field(search, f, e, Focus::Search, 34.);
                    });}
                    if graph { mission_graph::draw(center,f,e,l,lang,graph_position); }
                    else { overview(center, f, e, l, lang, count, window.width() - inspector_width - 290.); }
                    if e.advanced && !graph {
                        center.spawn(row()).with_children(|bar| {
                            action(bar, f, Action::Export, "export", "Export CSV", 115.);
                            action(bar, f, Action::Import, "import", "Import CSV", 115.);
                            action(bar, f, Action::Reload, if e.reload_confirm { "confirm_reload" } else { "reload" }, "Reload", 140.);
                        });
                    }
                }
                if editing && e.focus == Some(Focus::Row) { edit_panel(center, f, e, l, lang); }
            });
            body.spawn(Node { width: px(inspector_width), flex_shrink: 0., padding: UiRect::all(px(12)), border_radius: BorderRadius::all(px(8)),
                overflow: Overflow::clip(), ..stack() }).insert(BackgroundColor(Color::srgb(0.06, 0.085, 0.11)))
            .with_children(|card| {
                if creating {
                    if editing { scroll_editor(card, f, e, l, lang, position); return; }
                    label(card, f, tr(l, lang, if e.draft.as_ref().is_some_and(|d| d.name.is_some()) { "creation_help_own" } else { "creation_help" }, "Configure the required parameters, then create the record."), 15., WHITE);
                    if let Some(draft) = &e.draft {
                        label(card, f, format!("{}: {}", tr(l, lang, "template", "Settings copied from"), e.record_name(e.table, draft.template)), 14., MUTED);
                        let errors = e.draft_errors();
                        label(card, f, tr(l, lang, "required_help", "★ Required. In All fields you can mark additional parameters as required."), 14., ACCENT);
                        for (field, reason) in errors.iter().take(10) {
                            let focus = if draft.name.as_ref().is_some_and(|n| n.field == *field) { Focus::NewName } else { Focus::Draft(field.clone()) };
                            dynamic_button(card, f, Action::Field(focus), format!("{}: {}", schema::field_name(field, l, lang), tr(l, lang, &format!("error.{reason}"), reason)), false, 320.);
                        }
                    }
                } else if let Some(r) = e.row.filter(|r| *r < e.rows().len()) {
                    label(card, f, e.record_name(e.table, r), 22., WHITE);
                    if e.hnpc_value().is_some() {action(card,f,Action::Appearance,"hnpc.appearance","HNPC appearance",0.);action(card,f,Action::AddHnpc,"hnpc.new","+ Create HNPC",0.);}
                    card.spawn(row()).with_children(|bar| {
                        button(bar, f, Action::Relations(false), "ui.editor.xdt.parameters", "Parameters", !e.relations_open, 125.);
                        dynamic_button(bar, f, Action::Relations(true), format!("{} ({})", tr(l, lang, "connections", "Connections"), e.links.iter().filter(|link| link.confirmed).count()), e.relations_open, 140.);
                        dynamic_button(bar, f, Action::Back, "←".into(), false, 45.);
                    });
                    if e.advanced { action(card, f, Action::Raw, "raw", "Edit JSON", 220.); }
                    if editing && e.focus != Some(Focus::Row) { scroll_editor(card, f, e, l, lang, position); }
                    else { field_card(card, f, e, l, lang, r, position); }
                } else {
                    label(card, f, tr(l, lang, "select_help", "Select a record to edit its parameters and see what uses it."), 16., MUTED);
                }
            });
        });
        label(root, f, format!("{} / {} · {}{}", e.filtered.len(), e.rows().len(),
            tr(l, lang, if e.dirty() || creating { "unsaved" } else if e.unpublished() { "mission.pending_work" } else { "saved" },
                if e.dirty() || creating { "Unsaved changes" } else if e.unpublished() { "Saved · Not applied to game" } else { "Saved" }),
            if e.status.is_empty() || e.status == "Saved" { String::new() } else { format!(" · {}", schema::status(&e.status, l, lang)) }), 13., MUTED);
    });
}

fn overview(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    count: usize,
    width: f32,
) {
    let table = &e.tables[e.table];
    let mut cols = schema::overview(&table.label, &e.columns);
    let identity = sorting::identity(&table.label, &e.columns);
    let indexed = identity == SortKey::Index;
    if let SortKey::Field(id) = &identity {
        cols.retain(|col| col != id);
        cols.insert(0, id.clone());
    }
    let count_cols = if width < 600. {
        if indexed { 1 } else { 2 }
    } else {
        cols.len().min(if indexed { 3 } else { 4 })
    };
    let widths: Vec<f32> = (0..count_cols)
        .map(|i| {
            if i == 0 {
                75.
            } else {
                (width - 90.) / (count_cols - 1).max(1) as f32
            }
        })
        .collect();
    let col_width = |i| {
        if indexed {
            (width - 90.) / count_cols.max(1) as f32
        } else {
            widths[i]
        }
    };
    p.spawn((
        ScrollRegion(1),
        Node {
            flex_grow: 1.,
            overflow: Overflow::clip(),
            row_gap: px(2),
            ..stack()
        },
    ))
    .with_children(|grid| {
        grid.spawn((row(), BackgroundColor(Color::srgb(0.105, 0.15, 0.18))))
            .with_children(|header| {
                if indexed {
                    dynamic_button(
                        header,
                        f,
                        Action::Sort(SortKey::Index),
                        sorting::heading(e, &SortKey::Index, tr(l, lang, "row_index", "Index")),
                        false,
                        75.,
                    );
                }
                for (i, col) in cols.iter().take(count_cols).enumerate() {
                    dynamic_button(
                        header,
                        f,
                        Action::Sort(SortKey::Field(col.clone())),
                        sorting::heading(e, &SortKey::Field(col.clone()), schema::field_name(col, l, lang)),
                        false,
                        col_width(i),
                    );
                }
            });
        for &r in e.filtered.iter().skip(e.offset).take(count) {
            grid.spawn((
                Node {
                    border_radius: BorderRadius::all(px(4)),
                    ..row()
                },
                BackgroundColor(if e.row == Some(r) {
                    Color::srgb(0.10, 0.29, 0.31)
                } else if r % 2 == 0 {
                    Color::srgb(0.075, 0.105, 0.13)
                } else {
                    Color::srgb(0.06, 0.085, 0.11)
                }),
            ))
            .with_children(|line| {
                if indexed {
                    dynamic_button(
                        line,
                        f,
                        Action::Row(r),
                        r.to_string(),
                        e.row == Some(r),
                        75.,
                    );
                }
                for (i, col) in cols.iter().take(count_cols).enumerate() {
                    let value = if schema::name_field(&table.label) == Some(col.as_str()) {
                        e.record_name(e.table, r)
                    } else {
                        shown_value(e, r, col, l, lang)
                    };
                    dynamic_button(
                        line,
                        f,
                        Action::Row(r),
                        value,
                        e.row == Some(r),
                        col_width(i),
                    );
                }
            });
        }
        if e.filtered.is_empty() {
            label(
                grid,
                f,
                tr(l, lang, "no_results", "No matching records"),
                16.,
                MUTED,
            );
        }
    });
    p.spawn(row()).with_children(|nav| {
        action(nav, f, Action::Page(false), "previous", "↑ Previous", 120.);
        action(nav, f, Action::Page(true), "next", "↓ Next", 120.);
    });
}

fn block_header(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    block: &authoring::Block,
) {
    let open = e.block_open(block);
    let title = tr(l, lang, &format!("block.{}", block.key), block.key);
    dynamic_button(
        p,
        f,
        Action::Block(block.key.into(), !open),
        format!(
            "{} {}{}",
            if open { "−" } else { "+" },
            title,
            if !block.configured && !block.core {
                format!(" · {}", tr(l, lang, "block.empty", "empty"))
            } else {
                String::new()
            }
        ),
        false,
        0.,
    );
}

fn field_card(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    r: usize,
    position: Vec2,
) {
    p.spawn((ScrollRegion(2), ScrollPosition(position), Node { flex_grow: 1., overflow: Overflow::scroll_y(), ..stack() }))
    .with_children(|fields| {
        if !e.relations_open {
        if e.advanced {
            label(fields, f, tr(l, lang, "field_search", "Find a parameter"), 14., MUTED);
            field(fields, f, e, Focus::Columns, 34.);
        }
        for block in e.authoring_blocks(&e.rows()[r]) {
            let searching = e.advanced && !e.column_search.is_empty();
            let cols: Vec<_> = block.fields.iter().filter(|col| !searching || col.to_lowercase().contains(&e.column_search.to_lowercase())
                || schema::field_name(col, l, lang).to_lowercase().contains(&e.column_search.to_lowercase())).collect();
            if cols.is_empty() { continue; }
            block_header(fields, f, e, l, lang, &block);
            if !searching && !e.block_open(&block) { continue; }
            for col in cols {
            let value = shown_value(e, r, col, l, lang);
            property(fields, f, Action::Cell(r, col.clone()), schema::field_name(col, l, lang), value,
                e.picker_field.as_ref() == Some(col) || e.focus == Some(Focus::Cell(col.clone())));
            }
        }
        } else {
        label(fields, f, tr(l, lang, "relations", "Links and references"), 19., ACCENT);
        label(fields, f, tr(l, lang, "shared_help", "→ Uses this record · ← Used by these records. Editing shared data affects every user."), 13., MUTED);
        for reference in e.references.iter().filter(|r| r.table == e.table && Some(r.row) == e.row && r.target_row.is_none()) {
            label(fields, f, format!("{}: {} → {}", tr(l, lang, "missing_reference", "Missing reference"), schema::field_name(&reference.field, l, lang), reference.value), 14., Color::srgb(1., 0.55, 0.4));
        }
        for link in e.links.iter().filter(|link| link.confirmed || e.advanced) {
            let target = schema::table_name(&e.tables[link.table].label, l, lang);
            let field_label = schema::field_name(link.field.trim_start_matches('/'), l, lang);
            label(fields, f, format!("{} {} · {}{}", if !link.confirmed { "?" } else if link.incoming { "←" } else { "→" }, field_label, target,
                if e.advanced { format!(" · {}", link.value) } else { String::new() }), 13., MUTED);
            dynamic_button(fields, f, Action::Link(link.table, link.row), record_title(e, link.table, link.row, l, lang), false, 320.);
        }
        }
        if e.advanced { label(fields, f, e.tables[e.table].label.clone(), 12., MUTED); }
    });
}

fn creation(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    position: Vec2,
) {
    let Some(draft) = &e.draft else { return };
    label(p, f, tr(l, lang, if draft.placeholder { "new_placeholder" } else { "new_record" }, if draft.placeholder { "Create placeholder" } else { "New record" }), 19., WHITE);
    npc_templates::choices(p, f, e, l, lang);
    p.spawn(row()).with_children(|bar| {
        action(bar, f, Action::Create, "create", "Create record", 170.);
        action(bar, f, Action::Cancel, "cancel", "Cancel  Esc", 120.);
    });
    if draft.name.is_some() {
        label(
            p,
            f,
            tr(l, lang, "new_name", "Name of the new record *"),
            16.,
            ACCENT,
        );
        field(p, f, e, Focus::NewName, 40.);
        action(
            p,
            f,
            Action::ExistingName,
            "existing_text",
            "Choose existing text instead",
            300.,
        );
    }
    p.spawn((
        ScrollRegion(2),
        ScrollPosition(position),
        Node {
            flex_grow: 1.,
            overflow: Overflow::scroll_y(),
            ..stack()
        },
    ))
    .with_children(|form| {
        for block in e.authoring_blocks(&draft.value) {
            block_header(form, f, e, l, lang, &block);
            if !e.block_open(&block) {
                continue;
            }
            for col in &block.fields {
                if draft.name.as_ref().is_some_and(|n| n.field == *col) {
                    continue;
                }
                let Some(value) = draft.value.get(col) else {
                    continue;
                };
                let value = if let Some((t, id)) = e.reference_target(col) {
                    let r = if let Some(id) = id {
                        e.document
                            .pointer(&e.tables[t].pointer)
                            .and_then(Value::as_array)
                            .and_then(|rows| rows.iter().position(|r| r.get(id) == Some(value)))
                    } else {
                        value.as_u64().map(|v| v as usize)
                    };
                    r.map(|r| {
                        if e.advanced {
                            format!("{} · {}", e.record_name(t, r), display(value))
                        } else {
                            record_title(e, t, r, l, lang)
                        }
                    })
                    .unwrap_or_else(|| display(value))
                } else {
                    schema::value_name(&e.tables[e.table].label, col, value, l, lang)
                        .unwrap_or_else(|| display(value))
                };
                form.spawn(row()).with_children(|line| {
                    dynamic_button(
                        line,
                        f,
                        Action::Required(col.clone()),
                        if draft.required.contains(col) {
                            "★"
                        } else {
                            "☆"
                        }
                        .into(),
                        draft.required.contains(col),
                        34.,
                    );
                    line.spawn(Node {
                        flex_grow: 1.,
                        min_width: px(0),
                        ..default()
                    })
                    .with_children(|p| {
                        property(
                            p,
                            f,
                            Action::Field(Focus::Draft(col.clone())),
                            schema::field_name(col, l, lang),
                            value,
                            e.picker_field.as_ref() == Some(col),
                        );
                    });
                });
            }
        }
    });
}

fn scroll_editor(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    position: Vec2,
) {
    p.spawn((
        ScrollRegion(2),
        ScrollPosition(position),
        Node {
            flex_grow: 1.,
            overflow: Overflow::scroll_y(),
            ..stack()
        },
    ))
    .with_children(|p| edit_panel(p, f, e, l, lang));
}

pub(super) fn edit_panel(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
) {
    let col = e.picker_field.as_deref();
    if let Some(col) = col {
        label(p, f, schema::field_name(col, l, lang), 18., ACCENT);
        if let Some(slot) = e.array_slot {
            label(
                p,
                f,
                format!("{} {}", tr(l, lang, "list_element", "Element"), slot + 1),
                15.,
                ACCENT,
            );
        }
        label(p, f, schema::help(col, l, lang), 14., MUTED);
        if let Some(value) = e
            .field_value(col)
            .and_then(|v| e.array_slot.map_or(Some(v), |slot| v.get(slot)))
        {
            let kind = if value.is_number() && e.reference_target(col).is_some() {
                if e.reference_target(col)
                    .is_some_and(|(t, _)| e.tables[t].label.ends_with("StringData"))
                {
                    "text_link"
                } else {
                    "reference"
                }
            } else {
                match value {
                    Value::String(_) => "text",
                    Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Array(_) => "list",
                    Value::Object(_) => "object",
                    _ => "null",
                }
            };
            let required = e.draft.as_ref().is_some_and(|d| d.required.contains(col))
                || schema::required(&e.tables[e.table].label).contains(&col);
            label(
                p,
                f,
                format!(
                    "{}{}",
                    tr(l, lang, &format!("field_type.{kind}"), kind),
                    if required {
                        format!(" · {}", tr(l, lang, "field_required", "Required"))
                    } else {
                        String::new()
                    }
                ),
                13.,
                MUTED,
            );
        }
        if e.advanced {
            label(p, f, col, 12., MUTED);
        }
        if let Some(bounds) = schema::limits(&e.tables[e.table].label, col) {
            label(
                p,
                f,
                format!(
                    "{}: {}",
                    tr(l, lang, "field_range", "Allowed range"),
                    bounds
                        .max
                        .map(|max| format!("{}–{max}", bounds.min))
                        .unwrap_or_else(|| format!(
                            "{} {}",
                            tr(l, lang, "range_from", "from"),
                            bounds.min
                        ))
                ),
                14.,
                MUTED,
            );
        }
        if e.array_slot.is_none() {
            if let Some(values) = e.field_value(col).and_then(Value::as_array) {
                label(
                    p,
                    f,
                    tr(
                        l,
                        lang,
                        "list_help",
                        "Choose an element. Other elements and list size stay unchanged.",
                    ),
                    14.,
                    MUTED,
                );
                for (slot, value) in values.iter().enumerate() {
                    let shown = e
                        .reference_target(col)
                        .and_then(|(t, id)| {
                            let rows = e.document.pointer(&e.tables[t].pointer)?.as_array()?;
                            let r = if let Some(id) = id {
                                if value.as_i64() == Some(0) {
                                    return Some(tr(l, lang, "not_assigned", "Not assigned"));
                                }
                                rows.iter().position(|r| r.get(id) == Some(value))?
                            } else {
                                usize::try_from(value.as_u64()?).ok()?
                            };
                            Some(record_title(e, t, r, l, lang))
                        })
                        .unwrap_or_else(|| display(value));
                    property(
                        p,
                        f,
                        Action::ArrayCell(col.into(), slot),
                        format!("{} {}", tr(l, lang, "list_element", "Element"), slot + 1),
                        shown,
                        false,
                    );
                }
                action(p, f, Action::CancelEdit, "close_field", "Close field", 150.);
                return;
            }
        }
        let choices = schema::choices(&e.tables[e.table].label, col);
        if !choices.is_empty() {
            p.spawn(stack()).with_children(|bar| {
                for (value, name) in choices {
                    dynamic_button(
                        bar,
                        f,
                        Action::Choice(col.into(), *value),
                        tr(l, lang, &format!("attribute.{name}"), name),
                        false,
                        320.,
                    );
                }
                action(
                    bar,
                    f,
                    Action::CancelEdit,
                    "close_field",
                    "Close field",
                    110.,
                );
            });
            return;
        }
        if let Some((t, id)) = e.reference_target(col) {
            if id.is_some() || relations::optional_index(col) {
                action(
                    p,
                    f,
                    Action::ClearReference(col.into()),
                    "clear_reference",
                    "No linked record",
                    0.,
                );
            }
            if e.draft.is_none() {
                if let Some(target) = e.text_target(col) {
                    label(
                        p,
                        f,
                        format!(
                            "{}: {}",
                            tr(l, lang, "text_users", "Linked records"),
                            target.users
                        ),
                        14.,
                        ACCENT,
                    );
                    action(
                        p,
                        f,
                        Action::EditSharedText(col.into()),
                        "edit_shared",
                        "Edit linked text",
                        0.,
                    );
                    action(
                        p,
                        f,
                        Action::CopyText(col.into()),
                        "copy_text",
                        "Create a private text copy",
                        0.,
                    );
                }
            }
            label(
                p,
                f,
                schema::table_name(&e.tables[t].label, l, lang),
                16.,
                WHITE,
            );
            p.spawn(Node {
                width: percent(100),
                ..row()
            })
            .with_children(|bar| {
                field(bar, f, e, Focus::ReferenceSearch, 34.);
            });
            if e.draft.is_some() && schema::name_field(&e.tables[e.table].label) == Some(col) {
                action(
                    p,
                    f,
                    Action::OwnName(col.into()),
                    "own_text",
                    "+ Create a new name",
                    320.,
                );
            }
            label(
                p,
                f,
                tr(
                    l,
                    lang,
                    "reference_scroll",
                    "Search by name; use the wheel for more results",
                ),
                13.,
                MUTED,
            );
            p.spawn((
                ScrollRegion(4),
                Node {
                    height: px(130),
                    flex_shrink: 0.,
                    overflow: Overflow::clip(),
                    ..stack()
                },
            ))
            .with_children(|choices| {
                let rows = e
                    .document
                    .pointer(&e.tables[t].pointer)
                    .and_then(Value::as_array);
                for r in e.reference_candidates(t,col).into_iter()
                    .skip(e.reference_offset)
                    .take(3)
                {
                    let value=&rows.unwrap()[r];
                    let id = id
                        .and_then(|id| value.get(id))
                        .map(display)
                        .unwrap_or_else(|| r.to_string());
                    dynamic_button(
                        choices,
                        f,
                        Action::PickReference(col.into(), t, r),
                        if e.advanced {
                            format!("{} · {}", e.record_name(t, r), id)
                        } else {
                            record_title(e, t, r, l, lang)
                        },
                        false,
                        320.,
                    );
                }
            });
            action(p, f, Action::CancelEdit, "close_field", "Close field", 150.);
            return;
        }
    } else {
        label(p, f, "JSON", 16., WHITE);
    }
    if let Some(focus) = &e.focus {
        field(
            p,
            f,
            e,
            focus.clone(),
            if *focus == Focus::Row {
                160.
            } else if col.is_some_and(|c| c.starts_with("m_strComment") || c == "m_pstrNameString")
            {
                120.
            } else {
                40.
            },
        );
        p.spawn(row()).with_children(|bar| {
            action(bar, f, Action::Apply, "apply", "Apply  Ctrl+Enter", 170.);
            action(
                bar,
                f,
                Action::CancelEdit,
                "close_field",
                "Close field",
                130.,
            );
        });
    }
}
