use super::*;

#[test]
fn tutorial_visual_keeps_the_same_material_reveal_gate_as_editor_and_world() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<WorldAsset>()
        .insert_resource(production_content())
        .init_resource::<NetworkNpcVisualCatalogState0104>()
        .init_resource::<TutorialActorAnimationAssets>()
        .add_systems(Update, spawn_tutorial_actor_visuals);
    {
        let mut state = app
            .world_mut()
            .resource_mut::<NetworkNpcVisualCatalogState0104>();
        state.attempted = true;
        state.catalog = Some(production_visual_catalog());
    }
    let actor = app
        .world_mut()
        .spawn(TutorialActor {
            id: 1005,
            npc_type: 2675,
            team: 2,
            hp: 1_000,
            max_hp: 1_000,
            damaged: false,
            interacting: false,
            invulnerable: false,
        })
        .id();

    app.update();

    let visual = app
        .world()
        .get::<NetworkNpcVisual0104>(actor)
        .expect("tutorial Cerberus must use the shared XDT visual");
    assert!(
        app.world()
            .get::<LegacyCharacterSceneDeferredReveal>(visual.scene)
            .is_some(),
        "tutorial must not expose StandardMaterial or an unbound XDT texture before the shared finalizer"
    );
}
