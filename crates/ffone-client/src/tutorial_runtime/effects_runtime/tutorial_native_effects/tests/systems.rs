use super::*;

#[test]
fn legacy_particle_timer_emits_on_its_first_enabled_update() {
    let mut timer = 2.0;
    assert!(legacy_emission_due(&mut timer, 1.0 / 60.0, 3.0));
    assert_eq!(timer, 2.0);

    timer = 0.0;
    assert!(!legacy_emission_due(&mut timer, 1.0 / 60.0, 3.0));
    assert!((timer - 0.05).abs() < 1.0e-6);
}

#[test]
fn sword_trail_late_update_inserts_four_slerp_edges_then_current_stag_edge() {
    let mut edges = vec![
        NativeSwordTrailEdge {
            top: Vec3::Y,
            bottom: Vec3::ZERO,
        };
        RETROBUTION_SWORD_TRAIL_LENGTH
    ];
    let current_top = Vec3::new(1.0, 1.0, 0.0);
    let current_bottom = Vec3::X;
    advance_native_sword_trail(&mut edges, current_top, current_bottom);
    assert_eq!(edges[24].top, Vec3::Y);
    assert_eq!(edges[29].top, current_top);
    assert_eq!(edges[29].bottom, current_bottom);
    assert!(
        edges[25..29]
            .iter()
            .all(|edge| edge.top.is_finite() && edge.bottom.is_finite())
    );
}
