use super::*;

#[test]
fn plugin_startup_and_first_update_have_no_query_conflicts() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();
    assert!(
        app.world()
            .get_resource::<CharacterSelectionUiModel>()
            .is_some()
    );
    let root_count = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<NativeCharacterSelectionRoot>>();
        query.iter(world).count()
    };
    assert_eq!(root_count, 1);
    let modal_target = {
        let world = app.world_mut();
        let mut query = world
            .query_filtered::<(&UiTargetCamera, Option<&ChildOf>), With<SelectionDeleteModal>>(
            );
        let (target, parent) = query.single(world).unwrap();
        assert!(parent.is_none(), "modal must be a root UI node");
        target.0
    };
    let modal_order = app.world().get::<Camera>(modal_target).unwrap().order;
    assert_eq!(modal_order, CHARACTER_SELECTION_MODAL_CAMERA_ORDER);
    let (delete, delete_label) = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Children), With<SelectionDelete>>();
        let (delete, children) = query.single(world).unwrap();
        (delete, children[0])
    };
    let foreground_root = app.world().get::<ChildOf>(delete).unwrap().parent();
    let foreground_target = app
        .world()
        .get::<UiTargetCamera>(foreground_root)
        .expect("DELETE must live in the foreground root")
        .0;
    assert_eq!(
        app.world().get::<Camera>(foreground_target).unwrap().order,
        CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER
    );
    assert_eq!(
        app.world().get::<FocusPolicy>(delete_label),
        Some(&FocusPolicy::Pass),
        "the label must not steal the center of the delete button"
    );
    assert_eq!(
        app.world().get::<Pickable>(delete_label),
        Some(&Pickable::IGNORE)
    );
    let title_color = {
        let world = app.world_mut();
        let mut query = world.query::<(&Text, &TextColor)>();
        query
            .iter(world)
            .find_map(|(text, color)| (text.0 == "SELECT A CHARACTER").then_some(color.0))
            .unwrap()
    };
    assert_eq!(
        title_color,
        Color::srgb(0.0, 0.0, 0.405_109_5),
        "button-state binding must not recolor non-button labels"
    );
}
