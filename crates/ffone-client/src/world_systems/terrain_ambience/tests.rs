use crate::terrain_ambience::*;

fn sample(value: f32) -> NativeTerrainAmbienceSample {
    NativeTerrainAmbienceSample {
        fog_depth: value,
        fog_color: [value, value + 1.0, value + 2.0, 1.0],
        sky_color: [value + 3.0, value + 4.0, value + 5.0, 1.0],
        light_color: [value + 6.0, value + 7.0, value + 8.0, 1.0],
    }
}

#[test]
fn legacy_fraction_remap_and_four_neighbor_weights_are_exact() {
    let mut grid = NativeTerrainAmbienceGrid::default();
    grid.cells[grid_index(3, 4).unwrap()] = Some(sample(0.0));
    grid.cells[grid_index(4, 4).unwrap()] = Some(sample(10.0));
    grid.cells[grid_index(3, 5).unwrap()] = Some(sample(20.0));
    grid.cells[grid_index(4, 5).unwrap()] = Some(sample(30.0));

    // local fraction .5 remaps to .5 on both axes, producing four equal weights.
    let unity_x = (3.5 + 0.5) * LEGACY_AMBIENCE_GRID_CELL_SIZE;
    let unity_z = (4.5 + 0.5) * LEGACY_AMBIENCE_GRID_CELL_SIZE;
    let actual = grid.sample_unity_position(unity_x, unity_z);
    assert!((actual.fog_depth - 15.0).abs() < 0.000_01);
    assert_eq!(actual.fog_color, [15.0, 16.0, 17.0, 1.0]);

    // The old client deliberately holds the lower cell over the first quarter.
    let unity_x = (3.2 + 0.5) * LEGACY_AMBIENCE_GRID_CELL_SIZE;
    let unity_z = (4.2 + 0.5) * LEGACY_AMBIENCE_GRID_CELL_SIZE;
    assert_eq!(grid.sample_unity_position(unity_x, unity_z).fog_depth, 0.0);
}

#[test]
fn missing_neighbors_are_normalized_and_empty_grid_uses_unity_fallback() {
    let mut grid = NativeTerrainAmbienceGrid::default();
    grid.cells[grid_index(4, 4).unwrap()] = Some(sample(12.0));
    let actual = grid.sample_unity_position(4.0 * 512.0, 4.0 * 512.0);
    assert_eq!(actual, sample(12.0));

    assert_eq!(
        NativeTerrainAmbienceGrid::default().sample_unity_position(0.0, 0.0),
        NativeTerrainAmbienceSample::ZERO_WEIGHT_FALLBACK
    );
}

#[test]
fn horizon_haze_density_closes_the_camera_far_plane() {
    for far in [150.0_f32, 340.0, 1_000.0] {
        let density = horizon_haze_density(far);
        let depth = density * far;
        let visible = (-(depth * depth)).exp();
        assert!(
            (visible - HORIZON_HAZE_RESIDUAL).abs() < 0.000_1,
            "far {far} left {visible} of the surface visible"
        );
    }
    assert_eq!(horizon_haze_density(0.0), 0.0);
    assert_eq!(horizon_haze_density(f32::NAN), 0.0);
}

#[test]
fn distance_haze_keeps_a_denser_exact_ambience_and_lifts_a_thin_one() {
    let dense = apply_default_ambience(
        NativeTerrainAmbienceSample {
            fog_depth: 4.0,
            fog_color: [0.0, 0.25, 0.0, 1.0],
            sky_color: [0.0, 0.5, 0.0, 1.0],
            light_color: [1.0, 1.0, 1.0, 1.0],
        },
        false,
    );
    // fogDepth 4 already fogs out well inside 340 units, so the exact
    // `DefaultAmbience` density must survive untouched.
    assert_eq!(distance_haze_density(&dense, 340.0), dense.fog_density);

    let thin = apply_default_ambience(
        NativeTerrainAmbienceSample {
            fog_depth: 0.7,
            fog_color: [0.05, 0.58, 0.32, 1.0],
            sky_color: [0.1, 1.0, 0.6, 1.0],
            light_color: [1.0, 1.0, 1.0, 1.0],
        },
        false,
    );
    assert!(distance_haze_density(&thin, 340.0) > thin.fog_density);
    assert_eq!(
        distance_haze_density(&thin, 340.0),
        horizon_haze_density(340.0)
    );

    // A dong that disables fog entirely still gets the horizon haze; the
    // sampled area color is what keeps it green here and blue downtown.
    let disabled = apply_default_ambience(
        NativeTerrainAmbienceSample {
            fog_depth: 0.0,
            fog_color: [0.25, 0.55, 1.0, 1.0],
            sky_color: [1.0, 1.0, 1.0, 1.0],
            light_color: [1.0, 1.0, 1.0, 1.0],
        },
        false,
    );
    assert!(!disabled.fog_enabled);
    assert_eq!(
        distance_haze_density(&disabled, 340.0),
        horizon_haze_density(340.0)
    );
}

#[test]
fn native_x_reflection_and_default_ambience_match_legacy_formulas() {
    let mut grid = NativeTerrainAmbienceGrid::default();
    grid.cells[grid_index(2, 1).unwrap()] = Some(sample(2.0));
    let native = grid.sample_native_position(-2.0 * 512.0, 1.0 * 512.0);
    let unity = grid.sample_unity_position(2.0 * 512.0, 1.0 * 512.0);
    assert_eq!(native, unity);

    let applied = apply_default_ambience(
        NativeTerrainAmbienceSample {
            fog_depth: 2.0,
            fog_color: [0.2, 0.4, 0.6, 1.0],
            sky_color: [0.1, 0.2, 0.3, 1.0],
            light_color: [0.0, 0.5, 1.0, 1.0],
        },
        true,
    );
    assert!(applied.fog_enabled);
    assert!((applied.fog_density - 0.01).abs() < f32::EPSILON);
    assert_eq!(applied.fog_color, [0.6, 0.7, 0.8, 0.85]);
    assert_eq!(applied.light_color, [0.4, 0.70000005, 1.0, 1.0]);
}
