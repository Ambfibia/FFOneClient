use super::*;
use ffone_client::transportation_ui::TransportationUnlocks;

#[test]
fn camera_pose_faces_the_npc_before_applying_world_height() {
    let avatar = GlobalTransform::from(
        Transform::from_xyz(10.0, 2.0, -4.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
    );
    let pose = portrait_transform(&avatar);
    assert!(
        pose.translation
            .abs_diff_eq(Vec3::new(9.5, 3.2, -4.0), 1e-5)
    );
    assert!(Vec3::from(pose.forward()).abs_diff_eq(Vec3::X, 1e-5));
}

#[test]
fn live_portrait_binds_waits_for_meshes_and_releases_only_its_render_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let catalog = TransportationCatalog::open(&locator).unwrap();
    let mut model = TransportationModel::default();
    model
        .open(
            &catalog,
            TransportationOpenContext {
                player: TransportationPlayerSnapshot {
                    position: TransportationWorldPoint::new(4000.0, 120.0, 4000.0),
                    taros: 1000,
                    unlocks: TransportationUnlocks::default(),
                    cursor_was_locked: false,
                },
                target: TransportationTarget::Npc {
                    npc_instance_id: 9001,
                    npc_table_id: 964,
                    npc_position: TransportationWorldPoint::new(4100.0, 100.0, 4100.0),
                    has_move_ok_voice: false,
                },
            },
        )
        .unwrap();
    let mut production = TransportationProductionRuntime::default();
    production.begin_npc(9001, false, TransportationService::Warp, None);
    let mut app = App::new();
    app.insert_resource(model)
        .insert_resource(production)
        .init_resource::<Assets<Image>>()
        .add_plugins(TransportationPortraitPlugin);
    let npc = app
        .world_mut()
        .spawn((
            NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
                npc_id: 9001,
                npc_type: 964,
                hp: 1000,
                condition_bit_flag: 0,
                position: [0; 3],
                angle: 0,
                barker_type: 0,
            }),
            GlobalTransform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        ))
        .id();
    let slot = app
        .world_mut()
        .spawn((
            TransportationPresentationNpcCameraSlot,
            ImageNode::default(),
        ))
        .id();
    app.update();
    assert!(app.world().resource::<PortraitRuntime>().camera.is_none());
    let mesh = app
        .world_mut()
        .spawn((Mesh3d::default(), ChildOf(npc)))
        .id();
    let explicit = app
        .world_mut()
        .spawn((
            Mesh3d::default(),
            ChildOf(npc),
            RenderLayers::from_layers(&[0, 7]),
        ))
        .id();
    app.update();
    let runtime = app.world().resource::<PortraitRuntime>();
    let camera = runtime.camera.unwrap();
    let image = runtime.image.clone().unwrap();
    assert_eq!(app.world().get::<ImageNode>(slot).unwrap().image, image);
    assert_eq!(
        *app.world().get::<RenderLayers>(mesh).unwrap(),
        RenderLayers::from_layers(&[0, PORTRAIT_LAYER])
    );
    assert_eq!(
        *app.world().get::<RenderLayers>(explicit).unwrap(),
        RenderLayers::from_layers(&[0, 7, PORTRAIT_LAYER])
    );
    app.world_mut()
        .entity_mut(npc)
        .insert(GlobalTransform::from_translation(Vec3::new(5.0, 2.0, 3.0)));
    app.update();
    assert_eq!(
        app.world().resource::<PortraitRuntime>().camera,
        Some(camera)
    );
    assert_eq!(
        app.world().resource::<PortraitRuntime>().image.as_ref(),
        Some(&image)
    );
    assert!((app.world().get::<Transform>(camera).unwrap().translation.x - 5.0).abs() < 1e-5);
    // Preserve an unrelated layer writer while removing our own lease.
    app.world_mut()
        .entity_mut(explicit)
        .insert(RenderLayers::from_layers(&[0, 7, 8, PORTRAIT_LAYER]));
    app.world_mut()
        .resource_mut::<TransportationProductionRuntime>()
        .reset();
    app.update();
    assert!(app.world().get_entity(camera).is_err());
    assert!(app.world().resource::<PortraitRuntime>().image.is_none());
    assert_eq!(
        app.world().get::<ImageNode>(slot).unwrap().image,
        Handle::default()
    );
    assert!(app.world().get::<RenderLayers>(mesh).is_none());
    assert_eq!(
        *app.world().get::<RenderLayers>(explicit).unwrap(),
        RenderLayers::from_layers(&[0, 7, 8])
    );
}
