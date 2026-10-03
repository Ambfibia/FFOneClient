use super::*;

#[test]
fn model_binding_and_idle_commands_consume_exactly_one_draw_each() {
    let mut app = app();
    app.insert_resource(TutorialActorStandRandomStream::with_seed(0x1234_5678));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .play_pose(999_999, "idle", true);
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialActorStandRandomStream>()
            .draw_count(),
        0,
        "a command for a missing actor is not accepted and consumes no draw"
    );

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(1005, 2675, [0, 0, 0], None));
    app.update();
    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1005)
        .unwrap();
    let initial_pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(initial_pose.clip, Some("idle"));
    assert!(
        TUTORIAL_ACTOR_STAND_CLIPS.contains(&initial_pose.resolved_clip.unwrap()),
        "NpcAnimation.SetModel must start a stand animation"
    );
    assert_eq!(
        app.world()
            .resource::<TutorialActorStandRandomStream>()
            .draw_count(),
        1
    );

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .play_pose(1005, "idle", true);
    app.update();
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(pose.clip, Some("idle"));
    assert!(TUTORIAL_ACTOR_STAND_CLIPS.contains(&pose.resolved_clip.unwrap()));
    assert!(pose.once, "the semantic request retains AnimationNpcOnce");
    assert!(!tutorial_actor_pose_effective_once(&pose));
    assert_eq!(
        app.world()
            .resource::<TutorialActorStandRandomStream>()
            .draw_count(),
        2
    );

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .force_update(1005);
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialActorStandRandomStream>()
            .draw_count(),
        2
    );
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .play_pose(1005, "idle", false);
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialActorStandRandomStream>()
            .draw_count(),
        3
    );
}
