use super::*;

#[test]
fn pending_icon_keeps_its_strong_handle_without_a_preloaded_owner() {
    let root = tempfile::tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>();
    let checker = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    app.insert_resource(VendorUiAssets {
        images: std::array::from_fn(|_| Handle::default()),
        font: Handle::default(),
        service_font: Handle::default(),
        missing_checker: checker.clone(),
    })
    .add_systems(
        Update,
        |server: Res<AssetServer>,
         assets: Res<VendorUiAssets>,
         mut icons: Query<(&mut Node, &mut ImageNode)>| {
            for (mut node, image) in &mut icons {
                bind_vendor_icon(
                    &mut node,
                    Some(image),
                    &VendorPresentationIcon0104::Resolved(
                        VendorIconRef::new("icons/items/pending.png").unwrap(),
                    ),
                    &server,
                    &assets,
                    Color::WHITE,
                );
            }
        },
    );
    let entity = app
        .world_mut()
        .spawn((Node::default(), ImageNode::default()))
        .id();
    app.update();
    let image = app.world().get::<ImageNode>(entity).unwrap();
    assert_ne!(
        image.image, checker,
        "a pending load is not a missing catalog icon"
    );
    assert_ne!(image.image, Handle::<Image>::default());
    assert_eq!(
        app.world()
            .resource::<AssetServer>()
            .get_path(image.image.id())
            .unwrap()
            .path(),
        std::path::Path::new("icons/items/pending.png")
    );
}
