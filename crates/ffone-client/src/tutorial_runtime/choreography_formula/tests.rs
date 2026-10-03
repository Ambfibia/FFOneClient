use crate::tutorial_choreography_formula::*;

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 1.0e-4),
        "actual={actual:?}, expected={expected:?}"
    );
}

fn assert_quat_close(actual: Quat, expected: Quat) {
    assert!(
        actual.abs_diff_eq(expected, 1.0e-4) || actual.abs_diff_eq(-expected, 1.0e-4),
        "actual={actual:?}, expected={expected:?}"
    );
}

#[test]
fn euler_oriented_offset_matches_h_r_v_identity() {
    let expression = PositionExpr::OrientedOffset {
        origin: PositionAnchor::Client(ClientVec3::new(563.0, -131.0, 967.0)),
        world_offset: ClientVec3::UP,
        orientation: OrientationBasis::Euler(ClientVec3::new(10.0, 320.0, 0.0)),
        local_offset: ClientVec3::new(0.0, 0.0, -3.0),
    };
    let actual = resolve_position(expression, &mut |_| None, &Default::default()).unwrap();

    let unity_rotation = Quat::from_rotation_y(320_f32.to_radians())
        * Quat::from_rotation_x(10_f32.to_radians())
        * Quat::from_rotation_z(0.0);
    let unity = Vec3::new(563.0, -130.0, 967.0) + unity_rotation * Vec3::new(0.0, 0.0, -3.0);
    assert_vec3_close(actual, unity_to_native_vector(unity));
}

#[test]
fn basic_move_return_camera_stays_above_the_original_start_position() {
    // BasicMoveEvent lines 2683-2688:
    //   kCTargetPosition = StartPos + Vector3.up;
    //   kNewStartPosition = target - Euler(10,270,0) * forward * 7;
    //   TempPosition = target - Euler(10,270,0) * forward * 40;
    let start = ClientVec3::new(547.0, -105.4, 655.0);
    let expression = |distance: f32| PositionExpr::OrientedOffset {
        origin: PositionAnchor::Client(start),
        world_offset: ClientVec3::UP,
        orientation: OrientationBasis::Euler(ClientVec3::new(10.0, 270.0, 0.0)),
        local_offset: ClientVec3::new(0.0, 0.0, -distance),
    };
    let near = resolve_position(expression(7.0), &mut |_| None, &Default::default()).unwrap();
    let far = resolve_position(expression(40.0), &mut |_| None, &Default::default()).unwrap();
    let target_y = start.y + 1.0;

    assert!(near.y > target_y, "near={near:?}, target_y={target_y}");
    assert!(far.y > near.y, "far={far:?}, near={near:?}");
}

#[test]
fn live_entity_axes_preserve_forward_and_right_formulas() {
    let player = Transform {
        translation: Vec3::new(-566.0, -101.0, 665.0),
        rotation: Quat::from_rotation_y(90_f32.to_radians()),
        ..Default::default()
    };
    let expression = PositionExpr::OrientedOffset {
        origin: PositionAnchor::Client(ClientVec3::new(566.0, -101.0, 665.0)),
        world_offset: ClientVec3::ZERO,
        orientation: OrientationBasis::Entity(EntityRef::Player),
        local_offset: ClientVec3::new(-40.0, 0.0, 10.0),
    };
    let actual = resolve_position(
        expression,
        &mut |entity| (entity == EntityRef::Player).then_some(player),
        &Default::default(),
    )
    .unwrap();
    let expected = player.translation
        + (player.rotation * Vec3::X) * -40.0
        + (player.rotation * Vec3::NEG_Z) * 10.0;
    assert_vec3_close(actual, expected);
}

#[test]
fn captured_position_and_yaw_are_stable_after_live_camera_changes() {
    let captured = Transform {
        translation: Vec3::new(10.0, 20.0, 30.0),
        rotation: Quat::from_rotation_y(90_f32.to_radians()),
        ..Default::default()
    };
    let mut captures = TutorialCameraCaptureStore::default();
    captures.capture(CameraCaptureSlot::InfectionEntry, captured);
    captures.capture(CameraCaptureSlot::InfectionReturnStart, captured);

    let position = resolve_position(
        PositionExpr::OrientedOffset {
            origin: PositionAnchor::CapturedCamera(CameraCaptureSlot::InfectionReturnStart),
            world_offset: ClientVec3::ZERO,
            orientation: OrientationBasis::CapturedCameraYaw(CameraCaptureSlot::InfectionEntry),
            local_offset: ClientVec3::new(0.0, 0.0, -6.0),
        },
        &mut |_| None,
        &captures,
    )
    .unwrap();
    assert_vec3_close(
        position,
        captured.translation + (captured.rotation * Vec3::NEG_Z) * -6.0,
    );
}

#[test]
fn look_at_yaw_keeps_authored_pitch_and_uses_reflected_target_yaw() {
    let origin = unity_to_native_vector(Vec3::new(928.0, 40.0, 682.0));
    let rotation = resolve_rotation(
        RotationExpr::LookAtYaw {
            target: ClientVec3::new(994.0, 100.0, 554.0),
            pitch: -30.0,
            yaw_offset: 0.0,
            roll: 0.0,
        },
        origin,
        &mut |_| None,
        &Default::default(),
    )
    .unwrap();
    let direction = Vec3::new(994.0, 100.0, 554.0) - Vec3::new(928.0, 40.0, 682.0);
    let yaw = direction.x.atan2(direction.z).to_degrees();
    assert_quat_close(rotation, native_euler(ClientVec3::new(-30.0, yaw, 0.0)));
}

#[test]
fn missing_inputs_fail_closed_with_typed_identity() {
    assert_eq!(
        resolve_position(
            PositionExpr::Entity(EntityRef::Npc(4000)),
            &mut |_| None,
            &Default::default(),
        ),
        Err(ChoreographyFormulaError::MissingEntity(EntityRef::Npc(
            4000
        )))
    );
    assert_eq!(
        resolve_position(
            PositionExpr::OrientedOffset {
                origin: PositionAnchor::CapturedCamera(CameraCaptureSlot::InfectionReturnStart),
                world_offset: ClientVec3::ZERO,
                orientation: OrientationBasis::Euler(ClientVec3::ZERO),
                local_offset: ClientVec3::ZERO,
            },
            &mut |_| None,
            &Default::default(),
        ),
        Err(ChoreographyFormulaError::MissingCameraCapture(
            CameraCaptureSlot::InfectionReturnStart
        ))
    );
}
