use super::*;

#[must_use]
pub(super) const fn tutorial_player_appearance_attachment_slot(
    kind: NativePlayerPartKind,
) -> Option<LegacyPlayerAttachmentSlot> {
    match kind {
        NativePlayerPartKind::Hat => Some(LegacyPlayerAttachmentSlot::Hat),
        NativePlayerPartKind::Glasses => Some(LegacyPlayerAttachmentSlot::Glasses),
        NativePlayerPartKind::Back => Some(LegacyPlayerAttachmentSlot::Back),
        NativePlayerPartKind::Weapon
        | NativePlayerPartKind::Face
        | NativePlayerPartKind::Hair
        | NativePlayerPartKind::Shirt
        | NativePlayerPartKind::Pants
        | NativePlayerPartKind::Shoes
        | NativePlayerPartKind::Vehicle => None,
    }
}

/// Instantiate at the socket's world pose, then parent with worldPositionStays.
/// Unlike ordinary equipment, Skyway keeps the prefab's unit world scale at
/// admission. In particular, the Broomstick bone's ~0.365 scale must not shrink
/// the model or its internal animated offsets. Retain the resulting local TRS
/// afterwards so subsequent socket animation still drives the attachment.
pub(super) fn skyway_attachment_transform(socket_world: &GlobalTransform) -> Option<Transform> {
    let (scale, rotation, translation) = socket_world.to_scale_rotation_translation();
    if !scale.is_finite() || scale.abs().min_element() <= f32::EPSILON {
        return None;
    }
    let world = GlobalTransform::from(Transform {
        translation,
        rotation: rotation * standard_player_attachment_placement().item_local.rotation,
        scale: Vec3::ONE,
    });
    Some(world.reparented_to(socket_world))
}

pub(super) fn owning_skyway_attachment<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    attachments: &'a Query<&TutorialSkywayAttachment>,
) -> Option<&'a TutorialSkywayAttachment> {
    loop {
        if let Ok(attachment) = attachments.get(entity) {
            return Some(attachment);
        }
        entity = parents.get(entity).ok()?.parent();
    }
}

/// Exact `ActorSkinCombiner.MakeAvatar` sampling:
/// height is reversed across four intervals, body shape is forward across two.
#[must_use]
pub(super) fn legacy_body_shape_normalized_times(height_selector: i8, body_selector: i8) -> (f32, f32) {
    (
        1.0 - f32::from(height_selector.clamp(0, 4)) / 4.0,
        f32::from(body_selector.clamp(0, 2)) / 2.0,
    )
}

pub(super) fn switch_tutorial_base_layer_mask(
    adapter: &mut TutorialPlayerAnimationAdapter,
    player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    masked: bool,
) -> bool {
    let target = if masked {
        adapter.upper_masked_nodes.get(&adapter.active_clip)
    } else {
        adapter.nodes.get(&adapter.active_clip)
    }
    .copied();
    let Some(target) = target else {
        return false;
    };
    if target == adapter.active_node {
        return true;
    }
    let Some(active) = player.animation(adapter.active_node) else {
        return false;
    };
    let seek_time = active.seek_time();
    let repeat = active.repeat_mode();
    let speed = active.speed();
    let paused = active.is_paused();
    // The replacement inherits a hit-stop pause below; the outgoing twin must
    // still be released to its transition instead of staying blended.
    release_paused_tutorial_main_animation(player, transitions, target);
    let replacement = transitions
        .play(player, target, Duration::ZERO)
        .set_seek_time(seek_time)
        .set_repeat(repeat)
        .set_speed(speed);
    if paused {
        replacement.pause();
    } else {
        // Bevy's `start`/`replay` deliberately preserves the old paused bit.
        // Unity's legacy CrossFade enables the destination state, so a
        // previously used normal/masked node must be resumed explicitly.
        replacement.resume();
    }
    adapter.active_node = target;
    true
}

pub(super) fn switch_tutorial_body_shape_mask(
    adapter: &mut TutorialPlayerAnimationAdapter,
    player: &mut AnimationPlayer,
    masked: bool,
) -> bool {
    if adapter.body_shape.upper_masked == masked {
        return true;
    }
    let (source, target) = if masked {
        (
            adapter.body_shape.normal_nodes,
            adapter.body_shape.upper_masked_nodes,
        )
    } else {
        (
            adapter.body_shape.upper_masked_nodes,
            adapter.body_shape.normal_nodes,
        )
    };
    if source.iter().any(|node| player.animation(*node).is_none()) {
        return false;
    }

    for (source, target) in source.into_iter().zip(target) {
        let active = player
            .animation(source)
            .expect("all body-shape nodes were validated as active");
        let seek_time = active.seek_time();
        let repeat = active.repeat_mode();
        let speed = active.speed();
        let weight = active.weight();
        let paused = active.is_paused();
        player.stop(source);
        let replacement = player
            .start(target)
            .set_seek_time(seek_time)
            .set_repeat(repeat)
            .set_speed(speed)
            .set_weight(weight);
        if paused {
            replacement.pause();
        } else {
            replacement.resume();
        }
    }
    adapter.body_shape.upper_masked = masked;
    true
}

/// Exact `cnAvatarAnimation.UpdateTurn` target table. The source samples
/// additive turn clips at normalized time 1 for a pure strafe, 0.5 for a
/// diagonal and 0 for forward/back/idle.
#[must_use]
pub(super) const fn legacy_directional_turn_target(direction_key: u8) -> (f32, f32) {
    match direction_key {
        7 => (1.0, 0.0),
        3 => (0.0, 1.0),
        8 | 4 => (0.5, 0.0),
        2 | 6 => (0.0, 0.5),
        _ => (0.0, 0.0),
    }
}

pub(super) fn move_towards(current: f32, target: f32, maximum_delta: f32) -> f32 {
    if current < target {
        (current + maximum_delta).min(target)
    } else {
        (current - maximum_delta).max(target)
    }
}

/// Converts a linear Unity layer cross-fade into the relative weight used by
/// Bevy's normalized blend node: r / (1 + r) = alpha.
pub(super) fn normalized_upper_layer_weight(alpha: f32) -> f32 {
    let alpha = alpha.clamp(0.0, 0.999_999);
    alpha / (1.0 - alpha)
}

pub(super) fn tutorial_upper_layer_alpha(
    elapsed_seconds: f32,
    blend_seconds: f32,
    fading_out: bool,
) -> (f32, bool) {
    let progress = if blend_seconds <= f32::EPSILON {
        1.0
    } else {
        (elapsed_seconds / blend_seconds).clamp(0.0, 1.0)
    };
    (
        if fading_out { 1.0 - progress } else { progress },
        fading_out && progress >= 1.0,
    )
}

/// Unity's legacy `Animation.Play`/`CrossFade` always enables an animation
/// state. Bevy's `ActiveAnimation::replay`, in contrast, preserves `paused`,
/// speed and repeat state from an earlier use of the same graph node. Report a
/// removed clamp *or* loop node so the state bridge can complete/recover it
/// instead of leaving the last evaluated skeleton pose forever.
pub(super) fn keep_tutorial_gameplay_playback_alive(
    player: &mut AnimationPlayer,
    node: AnimationNodeIndex,
    clip: TutorialPlayerClip,
) -> bool {
    let Some(active) = player.animation_mut(node) else {
        return false;
    };
    if clip.playback() == TutorialPlayerClipPlayback::Loop {
        active.set_repeat(RepeatAnimation::Forever).resume();
    }
    if !active.speed().is_finite() || active.speed().abs() <= f32::EPSILON {
        active.set_speed(1.0);
    }
    true
}

/// Once an armed clamp was successfully applied, a missing Bevy node means it
/// was stopped/expired and must be treated as complete. Requiring a still-live
/// node here can strand the centralized state in attack/jump/landing forever.
pub(super) fn tutorial_clamp_finished_or_removed(
    player: &AnimationPlayer,
    node: Option<AnimationNodeIndex>,
    gender: PlayerRigGender,
    clip: TutorialPlayerClip,
) -> bool {
    node.and_then(|node| player.animation(node))
        .is_none_or(|active| {
            tutorial_player_clip_end_event_seconds(gender, clip)
                .is_some_and(|seconds| active.seek_time() >= seconds)
                || active.is_finished()
        })
}

pub(super) fn bind_tutorial_player_appearance_attachments(
    mut commands: Commands,
    mut issues: ResMut<TutorialPlayerRigIssueQueue>,
    bones: Query<&NativePlayerRigBones>,
    mut transforms: Query<&mut Transform>,
    pending: Query<(Entity, &PendingTutorialPlayerAppearanceAttachment)>,
    mut rigs: Query<(&NativePlayerRigStatus, &mut TutorialSelectedPlayerRigStatus)>,
) {
    for (attachment, pending) in &pending {
        let Ok((native_status, mut status)) = rigs.get_mut(pending.rig_root) else {
            continue;
        };
        if !matches!(*status, TutorialSelectedPlayerRigStatus::Loading) {
            continue;
        }
        let Ok(bones) = bones.get(pending.rig_root) else {
            if native_status.is_ready() {
                block_tutorial_player_appearance(
                    pending.rig_root,
                    &mut status,
                    &mut issues,
                    "ready selected-player rig has no resolved exact bone map".to_owned(),
                );
            }
            continue;
        };
        let Some(socket) = bones.by_full_path(&pending.socket_full_path) else {
            block_tutorial_player_appearance(
                pending.rig_root,
                &mut status,
                &mut issues,
                format!(
                    "selected-player appearance has no exact socket {:?}",
                    pending.socket_full_path
                ),
            );
            continue;
        };
        if let Some(scale) = pending.socket_local_scale_override
            && let Ok(mut transform) = transforms.get_mut(socket)
        {
            transform.scale = scale;
        }
        commands
            .entity(attachment)
            .queue_silenced(move |mut entity: EntityWorldMut| {
                entity.insert(ChildOf(socket));
                entity.remove::<PendingTutorialPlayerAppearanceAttachment>();
            });
    }
}

pub(super) fn bind_tutorial_player_weapon_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    children: Query<&Children>,
    weapons: Query<(
        Entity,
        &TutorialPlayerWeaponAttachment,
        &TutorialPlayerWeaponLook,
    )>,
    metadata: Query<&PendingLegacyModelMaterial>,
    surfaces: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        Without<TutorialPlayerWeaponMaterialBound>,
    >,
    mut issues: ResMut<TutorialPlayerRigIssueQueue>,
) {
    if weapons.is_empty() {
        return;
    }
    for (root, weapon, look) in &weapons {
        for entity in children.iter_descendants(root) {
            let Ok((entity, handle, own_metadata, companion)) = surfaces.get(entity) else {
                continue;
            };
            let Some(pending) = own_metadata
                .or_else(|| companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok()))
            else {
                continue;
            };
            let Some(source) = materials.get(&handle.0) else {
                continue;
            };
            // GLB materials can be shared by different item skins. Never mutate
            // the source asset when installing this equipped item's texture.
            let mut material = source.clone();
            match bind_native_player_look_material(
                &asset_server,
                pending,
                &mut material,
                &look.0,
                &look.0.parts[0],
                companion.is_some(),
            ) {
                Ok(binding) => {
                    commands.entity(entity).insert((
                        MeshMaterial3d(materials.add(material)),
                        TutorialPlayerWeaponMaterialBound {
                            _binding: Some(binding),
                        },
                    ));
                }
                Err(reason) => {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::EquipmentBlocked {
                            rig_root: weapon.rig_root,
                            item_id: weapon.item_id,
                            reason,
                        });
                    commands.entity(entity).insert((
                        Visibility::Hidden,
                        TutorialPlayerWeaponMaterialBound { _binding: None },
                    ));
                }
            }
        }
    }
}

pub(super) fn bind_tutorial_player_appearance(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut issues: ResMut<TutorialPlayerRigIssueQueue>,
    parents: Query<&ChildOf>,
    parts: Query<&TutorialPlayerAppearancePart>,
    metadata: Query<&PendingLegacyModelMaterial>,
    mut surfaces: Query<
        (
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        Without<TutorialPlayerAppearanceMaterialBound>,
    >,
    mut rigs: Query<(
        Entity,
        &TutorialSelectedPlayerRig,
        &NativePlayerRigStatus,
        &mut TutorialSelectedPlayerRigStatus,
    )>,
) {
    'rig: for (rig_root, selected, native_status, mut status) in &mut rigs {
        if !matches!(*status, TutorialSelectedPlayerRigStatus::Loading) || !native_status.is_ready()
        {
            continue;
        }
        for (entity, mut handle, own_metadata, companion) in &mut surfaces {
            if !is_descendant_of(entity, rig_root, &parents) {
                continue;
            }
            let Some(part_scene) =
                ancestor_tutorial_player_appearance_part(entity, rig_root, &parents, &parts)
            else {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!("appearance surface {entity:?} has no exact part ancestor"),
                );
                continue 'rig;
            };
            let matching_parts = selected
                .look
                .parts
                .iter()
                .filter(|part| {
                    part.kind == part_scene.kind && part.exact_route == part_scene.exact_route
                })
                .collect::<Vec<_>>();
            let [part_look] = matching_parts.as_slice() else {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!(
                        "appearance surface resolves exact route {:?} {} times in selected look",
                        part_scene.exact_route,
                        matching_parts.len()
                    ),
                );
                continue 'rig;
            };
            if part_look.glb != part_scene.glb {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!(
                        "appearance route {:?} loaded GLB {:?}; selected look requires {:?}",
                        part_scene.exact_route, part_scene.glb, part_look.glb
                    ),
                );
                continue 'rig;
            }
            let Some(pending) = own_metadata
                .or_else(|| companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok()))
            else {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!("appearance surface {entity:?} lacks exact legacy material metadata"),
                );
                continue 'rig;
            };
            crate::legacy_model_material::make_legacy_material_unique(
                &mut handle.0,
                &mut materials,
            );
            let Some(mut material) = materials.get_mut(&handle.0) else {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!("appearance surface {entity:?} lost its instance material"),
                );
                continue 'rig;
            };
            let binding = match bind_native_player_look_material(
                &asset_server,
                pending,
                &mut material,
                &selected.look,
                part_look,
                companion.is_some(),
            ) {
                Ok(binding) => binding,
                Err(reason) => {
                    block_tutorial_player_appearance(rig_root, &mut status, &mut issues, reason);
                    continue 'rig;
                }
            };
            if binding.hide_surface {
                commands.entity(entity).insert(Visibility::Hidden);
            }
            commands
                .entity(entity)
                .insert(TutorialPlayerAppearanceMaterialBound {
                    rig_root,
                    exact_route: part_scene.exact_route.clone(),
                    binding,
                });
        }
    }
}

pub(super) fn swap_tutorial_player_fallback_on_ready(
    mut commands: Commands,
    mut rigs: Query<
        (
            Entity,
            &TutorialSelectedPlayerRigStatus,
            &mut Visibility,
            &mut TutorialPlayerFallbackVisual,
        ),
        (
            With<TutorialSelectedPlayerRigActive>,
            Without<TutorialSelectedPlayerRigCandidate>,
            Without<TutorialPlayerFallbackReplaced>,
        ),
    >,
) {
    for (rig_root, status, mut visibility, mut fallback) in &mut rigs {
        if !matches!(status, TutorialSelectedPlayerRigStatus::Ready) {
            continue;
        }
        *visibility = Visibility::Inherited;
        let replaced = fallback.entity.take();
        if let Some(fallback) = replaced {
            commands.entity(fallback).despawn();
        }
        commands
            .entity(rig_root)
            .insert(TutorialPlayerFallbackReplaced { entity: replaced });
    }
}

pub(super) fn block_tutorial_player_appearance(
    rig_root: Entity,
    status: &mut TutorialSelectedPlayerRigStatus,
    issues: &mut TutorialPlayerRigIssueQueue,
    reason: String,
) {
    if matches!(status, TutorialSelectedPlayerRigStatus::Blocked(_)) {
        return;
    }
    *status = TutorialSelectedPlayerRigStatus::Blocked(reason.clone());
    issues
        .pending
        .push_back(TutorialPlayerRigIssue::AppearanceBlocked { rig_root, reason });
}

pub(super) fn ancestor_tutorial_player_appearance_part<'a>(
    mut entity: Entity,
    rig_root: Entity,
    parents: &Query<&ChildOf>,
    parts: &'a Query<&TutorialPlayerAppearancePart>,
) -> Option<&'a TutorialPlayerAppearancePart> {
    loop {
        if let Ok(part) = parts.get(entity)
            && part.rig_root == rig_root
        {
            return Some(part);
        }
        let parent = parents.get(entity).ok()?.parent();
        if parent == rig_root {
            return parts
                .get(parent)
                .ok()
                .filter(|part| part.rig_root == rig_root);
        }
        entity = parent;
    }
}

pub(super) fn pending_legacy_visual_completion(
    semantic_clip: LegacyVisualClip,
    animation_clip: TutorialPlayerClip,
) -> Option<PendingLegacyVisualCompletion> {
    (animation_clip.playback() == TutorialPlayerClipPlayback::Clamp).then_some(
        PendingLegacyVisualCompletion {
            semantic_clip,
            animation_clip,
            armed: false,
        },
    )
}

/// Connects the normal `cnAvatarAnimation` request stream to the selected
/// player's concrete Bevy graph.  The old implementation produced these
/// commands but had no consumer, so firing never changed the rendered pose.
pub(super) fn bridge_legacy_avatar_visual_requests(
    vehicle: Res<PersonalVehiclePresentation>,
    mut visuals: ResMut<LegacyVisualRequestQueue>,
    mut queue: ResMut<TutorialPlayerPresentationCommandQueue>,
    mut continuation: ResMut<PlayerEmoteContinuation>,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
    mut rigs: Query<
        (
            &TutorialSelectedPlayerRig,
            &mut TutorialPlayerAnimationAdapter,
            &TutorialSelectedPlayerRigStatus,
        ),
        With<TutorialSelectedPlayerRigActive>,
    >,
) {
    if visuals.is_empty() {
        return;
    }
    let Ok((selected, mut adapter, status)) = rigs.single_mut() else {
        return;
    };
    if !matches!(status, TutorialSelectedPlayerRigStatus::Ready) {
        return;
    }

    let requests = visuals.take_all();
    let death_override = requests.iter().any(|request| {
        request.actor == selected.controller_root
            && matches!(
                &request.command,
                LegacyVisualCommand::CrossFade {
                    requested_clip: LegacyVisualClip::Die | LegacyVisualClip::Death,
                    ..
                }
            )
    });
    if death_override {
        // Clean `AvatarDead` begins with EndEmote and cannot be masked by a
        // queued choreography pose from the previous frame.
        adapter.emote_state = false;
        adapter.hand_attachment_hidden = false;
        queue.cancel_pending_direct_pose_overrides();
        queue.cancel_pending_upper_layers();
    }
    // Owner-requested extension: an accepted shot interrupts a player emote
    // immediately. Rejected input produces no attack visual command. Keep
    // scripted spawn/stand-up poses protected by the ordinary guard below.
    let attack_override = requests.iter().any(|request| {
        request.actor == selected.controller_root
            && attack_interrupts_player_emote(&request.command)
    });
    if attack_override {
        queue.cancel_pending_avatar_emotes();
        if adapter.emote_state && adapter.active_clip.is_avatar_emote_code_clip() {
            adapter.emote_state = false;
            adapter.hand_attachment_hidden = false;
            adapter.emote_cursor = PlayerEmoteCursor::default();
        }
    }
    if death_override || attack_override {
        continuation
            .pending
            .retain(|(owner, _)| *owner != selected.controller_root);
    }
    // Direct scene commands and uninterruptible spawn/stand-up emotes own the
    // visible pose. Retrobution rejects ordinary AvatarStand/AvatarMove while
    // those states are active; it does not replay an accumulated backlog once
    // the scene ends. Drain such requests now so stale run/stand clips cannot
    // interrupt a later rifle attack.
    if adapter.emote_state || queue.has_pending_direct_pose_override() {
        return;
    }
    // Equip and animation events can arrive together in the tutorial. Resolve
    // the animation as armed even though the attachment consumer has not run
    // yet; both commands are drained later in this same renderer frame.
    let weapon_item_id = queue
        .pending_weapon_item_id()
        .or(selected.attached_weapon_item_id);
    let weapon_profile =
        weapon_item_id.and_then(|item_id| weapon_catalog.profile_for_item(item_id));
    let mut base_request = None;
    let mut upper_request = None;
    let mut delays = Vec::new();
    for request in requests {
        if request.actor != selected.controller_root {
            continue;
        }
        let (requested_clip, blend_seconds, layer) = match request.command {
            LegacyVisualCommand::CrossFade {
                requested_clip,
                blend_seconds,
                layer,
                ..
            } => (requested_clip, blend_seconds, layer),
            LegacyVisualCommand::DelayCurrent { seconds } => {
                delays.push(Duration::from_secs_f32(seconds.max(0.0)));
                continue;
            }
        };
        if death_override
            && !matches!(
                requested_clip,
                LegacyVisualClip::Die | LegacyVisualClip::Death
            )
        {
            continue;
        }
        let Some(animation_clip) = personal_vehicle::vehicle_clip(vehicle.family, requested_clip)
            .or_else(|| {
                legacy_visual_tutorial_clip(
                    requested_clip,
                    weapon_profile,
                    selected.tutorial_stand_semantics,
                )
            })
        else {
            continue;
        };
        let coalesced = CoalescedLegacyLayerAnimation {
            semantic_clip: requested_clip,
            animation_clip,
            blend_seconds,
        };
        // More than one base request can legitimately be produced in one
        // state-machine tick (for example attack completion -> ready while
        // movement simultaneously selects run). Only the final base state is
        // render-authoritative; playing every intermediate clip in the same
        // Bevy frame creates a visible pose spike.
        coalesce_legacy_layer_animation(&mut base_request, &mut upper_request, layer, coalesced);
    }
    if let Some(request) = base_request {
        queue_coalesced_legacy_layer_animation(
            &mut queue,
            &mut adapter.pending_base_completion,
            selected.gender,
            request,
        );
    }
    if let Some(request) = upper_request {
        queue_coalesced_legacy_layer_animation(
            &mut queue,
            &mut adapter.pending_upper_completion,
            selected.gender,
            request,
        );
    }
    for delay in delays {
        queue.push_animation_delay(delay);
    }
}

pub(super) fn attack_interrupts_player_emote(command: &LegacyVisualCommand) -> bool {
    matches!(
        command,
        LegacyVisualCommand::CrossFade {
            requested_clip: LegacyVisualClip::AttackFull(_)
                | LegacyVisualClip::AttackUpper(_)
                | LegacyVisualClip::Stun
                | LegacyVisualClip::Dash
                | LegacyVisualClip::DashAir,
            ..
        }
    )
}

pub(super) fn tutorial_unarmed_attack_masks_target(
    target: &AnimationTargetId,
    upper_targets: &BTreeSet<AnimationTargetId>,
    attack_body_chain: &BTreeSet<AnimationTargetId>,
) -> bool {
    !upper_targets.contains(target) && !attack_body_chain.contains(target)
}

pub(super) fn tutorial_emote_holds_locomotion(
    emote_state: bool,
    interrupt_requested: bool,
    active_clip_finished: bool,
    startup_loading_barrier: bool,
) -> bool {
    emote_state && !interrupt_requested && (!active_clip_finished || startup_loading_barrier)
}

pub(super) fn tutorial_startup_emote_loading_barrier(
    tutorial_stand_semantics: bool,
    collision_pending: bool,
    active_clip: TutorialPlayerClip,
) -> bool {
    tutorial_stand_semantics
        && collision_pending
        && matches!(active_clip, TutorialPlayerClip::Staying)
}

pub(super) fn tutorial_emote_interrupt_requested(direction_key: u8, jumping: bool) -> bool {
    direction_key != 0 || jumping
}
