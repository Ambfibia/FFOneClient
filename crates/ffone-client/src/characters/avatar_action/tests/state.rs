use super::*;

#[test]
fn target_selection_sorts_distance_and_excludes_friendly_npcs() {
    let ids = entities(3);
    let feed = LegacyAvatarTargetFeed {
        source_connected: true,
        samples: vec![
            LegacyTargetSample {
                entity: ids[0],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 9.0,
                in_view: true,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: true,
                talk_enabled: false,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[1],
                kind: LegacyTargetKind::Player,
                distance: 2.0,
                in_view: true,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: true,
                talk_enabled: true,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[2],
                kind: LegacyTargetKind::Npc { team: 1 },
                distance: 1.0,
                in_view: false,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: true,
                talk_enabled: true,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
        ],
        trigger: None,
    };
    let context = LegacyAvatarActionContext {
        attack_players_enabled: true,
        target_capacity: 2,
        ..default()
    };
    let selected = select_legacy_targets(&feed, &context);
    assert!(selected.source_connected);
    assert_eq!(selected.attack_targets.len(), 2);
    assert_eq!(selected.attack_targets[0].entity, ids[1]);
    assert_eq!(selected.attack_targets[1].entity, ids[0]);
}

#[test]
fn tab_is_weapon_cycle_and_never_changes_target_selection() {
    let mut state = LegacyAvatarActionState::default();
    let selected = LegacyTargetSelection::default();
    let context = LegacyAvatarActionContext {
        weapon_swap_available: true,
        ..default()
    };
    let frame = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            weapon_cycle_just_pressed: true,
            ..default()
        },
        &context,
        &selected,
        &mut state,
        &LegacyAvatarClipBindings::default(),
        1.0 / 60.0,
    );
    assert_eq!(frame.actions, vec![LegacyAvatarActionIntent::WeaponCycle]);
    assert_eq!(state.target_selection, LegacyTargetSelection::default());

    for blocked in [
        LegacyAvatarActionContext { weapon_change_in_progress: true, ..context.clone() },
        LegacyAvatarActionContext { input_enabled: false, ..context.clone() },
        LegacyAvatarActionContext { system_popup: true, ..context.clone() },
    ] {
        let frame = schedule_legacy_action_frame(
            LegacyAvatarActionInput { weapon_cycle_just_pressed: true, ..default() },
            &blocked,
            &selected,
            &mut state,
            &LegacyAvatarClipBindings::default(),
            1.0 / 60.0,
        );
        assert!(frame.actions.is_empty());
    }
}

#[test]
fn confirmed_hand_change_plays_ready_then_returns_to_locomotion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f32(0.2)))
        .init_resource::<LegacyVisualRequestQueue>()
        .init_resource::<LegacyVisualCompletionQueue>()
        .add_systems(Update, (process_legacy_visual_completions, update_legacy_avatar_locomotion).chain());
    let mut action = LegacyAvatarActionState::default();
    action.upper_action = Some(LegacyVisualClip::AttackUpper(1));
    action.begin_weapon_change_visual();
    assert_eq!(action.upper_action, None);
    let actor = app.world_mut().spawn((
        Transform::default(),
        LegacyPlayerController::from_baseline_table(),
        LegacyAvatarActionContext::default(),
        LegacyAvatarTargetFeed::default(),
        LegacyAvatarClipBindings::default(),
        action,
    )).id();
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion,
        LegacyLocomotionState::Ready);
    assert!(matches!(
        app.world_mut().resource_mut::<LegacyVisualRequestQueue>().pop_front(),
        Some(LegacyVisualRequest {
            command: LegacyVisualCommand::CrossFade {
                requested_clip: LegacyVisualClip::Ready,
                layer: LegacyAnimationLayer::FullBody,
                ..
            },
            ..
        })
    ));
    // A second confirmed change while Ready was already playing must replay
    // the new transition rather than inherit the previous loop's clock.
    app.world_mut().get_mut::<LegacyAvatarActionState>(actor).unwrap().begin_weapon_change_visual();
    app.update();
    assert!(matches!(app.world_mut().resource_mut::<LegacyVisualRequestQueue>().pop_front(),
        Some(LegacyVisualRequest { command: LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Ready, .. }, .. })));
    for _ in 0..5 { app.update(); }
    assert!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().weapon_change_visual_active());
    app.world_mut().resource_mut::<LegacyVisualCompletionQueue>().push(LegacyVisualCompletion {
        actor,
        clip: LegacyVisualClip::Ready,
    });
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion,
        LegacyLocomotionState::Stand);
    assert!(matches!(
        app.world_mut().resource_mut::<LegacyVisualRequestQueue>().pop_front(),
        Some(LegacyVisualRequest {
            command: LegacyVisualCommand::CrossFade {
                requested_clip: LegacyVisualClip::Stand1,
                layer: LegacyAnimationLayer::FullBody,
                ..
            },
            ..
        })
    ));
}

#[test]
fn presentation_facts_follow_traversal_inventory_normal_priority_table() {
    assert_eq!(
        [0, 1, 2, 3, 4].map(LegacyVehiclePresentationFamily::from_legacy_equip_type),
        [
            Some(LegacyVehiclePresentationFamily::None),
            Some(LegacyVehiclePresentationFamily::Board),
            Some(LegacyVehiclePresentationFamily::Scooter),
            Some(LegacyVehiclePresentationFamily::Scooter),
            None,
        ]
    );
    let cases = [
        (LegacyAvatarPresentationContext::default(), None, None),
        (
            LegacyAvatarPresentationContext {
                mounted_vehicle: LegacyVehiclePresentationFamily::Board,
                ..Default::default()
            },
            None,
            None,
        ),
        (
            LegacyAvatarPresentationContext {
                inventory_open: true,
                ..Default::default()
            },
            Some(LegacyLocomotionState::Inventory),
            Some("inven"),
        ),
        (
            LegacyAvatarPresentationContext {
                inventory_open: true,
                mounted_vehicle: LegacyVehiclePresentationFamily::Board,
                ..Default::default()
            },
            Some(LegacyLocomotionState::BoardInventory),
            Some("board_inven"),
        ),
        (
            LegacyAvatarPresentationContext {
                inventory_open: true,
                mounted_vehicle: LegacyVehiclePresentationFamily::Scooter,
                ..Default::default()
            },
            Some(LegacyLocomotionState::ScooterInventory),
            Some("scooter_inven"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::Slope,
                inventory_open: true,
                mounted_vehicle: LegacyVehiclePresentationFamily::Scooter,
            },
            Some(LegacyLocomotionState::Slide),
            Some("slide"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::Zipline,
                inventory_open: true,
                mounted_vehicle: LegacyVehiclePresentationFamily::Board,
            },
            Some(LegacyLocomotionState::RopeDown),
            Some("ropedown"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeDrop,
                inventory_open: true,
                mounted_vehicle: LegacyVehiclePresentationFamily::Scooter,
            },
            Some(LegacyLocomotionState::RopeDrop),
            Some("ropedrop"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeLeft,
                ..Default::default()
            },
            Some(LegacyLocomotionState::RopeLeft),
            Some("ropeleft"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeRight,
                ..Default::default()
            },
            Some(LegacyLocomotionState::RopeRight),
            Some("roperight"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeStand1,
                ..Default::default()
            },
            Some(LegacyLocomotionState::RopeStand1),
            Some("ropestand1"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeStand2,
                ..Default::default()
            },
            Some(LegacyLocomotionState::RopeStand2),
            Some("ropestand2"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeTurn,
                ..Default::default()
            },
            Some(LegacyLocomotionState::RopeTurn),
            Some("ropeturn"),
        ),
        (
            LegacyAvatarPresentationContext {
                traversal: LegacyAvatarTraversalPresentation::RopeUp,
                ..Default::default()
            },
            Some(LegacyLocomotionState::RopeUp),
            Some("ropeup"),
        ),
    ];

    for (facts, expected_state, expected_name) in cases {
        let state = facts.authoritative_locomotion_override();
        assert_eq!(state, expected_state, "facts={facts:?}");
        assert_eq!(
            state.map(|state| state.clip().legacy_name()),
            expected_name.map(str::to_owned),
            "facts={facts:?}"
        );
    }
}

#[test]
fn central_fsm_transitions_between_vehicle_inventory_traversal_and_run() {
    let mut app = App::new();
    app.init_resource::<LegacyVisualRequestQueue>()
        .add_systems(Update, update_legacy_avatar_locomotion);
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(true);
    assert!(controller.set_current_direction_key(1));
    let actor = app
        .world_mut()
        .spawn((
            controller,
            LegacyAvatarActionContext::default(),
            LegacyAvatarPresentationContext {
                inventory_open: true,
                mounted_vehicle: LegacyVehiclePresentationFamily::Board,
                ..Default::default()
            },
            LegacyAvatarClipBindings::default(),
            LegacyAvatarActionState::default(),
        ))
        .id();

    let assert_transition = |app: &mut App,
                             expected_state: LegacyLocomotionState,
                             expected_clip: LegacyVisualClip| {
        app.update();
        assert_eq!(
            app.world()
                .get::<LegacyAvatarActionState>(actor)
                .unwrap()
                .locomotion,
            expected_state
        );
        let request = app
            .world_mut()
            .resource_mut::<LegacyVisualRequestQueue>()
            .pop_front()
            .expect("presentation transition must publish one authoritative base clip");
        assert!(matches!(
            request.command,
            LegacyVisualCommand::CrossFade { requested_clip, .. }
                if requested_clip == expected_clip
        ));
        assert!(
            app.world()
                .resource::<LegacyVisualRequestQueue>()
                .is_empty()
        );
    };

    assert_transition(
        &mut app,
        LegacyLocomotionState::BoardInventory,
        LegacyVisualClip::BoardInventory,
    );

    app.world_mut()
        .get_mut::<LegacyAvatarPresentationContext>(actor)
        .unwrap()
        .traversal = LegacyAvatarTraversalPresentation::Slope;
    assert_transition(
        &mut app,
        LegacyLocomotionState::Slide,
        LegacyVisualClip::Slide,
    );

    app.world_mut()
        .get_mut::<LegacyAvatarPresentationContext>(actor)
        .unwrap()
        .traversal = LegacyAvatarTraversalPresentation::RopeTurn;
    assert_transition(
        &mut app,
        LegacyLocomotionState::RopeTurn,
        LegacyVisualClip::RopeTurn,
    );

    app.world_mut()
        .get_mut::<LegacyAvatarPresentationContext>(actor)
        .unwrap()
        .traversal = LegacyAvatarTraversalPresentation::None;
    assert_transition(
        &mut app,
        LegacyLocomotionState::BoardInventory,
        LegacyVisualClip::BoardInventory,
    );

    app.world_mut()
        .get_mut::<LegacyAvatarPresentationContext>(actor)
        .unwrap()
        .inventory_open = false;
    assert_transition(&mut app, LegacyLocomotionState::Run, LegacyVisualClip::Run);
}
