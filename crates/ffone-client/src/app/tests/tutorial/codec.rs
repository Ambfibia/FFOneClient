use super::*;

#[test]
fn post_cutscene_tutorial_entry_still_fails_closed_on_first_frame_resources() {
    let plan = TutorialStartupProbePlan {
        admission: vec![
            TutorialStartupProbe::Ready,
            TutorialStartupProbe::Blocked("initial tutorial actor failed".to_owned()),
        ],
        background: vec![TutorialStartupProbe::Ready],
    };

    assert_eq!(
        plan.readiness(),
        TutorialStartupProbe::Blocked("initial tutorial actor failed".to_owned())
    );
}

#[test]
fn dexter_cutscene_cleanup_tolerates_an_audio_auto_despawn_in_the_same_frame() {
    let mut world = World::new();
    world.init_resource::<DexterShipCutsceneRuntime>();
    let audio = world
        .spawn((
            DexterShipEntity,
            AudioPlayer::new(Handle::<AudioSource>::default()),
        ))
        .id();
    let mut cleanup = IntoSystem::into_system(cleanup_dexter_ship_cutscene);
    cleanup.initialize(&mut world);

    cleanup
        .run_without_applying_deferred((), &mut world)
        .unwrap();
    assert!(world.get_entity(audio).is_ok());
    world.despawn(audio);
    cleanup.apply_deferred(&mut world);

    assert!(world.get_entity(audio).is_err());
    let runtime = world.resource::<DexterShipCutsceneRuntime>();
    assert_eq!(runtime.elapsed, 0.0);
    assert_eq!(runtime.fired_audio, 0);
    assert!(!runtime.presentation_ready);
    assert!(!runtime.audio_started);
    assert!(!runtime.tutorial_world_requested);
}
