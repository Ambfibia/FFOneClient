use super::*;

#[test]
fn authoritative_tick_refreshes_stamina_without_reloading_the_same_nano() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn_empty().id();
    let nano = activate_for_test(&mut app, owner);
    let presentation = WorldNanoGameplayPresentation {
        model_path: TUTORIAL_NANO_MODEL_PATH.to_owned(),
        style: 1,
        skill_slot: 1,
    };
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .world_presentation = Some(presentation.clone());
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .skill_cooldown_remaining_seconds = 3.0;
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .equip_world(
            TUTORIAL_BUTTERCUP_NANO_ID,
            TUTORIAL_BUTTERCUP_SKILL_ID,
            77,
            presentation,
        );

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.entity(), Some(nano));
    assert_eq!(state.owner(), Some(owner));
    assert_eq!(state.stamina(), 77);
    assert!(matches!(state.status(), TutorialNanoGameplayStatus::Ready));
    assert_eq!(state.skill_cooldown_remaining_fraction(), Some(0.375));
}
