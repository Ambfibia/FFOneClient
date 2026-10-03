use super::*;

#[test]
fn foster_key_has_a_bound_looping_rotation_clip() {
    let registry: serde_json::Value =
        ffone_client_foundation::asset_tables::character_models(&asset_root()).unwrap();
    let entry = registry["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == "npc/npc_key")
        .unwrap();
    assert!(
        entry["animations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "stand1")
    );
    let bytes = fs::read(asset_root().join(entry["glb"].as_str().unwrap())).unwrap();
    assert_eq!(
        blake3::hash(&bytes).to_hex().as_str(),
        entry["glbBlake3"].as_str().unwrap()
    );
    let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
    let clip = gltf
        .animations()
        .find(|clip| clip.name() == Some("stand1"))
        .unwrap();
    let rotation = clip
        .channels()
        .find(|channel| {
            channel.target().node().name() == Some("icekey_ring NonAccum")
                && channel.target().property() == gltf::animation::Property::Rotation
        })
        .expect("the rotating key pivot must be bound by the published clip");
    let reader = rotation.reader(|_| gltf.blob.as_deref());
    let times: Vec<_> = reader.read_inputs().unwrap().collect();
    let gltf::animation::util::ReadOutputs::Rotations(values) = reader.read_outputs().unwrap()
    else {
        panic!("key rotation channel has no quaternion output");
    };
    let values: Vec<_> = values.into_f32().map(Quat::from_array).collect();
    assert!((times.last().unwrap() - 1.33).abs() < 0.01);
    assert!(
        values.iter().any(|value| value.dot(values[0]).abs() < 0.8),
        "stand1 must rotate the key, not retain a static pose"
    );
    assert!(values.first().unwrap().dot(*values.last().unwrap()).abs() > 0.999);
    let document: serde_json::Value = serde_json::from_slice(
        &bytes[20..20 + u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize],
    )
    .unwrap();
    assert_eq!(document["animations"][0]["extras"]["loop"], true);
}

#[test]
fn fast_scripted_platform_keeps_rider_at_current_pose() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .init_resource::<NativeTerrainSpatialRegistry>()
        .init_resource::<AuthoredColliderSpatialIndex>()
        .add_systems(
            Update,
            (
                sync_authored_collider_world_bounds,
                sync_runtime_attached_authored_collider_transforms,
                sync_authored_collider_spatial_index,
                resolve_authored_world_ground,
            )
                .chain(),
        );
    let pivot = app
        .world_mut()
        .spawn((Transform::IDENTITY, GlobalTransform::IDENTITY))
        .id();
    let geometry = closed_vehicle_test_collider();
    let bounds =
        AuthoredColliderWorldBounds::from_collider(&geometry, &GlobalTransform::IDENTITY)
            .unwrap();
    let collider = app
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            GlobalTransform::IDENTITY,
            ChildOf(pivot),
            geometry,
            bounds,
            NativeWorldDynamicObjectRange,
        ))
        .id();
    let player = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 2.0, 0.0),
            LegacyPlayerController::from_baseline_table(),
            NativeWorldGroundSupport {
                collider,
                last_world_from_local: Mat4::IDENTITY,
                surface_normal: Vec3::Y,
            },
        ))
        .id();
    // Leave GlobalTransform at the preceding rendered pose, as it is
    // during Update. Exercise both directions, reversals and fast steps.
    for step in [0.5, 1.5, -0.5, -1.5] {
        for _ in 0..30 {
            app.world_mut()
                .get_mut::<Transform>(pivot)
                .unwrap()
                .translation
                .y += step;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(1.0 / 30.0));
            app.update();
            let expected = app.world().get::<Transform>(pivot).unwrap().translation.y + 2.0;
            let feet = app.world().get::<Transform>(player).unwrap().translation.y;
            assert!(
                (feet - expected).abs() < 0.0001,
                "rider {feet}, platform top {expected}"
            );
            assert!(
                app.world()
                    .get::<NativeWorldGroundSupport>(player)
                    .is_some()
            );
        }
    }
}
