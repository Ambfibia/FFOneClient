use super::super::damage::PlayerDamagePlayback;
use super::*;
use bevy::animation::{AnimationPlugin, animated_field};
use bevy::math::curve::{EaseFunction, EasingCurve};

#[test]
fn damage_adds_first_frame_delta_over_attack_without_restarting_or_changing_body_scale() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        AnimationPlugin,
    ));
    let target = AnimationTargetId::from_name(&Name::new("Spine"));
    let rotation = Quat::from_rotation_y(0.7);
    let reference = Quat::from_rotation_x(0.4);
    let end = Quat::from_rotation_z(0.6) * reference;
    let mut attack = AnimationClip::default();
    attack.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(rotation, rotation, EaseFunction::Linear),
        ),
    );
    attack.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            EasingCurve::new(Vec3::Y, Vec3::Y, EaseFunction::Linear),
        ),
    );
    attack.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::scale),
            EasingCurve::new(Vec3::splat(0.85), Vec3::splat(0.85), EaseFunction::Linear),
        ),
    );
    let mut wound = AnimationClip::default();
    wound.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(reference, end, EaseFunction::Linear),
        ),
    );
    wound.add_curve_to_target(
        target,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            EasingCurve::new(
                Vec3::splat(2.0),
                Vec3::splat(2.0) + Vec3::X,
                EaseFunction::Linear,
            ),
        ),
    );
    let [attack, wound] = [attack, wound].map(|clip| {
        app.world_mut()
            .resource_mut::<Assets<AnimationClip>>()
            .add(clip)
    });
    let (mut graph, attack_node) = AnimationGraph::from_clip(attack);
    let damage = PlayerDamagePlayback::attach(&mut graph, wound);
    let graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    let mut player = AnimationPlayer::default();
    player.start(attack_node).pause();
    damage.play(&mut player);
    player.animation_mut(damage.motion).unwrap().pause();
    let owner = app
        .world_mut()
        .spawn((player, AnimationGraphHandle(graph)))
        .id();
    let bone = app
        .world_mut()
        .spawn((Transform::default(), target, AnimatedBy(owner)))
        .id();
    for time in [0.0, 0.5, 0.9] {
        app.world_mut()
            .get_mut::<AnimationPlayer>(owner)
            .unwrap()
            .animation_mut(damage.motion)
            .unwrap()
            .set_seek_time(time);
        app.update();
        app.update();
        let pose = app.world().get::<Transform>(bone).unwrap();
        assert!(
            pose.translation.abs_diff_eq(Vec3::Y + Vec3::X * time, 1e-5),
            "{pose:?}"
        );
        assert!(
            pose.rotation
                .angle_between(reference.slerp(end, time) * reference.inverse() * rotation)
                < 1e-3,
            "{pose:?}"
        );
        assert!(pose.scale.abs_diff_eq(Vec3::splat(0.85), 1e-5));
        let mut player = app.world_mut().get_mut::<AnimationPlayer>(owner).unwrap();
        damage.play(&mut player);
        assert_eq!(player.animation(damage.motion).unwrap().seek_time(), time);
        assert!(player.is_playing_animation(attack_node));
    }
    damage.advance(
        &mut app.world_mut().get_mut::<AnimationPlayer>(owner).unwrap(),
        true,
    );
    app.update();
    let pose = app.world().get::<Transform>(bone).unwrap();
    assert!(pose.translation.abs_diff_eq(Vec3::Y, 1e-5));
    assert!(pose.rotation.angle_between(rotation) < 1e-3);
    assert!(
        app.world()
            .get::<AnimationPlayer>(owner)
            .unwrap()
            .animation(damage.motion)
            .is_none()
    );
}
