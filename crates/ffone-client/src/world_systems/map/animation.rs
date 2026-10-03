use super::*;

/// Clean `WorldMapMode` keeps both scan phases on the mode instance and only
/// advances them while the map is being painted. The deterministic GPU
/// harness may freeze the phases, while production uses [`Default`].
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct WorldMapScanAnimation {
    pub(super) large_phase: f32,
    pub(super) small_phase: f32,
    pub(super) advancing: bool,
}

impl Default for WorldMapScanAnimation {
    fn default() -> Self {
        Self {
            large_phase: 0.0,
            small_phase: 0.0,
            advancing: true,
        }
    }
}

impl WorldMapScanAnimation {
    #[must_use]
    pub fn frozen(large_phase: f32, small_phase: f32) -> Self {
        Self {
            large_phase: valid_world_map_scan_phase(large_phase),
            small_phase: valid_world_map_scan_phase(small_phase),
            advancing: false,
        }
    }

    #[must_use]
    pub const fn phases(self) -> (f32, f32) {
        (self.large_phase, self.small_phase)
    }
}

pub(super) fn advance_world_map_scan_animation(
    time: Res<Time>,
    presentation: Res<WorldMapPresentation>,
    mut animation: ResMut<WorldMapScanAnimation>,
) {
    if !animation.advancing || presentation.model.phase() != WorldMapPhase::Open {
        return;
    }
    let delta_seconds = time.delta_secs();
    if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
        return;
    }
    let map_height = presentation.model.map_draw_rect().height;
    advance_world_map_scan_phases(&mut animation, map_height, delta_seconds);
}
