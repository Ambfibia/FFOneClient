//! Editor-only durable work. Gameplay files are written only by Rewrite.
use super::*;
use serde::{Deserialize, Serialize};

const SCHEMA: &str = "ffone.xdt-workspace.v1";

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct Snapshot {
    schema: String,
    base: Value,
    pub document: Value,
    locale_base: [BTreeMap<String, String>; 2],
    pub locale_drafts: BTreeMap<String, [Option<String>; 2]>,
    server: Option<(PathBuf, Vec<u8>)>,
}

impl XdtEditor {
    pub(super) fn work_path(&self) -> Result<PathBuf, String> {
        let root = self.path.parent().and_then(Path::parent).and_then(Path::parent)
            .ok_or("Missing asset root")?;
        let editor_root = if root.file_name().is_some_and(|name| name == "game") {
            root.parent().ok_or("Missing editor asset root")?.join("editor")
        } else { root.join("editor") };
        Ok(editor_root.join("xdt-workspace.json"))
    }

    pub(super) fn unpublished(&self) -> bool {
        self.document != self.base || !self.workspace.locale_drafts.is_empty()
    }

    pub(super) fn restore_work(&mut self) -> Result<(), String> {
        let path = self.work_path()?;
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        let saved: Snapshot = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if saved.schema != SCHEMA || !saved.document["tables"].is_array() || !saved.base["tables"].is_array() {
            return Err(format!("Invalid editor workspace: {}", path.display()));
        }
        // A published snapshot must not hide newer gameplay data on next launch.
        if saved.document == saved.base && saved.locale_drafts.is_empty() { return Ok(()); }
        self.base = saved.base.clone();
        self.document = saved.document.clone();
        self.workspace.locale_base = saved.locale_base.clone();
        self.workspace.locale_drafts = saved.locale_drafts.clone();
        if let (Some(server), Some((path, digest))) = (&mut self.publication, &saved.server) {
            if &server.path == path { server.digest = digest.clone(); }
        }
        if self.publication.is_none() {
            if let Some((path, digest)) = &saved.server {
                let root = self.path.parent().and_then(Path::parent).and_then(Path::parent).unwrap();
                let selected = fs::read(mission_server::settings_path(root)).ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .and_then(|value| value["xdt_path"].as_str().map(PathBuf::from));
                if selected.as_ref().is_none_or(|selected| selected == path) {
                    // Keep the original destination even if it is now unreadable or invalid.
                    // Rewrite must report the conflict instead of publishing client only.
                    self.publication = Some(mission_publish::Publication { path: path.clone(), digest: digest.clone() });
                }
            }
        }
        self.saved_work = Some(saved);
        if self.unpublished() { self.status = "Working copy restored; not applied to game".into(); }
        Ok(())
    }

    pub(super) fn save(&mut self) -> Result<(), String> {
        let saved = Snapshot {
            schema: SCHEMA.into(),
            base: self.base.clone(),
            document: self.document.clone(),
            locale_base: self.workspace.locale_base.clone(),
            locale_drafts: self.workspace.locale_drafts.clone(),
            server: self.publication.as_ref().map(|server| (server.path.clone(), server.digest.clone())),
        };
        let path = self.work_path()?;
        fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
        let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|error| error.to_string())?;
        serde_json::to_writer_pretty(&mut temp, &saved).map_err(|error| error.to_string())?;
        temp.write_all(b"\n").map_err(|error| error.to_string())?;
        temp.as_file().sync_all().map_err(|error| error.to_string())?;
        temp.persist(&path).map_err(|error| error.to_string())?;
        self.saved_work = Some(saved);
        self.status = "Working copy saved; not applied to game".into();
        self.revision += 1;
        Ok(())
    }

    pub(super) fn reload_game(&mut self) -> Result<(), String> {
        let document = read(&self.path)?;
        let root = self.path.parent().and_then(Path::parent).and_then(Path::parent).ok_or("Missing asset root")?;
        let locales = mission_workspace::Workspace::load(root).locale_base;
        self.base = document.clone();
        self.document = document;
        self.workspace.locale_base = locales;
        self.workspace.locale_drafts.clear();
        self.undo.clear();
        self.redo.clear();
        self.row = None;
        self.discover();
        self.save()?;
        self.reload_confirm = false;
        self.status.clear();
        Ok(())
    }
}
