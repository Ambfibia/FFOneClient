use super::*;

#[test]
fn jumppad_packet_uses_special_power_time_and_scaled_velocity() {
    let request = make_jumppad_request(
        Vec3::new(-1.0, 2.0, 3.0),
        Vec3::new(4.0, 13.0, -5.0),
        180.0,
        2,
        13.0,
    );
    assert_eq!(request.client_time, 1_300);
    assert_eq!(request.position, [100, 300, 200]);
    assert_eq!(request.velocity, [-400, -500, 1300]);
    assert_eq!(request.angle, 0);
    assert_eq!(request.key_value, 2);
}
