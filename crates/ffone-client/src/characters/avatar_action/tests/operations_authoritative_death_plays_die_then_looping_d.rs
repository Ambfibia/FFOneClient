use super::*;

#[test]
fn authoritative_death_plays_die_then_looping_death_and_revive_restores_stand() {
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
                dead: true,
                ..Default::default()
            },
            LegacyAvatarClipBindings::default(),
            LegacyAvatarActionState {
                visual_initialized: true,
                upper_action: Some(LegacyVisualClip::AttackUpper(1)),
                base_action: Some(LegacyVisualClip::AttackFull(1)),
                ..Default::default()
            },
        ))
        .id();

    app.update();
    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.death_phase, LegacyAvatarDeathPhase::Dying);
    assert_eq!(state.upper_action, None);
    assert_eq!(state.base_action(), None);
    let die = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("HP <= 0 must start the exact die clip");
    assert!(matches!(
        die.command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Die,
            blend_seconds: LEGACY_ANIMATION_BLEND_SECONDS,
            layer: LegacyAnimationLayer::FullBody,
            ..
        }
    ));

    app.world_mut()
        .resource_mut::<LegacyVisualCompletionQueue>()
        .push(LegacyVisualCompletion {
            actor,
            clip: LegacyVisualClip::Die,
        });
    app.update();
    assert_eq!(
        app.world()
            .get::<LegacyAvatarActionState>(actor)
            .unwrap()
            .death_phase,
        LegacyAvatarDeathPhase::Dead
    );
    let death = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("the die end event must transition to death");
    assert!(matches!(
        death.command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Death,
            blend_seconds: LEGACY_END_ANIMATION_BLEND_SECONDS,
            layer: LegacyAnimationLayer::FullBody,
            ..
        }
    ));
    app.update();
    assert!(
        app.world()
            .resource::<LegacyVisualRequestQueue>()
            .is_empty(),
        "the looping death pose must remain stable"
    );

    app.world_mut()
        .get_mut::<LegacyAvatarActionContext>(actor)
        .unwrap()
        .dead = false;
    app.update();
    let revived = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("revive must restore ordinary locomotion");
    assert!(matches!(
        revived.command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Stand1,
            ..
        }
    ));
    assert_eq!(
        app.world()
            .get::<LegacyAvatarActionState>(actor)
            .unwrap()
            .death_phase,
        LegacyAvatarDeathPhase::Alive
    );
}

#[test]
fn movement_interrupts_stationary_attack_base_without_stopping_upper_attack() {
    let mut state = LegacyAvatarActionState {
        visual_initialized: true,
        was_grounded: true,
        ..Default::default()
    };
    let _ = schedule_legacy_action_frame(
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
            LegacyAvatarActionContext {
                tutorial_event: true,
                ..Default::default()
            },
            LegacyAvatarClipBindings::default(),
            state,
        ))
        .id();

    app.update();

    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.base_action(), None);
    assert_eq!(state.locomotion, LegacyLocomotionState::Run);
    assert_eq!(state.upper_action, Some(LegacyVisualClip::AttackUpper(1)));
    let request = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("movement must replace the full-body shot in the same frame");
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
