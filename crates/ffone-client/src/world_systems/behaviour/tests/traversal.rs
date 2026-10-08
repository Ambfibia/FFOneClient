use super::*;
use crate::movement::{LegacyInputState, LegacyPlayerController};
use std::time::Duration;

fn zipline_app() -> (App, Entity) {
    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs_f32(0.1));
    app.insert_resource(time).init_resource::<LegacyInputState>()
        .init_resource::<WorldGameplayIntentQueue>()
        .add_systems(Update, (update_world_zipline_traversals, finish_world_zipline_steps).chain());
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.begin_scripted_traversal();
    let entity = app.world_mut().spawn((Transform::default(), controller,
        WorldZiplineTraversal { start: Vec3::ZERO, end: Vec3::new(-10.0,-10.0,0.0),
            speed: 5.0, travelled: 0.0, packet_elapsed: 0.0, hang_height: 2.0 })).id();
    (app, entity)
}

#[test]
fn zipline_jump_steps_then_sends_detachment_without_launch_impulse() {
    let (mut app, entity) = zipline_app();
    app.world_mut().resource_mut::<LegacyInputState>().jump_just_pressed = true;
    app.update();
    let requests = app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    assert_eq!(requests.len(),1,"exit packet must bypass 0.3s cadence");
    let packet = &requests[0];
    assert_eq!(packet.packet_type(), packet::P_CL2FE_REQ_PC_ZIPLINE);
    let request = PcZiplineRequest0104::decode(packet.payload()).unwrap();
    assert_eq!(request.down,1);
    assert!((request.moved_distance-0.5).abs()<1e-5);
    assert_eq!(request.roll,0);
    assert!((Vec3::from_array(request.velocity).length()-1.0).abs()<1e-5);
    assert!(app.world().get::<WorldZiplineTraversal>(entity).is_none());
    let controller=app.world().get::<LegacyPlayerController>(entity).unwrap();
    assert!(controller.movement_enabled);
    assert!(controller.jumping, "EpUpdate exits with Jump(0), preventing a midair second jump");
    assert_eq!(controller.velocity,Vec3::ZERO);
    assert!(app.world().get::<Transform>(entity).unwrap().translation.y < -2.0);
}

#[test]
fn zipline_downhill_is_riding_and_endpoint_forces_terminal_packet() {
    let (mut app, entity) = zipline_app();
    app.world_mut().get_mut::<WorldZiplineTraversal>(entity).unwrap().packet_elapsed = 0.3;
    app.update();
    let requests=app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    assert_eq!(PcZiplineRequest0104::decode(requests[0].payload()).unwrap().down,0);
    let mut traversal=app.world_mut().get_mut::<WorldZiplineTraversal>(entity).unwrap();
    traversal.travelled=traversal.start.distance(traversal.end)-0.1;
    traversal.packet_elapsed=0.0;
    app.update();
    let requests=app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    let terminal=PcZiplineRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(terminal.down,0);
    assert_eq!(terminal.moved_distance,terminal.maximum_distance);
    assert!(app.world().get::<WorldZiplineTraversal>(entity).is_none());
}

#[test]
fn launcher_continuation_uses_horizontal_vector_and_landing_sends_zero() {
    let mut app=App::new();
    let mut time=Time::<()>::default(); time.advance_by(Duration::from_secs_f32(0.1));
    app.insert_resource(time).init_resource::<WorldGameplayIntentQueue>()
        .add_systems(Update, update_world_launcher_traversals);
    let mut controller=LegacyPlayerController::from_baseline_table();
    controller.launch_scripted_ballistic(Vec3::new(-20.0,30.0,10.0));
    let entity=app.world_mut().spawn((Transform::from_xyz(-5.0,8.0,3.0),controller,
        WorldLauncherTraversal { horizontal_velocity:Vec3::new(-20.0,0.0,10.0),
            packet_elapsed:0.3,upward_pose:true })).id();
    app.update();
    let requests=app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    assert_eq!(requests[0].packet_type(),packet::P_CL2FE_REQ_PC_LAUNCHER);
    let request=PcLauncherRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(request.velocity,[2000,1000,0]); assert_eq!(request.speed,3000);
    app.world_mut().get_mut::<LegacyPlayerController>(entity).unwrap().land_on_external_collider();
    app.update();
    let requests=app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    let request=PcLauncherRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(request.velocity,[0;3]); assert_eq!(request.speed,0);
    assert!(app.world().get::<WorldLauncherTraversal>(entity).is_none());
    assert!(app.world().get::<LegacyPlayerController>(entity).unwrap().grounded);
}

#[test]
fn zipline_packet_and_hanging_pose_follow_the_resolved_cable_move() {
    let (mut app, entity) = zipline_app();
    // Simulate a ceiling blocking the CCT at Y=-0.25. The hang offset is
    // applied to that result, rather than fed into the sweep or overwritten.
    app.add_systems(Update, (|mut players: Query<&mut Transform, With<WorldZiplineTraversal>>| {
        for mut transform in &mut players { transform.translation.y = -0.25; }
    }).after(update_world_zipline_traversals).before(finish_world_zipline_steps));
    app.world_mut().get_mut::<WorldZiplineTraversal>(entity).unwrap().packet_elapsed = 0.3;
    app.update();
    let position = app.world().get::<Transform>(entity).unwrap().translation;
    assert!((position.y + 2.25).abs() < 1e-5);
    assert!(position.x < -0.35, "the submitted cable move retains the lateral displacement");
    let requests = app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    let request = PcZiplineRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(request.position, ProtocolPosition::from_native(position).raw());
    assert!(!app.world().get::<LegacyPlayerController>(entity).unwrap().grounded);
}

#[test]
fn descending_launcher_side_hit_preserves_terminal_speed_then_enters_zero_jump() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default()).init_resource::<WorldGameplayIntentQueue>()
        .add_systems(Update, update_world_launcher_traversals);
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.launch_scripted_ballistic(Vec3::new(10.0, -3.0, 0.0));
    controller.set_external_collision_result(crate::movement::LEGACY_COLLISION_SIDES, None);
    let entity = app.world_mut().spawn((Transform::default(), controller,
        WorldLauncherTraversal { horizontal_velocity: Vec3::X * 10.0,
            packet_elapsed: 0.0, upward_pose: false })).id();
    app.update();
    let requests = app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    let request = PcLauncherRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(request.speed, -300);
    assert_eq!(request.velocity, [0; 3]);
    let controller = app.world().get::<LegacyPlayerController>(entity).unwrap();
    assert!(controller.jumping);
    assert_eq!(controller.velocity, Vec3::ZERO);
    assert!(!controller.launcher_active());
}

#[test]
fn launcher_initial_packet_contains_full_shot_then_continuations_are_horizontal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
        .init_resource::<LauncherUiOutbox>()
        .init_resource::<ActiveWorldLauncher>()
        .init_resource::<WorldGameplayIntentQueue>()
        .add_systems(Update, (consume_world_launcher_outbox, update_world_launcher_traversals).chain());
    let actor = app.world_mut().spawn((Transform::default(),
        LegacyPlayerController::from_baseline_table(), crate::avatar_action::LegacyAvatarActionState::default())).id();
    let velocity = Vec3::new(-12.0, 16.0, 0.0);
    app.world_mut().resource_mut::<LauncherUiOutbox>().push(LauncherUiEffect::StartLauncher(
        crate::launcher_ui::LauncherShot {
            position: Vec3::new(-3.0, 5.0, 7.0), velocity, forward: velocity.normalize(),
            facing_yaw_degrees: 90.0, power: 20.0,
            request_packet_id: packet::P_CL2FE_REQ_PC_LAUNCHER,
            request_packet_size: PcLauncherRequest0104::SIZE,
        }));
    app.update();
    let requests = app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    assert_eq!(requests.len(), 1);
    let initial = PcLauncherRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(initial.velocity, [1200, 0, 1600]);
    assert_eq!(initial.speed, 1600);
    app.world_mut().get_mut::<WorldLauncherTraversal>(actor).unwrap().packet_elapsed = 0.3;
    app.update();
    let requests = app.world_mut().resource_mut::<WorldGameplayIntentQueue>().take_all();
    let continuation = PcLauncherRequest0104::decode(requests[0].payload()).unwrap();
    assert_eq!(continuation.velocity, [1200, 0, 0]);
    assert_eq!(continuation.speed, 1600);
}
