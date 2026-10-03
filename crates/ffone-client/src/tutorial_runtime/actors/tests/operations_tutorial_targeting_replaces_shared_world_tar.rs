use super::*;

#[test]
fn tutorial_targeting_replaces_shared_world_targets_with_script_owned_actors() {
    let mut app = app();
    let network_npc = app.world_mut().spawn_empty().id();
    let tutorial_actor = app
        .world_mut()
        .spawn((
            TutorialActor {
                id: 1,
                npc_type: 2674,
                team: 2,
                hp: 400,
                max_hp: 400,
                damaged: false,
                interacting: false,
                invulnerable: false,
            },
            Transform::from_xyz(0.0, 0.0, -2.0),
        ))
        .id();
    let avatar = app
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            LegacyAvatarActionContext::default(),
            LegacyAvatarTargetFeed {
                source_connected: true,
                samples: vec![LegacyTargetSample {
                    entity: network_npc,
                    kind: LegacyTargetKind::Npc { team: 2 },
                    distance: 1.0,
                    in_view: true,
                    in_attack_arc: true,
                    in_nano_arc: true,
                    in_attack_cone: true,
                    talk_enabled: false,
                    position: [0.0; 3],
                    radius: 1.0,
                    height: 2.0,
                }],
                ..default()
            },
        ))
        .id();

    app.update();
    app.update();

    let feed = app.world().get::<LegacyAvatarTargetFeed>(avatar).unwrap();
    assert_eq!(feed.samples.len(), 1);
    assert_eq!(feed.samples[0].entity, tutorial_actor);
    assert_ne!(feed.samples[0].entity, network_npc);
}

#[test]
fn target_feed_rejects_an_authored_line_of_sight_blocker() {
    let content = production_content();
    let hostile = TutorialActor {
        id: 1,
        npc_type: 2674,
        team: 2,
        hp: 400,
        max_hp: 400,
        damaged: false,
        interacting: false,
        invulnerable: false,
    };
    let sample = tutorial_target_sample_with_los(
        &content,
        Entity::from_bits(1),
        &hostile,
        &Transform::from_xyz(0.0, 0.0, -2.0),
        Vec3::ZERO,
        Vec3::NEG_Z,
        TutorialTargetingProfile::default(),
        |start, end| start != end,
    )
    .unwrap();
    assert!(!sample.in_view);
    assert!(sample.in_attack_arc);
    assert!(!sample.in_attack_cone);
}

#[test]
fn virtual_server_style_matrix_gives_exact_tutorial_damage() {
    for defender_style in 0..=2 {
        assert_eq!(
            tutorial_virtual_server_damage(-1, defender_style, false),
            100
        );
    }
    assert_eq!(tutorial_virtual_server_damage(1, 2, false), 100);
    assert_eq!(tutorial_virtual_server_damage(0, 2, false), 300);
    assert_eq!(tutorial_virtual_server_damage(-1, 2, true), 0);
}

#[test]
fn npc_attack_requires_living_actor_and_exact_xdt_range() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(1005, 2674, [0, 0, 0], None));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .attempt_player_attack(1005, Vec3::new(17.4, 1.0, 0.0));
    app.update();
    assert!(matches!(
        app.world_mut()
            .resource_mut::<TutorialActorEventQueue>()
            .pop_front(),
        Some(TutorialActorEvent::AttackedPlayer {
            id: 1005,
            damage: 50,
            ..
        })
    ));

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .attempt_player_attack(1005, Vec3::new(17.401, 1.0, 0.0));
    app.update();
    assert!(app.world().resource::<TutorialActorEventQueue>().is_empty());
}

#[test]
fn fusion_spawn_uses_both_melee_clips_from_the_shared_native_package() {
    assert_eq!(tutorial_actor_attack_clip(2674, 0), "melee1");
    assert_eq!(tutorial_actor_attack_clip(2674, 49), "melee1");
    assert_eq!(tutorial_actor_attack_clip(2674, 50), "melee2");
    assert_eq!(tutorial_actor_attack_clip(2674, 99), "melee2");
    assert_eq!(tutorial_actor_attack_clip(2897, 99), "melee2");
    assert_eq!(tutorial_actor_attack_clip(2675, 50), "melee2");
}
