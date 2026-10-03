use super::*;

#[test]
fn scripted_entry_uses_stationary_move_with_the_exact_pose() {
    let request = scripted_entry_move_request([54_700, 65_500, -10_540], 270);
    assert_eq!(request.client_time, 0);
    assert_eq!(request.position, [54_700, 65_500, -10_540]);
    assert_eq!(request.velocity, [0.0; 3]);
    assert_eq!(request.angle, 270);
    assert_eq!(request.key_value, 0);
    assert_eq!(request.speed, 0);
}
