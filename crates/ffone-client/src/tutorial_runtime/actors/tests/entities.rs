use super::*;

#[test]
fn spawn_coordinates_and_angle_follow_protocol_conversion_once() {
    let spawn = spawn(1005, 2671, [65_100, 73_900, -8_200], Some(68));
    let npc_class = production_content()
        .gameplay_npc(spawn.npc_type)
        .unwrap()
        .npc_class;
    let transform = tutorial_spawn_transform(spawn, npc_class);
    assert_vec3_close(transform.translation, Vec3::new(-651.0, -81.0, 739.0));
    assert_vec3_close(
        transform.rotation * Vec3::NEG_Z,
        ProtocolYawDegrees::new(68).native_root_rotation() * Vec3::NEG_Z,
    );
}
