use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_tutorial_player_presentation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    character_data: Res<CharacterCreationDataResource>,
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    mut queue: ResMut<TutorialPlayerPresentationCommandQueue>,
    mut issues: ResMut<TutorialPlayerRigIssueQueue>,
    mut rigs: Query<
        (
            Entity,
            &mut TutorialSelectedPlayerRig,
            &mut TutorialPlayerAnimationAdapter,
            &TutorialSelectedPlayerRigStatus,
        ),
        With<TutorialSelectedPlayerRigActive>,
    >,
    bones: Query<&NativePlayerRigBones>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut controllers: Query<&mut LegacyPlayerController>,
    mut transforms: Query<&mut Transform>,
) {
    let Ok((rig_root, mut selected, mut adapter, status)) = rigs.single_mut() else {
        return;
    };
    if !matches!(status, TutorialSelectedPlayerRigStatus::Ready) {
        return;
    }
    // Unity receives the full-body and layer-101 attack CrossFade calls in the
    // same frame. Resolve the current FIFO batch together so their clip clocks
    // stay synchronized.
    for resolution in selected.consumer.consume_available(&mut queue) {
        match resolution {
            TutorialPlayerPresentationResolution::AnimationContractResolved(resolved) => {
                let is_upper_layer = resolved.request.clip.is_upper_layer();
                let base_masked = adapter
                    .upper_masked_nodes
                    .get(&adapter.active_clip)
                    .is_some_and(|node| *node == adapter.active_node);
                let node = if is_upper_layer || !base_masked {
                    adapter.nodes.get(&resolved.request.clip)
                } else {
                    adapter.upper_masked_nodes.get(&resolved.request.clip)
                }
                .copied();
                let Some(node) = node else {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::AnimationBlocked {
                            rig_root,
                            clip: resolved.request.clip,
                            reason: "prepared graph has no resolved contract node".to_owned(),
                        });
                    continue;
                };
                let Ok((mut player, mut transitions)) = players.get_mut(adapter.animation_player)
                else {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::AnimationBlocked {
                            rig_root,
                            clip: resolved.request.clip,
                            reason: "selected rig AnimationPlayer disappeared".to_owned(),
                        });
                    return;
                };

                if resolved.request.clip == TutorialPlayerClip::WoundUpper {
                    adapter.damage.play(&mut player);
                    // Layer 104 does not change emote ownership, weapon
                    // visibility, base transitions or attack completions.
                    continue;
                }
                if is_upper_layer {
                    let mut initial_alpha = 0.0;
                    if let Some(previous) = adapter.upper_layer.take() {
                        initial_alpha = tutorial_upper_layer_alpha(
                            previous.elapsed_seconds,
                            previous.blend_seconds,
                            previous.fading_out,
                        )
                        .0;
                        player.stop(previous.node);
                        if previous.base_masked || adapter.body_shape.upper_masked {
                            switch_tutorial_composed_pose_mask(
                                &mut adapter,
                                &mut player,
                                &mut transitions,
                                false,
                            );
                        }
                    }
                    let blend_seconds = match resolved.request.dispatch {
                        TutorialPlayerAnimationDispatch::PlayImmediate => 0.0,
                        TutorialPlayerAnimationDispatch::CrossFade(duration) => {
                            duration.as_secs_f32()
                        }
                    };
                    let repeat = match resolved.playback {
                        TutorialPlayerClipPlayback::Loop => RepeatAnimation::Forever,
                        TutorialPlayerClipPlayback::Clamp => RepeatAnimation::Never,
                    };
                    player
                        .start(node)
                        .set_repeat(repeat)
                        .set_weight(if blend_seconds <= f32::EPSILON {
                            1.0
                        } else {
                            normalized_upper_layer_weight(initial_alpha)
                        })
                        .resume();
                    adapter.upper_layer = Some(TutorialUpperLayerPlayback {
                        node,
                        // Retriggering the same firing layer continues from its
                        // current blend weight instead of dropping to zero for
                        // one frame and visibly popping the shoulders/arms.
                        elapsed_seconds: blend_seconds * initial_alpha,
                        blend_seconds,
                        // Keep the base graph continuously active. Weighting
                        // the upper clip in/out is smooth; switching duplicate
                        // masked base nodes at alpha=1 caused a visible pose
                        // discontinuity at both ends of every shot.
                        base_masked: false,
                        fading_out: false,
                    });
                } else {
                    apply_resolved_tutorial_animation(
                        &mut player,
                        &mut transitions,
                        node,
                        resolved,
                    );
                    adapter.active_node = node;
                    adapter.active_clip = resolved.request.clip;
                }
                let pending = if is_upper_layer {
                    &mut adapter.pending_upper_completion
                } else {
                    &mut adapter.pending_base_completion
                };
                if let Some(pending) = pending.as_mut()
                    && pending.animation_clip == resolved.request.clip
                {
                    pending.armed = true;
                } else if pending.is_some() {
                    *pending = None;
                }
                adapter.emote_state = resolved.request.sets_emote_state;
                adapter.emote_cursor = PlayerEmoteCursor::default();
                adapter.hand_attachment_hidden = resolved.request.hide_hand_attachment;
                if resolved.request.reset_move_direction
                    && let Ok(mut controller) = controllers.get_mut(selected.controller_root)
                {
                    controller.reset_current_move_direction();
                }
                commands
                    .entity(rig_root)
                    .insert(TutorialPlayerAnimationApplied {
                        rig_root,
                        animation_player: adapter.animation_player,
                        clip: resolved.request.clip,
                        source_path_id: resolved.request.source_path_id,
                        animation_node: node,
                        playback: resolved.playback,
                        dispatch: resolved.request.dispatch,
                    });
            }
            TutorialPlayerPresentationResolution::AnimationBlocked { request, reason } => {
                issues
                    .pending
                    .push_back(TutorialPlayerRigIssue::AnimationBlocked {
                        rig_root,
                        clip: request.clip,
                        reason: reason.to_string(),
                    });
            }
            TutorialPlayerPresentationResolution::AnimationDelay(duration) => {
                let Ok((mut player, _)) = players.get_mut(adapter.animation_player) else {
                    return;
                };
                if let Some(previous) = adapter.delay.take() {
                    let upper_node = adapter.upper_layer.map(|upper| upper.node);
                    resume_tutorial_delayed_playback(
                        &mut player,
                        previous,
                        adapter.active_node,
                        upper_node,
                    );
                }
                let base_node = matches!(
                    adapter.active_clip,
                    TutorialPlayerClip::Attack1
                        | TutorialPlayerClip::StickAttack1
                        | TutorialPlayerClip::PistolAttack1
                        | TutorialPlayerClip::RifleAttack1
                        | TutorialPlayerClip::BombAttack1
                        | TutorialPlayerClip::RocketAttack1
                )
                .then_some(adapter.active_node);
                let upper_node = adapter.upper_layer.map(|upper| upper.node);
                // A zero-length stop has no expiry to resume what it paused.
                if duration.is_zero() || (base_node.is_none() && upper_node.is_none()) {
                    continue;
                }
                for node in [base_node, upper_node].into_iter().flatten() {
                    if let Some(active) = player.animation_mut(node) {
                        active.pause();
                    }
                }
                adapter.delay = Some(TutorialAnimationDelayPlayback {
                    remaining_seconds: duration.as_secs_f32(),
                    base: base_node.is_some(),
                    upper: upper_node.is_some(),
                });
            }
            TutorialPlayerPresentationResolution::EquipmentForwarded(request) => {
                let item_id = request.item.item_id;
                if request.slot != CharacterEquipSlot0104::Hand {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::EquipmentBlocked {
                            rig_root,
                            item_id,
                            reason: "player equipment request is not the exact hand route"
                                .to_owned(),
                        });
                    continue;
                }
                // `OCSlotEntity.SetItemBase` treats every non-positive item ID
                // as empty. Ordinary-world inventory replies use that same
                // sentinel when a weapon is unequipped, so consume it as a
                // real detach instead of trying to resolve weapon model 0.
                if item_id <= 0 {
                    if let Some(previous) = selected.attached_weapon.take() {
                        commands.entity(previous).despawn();
                    }
                    selected.attached_weapon_item_id = None;
                    adapter.hand_attachment_hidden = false;
                    commands
                        .entity(rig_root)
                        .remove::<TutorialPlayerEquipmentAttachmentSpawned>();
                    continue;
                }
                if request.item.item_type != 0 {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::EquipmentBlocked {
                            rig_root,
                            item_id,
                            reason: "player hand item is not a protocol weapon".to_owned(),
                        });
                    continue;
                }
                let part = if let Some(part) = selected.weapon_models.get(&item_id).cloned() {
                    part
                } else {
                    match character_data
                        .0
                        .resolve_weapon_attachment(item_id as u32, selected.gender)
                    {
                        Ok(part) => {
                            selected.weapon_models.insert(item_id, part.clone());
                            part
                        }
                        Err(error) => {
                            issues
                                .pending
                                .push_back(TutorialPlayerRigIssue::EquipmentBlocked {
                                    rig_root,
                                    item_id,
                                    reason: format!(
                                        "weapon item has no validated native attachment: {error}"
                                    ),
                                });
                            continue;
                        }
                    }
                };
                let Ok(bones) = bones.get(rig_root) else {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::EquipmentBlocked {
                            rig_root,
                            item_id,
                            reason: "selected rig has no resolved exact bone map".to_owned(),
                        });
                    continue;
                };
                let socket_full_path = tutorial_player_weapon_socket_full_path(selected.gender);
                let Some(socket) = bones.by_full_path(&socket_full_path) else {
                    issues
                        .pending
                        .push_back(TutorialPlayerRigIssue::EquipmentBlocked {
                            rig_root,
                            item_id,
                            reason: format!(
                                "selected rig has no exact socket {socket_full_path:?}"
                            ),
                        });
                    continue;
                };
                if let Some(previous) = selected.attached_weapon.take() {
                    commands.entity(previous).despawn();
                }
                selected.attached_weapon_item_id = None;
                let placement = standard_player_attachment_placement();
                if let Some(scale) = placement.socket_local_scale_override
                    && let Ok(mut transform) = transforms.get_mut(socket)
                {
                    transform.scale = scale;
                }
                let scene = asset_cache.scene(&asset_server, part.glb.clone());
                let mut weapon_look = selected.look.clone();
                weapon_look.parts = vec![part.clone()];
                let attachment = commands
                    .spawn((
                        Name::new(format!("Tutorial player weapon: {}", part.exact_route)),
                        TutorialPlayerWeaponLook(weapon_look),
                        TutorialPlayerWeaponAttachment {
                            rig_root,
                            item_id,
                            exact_route: part.exact_route.clone(),
                            socket_full_path: socket_full_path.clone(),
                        },
                        ChildOf(socket),
                        WorldAssetRoot(scene),
                        placement.item_local,
                        Visibility::Inherited,
                        selected.render_layers.clone(),
                    ))
                    .id();
                selected.attached_weapon = Some(attachment);
                selected.attached_weapon_item_id = Some(item_id);
                adapter.hand_attachment_hidden = false;
                commands
                    .entity(rig_root)
                    .insert(TutorialPlayerEquipmentAttachmentSpawned {
                        attachment,
                        item_id,
                        exact_route: part.exact_route,
                        socket_full_path,
                    });
            }
        }
    }
}

pub(super) fn resume_tutorial_delayed_playback(
    player: &mut AnimationPlayer,
    delay: TutorialAnimationDelayPlayback,
    active_node: AnimationNodeIndex,
    upper_node: Option<AnimationNodeIndex>,
) {
    let base = delay.base.then_some(active_node);
    let upper = upper_node.filter(|_| delay.upper);
    for node in [base, upper].into_iter().flatten() {
        if let Some(active) = player.animation_mut(node) {
            active.resume();
        }
    }
}
