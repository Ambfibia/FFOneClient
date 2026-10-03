use super::transportation::SkywayTraversalMotion;
use super::*;

#[test]
fn segment_distance_becomes_one_second_motion_speed() {
    let mut position = Vec3::ZERO;
    let mut motion = SkywayTraversalMotion::new(position);
    motion.retarget(&mut position, Vec3::new(12.0, 5.0, 0.0));
    let mut transform = Transform::from_translation(position);

    motion.advance(&mut transform, 0.25);

    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(3.0, 1.25, 0.0), 0.000_01)
    );
    motion.advance(&mut transform, 0.75);
    assert!(transform.translation.abs_diff_eq(motion.target, 0.000_01));
}

#[test]
fn retarget_finishes_a_prior_segment_when_the_avatar_lags_over_five_units() {
    let mut position = Vec3::ZERO;
    let mut motion = SkywayTraversalMotion::new(Vec3::new(10.0, 0.0, 0.0));

    motion.retarget(&mut position, Vec3::new(20.0, 0.0, 0.0));

    assert_eq!(position, Vec3::new(10.0, 0.0, 0.0));
    assert_eq!(motion.target, Vec3::new(20.0, 0.0, 0.0));
    assert_eq!(motion.segment_speed, 10.0);
}
