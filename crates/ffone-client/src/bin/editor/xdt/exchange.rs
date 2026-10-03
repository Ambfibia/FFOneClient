use super::*;
use std::sync::{Mutex, mpsc};

pub(super) struct FileJob {
    receiver: Mutex<mpsc::Receiver<Result<Option<String>, String>>>,
    pointer: String,
    snapshot: Value,
    import: bool,
}
pub(super) fn csv_export(rows: &[Value]) -> Result<String, String> {
    let columns: Vec<_> = rows
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|r| r.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(&columns).map_err(|e| e.to_string())?;
    for row in rows {
        writer
            .write_record(
                columns
                    .iter()
                    .map(|c| row.get(c).map(Value::to_string).unwrap_or_default()),
            )
            .map_err(|e| e.to_string())?;
    }
    String::from_utf8(writer.into_inner().map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub(super) fn csv_import(text: &str, before: &[Value]) -> Result<Vec<Value>, String> {
    let text = text.trim_start_matches('\u{feff}');
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();
    let mut names = BTreeSet::new();
    for field in &headers {
        if field.is_empty() || !names.insert(field.to_owned()) {
            return Err("Duplicate or empty CSV column".into());
        }
    }
    let expected: BTreeSet<_> = before
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|r| r.keys().cloned())
        .collect();
    if !expected.is_empty() && names != expected {
        return Err("CSV columns must match the selected table".into());
    }
    let mut result = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;
        let mut row = serde_json::Map::new();
        for (field, text) in headers.iter().zip(record.iter()) {
            if text.is_empty() {
                continue;
            } // Missing field differs from JSON "" and null.
            let value: Value = serde_json::from_str(text)
                .map_err(|e| format!("CSV row {}, {field}: {e}", result.len() + 2))?;
            row.insert(field.to_owned(), value);
        }
        result.push(Value::Object(row));
    }
    validate_rows(before, &result)?;
    Ok(result)
}
pub(super) fn start(editor: &mut XdtEditor, import: bool) -> Result<(), String> {
    if editor.file_job.is_some() {
        return Ok(());
    }
    let table = editor.tables.get(editor.table).ok_or("No table")?;
    let snapshot = Value::Array(editor.rows().to_vec());
    let text = if import {
        String::new()
    } else {
        csv_export(editor.rows())?
    };
    let filename = format!("{}.csv", table.label.rsplit('/').next().unwrap_or("table"));
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        #[cfg(windows)]
        let result = (|| {
            let dialog = rfd::FileDialog::new()
                .add_filter("CSV", &["csv"])
                .set_file_name(&filename);
            let path = if import {
                dialog.pick_file()
            } else {
                dialog.save_file()
            };
            let Some(path) = path else { return Ok(None) };
            if import {
                fs::read_to_string(&path)
                    .map(Some)
                    .map_err(|e| e.to_string())
            } else {
                fs::write(&path, format!("\u{feff}{text}")).map_err(|e| e.to_string())?;
                Ok(Some(path.display().to_string()))
            }
        })();
        #[cfg(not(windows))]
        let result = {
            let _ = (text, filename);
            Err("Native CSV dialogs require Windows".into())
        };
        let _ = tx.send(result);
    });
    editor.file_job = Some(FileJob {
        receiver: Mutex::new(rx),
        pointer: table.pointer.clone(),
        snapshot,
        import,
    });
    Ok(())
}
pub(super) fn poll(editor: &mut XdtEditor) {
    let Some(job) = editor.file_job.as_ref() else {
        return;
    };
    let result = job.receiver.lock().unwrap().try_recv();
    let result = match result {
        Ok(result) => result,
        Err(mpsc::TryRecvError::Empty) => return,
        Err(_) => Err("CSV dialog closed unexpectedly".into()),
    };
    let job = editor.file_job.take().unwrap();
    let result: Result<(), String> = (|| {
        let Some(text) = result? else { return Ok(()) };
        if !job.import {
            editor.status = text;
            return Ok(());
        }
        let table = editor
            .tables
            .iter()
            .position(|t| t.pointer == job.pointer)
            .ok_or("Table no longer exists")?;
        if editor.document.pointer(&job.pointer) != Some(&job.snapshot) {
            return Err("Table changed while the CSV dialog was open".into());
        }
        if !editor.apply() {
            return Err("Apply or cancel the active draft before importing".into());
        }
        // Applying a cell can change the same table; check again after committing it.
        if editor.document.pointer(&job.pointer) != Some(&job.snapshot) {
            return Err("Table changed while the CSV dialog was open".into());
        }
        let rows = csv_import(&text, job.snapshot.as_array().unwrap())?;
        editor.select_table(table);
        editor.change(rows)?;
        editor.row = None;
        Ok(())
    })();
    if let Err(error) = result {
        editor.status = error;
    }
    editor.revision += 1;
}
