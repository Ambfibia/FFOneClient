//! Exercise real resident UI constructors through repeated character-selection
//! boundaries without a GPU or network session.
use bevy::{prelude::*, state::state::StateTransition};
use ffone_client::{
    group_ui::GroupUiPlugin,
    mission_ui::{MissionUiPlugin, MissionUiRoot},
    shared_input_ui::SharedInputUiPlugin,
    ui_startup::NativeUiStartupPhase,
};

fn transition(app: &mut App, phase: NativeUiStartupPhase) {
    app.world_mut()
        .resource_mut::<NextState<NativeUiStartupPhase>>()
        .set(phase);
    app.world_mut().run_schedule(StateTransition);
}

fn ui_entities(world: &mut World) -> Vec<Entity> {
    let mut entities: Vec<_> = world
        .query_filtered::<Entity, With<Node>>()
        .iter(world)
        .collect();
    entities.sort();
    entities
}

#[test]
fn resident_ui_is_created_once_across_character_changes() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
    ))
    .init_asset::<Image>()
    .init_asset::<Font>()
    .init_asset::<AudioSource>()
    .insert_state(NativeUiStartupPhase::Deferred)
    .add_plugins((GroupUiPlugin, MissionUiPlugin, SharedInputUiPlugin));

    transition(&mut app, NativeUiStartupPhase::CharacterSelection);
    assert!(ui_entities(app.world_mut()).is_empty());
    transition(&mut app, NativeUiStartupPhase::Gameplay);
    let first = ui_entities(app.world_mut());
    assert!(
        first.len() > 100,
        "production constructors must actually run"
    );
    let mission = app
        .world_mut()
        .query_filtered::<Entity, With<MissionUiRoot>>()
        .single(app.world())
        .unwrap();

    for _ in 0..8 {
        transition(&mut app, NativeUiStartupPhase::CharacterSelection);
        transition(&mut app, NativeUiStartupPhase::CharacterCreation);
        transition(&mut app, NativeUiStartupPhase::Gameplay);
        let current = ui_entities(app.world_mut());
        assert_eq!(
            current.len(),
            first.len(),
            "resident UI node count grew after character change"
        );
        assert_eq!(
            current, first,
            "returning to gameplay must reuse every resident node, without duplicating windows"
        );
        assert!(app.world().get_entity(mission).is_ok());
    }
}
