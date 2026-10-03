use crate::legacy_glow::*;

#[test]
fn serialized_retrobution_defaults_are_preserved() {
    let settings = LegacyGlowSettings::default();
    assert_eq!(settings.filter_color, Vec4::ONE);
    assert_eq!(
        settings.glow_tint,
        Vec4::new(20.0 / 255.0, 20.0 / 255.0, 20.0 / 255.0, 1.0)
    );
    assert_eq!(settings.parameters, Vec4::new(1.8, 1.0, 0.0, 0.0));
}

#[test]
fn shader_preserves_mask_and_normalizes_blur_before_applying_intensity_once() {
    let shader = include_str!("legacy_glow.wgsl");
    for operation in [
        "average.rgb * settings.glow_tint.rgb * (1.0 - average.a)",
        "(samples[0] + samples[1] + samples[2] + samples[3]) * 0.25",
        "+ glow * 2.0 * max(settings.parameters.x, 0.0)",
    ] {
        assert!(
            shader.contains(operation),
            "missing legacy operation {operation}"
        );
    }
    assert!(!shader.contains("let blur_boost"));
}
