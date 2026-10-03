use super::*;

#[test]
fn interaction_command_is_exclusive_and_observation_refreshes() {
    let mut app = app();
    app.world_mut()
        .spawn((LegacyAvatarTargetFeed::default(), Transform::IDENTITY));
    {
        let mut commands = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        commands.spawn(spawn(NUMBUH_TWO_ID, 2671, [300, 0, 0], None));
        commands.spawn(spawn(BUTTERCUP_ID, 2672, [900, 0, 0], None));
        commands.set_interacting(NUMBUH_TWO_ID, true);
        commands.set_interacting(BUTTERCUP_ID, true);
    }
    app.update();

    let snapshot = app.world().resource::<TutorialNpcObservationSnapshot>();
    assert!(!snapshot.0.numbuh_two.interacting);
    assert!(snapshot.0.buttercup.interacting);
    assert_eq!(snapshot.0.numbuh_two.distance, Some(10.0_f32.sqrt()));
    assert_eq!(snapshot.0.buttercup.distance, Some(82.0_f32.sqrt()));
}

#[test]
fn demo_invulnerability_preserves_hp_but_still_emits_damage_event() {
    let mut app = app();
    {
        let mut commands = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        commands.configure_demo_monster(true);
        commands.spawn(spawn(DEMO_MONSTER_ID, 2677, [0, 0, 0], None));
        commands.damage(DEMO_MONSTER_ID, 100);
    }
    app.update();

    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(DEMO_MONSTER_ID)
        .unwrap();
    let actor = app.world().get::<TutorialActor>(entity).unwrap();
    assert_eq!(actor.hp, 400);
    assert!(actor.invulnerable);
    assert!(actor.damaged);
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialActorEventQueue>()
            .take_all()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![TutorialActorEvent::Damaged {
            id: DEMO_MONSTER_ID,
            entity,
            amount: 0,
            remaining_hp: 400,
            first_hit: true,
        }]
    );
}
