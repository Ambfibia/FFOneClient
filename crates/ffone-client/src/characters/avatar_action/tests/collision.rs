use super::*;

#[test]
fn pending_first_world_collision_holds_stand_without_fake_jump_or_landing() {
    let mut app = App::new();
    app.init_resource::<LegacyVisualRequestQueue>()
        .add_systems(Update, update_legacy_avatar_locomotion);
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    let actor = app
        .world_mut()
        .spawn((
            controller,
            LegacyAvatarActionContext::default(),
            LegacyAvatarClipBindings::default(),
            LegacyAvatarActionState::default(),
            LegacyWorldColliderPending,
        ))
        .id();

    app.update();
    let state = app.world().get::<LegacyAvatarActionState>(actor).unwrap();
    assert_eq!(state.locomotion, LegacyLocomotionState::Stand);
    assert!(state.visual_initialized);
    assert!(state.was_grounded);
    let request = app
        .world_mut()
        .resource_mut::<LegacyVisualRequestQueue>()
        .pop_front()
        .expect("pending collision must publish one initial stand");
    assert!(matches!(
        request.command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::Stand1,
            ..
        }
    ));

    app.world_mut()
        .entity_mut(actor)
        .remove::<LegacyWorldColliderPending>();
    app.world_mut()
        .get_mut::<LegacyPlayerController>(actor)
        .unwrap()
        .set_grounded(true);
    app.update();
    assert!(
        app.world()
            .resource::<LegacyVisualRequestQueue>()
            .is_empty(),
        "first real grounded frame must not invent a landing clip"
    );
    assert_eq!(
        app.world()
            .get::<LegacyAvatarActionState>(actor)
            .unwrap()
            .locomotion,
        LegacyLocomotionState::Stand
    );
}
