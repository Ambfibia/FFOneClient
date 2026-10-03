use super::*;
use ffone_client::race_ui::hud::RaceHudState;

#[test]
fn resurrection_context_allows_the_server_to_resolve_an_instance_respawn() {
    let mut app = App::new();
    let mut status = RuntimeStatus::default();
    status.player_id = Some(77);
    status.hp = Some(0);
    status.map_number = Some(14);
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(runtime_test_mission_content())
        .init_resource::<GameplayUiModel>()
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<SkillBuffUiModel>()
        .init_resource::<LocalInventoryRuntime>()
        .init_resource::<NanoFreeTuningModel>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<ResurrectUiModel>()
        .init_resource::<ResurrectUiContext>()
        .init_resource::<ResurrectUiOutbox>()
        .insert_resource(status)
        .add_systems(Update, sync_resurrect_ui_context);
    app.world_mut().spawn((LocalPlayer, Transform::default()));
    app.update();
    let context = *app.world().resource::<ResurrectUiContext>();
    assert!(context.ready_for_play && context.player_available);
    assert_eq!(context.nearest_xcom_index, Some(0));
    let mut outbox = ResurrectUiOutbox::default();
    let request = app
        .world_mut()
        .resource_mut::<ResurrectUiModel>()
        .advance(61.0, context, &mut outbox)
        .expect("a missing instance ZoneNum must not strand a dead player");
    assert_eq!(request.index, 0);
    assert_eq!(request.regen_type, 1);
    assert_eq!(outbox.len(), 1);
}

#[test]
fn new_actor_interaction_locks_before_read_and_clears_existing_auto_run() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .insert_resource(tutorial_gate_test_native_mechanics())
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .init_resource::<LegacyInputGate>()
        .configure_sets(
            Update,
            (
                TutorialActorSet::ApplyCommands,
                LegacyMovementSet::ReadInput,
            )
                .chain(),
        )
        .add_systems(
            Update,
            begin_test_npc_interaction.in_set(TutorialActorSet::ApplyCommands),
        )
        .add_systems(
            Update,
            sync_tutorial_input_gate
                .after(TutorialActorSet::ApplyCommands)
                .before(LegacyMovementSet::ReadInput),
        );
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Tutorial);
    app.world_mut().spawn(tutorial_gate_test_actor(false));
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_auto_run(true);
    let player = app.world_mut().spawn((LocalPlayer, controller)).id();

    app.update();

    assert_eq!(
        *app.world().resource::<LegacyInputGate>(),
        LegacyInputGate {
            allow_forward: false,
            allow_backward: false,
            allow_strafe: false,
            allow_keyboard_turning: false,
            allow_jump: false,
            allow_mouse_camera: false,
        }
    );
    assert!(
        !app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .is_auto_running()
    );
}

#[test]
fn race_pods_stay_hidden_until_start_and_collect_once() {
    use ffone_client::world_behaviour::{WorldTrigger, WorldTriggerKind};
    let mut app = App::new();
    app.init_resource::<RaceProductionRuntime>().init_resource::<WorldGameplayIntentQueue>()
        .add_systems(Update, sync_race_world_rings);
    let model = app.world_mut().spawn(Visibility::Inherited).id();
    app.world_mut().spawn((WorldTrigger {
        kind: WorldTriggerKind::Ring, server_id: 1, object_id: 1, cne_id: 0, trigger_type: 0,
        radius: 1.0, velocity: 0.0, speed: 0.0, add_power: 0.0, min_power: 0.0, max_power: 0.0,
        move_type: 0, waypoint_count: 0, start_position: Vec3::ZERO, from: Vec3::ZERO, to: Vec3::ZERO,
        spin_velocity: 0.0, initial_rotate: Vec3::ZERO, max_rotate: Vec3::ZERO, head: None, tail: None,
        target_element_id: 0, target_element_trigger: 0, model_entities: vec![model], path_points: vec![],
    }, GlobalTransform::IDENTITY, Name::new("pod")));
    app.world_mut().spawn((LocalPlayer, Transform::from_xyz(10.0,0.0,0.0)));
    app.update();
    assert_eq!(*app.world().get::<Visibility>(model).unwrap(), Visibility::Hidden);
    {
        let mut race = app.world_mut().resource_mut::<RaceProductionRuntime>();
        race.course_bounds = Some([-1000,1000,-1000,1000]);
        race.player.ring_race_active = true;
    }
    app.update();
    assert_eq!(*app.world().get::<Visibility>(model).unwrap(), Visibility::Inherited);
    let world = app.world_mut();
    for mut transform in world.query_filtered::<&mut Transform, With<LocalPlayer>>().iter_mut(world) { transform.translation = Vec3::ZERO; }
    app.update();
    assert_eq!(*app.world().get::<Visibility>(model).unwrap(), Visibility::Hidden);
    assert_eq!(app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all().len(), 1);
    app.update();
    assert!(app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all().is_empty());
    {
        let mut race = app.world_mut().resource_mut::<RaceProductionRuntime>();
        race.pending_rings.clear();
        race.collected_rings.insert(1);
    }
    app.update();
    assert!(app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all().is_empty());
}
#[test]
fn race_ring_cue_requires_one_confirmed_pending_pickup() {
    let mut race = RaceProductionRuntime::default();
    race.player.ring_race_active = true;
    race.pending_rings.insert(7);
    assert!(race.confirm_ring(7, 1));
    assert_eq!(race.player.ring_count, 1);
    assert_eq!(race.pending_ring_cues, 1);
    assert!(!race.confirm_ring(7, 1));
    assert!(!race.confirm_ring(8, 2));
    assert_eq!(race.pending_ring_cues, 1);
    race.pending_rings.insert(8);
    race.player.ring_race_active = false;
    assert!(!race.confirm_ring(8, 2));
    assert_eq!(race.player.ring_count, 1);
}

#[test]
fn confirmed_race_finish_suppresses_repeated_matching_farewells() {
    let mut race = RaceProductionRuntime::default();
    race.finished_voice_npc_id = Some(42);
    assert!(!race.owns_final_voice_for(7));
    assert!(race.owns_final_voice_for(42));
    assert!(race.owns_final_voice_for(42));
}

#[test]
fn expired_race_queues_one_server_cancel() {
    let mut app = App::new();
    app.init_resource::<RaceModeModel>()
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<RaceHudState>()
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, cancel_expired_race);
    app.world_mut().resource_mut::<RaceProductionRuntime>().player.ring_race_active = true;
    *app.world_mut().resource_mut::<RaceHudState>() = RaceHudState {
        running: true,
        remaining_seconds: 0,
        ..default()
    };
    app.update();
    let mode = app.world().resource::<RaceModeModel>();
    assert_eq!(mode.phase(), RaceModePhase::AwaitingCancel);
    assert_eq!(mode.output_len(), 2); // cursor release and one cancel request
    app.update();
    assert_eq!(app.world().resource::<RaceModeModel>().output_len(), 2);
}
