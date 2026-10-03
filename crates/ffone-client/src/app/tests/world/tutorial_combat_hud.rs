use super::*;
use crate::app::tutorial_combat::collect_tutorial_actor_events;
use crate::app::tutorial_session::TutorialProjectileRandomStream;
use ffone_client::tutorial_actors::{
    TutorialActorCombatConfig, TutorialActorIssueQueue, TutorialActorStandRandomStream,
    apply_tutorial_actor_commands,
};
use ffone_client::tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn};

#[test]
fn tutorial_kills_award_thirty_fm_once_before_any_mission_is_accepted() {
    let mut app = App::new();
    app.insert_resource(runtime_test_mission_content())
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialMissionRuntime>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialActorEventQueue>()
        .init_resource::<TutorialActorRegistry>()
        .init_resource::<TutorialActorIssueQueue>()
        .init_resource::<TutorialActorCombatConfig>()
        .init_resource::<TutorialActorStandRandomStream>()
        .init_resource::<TutorialEffectRuntime>()
        .init_resource::<TutorialProjectileRandomStream>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(
            Update,
            (apply_tutorial_actor_commands, collect_tutorial_actor_events).chain(),
        );
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            GlobalTransform::IDENTITY,
            LegacyAvatarEnvironmentState::default(),
        ))
        .id();
    // The three first Spawn followed by the Kerber: no accepted task required.
    for (index, (id, npc_type)) in [(1001, 2674), (1002, 2674), (1003, 2674), (1005, 2675)]
        .into_iter()
        .enumerate()
    {
        {
            let mut commands = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
            commands.spawn(TutorialNpcSpawn::new(
                id,
                npc_type,
                LegacySpawnPosition::centiunits(0, 0, 0),
                None,
            ));
            commands.damage(id, 1);
        }
        app.update();
        assert_eq!(
            app.world()
                .resource::<TutorialMissionRuntime>()
                .fusion_matter,
            index as i32 * 30
        );
        assert_eq!(
            app.world()
                .get::<LegacyAvatarEnvironmentState>(player)
                .unwrap()
                .local_combat_timeout_remaining_seconds,
            5.0
        );
        {
            let mut commands = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
            commands.damage(id, i32::MAX);
            commands.damage(id, i32::MAX);
        }
        app.update();
        let mission = app.world().resource::<TutorialMissionRuntime>();
        assert_eq!(mission.fusion_matter, (index as i32 + 1) * 30);
        assert!(mission.active_tasks.is_empty());
        app.update();
        assert_eq!(
            app.world()
                .resource::<TutorialMissionRuntime>()
                .fusion_matter,
            (index as i32 + 1) * 30
        );
    }
    let mut commands = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
    commands.spawn(TutorialNpcSpawn::new(
        1004,
        2674,
        LegacySpawnPosition::centiunits(0, 0, 0),
        None,
    ));
    commands.play_pose(1004, "death", true);
    commands.delete(1004);
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .fusion_matter,
        120,
        "scripted death/removal must not award player-kill FM"
    );
}

#[test]
fn tutorial_combat_hud_follows_activity_and_the_live_event_scene_flag() {
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::Tutorial))
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialNanoGameplayState>()
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<WorldNanoCooldownRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .insert_resource(runtime_test_mission_content())
        .init_resource::<PlayerWeaponAnimationCatalog>()
        .add_systems(Update, sync_tutorial_action_gate);
    app.world_mut()
        .resource_mut::<TutorialSession>()
        .init_chapter(1)
        .unwrap();
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            LegacyAvatarActionContext::default(),
            LegacyAvatarEnvironmentState::default(),
        ))
        .id();
    app.update();
    assert!(
        !app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .combat_condition
    );
    app.world_mut()
        .get_mut::<LegacyAvatarEnvironmentState>(player)
        .unwrap()
        .observe_local_combat();
    app.update();
    assert!(
        app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .combat_condition
    );
    for event_scene in [true, false] {
        app.world_mut()
            .resource_mut::<TutorialChoreographyPresentation>()
            .event_scene = event_scene;
        app.update();
        let context = app
            .world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap();
        assert_eq!(context.tutorial_event, event_scene);
        assert!(
            context.combat_condition,
            "cutscene presentation must not erase combat activity"
        );
    }
    app.world_mut()
        .get_mut::<LegacyAvatarEnvironmentState>(player)
        .unwrap()
        .local_combat_timeout_remaining_seconds = 0.0;
    app.update();
    assert!(
        !app.world()
            .get::<LegacyAvatarActionContext>(player)
            .unwrap()
            .combat_condition,
        "the combat chapter must not hold the overlay after the timeout"
    );
}
