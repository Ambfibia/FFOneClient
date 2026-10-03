use super::*;

pub(super) fn positive_speed_or_one(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[must_use]
pub(super) const fn network_pc_appearance_attachment_slot(
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

pub(super) fn network_pc_expected_appearance_routes(
    look: &NativePlayerLook,
) -> std::collections::BTreeSet<(NativePlayerPartKind, String)> {
    look.parts
        .iter()
        .filter(|part| part.kind != NativePlayerPartKind::Weapon)
        .map(|part| (part.kind, part.exact_route.clone()))
        .collect()
}

pub(super) fn network_pc_appearance_routes_ready(
    expected_routes: &std::collections::BTreeSet<(NativePlayerPartKind, String)>,
    expected_skinned_parts: usize,
    native_part_count: usize,
    actual_part_count: usize,
    actual_routes: &std::collections::BTreeSet<(NativePlayerPartKind, String)>,
    has_pending_attachment: bool,
) -> bool {
    !has_pending_attachment
        && native_part_count == expected_skinned_parts
        && actual_part_count == expected_routes.len()
        && actual_routes == expected_routes
}

pub(super) fn bind_network_pc_appearance_attachments_0104(
    mut commands: Commands,
    bones: Query<&NativePlayerRigBones>,
    mut transforms: Query<&mut Transform>,
    pending: Query<(Entity, &PendingNetworkPcAppearanceAttachment0104)>,
    mut rigs: Query<(
        &NetworkPcRig0104,
        &NativePlayerRigStatus,
        &mut NetworkPcRigAppearanceStatus0104,
    )>,
) {
    for (attachment, pending) in &pending {
        let Ok((rig, native_status, mut status)) = rigs.get_mut(pending.rig_root) else {
            continue;
        };
        if !matches!(*status, NetworkPcRigAppearanceStatus0104::Loading) {
            continue;
        }
        let Ok(bones) = bones.get(pending.rig_root) else {
            if native_status.is_ready() {
                block_network_pc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    "ready remote-player rig has no resolved exact bone map".to_owned(),
                );
            }
            continue;
        };
        let Some(socket) = bones.by_full_path(&pending.socket_full_path) else {
            block_network_pc_rig(
                &mut commands,
                rig,
                &mut status,
                format!(
                    "remote-player appearance has no exact socket {:?}",
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
                entity.remove::<PendingNetworkPcAppearanceAttachment0104>();
            });
    }
}

pub(super) fn bind_network_pc_weapon_attachment_0104(
    mut commands: Commands,
    bones: Query<&NativePlayerRigBones>,
    mut transforms: Query<&mut Transform>,
    pending: Query<(Entity, &PendingNetworkPcWeaponAttachment0104)>,
) {
    for (attachment, pending) in &pending {
        let Ok(bones) = bones.get(pending.rig_root) else {
            continue;
        };
        let Some(socket) = bones.by_full_path(&pending.socket_full_path) else {
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
                entity.remove::<PendingNetworkPcWeaponAttachment0104>();
            });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_network_pc_appearance_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
    parents: Query<&ChildOf>,
    parts: Query<&NetworkPcAppearancePart0104>,
    metadata: Query<&PendingLegacyModelMaterial>,
    mut surfaces: Query<
        (
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        Without<NetworkPcAppearanceMaterialBound0104>,
    >,
    mut rigs: Query<(
        Entity,
        &NetworkPcRig0104,
        &NativePlayerRigStatus,
        &mut NetworkPcRigAppearanceStatus0104,
    )>,
) {
    let (Some(asset_server), Some(mut materials)) = (asset_server, materials) else {
        return;
    };
    'rigs: for (rig_root, rig, native_status, mut status) in &mut rigs {
        if !matches!(*status, NetworkPcRigAppearanceStatus0104::Loading)
            || !native_status.is_ready()
        {
            continue;
        }
        for (entity, mut handle, own_metadata, companion) in &mut surfaces {
            if !is_descendant_of(entity, rig_root, &parents) {
                continue;
            }
            // The separately owned weapon is also a rig descendant, but has no
            // appearance-part marker. Its packet/animation semantics and the
            // ordinary model-material pipeline therefore remain unchanged.
            let Some(part_scene) =
                ancestor_network_pc_appearance_part(entity, rig_root, &parents, &parts)
            else {
                continue;
            };
            let matching = rig
                .look
                .parts
                .iter()
                .filter(|part| {
                    part.kind == part_scene.kind && part.exact_route == part_scene.exact_route
                })
                .collect::<Vec<_>>();
            let [part_look] = matching.as_slice() else {
                block_network_pc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!(
                        "appearance route {:?} resolves {} times in live player look",
                        part_scene.exact_route,
                        matching.len()
                    ),
                );
                continue 'rigs;
            };
            if part_look.glb != part_scene.glb {
                block_network_pc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!(
                        "appearance route {:?} loaded {:?}, expected {:?}",
                        part_scene.exact_route, part_scene.glb, part_look.glb
                    ),
                );
                continue 'rigs;
            }
            let Some(pending) = own_metadata
                .or_else(|| companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok()))
            else {
                block_network_pc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!("appearance surface {entity:?} has no legacy material metadata"),
                );
                continue 'rigs;
            };
            crate::legacy_model_material::make_legacy_material_unique(
                &mut handle.0,
                &mut materials,
            );
            let Some(mut material) = materials.get_mut(&handle.0) else {
                block_network_pc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!("appearance surface {entity:?} lost its instance material"),
                );
                continue 'rigs;
            };
            let binding = match bind_native_player_look_material(
                &asset_server,
                pending,
                &mut material,
                &rig.look,
                part_look,
                companion.is_some(),
            ) {
                Ok(binding) => binding,
                Err(error) => {
                    block_network_pc_rig(&mut commands, rig, &mut status, error);
                    continue 'rigs;
                }
            };
            if binding.hide_surface {
                commands.entity(entity).insert(Visibility::Hidden);
            }
            commands
                .entity(entity)
                .insert(NetworkPcAppearanceMaterialBound0104 {
                    rig_root,
                    exact_route: part_scene.exact_route.clone(),
                    binding,
                });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize_network_pc_visuals_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    parents: Query<&ChildOf>,
    appearance_parts: Query<&NetworkPcAppearancePart0104>,
    pending_attachments: Query<&PendingNetworkPcAppearanceAttachment0104>,
    metadata_errors: Query<(Entity, &LegacyMaterialMetadataError)>,
    surfaces: Query<
        (Entity, Option<&NetworkPcAppearanceMaterialBound0104>),
        With<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    mut rigs: Query<(
        Entity,
        &NetworkPcRig0104,
        &NativePlayerRigStatus,
        &mut NetworkPcRigAppearanceStatus0104,
        &mut Visibility,
    )>,
) {
    let Some(asset_server) = asset_server else {
        return;
    };
    'rigs: for (rig_root, rig, native_status, mut status, mut visibility) in &mut rigs {
        if !matches!(*status, NetworkPcRigAppearanceStatus0104::Loading) {
            continue;
        }
        if let NativePlayerRigStatus::Blocked(error) = native_status {
            block_network_pc_rig(
                &mut commands,
                rig,
                &mut status,
                format!("native shared rig blocked: {error}"),
            );
            continue;
        }
        let NativePlayerRigStatus::ReadyAnimated { parts, .. } = native_status else {
            continue;
        };
        if let Some((_, error)) = metadata_errors.iter().find(|(entity, _)| {
            is_descendant_of(*entity, rig_root, &parents)
                && ancestor_network_pc_appearance_part(
                    *entity,
                    rig_root,
                    &parents,
                    &appearance_parts,
                )
                .is_some()
        }) {
            block_network_pc_rig(
                &mut commands,
                rig,
                &mut status,
                format!("live player material metadata failed: {}", error.0),
            );
            continue;
        }
        let expected_routes = network_pc_expected_appearance_routes(&rig.look);
        let expected_skinned_parts = rig
            .look
            .parts
            .iter()
            .filter(|part| part.uses_shared_skin())
            .count();
        let actual_parts = appearance_parts
            .iter()
            .filter(|part| part.rig_root == rig_root)
            .collect::<Vec<_>>();
        let actual_routes = actual_parts
            .iter()
            .map(|part| (part.kind, part.exact_route.clone()))
            .collect::<std::collections::BTreeSet<_>>();
        let has_pending_attachment = pending_attachments
            .iter()
            .any(|pending| pending.rig_root == rig_root);
        if !network_pc_appearance_routes_ready(
            &expected_routes,
            expected_skinned_parts,
            *parts,
            actual_parts.len(),
            &actual_routes,
            has_pending_attachment,
        ) {
            continue;
        }

        let mut surface_count = 0_usize;
        let mut bound_routes = std::collections::BTreeSet::new();
        let mut loading_texture = false;
        for (entity, bound) in &surfaces {
            let Some(part_scene) =
                ancestor_network_pc_appearance_part(entity, rig_root, &parents, &appearance_parts)
            else {
                // Most importantly, the separately owned weapon has no
                // appearance marker and remains outside this readiness gate.
                continue;
            };
            surface_count += 1;
            let Some(bound) = bound else {
                loading_texture = true;
                continue;
            };
            if bound.rig_root != rig_root || bound.exact_route != part_scene.exact_route {
                block_network_pc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!("live player material surface {entity:?} has stale binder ownership"),
                );
                continue 'rigs;
            }
            bound_routes.insert((part_scene.kind, bound.exact_route.clone()));
            if let Some(texture) = &bound.binding.texture {
                match asset_server.load_state(texture.handle.id()) {
                    LoadState::Failed(error) => {
                        block_network_pc_rig(
                            &mut commands,
                            rig,
                            &mut status,
                            format!("live player texture {:?} failed: {error}", texture.path),
                        );
                        continue 'rigs;
                    }
                    LoadState::Loaded => {}
                    _ => loading_texture = true,
                }
            }
        }
        if surface_count == 0 || bound_routes != expected_routes || loading_texture {
            continue;
        }
        *status = NetworkPcRigAppearanceStatus0104::Ready;
        *visibility = Visibility::Inherited;
        commands
            .entity(rig.pc_root)
            .remove::<NetworkPcVisualIssue0104>();
    }
}

pub(super) fn finish_network_pc_emote_0104(
    mut commands: Commands,
    roots: Query<(
        Entity,
        &NetworkPcVisual0104,
        &RemoteAnimation,
        &crate::remote::RemoteMotion,
    )>,
    rigs: Query<(
        Option<&NativePlayerRigAnimationApplied>,
        Option<&NativePlayerRigStand1Playback>,
        Option<&NativePlayerRigAnimationRequest>,
    )>,
    players: Query<&AnimationPlayer>,
) {
    for (root, visual, animation, motion) in &roots {
        if !matches!(
            animation.state,
            RemoteAnimationState::Emoting { .. } | RemoteAnimationState::Attacking
        ) {
            continue;
        }
        let Ok((Some(applied), Some(playback), Some(request))) = rigs.get(visual.rig_root) else {
            continue;
        };
        let expected = match animation.state {
            RemoteAnimationState::Emoting { clip } => applied.clip == clip.name(),
            RemoteAnimationState::Attacking => applied.clip == request.clip,
            _ => false,
        };
        if expected
            && applied.revision == request.revision
            && !applied.repeat
            && players.get(playback.animation_player).is_ok_and(|player| {
                player
                    .animation(playback.animation_node)
                    .is_some_and(|active| active.is_finished())
            })
        {
            let state = if motion.movement_key != 0 {
                RemoteAnimationState::Moving {
                    direction_key: motion.movement_key,
                }
            } else {
                RemoteAnimationState::Idle
            };
            commands.entity(root).insert(RemoteAnimation { state });
        }
    }
}

pub(super) fn ancestor_network_pc_appearance_part<'a>(
    mut entity: Entity,
    rig_root: Entity,
    parents: &Query<&ChildOf>,
    parts: &'a Query<&NetworkPcAppearancePart0104>,
) -> Option<&'a NetworkPcAppearancePart0104> {
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

pub(super) fn bind_network_hnpc_attachments_0104(
    mut commands: Commands,
    bones: Query<&NativePlayerRigBones>,
    mut transforms: Query<&mut Transform>,
    pending: Query<(Entity, &PendingNetworkHnpcAttachment0104)>,
) {
    for (attachment, pending) in &pending {
        let Ok(bones) = bones.get(pending.rig_root) else {
            continue;
        };
        let Some(socket) = bones.by_full_path(&pending.socket_full_path) else {
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
                entity.remove::<PendingNetworkHnpcAttachment0104>();
            });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_network_hnpc_materials_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
    parents: Query<&ChildOf>,
    parts: Query<&NetworkHnpcPart0104>,
    metadata: Query<&PendingLegacyModelMaterial>,
    mut surfaces: Query<
        (
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        Without<NetworkHnpcMaterialBound0104>,
    >,
    mut rigs: Query<(
        Entity,
        &NetworkHnpcRig0104,
        &NativePlayerRigStatus,
        &mut NetworkHnpcRigAppearanceStatus0104,
    )>,
) {
    let (Some(asset_server), Some(mut materials)) = (asset_server, materials) else {
        return;
    };
    'rigs: for (rig_root, rig, native_status, mut status) in &mut rigs {
        if !matches!(*status, NetworkHnpcRigAppearanceStatus0104::Loading)
            || !native_status.is_ready()
        {
            continue;
        }
        for (entity, mut handle, own_metadata, companion) in &mut surfaces {
            let Some(part) = network_hnpc_part_ancestor(entity, rig_root, &parents, &parts) else {
                continue;
            };
            let matching = rig
                .look
                .parts
                .iter()
                .filter(|candidate| candidate.exact_route == part.exact_route)
                .collect::<Vec<_>>();
            let [part_look] = matching.as_slice() else {
                block_network_hnpc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!(
                        "HNPC appearance route {:?} resolves {} times",
                        part.exact_route,
                        matching.len()
                    ),
                );
                continue 'rigs;
            };
            if part_look.glb != part.glb {
                block_network_hnpc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!(
                        "HNPC route {:?} loaded {:?}, expected {:?}",
                        part.exact_route, part.glb, part_look.glb
                    ),
                );
                continue 'rigs;
            }
            let Some(pending) = own_metadata
                .or_else(|| companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok()))
            else {
                block_network_hnpc_rig(
                    &mut commands,
                    rig,
                    &mut status,
                    format!("HNPC appearance surface {entity:?} has no material metadata"),
                );
                continue 'rigs;
            };
            crate::legacy_model_material::make_legacy_material_unique(
                &mut handle.0,
                &mut materials,
            );
            let Some(mut material) = materials.get_mut(&handle.0) else {
                continue;
            };
            let binding = match bind_native_player_look_material(
                &asset_server,
                pending,
                &mut material,
                &rig.look,
                part_look,
                companion.is_some(),
            ) {
                Ok(binding) => binding,
                Err(error) => {
                    block_network_hnpc_rig(&mut commands, rig, &mut status, error);
                    continue 'rigs;
                }
            };
            // ActorSkinCombiner assigns primaryTextures[j] even when the
            // requested texture is null. HNPC v2 represents that native
            // state as an absent primary texture, preserving shader defaults.
            if binding.role == crate::player_appearance_material::ActorSkinTextureRole::Primary
                && part_look.primary_texture.is_none()
            {
                material.base_texture = None;
            }
            if binding.hide_surface {
                commands.entity(entity).insert(Visibility::Hidden);
            }
            commands
                .entity(entity)
                .insert(NetworkHnpcMaterialBound0104 {
                    rig_root,
                    exact_route: part.exact_route.clone(),
                    binding,
                });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize_network_hnpc_visuals_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    parents: Query<&ChildOf>,
    parts: Query<&NetworkHnpcPart0104>,
    metadata_errors: Query<(Entity, &LegacyMaterialMetadataError)>,
    surfaces: Query<
        (Entity, Option<&NetworkHnpcMaterialBound0104>),
        With<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    mut rigs: Query<(
        Entity,
        &NetworkHnpcRig0104,
        &NativePlayerRigStatus,
        &mut NetworkHnpcRigAppearanceStatus0104,
        &mut Visibility,
    )>,
) {
    let Some(asset_server) = asset_server else {
        return;
    };
    'rigs: for (rig_root, rig, native_status, mut status, mut visibility) in &mut rigs {
        if !matches!(*status, NetworkHnpcRigAppearanceStatus0104::Loading) {
            continue;
        }
        if let NativePlayerRigStatus::Blocked(error) = native_status {
            block_network_hnpc_rig(
                &mut commands,
                rig,
                &mut status,
                format!("native HNPC shared rig blocked: {error}"),
            );
            continue;
        }
        let NativePlayerRigStatus::ReadyAnimated {
            parts: skinned_parts,
            ..
        } = native_status
        else {
            continue;
        };
        if let Some((_, error)) = metadata_errors.iter().find(|(entity, _)| {
            network_hnpc_part_ancestor(*entity, rig_root, &parents, &parts).is_some()
        }) {
            block_network_hnpc_rig(
                &mut commands,
                rig,
                &mut status,
                format!("HNPC material metadata failed: {}", error.0),
            );
            continue;
        }
        let expected_routes = rig
            .look
            .parts
            .iter()
            .map(|part| part.exact_route.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let expected_skinned = rig
            .look
            .parts
            .iter()
            .filter(|part| part.uses_shared_skin())
            .count();
        let actual_routes = parts
            .iter()
            .filter(|part| part.rig_root == rig_root)
            .map(|part| part.exact_route.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if *skinned_parts != expected_skinned || actual_routes != expected_routes {
            continue;
        }

        let mut surface_count = 0_usize;
        let mut bound_routes = std::collections::BTreeSet::new();
        let mut loading = false;
        for (entity, bound) in &surfaces {
            if network_hnpc_part_ancestor(entity, rig_root, &parents, &parts).is_none() {
                continue;
            }
            surface_count += 1;
            let Some(bound) = bound else {
                loading = true;
                continue;
            };
            if bound.rig_root != rig_root {
                loading = true;
                continue;
            }
            bound_routes.insert(bound.exact_route.as_str());
            if let Some(texture) = &bound.binding.texture {
                match asset_server.load_state(texture.handle.id()) {
                    LoadState::Failed(error) => {
                        block_network_hnpc_rig(
                            &mut commands,
                            rig,
                            &mut status,
                            format!("HNPC texture {:?} failed: {error}", texture.path),
                        );
                        continue 'rigs;
                    }
                    LoadState::Loaded => {}
                    _ => loading = true,
                }
            }
        }
        if surface_count == 0 || bound_routes != expected_routes || loading {
            continue;
        }
        *status = NetworkHnpcRigAppearanceStatus0104::Ready;
        *visibility = Visibility::Inherited;
        commands
            .entity(rig.npc_root)
            .remove::<NetworkNpcVisualIssue0104>();
    }
}

pub(super) fn hnpc_idle_clips(
    row: &Value,
    meshes: &[Value],
    context: &str,
) -> Result<Option<[String; 3]>, String> {
    let index = optional_i64(row, "m_iMesh", context)?.unwrap_or_default();
    if index <= 0 {
        return Ok(None);
    }
    let Some(mesh) = usize::try_from(index)
        .ok()
        .and_then(|index| meshes.get(index))
    else {
        return Ok(None);
    };
    Ok(Some([
        required_string(mesh, "m_pstrMMeshModelString", context)?.to_owned(),
        required_string(mesh, "m_pstrMTextureString", context)?.to_owned(),
        required_string(mesh, "m_pstrMTextureString2", context)?.to_owned(),
    ]))
}

pub(super) fn network_hnpc_part_ancestor<'a>(
    mut entity: Entity,
    rig_root: Entity,
    parents: &Query<&ChildOf>,
    parts: &'a Query<&NetworkHnpcPart0104>,
) -> Option<&'a NetworkHnpcPart0104> {
    loop {
        if let Ok(part) = parts.get(entity)
            && part.rig_root == rig_root
        {
            return Some(part);
        }
        if entity == rig_root {
            return None;
        }
        entity = parents.get(entity).ok()?.parent();
    }
}

pub(super) fn network_npc_fusion_matter_params_0104(
    source: &LegacyModelMaterialParams,
) -> LegacyModelMaterialParams {
    let mut promoted =
        LegacyModelMaterialParams::for_shader(LegacyShaderKind::FusionMatterLightDir);
    // Unity texture replacement retains the source material's MainTex scale
    // and offset. The replacement shader owns every other property default.
    promoted.uv_scale = source.uv_scale;
    promoted.uv_offset = source.uv_offset;
    promoted.uv_pivot = source.uv_pivot;
    promoted.uv_rotation_degrees = source.uv_rotation_degrees;
    promoted
}
