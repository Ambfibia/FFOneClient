//! Native implementation of the legacy `DongLoader.GetPositionColor` terrain ambience grid.
//!
//! The runtime consumes verified [`NativeTerrainEnvironment`] documents. It never queries a
//! Unity scene or bundle. Coordinates are kept in the old client's Unity X/Z space at the API
//! boundary, with a separate helper for the reflected native Bevy X axis.

use crate::native_terrain::NativeTerrainEnvironment;

pub const LEGACY_AMBIENCE_GRID_WIDTH: usize = 16;
pub const LEGACY_AMBIENCE_GRID_CELL_SIZE: f32 = 512.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeTerrainAmbienceSample {
    pub fog_depth: f32,
    pub fog_color: [f32; 4],
    pub sky_color: [f32; 4],
    pub light_color: [f32; 4],
}

impl NativeTerrainAmbienceSample {
    const ZERO_WEIGHT_FALLBACK: Self = Self {
        fog_depth: 0.0,
        // `DongLoader` initializes this with Unity `Color.black`, whose alpha is one.
        fog_color: [0.0, 0.0, 0.0, 1.0],
        sky_color: [1.0, 1.0, 1.0, 1.0],
        light_color: [1.0, 1.0, 1.0, 1.0],
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeTerrainAppliedAmbience {
    pub fog_enabled: bool,
    pub fog_density: f32,
    pub fog_color: [f32; 4],
    pub sky_color: [f32; 4],
    pub light_color: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeTerrainAmbienceBlocker {
    pub tile_id: String,
    pub status: String,
    pub ambience_status: String,
}

#[derive(Clone, Debug)]
pub struct NativeTerrainAmbienceGrid {
    cells: [Option<NativeTerrainAmbienceSample>;
        LEGACY_AMBIENCE_GRID_WIDTH * LEGACY_AMBIENCE_GRID_WIDTH],
    blockers: Vec<NativeTerrainAmbienceBlocker>,
}

impl Default for NativeTerrainAmbienceGrid {
    fn default() -> Self {
        Self {
            cells: [None; LEGACY_AMBIENCE_GRID_WIDTH * LEGACY_AMBIENCE_GRID_WIDTH],
            blockers: Vec::new(),
        }
    }
}

impl NativeTerrainAmbienceGrid {
    /// Registers an environment only when its placement and serialized DongColor coordinates are
    /// exact. Source contradictions remain visible as typed blockers instead of creating an
    /// order-dependent duplicate grid entry.
    pub fn register(
        &mut self,
        environment: &NativeTerrainEnvironment,
    ) -> Result<(), NativeTerrainAmbienceBlocker> {
        if environment.status != "complete" || environment.ambience.status != "exactSource" {
            let blocker = NativeTerrainAmbienceBlocker {
                tile_id: environment.tile_id.clone(),
                status: environment.status.clone(),
                ambience_status: environment.ambience.status.clone(),
            };
            self.blockers.push(blocker.clone());
            return Err(blocker);
        }
        let [x, y] = environment.ambience.grid_coordinates;
        let Some(index) = grid_index(x, y) else {
            let blocker = NativeTerrainAmbienceBlocker {
                tile_id: environment.tile_id.clone(),
                status: "gridCoordinatesOutsideLegacy16x16".to_owned(),
                ambience_status: environment.ambience.status.clone(),
            };
            self.blockers.push(blocker.clone());
            return Err(blocker);
        };
        self.cells[index] = Some(NativeTerrainAmbienceSample {
            fog_depth: environment.ambience.fog_depth as f32,
            fog_color: environment.ambience.fog_color.map(|value| value as f32),
            sky_color: environment.ambience.sky_color.map(|value| value as f32),
            light_color: environment.ambience.light_color.map(|value| value as f32),
        });
        Ok(())
    }

    #[must_use]
    pub fn blockers(&self) -> &[NativeTerrainAmbienceBlocker] {
        &self.blockers
    }

    #[must_use]
    pub fn registered_cell_count(&self) -> usize {
        self.cells.iter().filter(|cell| cell.is_some()).count()
    }

    /// Exact `DongLoader.GetPositionColor` sampling in legacy Unity X/Z coordinates.
    #[must_use]
    pub fn sample_unity_position(&self, unity_x: f32, unity_z: f32) -> NativeTerrainAmbienceSample {
        if !unity_x.is_finite() || !unity_z.is_finite() {
            return NativeTerrainAmbienceSample::ZERO_WEIGHT_FALLBACK;
        }
        let local_x = unity_x / LEGACY_AMBIENCE_GRID_CELL_SIZE - 0.5;
        let local_y = unity_z / LEGACY_AMBIENCE_GRID_CELL_SIZE - 0.5;
        let x0 = local_x.floor() as i32;
        let y0 = local_y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;
        let tx = ((local_x - x0 as f32 - 0.25) * 2.0).clamp(0.0, 1.0);
        let ty = ((local_y - y0 as f32 - 0.25) * 2.0).clamp(0.0, 1.0);
        let candidates = [
            (x0, y0, (1.0 - tx) * (1.0 - ty)),
            (x1, y0, tx * (1.0 - ty)),
            (x0, y1, (1.0 - tx) * ty),
            (x1, y1, tx * ty),
        ];

        let mut fog_depth = 0.0;
        let mut fog_color = [0.0; 4];
        let mut sky_color = [0.0; 4];
        let mut light_color = [0.0; 4];
        let mut total_weight = 0.0;
        for (x, y, weight) in candidates {
            let Some(sample) = grid_index(x, y).and_then(|index| self.cells[index]) else {
                continue;
            };
            fog_depth += sample.fog_depth * weight;
            add_weighted(&mut fog_color, sample.fog_color, weight);
            add_weighted(&mut sky_color, sample.sky_color, weight);
            add_weighted(&mut light_color, sample.light_color, weight);
            total_weight += weight;
        }
        if total_weight <= 0.0 {
            return NativeTerrainAmbienceSample::ZERO_WEIGHT_FALLBACK;
        }
        let reciprocal = total_weight.recip();
        NativeTerrainAmbienceSample {
            fog_depth: fog_depth * reciprocal,
            fog_color: fog_color.map(|value| value * reciprocal),
            sky_color: sky_color.map(|value| value * reciprocal),
            light_color: light_color.map(|value| value * reciprocal),
        }
    }

    /// Native Bevy coordinates reflect the legacy Unity X axis and preserve Z.
    #[must_use]
    pub fn sample_native_position(
        &self,
        native_x: f32,
        native_z: f32,
    ) -> NativeTerrainAmbienceSample {
        self.sample_unity_position(-native_x, native_z)
    }
}

/// Exact `cnPlayerCamera.DefaultAmbience` post-processing of a sampled grid value.
#[must_use]
pub fn apply_default_ambience(
    sample: NativeTerrainAmbienceSample,
    tutorial: bool,
) -> NativeTerrainAppliedAmbience {
    let mut fog_color = sample.fog_color;
    if tutorial {
        for channel in &mut fog_color[..3] {
            *channel = *channel * 0.5 + 0.5;
        }
    }
    fog_color[3] = 0.85;
    NativeTerrainAppliedAmbience {
        fog_enabled: sample.fog_depth > 0.0,
        fog_density: sample.fog_depth * 0.005,
        fog_color,
        sky_color: sample.sky_color,
        light_color: sample.light_color.map(|channel| channel * 0.6 + 0.4),
    }
}

/// Fraction of a surface that is still allowed to show through the haze at the
/// camera far plane. `DefaultAmbience` alone leaves a legible band of unfogged
/// geometry against the sky there, which is exactly what makes the native draw
/// distance visible.
pub const HORIZON_HAZE_RESIDUAL: f32 = 0.02;

/// Smallest exponential-squared density that closes the horizon at `far`.
///
/// The legacy stage is `1 - exp(-(density * depth)^2)`, so solving for
/// `1 - HORIZON_HAZE_RESIDUAL` at `depth == far` gives
/// `density = sqrt(-ln(residual)) / far`.
#[must_use]
pub fn horizon_haze_density(far: f32) -> f32 {
    if !far.is_finite() || far <= 0.0 {
        return 0.0;
    }
    (-HORIZON_HAZE_RESIDUAL.ln()).sqrt() / far
}

/// Density the runtime actually applies. The exact `DefaultAmbience` value is
/// kept whenever it already closes the horizon; the sampled area fog color is
/// never altered, so the Future stays green and Downtown stays blue.
#[must_use]
pub fn distance_haze_density(applied: &NativeTerrainAppliedAmbience, far: f32) -> f32 {
    let legacy = if applied.fog_enabled {
        applied.fog_density
    } else {
        0.0
    };
    legacy.max(horizon_haze_density(far))
}

fn grid_index(x: i32, y: i32) -> Option<usize> {
    let x = usize::try_from(x).ok()?;
    let y = usize::try_from(y).ok()?;
    (x < LEGACY_AMBIENCE_GRID_WIDTH && y < LEGACY_AMBIENCE_GRID_WIDTH)
        .then_some(x + LEGACY_AMBIENCE_GRID_WIDTH * y)
}

fn add_weighted(destination: &mut [f32; 4], value: [f32; 4], weight: f32) {
    for (destination, value) in destination.iter_mut().zip(value) {
        *destination += value * weight;
    }
}

#[cfg(test)]
mod tests;
