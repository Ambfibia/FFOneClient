use super::*;

pub(super) type FileStamp = (u64, Option<std::time::SystemTime>);

#[derive(Resource)]
pub(in super::super) struct StringEditor {
    pub(super) root: PathBuf,
    pub(super) en: BTreeMap<String, String>,
    pub(super) en_base: BTreeMap<String, String>,
    pub(super) russian: bool,
    pub(super) base: BTreeMap<String, String>,
    pub(super) draft: BTreeMap<String, String>,
    pub(super) selected: String,
    pub(super) search: String,
    pub(super) search_focus: bool,
    pub(super) tools: editing_tools::Tools,
    pub(super) editing: bool,
    pub(super) cursor: usize,
    pub(super) anchor: usize,
    pub(super) changed_at: Option<Instant>,
    pub(super) last_poll: Instant,
    pub(super) status: LocalizedText,
    pub(super) revision: u64,
    pub(super) undo: Vec<(bool, Vec<(String, String)>)>,
    pub(super) redo: Vec<(bool, Vec<(String, String)>)>,
    pub(super) discard_confirm: bool,
    pub(super) refresh_localization: bool,
    pub(super) file_stamps: Option<[FileStamp; 2]>,
    pub(super) filtered: Vec<String>,
    pub(super) row_indices: BTreeMap<String, usize>,
    pub(super) filter_query: String,
    pub(super) filter_dirty: bool,
    pub(super) offset: f32,
    pub(super) viewport: Vec2,
    pub(super) row_starts: Vec<f32>,
    pub(super) metrics_dirty: bool,
    pub(super) edited_metric: Option<String>,
    pub(super) rows_selected: exchange::RowSelection,
    pub(super) context_menu: Option<Vec2>,
    pub(super) file_job: Option<exchange::FileJob>,
    pub(super) exchange_directory: Option<PathBuf>,
}

impl StringEditor {
    pub(in super::super) fn open(root: PathBuf) -> Self {
        let mut editor = Self {
            root,
            en: BTreeMap::new(),
            en_base: BTreeMap::new(),
            russian: true,
            base: BTreeMap::new(),
            draft: BTreeMap::new(),
            selected: String::new(),
            search: String::new(),
            search_focus: false,
            tools: default(),
            editing: false,
            cursor: 0,
            anchor: 0,
            changed_at: None,
            last_poll: Instant::now(),
            revision: 0,
            status: LocalizedText::new("ui.editor.strings.ready", "Ready"),
            undo: Vec::new(),
            redo: Vec::new(),
            discard_confirm: false,
            refresh_localization: false,
            file_stamps: None,
            filtered: Vec::new(),
            row_indices: BTreeMap::new(),
            filter_query: String::new(),
            filter_dirty: true,
            offset: 0.,
            viewport: Vec2::ZERO,
            row_starts: Vec::new(),
            metrics_dirty: true,
            edited_metric: None,
            rows_selected: default(),
            context_menu: None,
            file_job: None,
            exchange_directory: None,
        };
        if let Err(error) = editor.sync(false) {
            editor.error(error);
        }
        editor
    }
    pub(in super::super) fn with_search(mut self, search: &str) -> Self {
        self.search = search.to_owned();
        self.tools.pending_find = !search.is_empty();
        self.tools.find_open = !search.is_empty();
        if let Some(key) = self.keys().first() {
            self.selected = key.clone();
        }
        self
    }
    pub(in super::super) fn capture_report(&self) -> Value {
        serde_json::json!({"rows": self.filtered.len(), "visible_rows": visible_range(&self.row_starts, self.offset, self.viewport.y).len(), "viewport": [self.viewport.x, self.viewport.y], "revision": self.revision, "selected_rows": self.rows_selected.keys.len(), "context_menu": self.context_menu.is_some(), "content_height": self.row_starts.last()})
    }
    pub(super) fn dirty(&self) -> bool {
        self.base != self.draft || self.en_base != self.en
    }
    pub(super) fn error(&mut self, error: String) {
        self.status = LocalizedText::new("ui.editor.strings.error", "Not saved: {error}")
            .with_arg("error", error);
        self.revision += 1;
    }
    pub(super) fn active(&self) -> &BTreeMap<String, String> {
        if self.russian { &self.draft } else { &self.en }
    }
    pub(super) fn clamp_selection(&mut self) {
        let text = self
            .active()
            .get(&self.selected)
            .map(String::as_str)
            .unwrap_or("");
        let boundary = |mut i: usize| {
            i = i.min(text.len());
            while !text.is_char_boundary(i) {
                i -= 1;
            }
            i
        };
        let cursor = boundary(self.cursor);
        let anchor = boundary(self.anchor);
        self.cursor = cursor;
        self.anchor = anchor;
    }
    pub(super) fn sync(&mut self, save: bool) -> Result<(), String> {
        let before_stamps = stamps(&self.root);
        let mut en = read_bundle(&self.root, "en")?;
        let mut ru = read_bundle(&self.root, "ru")?;
        let merged_en = merge(&self.en_base, &self.en, &en.entries)?;
        let mut merged = merge(&self.base, &self.draft, &ru.entries)?;
        for (key, value) in &merged_en {
            merged.entry(key.clone()).or_insert_with(|| value.clone());
        }
        if merged.keys().any(|key| !merged_en.contains_key(key)) {
            return Err("RU contains keys absent from EN".into());
        }
        if save {
            validate(&merged_en, &merged)?;
            // Stage and validate both files before replacing either. Each replacement is atomic.
            let mut staged = Vec::new();
            for (locale, bundle, entries) in [("en", &en, &merged_en), ("ru", &ru, &merged)] {
                if bundle.entries == *entries {
                    continue;
                }
                let path = self.root.join(format!("localization/{locale}.json"));
                let mut document = bundle.document.clone();
                document["entries"] = serde_json::to_value(entries).map_err(|e| e.to_string())?;
                let mut bytes = serde_json::to_vec_pretty(&document).map_err(|e| e.to_string())?;
                bytes.push(b'\n');
                let mut temporary = tempfile::NamedTempFile::new_in(path.parent().unwrap())
                    .map_err(|e| e.to_string())?;
                temporary.write_all(&bytes).map_err(|e| e.to_string())?;
                temporary.as_file().sync_all().map_err(|e| e.to_string())?;
                staged.push((path, temporary));
            }
            if fs::read(self.root.join("localization/en.json")).map_err(|e| e.to_string())?
                != en.bytes
                || fs::read(self.root.join("localization/ru.json")).map_err(|e| e.to_string())?
                    != ru.bytes
            {
                return Err("Files changed during save; synchronize and retry".into());
            }
            for (path, temporary) in staged {
                temporary.persist(path).map_err(|e| e.to_string())?;
            }
            en.entries = merged_en.clone();
            ru.entries = merged.clone();
        }
        let changed = self.en != merged_en
            || self.draft != merged
            || self.base != ru.entries
            || self.en_base != en.entries;
        self.file_stamps = if save { None } else { before_stamps };
        self.filter_dirty |= self.en != merged_en || self.draft != merged;
        self.refresh_localization |= changed || save;
        self.en = merged_en;
        self.en_base = en.entries;
        self.base = ru.entries;
        self.draft = merged;
        if !self.en.contains_key(&self.selected) {
            self.selected = self.en.keys().next().cloned().unwrap_or_default();
        }
        self.clamp_selection();
        if save {
            self.changed_at = None;
            self.status = LocalizedText::new("ui.editor.strings.saved", "Saved to {path}")
                .with_arg("path", self.root.join("localization").display().to_string());
            self.revision += 1;
        } else if changed {
            if self.dirty() && self.changed_at.is_none() {
                self.changed_at = Some(Instant::now());
            }
            self.revision += 1;
        }
        Ok(())
    }
    pub(super) fn keys(&self) -> Vec<String> {
        self.en
            .iter()
            .filter(|(key, en)| {
                self.tools.filter_matches(key)
                    || self.tools.filter_matches(en)
                    || self
                        .draft
                        .get(*key)
                        .is_some_and(|ru| self.tools.filter_matches(ru))
            })
            .map(|(key, _)| key.clone())
            .collect()
    }

    pub(super) fn replace_selection(&mut self, inserted: &str) {
        let source = if self.russian {
            &mut self.draft
        } else {
            &mut self.en
        };
        let Some(text) = source.get_mut(&self.selected) else {
            return;
        };
        self.undo
            .push((self.russian, vec![(self.selected.clone(), text.clone())]));
        self.redo.clear();
        self.discard_confirm = false;
        let start = self.cursor.min(self.anchor);
        text.replace_range(start..self.cursor.max(self.anchor), inserted);
        self.cursor = start + inserted.len();
        self.anchor = self.cursor;
        self.edited_metric = Some(self.selected.clone());
        self.changed_at = Some(Instant::now());
        self.status = LocalizedText::new("ui.editor.strings.dirty", "Unsaved changes");
        self.revision += 1;
    }
}

#[derive(Component)]
pub(in super::super) struct EditorSectionTitle;

pub(in super::super) struct StringsPlugin;

impl Plugin for StringsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                exchange::poll_dialog,
                exchange::context_input,
                editing_tools::field_click,
                editing_tools::drag_cell,
                buttons,
                keyboard,
                persistence,
                body_visibility,
                section_header,
                measure_rows,
                scroll_table,
                draw,
                editing_tools::remember_tab,
            )
                .chain()
                .after(handle_editor_buttons)
                .before(LocalizationSet::Apply),
        );
        app.add_systems(
            PostUpdate,
            (
                editing_tools::carets,
                editing_tools::field_layout,
                editing_tools::selection_layout,
            )
                .chain()
                .after(bevy::ui::UiSystems::Layout),
        );
    }
}

#[derive(Component)]
pub(super) struct StringsRoot;

#[derive(Component)]
pub(super) struct StringCell(pub(super) String, pub(super) bool);

#[derive(Component)]
pub(super) struct CellMetric {
    pub(super) key: String,
    pub(super) revision: u64,
}

// The table keeps only visible rows in the ECS, independent of bundle size.
#[derive(Component)]
pub(super) struct TableViewport;

#[derive(Component)]
pub(super) struct TableTrack;
