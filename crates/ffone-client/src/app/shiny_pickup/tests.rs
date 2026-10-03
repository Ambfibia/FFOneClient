use super::*;
#[test]
fn coco_pickup_retains_exact_three_dimensional_radius() {
    assert!(in_pickup_range(Vec3::ZERO, Vec3::X));
    assert!(!in_pickup_range(Vec3::ZERO, Vec3::Y * 1.001));
    assert!(!in_pickup_range(Vec3::ZERO, Vec3::splat(0.6)));
    assert!(!in_pickup_range(Vec3::ZERO, Vec3::splat(f32::NAN)));
}
