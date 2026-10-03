use super::*;
#[test]
fn vendor_camera_requires_hnpc_and_releases_on_close() {
    let mut app = App::new();
    app.init_resource::<Assets<Image>>()
        .init_resource::<CombiProductionShell0104>()
        .init_resource::<CombiUiState0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .init_resource::<EnchantModeProjection0104>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorModeProjection0104>()
        .add_plugins(ServicePortraitPlugin);
    app.world_mut().resource_mut::<VendorUiState>().phase = VendorLifecyclePhase::Visible;
    app.world_mut()
        .resource_mut::<VendorModeProjection0104>()
        .session
        .requested_npc_id = 9001;
    let root = app
        .world_mut()
        .spawn((
            NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
                npc_id: 9001,
                npc_type: 100,
                hp: 100,
                condition_bit_flag: 0,
                position: [0; 3],
                angle: 0,
                barker_type: 0,
            }),
            GlobalTransform::IDENTITY,
        ))
        .id();
    let mesh = app
        .world_mut()
        .spawn((Mesh3d::default(), ChildOf(root)))
        .id();
    let slot = app
        .world_mut()
        .spawn((
            ServicePortraitSlot::Vendor,
            ImageNode {
                color: Color::NONE,
                ..default()
            },
        ))
        .id();
    let index = ServicePortraitSlot::Vendor.index();
    app.update();
    assert!(
        app.world().resource::<ServicePortraitRuntime>().surfaces[index]
            .camera
            .is_none()
    );
    app.world_mut()
        .entity_mut(root)
        .insert(NetworkHnpcVisual0104 {
            npc_type: 100,
            appearance_index: 0,
            rig_root: mesh,
            generation: 1,
            walk_animation_speed: 1.,
            run_animation_speed: 1.,
        });
    app.update();
    let camera = app.world().resource::<ServicePortraitRuntime>().surfaces[index]
        .camera
        .unwrap();
    let target = app.world().get::<ImageNode>(slot).unwrap().image.clone();
    assert_ne!(target, Handle::default());
    assert_eq!(
        app.world().get::<ImageNode>(slot).unwrap().color,
        Color::WHITE
    );
    let pose = app.world().get::<Transform>(camera).unwrap();
    assert!(
        pose.translation
            .abs_diff_eq(Vec3::new(-0.13680806, 1.1, -0.37587705), 1e-5)
    );
    assert!(Vec3::from(pose.forward()).abs_diff_eq(Vec3::new(0.34202015, 0., 0.9396926), 1e-5));
    let Projection::Perspective(projection) = app.world().get::<Projection>(camera).unwrap()
    else {
        panic!("perspective camera")
    };
    assert_eq!(projection.fov, 70_f32.to_radians());
    assert_eq!(projection.near, 0.2);
    assert_eq!(projection.far, 2.0);
    app.update();
    assert_eq!(
        app.world().resource::<ServicePortraitRuntime>().surfaces[index].camera,
        Some(camera)
    );
    assert_eq!(app.world().get::<ImageNode>(slot).unwrap().image, target);
    app.world_mut()
        .entity_mut(mesh)
        .insert(RenderLayers::layer(0));
    app.update();
    assert!(
        app.world()
            .get::<RenderLayers>(mesh)
            .unwrap()
            .iter()
            .any(|layer| layer == FIRST_LAYER + index)
    );
    app.world_mut()
        .resource_mut::<VendorModeProjection0104>()
        .session
        .requested_npc_id = 9002;
    app.update();
    assert!(app.world().get_entity(camera).is_err());
    assert_eq!(
        app.world().get::<ImageNode>(slot).unwrap().color,
        Color::NONE
    );
    assert!(app.world().get::<RenderLayers>(mesh).is_none());
    app.world_mut()
        .resource_mut::<VendorModeProjection0104>()
        .session
        .requested_npc_id = 9001;
    app.update();
    app.world_mut().resource_mut::<VendorUiState>().phase = VendorLifecyclePhase::Hidden;
    app.update();
    assert!(
        app.world().resource::<ServicePortraitRuntime>().surfaces[index]
            .image
            .is_none()
    );
    assert_eq!(
        app.world().get::<ImageNode>(slot).unwrap().image,
        Handle::default()
    );
}
#[test]
fn waiting_combi_uses_its_independent_wide_camera_framing() {
    let root = GlobalTransform::from_translation(Vec3::new(10.0, 2.0, 3.0));
    let pose = camera_pose(ServicePortraitSlot::CombiWaiting, &root);
    assert!(
        pose.translation
            .abs_diff_eq(Vec3::new(10.0, 2.42, 0.8), 1e-5)
    );
    assert!(Vec3::from(pose.forward()).abs_diff_eq(Vec3::Z, 1e-5));
}
#[test]
fn pose_keeps_root_fallback_and_height_after_look_at() {
    let root = GlobalTransform::from_translation(Vec3::new(10.0, 2.0, 3.0));
    let pose = camera_pose(ServicePortraitSlot::CombiPrimary, &root);
    assert!((pose.translation.y - 2.55).abs() < 1e-5);
    assert!(Vec3::from(pose.forward()).y.abs() < 1e-5);
    let mut unraised = pose.translation;
    unraised.y -= 0.55;
    assert!((unraised.distance(root.translation()) - 1.3).abs() < 1e-5);
    assert!(
        Vec3::from(pose.forward())
            .abs_diff_eq((root.translation() - unraised).normalize(), 1e-5)
    );
}

#[test]
fn surfaces_wait_for_meshes_and_release_images_and_layer_leases() {
    let mut app = App::new();
    app.init_resource::<Assets<Image>>()
        .init_resource::<CombiProductionShell0104>()
        .init_resource::<CombiUiState0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .init_resource::<EnchantModeProjection0104>()
        .add_plugins(ServicePortraitPlugin);
    app.world_mut()
        .resource_mut::<CombiProductionShell0104>()
        .camera_npc_table_id = Some(3219);
    app.world_mut().resource_mut::<CombiUiState0104>().phase = CombiPhase0104::Ready;
    let root = app
        .world_mut()
        .spawn((
            NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
                npc_id: 9001,
                npc_type: 3219,
                hp: 100,
                condition_bit_flag: 0,
                position: [0; 3],
                angle: 0,
                barker_type: 0,
            }),
            GlobalTransform::IDENTITY,
        ))
        .id();
    let slot = app
        .world_mut()
        .spawn((ServicePortraitSlot::CombiPrimary, ImageNode::default()))
        .id();
    app.update();
    assert!(
        app.world().resource::<ServicePortraitRuntime>().surfaces[0]
            .camera
            .is_none()
    );
    let mesh = app
        .world_mut()
        .spawn((
            Mesh3d::default(),
            ChildOf(root),
            RenderLayers::from_layers(&[0, 7]),
        ))
        .id();
    app.update();
    let camera = app.world().resource::<ServicePortraitRuntime>().surfaces[0]
        .camera
        .unwrap();
    let image = app.world().get::<ImageNode>(slot).unwrap().image.clone();
    assert_ne!(image, Handle::default());
    assert_eq!(
        *app.world().get::<RenderLayers>(mesh).unwrap(),
        RenderLayers::from_layers(&[0, 7, FIRST_LAYER])
    );
    app.world_mut().resource_mut::<CombiUiState0104>().phase = CombiPhase0104::Waiting {
        elapsed_seconds: 1.0,
    };
    app.update();
    let runtime = app.world().resource::<ServicePortraitRuntime>();
    assert_eq!(runtime.surfaces[0].camera, Some(camera));
    assert_ne!(runtime.surfaces[0].image, runtime.surfaces[1].image);
    assert!(runtime.surfaces[1].camera.is_some());
    app.world_mut()
        .entity_mut(mesh)
        .insert(RenderLayers::from_layers(&[
            0,
            7,
            8,
            FIRST_LAYER,
            FIRST_LAYER + 1,
        ]));
    app.world_mut().resource_mut::<CombiUiState0104>().phase = CombiPhase0104::Hidden;
    app.update();
    assert!(app.world().get_entity(camera).is_err());
    assert!(
        app.world()
            .resource::<ServicePortraitRuntime>()
            .surfaces
            .iter()
            .all(|s| s.image.is_none() && s.camera.is_none())
    );
    assert_eq!(
        app.world().get::<ImageNode>(slot).unwrap().image,
        Handle::default()
    );
    assert_eq!(
        *app.world().get::<RenderLayers>(mesh).unwrap(),
        RenderLayers::from_layers(&[0, 7, 8])
    );
}
