use super::*;

#[test]
fn ordinary_world_action_readiness_waits_for_native_collider() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialNanoGameplayState>()
        .init_resource::<MissionUiModel>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .insert_resource(runtime_test_mission_content())
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .add_systems(Update, sync_tutorial_action_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::World);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            LegacyAvatarActionContext {
                ready_for_play: true,
                ..default()
            },
            LegacyWorldColliderPending,
        ))
        .id();

    app.update();

    assert!(
        !app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .ready_for_play
    );

    app.world_mut()
        .entity_mut(player)
        .remove::<LegacyWorldColliderPending>();
    app.update();

    assert!(
        app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .ready_for_play
    );
}

#[test]
fn tutorial_action_readiness_recovers_after_native_collider_admission() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialNanoGameplayState>()
        .init_resource::<MissionUiModel>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .insert_resource(runtime_test_mission_content())
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .add_systems(Update, sync_tutorial_action_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            LegacyAvatarActionContext::default(),
            LegacyWorldColliderPending,
        ))
        .id();

    app.update();
    assert!(
        !app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .ready_for_play
    );

    app.world_mut()
        .entity_mut(player)
        .remove::<LegacyWorldColliderPending>();
    app.update();

    assert!(
        app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .ready_for_play,
        "tutorial input must become ready on the frame after world admission"
    );
}
