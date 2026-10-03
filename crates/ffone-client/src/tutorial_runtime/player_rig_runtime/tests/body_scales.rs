use super::super::animation_prepare_tutorial_player_animation_adapter::motion_without_body_scales;
use super::*;
use bevy::animation::{AnimationPlugin, animated_field};
use bevy::math::curve::{EaseFunction, EasingCurve};

#[test]
fn emote_motion_preserves_body_scale_ownership_in_bevy_graph() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        AnimationPlugin,
    ));
    let body = AnimationTargetId::from_name(&Name::new("Bip01 Head"));
    let socket = AnimationTargetId::from_name(&Name::new("Bip01 helmet01"));
    let mut motion = AnimationClip::default();
    let rotation = Quat::from_rotation_z(0.6);
    motion.add_curve_to_target(
        body,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(rotation, rotation, EaseFunction::Linear),
        ),
    );
    let translation = Vec3::new(0.1, 0.2, 0.3);
    motion.add_curve_to_target(
        body,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            EasingCurve::new(translation, translation, EaseFunction::Linear),
        ),
    );
    for target in [body, socket] {
        motion.add_curve_to_target(
            target,
            AnimatableCurve::new(
                animated_field!(Transform::scale),
                EasingCurve::new(Vec3::ONE, Vec3::ONE, EaseFunction::Linear),
            ),
        );
    }
    let projected = motion_without_body_scales(&motion, &BTreeSet::from([body])).unwrap();
    assert_eq!(projected.duration(), motion.duration());
    assert_eq!(
        motion.curves()[&body].len(),
        3,
        "shared source stays intact"
    );
    assert!(motion_without_body_scales(&projected, &BTreeSet::from([body])).is_none());
    let [original, projected] = [motion, projected].map(|clip| {
        app.world_mut()
            .resource_mut::<Assets<AnimationClip>>()
            .add(clip)
    });
    for proportions in [Vec3::splat(0.85), Vec3::ONE, Vec3::new(1.1, 1.25, 0.9)] {
        let mut scale = AnimationClip::default();
        scale.add_curve_to_target(
            body,
            AnimatableCurve::new(
                animated_field!(Transform::scale),
                EasingCurve::new(proportions, proportions, EaseFunction::Linear),
            ),
        );
        let scale = app
            .world_mut()
            .resource_mut::<Assets<AnimationClip>>()
            .add(scale);
        let mut instances = Vec::new();
        for clip in [original.clone(), projected.clone()] {
            let mut graph = AnimationGraph::new();
            let composition = graph.add_additive_blend(1.0, graph.root);
            let motion_layer = graph.add_blend(1.0, composition);
            let motion_node = graph.add_clip(clip, 1.0, motion_layer);
            let scale_layer = graph.add_blend(1.0, composition);
            let scale_node = graph.add_clip(scale.clone(), 1.0, scale_layer);
            let graph = app
                .world_mut()
                .resource_mut::<Assets<AnimationGraph>>()
                .add(graph);
            let mut player = AnimationPlayer::default();
            player.start(motion_node).pause();
            player.start(scale_node).pause();
            let owner = app
                .world_mut()
                .spawn((player, AnimationGraphHandle(graph)))
                .id();
            let body_entity = app
                .world_mut()
                .spawn((Transform::default(), body, AnimatedBy(owner)))
                .id();
            let socket_entity = app
                .world_mut()
                .spawn((Transform::default(), socket, AnimatedBy(owner)))
                .id();
            instances.push((body_entity, socket_entity));
        }
        app.update();
        app.update();
        let broken = app.world().get::<Transform>(instances[0].0).unwrap();
        assert!(broken.scale.abs_diff_eq(proportions + Vec3::ONE, 1e-5));
        let fixed = app.world().get::<Transform>(instances[1].0).unwrap();
        assert!(fixed.scale.abs_diff_eq(proportions, 1e-5), "{fixed:?}");
        assert!(fixed.translation.abs_diff_eq(translation, 1e-5));
        assert!(fixed.rotation.angle_between(rotation) < 1e-3);
        assert!(
            app.world()
                .get::<Transform>(instances[1].1)
                .unwrap()
                .scale
                .abs_diff_eq(Vec3::ONE, 1e-5)
        );
    }
}
