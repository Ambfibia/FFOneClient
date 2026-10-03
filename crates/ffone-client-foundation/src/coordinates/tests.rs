use super::*;

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 0.000_01),
        "actual={actual:?}, expected={expected:?}"
    );
}

fn assert_quat_close(actual: Quat, expected: Quat) {
    // q and -q encode the same rotation. Component comparison avoids the
    // acos precision loss of angle_between for an already identical q.
    assert!(
        actual.abs_diff_eq(expected, 0.000_01) || actual.abs_diff_eq(-expected, 0.000_01),
        "actual={actual:?}, expected={expected:?}"
    );
}

#[test]
fn h_basis_is_exact_involution_and_reverses_handedness() {
    assert_eq!(unity_to_native_vector(Vec3::ZERO), Vec3::ZERO);
    assert_eq!(unity_to_native_vector(Vec3::X), Vec3::NEG_X);
    assert_eq!(unity_to_native_vector(Vec3::Y), Vec3::Y);
    assert_eq!(unity_to_native_vector(Vec3::Z), Vec3::Z);

    let value = Vec3::new(12.5, -3.25, 9.0);
    assert_eq!(native_to_unity_vector(unity_to_native_vector(value)), value);
    assert_eq!(unity_to_native_vector(value).length(), value.length());
    assert_eq!(
        unity_to_native_scale(Vec3::new(1.69, 0.75, 2.0)),
        Vec3::new(1.69, 0.75, 2.0)
    );

    let reflected_cross =
        unity_to_native_vector(Vec3::X).cross(unity_to_native_vector(Vec3::Y));
    assert_eq!(reflected_cross, -unity_to_native_vector(Vec3::Z));
}

#[test]
fn protocol_basis_matches_coordutil_then_h() {
    assert_eq!(ProtocolPosition::new([100, 0, 0]).to_native(), Vec3::NEG_X);
    assert_eq!(ProtocolPosition::new([0, 100, 0]).to_native(), Vec3::Z);
    assert_eq!(ProtocolPosition::new([0, 0, 100]).to_native(), Vec3::Y);
}

#[test]
fn positions_and_both_velocity_encodings_round_trip() {
    let position = ProtocolPosition::new([125, 375, -250]);
    let native_position = position.to_native();
    assert_eq!(native_position, Vec3::new(-1.25, -2.5, 3.75));
    assert_eq!(ProtocolPosition::from_native(native_position), position);

    let movement = ProtocolMoveVelocity::new([1.5, 3.25, -2.0]);
    let native_movement = movement.to_native();
    assert_eq!(native_movement, Vec3::new(-1.5, -2.0, 3.25));
    assert_eq!(ProtocolMoveVelocity::from_native(native_movement), movement);

    let jump = ProtocolScaledVelocity::new([150, 325, -200]);
    let native_jump = jump.to_native();
    assert_eq!(native_jump, Vec3::new(-1.5, -2.0, 3.25));
    assert_eq!(ProtocolScaledVelocity::from_native(native_jump), jump);
}

#[test]
fn quaternion_rule_is_exact_h_r_h_for_basis_vectors() {
    let unity_rotation = Quat::from_rotation_y(63.0_f32.to_radians());
    let native_rotation = unity_to_native_rotation(unity_rotation);
    for basis in [Vec3::X, Vec3::Y, Vec3::Z] {
        assert_vec3_close(
            native_rotation * unity_to_native_vector(basis),
            unity_to_native_vector(unity_rotation * basis),
        );
    }
    assert_quat_close(
        native_rotation,
        Quat::from_rotation_y((-63.0_f32).to_radians()),
    );
}

#[test]
fn packet_yaw_root_yaw_and_model_forward_have_one_consistent_sign() {
    for packet_degrees in [-180, -90, 0, 90, 180] {
        let packet = ProtocolYawDegrees::new(packet_degrees);
        let heading = packet.legacy_heading();
        assert_eq!(heading.to_protocol(), packet);

        // H * (Unity heading applied to legacy +Z).
        let expected_native_forward = unity_to_native_vector(
            Quat::from_rotation_y(heading.degrees().to_radians()) * Vec3::Z,
        );

        // A native gameplay root uses Bevy's conventional -Z forward.
        assert_vec3_close(heading.native_forward(), expected_native_forward);
        assert_vec3_close(
            packet.native_root_rotation() * Vec3::NEG_Z,
            expected_native_forward,
        );

        // The exported model itself still has authored +Z forward, so its
        // child half-turn produces the same physical direction.
        let model_world_rotation =
            packet.native_root_rotation() * native_model_forward_child_rotation();
        assert_vec3_close(model_world_rotation * Vec3::Z, expected_native_forward);

        assert_quat_close(
            packet.native_root_rotation(),
            Quat::from_rotation_y((-packet_degrees as f32).to_radians()),
        );
    }
}

#[test]
fn outgoing_yaw_keeps_the_exact_legacy_truncation_point() {
    assert_eq!(
        LegacyUnityHeadingDegrees::new(270.9)
            .to_protocol()
            .degrees(),
        90
    );
    assert_eq!(
        LegacyUnityHeadingDegrees::new(-90.5)
            .to_protocol()
            .degrees(),
        -270
    );
}

#[test]
fn character_spawn_policy_separates_lossless_artifact_trs_from_runtime_root_trs() {
    let penguin_artifact = Transform {
        translation: Vec3::new(-1.886_962_2, 0.0, 0.0),
        rotation: Quat::from_rotation_y(0.75),
        scale: Vec3::splat(1.3),
    };
    assert_eq!(
        LegacyCharacterRootPolicy::Npc { table_scale: 1.5 }.resolve_root(penguin_artifact),
        Transform::from_scale(Vec3::splat(1.5))
    );

    let coco_artifact = Transform {
        translation: Vec3::new(4.0, 2.0, -3.0),
        rotation: Quat::from_rotation_x(0.25),
        scale: Vec3::splat(1.350_000_023_841_858),
    };
    assert_eq!(
        LegacyCharacterRootPolicy::Nano.resolve_root(coco_artifact),
        Transform::from_scale(coco_artifact.scale)
    );
    assert_eq!(
        LegacyCharacterRootPolicy::Player.resolve_root(coco_artifact),
        Transform::IDENTITY
    );
}

#[test]
fn character_spawn_policy_rejects_invalid_scale_instead_of_hiding_it() {
    let valid = Transform::from_scale(Vec3::splat(1.35));
    assert_eq!(
        LegacyCharacterRootPolicy::Npc { table_scale: 0.0 }.try_resolve_root(valid),
        Err(LegacyCharacterRootError::InvalidNpcTableScale)
    );
    assert_eq!(
        LegacyCharacterRootPolicy::Nano
            .try_resolve_root(Transform::from_scale(Vec3::new(1.0, -1.0, 1.0))),
        Err(LegacyCharacterRootError::NonPositiveAuthoredScale)
    );
    assert_eq!(
        LegacyCharacterRootPolicy::Player.try_resolve_root(Transform {
            rotation: Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0),
            ..Transform::IDENTITY
        }),
        Err(LegacyCharacterRootError::InvalidAuthoredTransform)
    );
    assert_eq!(
        LegacyCharacterRootPolicy::Player.try_resolve_root(Transform {
            rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 2.0),
            ..Transform::IDENTITY
        }),
        Err(LegacyCharacterRootError::InvalidAuthoredTransform)
    );
}
