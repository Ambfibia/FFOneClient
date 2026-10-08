use super::*;

pub(super) fn entities(count: usize) -> Vec<Entity> {
    let mut world = World::new();
    (0..count).map(|_| world.spawn_empty().id()).collect()
}

#[test]
fn zipline_cancels_attack_layers_and_allows_a_new_attack_after_exit() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<LegacyVisualRequestQueue>()
        .add_systems(Update, update_legacy_avatar_locomotion);
    let actor = app.world_mut().spawn((
        LegacyPlayerController::from_baseline_table(),
        LegacyAvatarActionContext::default(),
        LegacyAvatarPresentationContext::default(),
        LegacyAvatarClipBindings::default(),
        LegacyAvatarActionState {
            upper_action: Some(LegacyVisualClip::AttackUpper(1)),
            base_action: Some(LegacyVisualClip::AttackFull(1)),
            attack_delay_remaining: Some(0.2),
            weapon_change_visual_active: true,
            visual_initialized: true,
            ..default()
        },
    )).id();
    app.world_mut().get_mut::<LegacyAvatarPresentationContext>(actor).unwrap().traversal =
        LegacyAvatarTraversalPresentation::Zipline;
    app.world_mut().resource_mut::<LegacyVisualRequestQueue>().pending.push_back(
        LegacyVisualRequest {
            actor,
            command: cross_fade(&LegacyAvatarClipBindings::default(),
                LegacyVisualClip::AttackUpper(1), LegacyAnimationLayer::UpperBody, true),
        },
    );
    app.update();
    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.locomotion, LegacyLocomotionState::RopeDown);
    assert_eq!(state.base_action(), None);
    assert_eq!(state.upper_action, None);
    assert_eq!(state.attack_delay_remaining, None);
    assert!(!state.weapon_change_visual_active());
    assert_eq!(state.attack_generation(), 1);
    let requests = app.world_mut().resource_mut::<LegacyVisualRequestQueue>().take_all();
    assert_eq!(requests.len(), 1, "queued upper attack must be discarded");
    assert!(requests.iter().any(|request|
        matches!(&request.command, LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::RopeDown,
            layer: LegacyAnimationLayer::FullBody,
            blend_seconds: 0.0,
            ..
        })));
    app.world_mut().get_mut::<LegacyAvatarPresentationContext>(actor).unwrap().traversal =
        LegacyAvatarTraversalPresentation::None;
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion,
        LegacyLocomotionState::Stand);
    let mut state = app.world().get::<LegacyAvatarActionState>(actor).unwrap().clone();
    let schedule = schedule_legacy_action_frame(
        LegacyAvatarActionInput { primary_just_pressed: true, ..default() },
        &LegacyAvatarActionContext::default(),
        &LegacyTargetSelection::default(),
        &mut state,
        &LegacyAvatarClipBindings::default(),
        0.0,
    );
    assert!(matches!(schedule.actions.as_slice(), [LegacyAvatarActionIntent::PrimaryAttack { .. }]));
    assert_eq!(state.attack_generation(), 2);
}

#[test]
fn damage_interrupts_attack_and_repeated_hits_advance_generation() {
    let actor = entities(1)[0];
    let mut visuals = LegacyVisualRequestQueue::default();
    visuals.pending.push_back(LegacyVisualRequest { actor,
        command: cross_fade(&LegacyAvatarClipBindings::default(), LegacyVisualClip::AttackUpper(1),
            LegacyAnimationLayer::UpperBody, true) });
    let mut state = LegacyAvatarActionState {
        upper_action: Some(LegacyVisualClip::AttackUpper(1)),
        base_action: Some(LegacyVisualClip::AttackFull(1)),
        attack_delay_remaining: Some(0.2),
        visual_initialized: true,
        ..default()
    };
    state.interrupt_for_damage();
    visuals.cancel_attack_for(actor);
    assert!(visuals.is_empty());
    assert_eq!(state.upper_action, None);
    assert_eq!(state.base_action(), None);
    assert_eq!(state.attack_delay_remaining, None);
    assert!(!state.visual_initialized);
    assert_eq!(state.attack_generation(), 1);
    state.interrupt_for_damage();
    assert_eq!(state.attack_generation(), 2);
}

#[test]
fn cannon_pose_preempts_both_attack_layers_with_immediate_source_play() {
    let mut app = App::new();
    app.init_resource::<Time>().init_resource::<LegacyVisualRequestQueue>()
        .add_systems(Update, update_legacy_avatar_locomotion);
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.launch_scripted_ballistic(Vec3::new(20.0,30.0,0.0));
    let actor = app.world_mut().spawn((controller, LegacyAvatarActionContext::default(),
        LegacyAvatarPresentationContext { traversal: LegacyAvatarTraversalPresentation::Launcher, ..default() },
        LegacyAvatarClipBindings::default(), LegacyAvatarActionState {
            upper_action: Some(LegacyVisualClip::AttackUpper(1)),
            base_action: Some(LegacyVisualClip::AttackFull(1)),
            attack_delay_remaining: Some(0.2), visual_initialized: true, ..default()
        })).id();
    app.update();
    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.locomotion, LegacyLocomotionState::Launcher);
    assert_eq!(state.upper_action, None); assert_eq!(state.base_action(), None);
    assert_eq!(state.attack_delay_remaining, None);
    assert_eq!(state.attack_generation(),1);
    let requests = app.world_mut().resource_mut::<LegacyVisualRequestQueue>().take_all();
    assert!(matches!(requests[0].command, LegacyVisualCommand::CrossFade {
        requested_clip: LegacyVisualClip::Launcher, blend_seconds: 0.0, ..
    }));
}

#[test]
fn nano_dash_and_stun_preempt_locomotion_and_release_without_retriggering() {
    let mut app=App::new();
    app.init_resource::<LegacyVisualRequestQueue>().add_systems(Update, update_legacy_avatar_locomotion);
    let actor=app.world_mut().spawn((LegacyPlayerController::from_baseline_table(), LegacyAvatarActionContext::default(),
        LegacyAvatarClipBindings::default(), LegacyAvatarActionState::default())).id();
    app.world_mut().get_mut::<LegacyPlayerController>(actor).unwrap().launch_nano_dash_from(false);
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion,LegacyLocomotionState::Dash);
    app.world_mut().resource_mut::<LegacyVisualRequestQueue>().take_all();
    app.update();
    assert!(app.world().resource::<LegacyVisualRequestQueue>().is_empty());
    app.world_mut().get_mut::<LegacyAvatarActionContext>(actor).unwrap().time_buff_condition=5;
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().authoritative_visual_clip(),LegacyVisualClip::Stun);
    *app.world_mut().get_mut::<LegacyPlayerController>(actor).unwrap()=LegacyPlayerController::from_baseline_table();
    app.world_mut().get_mut::<LegacyAvatarActionContext>(actor).unwrap().time_buff_condition=0;
    app.update();
    assert_ne!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion,LegacyLocomotionState::Stun);
    app.world_mut().resource_mut::<LegacyVisualRequestQueue>().pending.clear();
    app.world_mut().get_mut::<LegacyPlayerController>(actor).unwrap().launch_nano_dash_from(true);
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion,LegacyLocomotionState::DashAir);
    let requests = &app.world().resource::<LegacyVisualRequestQueue>().pending;
    assert_eq!(requests.len(), 3);
    assert!(matches!(requests[0].command, LegacyVisualCommand::CrossFade { requested_clip: LegacyVisualClip::Dash, blend_seconds: 0.0, .. }));
    assert!(matches!(requests[1].command, LegacyVisualCommand::CrossFade { requested_clip: LegacyVisualClip::DashAir, blend_seconds: 0.1, .. }));
    assert!(matches!(requests[2].command, LegacyVisualCommand::CrossFade { requested_clip: LegacyVisualClip::DashUpper, .. }));
    app.world_mut().resource_mut::<LegacyVisualRequestQueue>().pending.clear();
    app.world_mut().entity_mut(actor).insert(LegacyAvatarEnvironmentState { in_water: true, ..default() });
    app.world_mut().get_mut::<LegacyAvatarActionState>(actor).unwrap().visual_initialized = false;
    app.update();
    let requests = &app.world().resource::<LegacyVisualRequestQueue>().pending;
    assert_eq!(requests.len(), 2);
    assert!(matches!(requests[0].command, LegacyVisualCommand::CrossFade { requested_clip: LegacyVisualClip::DashAir, blend_seconds: 0.0, .. }));
    assert!(matches!(requests[1].command, LegacyVisualCommand::CrossFade { requested_clip: LegacyVisualClip::DashWaterUpper, .. }));
}

#[test]
fn npc_dialogue_pointer_press_cannot_also_become_world_talk_input() {
    let mut app = App::new();
    app.add_plugins(crate::input_focus::GameplayPointerCapturePlugin)
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<LegacyAvatarActionInput>()
        .add_systems(Update, read_legacy_avatar_action_input);
    let button = app
        .world_mut()
        .spawn((
            Interaction::Pressed,
            ComputedNode {
                size: Vec2::new(32.0, 32.0),
                ..default()
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    let input = app.world().resource::<LegacyAvatarActionInput>();
    assert!(!input.primary_held && !input.primary_just_pressed);
    // Hiding the menu cannot transfer its held press to gameplay.
    app.world_mut()
        .get_mut::<ComputedNode>(button)
        .unwrap()
        .size = Vec2::ZERO;
    app.update();
    assert!(!app.world().resource::<LegacyAvatarActionInput>().primary_held);
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Left);
    app.update();
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().clear();
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    app.update();
    assert!(app.world().resource::<LegacyAvatarActionInput>().primary_just_pressed);
}

#[test]
fn z_mouse_and_x_mouse_use_the_exact_legacy_edges() {
    let mut keyboard = ButtonInput::<KeyCode>::default();
    let mut mouse = ButtonInput::<MouseButton>::default();
    keyboard.press(KeyCode::KeyZ);
    keyboard.press(KeyCode::Tab);
    mouse.press(MouseButton::Right);
    let input = action_input_from_devices(Some(&keyboard), Some(&mouse));
    assert!(input.primary_held);
    assert!(input.primary_just_pressed);
    assert!(input.nano_just_pressed);
    assert!(input.weapon_cycle_just_pressed);
}

#[test]
fn held_primary_claims_the_branch_and_suppresses_tab_during_overheat() {
    let mut state = LegacyAvatarActionState::default();
    let context = LegacyAvatarActionContext {
        overheat_allows_attack: false,
        weapon_swap_available: true,
        ..default()
    };
    let frame = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            weapon_cycle_just_pressed: true,
            ..default()
        },
        &context,
        &LegacyTargetSelection::default(),
        &mut state,
        &LegacyAvatarClipBindings::default(),
        1.0 / 60.0,
    );
    assert!(frame.actions.is_empty());
}

#[test]
fn held_primary_uses_the_exact_game_condition_cooldown_window() {
    let context = LegacyAvatarActionContext {
        attack_cooldown_seconds: 1.0,
        ..default()
    };
    let input = LegacyAvatarActionInput {
        primary_held: true,
        ..default()
    };
    let mut state = LegacyAvatarActionState::default();
    let first = schedule_legacy_action_frame(
        input,
        &context,
        &LegacyTargetSelection::default(),
        &mut state,
        &LegacyAvatarClipBindings::default(),
        0.0,
    );
    assert!(matches!(
        first.actions.as_slice(),
        [LegacyAvatarActionIntent::PrimaryAttack { .. }]
    ));
    let blocked = schedule_legacy_action_frame(
        input,
        &context,
        &LegacyTargetSelection::default(),
        &mut state,
        &LegacyAvatarClipBindings::default(),
        0.999,
    );
    assert!(blocked.actions.is_empty());
    let ready = schedule_legacy_action_frame(
        input,
        &context,
        &LegacyTargetSelection::default(),
        &mut state,
        &LegacyAvatarClipBindings::default(),
        0.002,
    );
    assert!(matches!(
        ready.actions.as_slice(),
        [LegacyAvatarActionIntent::PrimaryAttack { .. }]
    ));
}

#[test]
fn nano_is_independent_from_the_primary_trigger_branch() {
    let ids = entities(1);
    let mut state = LegacyAvatarActionState::default();
    let context = LegacyAvatarActionContext {
        nano_skill_usable: true,
        nano_target_range: 10.0,
        nano_target_capacity: 2,
        ..default()
    };
    let selection = LegacyTargetSelection {
        trigger: Some(ids[0]),
        nano_targets: vec![LegacyAttackTarget {
            entity: ids[0],
            kind: LegacyTargetKind::Npc { team: 2 },
            distance: 4.0,
        }],
        ..default()
    };
    let frame = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            primary_just_pressed: true,
            nano_just_pressed: true,
            ..default()
        },
        &context,
        &selection,
        &mut state,
        &LegacyAvatarClipBindings::default(),
        1.0 / 60.0,
    );
    assert_eq!(
        frame.actions,
        vec![
            LegacyAvatarActionIntent::UseTrigger(ids[0]),
            LegacyAvatarActionIntent::NanoSkill {
                targets: vec![LegacyAttackTarget {
                    entity: ids[0],
                    kind: LegacyTargetKind::Npc { team: 2 },
                    distance: 4.0,
                }],
            },
        ]
    );
}

#[test]
fn vehicle_blocks_nano_skill_until_dismount() {
    let mut context = LegacyAvatarActionContext {
        vehicle_mounted: true,
        nano_skill_usable: true,
        ..default()
    };
    let input = LegacyAvatarActionInput { nano_just_pressed: true, ..default() };
    let mut state = LegacyAvatarActionState::default();
    let mut frame = schedule_legacy_action_frame(input, &context,
        &LegacyTargetSelection::default(), &mut state, &LegacyAvatarClipBindings::default(), 1.0 / 60.0);
    assert!(!frame.actions.iter().any(|a| matches!(a, LegacyAvatarActionIntent::NanoSkill { .. })));
    context.vehicle_mounted = false;
    frame = schedule_legacy_action_frame(input, &context,
        &LegacyTargetSelection::default(), &mut state, &LegacyAvatarClipBindings::default(), 1.0 / 60.0);
    assert!(frame.actions.iter().any(|a| matches!(a, LegacyAvatarActionIntent::NanoSkill { .. })));
}

#[test]
fn vehicle_keeps_riding_locomotion_in_water() {
    let mut app = App::new();
    app.init_resource::<LegacyVisualRequestQueue>()
        .add_systems(Update, update_legacy_avatar_locomotion);
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(true);
    let actor = app.world_mut().spawn((controller,
        LegacyAvatarActionContext { vehicle_mounted: true, ..default() },
        LegacyAvatarEnvironmentState { in_water: true, ..default() },
        LegacyAvatarPresentationContext { mounted_vehicle: LegacyVehiclePresentationFamily::Board, ..default() },
        LegacyAvatarClipBindings::default(), LegacyAvatarActionState::default())).id();
    app.update();
    assert_ne!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion, LegacyLocomotionState::SwimIdle);
    app.world_mut().get_mut::<LegacyAvatarActionContext>(actor).unwrap().vehicle_mounted = false;
    app.world_mut().get_mut::<LegacyAvatarPresentationContext>(actor).unwrap().mounted_vehicle = LegacyVehiclePresentationFamily::None;
    app.update();
    assert_eq!(app.world().get::<LegacyAvatarActionState>(actor).unwrap().locomotion, LegacyLocomotionState::SwimIdle);
}

#[test]
fn nano_targeting_requires_the_focused_mob_and_promotes_it_to_slot_zero() {
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
                in_attack_cone: false,
                talk_enabled: false,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[1],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 4.0,
                in_view: false,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: false,
                talk_enabled: false,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[2],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 10.01,
                in_view: false,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: false,
                talk_enabled: false,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
        ],
        trigger: None,
    };
    let selected = select_legacy_targets(
        &feed,
        &LegacyAvatarActionContext {
            nano_skill_usable: true,
            nano_target_policy: Some(LegacyNanoTargetPolicy {
                effect_target: 3,
                target_type: 1,
                range: 10.0,
                area: 0.0,
                half_angle_degrees: 45.0,
                capacity: 2,
            }),
            ..default()
        },
    );
    assert_eq!(
        selected.nano_targets,
        vec![
            LegacyAttackTarget {
                entity: ids[0],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 9.0,
            },
            LegacyAttackTarget {
                entity: ids[1],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 4.0,
            },
        ]
    );
}

#[test]
fn nano_targeting_clears_the_cone_when_focus_is_outside_it() {
    let ids = entities(2);
    let feed = LegacyAvatarTargetFeed {
        source_connected: true,
        samples: vec![
            LegacyTargetSample {
                entity: ids[0],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 2.0,
                in_view: true,
                in_attack_arc: false,
                in_nano_arc: false,
                in_attack_cone: false,
                talk_enabled: false,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[1],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 3.0,
                in_view: true,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: false,
                talk_enabled: false,
                position: [0.0; 3],
                radius: 0.5,
                height: 1.0,
            },
        ],
        trigger: None,
    };
    let selected = select_legacy_targets(
        &feed,
        &LegacyAvatarActionContext {
            nano_skill_usable: true,
            nano_target_policy: Some(LegacyNanoTargetPolicy {
                effect_target: 3,
                target_type: 1,
                range: 10.0,
                area: 0.0,
                half_angle_degrees: 45.0,
                capacity: 2,
            }),
            ..default()
        },
    );
    assert!(selected.nano_targets.is_empty());
}

#[test]
fn nano_policy_selects_player_focus_and_focus_centered_area() {
    let ids = entities(3);
    let feed = LegacyAvatarTargetFeed {
        source_connected: true,
        samples: vec![
            LegacyTargetSample {
                entity: ids[0],
                kind: LegacyTargetKind::Player,
                distance: 8.0,
                in_view: true,
                in_attack_arc: false,
                in_nano_arc: true,
                in_attack_cone: false,
                talk_enabled: true,
                position: [8.0, 0.0, 0.0],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[1],
                kind: LegacyTargetKind::Player,
                distance: 10.0,
                in_view: false,
                in_attack_arc: false,
                in_nano_arc: true,
                in_attack_cone: false,
                talk_enabled: true,
                position: [10.0, 0.0, 0.0],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[2],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 8.1,
                in_view: true,
                in_attack_arc: true,
                in_nano_arc: true,
                in_attack_cone: true,
                talk_enabled: false,
                position: [8.1, 0.0, 0.0],
                radius: 0.5,
                height: 1.0,
            },
        ],
        trigger: None,
    };
    let context = LegacyAvatarActionContext {
        nano_skill_usable: true,
        nano_target_policy: Some(LegacyNanoTargetPolicy {
            effect_target: 6,
            target_type: 2,
            range: 9.0,
            area: 3.0,
            half_angle_degrees: 180.0,
            capacity: 4,
        }),
        ..default()
    };

    let selected = select_legacy_targets(&feed, &context);
    assert_eq!(
        selected
            .nano_targets
            .iter()
            .map(|target| target.entity)
            .collect::<Vec<_>>(),
        vec![ids[0], ids[1]]
    );
}

#[test]
fn first_noncombat_hostile_click_is_claimed_before_held_attack() {
    let ids = entities(1);
    let selected = LegacyTargetSelection {
        focused_npc: Some(LegacyFocusedTarget {
            entity: ids[0],
            kind: LegacyTargetKind::Npc { team: 2 },
            distance: 1.0,
            talk_enabled: false,
        }),
        check_attack_target: true,
        ..default()
    };
    let mut state = LegacyAvatarActionState::default();
    let bindings = LegacyAvatarClipBindings::default();
    let first = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            primary_just_pressed: true,
            ..default()
        },
        &LegacyAvatarActionContext::default(),
        &selected,
        &mut state,
        &bindings,
        1.0 / 60.0,
    );
    assert!(first.actions.is_empty());
    let held = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            ..default()
        },
        &LegacyAvatarActionContext::default(),
        &selected,
        &mut state,
        &bindings,
        1.0 / 60.0,
    );
    assert!(matches!(
        held.actions.as_slice(),
        [LegacyAvatarActionIntent::PrimaryAttack { .. }]
    ));
}

#[test]
fn missing_visuals_remain_unconnected_and_attack1_is_real_fallback_only() {
    let mut state = LegacyAvatarActionState::default();
    let selected = LegacyTargetSelection {
        attack_targets: vec![LegacyAttackTarget {
            entity: entities(1)[0],
            kind: LegacyTargetKind::Npc { team: 2 },
            distance: 1.0,
        }],
        ..default()
    };
    let missing = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            ..default()
        },
        &LegacyAvatarActionContext::default(),
        &selected,
        &mut state,
        &LegacyAvatarClipBindings::default(),
        1.0 / 60.0,
    );
    assert!(missing.visuals.iter().all(|visual| matches!(
        visual,
        LegacyVisualCommand::CrossFade {
            resolution: LegacyClipResolution::Unconnected,
            ..
        }
    )));

    let mut bindings = LegacyAvatarClipBindings::default();
    bindings.connect(LegacyVisualClip::AttackFull(1), "actors/avatar.glb#attack1");
    assert!(matches!(
        bindings.resolve(LegacyVisualClip::AttackFull(2)),
        LegacyClipResolution::Connected {
            clip: LegacyVisualClip::AttackFull(1),
            ..
        }
    ));
}

#[test]
fn target_attack_delay_uses_strict_less_than_zero_completion() {
    let ids = entities(1);
    let selected = LegacyTargetSelection {
        attack_targets: vec![LegacyAttackTarget {
            entity: ids[0],
            kind: LegacyTargetKind::Npc { team: 2 },
            distance: 1.0,
        }],
        ..default()
    };
    let mut state = LegacyAvatarActionState::default();
    let context = LegacyAvatarActionContext::default();
    let bindings = LegacyAvatarClipBindings::default();
    let _ = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            ..default()
        },
        &context,
        &selected,
        &mut state,
        &bindings,
        0.01,
    );
    let before = schedule_legacy_action_frame(
        LegacyAvatarActionInput::default(),
        &context,
        &selected,
        &mut state,
        &bindings,
        0.59,
    );
    assert!(before.visuals.is_empty());
    let after = schedule_legacy_action_frame(
        LegacyAvatarActionInput::default(),
        &context,
        &selected,
        &mut state,
        &bindings,
        0.02,
    );
    assert_eq!(
        after.visuals,
        vec![LegacyVisualCommand::DelayCurrent {
            seconds: LEGACY_ANIMATION_BLEND_SECONDS,
        }]
    );
}

#[test]
fn locomotion_names_match_legacy_direction_and_jump_table() {
    let context = LegacyAvatarActionContext::default();
    assert_eq!(
        ground_locomotion(5, &context, None),
        LegacyLocomotionState::RunBack
    );
    assert_eq!(
        ground_locomotion(8, &context, None),
        LegacyLocomotionState::Run
    );
    assert_eq!(
        LegacyLocomotionState::JumpStart.clip().legacy_name(),
        "jumpstart"
    );
    assert_eq!(LegacyLocomotionState::Jump.clip().legacy_name(), "jump");
    assert_eq!(
        LegacyLocomotionState::Landing.clip().legacy_name(),
        "jumpend"
    );
    assert_eq!(
        LegacyLocomotionState::LandingRun.clip().legacy_name(),
        "jumplandrun"
    );
}

#[test]
fn one_central_locomotion_table_owns_all_five_swim_directions() {
    let expected = [
        (0, LegacyLocomotionState::SwimIdle),
        (1, LegacyLocomotionState::Swim),
        (2, LegacyLocomotionState::SwimRight),
        (3, LegacyLocomotionState::SwimRight),
        (4, LegacyLocomotionState::SwimBack),
        (5, LegacyLocomotionState::SwimBack),
        (6, LegacyLocomotionState::SwimBack),
        (7, LegacyLocomotionState::SwimLeft),
        (8, LegacyLocomotionState::SwimLeft),
    ];
    for (direction_key, state) in expected {
        assert_eq!(water_locomotion(direction_key), state);
        assert_eq!(
            state.clip().legacy_name(),
            match state {
                LegacyLocomotionState::Swim => "swim",
                LegacyLocomotionState::SwimBack => "swimback",
                LegacyLocomotionState::SwimIdle => "swimidle",
                LegacyLocomotionState::SwimLeft => "swimleft",
                LegacyLocomotionState::SwimRight => "swimright",
                _ => unreachable!(),
            }
        );
    }
}

#[test]
fn stationary_attack_keeps_full_body_until_renderer_completion() {
    let mut state = LegacyAvatarActionState {
        visual_initialized: true,
        was_grounded: true,
        ..Default::default()
    };
    let frame = schedule_legacy_action_frame(
        LegacyAvatarActionInput {
            primary_held: true,
            primary_just_pressed: true,
            ..Default::default()
        },
        &LegacyAvatarActionContext::default(),
        &LegacyTargetSelection {
            source_connected: true,
            ..Default::default()
        },
        &mut state,
        &LegacyAvatarClipBindings::default(),
        0.016,
    );
    assert_eq!(state.base_action(), Some(LegacyVisualClip::AttackFull(1)));
    assert_eq!(frame.visuals.len(), 2);

    let mut app = App::new();
    app.init_resource::<LegacyVisualCompletionQueue>()
        .init_resource::<LegacyVisualRequestQueue>()
        .add_systems(
            Update,
            (
                process_legacy_visual_completions,
                update_legacy_avatar_locomotion,
            )
                .chain(),
        );
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(true);
    let actor = app
        .world_mut()
        .spawn((
            controller,
            LegacyAvatarActionContext {
                combat_condition: true,
                tutorial_event: true,
                ..Default::default()
            },
            LegacyAvatarClipBindings::default(),
            state,
        ))
        .id();

    app.update();
    assert_eq!(
        app.world()
            .get::<LegacyAvatarActionState>(actor)
            .unwrap()
            .locomotion,
        LegacyLocomotionState::Stand,
        "locomotion must not replace the stationary full-body shot"
    );
    assert!(
        app.world()
            .resource::<LegacyVisualRequestQueue>()
            .is_empty(),
        "the old duplicate machine emitted rifleready in this frame"
    );

    app.world_mut()
        .resource_mut::<LegacyVisualCompletionQueue>()
        .push(LegacyVisualCompletion {
            actor,
            clip: LegacyVisualClip::AttackFull(1),
        });
    app.update();
    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.base_action(), None);
    assert_eq!(state.locomotion, LegacyLocomotionState::Ready);
    let request = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("completed full-body attack must return to ready");
    assert!(matches!(
        request.command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Ready,
            blend_seconds: LEGACY_END_ANIMATION_BLEND_SECONDS,
            layer: LegacyAnimationLayer::FullBody,
            ..
        }
    ));
    assert!(
        app.world()
            .resource::<LegacyVisualRequestQueue>()
            .is_empty(),
        "ready must remain stable while combat is active"
    );

    app.world_mut()
        .get_mut::<LegacyAvatarActionContext>(actor)
        .unwrap()
        .combat_condition = false;
    assert!(
        app.world_mut()
            .get_mut::<LegacyPlayerController>(actor)
            .unwrap()
            .set_current_direction_key(1)
    );
    app.update();

    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.locomotion, LegacyLocomotionState::Run);
    assert_eq!(state.upper_action, None);
    let request = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("combat exit must release ready into movement in the same frame");
    assert!(matches!(
        request.command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Run,
            blend_seconds: LEGACY_ANIMATION_BLEND_SECONDS,
            layer: LegacyAnimationLayer::FullBody,
            ..
        }
    ));
}
