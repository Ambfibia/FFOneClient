use super::*;

#[test]
fn cyber_combat_command_faces_attacks_and_enters_death_clip() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ffone_client::tutorial_actors::TutorialActorPlugin);
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.spawn(TutorialNpcSpawn {
            id: 1005,
            npc_type: 2675,
            position: LegacySpawnPosition::centiunits(0, 0, 0),
            angle: None,
        });
        queue.attempt_player_attack(1005, Vec3::new(10.0, 0.0, 0.0));
    }
    app.update();
    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1005)
        .unwrap();
    let pose = *app
        .world()
        .get::<ffone_client::tutorial_actors::TutorialActorPose>(entity)
        .unwrap();
    assert!(matches!(pose.resolved_clip, Some("melee1" | "melee2")));
    assert!(pose.once);
    let forward = app.world().get::<Transform>(entity).unwrap().rotation * Vec3::NEG_Z;
    assert!(forward.abs_diff_eq(Vec3::X, 0.000_01));

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .damage(1005, 1_000);
    app.update();
    let pose = *app
        .world()
        .get::<ffone_client::tutorial_actors::TutorialActorPose>(entity)
        .unwrap();
    assert_eq!(pose.resolved_clip, Some("death"));
    assert!(pose.once);

    app.world_mut().spawn((
        AnimationPlayer::default(),
        ffone_client::tutorial_actors::TutorialActorAnimationPlayback {
            actor_root: entity,
            request_serial: pose.request_serial,
            restart_serial: pose.restart_serial,
            clip: pose.clip,
            resolved_clip: pose.resolved_clip,
            node: None,
            additive: false,
            once: true,
            state: pose.state,
            force_update: false,
            terminally_unavailable: false,
        },
    ));
    app.update();
    app.update();
    assert!(
        app.world()
            .resource::<TutorialActorRegistry>()
            .entity(1005)
            .is_none()
    );
}

#[test]
fn ordinary_world_loader_waits_for_the_shared_player_presentation_rig() {
    assert!(matches!(
        selected_player_presentation_probe(
            ClientState::World,
            Some(&TutorialSelectedPlayerRigStatus::Loading),
            None,
            false,
        ),
        TutorialStartupProbe::Loading(_)
    ));
    assert_eq!(
        selected_player_presentation_probe(
            ClientState::World,
            Some(&TutorialSelectedPlayerRigStatus::Blocked(
                "adapter failed".to_owned()
            )),
            None,
            false,
        ),
        TutorialStartupProbe::Blocked(
            "selected player presentation blocked: adapter failed".to_owned()
        )
    );
    assert_eq!(
        selected_player_presentation_probe(
            ClientState::World,
            Some(&TutorialSelectedPlayerRigStatus::Ready),
            Some(TutorialPlayerClip::Stand1),
            false,
        ),
        TutorialStartupProbe::Ready
    );
}
