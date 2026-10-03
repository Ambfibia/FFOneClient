use super::*;

#[test]
fn portal_and_mob_wait_for_their_local_streamed_floor_without_a_player_loading_flag() {
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    for (id, npc_type) in [(FUSION_PORTAL_ID, 2695), (267_400, 2674)] {
        app.world_mut()
            .resource_mut::<TutorialActorCommandQueue>()
            .spawn(spawn(id, npc_type, [56_431, 72_416, -9_100], None));
    }
    app.update();
    let actors: Vec<_> = [FUSION_PORTAL_ID, 267_400]
        .into_iter()
        .map(|id| {
            let entity = app
                .world()
                .resource::<TutorialActorRegistry>()
                .entity(id)
                .unwrap();
            (
                entity,
                app.world().get::<Transform>(entity).unwrap().translation,
            )
        })
        .collect();
    // Collision elsewhere does not mean the actors' own floor is resident.
    app.world_mut().spawn((
        GlobalTransform::IDENTITY,
        NativeHeightmapCollider::test_heightfield(2, 2, &[0.0; 4]),
    ));
    for _ in 0..80 {
        app.update();
    }
    for (entity, initial) in &actors {
        assert_eq!(
            app.world().get::<Transform>(*entity).unwrap().translation,
            *initial,
            "actor fell while its own streamed floor was absent"
        );
    }
    let initial = actors[0].1;
    let floor = app
        .world_mut()
        .spawn((
            GlobalTransform::from_translation(Vec3::new(initial.x + 0.5, -91.0, initial.z - 0.5)),
            NativeHeightmapCollider::test_heightfield(2, 2, &[0.0; 4]),
        ))
        .id();
    for _ in 0..20 {
        app.update();
    }
    for (entity, _) in &actors {
        assert_eq!(
            app.world().get::<Transform>(*entity).unwrap().translation.y,
            -91.0
        );
    }
    app.world_mut().despawn(floor);
    for _ in 0..80 {
        app.update();
    }
    for (entity, _) in &actors {
        assert_eq!(
            app.world().get::<Transform>(*entity).unwrap().translation.y,
            -91.0,
            "retiring the local floor must not strand an actor below its reload"
        );
    }
    app.world_mut().spawn((
        GlobalTransform::from_translation(Vec3::new(initial.x + 0.5, -91.0, initial.z - 0.5)),
        NativeHeightmapCollider::test_heightfield(2, 2, &[0.0; 4]),
    ));
    for _ in 0..20 {
        app.update();
    }
    for (entity, _) in &actors {
        assert_eq!(
            app.world().get::<Transform>(*entity).unwrap().translation.y,
            -91.0
        );
        assert_eq!(
            app.world()
                .get::<TutorialActorGrounding>(*entity)
                .unwrap()
                .vertical_velocity,
            0.0
        );
    }
}

#[test]
fn retrobution_npc_grounding_snaps_up_falls_and_clamps_at_the_surface() {
    assert_eq!(
        advance_tutorial_actor_grounding(4.0, Some(4.25), -3.0, 0.1),
        (4.25, 0.0)
    );

    let (falling_y, falling_velocity) = advance_tutorial_actor_grounding(5.0, Some(4.0), 0.0, 0.1);
    assert!((falling_y - 4.9).abs() < f32::EPSILON);
    assert!((falling_velocity + 1.0).abs() < f32::EPSILON);

    assert_eq!(
        advance_tutorial_actor_grounding(4.05, Some(4.0), -1.0, 0.1),
        (4.0, 0.0)
    );
    assert_eq!(
        advance_tutorial_actor_grounding(5.0, None, -9.5, 0.1),
        (4.0, -10.0)
    );
}

#[test]
fn fusion_portal_waits_for_warp_collision_before_applying_gravity() {
    use crate::movement::{LegacyPlayerController, LegacyWorldColliderPending};

    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    let player = app
        .world_mut()
        .spawn((
            LegacyPlayerController::from_baseline_table(),
            LegacyWorldColliderPending,
        ))
        .id();
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(
            FUSION_PORTAL_ID,
            2695,
            [56_431, 72_416, -9_100],
            None,
        ));
    app.update();
    let portal = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(FUSION_PORTAL_ID)
        .unwrap();
    let initial = app.world().get::<Transform>(portal).unwrap().translation;
    assert_eq!(initial.y, -90.0);

    // A slow warp must not spend five seconds integrating gravity into
    // a world whose floor has not been installed yet.
    for _ in 0..50 {
        app.update();
    }
    assert_eq!(
        app.world().get::<Transform>(portal).unwrap().translation,
        initial
    );
    assert_eq!(
        app.world()
            .get::<TutorialActorGrounding>(portal)
            .unwrap()
            .vertical_velocity,
        0.0
    );

    app.world_mut().spawn((
        GlobalTransform::from_translation(Vec3::new(initial.x + 0.5, -91.0, initial.z - 0.5)),
        NativeHeightmapCollider::test_heightfield(2, 2, &[0.0; 4]),
    ));
    app.world_mut()
        .entity_mut(player)
        .remove::<LegacyWorldColliderPending>();
    for _ in 0..20 {
        app.update();
    }
    let grounded = app.world().get::<Transform>(portal).unwrap().translation;
    assert_eq!(grounded, Vec3::new(initial.x, -91.0, initial.z));
    assert_eq!(
        app.world()
            .get::<TutorialActorGrounding>(portal)
            .unwrap()
            .vertical_velocity,
        0.0
    );
}

#[test]
fn portal_waits_for_its_tile_platform_even_when_terrain_is_already_resident() {
    use crate::world::{NativeWorldPresentationStatus, NativeWorldSceneRoot, NativeWorldScope};
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    let tile = app
        .world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "delayed tutorial collision".into(),
                tile: [1, 1],
                scope: NativeWorldScope::Tutorial,
                selection_scope: NativeWorldScope::Tutorial,
            },
            NativeWorldPresentationStatus::Loading,
        ))
        .id();
    // The actor is above a platform that has not been cooked yet. Terrain alone
    // must not let it settle below that platform while its owning tile loads.
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(
            FUSION_PORTAL_ID,
            2695,
            [56_431, 72_416, -9_100],
            None,
        ));
    app.update();
    let portal = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(FUSION_PORTAL_ID)
        .unwrap();
    let initial = app.world().get::<Transform>(portal).unwrap().translation;
    let floor = app
        .world_mut()
        .spawn((
            GlobalTransform::from_translation(Vec3::new(initial.x + 0.5, -96.0, initial.z - 0.5)),
            NativeHeightmapCollider::test_heightfield(2, 2, &[0.0; 4]),
        ))
        .id();
    for _ in 0..80 {
        app.update();
    }
    assert_eq!(
        app.world().get::<Transform>(portal).unwrap().translation,
        initial
    );
    app.world_mut()
        .entity_mut(floor)
        .insert(GlobalTransform::from_translation(Vec3::new(
            initial.x + 0.5,
            -91.0,
            initial.z - 0.5,
        )));
    app.world_mut()
        .entity_mut(tile)
        .insert(NativeWorldPresentationStatus::Ready);
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(
        app.world().get::<Transform>(portal).unwrap().translation.y,
        -91.0
    );
}

#[test]
fn resident_floor_below_the_short_probe_still_allows_normal_gravity() {
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(
            FUSION_PORTAL_ID,
            2695,
            [56_431, 72_416, -7_000],
            None,
        ));
    app.world_mut().spawn((
        GlobalTransform::from_translation(Vec3::new(-563.81, -91.0, 723.66)),
        NativeHeightmapCollider::test_heightfield(2, 2, &[0.0; 4]),
    ));
    for _ in 0..80 {
        app.update();
    }
    let portal = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(FUSION_PORTAL_ID)
        .unwrap();
    assert_eq!(
        app.world().get::<Transform>(portal).unwrap().translation.y,
        -91.0
    );
    assert_eq!(
        app.world()
            .get::<TutorialActorGrounding>(portal)
            .unwrap()
            .vertical_velocity,
        0.0
    );
}
