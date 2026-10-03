use crate::tutorial_overlay_ui::*;
use bevy::asset::AssetPlugin;

#[test]
fn hidden_overlay_cannot_be_revived_by_stale_content() {
    let mut model = TutorialOverlayUiModel::retrobution_reference_frame();
    model.visible = false;
    assert!(model.tutorial.has_content());
    assert!(!model.is_renderable());
    model.hide_and_clear();
    assert_eq!(model, TutorialOverlayUiModel::default());
    assert!(!model.is_renderable());
}

#[test]
fn tutorial_overlay_never_blocks_underlying_mission_buttons() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_systems(Startup, spawn_tutorial_overlay);
    app.update();

    let root = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialOverlayRoot>>()
        .single(app.world())
        .expect("tutorial overlay root");
    let mut pending = vec![root];
    while let Some(entity) = pending.pop() {
        assert_eq!(
            app.world().get::<Pickable>(entity),
            Some(&Pickable::IGNORE),
            "tutorial overlay entity {entity:?} intercepted mission input"
        );
        if let Some(children) = app.world().get::<Children>(entity) {
            pending.extend(children.iter());
        }
    }
}
