use super::*;

#[test]
fn npc_motion_reaches_horizontal_destination_without_using_packet_y() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, advance_network_npc_motion_0104);
    let entity = app
        .world_mut()
        .spawn((
            NetworkNpc0104 {
                npc_id: 7,
                npc_type: 728,
            },
            Transform::from_xyz(0.0, 0.75, 0.0),
            NetworkNpcMotion0104 {
                destination: Vec3::new(2.0, 8.0, 0.0),
                speed: 3.0,
                move_style: 1,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.5));
    app.update();
    assert!(
        app.world()
            .get::<Transform>(entity)
            .unwrap()
            .translation
            .abs_diff_eq(Vec3::new(1.5, 0.75, 0.0), 0.000_01)
    );
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_some());

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.5));
    app.update();
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation,
        Vec3::new(2.0, 0.75, 0.0)
    );
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_some());
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(160));
    app.update();
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_none());
    assert!(
        app.world()
            .get::<NetworkNpcReadyAnimation0104>(entity)
            .is_some(),
        "run completion must enter the clean AttackReady continuation"
    );

    app.world_mut().entity_mut(entity).insert((
        NetworkNpcMotion0104 {
            destination: Vec3::new(3.0, 12.0, 0.0),
            speed: 3.0,
            move_style: 0,
        },
        NetworkNpcReadyAnimation0104,
    ));
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.5));
    app.update();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(160));
    app.update();
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_none());
    assert!(
        app.world()
            .get::<NetworkNpcReadyAnimation0104>(entity)
            .is_none(),
        "walk completion must settle to stand instead of combat ready"
    );
}

#[test]
fn remote_pc_packet_state_selects_each_weapon_profile() {
    assert_eq!(
        network_pc_animation_clip(RemoteAnimationState::Idle, None),
        ("stand1", true)
    );
    assert_eq!(
        network_pc_animation_clip(RemoteAnimationState::Moving { direction_key: 1 }, None),
        ("run", true)
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Idle,
            Some(PlayerWeaponAnimationProfile::Stick),
        ),
        ("stickstand1", true)
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Moving { direction_key: 1 },
            Some(PlayerWeaponAnimationProfile::Pistol),
        ),
        ("pistolrun", true)
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Moving { direction_key: 5 },
            Some(PlayerWeaponAnimationProfile::Rifle),
        ),
        ("riflerunback", true)
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Jumping {
                direction_key: 1,
                double_jump: None,
            },
            Some(PlayerWeaponAnimationProfile::Rifle),
        ),
        ("riflejump", false)
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Emoting {
                clip: TutorialPlayerClip::Dance5,
            },
            Some(PlayerWeaponAnimationProfile::Rifle),
        ),
        ("dance5", false),
        "AvatarEmote owns the full body and must not receive a weapon prefix"
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Idle,
            Some(PlayerWeaponAnimationProfile::Bomb),
        ),
        ("bombstand1", true)
    );
    assert_eq!(
        network_pc_animation_clip(
            RemoteAnimationState::Moving { direction_key: 1 },
            Some(PlayerWeaponAnimationProfile::Rocket),
        ),
        ("rocketrun", true)
    );
}

#[test]
fn npc_high_layer_adds_only_its_difference_from_the_first_frame() {
    use bevy::animation::{AnimatedBy, AnimationPlugin, animated_field};
    use bevy::math::curve::{EaseFunction, EasingCurve};

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), AnimationPlugin));
    let bone = AnimationTargetId::from_name(&Name::new("Bip01 Spine"));
    let prop = AnimationTargetId::from_name(&Name::new("Bip01 Mouth00"));
    let low_pose = Quat::from_rotation_z(0.4);
    // Like the road golem wound: authored from the bind pose.
    let bind_pose = Quat::from_rotation_y(1.2);
    let flinch = Quat::from_rotation_x(0.3);
    let mut ready = AnimationClip::default();
    ready.add_curve_to_target(
        bone,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(low_pose, low_pose, EaseFunction::Linear),
        ),
    );
    ready.add_curve_to_target(
        bone,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            EasingCurve::new(Vec3::Y, Vec3::Y, EaseFunction::Linear),
        ),
    );
    let mut wound = AnimationClip::default();
    wound.add_curve_to_target(
        bone,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            EasingCurve::new(bind_pose, flinch * bind_pose, EaseFunction::Linear),
        ),
    );
    wound.add_curve_to_target(
        bone,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            EasingCurve::new(
                Vec3::X * 5.0,
                Vec3::new(5.0, 0.5, 0.0),
                EaseFunction::Linear,
            ),
        ),
    );
    // No low-layer clip owns this scale; it must not collapse to zero.
    wound.add_curve_to_target(
        prop,
        AnimatableCurve::new(
            animated_field!(Transform::scale),
            EasingCurve::new(Vec3::ONE, Vec3::splat(1.5), EaseFunction::Linear),
        ),
    );
    let (ready, wound) = {
        let mut clips = app.world_mut().resource_mut::<Assets<AnimationClip>>();
        (clips.add(ready), clips.add(wound))
    };
    let prepared =
        app.world_mut()
            .resource_scope(|world, mut graphs: Mut<Assets<AnimationGraph>>| {
                prepare_network_npc_animation_graph_0104(
                    [("ready", &ready), ("wound", &wound)],
                    world.resource::<Assets<AnimationClip>>(),
                    &mut graphs,
                )
                .expect("both clips are loaded")
            });
    let pair = prepared.additive_nodes["wound"];
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();
    cross_fade_network_npc_low_layer_0104(
        &mut player,
        &mut transitions,
        prepared.nodes["ready"],
        Duration::ZERO,
        1.0,
        true,
    );
    restart_network_npc_additive_pair_0104(&mut player, &prepared, pair);
    player.animation_mut(pair.clip).unwrap().pause();
    let player = app
        .world_mut()
        .spawn((
            player,
            transitions,
            AnimationGraphHandle(prepared.graph.clone()),
        ))
        .id();
    let bone_entity = app
        .world_mut()
        .spawn((Transform::default(), (bone, AnimatedBy(player))))
        .id();
    let prop_entity = app
        .world_mut()
        .spawn((Transform::default(), (prop, AnimatedBy(player))))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let at_start = *app.world().get::<Transform>(bone_entity).unwrap();
    assert!(
        at_start.rotation.angle_between(low_pose) < 1e-4,
        "{at_start:?}"
    );
    assert!(
        at_start.translation.abs_diff_eq(Vec3::Y, 1e-4),
        "{at_start:?}"
    );
    app.world_mut()
        .get_mut::<AnimationPlayer>(player)
        .unwrap()
        .animation_mut(pair.clip)
        .unwrap()
        .set_seek_time(1.0);
    app.update();
    let flinched = *app.world().get::<Transform>(bone_entity).unwrap();
    assert!(
        flinched.rotation.angle_between(flinch * low_pose) < 1e-3,
        "{flinched:?}"
    );
    assert!(
        flinched
            .translation
            .abs_diff_eq(Vec3::new(0.0, 1.5, 0.0), 1e-4),
        "{flinched:?}"
    );
    assert_eq!(
        app.world().get::<Transform>(prop_entity).unwrap().scale,
        Vec3::ONE
    );
}
