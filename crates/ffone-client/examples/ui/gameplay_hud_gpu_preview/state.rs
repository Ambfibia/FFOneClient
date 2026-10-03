use super::*;

#[derive(Resource)]
pub(super) struct PreviewState {
    pub(super) started_at: Instant,
    pub(super) portrait_ready_at: Option<Instant>,
    pub(super) capture_issued: bool,
    pub(super) capture_saved: bool,
    pub(super) damage_at: Option<Instant>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            portrait_ready_at: None,
            capture_issued: false,
            capture_saved: false,
            damage_at: None,
        }
    }
}
