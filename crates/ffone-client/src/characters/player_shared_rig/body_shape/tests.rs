use super::*;
use bevy::animation::{AnimatedBy, AnimationPlugin, AnimationTargetId, animated_field};
use bevy::math::curve::{EaseFunction, EasingCurve};

#[test]
fn production_body_clips_change_all_fifteen_poses_for_both_genders() {
    use bevy::gltf::{Gltf, GltfPlugin};
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin {
            file_path: root.to_string_lossy().into_owned(),
            ..default()
        },
        bevy::world_serialization::WorldSerializationPlugin,
        AnimationPlugin,
        GltfPlugin::default(),
    ))
    .init_asset::<Mesh>()
    .init_asset::<StandardMaterial>()
    .init_asset::<Image>()
    .init_asset::<SkinnedMeshInverseBindposes>();
    app.finish();
    app.cleanup();
    for gender in ["male", "female"] {
        let gltf_handle: Handle<Gltf> = app.world().resource::<AssetServer>().load(format!(
            "characters/player/{gender}/base/{gender}_skeleton.glb"
        ));
        let started = std::time::Instant::now();
        while !app
            .world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(gltf_handle.id())
        {
            assert!(
                started.elapsed().as_secs() < 30,
                "{gender} skeleton failed to load"
            );
            app.update();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let gltf = app
            .world()
            .resource::<Assets<Gltf>>()
            .get(&gltf_handle)
            .unwrap();
        let handles = ["height_Add", "shape_Add", "height", "shape"]
            .map(|name| gltf.named_animations[name].clone());
        let clips = app.world().resource::<Assets<AnimationClip>>();
        let durations = handles
            .clone()
            .map(|handle| clips.get(&handle).unwrap().duration());
        let targets = handles
            .iter()
            .flat_map(|handle| clips.get(handle).unwrap().curves().keys().copied())
            .collect::<BTreeSet<_>>();
        assert_eq!(targets.len(), 19);
        let (graph, idle, nodes) = body_graph(gltf.named_animations["stand1"].clone(), handles);
        let graph = app
            .world_mut()
            .resource_mut::<Assets<AnimationGraph>>()
            .add(graph);
        let body = NativePlayerBodyShapePlayback { nodes, durations };
        let mut player = AnimationPlayer::default();
        player.start(idle).set_seek_time(0.3).pause();
        let owner = app
            .world_mut()
            .spawn((player, AnimationGraphHandle(graph)))
            .id();
        let bones = targets
            .into_iter()
            .map(|id| {
                app.world_mut()
                    .spawn((Transform::default(), (id, AnimatedBy(owner))))
                    .id()
            })
            .collect::<Vec<_>>();
        let mut poses = Vec::new();
        for build in 0..=2 {
            for height in 0..=4 {
                body.apply(
                    &mut app.world_mut().get_mut::<AnimationPlayer>(owner).unwrap(),
                    NativePlayerBodyShape {
                        height,
                        body: build,
                    },
                );
                for _ in 0..3 {
                    app.update();
                }
                let pose = bones
                    .iter()
                    .map(|bone| *app.world().get::<Transform>(*bone).unwrap())
                    .collect::<Vec<_>>();
                assert!(pose.iter().all(|t| t.translation.is_finite()
                    && t.rotation.is_finite()
                    && t.scale.is_finite()));
                assert!(
                    !poses.contains(&pose),
                    "{gender} duplicate pose height={height} body={build}"
                );
                poses.push(pose);
            }
        }
        assert_eq!(poses.len(), 15);
        for bone in bones {
            app.world_mut().despawn(bone);
        }
        app.world_mut().despawn(owner);
    }
}

#[test]
fn body_shape_changes_bone_pose_in_place_and_preserves_idle_rotation() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        AnimationPlugin,
    ));
    let target = AnimationTargetId::from_name(&Name::new("body"));
    let rotation = Quat::from_rotation_z(2.8);
    let mut idle = AnimationClip::default();
    idle.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(rotation, rotation, EaseFunction::Linear),
        ),
    );
    idle.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            EasingCurve::new(Vec3::Y, Vec3::Y, EaseFunction::Linear),
        ),
    );
    let idle = app
        .world_mut()
        .resource_mut::<Assets<AnimationClip>>()
        .add(idle);
    let handles = [Vec3::Y, Vec3::X, Vec3::ZERO, Vec3::ZERO].map(|offset| {
        let mut clip = AnimationClip::default();
        clip.add_curve_to_target(
            target,
            AnimatableCurve::new(
                animated_field!(Transform::translation),
                EasingCurve::new(Vec3::ZERO, offset, EaseFunction::Linear),
            ),
        );
        app.world_mut()
            .resource_mut::<Assets<AnimationClip>>()
            .add(clip)
    });
    let (graph, idle_node, nodes) = body_graph(idle, handles);
    let graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    let body = NativePlayerBodyShapePlayback {
        nodes,
        durations: [1.0; 4],
    };
    let mut player = AnimationPlayer::default();
    player.start(idle_node).repeat();
    let owner = app
        .world_mut()
        .spawn((player, AnimationGraphHandle(graph)))
        .id();
    let bone = app
        .world_mut()
        .spawn((Transform::default(), (target, AnimatedBy(owner))))
        .id();
    for height in 0..=4 {
        for build in 0..=2 {
            body.apply(
                &mut app.world_mut().get_mut::<AnimationPlayer>(owner).unwrap(),
                NativePlayerBodyShape {
                    height,
                    body: build,
                },
            );
            for _ in 0..3 {
                app.update();
            }
            let transform = app.world().get::<Transform>(bone).unwrap();
            let expected =
                Vec3::new(f32::from(build) / 2.0, 2.0 - f32::from(height) / 4.0, 0.0);
            assert!(
                transform.translation.abs_diff_eq(expected, 1e-5),
                "height={height} body={build}: {transform:?}"
            );
            assert!(transform.rotation.angle_between(rotation) < 1e-3);
            let player = app.world().get::<AnimationPlayer>(owner).unwrap();
            assert!(!player.animation(idle_node).unwrap().is_paused());
            assert_eq!(player.playing_animations().count(), 5);
        }
    }
    let mut player = app.world_mut().get_mut::<AnimationPlayer>(owner).unwrap();
    player.stop_all();
    player.start(idle_node).repeat();
    body.apply(&mut player, NativePlayerBodyShape { height: 0, body: 2 });
    assert_eq!(player.playing_animations().count(), 5);
}
