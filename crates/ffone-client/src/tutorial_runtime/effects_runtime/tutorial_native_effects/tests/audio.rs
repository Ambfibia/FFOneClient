use super::*;

#[test]
fn impact_sfx_is_spatial_and_uses_clamped_effect_mix_gain() {
    let settings = native_impact_playback_settings(0.35);
    assert!(settings.spatial);
    assert!(
        settings
            .spatial_scale
            .unwrap()
            .0
            .abs_diff_eq(LEGACY_SPATIAL_SCALE.0, 1.0e-6)
    );
    assert!(matches!(settings.volume, Volume::Linear(gain) if gain == 0.35));

    let clamped = native_impact_playback_settings(4.0);
    assert!(matches!(clamped.volume, Volume::Linear(gain) if gain == 1.0));
}
