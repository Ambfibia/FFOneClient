use super::*;

#[test]
fn tutorial_ledge_meshes_reject_wa_and_wd_corner_penetration() {
    let collider_paths = [
        "models/world/tutorial/tile_01_01/c-00410-6599-bf8afd22b95a.glb",
        "models/world/tutorial/tile_01_01/c-00411-6602-d83714d095c7.glb",
        "models/world/tutorial/tile_01_01/c-00412-6605-b6918f910d7e.glb",
        "models/world/tutorial/tile_01_01/c-00413-6608-f652008795a6.glb",
        "models/world/tutorial/tile_01_01/c-00416-6617-59a687d6a70f.glb",
    ];
    let ledges = collider_paths.map(published_map_collider_for_source);
    let colliders = ledges
        .iter()
        .map(|(world_from_local, collider)| (*world_from_local, collider))
        .collect::<Vec<_>>();
    let start = Vec3::new(-555.82, -105.4, 654.2);
    let forward = Vec3::new(-10.18, 0.0, 10.8).normalize();
    let right = Vec3::new(forward.z, 0.0, -forward.x);
    let mut contact = start;
    for _ in 0..100 {
        contact = resolve_authored_wall_motion(contact, forward * 0.08, &colliders);
    }
    assert!(
        contact.distance(start + forward * 8.0) > 5.0,
        "the real tutorial MeshColliders must stop the initial forward run: {contact:?}"
    );

    for (keys, direction) in [
        ("W+A", (forward - right).normalize()),
        ("W+D", (forward + right).normalize()),
    ] {
        let mut feet = contact;
        for frame in 0..200 {
            feet = resolve_authored_wall_motion(feet, direction * 0.08, &colliders);
            let nearby_triangles =
                collect_authored_motion_triangles(feet, Vec3::ZERO, &colliders);
            let residual =
                deepest_authored_capsule_penetration(feet, feet.y, &nearby_triangles, true);
            assert!(
                residual.is_none_or(|push| {
                    push.length() <= AUTHORED_COLLISION_CONTACT_TOLERANCE
                }),
                "{keys} left the character capsule penetrating a tutorial MeshCollider on \
                 frame {frame}: contact={contact:?}, resolved={feet:?}, residual={residual:?}"
            );
            let capsule_center = feet + Vec3::Y * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5);
            assert!(
                ledges.iter().all(|(world_from_local, collider)| {
                    !authored_collider_contains_point(
                        collider,
                        world_from_local.inverse().transform_point3(capsule_center),
                    )
                }),
                "{keys} moved the capsule center inside a tutorial MeshCollider on frame \
                 {frame}: contact={contact:?}, resolved={feet:?}"
            );
        }
    }
}
