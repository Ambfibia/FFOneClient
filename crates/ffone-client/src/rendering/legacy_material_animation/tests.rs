use crate::legacy_material_animation::*;
use ffone_skinned_model::AnimationBindingPointer;

fn linear_curve() -> FloatCurve {
    FloatCurve {
        target_node: 0,
        target_path: "waves_in_1".to_owned(),
        source_index: 0,
        source_key_count: 2,
        source_key_indices: vec![0, 1],
        property: "_MainTex.offset.y".to_owned(),
        class_id: 23,
        script: AnimationBindingPointer {
            file_id: 0,
            path_id: 0,
        },
        pre_infinity: 2,
        post_infinity: 2,
        interpolation: Interpolation::Linear,
        times: vec![0.0, 2.0],
        values: vec![2.0, 0.0],
        in_tangents: None,
        out_tangents: None,
        tangent_modes: vec![],
    }
}

#[test]
fn samples_exact_linear_material_curve() {
    let curve = linear_curve();
    assert_eq!(
        sample_legacy_float_curve(&curve, 1.0, 2.0, false),
        Some(1.0)
    );
    assert_eq!(
        sample_legacy_float_curve(&curve, 3.0, 2.0, false),
        Some(0.0)
    );
    assert_eq!(sample_legacy_float_curve(&curve, 3.0, 2.0, true), Some(1.0));
}

#[test]
fn zero_key_binding_remains_metadata_only() {
    let mut curve = linear_curve();
    curve.source_key_count = 0;
    curve.source_key_indices.clear();
    curve.times.clear();
    curve.values.clear();
    assert_eq!(sample_legacy_float_curve(&curve, 1.0, 2.0, true), None);
}
