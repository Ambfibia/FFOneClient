use super::*;

#[test]
fn plugin_first_update_and_screen_switch_have_no_query_conflicts() {
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
        .add_message::<KeyboardInput>()
        .add_plugins(NativeCharacterCreationUiPlugin);
    app.update();

    let (appearance, name) = {
        let world = app.world_mut();
        let mut appearance = world.query_filtered::<&Visibility, With<AppearanceRoot>>();
        let appearance = *appearance.single(world).unwrap();
        let mut name = world.query_filtered::<&Visibility, With<NameRoot>>();
        let name = *name.single(world).unwrap();
        (appearance, name)
    };
    assert_eq!(appearance, Visibility::Hidden);
    assert_eq!(name, Visibility::Inherited);

    app.world_mut()
        .resource_mut::<CharacterCreationUiModel>()
        .screen = CharacterCreationScreen::Appearance;
    app.update();
    let (appearance, name) = {
        let world = app.world_mut();
        let mut appearance = world.query_filtered::<&Visibility, With<AppearanceRoot>>();
        let appearance = *appearance.single(world).unwrap();
        let mut name = world.query_filtered::<&Visibility, With<NameRoot>>();
        let name = *name.single(world).unwrap();
        (appearance, name)
    };
    assert_eq!(appearance, Visibility::Inherited);
    assert_eq!(name, Visibility::Hidden);
}
