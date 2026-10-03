use super::*;

#[test]
fn player_kill_arms_fusion_spawn_dead_motion_payload_but_forced_death_does_not() {
    let mut app = app();
    {
        let mut commands = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        commands.spawn(spawn(1005, 2674, [0, 0, 0], None));
        commands.damage(1005, i32::MAX);
    }
    app.update();

    let killed = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1005)
        .expect("killed Fusion Spawn remains for its death clip");
    let marker = app
        .world()
        .get::<TutorialActorPlayerKillDeathPresentation>(killed)
        .expect("DeadMotion player-kill payload marker");
    let pose = app.world().get::<TutorialActorPose>(killed).unwrap();
    assert_eq!(pose.clip, Some("death"));
    assert_eq!(marker.request_serial, pose.request_serial);

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(1006, 2674, [0, 0, 0], None));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .play_pose(1006, "death", true);
    app.update();
    let scripted = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1006)
        .unwrap();
    assert!(
        app.world()
            .get::<TutorialActorPlayerKillDeathPresentation>(scripted)
            .is_none(),
        "AnimationNpc(\"death\") does not call DeadMotion.Dead"
    );
}
