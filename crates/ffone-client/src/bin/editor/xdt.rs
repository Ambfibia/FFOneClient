//! Lossless native TableData authoring. JSON pointers retain every table and field.
use super::*;
use std::{io::Write, path::Path};

#[path = "xdt/authoring.rs"]
mod authoring;
#[path = "xdt/mission_graph.rs"]
mod mission_graph;
#[path = "xdt/mission_fields.rs"]
mod mission_fields;
#[path = "xdt/mission_workspace.rs"]
mod mission_workspace;
#[path = "xdt/mission_commands.rs"]
mod mission_commands;
#[path = "xdt/mission_order.rs"]
mod mission_order;
#[path = "xdt/mission_canvas.rs"]
mod mission_canvas;
#[path = "xdt/mission_inline.rs"]
mod mission_inline;
#[path = "xdt/mission_view.rs"]
mod mission_view;
#[path = "xdt/mission_skin.rs"]
mod mission_skin;
#[path = "xdt/mission_search.rs"]
mod mission_search;
#[path = "xdt/mission_pointer.rs"]
mod mission_pointer;
#[path = "xdt/mission_context.rs"]
mod mission_context;
#[path = "xdt/mission_dependencies.rs"]
mod mission_dependencies;
#[path = "xdt/mission_scene.rs"]
mod mission_scene;
#[path = "xdt/mission_text.rs"]
mod mission_text;
#[path = "xdt/mission_events.rs"]
mod mission_events;
#[path = "xdt/mission_visibility.rs"]
mod mission_visibility;
#[path = "xdt/mail_fields.rs"]
mod mail_fields;
#[path = "xdt/string_context.rs"]
mod string_context;
#[path = "xdt/mission_publish.rs"]
mod mission_publish;
#[path = "xdt/working_copy.rs"]
mod working_copy;
#[path = "xdt/npc_templates.rs"]
mod npc_templates;
#[path="xdt/npc_inspector.rs"]
mod npc_inspector;
#[path = "xdt/mission_server.rs"]
mod mission_server;
#[cfg(test)]
#[path = "xdt/mission_tests.rs"]
mod mission_tests;

#[path = "xdt/exchange.rs"]
mod exchange;
#[path = "xdt/forms.rs"]
mod forms;
#[path = "xdt/mission_creation.rs"]
mod mission_creation;
#[path = "xdt/mission_reward.rs"]
mod mission_reward;
#[path = "xdt/input.rs"]
mod input;
#[path = "xdt/model.rs"]
mod model;
#[path = "xdt/sections.rs"]
mod sections;
#[path = "xdt/sorting.rs"]
mod sorting;
use sorting::SortKey;
#[path = "xdt/mission_journal.rs"]
mod mission_journal;
#[path = "xdt/mission_preview.rs"]
mod mission_preview;
#[path = "xdt/mission_stage_card.rs"]
mod mission_stage_card;
#[path = "xdt/presentation.rs"]
mod presentation;
#[path = "xdt/relations.rs"]
mod relations;
#[path = "xdt/schema.rs"]
mod schema;
#[cfg(test)]
#[path = "xdt/tests.rs"]
mod tests;
#[path = "xdt/view.rs"]
mod view;
use model::*;
#[derive(SystemSet,Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub(super) struct XdtInput;

pub(super) struct XdtPlugin;
impl Plugin for XdtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<mission_skin::MissionSkin>();
        app.add_systems(
            Update,
            (
                npc_inspector::select.after(handle_editor_buttons),
                input::buttons.in_set(XdtInput),
                sync_section,
                mission_server::poll,
                mission_pointer::pointer,
                input::keyboard,
                input::drag,
                input::scroll,
                input::clamp_viewport,
                input::remember_field_scroll,
                view::draw,
                npc_inspector::draw,
                mission_canvas::redraw,
                view::hover,
            )
                .chain()
                .after(handle_editor_buttons)
                .before(LocalizationSet::Apply),
        );
        app.add_systems(
            Update,
            refresh_viewers
                .after(input::keyboard)
                .before(bind_editor_ui),
        );
        app.add_systems(
            PostUpdate,
            (input::reveal_caret, mission_context::place).after(bevy::ui::UiSystems::Layout),
        );
    }
}

#[derive(Resource)]
pub(super) struct XdtEditor {
    path: PathBuf,
    publication: Option<mission_publish::Publication>,
    server_job: Option<mission_server::SelectionJob>,
    base: Value,
    document: Value,
    tables: Vec<Table>,
    table: usize,
    table_tab: Option<sections::Navigation>,
    mission_tab: Option<sections::Navigation>,
    row: Option<usize>,
    columns: Vec<String>,
    filtered: Vec<usize>,
    table_search: String,
    search: String,
    column_search: String,
    sort: Option<(SortKey, bool)>,
    offset: usize,
    column_offset: usize,
    table_offset: usize,
    focus: Option<Focus>,
    edit: String,
    cursor: usize,
    anchor: usize,
    status: String,
    undo: Vec<Change>,
    redo: Vec<Change>,
    history: Vec<(usize, usize)>,
    revision: u64,
    delete_confirm: bool,
    reload_confirm: bool,
    links: Vec<Link>,
    index: BTreeMap<(String, String), Vec<(usize, usize)>>,
    primary_domains: BTreeSet<(usize, String)>,
    file_job: Option<exchange::FileJob>,
    references: Vec<relations::Reference>,
    text_undo: Vec<(String, usize, usize)>,
    text_redo: Vec<(String, usize, usize)>,
    reload_models: bool,
    draft: Option<forms::Draft>,
    advanced: bool,
    reference_search: String,
    reference_offset: usize,
    summaries: Vec<Vec<String>>,
    picker_field: Option<String>,
    relations_open: bool,
    all_tables: bool,
    block_overrides: BTreeMap<(usize, Option<usize>, bool, String), bool>,
    array_slot: Option<usize>,
    mission_filter: Option<i64>,
    graph_open: bool,
    graph_stages: bool,
    graph_zoom: f32,
    graph_all: bool,
    workspace: mission_workspace::Workspace,
    saved_work: Option<working_copy::Snapshot>,
}

fn sync_section(editor: Res<XdtEditor>, mut state: ResMut<EditorState>) {
    if state.xdt_open {
        let missions = editor.mission_workspace_table().is_some();
        if state.missions_open != missions { state.missions_open = missions; }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Focus {
    Tables,
    Search,
    Columns,
    Cell(String),
    Row,
    Draft(String),
    ReferenceSearch,
    NewName,
    Locale(usize),
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct ScrollRegion(u8);
#[derive(Component, Clone)]
enum Action {
    MobLevels,
    MobLevel(i64),
    MobStats(usize),
    Mission(mission_workspace::Command),
    Table(usize),
    Row(usize),
    Cell(usize, String),
    Field(Focus),
    Sort(SortKey),
    Save,
    Rewrite,
    Undo,
    Redo,
    Add,
    AddPlaceholder,
    Duplicate,
    Delete,
    Raw,
    Apply,
    Cancel,
    Back,
    Link(usize, usize),
    Reload,
    Export,
    Import,
    Advanced,
    Required(String),
    Create,
    PickReference(String, usize, usize),
    Page(bool),
    CancelEdit,
    Relations(bool),
    Choice(String, i64),
    AllTables,
    OwnName(String),
    ExistingName,
    Block(String, bool),
    EditSharedText(String),
    CopyText(String),
    ArrayCell(String, usize),
    ClearReference(String),
    AddTask,
    MissionStages,
    Graph,
    GraphAll,
    GraphMode(bool),
    GraphZoom(f32),
    GraphNode(usize),
    GraphAdd(String),
    GraphRemove(usize, String, usize),
    Appearance,
    AddHnpc,
}
#[derive(Component)]
struct EditText;
#[derive(Component)]
struct ActiveText;
#[derive(Component)]
struct Track(u8);

impl XdtEditor {
    pub(super) fn object_npc_types(&self) -> BTreeSet<i64> {
        let rows = |suffix: &str| {
            self.tables.iter().find(|table| table.label.ends_with(suffix))
                .and_then(|table| self.document.pointer(&table.pointer))
                .and_then(Value::as_array)
        };
        let Some(meshes) = rows("/m_pNpcMeshData") else { return BTreeSet::new(); };
        rows("/m_pNpcTable/m_pNpcData").into_iter().flatten().filter_map(|row| {
            let mesh = usize::try_from(row["m_iMesh"].as_i64()?).ok()?;
            let name = meshes.get(mesh)?["m_pstrMMeshModelString"].as_str()?;
            name.eq_ignore_ascii_case("ObjectNPC1").then(|| row["m_iNpcNumber"].as_i64()).flatten()
        }).collect()
    }
    pub(super) fn npc_map_icons(&self) -> BTreeMap<i64, i32> {
        self.tables.iter().find(|t|t.label.ends_with("/m_pNpcTable/m_pNpcData"))
            .and_then(|t|self.document.pointer(&t.pointer)).and_then(Value::as_array)
            .into_iter().flatten().filter_map(|row| {
                let npc=row["m_iNpcNumber"].as_i64()?;
                let icon=i32::try_from(row["m_iMapIcon"].as_i64()?).ok()?;
                (npc>0).then_some((npc,icon))
            }).collect()
    }
    pub(super) fn has_npc_type(&self, id: i64) -> bool {
        self.tables.iter().find(|t|t.label.ends_with("/m_pNpcTable/m_pNpcData"))
            .and_then(|t|self.document.pointer(&t.pointer)).and_then(Value::as_array)
            .is_some_and(|rows|rows.iter().any(|r|r["m_iNpcNumber"].as_i64()==Some(id)))
    }
    pub(super) fn server_folder(&self) -> Option<PathBuf> {
        self.publication.as_ref().and_then(|p| p.path.parent()).map(Path::to_path_buf)
    }
    pub(super) fn open(root: PathBuf) -> Self {
        let path = root.join(ffone_client::assets::TABLE_SET_PATH);
        let loaded = read(&path);
        let status = loaded.as_ref().err().cloned().unwrap_or_default();
        let document = loaded.unwrap_or(Value::Null);
        let mut editor = Self {
            publication: mission_publish::Publication::discover(&root),
            server_job: None,
            path,
            base: document.clone(),
            document,
            tables: Vec::new(),
            table: 0,
            table_tab: None,
            mission_tab: None,
            row: None,
            columns: Vec::new(),
            filtered: Vec::new(),
            table_search: String::new(),
            search: String::new(),
            column_search: String::new(),
            sort: None,
            offset: 0,
            column_offset: 0,
            table_offset: 0,
            focus: None,
            edit: String::new(),
            cursor: 0,
            anchor: 0,
            status,
            undo: Vec::new(),
            redo: Vec::new(),
            history: Vec::new(),
            revision: 1,
            delete_confirm: false,
            reload_confirm: false,
            links: Vec::new(),
            index: BTreeMap::new(),
            primary_domains: BTreeSet::new(),
            file_job: None,
            references: Vec::new(),
            text_undo: Vec::new(),
            text_redo: Vec::new(),
            reload_models: false,
            draft: None,
            advanced: false,
            reference_search: String::new(),
            reference_offset: 0,
            summaries: Vec::new(),
            picker_field: None,
            relations_open: false,
            all_tables: false,
            block_overrides: BTreeMap::new(),
            array_slot: None,
            mission_filter: None,
            graph_open: false,
            graph_stages: true,
            graph_zoom: 1.,
            graph_all: false,
            workspace: mission_workspace::Workspace::load(&root),
            saved_work: None,
        };
        if !editor.document.is_null() {
            if let Err(error) = editor.restore_work() { editor.status = error; }
        }
        editor.discover();
        editor
    }
    fn dirty(&self) -> bool {
        self.saved_work.as_ref().map_or_else(|| self.unpublished(), |saved|
            self.document != saved.document || self.workspace.locale_drafts != saved.locale_drafts)
    }
    pub(super) fn hnpc_value(&self) -> Option<&Value> {
        if !self.tables.get(self.table)?.label.ends_with("/m_pNpcTable/m_pNpcData") { return None; }
        self.draft.as_ref().map(|d| &d.value).or_else(|| self.row.and_then(|r| self.rows().get(r)))
    }
    pub(super) fn with_graph(mut self, open:bool)->Self {self.graph_open=open && self.mission_table();self}
    pub(super) fn apply_hnpc(&mut self, index: usize) -> Result<(), String> {
        if self.hnpc_value().is_none() { return Err("Select an NPC".into()); }
        if let Some(draft) = &mut self.draft {
            draft.value["m_iHNpc"] = Value::from(1);
            draft.value["m_iHNpcNum"] = Value::from(index);
        } else {
            let row = self.row.ok_or("Select an NPC")?;
            let mut rows = self.rows().to_vec();
            rows[row]["m_iHNpc"] = Value::from(1);
            rows[row]["m_iHNpcNum"] = Value::from(index);
            self.change(rows)?;
        }
        self.revision += 1;
        Ok(())
    }
    pub(super) fn with_selection(
        mut self,
        table: Option<&str>,
        row: Option<usize>,
        search: &str,
    ) -> Self {
        if let Some(table) = table {
            let matches: Vec<_> = self
                .tables
                .iter()
                .enumerate()
                .filter(|(_, t)| t.label == table || t.label.ends_with(&format!("/{table}")))
                .map(|(i, _)| i)
                .collect();
            if matches.len() == 1 {
                self.select_table(matches[0]);
            } else {
                self.status = format!(
                    "Table selector needs one match, found {}: {table}",
                    matches.len()
                );
            }
        }
        self.row = row.filter(|r| *r < self.rows().len()).or(self.row);
        if self.mission_table() {
            self.workspace.enabled=true;
            self.workspace.enabled=true;
            if let Some(id)=self.row.and_then(|r|self.rows()[r]["m_iHMissionID"].as_i64()) {self.workspace.expanded.insert(id);}
        }
        self.search = search.to_owned();
        self.refresh();
        if let Some(row) = self.row {
            self.offset = self.filtered.iter().position(|r| *r == row).unwrap_or(0);
        }
        self
    }
    pub(super) fn capture_report(&self) -> Value {
        serde_json::json!({"tables":self.tables.len(),"table":self.tables.get(self.table).map(|t| &t.label),
            "rows":self.rows().len(),"filtered":self.filtered.len(),"row":self.row,
            "links":self.links.len(),"confirmed_links":self.links.iter().filter(|l| l.confirmed).count(),
            "dirty":self.dirty(),"status":self.status})
    }
    pub(super) fn save_on_close(&mut self) -> bool {
        if self.draft.is_some() {
            self.status = "Finish or cancel the new record first".into();
            self.revision += 1;
            return false;
        }
        if !self.apply() {
            return false;
        }
        if !self.dirty() {
            return true;
        }
        match self.save() {
            Ok(()) => true,
            Err(error) => {
                self.status = error;
                self.revision += 1;
                false
            }
        }
    }
    fn rows(&self) -> &[Value] {
        self.tables
            .get(self.table)
            .and_then(|t| self.document.pointer(&t.pointer))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    fn refresh(&mut self) {
        self.row = self.row.filter(|i| {
            *i < self.rows().len()
                || (*i == self.rows().len() && matches!(self.focus, Some(Focus::Row)))
        });
        let rows = self.rows();
        self.columns = rows
            .iter()
            .filter_map(Value::as_object)
            .flat_map(|r| r.keys().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        self.columns.sort_by_key(|field| {
            let rank = if matches!(
                field.as_str(),
                "m_iNpcNumber" | "m_iNanoNumber" | "m_iItemNumber" | "m_iHTaskID" | "m_iID" | "id"
            ) {
                0
            } else if field.to_lowercase().contains("name") {
                1
            } else if id_field(field) {
                2
            } else {
                3
            };
            (rank, field.clone())
        });
        if self.mission_filter.is_some_and(|id| !self.rows().iter().any(|r| r.get("m_iHMissionID").and_then(Value::as_i64) == Some(id))) { self.mission_filter = None; }
        let needle = self.search.to_lowercase();
        let mut filtered: Vec<_> = self
            .rows()
            .iter()
            .enumerate()
            .filter(|(i, row)| {
                if self.mission_filter.is_some_and(|id| row.get("m_iHMissionID").and_then(Value::as_i64) != Some(id)) { return false; }
                needle.is_empty()
                    || self.matches_search(self.table, *i, &needle)
                    || self
                        .summaries
                        .get(self.table)
                        .and_then(|s| s.get(*i))
                        .is_some_and(|s| s.to_lowercase().contains(&needle))
            })
            .map(|(i, _)| i)
            .collect();
        if let Some((field, descending)) = &self.sort {
            filtered.sort_by(|a, b| {
                let order = match field {
                    SortKey::Index => a.cmp(b),
                    SortKey::Field(field) => compare(&self.rows()[*a][field], &self.rows()[*b][field]),
                };
                if *descending { order.reverse() } else { order }
            });
        }
        self.filtered = filtered;
        self.offset = self.offset.min(self.filtered.len().saturating_sub(1));
        self.column_offset = self.column_offset.min(self.columns.len().saturating_sub(1));
        self.rebuild_links();
        self.revision += 1;
    }
    fn select_table(&mut self, table: usize) {
        self.mission_filter = None;
        self.table = table;
        self.row = None;
        self.offset = 0;
        self.column_offset = 0;
        self.search.clear();
        self.column_search.clear();
        self.sort = None;
        self.focus = None;
        self.picker_field = None;
        self.delete_confirm = false;
        self.refresh();
        if self.mission_table() {
            self.row=self.rows().iter().position(|v|v["m_iHTaskID"].as_i64().is_some_and(|id|id>0));
            if let Some(id)=self.row.and_then(|r|self.rows()[r]["m_iHMissionID"].as_i64()) {self.workspace.expanded.insert(id);}
        }
    }
    fn begin(&mut self, focus: Focus) {
        if focus != Focus::ReferenceSearch { self.array_slot = None; }
        if let Focus::Cell(field) | Focus::Draft(field) = &focus {
            self.picker_field = Some(field.clone());
        } else if focus != Focus::ReferenceSearch {
            self.picker_field = None;
        }
        self.edit = match &focus {
            Focus::Locale(index)=>self.workspace.quick_text.as_ref().map(|d|d.values[*index].clone()).unwrap_or_default(),
            Focus::Tables => self.table_search.clone(),
            Focus::Search => self.search.clone(),
            Focus::Columns => self.column_search.clone(),
            Focus::ReferenceSearch => self.reference_search.clone(),
            Focus::NewName => self
                .draft
                .as_ref()
                .and_then(|d| d.name.as_ref())
                .map(|n| n.text.clone())
                .unwrap_or_default(),
            Focus::Draft(field) => self
                .draft
                .as_ref()
                .and_then(|d| d.value.get(field))
                .map(display)
                .unwrap_or_default(),
            Focus::Cell(field) => self
                .row
                .and_then(|i| self.rows().get(i))
                .and_then(|row| row.get(field))
                .map(display)
                .unwrap_or_default(),
            Focus::Row => self
                .row
                .and_then(|i| self.rows().get(i))
                .map(|row| serde_json::to_string_pretty(row).unwrap())
                .unwrap_or_default(),
        };
        self.cursor = self.edit.len();
        self.anchor = 0;
        self.focus = Some(focus);
        self.text_undo.clear();
        self.text_redo.clear();
        if matches!(self.focus, Some(Focus::Cell(_) | Focus::Draft(_))) {
            self.reference_search.clear();
            self.reference_offset = 0;
            if self.picker_field.as_deref().is_some_and(|field|self.reference_target(field).is_some() && self.field_value(field).is_some_and(Value::is_number)) {
                self.focus=Some(Focus::ReferenceSearch);
                self.edit.clear();self.cursor=0;self.anchor=0;
            }
        }
        self.revision += 1;
    }
    fn replace(&mut self, text: &str) {
        self.text_undo
            .push((self.edit.clone(), self.cursor, self.anchor));
        self.text_redo.clear();
        let start = self.cursor.min(self.anchor);
        let end = self.cursor.max(self.anchor);
        self.edit.replace_range(start..end, text);
        self.cursor = start + text.len();
        self.anchor = self.cursor;
        match self.focus {
            Some(Focus::Tables) => {
                self.table_search = self.edit.clone();
                self.table_offset = 0;
            }
            Some(Focus::Search) => {
                self.search = self.edit.clone();
                self.workspace.catalog_offset=0;
                self.offset = 0;
                self.refresh();
            }
            Some(Focus::Columns) => {
                self.column_search = self.edit.clone();
                self.column_offset = 0;
            }
            Some(Focus::ReferenceSearch) => {
                self.reference_search = self.edit.clone();
                self.reference_offset = 0;
            }
            _ => {}
        }
        self.revision += 1;
    }
    fn tab_cell(&mut self, back: bool, localization: Option<(&Localization, &Language)>) {
        let Some(Focus::Cell(field)) = self.focus.clone() else {
            return;
        };
        let visible = self.row.and_then(|r| self.rows().get(r))
            .map(|value| self.authoring_fields(value)).unwrap_or_default();
        let fields: Vec<_> = visible
            .iter()
            .filter(|f| {
                !self.advanced || f.to_lowercase()
                    .contains(&self.column_search.to_lowercase())
                    || localization.is_some_and(|(l, lang)| {
                        schema::field_name(f, l, lang)
                            .to_lowercase()
                            .contains(&self.column_search.to_lowercase())
                    })
            })
            .cloned()
            .collect();
        if !self.apply() || fields.is_empty() || self.filtered.is_empty() {
            return;
        }
        let column = fields.iter().position(|f| f == &field).unwrap_or(0);
        let total = fields.len() * self.filtered.len();
        let current = self
            .row
            .and_then(|r| self.filtered.iter().position(|i| *i == r))
            .map(|r| r * fields.len() + column);
        let mut position = current
            .map(|p| {
                if back {
                    p.saturating_sub(1)
                } else {
                    (p + 1).min(total - 1)
                }
            })
            .unwrap_or(if back { total - 1 } else { 0 });
        loop {
            let row = position / fields.len();
            let column = position % fields.len();
            let source = self.filtered[row];
            if self
                .rows()
                .get(source)
                .and_then(|r| r.get(&fields[column]))
                .is_some()
            {
                self.row = Some(source);
                self.offset = row;
                self.column_offset = column;
                self.rebuild_links();
                self.begin(Focus::Cell(fields[column].clone()));
                return;
            }
            let next = if back {
                position.saturating_sub(1)
            } else {
                (position + 1).min(total - 1)
            };
            if next == position {
                return;
            }
            position = next;
        }
    }
    fn apply(&mut self) -> bool {
        let Some(focus) = self.focus.clone() else {
            return true;
        };
        if let Focus::Locale(index)=focus{return self.apply_locale_text(index);}
        if let Some(slot) = self.array_slot {
            if let Focus::Cell(field) | Focus::Draft(field) = &focus {
                return self.apply_element(field, slot);
            }
        }
        if focus == Focus::NewName {
            if let Some(name) = self.draft.as_mut().and_then(|d| d.name.as_mut()) {
                name.text = self.edit.clone();
            }
            self.focus = None;
            self.revision += 1;
            return true;
        }
        if let Focus::Draft(field) = &focus {
            return self.apply_draft_field(field.clone());
        }
        if matches!(
            focus,
            Focus::Tables | Focus::Search | Focus::Columns | Focus::ReferenceSearch
        ) {
            self.focus = None;
            return true;
        }
        let Some(row) = self.row else { return true };
        let result = (|| {
            let mut rows = self.rows().to_vec();
            match focus {
                Focus::Cell(field) => {
                    let old = rows
                        .get(row)
                        .and_then(|row| row.get(&field))
                        .ok_or("Missing field")?;
                    let value = parse_cell(old, &self.edit)
                        .map_err(|_| schema::task_error(&rows[row], format!("{field}: {}", schema::type_error(old))))?;
                    if value != *old {
                        if let Some(error) =
                            schema::invalid(&self.tables[self.table].label, &field, &value)
                        {
                            return Err(schema::task_error(&rows[row], format!("{field}: {error}")));
                        }
                    }
                    rows[row][field] = value;
                }
                Focus::Row => {
                    if row > rows.len() {
                        return Err("Row no longer exists".into());
                    }
                    let value: Value =
                        serde_json::from_str(&self.edit).map_err(|e| e.to_string())?;
                    if !value.is_object() {
                        return Err("Row must be a JSON object".into());
                    }
                    if row == rows.len() {
                        rows.push(value);
                    } else {
                        rows[row] = value;
                    }
                }
                _ => {}
            }
            self.change(rows)
        })();
        match result {
            Ok(()) => {
                self.focus = None;
                self.status.clear();
                self.revision += 1;
                true
            }
            Err(error) => {
                self.status = error;
                self.revision += 1;
                false
            }
        }
    }
    fn change(&mut self, mut rows: Vec<Value>) -> Result<(), String> {
        if self.mission_table() { mission_visibility::propagate(self.rows(), &mut rows)?; }
        let Some(table) = self.tables.get(self.table) else {
            return Err("No table".into());
        };
        validate_rows(self.rows(), &rows)?;
        if self.mission_table() { mission_graph::validate(self.rows(), &rows)?; }
        for (i, row) in rows.iter().enumerate() {
            let old = self.rows().get(i);
            for field in schema::required(&table.label) {
                if row.get(*field).is_none()
                    && (old.is_none() || old.is_some_and(|r| r.get(*field).is_some()))
                {
                    return Err(schema::task_error(row, format!("Required parameters: {field}")));
                }
            }
            for (field, value) in row.as_object().into_iter().flatten() {
                if old.and_then(|r| r.get(field)) != Some(value) {
                    if let Some(error) = schema::invalid(&table.label, field, value) {
                        return Err(schema::task_error(row, format!("{field}: {error}")));
                    }
                }
            }
        }
        let mut before = Value::Array(self.rows().to_vec());
        let mut after = Value::Array(rows);
        if before == after {
            return Ok(());
        }
        let mut pointer = table.pointer.clone();
        if before.as_array().unwrap().len() == after.as_array().unwrap().len() {
            let changed: Vec<_> = before
                .as_array()
                .unwrap()
                .iter()
                .zip(after.as_array().unwrap())
                .enumerate()
                .filter(|(_, (b, a))| b != a)
                .map(|(i, _)| i)
                .take(2)
                .collect();
            if changed.len() == 1 {
                let i = changed[0];
                pointer = format!("{pointer}/{i}");
                before = before[i].take();
                after = after[i].take();
            }
        }
        *self.document.pointer_mut(&pointer).ok_or("Missing table")? = after.clone();
        self.undo.push(Change {
            pointer,
            before,
            after,
        });
        self.redo.clear();
        self.reindex();
        self.refresh();
        Ok(())
    }
    fn undo(&mut self, redo: bool) {
        let mission_selection=self.mission_table().then(||self.row.and_then(|r|self.rows().get(r))
            .and_then(|v|Some((v["m_iHTaskID"].as_i64()?,v["m_iHMissionID"].as_i64()?)))).flatten();
        let change = if redo {
            self.redo.pop()
        } else {
            self.undo.pop()
        };
        if let Some(change) = change {
            if change.pointer=="@mission-localized" {
                let value=if redo {&change.after}else{&change.before};
                self.document=value["document"].clone();
                let locales:BTreeMap<String,[Option<String>;2]>=serde_json::from_value(value["locales"].clone()).unwrap_or_default();
                self.workspace.locale_drafts.extend(locales);
                if redo{self.undo.push(change);}else{self.redo.push(change);}
                self.row=self.row.filter(|r|*r<self.rows().len());
                self.restore_mission_selection(mission_selection);
                self.reindex();self.refresh();return;
            }
            if change.pointer=="@mission-layout" {
                self.workspace.positions=serde_json::from_value(if redo {change.after.clone()}else{change.before.clone()}).unwrap_or_default();
                if redo {self.undo.push(change);}else{self.redo.push(change);}
                if let Err(error)=self.save_layout(){self.status=error;}
                self.revision+=1;
                return;
            }
            *self.document.pointer_mut(&change.pointer).unwrap() = if redo {
                change.after.clone()
            } else {
                change.before.clone()
            };
            if redo {
                self.undo.push(change);
            } else {
                self.redo.push(change);
            }
            self.row = self.row.filter(|i| *i < self.rows().len());
            self.restore_mission_selection(mission_selection);
            self.reindex();
            self.refresh();
        }
    }
    fn navigate(&mut self, table: usize, row: usize) {
        if let Some(previous) = self.row {
            self.history.push((self.table, previous));
        }
        self.select_table(table);
        self.row = Some(row);
        self.offset = self.filtered.iter().position(|i| *i == row).unwrap_or(0);
        self.rebuild_links();
        self.revision += 1;
    }
    fn rewrite(&mut self) -> Result<(), String> {
        self.enable_edited_nanocom();
        self.sync_mission_text_links();
        // Preserve the current work even if publication validation or writing fails.
        self.save()?;
        let locales=self.prepare_locales()?;
        self.validate_document_change(&self.base, &self.document)?;
        let disk = read(&self.path)?;
        let merged = merge_document(&self.base, &self.document, &disk)?;
        self.validate_document_change(&self.base, &merged)?;
        mission_publish::validate_tasks(&self.base, &merged)?;
        let waypoint = self.prepare_waypoints(&merged)?;
        let published = ffone_client::xdt::into_server_document(merged.clone())?;
        self.publish_files(&published, &locales, waypoint)?;
        if merged != self.document {
            self.undo.clear();
            self.redo.clear();
        }
        self.base = merged.clone();
        self.document = merged;
        for (index, (_, value)) in locales.into_iter().enumerate() {
            self.workspace.locale_base[index] = value["entries"].as_object().unwrap().iter()
                .filter_map(|(k,v)| v.as_str().map(|v| (k.clone(), v.into()))).collect();
        }
        self.workspace.locale_drafts.clear();
        self.save()?;
        self.discover();
        self.reload_models = true;
        self.status = if self.publication.is_some() { "Saved to client and server; restart the server" } else { "Saved; server TableData was not found" }.into();
        self.revision += 1;
        Ok(())
    }
}

fn refresh_viewers(
    mut commands: Commands,
    mut editor: ResMut<XdtEditor>,
    mut state: ResMut<EditorState>,
    mut catalog: ResMut<EditorCatalog>,
    mut equipment: ResMut<EquipmentLibrary>,
    mut preview: ResMut<ModelPreview>,
    asset_server: Res<AssetServer>,
    fonts: Option<Res<EditorFonts>>,
    bodies: Query<(Entity, &Children), With<strings::ModelEditorBody>>,
) {
    if !editor.reload_models || state.xdt_open || state.strings_open {
        return;
    }
    let Some(fonts) = fonts else { return };
    editor.reload_models = false;
    let result = (|| {
        let root = editor
            .path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or("Missing asset root")?;
        let locator = AssetLocator::open(root).map_err(|e| e.to_string())?;
        let mut updated = EditorCatalog::open(&locator)?;
        let replacement = EquipmentLibrary::open(root, &mut updated)?;
        let positions: BTreeMap<_, _> = updated
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (e.semantic_id.clone(), i))
            .collect();
        let selected = positions
            .get(&catalog.entries[state.selected].semantic_id)
            .copied()
            .or_else(|| updated.entries.iter().position(|e| e.kind == state.kind))
            .unwrap_or(0);
        for (kind, (index, _)) in &mut state.viewer_tabs {
            *index = catalog
                .entries
                .get(*index)
                .and_then(|e| positions.get(&e.semantic_id))
                .copied()
                .or_else(|| updated.entries.iter().position(|e| e.kind == *kind))
                .unwrap_or(selected);
        }
        equipment.replace_tables(replacement);
        *catalog = updated;
        state.selected = selected;
        state.choose_default_clip(&catalog);
        preview.current_index = None;
        if let Some(hnpc) = catalog.hnpc.as_ref() {
            commands.insert_resource(hnpc.rig_catalog().clone());
        }
        for (body, children) in &bodies {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
            commands.entity(body).with_children(|body| {
                spawn_catalog_panel(body, &fonts, &asset_server, &catalog);
                spawn_viewport_overlay(body, &fonts);
                spawn_inspector_panel(body, &fonts);
            });
        }
        Ok::<_, String>(())
    })();
    if let Err(error) = result {
        editor.status = format!("Saved; model catalog could not reload: {error}");
        editor.revision += 1;
    }
}
