use super::*;

#[test]
fn dexter_ship_performances_finish_without_rewinding_and_run_still_loops() {
    use crate::app::dexter_ship_drive::apply_dexter_ship_animations;
    use crate::app::dexter_ship_scene::set_dexter_ship_actor_clip;
    use bevy::ecs::system::RunSystemOnce;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<AnimationClip>()
        .init_asset::<AnimationGraph>()
        .init_resource::<GameplayLoadingState>()
        .insert_resource(DexterShipCutsceneRuntime {
            presentation_ready: true,
            ..default()
        });
    let mut named_animations = bevy::platform::collections::HashMap::default();
    for name in ["observe", "excellent", "deedeeno", "busyrun"] {
        let mut clip = AnimationClip::default();
        clip.set_duration(1.0);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<AnimationClip>>()
            .add(clip);
        named_animations.insert(Box::<str>::from(name), handle);
    }
    let gltf = app.world_mut().resource_mut::<Assets<Gltf>>().add(Gltf {
        scenes: default(),
        named_scenes: default(),
        meshes: default(),
        named_meshes: default(),
        materials: default(),
        named_materials: default(),
        nodes: default(),
        named_nodes: default(),
        skins: default(),
        named_skins: default(),
        default_scene: None,
        animations: default(),
        named_animations,
        source: None,
    });
    let root = app
        .world_mut()
        .spawn(DexterShipActor {
            role: DexterShipActorRole::Dexter,
            gltf,
            clip: "observe",
            revision: 1,
        })
        .id();
    let player_entity = app
        .world_mut()
        .spawn((AnimationPlayer::default(), ChildOf(root)))
        .id();
    for name in ["observe", "excellent", "deedeeno", "busyrun"] {
        set_dexter_ship_actor_clip(
            &mut app.world_mut().get_mut::<DexterShipActor>(root).unwrap(),
            name,
        );
        app.world_mut()
            .run_system_once(apply_dexter_ship_animations)
            .unwrap();
        for _ in 0..6 {
            // Reapplying the same revision must not restart a finished clip.
            app.world_mut()
                .run_system_once(apply_dexter_ship_animations)
                .unwrap();
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(0.25));
            app.world_mut()
                .run_system_once(bevy::animation::advance_animations)
                .unwrap();
        }
        let player = app.world().get::<AnimationPlayer>(player_entity).unwrap();
        let (_, active) = player.playing_animations().next().unwrap();
        if name == "busyrun" {
            assert!(!active.is_finished());
            assert!((active.seek_time() - 0.5).abs() < 0.001);
        } else {
            assert!(active.is_finished(), "{name} restarted after completion");
            assert!((active.seek_time() - 1.0).abs() < 0.001);
            assert_eq!(active.completions(), 1);
        }
    }
}

#[test]
fn dexter_ship_loader_waits_for_every_scene_and_first_animation_revision() {
    assert!(dexter_ship_presentation_ready(4, 4, 4, 3, 3, 3, 1, 1, 1));
    assert!(!dexter_ship_presentation_ready(4, 4, 3, 3, 3, 3, 1, 1, 1));
    assert!(!dexter_ship_presentation_ready(4, 4, 4, 3, 3, 2, 1, 1, 1));
    assert!(!dexter_ship_presentation_ready(4, 4, 4, 3, 3, 3, 1, 1, 0));
    assert!(dexter_ship_presentation_ready(3, 3, 3, 3, 3, 3, 0, 0, 0));
}

#[test]
fn tutorial_loader_opens_only_on_the_source_staying_pose() {
    assert!(!tutorial_player_startup_pose_ready(
        &TutorialSelectedPlayerRigStatus::Loading,
        Some(TutorialPlayerClip::Staying),
    ));
    assert!(!tutorial_player_startup_pose_ready(
        &TutorialSelectedPlayerRigStatus::Ready,
        Some(TutorialPlayerClip::Stand1),
    ));
    assert!(tutorial_player_startup_pose_ready(
        &TutorialSelectedPlayerRigStatus::Ready,
        Some(TutorialPlayerClip::Staying),
    ));
}
