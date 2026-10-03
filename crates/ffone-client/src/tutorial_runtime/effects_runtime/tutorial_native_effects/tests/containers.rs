use super::*;

#[test]
fn hermite_curve_uses_serialized_tangents() {
    let curve = AnimationCurve(vec![
        CurveKey {
            time: 0.0,
            value: 0.0,
            in_slope: 0.0,
            out_slope: 2.0,
        },
        CurveKey {
            time: 1.0,
            value: 1.0,
            in_slope: 0.0,
            out_slope: 0.0,
        },
    ]);
    assert!((curve.evaluate(0.5, 9.0) - 0.75).abs() < 1e-6);
}

#[test]
fn emitted_particle_offset_is_not_rotated_or_scaled_with_the_prefab() {
    let emitter_world_position = Vec3::new(10.0, -2.0, 5.0);
    let generated_position = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(
        native_emitted_particle_position(emitter_world_position, generated_position),
        Vec3::new(11.0, 0.0, 8.0)
    );

    let transformed = GlobalTransform::from(
        Transform::from_translation(emitter_world_position)
            .with_rotation(Quat::from_rotation_y(1.2))
            .with_scale(Vec3::splat(4.0)),
    )
    .transform_point(generated_position);
    assert_ne!(
        transformed,
        native_emitted_particle_position(emitter_world_position, generated_position),
        "the former full-transform path contradicted ParticleEmitterController.Emit"
    );
}

#[test]
fn serialized_particle_vectors_cross_the_unity_to_native_boundary_once() {
    let object = serde_json::json!({
        "force": { "x": 2.5, "y": -3.0, "z": 4.25 }
    });
    assert_eq!(
        native_vector3(&object, "force"),
        Ok(Vec3::new(-2.5, -3.0, 4.25))
    );

    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let compiled = compile_effect_plan(771, library.effects.get(&771).unwrap());
    let plan = compiled
        .plan
        .as_ref()
        .unwrap_or_else(|| panic!("effect 771 blockers: {:#?}", compiled.blockers));
    let emitter = plan
        .emitters
        .iter()
        .find(|emitter| emitter.initial_velocity.x.abs() > 0.01)
        .expect("ES771 asymmetric Unity-space initial velocity");

    let mut unity_random = NativeParticleRandomStream::with_seed(0x7510_0771);
    let mut native_random = NativeParticleRandomStream::with_seed(0x7510_0771);
    let (unity_position, unity_velocity) =
        legacy_particle_initial_state_in_unity(emitter, &mut unity_random);
    let (native_position, native_velocity) =
        legacy_particle_initial_state(emitter, &mut native_random);
    assert!(native_position.abs_diff_eq(unity_to_native_vector(unity_position), 1.0e-6));
    assert!(native_velocity.abs_diff_eq(unity_to_native_vector(unity_velocity), 1.0e-6));
    assert!(unity_velocity.x.abs() > 1.0e-5);
    assert!((native_velocity.x + unity_velocity.x).abs() < 1.0e-6);
}
