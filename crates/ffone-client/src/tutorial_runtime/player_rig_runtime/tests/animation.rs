use super::*;

#[test]
fn skyway_attachment_preserves_world_pose_and_scale_at_parenting() {
    for scale in [0.364_763_6, 0.5, 1.0, 1.7] {
        let socket = GlobalTransform::from(Transform {
            translation: Vec3::new(12.0, 3.4, -6.0),
            rotation: Quat::from_euler(EulerRot::YXZ, 0.7, -0.4, 1.2),
            scale: Vec3::splat(scale),
        });
        let local = skyway_attachment_transform(&socket).unwrap();
        let attached = socket.mul_transform(local);
        let (actual_scale, actual_rotation, actual_position) =
            attached.to_scale_rotation_translation();
        assert!(actual_scale.abs_diff_eq(Vec3::ONE, 1e-5));
        assert!(actual_position.abs_diff_eq(socket.translation(), 1e-5));
        let expected_rotation =
            socket.rotation() * standard_player_attachment_placement().item_local.rotation;
        assert!(actual_rotation.angle_between(expected_rotation).abs() < 1e-3);
        // The model's internal offsets (and therefore its seat) keep their
        // authored size instead of being pulled toward the attachment bone.
        let seat = Vec3::new(0.0, 0.93, -0.17);
        assert!(
            attached
                .transform_point(seat)
                .abs_diff_eq(socket.translation() + expected_rotation * seat, 1e-5)
        );
    }
    assert!(skyway_attachment_transform(&GlobalTransform::from_scale(Vec3::ZERO)).is_none());
}

#[test]
fn dash_selects_weapon_clips_and_water_keeps_its_fixed_upper_pose() {
    for (profile, ground, upper) in [
        (
            None,
            TutorialPlayerClip::StickDash,
            Some(TutorialPlayerClip::StickDodgeUpper),
        ),
        (
            Some(PlayerWeaponAnimationProfile::Stick),
            TutorialPlayerClip::StickDash,
            Some(TutorialPlayerClip::StickDodgeUpper),
        ),
        (
            Some(PlayerWeaponAnimationProfile::Pistol),
            TutorialPlayerClip::RifleTumbling,
            Some(TutorialPlayerClip::StickDodgeUpper),
        ),
        (
            Some(PlayerWeaponAnimationProfile::Rifle),
            TutorialPlayerClip::RifleDash,
            Some(TutorialPlayerClip::RifleDodgeUpper),
        ),
        (
            Some(PlayerWeaponAnimationProfile::Bomb),
            TutorialPlayerClip::StickDash,
            None,
        ),
        (
            Some(PlayerWeaponAnimationProfile::Rocket),
            TutorialPlayerClip::StickDash,
            Some(TutorialPlayerClip::StickDodgeUpper),
        ),
    ] {
        assert_eq!(
            legacy_visual_tutorial_clip(LegacyVisualClip::Dash, profile, false),
            Some(ground)
        );
        assert_eq!(
            legacy_visual_tutorial_clip(LegacyVisualClip::DashUpper, profile, false),
            upper
        );
        assert_eq!(
            legacy_visual_tutorial_clip(LegacyVisualClip::DashWaterUpper, profile, false),
            Some(TutorialPlayerClip::StickDodgeUpper)
        );
    }
}

#[test]
fn tutorial_dispatch_preserves_primary_play_and_end_animation_fades() {
    assert_eq!(
        tutorial_animation_transition_duration(TutorialPlayerAnimationDispatch::PlayImmediate),
        Duration::ZERO
    );
    let return_request = TutorialPlayerAnimationRequest::end_animation_cross_fade(
        PlayerRigGender::Male,
        TutorialPlayerClip::Stand1,
    );
    assert_eq!(
        tutorial_animation_transition_duration(return_request.dispatch),
        Duration::from_secs_f32(crate::avatar_action::LEGACY_END_ANIMATION_BLEND_SECONDS)
    );
}

#[test]
fn standup_return_crossfade_expires_the_terminal_pose() {
    let (_, nodes) = AnimationGraph::from_clips([
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
    ]);
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();
    transitions
        .play(&mut player, nodes[0], Duration::ZERO)
        .set_repeat(RepeatAnimation::Never)
        .resume();
    let request = TutorialPlayerAnimationRequest::end_animation_cross_fade(
        PlayerRigGender::Male,
        TutorialPlayerClip::Stand1,
    );
    apply_resolved_tutorial_animation(
        &mut player,
        &mut transitions,
        nodes[1],
        ContractResolvedTutorialPlayerAnimation {
            request,
            gltf_animation_index: 0,
            playback: TutorialPlayerClipPlayback::Loop,
        },
    );
    assert!(player.animation(nodes[0]).is_some());

    let mut app = App::new();
    app.init_resource::<Time>().add_systems(
        Update,
        (
            bevy::animation::transition::advance_transitions,
            bevy::animation::transition::expire_completed_transitions,
        )
            .chain(),
    );
    let entity = app.world_mut().spawn((player, transitions)).id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.31));
    app.update();

    let entity_ref = app.world().entity(entity);
    let player = entity_ref.get::<AnimationPlayer>().unwrap();
    let transitions = entity_ref.get::<AnimationTransitions>().unwrap();
    assert!(player.animation(nodes[0]).is_none());
    assert_eq!(transitions.get_main_animation(), Some(nodes[1]));
    assert_eq!(player.animation(nodes[1]).unwrap().weight(), 1.0);
}

#[test]
fn appearance_blocker_keeps_fallback_visible_and_selected_rig_hidden() {
    let mut app = App::new();
    app.add_systems(Update, swap_tutorial_player_fallback_on_ready);
    let fallback = app.world_mut().spawn(Visibility::Inherited).id();
    let rig = app.world_mut().spawn_empty().id();
    let mut status = TutorialSelectedPlayerRigStatus::Loading;
    let mut issues = TutorialPlayerRigIssueQueue::default();
    block_tutorial_player_appearance(
        rig,
        &mut status,
        &mut issues,
        "selected-player texture missing.png failed to load".to_owned(),
    );
    assert!(matches!(
        issues.take_all().pop_front(),
        Some(TutorialPlayerRigIssue::AppearanceBlocked {
            rig_root,
            reason
        }) if rig_root == rig && reason.contains("texture")
    ));
    app.world_mut().entity_mut(rig).insert((
        status,
        Visibility::Hidden,
        TutorialSelectedPlayerRigActive,
        TutorialPlayerFallbackVisual {
            entity: Some(fallback),
        },
    ));

    app.update();

    assert!(app.world().get_entity(fallback).is_ok());
    assert_eq!(
        app.world().get::<Visibility>(rig),
        Some(&Visibility::Hidden)
    );
    assert!(
        app.world()
            .get::<TutorialPlayerFallbackReplaced>(rig)
            .is_none()
    );
}

#[test]
fn ready_gate_reveals_selected_rig_and_replaces_fallback_atomically() {
    let mut app = App::new();
    app.add_systems(Update, swap_tutorial_player_fallback_on_ready);
    let fallback = app.world_mut().spawn(Visibility::Inherited).id();
    let rig = app
        .world_mut()
        .spawn((
            TutorialSelectedPlayerRigStatus::Ready,
            Visibility::Hidden,
            TutorialSelectedPlayerRigActive,
            TutorialPlayerFallbackVisual {
                entity: Some(fallback),
            },
        ))
        .id();

    app.update();

    assert!(app.world().get_entity(fallback).is_err());
    assert_eq!(
        app.world().get::<Visibility>(rig),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        app.world().get::<TutorialPlayerFallbackReplaced>(rig),
        Some(&TutorialPlayerFallbackReplaced {
            entity: Some(fallback)
        })
    );
}

#[test]
fn appearance_attachment_binds_to_exact_socket_or_blocks_the_hidden_rig() {
    let mut app = App::new();
    app.init_resource::<TutorialPlayerRigIssueQueue>()
        .add_systems(Update, bind_tutorial_player_appearance_attachments);
    let socket_path = player_attachment_socket_full_path(
        PlayerRigGender::Male,
        LegacyPlayerAttachmentSlot::Hat,
    );
    let socket = app.world_mut().spawn(Transform::default()).id();
    let rig = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(socket).insert(ChildOf(rig));
    app.world_mut().entity_mut(rig).insert((
        NativePlayerRigStatus::ReadyAnimated {
            actor_bones: 1,
            parts: 5,
            skin_palettes: 5,
            skinned_surfaces: 5,
        },
        NativePlayerRigBones::new(vec![crate::player_shared_rig::NativePlayerRigBoneEntity {
            actor_bone_index: 0,
            true_name: "Bip01 helmet01".to_owned(),
            full_path: socket_path.clone(),
            entity: socket,
        }]),
        TutorialSelectedPlayerRigStatus::Loading,
    ));
    let attachment = app
        .world_mut()
        .spawn((
            Transform::default(),
            ChildOf(rig),
            PendingTutorialPlayerAppearanceAttachment {
                rig_root: rig,
                socket_full_path: socket_path,
                socket_local_scale_override: Some(Vec3::ONE),
            },
        ))
        .id();

    app.update();

    assert_eq!(
        app.world().get::<ChildOf>(attachment).map(ChildOf::parent),
        Some(socket)
    );
    assert!(
        app.world()
            .get::<PendingTutorialPlayerAppearanceAttachment>(attachment)
            .is_none()
    );
    assert_eq!(
        app.world().get::<TutorialSelectedPlayerRigStatus>(rig),
        Some(&TutorialSelectedPlayerRigStatus::Loading)
    );

    let missing_rig = app
        .world_mut()
        .spawn((
            NativePlayerRigStatus::ReadyAnimated {
                actor_bones: 0,
                parts: 5,
                skin_palettes: 5,
                skinned_surfaces: 5,
            },
            NativePlayerRigBones::new(Vec::new()),
            TutorialSelectedPlayerRigStatus::Loading,
        ))
        .id();
    app.world_mut().spawn((
        Transform::default(),
        ChildOf(missing_rig),
        PendingTutorialPlayerAppearanceAttachment {
            rig_root: missing_rig,
            socket_full_path: "m/missing".to_owned(),
            socket_local_scale_override: None,
        },
    ));

    app.update();

    assert!(matches!(
        app.world()
            .get::<TutorialSelectedPlayerRigStatus>(missing_rig),
        Some(TutorialSelectedPlayerRigStatus::Blocked(reason))
            if reason.contains("exact socket")
    ));
}
