use super::*;

#[test]
fn music_toggle_uses_clean_normal_and_on_normal_texture_semantics() {
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
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();
    let assets = app.world().resource::<CharacterSelectionAssets>().clone();
    let image = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&ImageNode, With<SelectionMusicToggle>>();
        query.single(world).unwrap().image.clone()
    };
    assert_eq!(image, assets.music_toggle_off);

    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .music_enabled = false;
    app.update();
    let image = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&ImageNode, With<SelectionMusicToggle>>();
        query.single(world).unwrap().image.clone()
    };
    assert_eq!(image, assets.music_toggle_on);
}
