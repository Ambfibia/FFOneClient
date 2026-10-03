use super::*;

pub(super) fn mechanics_app() -> App {
    let mut app = App::new();
    app.init_resource::<TutorialNanoGameplayCommandQueue>()
        .init_resource::<TutorialNanoGameplayEventQueue>()
        .init_resource::<TutorialNanoGameplayIssueQueue>()
        .init_resource::<TutorialNanoGameplayState>()
        .init_resource::<TutorialNanoGameplayAssets>()
        .init_resource::<LegacyNanoStandRandomStream>()
        .init_resource::<Time>()
        .configure_sets(
            Update,
            (
                TutorialNanoGameplaySet::AdvanceCooldown,
                TutorialNanoGameplaySet::ApplyCommands,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                advance_tutorial_nano_skill_cooldown
                    .in_set(TutorialNanoGameplaySet::AdvanceCooldown),
                apply_tutorial_nano_gameplay_commands
                    .in_set(TutorialNanoGameplaySet::ApplyCommands),
            ),
        );
    app
}

pub(super) fn demo_actor() -> TutorialActor {
    TutorialActor {
        id: DEMO_MONSTER_ID,
        npc_type: 2677,
        team: 2,
        hp: 400,
        max_hp: 400,
        damaged: false,
        interacting: false,
        invulnerable: true,
    }
}

pub(super) fn activate_for_test(app: &mut App, owner: Entity) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            TutorialGameplayNanoRoot {
                owner,
                generation: 1,
            },
            Transform::IDENTITY,
        ))
        .id();
    *app.world_mut().resource_mut::<TutorialNanoGameplayState>() = TutorialNanoGameplayState {
        loadout: Some(TutorialNanoGameplayLoadout {
            nano_id: TUTORIAL_BUTTERCUP_NANO_ID,
            skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
        }),
        entity: Some(entity),
        owner: Some(owner),
        stamina: TUTORIAL_BUTTERCUP_INITIAL_STAMINA,
        status: TutorialNanoGameplayStatus::Ready,
        generation: 1,
        activated_generation: Some(1),
        ..default()
    };
    entity
}

pub(super) fn target(entity: Entity, distance: f32) -> LegacyAttackTarget {
    LegacyAttackTarget {
        entity,
        kind: LegacyTargetKind::Npc { team: 2 },
        distance,
    }
}

#[test]
fn cheese_case_alias_is_exact_and_chowder_does_not_borrow_a_skill() {
    let cheese_clips = HashSet::from([
        CALL_CLIP,
        CHEESE_SKILL1_ASSET_CLIP,
        "stand1",
        "stand2",
        "stand3",
    ]);
    let resolved_cheese = GAMEPLAY_REQUIRED_CLIPS.map(|logical_clip| {
        gameplay_nano_asset_clip_name(
            Some(CHEESE_NANO_ID),
            Some(CHEESE_NANO_MODEL_PATH),
            logical_clip,
        )
    });
    assert!(
        resolved_cheese
            .iter()
            .all(|clip| cheese_clips.contains(clip)),
        "the exact clean Cheese route must accept its source-owned `Skill1` spelling"
    );
    assert_eq!(
        gameplay_nano_asset_clip_name(
            Some(CHEESE_NANO_ID),
            Some(CHEESE_NANO_MODEL_PATH),
            "skill2",
        ),
        "skill2",
        "the primary Cheese route supplies no evidence for any other alias"
    );
    assert_eq!(
        gameplay_nano_asset_clip_name(
            Some(CHEESE_NANO_ID),
            Some("characters/nanos/not_cheese/not_cheese.glb"),
            SKILL_CLIP,
        ),
        SKILL_CLIP,
        "the alias must not escape the exact native model contract"
    );

    let chowder_path = "characters/nanos/nano_chowder/nano_chowder.glb";
    let chowder_clips = HashSet::from([CALL_CLIP, "stand1", "stand2", "stand3"]);
    let missing = GAMEPLAY_REQUIRED_CLIPS
        .into_iter()
        .filter(|logical_clip| {
            let asset_clip =
                gameplay_nano_asset_clip_name(Some(55), Some(chowder_path), logical_clip);
            !chowder_clips.contains(asset_clip)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        missing,
        vec![SKILL_CLIP],
        "clean Chowder owns no skill clip; do not invent a replacement"
    );
}

#[test]
fn movement_pattern_keeps_the_summon_heading_when_the_owner_turns() {
    let summoned_owner = Transform::from_xyz(2.0, 3.0, 4.0)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    let mut random = LegacyNanoStandRandomStream::with_seed(7);
    let mut movement = TutorialNanoMovementPattern::new(&summoned_owner, 5.0, &mut random);
    movement.pulse = TutorialNanoPulsePattern::None;
    let turned_owner = Transform::from_xyz(8.0, 3.0, 9.0)
        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2));
    let target = turned_owner.translation + movement.lateral_offset + Vec3::Y * 1.12;
    assert!(target.abs_diff_eq(
        turned_owner.translation
            + summoned_owner.rotation * Vec3::new(0.7, 0.0, 0.0)
            + Vec3::Y * 1.12,
        0.000_01,
    ));
    assert!(!target.abs_diff_eq(tutorial_nano_follow_target(&turned_owner), 0.000_01));
    assert_eq!(random.draw_count(), 7);
}

#[test]
fn clockwise_and_counterclockwise_patterns_orbit_the_owner_at_fixed_radius() {
    let initial = Vec3::X * 0.7;
    let clockwise =
        advance_tutorial_nano_orbit(initial, TutorialNanoOrbitPattern::Clockwise, 1.5, 0.25);
    let counterclockwise = advance_tutorial_nano_orbit(
        initial,
        TutorialNanoOrbitPattern::CounterClockwise,
        1.5,
        0.25,
    );
    assert!((clockwise.length() - 0.7).abs() < 0.000_01);
    assert!((counterclockwise.length() - 0.7).abs() < 0.000_01);
    assert!(clockwise.z > 0.0);
    assert!(counterclockwise.z < 0.0);
    assert!(!clockwise.abs_diff_eq(initial, 0.000_01));
    assert!(!counterclockwise.abs_diff_eq(initial, 0.000_01));
}

#[test]
fn new_loadout_and_full_cleanup_clear_the_tutorial_slot_cooldown() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn_empty().id();
    activate_for_test(&mut app, owner);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .skill_cooldown_remaining_seconds = 5.0;
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .equip(2, 7, 90);

    app.update();

    {
        let state = app.world().resource::<TutorialNanoGameplayState>();
        assert_eq!(
            state.loadout(),
            Some(TutorialNanoGameplayLoadout {
                nano_id: 2,
                skill_id: 7,
            })
        );
        assert_eq!(state.stamina(), 90);
        assert_eq!(state.skill_cooldown_remaining_fraction(), None);
    }

    activate_for_test(&mut app, owner);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .skill_cooldown_remaining_seconds = 5.0;
    cleanup_tutorial_nano_gameplay(app.world_mut());
    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.loadout(), None);
    assert_eq!(state.skill_cooldown_remaining_fraction(), None);
}

#[test]
fn duplicate_tutorial_grants_preserve_active_nano_stun_stamina_and_cooldown() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let nano = activate_for_test(&mut app, owner);
    let mob = app
        .world_mut()
        .spawn((demo_actor(), Transform::from_xyz(0.0, 0.0, 2.0)))
        .id();
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .use_skill(owner, vec![target(mob, 2.0)]);
    app.update();
    assert_eq!(
        app.world()
            .get::<TutorialNanoSkillCondition>(mob)
            .unwrap()
            .0,
        512
    );
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayCommandQueue>()
            .equip(1, 1, 100);
        app.update();
    }
    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.entity(), Some(nano));
    assert!(state.is_active());
    assert_eq!(state.stamina(), 80);
    assert_eq!(state.skill_cooldown_remaining_seconds(), 8.0);
    assert!(!state.can_use_skill());
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .advance_skill_cooldown(8.0);
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .can_use_skill()
    );
}

#[test]
fn empty_and_out_of_range_skill_requests_fail_closed() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    activate_for_test(&mut app, owner);
    let far = app
        .world_mut()
        .spawn((demo_actor(), Transform::from_xyz(10.01, 0.0, 0.0)))
        .id();
    {
        let mut queue = app
            .world_mut()
            .resource_mut::<TutorialNanoGameplayCommandQueue>();
        queue.use_skill(owner, Vec::new());
        queue.use_skill(owner, vec![target(far, 1.0)]);
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .stamina(),
        100
    );
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty()
    );
    assert!(app.world().get::<TutorialNanoSkillCondition>(far).is_none());
    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.skill_cooldown_remaining_seconds(), 0.0);
    assert_eq!(state.skill_cooldown_remaining_fraction(), None);
    assert!(state.can_use_skill());
}

#[test]
fn valid_demo_target_drains_stamina_sets_512_and_orders_damage_before_use_skill() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    activate_for_test(&mut app, owner);
    let demo = app
        .world_mut()
        .spawn((demo_actor(), Transform::from_xyz(9.5, 0.0, 0.0)))
        .id();
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .use_skill(owner, vec![target(demo, 9.5)]);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .use_skill(owner, vec![target(demo, 9.5)]);
    app.update();

    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .stamina(),
        80
    );
    {
        let state = app.world().resource::<TutorialNanoGameplayState>();
        assert_eq!(
            state.skill_cooldown_remaining_seconds(),
            TUTORIAL_BUTTERCUP_SKILL_COOLDOWN_SECONDS
        );
        assert_eq!(state.skill_cooldown_remaining_fraction(), Some(1.0));
        assert!(!state.can_use_skill());
    }
    assert_eq!(
        app.world().get::<TutorialNanoSkillCondition>(demo),
        Some(&TutorialNanoSkillCondition(512))
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayEventQueue>()
            .take_all(),
        VecDeque::from([
            TutorialNanoGameplayEvent::DamageNpc {
                target: demo,
                actor_id: DEMO_MONSTER_ID,
                condition: 512,
            },
            TutorialNanoGameplayEvent::UseSkill {
                owner,
                skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
                stamina: 80,
            },
        ])
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(2.0));
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .use_skill(owner, vec![target(demo, 9.5)]);
    app.update();
    {
        let state = app.world().resource::<TutorialNanoGameplayState>();
        assert_eq!(state.skill_cooldown_remaining_seconds(), 6.0);
        assert_eq!(state.skill_cooldown_remaining_fraction(), Some(0.75));
        assert!(!state.can_use_skill());
    }
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .stamina(),
        80,
        "a request during cooldown must not drain stamina"
    );
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty(),
        "a request during cooldown must not emit gameplay results"
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(6.0));
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .use_skill(owner, vec![target(demo, 9.5)]);
    app.update();
    {
        let state = app.world().resource::<TutorialNanoGameplayState>();
        assert_eq!(
            state.skill_cooldown_remaining_seconds(),
            TUTORIAL_BUTTERCUP_SKILL_COOLDOWN_SECONDS
        );
        assert_eq!(state.skill_cooldown_remaining_fraction(), Some(1.0));
        assert!(!state.can_use_skill());
    }
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .stamina(),
        60,
        "the skill becomes usable again when the slot cooldown completes"
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayEventQueue>()
            .take_all(),
        VecDeque::from([
            TutorialNanoGameplayEvent::DamageNpc {
                target: demo,
                actor_id: DEMO_MONSTER_ID,
                condition: TUTORIAL_BUTTERCUP_SKILL_CONDITION,
            },
            TutorialNanoGameplayEvent::UseSkill {
                owner,
                skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
                stamina: 60,
            },
        ])
    );
}

#[test]
fn invalid_leading_targets_do_not_consume_the_two_valid_target_slots() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    activate_for_test(&mut app, owner);
    let mut wrong_actor = demo_actor();
    wrong_actor.id = DEMO_MONSTER_ID + 1;
    let wrong = app
        .world_mut()
        .spawn((wrong_actor, Transform::from_xyz(2.0, 0.0, 0.0)))
        .id();
    let far = app
        .world_mut()
        .spawn((demo_actor(), Transform::from_xyz(10.01, 0.0, 0.0)))
        .id();
    let valid_third = app
        .world_mut()
        .spawn((demo_actor(), Transform::from_xyz(9.5, 0.0, 0.0)))
        .id();
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .use_skill(
            owner,
            vec![
                target(wrong, 2.0),
                target(far, 10.01),
                target(valid_third, 9.5),
            ],
        );

    app.update();

    assert_eq!(
        app.world().get::<TutorialNanoSkillCondition>(valid_third),
        Some(&TutorialNanoSkillCondition(512))
    );
    assert!(
        app.world()
            .get::<TutorialNanoSkillCondition>(wrong)
            .is_none()
    );
    assert!(app.world().get::<TutorialNanoSkillCondition>(far).is_none());
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .stamina(),
        80
    );
}
