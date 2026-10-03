use super::*;

#[test]
fn escort_attack_faces_npc_and_stops_previous_follow_segment() {
    let mut world = World::new();
    world.init_resource::<NetworkNpcRegistry0104>();
    let attacker = world
        .spawn((
            Transform::default(),
            NetworkNpcMotion0104 {
                destination: Vec3::Z * 5.0,
                speed: 8.0,
                move_style: 0,
            },
        ))
        .id();
    let target = world.spawn(Transform::from_xyz(3.0, 10.0, 0.0)).id();
    world
        .resource_mut::<NetworkNpcRegistry0104>()
        .insert(1, attacker);
    world
        .resource_mut::<NetworkNpcRegistry0104>()
        .insert(2, target);
    face_network_npc_toward_npc(&mut world, 1, 2);
    let transform = world.get::<Transform>(attacker).unwrap();
    assert!(transform.forward().as_vec3().abs_diff_eq(Vec3::X, 0.00001));
    assert_eq!(transform.translation, Vec3::ZERO);
    assert!(world.get::<NetworkNpcMotion0104>(attacker).is_none());
}
