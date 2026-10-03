//! Regression coverage moved from the synthetic race pulse to authored controllers.
use super::*;
use crate::legacy_model_material::{LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind};

#[test]
fn glow_fades_and_repeats_without_blinking() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let document = load_native_world_behaviours(&root, NativeWorldScope::WorldMap, "map_00_09").unwrap();
    let mut checked = 0;
    for clip in &document.animation_clips {
        for curve in &clip.float_curves {
            if curve.target_path != "Sphere01h/Sphere01h:1" || curve.attribute != "_Emission.r" { continue; }
            let period = (curve.times.last().unwrap() - curve.times[0]) as f32;
            let sample = |t| sample_world_clip_material_curve(clip, 0, false, t, curve).unwrap();
            assert!((sample(0.5) - sample(period + 0.5)).abs() < 0.0001);
            assert!((sample(0.0) - sample(1.0)).abs() > 0.1);
            checked += 1;
        }
    }
    assert!(checked > 0, "real pod material must participate in the authored cycle");
}

#[test]
fn pod_passes_pulse_without_changing_shared_materials() {
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal);
    let original = params.material_for_pass(params.render_plan().passes[0], &LegacyModelTextures::default()).unwrap();
    let mut private = original.clone();
    assert!(apply_world_material_float_curve(&mut private, "_Emission.r", 0.7));
    assert_eq!(private.uniform.emission.red, 0.7);
    assert_ne!(original.uniform.emission.red, private.uniform.emission.red);
}

#[test]
fn nif_texture_color_controller_repeats_beyond_nonlooping_clip_wrapper() {
    let curve = linear_material_curve("arrow", "_Emission.g", 1.0, 1.0);
    let mut clip = material_clip("nif-default", false, vec![curve.clone()]);
    clip.duration = 1.0;
    assert_eq!(sample_world_clip_material_curve(&clip, 0, false, 2.5, &curve), Some(0.5));
    clip.name = "one-shot-effect".into();
    assert_eq!(sample_world_clip_material_curve(&clip, 0, false, 2.5, &curve), Some(1.0));
}

#[test]
fn launcher_heading_survives_native_rotation_beyond_ninety_degrees() {
    for heading in [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] {
        let native = crate::coordinates::unity_to_native_rotation(Quat::from_rotation_y(f32::to_radians(heading)));
        let actual = native_rotation_to_unity_euler_degrees(native);
        let delta = (actual.y - heading + 180.0).rem_euclid(360.0) - 180.0;
        assert!(delta.abs() < 0.001, "heading={heading}, actual={actual:?}");
    }
}
