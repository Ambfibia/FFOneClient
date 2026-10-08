//! Explicit, persistent server destination; the native folder dialog runs off-thread.
use super::*;
use std::sync::{Mutex, mpsc};

pub(super) struct SelectionJob(Mutex<mpsc::Receiver<Result<Option<PathBuf>, String>>>);

pub(super) fn settings_path(root: &Path) -> PathBuf { root.join("../../target/editor/mission-server.json") }

impl XdtEditor {
    pub(super) fn select_server_folder(&mut self, folder: &Path) -> Result<(), String> {
        let path = if folder.join("xdt.json").is_file() { folder.join("xdt.json") }
            else { folder.join("tabledata/xdt.json") };
        let publication = mission_publish::Publication::open(path)?;
        let root = self.path.parent().and_then(Path::parent).and_then(Path::parent).ok_or("Missing asset root")?;
        let settings = settings_path(root);
        fs::create_dir_all(settings.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut temp = tempfile::NamedTempFile::new_in(settings.parent().unwrap()).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut temp, &serde_json::json!({"xdt_path":publication.path})).map_err(|e| e.to_string())?;
        temp.persist(settings).map_err(|e| e.to_string())?;
        self.publication = Some(publication);
        self.status.clear();
        self.revision += 1;
        Ok(())
    }

    pub(in super::super) fn start_server_selection(&mut self, l: &Localization, lang: &Language) {
        if self.server_job.is_some() { return; }
        let initial = self.publication.as_ref().and_then(|p| p.path.parent()).map(Path::to_path_buf)
            .unwrap_or_else(|| self.path.parent().unwrap().to_path_buf());
        let title = presentation::tr(l, lang, "mission.server_dialog", "Select the server or TableData folder");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            #[cfg(windows)]
            let result = Ok(rfd::FileDialog::new().set_title(&title).set_directory(initial).pick_folder());
            #[cfg(not(windows))]
            let result = { let _ = (initial, title); Err("Native folder dialogs require Windows".into()) };
            let _ = tx.send(result);
        });
        self.server_job = Some(SelectionJob(Mutex::new(rx)));
    }
}

pub(super) fn poll(mut e: ResMut<XdtEditor>) {
    let result = e.server_job.as_ref().and_then(|job| job.0.lock().ok()?.try_recv().ok());
    if let Some(result) = result {
        e.server_job = None;
        if let Err(error) = result.and_then(|folder| folder.map(|p| e.select_server_folder(&p)).transpose()) {
            e.status = error;
            e.revision += 1;
        }
    }
}

pub(super) fn destination(p: &mut ChildSpawnerCommands, f: &EditorFonts, e: &XdtEditor,
    l: &Localization, lang: &Language) {
    if e.draft.is_some() { return; }
    p.spawn(mission_inline::row()).with_children(|p| {
        view::button(p, f, Action::Mission(mission_workspace::Command::SelectServer),
            "ui.editor.xdt.mission.select_server", "Select server", false, 140.);
        p.spawn(Node { min_width: px(0), flex_grow: 1., height: px(34), align_items: AlignItems::Center,
            overflow: Overflow::clip(), ..default() }).with_children(|p| {
            let path = e.publication.as_ref().map(|server| server.path.display().to_string())
                .unwrap_or_else(|| presentation::tr(l, lang, "mission.server_none", "No server selected"));
            view::label(p, f, path, 12., mission_skin::MUTED);
        });
    });
}
