use crate::tutorial_nano_presentation::*;
use bevy::asset::{AssetApp, AssetPlugin};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<WorldAsset>()
        .init_asset::<AnimationClip>()
        .init_asset::<AnimationGraph>()
        .init_asset::<LegacyModelMaterial>()
        .add_plugins(TutorialNanoPresentationPlugin);
    app
}

#[test]
fn exact_animation_contract_keeps_direct_names_and_resolves_stand_variants() {
    let mut random = LegacyNanoStandRandomStream::with_seed(0x1234_5678);
    let mut machine = LegacyNanoAnimationMachine::default();
    machine.request(
        event_nano_mode_for_action("call2"),
        "call2",
        event_nano_blend_for_action("call2"),
    );
    assert_eq!(machine.clip(), Some("call2"));
    for _ in 0..6 {
        machine.request_stand(&mut random);
        assert!(matches!(
            machine.clip(),
            Some("stand1" | "stand2" | "stand3")
        ));
    }
    assert_eq!(random.draw_count(), 6);
    assert_eq!(
        TUTORIAL_NANO_REQUIRED_ANIMATION_CLIPS,
        [
            "call2", "happy", "flex", "call", "hello", "stand1", "stand2", "stand3", "dance2",
            "dance5", "shocked"
        ]
    );
}

#[test]
fn identical_happy_clip_preserves_distinct_legacy_dispatch_and_blend() {
    let mut random = LegacyNanoStandRandomStream::with_seed(1);
    let mut machine = LegacyNanoAnimationMachine::default();

    TutorialNanoPresentationAnimation::Happy.apply(&mut machine, &mut random);
    assert_eq!(machine.mode(), LegacyNanoAnimationMode::Happy);
    assert_eq!(machine.clip(), Some("happy"));
    assert_eq!(machine.blend(), LegacyAnimationBlend::CrossFade100Ms);

    TutorialNanoPresentationAnimation::Emote("happy".to_owned())
        .apply(&mut machine, &mut random);
    assert_eq!(machine.mode(), LegacyNanoAnimationMode::Emote);
    assert_eq!(machine.clip(), Some("happy"));
    assert_eq!(machine.blend(), LegacyAnimationBlend::CrossFade300Ms);
}

#[test]
fn startup_ready_requires_both_gltf_contract_and_exact_face_bind() {
    let mut state = TutorialNanoPresentationState {
        status: TutorialNanoPresentationStatus::Loading,
        ..default()
    };
    state.asset_contract_ready = true;
    state.refresh_readiness();
    assert_eq!(state.status(), &TutorialNanoPresentationStatus::Loading);

    state.face_texture_bound = true;
    state.refresh_readiness();
    assert_eq!(state.status(), &TutorialNanoPresentationStatus::Ready);
}

#[test]
fn command_queue_exposes_live_transform_and_destroy_removes_the_presentation() {
    let mut app = app();
    let spawn_transform =
        Transform::from_xyz(1.0, 2.0, 3.0).with_rotation(Quat::from_rotation_y(0.75));
    app.world_mut()
        .resource_mut::<TutorialNanoPresentationCommandQueue>()
        .spawn(TutorialNanoPresentationSpawn::at(spawn_transform));
    app.update();

    let entity = app
        .world()
        .resource::<TutorialNanoPresentationState>()
        .entity()
        .expect("spawn command must publish the presentation entity");
    assert_eq!(
        app.world()
            .resource::<TutorialNanoPresentationState>()
            .live_transform(),
        Some(spawn_transform)
    );
    assert_eq!(app.world().get::<Transform>(entity), Some(&spawn_transform));

    let moved = Vec3::new(-4.0, 5.0, 6.0);
    app.world_mut()
        .resource_mut::<TutorialNanoPresentationCommandQueue>()
        .set_translation(moved);
    app.update();
    assert_eq!(
        app.world()
            .resource::<TutorialNanoPresentationState>()
            .live_transform()
            .unwrap()
            .translation,
        moved
    );

    app.world_mut()
        .resource_mut::<TutorialNanoPresentationCommandQueue>()
        .destroy();
    app.update();
    assert!(
        app.world()
            .resource::<TutorialNanoPresentationState>()
            .entity()
            .is_none()
    );
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn face_override_contract_matches_event_nano_controller_inputs() {
    assert_eq!(TUTORIAL_NANO_LEGACY_ROUTE, "nano/nano_buttercup.kfm");
    assert_eq!(
        TUTORIAL_NANO_FACE_MATERIAL_NAME,
        "nano_buttercup-sub-link_b.dds"
    );
    assert_eq!(
        TUTORIAL_NANO_FACE_TEXTURE_PATH,
        "characters/nanos/nano_buttercup/runtime-textures/nano_buttercup_face.png"
    );
}
