use super::*;

#[derive(Resource)]
pub(super) struct Capture {
    pub(super) output: PathBuf,
    pub(super) output_root: PathBuf,
    pub(super) entries: usize,
    pub(super) entry: usize,
    pub(super) restart: bool,
    pub(super) selection_since: Option<Instant>,
    pub(super) position: Vec3,
    pub(super) started: Instant,
    pub(super) ready: Option<Instant>,
    pub(super) previous: Instant,
    pub(super) samples: Vec<f64>,
    pub(super) captured: bool,
    pub(super) frozen: bool,
    pub(super) orbit: bool,
    pub(super) transport_npc_type: Option<i32>,
    pub(super) transport_opened: bool,
    pub(super) transport_diagnostics_written: bool,
    pub(super) npc_chat_probe: bool,
    pub(super) npc_chat_requested: bool,
    pub(super) focus_probe: bool,
    pub(super) focus_phase: u8,
}

#[derive(Resource)]
pub(super) struct VehicleFixture(pub(super) i16);
