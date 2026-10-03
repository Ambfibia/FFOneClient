use super::*;

#[test]
fn billboard_mode_matches_the_legacy_values() {
    assert_eq!(BillboardMode::from_legacy(0), BillboardMode::Camera);
    assert_eq!(BillboardMode::from_legacy(1), BillboardMode::Up);
    assert_eq!(BillboardMode::from_legacy(2), BillboardMode::RigidCamera);
    assert_eq!(BillboardMode::from_legacy(3), BillboardMode::Center);
    assert_eq!(BillboardMode::from_legacy(4), BillboardMode::RigidCenter);
}
