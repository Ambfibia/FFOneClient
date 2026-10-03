//! Persistent presentation state and commands for the mission authoring screen.
use super::*;

#[derive(Default)]
pub(super) struct Workspace {
    pub canvas_revision: u64,
    pub positions: BTreeMap<String, [f32; 2]>,
    pub pan: Vec2,
    pub selected: BTreeSet<usize>,
    pub expanded: BTreeSet<i64>,
    pub collapsed_stages: BTreeSet<i64>,
    pub catalog_width: f32,
    pub inspector_width: f32,
    pub catalog_hidden: bool,
    pub enabled: bool,
    pub catalog_offset: usize,
    pub view_request: Option<bool>,
    pub marquee: Option<(Vec2, Vec2)>,
    pub pending_link: Option<(usize, String)>,
    pub selected_edge: Option<(usize, String, usize)>,
    pub context: Option<(Vec2, Context)>,
    pub quick: Option<QuickOrigin>,
    pub quick_text: Option<mission_text::TextDraft>,
    pub quick_existing: Option<usize>,
    pub creation: Vec<mission_creation::Frame>,
    pub locale_drafts: BTreeMap<String, [Option<String>; 2]>,
    pub locale_base: [BTreeMap<String, String>; 2],
    pub help: Option<String>,
    pub diagnostics: Vec<(usize, String, String)>,
    pub layout_dirty: bool,
    pub index: Vec<Vec<String>>,
    pub locale_text: BTreeMap<String, String>,
}

#[derive(Clone)]
pub(super) struct QuickOrigin {
    pub table: usize,
    pub row: usize,
    pub field: String,
    pub slot: Option<usize>,
}

#[derive(Clone)]
pub(super) enum Context {
    Canvas,
    Node(usize),
    Mission(usize),
    Edge(usize, String, usize),
    Field(String),
}

#[derive(Clone, Debug)]
pub(super) enum Command {
    NewMission,
    NewStage,
    Duplicate,
    ToggleMission(i64),
    ToggleStage(i64),
    Select(usize),
    ToggleCatalog,
    Tables,
    CatalogPage(bool),
    Collapse,
    Validate,
    Fit,
    Center,
    Arrange,
    Zoom(f32),
    ResetZoom,
    Connect(usize, String),
    Edge(usize, String, usize),
    RemoveEdge,
    InsertStage,
    Delete,
    ConfirmDelete,
    QuickCreate(String),
    QuickPlaceholder(String),
    EditJournal(String),
    InspectField(usize, String),
    CopyId,
    OpenMission(usize),
    CopyMissionId(usize),
    Help(String),
}

impl Workspace {
    pub fn load(root: &Path) -> Self {
        let mut state = Self {
            catalog_width: 270.,
            inspector_width: 355.,
            enabled: true,
            view_request: Some(true),
            ..default()
        };
        if let Ok(bytes) = fs::read(root.join("../../target/editor/mission-layout.json")) {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                if value["schema"] == "ffone.mission-layout.v1" {
                    state.positions =
                        serde_json::from_value(value["positions"].clone()).unwrap_or_default();
                    state.collapsed_stages = serde_json::from_value(value["collapsed_stages"].clone()).unwrap_or_default();
                    state
                        .positions
                        .retain(|_, p| p.iter().all(|n| n.is_finite() && n.abs() < 1e6));
                    state.catalog_width = value["catalog_width"].as_f64().unwrap_or(235.) as f32;
                    state.inspector_width =
                        value["inspector_width"].as_f64().unwrap_or(380.) as f32;
                }
            }
        }
        for (index, language) in ["en", "ru"].into_iter().enumerate() {
            if let Ok(bytes) = fs::read(root.join(format!("localization/{language}.json"))) {
                if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                    for (key, value) in value["entries"].as_object().into_iter().flatten() {
                        if let Some(text) = value.as_str() {
                            state.locale_base[index].insert(key.clone(), text.into());
                        }
                        if key.starts_with("content.") {
                            if let Some(text) = value.as_str() {
                                state
                                    .locale_text
                                    .entry(key.clone())
                                    .or_default()
                                    .push_str(&format!(" {text}"));
                            }
                        }
                    }
                }
            }
        }
        state
    }
}

impl XdtEditor {
    pub(super) fn dismiss_mission_context(&mut self) {
        self.workspace.context = None;
        self.workspace.help = None;
        self.workspace.pending_link = None;
        self.delete_confirm = false;
        self.revision += 1;
    }
    pub(super) fn restore_mission_selection(&mut self, previous: Option<(i64, i64)>) {
        let Some((task, group)) = previous else {
            return;
        };
        let exact = self
            .rows()
            .iter()
            .position(|v| v["m_iHTaskID"].as_i64() == Some(task));
        self.row = exact.or_else(|| {
            self.rows()
                .iter()
                .position(|v| v["m_iHMissionID"].as_i64() == Some(group))
        });
        self.workspace.selected = self.row.into_iter().collect();
        if exact.is_none() {
            self.workspace.view_request = Some(true);
        }
    }
    pub(super) fn mission_workspace_table(&self) -> Option<usize> {
        if !self.workspace.enabled {
            return None;
        }
        let table = self
            .workspace
            .creation
            .first()
            .map_or(self.table, |frame| frame.table);
        self.tables
            .get(table)
            .filter(|t| t.label.ends_with("/m_pMissionTable/m_pMissionData"))
            .map(|_| table)
    }
    pub(super) fn mission_rows(&self) -> &[Value] {
        self.mission_workspace_table()
            .and_then(|t| self.document.pointer(&self.tables[t].pointer))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub(super) fn mission_row(&self) -> Option<usize> {
        self.workspace
            .creation
            .first()
            .and_then(|frame| frame.row)
            .or(self.row)
    }
    pub(super) fn layout_key(&self, id: i64) -> String {
        format!(
            "{}:{id}",
            if self.graph_stages { "task" } else { "mission" }
        )
    }
    pub(super) fn save_layout(&mut self) -> Result<(), String> {
        let path = self
            .path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or("Missing asset root")?
            .join("../../target/editor/mission-layout.json");
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let value = serde_json::json!({"schema":"ffone.mission-layout.v1", "positions":self.workspace.positions,
            "collapsed_stages":self.workspace.collapsed_stages,
            "catalog_width":self.workspace.catalog_width,"inspector_width":self.workspace.inspector_width});
        let mut temp =
            tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut temp, &value).map_err(|e| e.to_string())?;
        temp.persist(path).map_err(|e| e.to_string())?;
        self.workspace.layout_dirty = false;
        Ok(())
    }
    pub(super) fn cancel_quick(&mut self) {
        self.finish_creation(None);
    }
    pub(super) fn start_quick(&mut self, field: &str) -> Result<(), String> {
        self.start_quick_kind(field, false)
    }
    pub(super) fn quick_create_command(&self, field: &str) -> Command {
        if self.reference_target(field).is_some_and(|(table, _)| self.tables[table].label.ends_with("/m_pNpcTable/m_pNpcData")) {
            Command::QuickPlaceholder(field.into())
        } else { Command::QuickCreate(field.into()) }
    }
    pub(super) fn start_quick_kind(&mut self, field: &str, placeholder: bool) -> Result<(), String> {
        let row = self.row.unwrap_or(0);
        let (table, _) = self
            .reference_target(field)
            .ok_or("Unknown reference route")?;
        if table == self.table {
            return Err("Use New stage to create mission stages".into());
        }
        let origin = QuickOrigin {
            table: self.table,
            row,
            field: field.into(),
            slot: self.array_slot,
        };
        let source = self
            .field_value(field)
            .and_then(|v| origin.slot.map_or(Some(v), |s| v.get(s)))
            .cloned();
        let (_, id) = self.reference_target(field).unwrap();
        let template = self
            .document
            .pointer(&self.tables[table].pointer)
            .and_then(Value::as_array)
            .and_then(|rows| {
                if let Some(id) = id {
                    rows.iter().position(|r| r.get(id) == source.as_ref())
                } else {
                    source
                        .as_ref()
                        .and_then(Value::as_u64)
                        .map(|i| i as usize)
                        .filter(|i| *i < rows.len())
                }
            });
        self.push_creation();
        self.select_table(table);
        self.row = template;
        self.workspace.quick = Some(origin);
        let result = if placeholder { self.start_placeholder_draft() } else { self.start_draft(false) };
        if let Err(error) = result {
            self.cancel_quick();
            return Err(error);
        }
        if self.tables[table].label.ends_with("/m_pRewardData") {
            let draft = self.draft.as_mut().unwrap();
            let id = draft.value["m_iMissionRewardID"].clone();
            draft.value = mission_fields::neutral(&draft.value);
            draft.value["m_iMissionRewardID"] = id;
        }
        if self.tables[table].label.ends_with("StringData") {
            let draft = self.draft.as_mut().unwrap();
            for (_, value) in draft.value.as_object_mut().into_iter().flatten() {
                if value.is_string() {
                    *value = Value::from("");
                }
            }
        }
        if self.draft.as_ref().is_some_and(|d| d.name.is_some())
            || self.tables[table].label.ends_with("StringData")
        {
            self.workspace.quick_text = Some(mission_text::TextDraft::default());
            self.begin(Focus::Locale(1));
            return Ok(());
        }
        if self.draft.as_ref().is_some_and(|d| d.name.is_some()) {
            self.begin(Focus::NewName);
        } else if let Some(field) = schema::required(&self.tables[table].label).first() {
            self.begin(Focus::Draft((*field).into()));
        }
        Ok(())
    }
}
