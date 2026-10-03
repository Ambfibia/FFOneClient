use super::*;

#[test]
fn legacy_sample_height_clamps_unity_coordinates_and_reflects_native_x() {
    let collider = legacy_height_test_collider();
    let owner_and_root = GlobalTransform::from(Transform::from_xyz(100.0, -30.0, 200.0));

    // Moving left in native space moves toward positive Unity terrain X.
    let reflected_quarter = require_legacy_height(collider.legacy_interpolated_height(
        &owner_and_root,
        99.5,
        200.0,
    ));
    assert!((reflected_quarter - -27.5).abs() <= f32::EPSILON);

    let clamped_h11 = require_legacy_height(collider.legacy_interpolated_height(
        &owner_and_root,
        -1000.0,
        1000.0,
    ));
    let clamped_h00 = require_legacy_height(collider.legacy_interpolated_height(
        &owner_and_root,
        1000.0,
        -1000.0,
    ));
    assert!((clamped_h11 - 10.0).abs() <= f32::EPSILON);
    assert!((clamped_h00 - -30.0).abs() <= f32::EPSILON);
}
