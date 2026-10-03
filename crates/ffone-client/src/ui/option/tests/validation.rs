use super::*;

#[test]
fn production_deferred_phase_does_not_require_option_assets() {
    let empty_assets = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: empty_assets.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_state(crate::ui_startup::NativeUiStartupPhase::Deferred)
        .add_plugins(OptionUiPlugin);

    app.update();

    assert!(!app.world().contains_resource::<OptionUiAssets>());
    assert_eq!(
        *app.world().resource::<OptionUiAssetGate>(),
        OptionUiAssetGate::default()
    );
    let world = app.world_mut();
    let mut roots = world.query_filtered::<Entity, With<OptionUiRoot>>();
    assert_eq!(roots.iter(world).count(), 0);
}
