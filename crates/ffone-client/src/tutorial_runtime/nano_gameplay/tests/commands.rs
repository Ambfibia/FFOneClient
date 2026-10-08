use super::*;

#[test]
fn corruption_requests_all_three_reactions_without_changing_authoritative_stamina() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn_empty().id();
    activate_for_test(&mut app, owner);
    app.world_mut().resource_mut::<TutorialNanoGameplayState>().world_presentation = Some(
        WorldNanoGameplayPresentation {
            model_path: "characters/nanos/nano_buttercup/nano_buttercup.glb".into(),
            style: 0, skill_slot: 0,
        });
    for (flag, mode, clip) in [(4, LegacyNanoAnimationMode::Win, "win"),
        (8, LegacyNanoAnimationMode::Tie, "tie"), (16, LegacyNanoAnimationMode::Lose, "lose")]
    {
        app.world_mut().resource_mut::<TutorialNanoGameplayCommandQueue>()
            .play_corruption(owner, TUTORIAL_BUTTERCUP_NANO_ID, flag);
        app.update();
        let state = app.world().resource::<TutorialNanoGameplayState>();
        assert_eq!(state.animation.mode(), mode);
        assert_eq!(state.animation.clip(), Some(clip));
        assert_eq!(state.stamina(), TUTORIAL_BUTTERCUP_INITIAL_STAMINA);
    }
    app.world_mut().resource_mut::<TutorialNanoGameplayCommandQueue>().play_corruption(owner, 99, 4);
    app.update();
    assert_eq!(app.world().resource::<TutorialNanoGameplayState>().animation.clip(), Some("lose"));
}

#[test]
fn explicit_dismiss_emits_es10_at_the_nano_before_the_dismissal_event() {
    let mut app = mechanics_app();
    app.init_resource::<GameplayAudioRuntime>();
    let owner = app.world_mut().spawn_empty().id();
    let nano = activate_for_test(&mut app, owner);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .skill_cooldown_remaining_seconds = 6.0;
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .dismiss();

    app.update();

    assert!(app.world().get_entity(nano).is_err());
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_nano_dismissal_count(),
        1
    );
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .dismiss();
    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_nano_dismissal_count(),
        1,
        "repeated authoritative absent state must not replay farewell"
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayEventQueue>()
            .take_all(),
        VecDeque::from([
            TutorialNanoGameplayEvent::EffectRequested {
                effect_id: TUTORIAL_NANO_DISMISS_EFFECT_ID,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                source_line: 161,
            },
            TutorialNanoGameplayEvent::Dismissed {
                owner: Some(owner),
                entity: nano,
            },
        ])
    );
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .skill_cooldown_remaining_fraction(),
        Some(0.75),
        "the equipped-slot cooldown must survive a regular Nano dismiss"
    );
}

#[test]
fn nano_replacement_and_depletion_do_not_request_manual_farewell() {
    let mut app = mechanics_app();
    app.init_resource::<GameplayAudioRuntime>();
    let owner = app.world_mut().spawn_empty().id();
    let nano = activate_for_test(&mut app, owner);
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayAssets>()
        .nodes
        .insert("withdraw", nodes[0]);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .withdraw();
    app.update();
    assert!(
        app.world().get_entity(nano).is_ok(),
        "withdraw waits for animation completion"
    );
    let serial = app
        .world()
        .resource::<TutorialNanoGameplayState>()
        .animation
        .request_serial();
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .animation
            .mode(),
        LegacyNanoAnimationMode::Withdraw
    );
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .withdraw();
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialNanoGameplayState>()
            .animation
            .request_serial(),
        serial
    );
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_nano_dismissal_count(),
        0
    );
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .equip(2, 7, 100);
    app.update();
    assert!(app.world().get_entity(nano).is_err());
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_nano_dismissal_count(),
        0
    );
}
