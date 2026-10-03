use super::*;

#[test]
fn exact_follow_offset_and_unclamped_legacy_blend_are_preserved() {
    let owner = Transform::from_xyz(2.0, 3.0, 4.0)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    let target = tutorial_nano_follow_target(&owner);
    let expected =
        owner.translation + owner.rotation * Vec3::new(0.7, 0.0, 0.0) + Vec3::Y * 1.12;
    assert!(target.abs_diff_eq(expected, 0.000_01));
    assert!(
        advance_tutorial_nano_follow(Vec3::ZERO, target, 1.0 / 60.0)
            .abs_diff_eq(target * 0.05, 0.000_01)
    );
}
