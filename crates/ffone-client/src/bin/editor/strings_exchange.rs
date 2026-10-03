//! Selected-row CSV exchange. Selection and import validation use semantic keys, never row offsets.
use super::*;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, mpsc};

#[derive(Default)]
pub(super) struct RowSelection {
    pub(super) keys: BTreeSet<String>,
    anchor: Option<String>,
}
impl RowSelection {
    fn select(&mut self, visible: &[String], key: &str, toggle: bool, range: bool) {
        if range {
            if let (Some(anchor), Some(end)) =
                (self.anchor.as_ref(), visible.iter().position(|v| v == key))
            {
                if let Some(start) = visible.iter().position(|v| v == anchor) {
                    // Shift replaces the range; Ctrl+Shift is deliberately unnecessary for checkboxes.
                    self.keys.clear();
                    self.keys
                        .extend(visible[start.min(end)..=start.max(end)].iter().cloned());
                    return;
                }
            }
        }
        if toggle {
            if !self.keys.remove(key) {
                self.keys.insert(key.to_owned());
            }
        } else {
            self.keys = BTreeSet::from([key.to_owned()]);
        }
        self.anchor = Some(key.to_owned());
    }
}
impl StringEditor {
    pub(super) fn select_row(&mut self, key: &str, toggle: bool, range: bool) {
        let StringEditor {
            rows_selected,
            filtered,
            ..
        } = self;
        rows_selected.select(filtered, key, toggle, range);
        self.editing = false;
        self.search_focus = false;
        self.context_menu = None;
    }
    pub(super) fn apply_history(&mut self, undo: bool) {
        let batch = if undo {
            self.undo.pop()
        } else {
            self.redo.pop()
        };
        let Some((russian, batch)) = batch else {
            return;
        };
        self.russian = russian;
        let source = if russian {
            &mut self.draft
        } else {
            &mut self.en
        };
        let mut inverse = Vec::with_capacity(batch.len());
        for (key, value) in batch {
            if let Some(current) = source.get_mut(&key) {
                inverse.push((key.clone(), std::mem::replace(current, value)));
            }
        }
        if let Some((key, _)) = inverse.first() {
            self.selected = key.clone();
            self.cursor = source[key].len();
            self.anchor = self.cursor;
        }
        if undo {
            self.redo.push((russian, inverse));
        } else {
            self.undo.push((russian, inverse));
        }
        self.metrics_dirty = true;
        self.filter_dirty = true;
        self.changed_at = Some(Instant::now());
        self.status = LocalizedText::new("ui.editor.strings.dirty", "Unsaved changes");
        self.revision += 1;
    }
}

#[derive(Component)]
pub(super) struct RowHit(pub(super) String);
pub(super) fn context_input(
    state: Res<EditorState>,
    mouse: Res<ButtonInput<MouseButton>>,
    rows: Query<(&RowHit, &RelativeCursorPosition)>,
    window: Query<&Window>,
    actions: Query<(&Interaction, &Action)>,
    mut editor: ResMut<StringEditor>,
) {
    if !state.strings_open || editor.file_job.is_some() {
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        if let Some((row, _)) = rows.iter().find(|(_, cursor)| cursor.cursor_over()) {
            if !editor.rows_selected.keys.contains(&row.0) {
                editor.select_row(&row.0, false, false);
            }
            editor.editing = false;
            editor.search_focus = false;
            if let Ok(window) = window.single() {
                if let Some(point) = window.cursor_position() {
                    editor.context_menu = Some(Vec2::new(
                        point.x.clamp(0., (window.width() - 222.).max(0.)),
                        point.y.clamp(
                            EDITOR_HEADER_HEIGHT,
                            (window.height() - 94.).max(EDITOR_HEADER_HEIGHT),
                        ),
                    ));
                }
            }
        } else {
            editor.context_menu = None;
        }
        editor.revision += 1;
    } else if mouse.just_pressed(MouseButton::Left) && editor.context_menu.is_some() {
        let menu_pressed = actions.iter().any(|(i, a)| {
            *i == Interaction::Pressed && matches!(a, Action::ExportRows | Action::ImportRows)
        });
        if !menu_pressed {
            editor.context_menu = None;
            editor.revision += 1;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ExchangeRow {
    key: String,
    en: String,
    ru: String,
}
fn io_error(error: impl std::fmt::Display) -> LocalizedText {
    LocalizedText::new(
        "ui.editor.strings.exchange_error",
        "Import/export failed: {error}",
    )
    .with_arg("error", error.to_string())
}
fn row_error(key: &'static str, fallback: &'static str, row: &str) -> LocalizedText {
    LocalizedText::new(key, fallback).with_arg("key", row)
}
fn encode(rows: &[ExchangeRow]) -> Result<Vec<u8>, LocalizedText> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());
    writer.write_record(["key", "en", "ru"]).map_err(io_error)?;
    for row in rows {
        writer.serialize(row).map_err(io_error)?;
    }
    let mut bytes = vec![0xef, 0xbb, 0xbf]; // UTF-8 BOM also keeps Cyrillic intact in desktop spreadsheet apps.
    bytes.extend(writer.into_inner().map_err(io_error)?);
    Ok(bytes)
}
fn decode(bytes: &[u8]) -> Result<Vec<ExchangeRow>, LocalizedText> {
    let mut reader = csv::ReaderBuilder::new().from_reader(bytes);
    let headers = reader.headers().map_err(io_error)?;
    if headers.len() != 3
        || headers.iter().collect::<BTreeSet<_>>() != BTreeSet::from(["key", "en", "ru"])
    {
        return Err(LocalizedText::new(
            "ui.editor.strings.exchange_header",
            "Expected CSV columns: key, en, ru.",
        ));
    }
    reader
        .deserialize()
        .map(|row| row.map_err(io_error))
        .collect()
}
fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}
fn plan_import(
    rows: Vec<ExchangeRow>,
    selected: &[ExchangeRow],
    en: &BTreeMap<String, String>,
    draft: &BTreeMap<String, String>,
) -> Result<Vec<(String, String)>, LocalizedText> {
    let mut keyed = BTreeMap::new();
    for row in rows {
        if keyed.contains_key(&row.key) {
            return Err(row_error(
                "ui.editor.strings.exchange_duplicate",
                "Duplicate key: {key}",
                &row.key,
            ));
        }
        keyed.insert(row.key.clone(), row);
    }
    let expected: BTreeSet<_> = selected.iter().map(|row| row.key.as_str()).collect();
    if expected.is_empty() || expected != keyed.keys().map(String::as_str).collect() {
        return Err(LocalizedText::new(
            "ui.editor.strings.exchange_selection",
            "The CSV must contain exactly the selected {count} rows.",
        )
        .with_arg("count", expected.len().to_string()));
    }
    let mut plan = Vec::with_capacity(selected.len());
    for original in selected {
        let row = &keyed[&original.key];
        if en.get(&row.key) != Some(&original.en)
            || normalize_newlines(&row.en) != normalize_newlines(&original.en)
        {
            return Err(row_error(
                "ui.editor.strings.exchange_source_changed",
                "English source changed: {key}",
                &row.key,
            ));
        }
        let ru = normalize_newlines(&row.ru);
        if draft.get(&row.key) != Some(&original.ru) && draft.get(&row.key) != Some(&ru) {
            return Err(row_error(
                "ui.editor.strings.exchange_conflict",
                "The row changed while the file dialog was open: {key}",
                &row.key,
            ));
        }
        if placeholders(&original.en) != placeholders(&ru) {
            return Err(row_error(
                "ui.editor.strings.exchange_placeholder",
                "Template parameters do not match: {key}",
                &row.key,
            ));
        }
        plan.push((row.key.clone(), ru));
    }
    Ok(plan)
}
pub(super) fn apply_import(editor: &mut StringEditor, plan: Vec<(String, String)>) {
    apply_batch(editor, true, plan);
}
pub(super) fn apply_batch(editor: &mut StringEditor, russian: bool, plan: Vec<(String, String)>) {
    let count = plan.len();
    let source = if russian {
        &mut editor.draft
    } else {
        &mut editor.en
    };
    let mut history = Vec::new();
    for (key, value) in plan {
        if source.get(&key) != Some(&value) {
            let previous = source
                .insert(key.clone(), value)
                .expect("validated selected key");
            history.push((key, previous));
        }
    }
    if !history.is_empty() {
        editor.undo.push((russian, history));
        editor.redo.clear();
        editor.changed_at = Some(Instant::now());
        editor.filter_dirty = true;
        editor.metrics_dirty = true;
    }
    editor.editing = false;
    editor.clamp_selection();
    editor.status = LocalizedText::new("ui.editor.strings.imported", "Imported: {count} rows")
        .with_arg("count", count.to_string());
    editor.revision += 1;
}
fn write_csv(path: &Path, rows: &[ExchangeRow]) -> Result<(), LocalizedText> {
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
    {
        return Err(LocalizedText::new(
            "ui.editor.strings.exchange_extension",
            "Choose a .csv file.",
        ));
    }
    let bytes = encode(rows)?;
    let mut temporary = tempfile::NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))
        .map_err(io_error)?;
    temporary.write_all(&bytes).map_err(io_error)?;
    temporary.as_file().sync_all().map_err(io_error)?;
    temporary.persist(path).map_err(io_error)?;
    Ok(())
}

pub(super) struct FileJob {
    receiver: Mutex<mpsc::Receiver<Option<PathBuf>>>,
    import: bool,
    selected: Vec<ExchangeRow>,
    previous_status: LocalizedText,
}
pub(super) fn start_dialog(
    editor: &mut StringEditor,
    import: bool,
    localization: &Localization,
    language: &Language,
) {
    editor.context_menu = None;
    if editor.rows_selected.keys.is_empty() {
        return;
    }
    let selected: Vec<_> = editor
        .rows_selected
        .keys
        .iter()
        .filter_map(|key| {
            Some(ExchangeRow {
                key: key.clone(),
                en: editor.en.get(key)?.clone(),
                ru: editor.draft.get(key)?.clone(),
            })
        })
        .collect();
    if selected.len() != editor.rows_selected.keys.len() {
        editor.status = LocalizedText::new(
            "ui.editor.strings.exchange_selection",
            "The CSV must contain exactly the selected {count} rows.",
        )
        .with_arg("count", editor.rows_selected.keys.len().to_string());
        return;
    }
    let title = localization.text(
        language,
        &if import {
            LocalizedText::new("ui.editor.strings.import_dialog", "Import selected rows")
        } else {
            LocalizedText::new("ui.editor.strings.export_dialog", "Export selected rows")
        },
    );
    let directory = editor.exchange_directory.clone().unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_default()
            .join("target/localization")
    });
    if let Err(e) = fs::create_dir_all(&directory) {
        editor.status = io_error(e);
        return;
    }
    let count = selected.len();
    let (sender, receiver) = mpsc::channel();
    #[cfg(windows)]
    let spawn = std::thread::Builder::new()
        .name("string-exchange-dialog".into())
        .spawn(move || {
            let dialog = rfd::FileDialog::new()
                .set_title(title)
                .set_directory(directory)
                .add_filter("CSV", &["csv"]);
            let path = if import {
                dialog.pick_file()
            } else {
                dialog
                    .set_file_name(format!("strings-{count}.csv"))
                    .save_file()
            };
            let _ = sender.send(path);
        });
    #[cfg(not(windows))]
    let spawn = std::thread::Builder::new()
        .name("string-exchange-dialog".into())
        .spawn(move || {
            let _ = (title, directory, count);
            let _ = sender.send(None);
        });
    if let Err(e) = spawn {
        editor.status = io_error(e);
        return;
    }
    editor.file_job = Some(FileJob {
        receiver: Mutex::new(receiver),
        import,
        selected,
        previous_status: editor.status.clone(),
    });
    editor.editing = false;
    editor.search_focus = false;
    editor.status = LocalizedText::new("ui.editor.strings.file_busy", "Choose a CSV file…");
    editor.revision += 1;
}
pub(super) fn poll_dialog(mut editor: ResMut<StringEditor>) {
    let Some(job) = &editor.file_job else {
        return;
    };
    let result = job
        .receiver
        .lock()
        .expect("dialog channel mutex")
        .try_recv();
    let path = match result {
        Err(mpsc::TryRecvError::Empty) => return,
        Err(mpsc::TryRecvError::Disconnected) => None,
        Ok(path) => path,
    };
    let job = editor.file_job.take().unwrap();
    editor.revision += 1;
    let Some(mut path) = path else {
        if editor.status.key == "ui.editor.strings.file_busy" {
            editor.status = job.previous_status;
        }
        return;
    };
    editor.exchange_directory = path.parent().map(Path::to_path_buf);
    if !job.import && path.extension().is_none() {
        path.set_extension("csv");
    }
    let result = if job.import {
        fs::read(&path)
            .map_err(io_error)
            .and_then(|bytes| decode(&bytes))
            .and_then(|rows| {
                // Reconcile disk changes before comparing the snapshot from opening the dialog.
                editor.sync(false).map_err(io_error)?;
                let plan = plan_import(rows, &job.selected, &editor.en, &editor.draft)?;
                apply_import(&mut editor, plan);
                Ok(())
            })
    } else {
        write_csv(&path, &job.selected).map(|()| {
            editor.status =
                LocalizedText::new("ui.editor.strings.exported", "Exported: {count} rows")
                    .with_arg("count", job.selected.len().to_string());
        })
    };
    if let Err(error) = result {
        editor.status = error;
    }
}

#[cfg(test)]
#[path = "strings_exchange/tests.rs"]
mod tests;
