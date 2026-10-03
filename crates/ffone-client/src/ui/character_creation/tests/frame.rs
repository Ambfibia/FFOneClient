use super::*;

#[test]
fn held_camera_button_emits_one_duration_aware_action_each_frame() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_millis(100),
        ))
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins(NativeCharacterCreationUiPlugin);
    app.update();
    {
        let mut model = app.world_mut().resource_mut::<CharacterCreationUiModel>();
        model.visible = true;
        model.screen = CharacterCreationScreen::Appearance;
    }
    let rotate_left = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &CharacterCreationButton)>();
        query
            .iter(world)
            .find_map(|(entity, button)| {
                (button.0
                    == CharacterCreationControl::Camera(
                        CharacterCreationCameraAction::RotateLeft,
                    ))
                .then_some(entity)
            })
            .unwrap()
    };
    app.world_mut()
        .entity_mut(rotate_left)
        .insert(Interaction::Pressed);
    for _ in 0..2 {
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<CharacterCreationUiOutbox>()
                .drain()
                .collect::<Vec<_>>(),
            vec![CharacterCreationUiAction::Camera {
                action: CharacterCreationCameraAction::RotateLeft,
                delta: Duration::from_millis(100),
            }]
        );
    }
}
