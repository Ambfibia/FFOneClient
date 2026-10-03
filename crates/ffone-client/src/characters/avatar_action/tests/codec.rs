use super::*;

#[test]
fn authored_collision_selects_landing_in_the_same_animation_frame() {
    fn resolve_test_ground(mut players: Query<&mut LegacyPlayerController>) {
        for mut controller in &mut players {
            controller.land_on_external_collider();
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, LegacyAvatarActionPlugin))
        .add_systems(
            Update,
            resolve_test_ground
                .in_set(NativeWorldSet::ResolveCollision)
                .after(LegacyMovementSet::Simulate),
        );
    let mut controller = LegacyPlayerController::from_baseline_table();
    controller.set_grounded(false);
    let actor = app
        .world_mut()
        .spawn((
            Transform::default(),
            controller,
            LegacyAvatarActionContext::default(),
            LegacyAvatarTargetFeed::default(),
            LegacyAvatarClipBindings::default(),
            LegacyAvatarActionState {
                locomotion: LegacyLocomotionState::Jump,
                visual_initialized: true,
                was_grounded: false,
                ..Default::default()
            },
        ))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .get::<LegacyAvatarActionState>(actor)
            .unwrap()
            .locomotion,
        LegacyLocomotionState::Landing,
        "the rig must not render one extra airborne frame after collision grounded it"
    );
}
