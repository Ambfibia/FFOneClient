//! Each opening starts from published files; restoring a work backup is explicit.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Request {
    Reload,
    Restore,
}

impl WorldEditor {
    pub(crate) fn request_open(&mut self) {
        self.session_request = Some(Request::Reload);
    }

    fn reload(&mut self, folder: Option<&Path>) -> Result<(), String> {
        let mut next = Self::open(self.root.clone());
        let result = folder.map_or(Ok(()), |folder| next.load_server(folder));
        if next.folder == self.folder {
            next.clipboard = self.clipboard.clone();
            next.coordinate_clipboard = self.coordinate_clipboard;
        }
        // Camera settings are view state. Placement and terrain drafts come only from disk.
        if self.open_mode.is_some() {
            next.center = self.center;
            next.zoom = self.zoom;
            next.yaw = self.yaw;
            next.pitch = self.pitch;
            next.distance = self.distance;
            next.instance = self.instance;
            next.region = self
                .region
                .filter(|tile| next.available_tiles.contains(tile));
            next.selected = self
                .selected()
                .and_then(|selected| next.entities.iter().position(|p| p.key == selected.key));
        }
        next.revision = self.revision + 1;
        next.geometry_revision = self.geometry_revision + 1;
        *self = next;
        result
    }

    pub(super) fn enter_session(&mut self, mode: Option<bool>, folder: Option<PathBuf>) -> bool {
        let Some(mode) = mode else {
            self.open_mode = None;
            self.session_request = None;
            return false;
        };
        let folder = folder.or_else(|| self.folder.clone());
        let request = self.session_request.take();
        let new_open = self.open_mode != Some(mode) || request.is_some();
        let server_changed =
            folder != self.folder && !self.sources.iter().any(|s| s.base != s.draft);
        if !new_open && !server_changed {
            return false;
        }
        let result = if request == Some(Request::Restore) {
            self.restore_work()
        } else {
            self.reload(folder.as_deref())
        };
        if let Err(error) = result {
            self.error(error);
        }
        self.open_mode = Some(mode);
        if !mode {
            self.map_picker = false;
        }
        self.ignore_pointer_press = true;
        true
    }
}

pub(super) fn update(
    mut commands: Commands,
    mut e: ResMut<WorldEditor>,
    state: Res<EditorState>,
    xdt: Res<xdt::XdtEditor>,
    mut stream: ResMut<stream::TileStreaming>,
    mut search: ResMut<objects::Search>,
    mut preview: ResMut<preview::WorldPreview>,
) {
    if !e.enter_session(state.world_open, xdt.server_folder()) {
        return;
    }
    *stream = stream::TileStreaming::default();
    *search = objects::Search::default();
    preview.reset(&mut commands);
}
