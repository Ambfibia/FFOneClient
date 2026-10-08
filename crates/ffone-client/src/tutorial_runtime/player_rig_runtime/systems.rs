use super::*;

pub(super) const LEGACY_DIRECTIONAL_TURN_TICK: f32 = 2.0;

pub(super) fn sync_tutorial_skyway_attachment(
    mut commands: Commands,
    presentation: Res<TutorialSkywayPresentation>,
    asset_server: Res<AssetServer>,
    rigs: Query<
        (Entity, &TutorialSelectedPlayerRig, &NativePlayerRigBones),
        (
            With<TutorialSelectedPlayerRigActive>,
            Without<TutorialSelectedPlayerRigCandidate>,
        ),
    >,
    attachments: Query<(Entity, &TutorialSkywayAttachment)>,
    transforms: Query<&GlobalTransform>,
) {
    for (entity, attachment) in &attachments {
        let owns_active_rig = rigs
            .iter()
            .any(|(rig_root, _, _)| rig_root == attachment.rig_root);
        if !presentation.active || !owns_active_rig {
            commands.entity(entity).despawn();
        }
    }
    if !presentation.active {
        return;
    }
    let Ok((rig_root, rig, bones)) = rigs.single() else {
        return;
    };
    if attachments
        .iter()
        .any(|(_, attachment)| attachment.rig_root == rig_root)
    {
        return;
    }
    let Some(socket) = bones.by_full_path(&skyway_socket_full_path(rig.gender)) else {
        return;
    };
    let Ok(socket_world) = transforms.get(socket) else {
        return;
    };
    let Some(placement) = skyway_attachment_transform(socket_world) else {
        return;
    };
    let gltf: Handle<Gltf> = asset_server.load(SKYWAY_MODEL_PATH);
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(SKYWAY_MODEL_PATH));
    commands.spawn((
        Name::new("Monkey Skyway attachment"),
        TutorialSkywayAttachment { rig_root, gltf },
        ChildOf(socket),
        WorldAssetRoot(scene),
        placement,
        Visibility::Inherited,
        rig.render_layers.clone(),
    ));
}

pub(super) fn sync_tutorial_zipline_attachment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rigs: Query<
        (Entity, &TutorialSelectedPlayerRig, &NativePlayerRigBones),
        (
            With<TutorialSelectedPlayerRigActive>,
            Without<TutorialSelectedPlayerRigCandidate>,
        ),
    >,
    presentations: Query<&LegacyAvatarPresentationContext>,
    attachments: Query<(Entity, &TutorialZiplineAttachment)>,
    mut transforms: Query<&mut Transform>,
) {
    let active = rigs.iter().find(|(_, rig, _)| {
        presentations
            .get(rig.controller_root)
            .is_ok_and(|context| context.traversal == LegacyAvatarTraversalPresentation::Zipline)
    });
    for (entity, attachment) in &attachments {
        if active
            .as_ref()
            .is_none_or(|(root, _, _)| *root != attachment.rig_root)
        {
            commands.entity(entity).despawn();
        }
    }
    let Some((rig_root, rig, bones)) = active else {
        return;
    };
    if attachments
        .iter()
        .any(|(_, attachment)| attachment.rig_root == rig_root)
    {
        return;
    }
    let path = player_attachment_socket_full_path(rig.gender, LegacyPlayerAttachmentSlot::Zipline);
    let Some(socket) = bones.by_full_path(&path) else {
        return;
    };
    let Ok(mut socket_transform) = transforms.get_mut(socket) else {
        return;
    };
    let placement = standard_player_attachment_placement();
    socket_transform.scale = placement.socket_local_scale_override.unwrap_or(Vec3::ONE);
    commands.spawn((
        Name::new("Zipline trolley attachment"),
        TutorialZiplineAttachment { rig_root },
        ChildOf(socket),
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(ZIPLINE_MODEL_PATH))),
        placement.item_local,
        Visibility::Inherited,
        rig.render_layers.clone(),
    ));
}

pub(super) fn update_tutorial_directional_turn(
    adapter: &mut TutorialPlayerAnimationAdapter,
    player: &mut AnimationPlayer,
    direction_key: u8,
    rifle_equipped: bool,
    scooter_mounted: bool,
    delta_seconds: f32,
) {
    let (target_left, target_right) = legacy_directional_turn_target(direction_key);
    let maximum_delta = delta_seconds.max(0.0) * LEGACY_DIRECTIONAL_TURN_TICK;
    adapter.directional_turn.current_left = move_towards(
        adapter.directional_turn.current_left,
        target_left,
        maximum_delta,
    );
    adapter.directional_turn.current_right = move_towards(
        adapter.directional_turn.current_right,
        target_right,
        maximum_delta,
    );
    let nodes = if adapter.body_shape.upper_masked {
        adapter.directional_turn.upper_masked_nodes
    } else {
        adapter.directional_turn.normal_nodes
    };
    let (unarmed_left, unarmed_right, rifle_left, rifle_right) = if rifle_equipped {
        (
            0.0,
            0.0,
            adapter.directional_turn.current_left,
            adapter.directional_turn.current_right,
        )
    } else {
        (
            adapter.directional_turn.current_left,
            adapter.directional_turn.current_right,
            0.0,
            0.0,
        )
    };
    let normalized_times = if scooter_mounted {
        [
            0.0,
            0.0,
            0.0,
            0.0,
            adapter.directional_turn.current_left,
            adapter.directional_turn.current_right,
        ]
    } else {
        [
            unarmed_left,
            unarmed_right,
            rifle_left,
            rifle_right,
            0.0,
            0.0,
        ]
    };
    for ((node, duration), normalized_time) in nodes
        .into_iter()
        .zip(adapter.directional_turn.durations)
        .zip(normalized_times)
    {
        if let Some(active) = player.animation_mut(node) {
            active.set_seek_time(duration * normalized_time);
        }
    }
}

pub(super) fn advance_tutorial_upper_layer_playback(
    upper: &mut TutorialUpperLayerPlayback,
    player: &mut AnimationPlayer,
    delta_seconds: f32,
) -> bool {
    upper.elapsed_seconds += delta_seconds.max(0.0);
    let (alpha, fade_out_complete) =
        tutorial_upper_layer_alpha(upper.elapsed_seconds, upper.blend_seconds, upper.fading_out);
    if let Some(active) = player.animation_mut(upper.node) {
        active.set_weight(normalized_upper_layer_weight(alpha));
    }
    fade_out_complete
}

pub(super) fn sync_tutorial_player_weapon_visibility(
    vehicle: Res<PersonalVehiclePresentation>,
    rigs: Query<(&TutorialPlayerAnimationAdapter, &TutorialSelectedPlayerRig), With<TutorialSelectedPlayerRigActive>>,
    controllers: Query<&crate::movement::LegacyPlayerController>,
    mut attachments: Query<(&TutorialPlayerWeaponAttachment, &mut Visibility)>,
) {
    for (attachment, mut visibility) in &mut attachments {
        let Ok((adapter, rig)) = rigs.get(attachment.rig_root) else {
            continue;
        };
        // Traversal owns HidePistol even if equipment is refreshed mid-ride.
        let traversal_hidden = controllers.get(rig.controller_root).is_ok_and(|controller|
            !controller.movement_enabled || controller.launcher_active());
        let desired = if traversal_hidden || adapter.hand_attachment_hidden || vehicle.item_id.is_some() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
}
