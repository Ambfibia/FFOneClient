use super::*;

#[test]
fn tutorial_scene_angle_is_a_direct_unity_heading_and_turns_with_retro_speed() {
    let heading = tutorial_scene_angle_heading(50);
    assert_eq!(heading.degrees(), 50.0);
    assert_ne!(
        heading.degrees(),
        ProtocolYawDegrees::new(50).legacy_heading().degrees()
    );

    let target = heading.native_root_rotation();
    assert!(
        tutorial_cinematic_turn_rotation(Quat::IDENTITY, target, 0.25)
            .abs_diff_eq(target, 0.000_01)
    );
    assert!(
        !tutorial_cinematic_turn_rotation(Quat::IDENTITY, target, 0.125)
            .abs_diff_eq(target, 0.000_01)
    );
}
