use super::*;

#[test]
fn dexter_ship_actors_use_table_models_textures_scale_and_available_event_clips() {
    use bevy::ecs::system::RunSystemOnce;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Gltf>();
    app.init_asset::<WorldAsset>();
    for role in DexterShipActorRole::ALL {
        let definition = role.definition(&catalog).unwrap().clone();
        assert_eq!(definition.npc_type, role.npc_type());
        let mut changed = definition;
        // Simulate an edited table definition: the spawner must consume the
        // supplied route and texture pairing, not a second role-to-path map.
        let replacement = catalog
            .get(if role == DexterShipActorRole::Dexter {
                701
            } else {
                728
            })
            .unwrap();
        changed.glb = replacement.glb.clone();
        changed.main_texture = replacement.main_texture.clone();
        changed.sub_texture = replacement.sub_texture.clone();
        changed.table_scale = 1.25;
        let expected_path = changed.glb.clone();
        let expected_main = changed.main_texture.clone();
        let expected_sub = changed.sub_texture.clone();
        app.world_mut()
            .run_system_once(move |mut commands: Commands, assets: Res<AssetServer>| {
                spawn_dexter_ship_actor(
                    &mut commands,
                    &assets,
                    role,
                    &changed,
                    Vec3::ZERO,
                    "stand1",
                );
            })
            .unwrap();
        let mut actors = app
            .world_mut()
            .query::<(&DexterShipActor, &NpcSceneTextureOverrides0104)>();
        let (actor, textures) = actors
            .iter(app.world())
            .find(|(actor, _)| actor.role == role)
            .unwrap();
        assert_eq!(
            app.world()
                .resource::<AssetServer>()
                .get_path(actor.gltf.id())
                .unwrap()
                .path()
                .to_str(),
            Some(expected_path.as_str())
        );
        assert_eq!(textures.main_texture, expected_main);
        assert_eq!(textures.sub_texture, expected_sub);
    }
    let mut scenes = app
        .world_mut()
        .query_filtered::<&Transform, With<DexterShipSceneRoot>>();
    assert_eq!(scenes.iter(app.world()).count(), 3);
    for transform in scenes.iter(app.world()) {
        assert_eq!(transform.scale, Vec3::splat(1.25));
        assert_eq!(
            transform.rotation,
            Quat::IDENTITY,
            "cinematic basis must not receive the gameplay half-turn"
        );
    }

    let dexter = DexterShipActorRole::Dexter.definition(&catalog).unwrap();
    assert_eq!(dexter.logical_name, "npc_dexter2");
    let model = gltf::Gltf::open(root.join(&dexter.glb)).unwrap();
    let clips: BTreeSet<_> = model.animations().filter_map(|clip| clip.name()).collect();
    for (name_scene, elapsed, expected) in [
        (true, 0.0, "stand1"),
        (true, 2.0, "observe"),
        (true, 8.0, "observe"),
        (true, 16.5, "cutscene_stand_1"),
        (false, 0.0, "excellent"),
        (false, 19.0, "excellent"),
        (false, 27.4, "deedeeno"),
        (false, 28.9, "busyrun"),
    ] {
        assert_eq!(dexter_ship_dexter_clip(name_scene, elapsed), expected);
        assert!(
            clips.contains(expected),
            "cutscene requests missing {expected}"
        );
    }
    assert_eq!(
        DexterShipActorRole::DeeDee
            .definition(&catalog)
            .unwrap()
            .logical_name,
        "npc_deedee"
    );
}

#[test]
fn tutorial_event_camera_starts_from_live_transform_not_stale_temp_position() {
    let camera = ffone_client::tutorial_choreography_runtime::TutorialCameraPresentation {
        mode: CameraMode::EventCameraControl,
        resolved_target: Some(Vec3::ZERO),
        resolved_target_rotation: Some(Quat::IDENTITY),
        resolved_stored_start: Some(Vec3::splat(100.0)),
        distance: 5.0,
        ..default()
    };
    let (mut app, camera_entity) =
        tutorial_camera_test_app(Transform::from_xyz(10.0, 0.0, 10.0), camera);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.5));
    app.update();

    let transform = app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .unwrap();
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(5.0, 0.0, 2.5), 0.0001)
    );
}
