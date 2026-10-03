use super::*;

pub(super) fn load_network_pc_visual_data_0104(
    shared: Option<Res<CharacterCreationDataResource>>,
    locator: Option<Res<AssetLocator>>,
    mut state: ResMut<NetworkPcVisualDataState0104>,
) {
    if state.attempted {
        return;
    }
    if let Some(shared) = shared {
        state.attempted = true;
        state.data = Some(shared.0.clone());
        return;
    }
    let Some(locator) = locator else {
        return;
    };
    state.attempted = true;
    match CharacterCreationData::open(locator.root()) {
        Ok(data) => state.data = Some(Arc::new(data)),
        Err(error) => {
            let error = format!("failed to load live player appearance contracts: {error}");
            error!("{error}");
            state.error = Some(error);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_network_pc_visuals_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    data_state: Res<NetworkPcVisualDataState0104>,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
    catalog: Option<Res<NativePlayerRigCatalog>>,
    asset_cache: Option<ResMut<NativePlayerRigAssetCache>>,
    mut generations: ResMut<NetworkPcVisualGeneration0104>,
    players: Query<
        (
            Entity,
            &NetworkRemotePc0104,
            &NetworkPcAppearance0104,
            &PendingPcVisual0104,
            Option<&NetworkPcVisual0104>,
        ),
    >,
) {
    let (Some(asset_server), Some(data), Some(catalog), Some(mut asset_cache)) =
        (asset_server, data_state.data.as_ref(), catalog, asset_cache)
    else {
        return;
    };

    for (pc_root, remote, appearance, pending, current) in &players {
        if current.is_some_and(|current| current.request == *pending) {
            commands
                .entity(pc_root)
                .remove::<NetworkPcVisualIssue0104>();
            continue;
        }
        if let Some(current) = current {
            commands.entity(current.rig_root).despawn();
        }

        let look = match data.resolve_pc_appearance(&appearance.0) {
            Ok(look) => look,
            Err(error) => {
                commands.entity(pc_root).insert(NetworkPcVisualIssue0104 {
                    pc_id: remote.pc_id,
                    detail: format!("live player appearance did not resolve: {error}"),
                });
                continue;
            }
        };
        let catalog_weapon_profile = weapon_catalog.profile_for_item(pending.equipment[0].item_id);
        if look.weapon_animation_profile != catalog_weapon_profile {
            commands.entity(pc_root).insert(NetworkPcVisualIssue0104 {
                pc_id: remote.pc_id,
                detail: format!(
                    "live player weapon animation profile disagrees between avatar and XDT catalogs: avatar={:?}, xdt={catalog_weapon_profile:?}",
                    look.weapon_animation_profile
                ),
            });
            continue;
        }
        let skinned_parts = look
            .parts
            .iter()
            .filter(|part| part.uses_shared_skin())
            .cloned()
            .collect::<Vec<_>>();
        generations.0 = generations.0.wrapping_add(1).max(1);
        let generation = generations.0;
        let mut request = match NativePlayerRigSpawnRequest::new(
            look.identity.clone(),
            generation,
            look.gender,
            RenderLayers::layer(0),
        )
        .use_part_routes(
            &catalog,
            skinned_parts.iter().map(|part| part.exact_route.clone()),
        ) {
            Ok(request) => request,
            Err(error) => {
                commands.entity(pc_root).insert(NetworkPcVisualIssue0104 {
                    pc_id: remote.pc_id,
                    detail: format!("live player shared-rig request is invalid: {error}"),
                });
                continue;
            }
        };
        request.parent = Some(pc_root);
        request.transform = native_scene_container_transform(NativeSceneRole::CharacterGameplay);
        request.visibility = Visibility::Hidden;
        if let Some(profile) = look.weapon_animation_profile {
            request.animation_name = profile.stand().name().to_owned();
        }
        let spawned = match spawn_native_player_rig(
            &mut commands,
            &asset_server,
            &mut asset_cache,
            &catalog,
            request,
        ) {
            Ok(spawned) => spawned,
            Err(error) => {
                commands.entity(pc_root).insert(NetworkPcVisualIssue0104 {
                    pc_id: remote.pc_id,
                    detail: format!("live player shared rig failed to spawn: {error}"),
                });
                continue;
            }
        };
        for (part_root, part) in spawned.part_scenes.iter().copied().zip(&skinned_parts) {
            commands
                .entity(part_root)
                .insert(NetworkPcAppearancePart0104 {
                    rig_root: spawned.root,
                    kind: part.kind,
                    exact_route: part.exact_route.clone(),
                    glb: part.glb.clone(),
                });
        }
        for part in look.parts.iter().filter(|part| !part.uses_shared_skin()) {
            let Some(slot) = network_pc_appearance_attachment_slot(part.kind) else {
                continue;
            };
            let placement = standard_player_attachment_placement();
            let scene = asset_cache.scene(&asset_server, part.glb.clone());
            commands.spawn((
                Name::new(format!(
                    "Network player appearance attachment: {}",
                    part.exact_route
                )),
                NetworkPcAppearancePart0104 {
                    rig_root: spawned.root,
                    kind: part.kind,
                    exact_route: part.exact_route.clone(),
                    glb: part.glb.clone(),
                },
                PendingNetworkPcAppearanceAttachment0104 {
                    rig_root: spawned.root,
                    socket_full_path: player_attachment_socket_full_path(look.gender, slot),
                    socket_local_scale_override: placement.socket_local_scale_override,
                },
                ChildOf(spawned.root),
                WorldAssetRoot(scene),
                placement.item_local,
                Visibility::Inherited,
                RenderLayers::layer(0),
            ));
        }
        if let Some(weapon) = look
            .parts
            .iter()
            .find(|part| part.kind == NativePlayerPartKind::Weapon)
        {
            let placement = standard_player_attachment_placement();
            let socket_full_path = player_attachment_socket_full_path(
                look.gender,
                LegacyPlayerAttachmentSlot::RightPistol,
            );
            let scene = asset_cache.scene(&asset_server, weapon.glb.clone());
            commands.spawn((
                Name::new(format!("Network player weapon: {}", weapon.exact_route)),
                PendingNetworkPcWeaponAttachment0104 {
                    rig_root: spawned.root,
                    socket_full_path,
                    socket_local_scale_override: placement.socket_local_scale_override,
                },
                NetworkPcWeaponAttachment0104,
                ChildOf(spawned.root),
                WorldAssetRoot(scene),
                placement.item_local,
                Visibility::Inherited,
                RenderLayers::layer(0),
            ));
        }
        commands.entity(spawned.root).insert((
            NetworkPcRig0104 {
                pc_root,
                pc_id: remote.pc_id,
                look,
            },
            NetworkPcRigAppearanceStatus0104::Loading,
        ));
        commands.entity(pc_root).insert(NetworkPcVisual0104 {
            pc_id: remote.pc_id,
            rig_root: spawned.root,
            generation,
            request: pending.clone(),
        });
        commands
            .entity(pc_root)
            .remove::<NetworkPcVisualIssue0104>();
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_network_hnpc_visuals_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    catalog_state: Res<NetworkNpcVisualCatalogState0104>,
    asset_cache: Option<ResMut<NativePlayerRigAssetCache>>,
    mut generations: ResMut<NetworkHnpcVisualGeneration0104>,
    npcs: Query<
        (
            Entity,
            &NetworkNpc0104,
            &PendingNpcVisual0104,
            Option<&NetworkHnpcVisual0104>,
            Option<&NetworkNpcVisual0104>,
        ),
        Changed<PendingNpcVisual0104>,
    >,
) {
    let (Some(asset_server), Some(catalog), Some(hnpc), Some(mut asset_cache)) = (
        asset_server,
        catalog_state.catalog.as_ref(),
        catalog_state.hnpc.as_ref(),
        asset_cache,
    ) else {
        return;
    };
    for (npc_root, npc, pending, current, direct) in &npcs {
        let Some(definition) = catalog.get_hnpc(pending.npc_type) else {
            continue;
        };
        let Some(appearance) = hnpc.appearance(definition.appearance_index) else {
            commands.entity(npc_root).insert(NetworkNpcVisualIssue0104 {
                npc_type: pending.npc_type,
                detail: format!(
                    "HNPC appearance {} is absent from the production catalog",
                    definition.appearance_index
                ),
            });
            continue;
        };
        let Some(_) = appearance.look.as_ref() else {
            commands.entity(npc_root).insert(NetworkNpcVisualIssue0104 {
                npc_type: pending.npc_type,
                detail: format!(
                    "HNPC appearance {} has no renderable WearItems parts",
                    definition.appearance_index
                ),
            });
            continue;
        };
        let grounding = NetworkNpcGrounding0104::from_server_height(definition.height_server_units);
        if current.is_some_and(|current| {
            current.npc_type == definition.npc_type
                && current.appearance_index == definition.appearance_index
        }) {
            commands
                .entity(npc_root)
                .insert(grounding)
                .remove::<NetworkNpcVisualIssue0104>();
            continue;
        }
        if let Some(current) = current {
            commands.entity(current.rig_root).despawn();
        }
        if let Some(direct) = direct {
            commands.entity(direct.visual_container).despawn();
            commands.entity(npc_root).remove::<NetworkNpcVisual0104>();
        }

        generations.0 = generations.0.wrapping_add(1).max(1);
        let generation = generations.0;
        let visual = match spawn_network_hnpc_visual_0104(
            &mut commands,
            &asset_server,
            &mut asset_cache,
            hnpc,
            npc_root,
            definition,
            generation,
        ) {
            Ok(visual) => visual,
            Err(detail) => {
                commands.entity(npc_root).insert(NetworkNpcVisualIssue0104 {
                    npc_type: definition.npc_type,
                    detail,
                });
                continue;
            }
        };
        commands.entity(npc_root).insert((
            Name::new(format!("network HNPC {} type {}", npc.npc_id, npc.npc_type)),
            Visibility::Inherited,
            grounding,
            visual,
        ));
        commands
            .entity(npc_root)
            .remove::<NetworkNpcVisualIssue0104>();
    }
}

pub(super) fn resolve_network_npc_visuals_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    catalog_state: Res<NetworkNpcVisualCatalogState0104>,
    npcs: Query<
        (
            Entity,
            &NetworkNpc0104,
            &PendingNpcVisual0104,
            Option<&NetworkNpcVisual0104>,
        ),
        Changed<PendingNpcVisual0104>,
    >,
) {
    let (Some(asset_server), Some(catalog)) = (asset_server, catalog_state.catalog.as_ref()) else {
        return;
    };
    for (entity, npc, pending, current) in &npcs {
        if catalog.get_hnpc(pending.npc_type).is_some() {
            continue;
        }
        let Some(definition) = catalog.get(pending.npc_type) else {
            commands.entity(entity).insert(NetworkNpcVisualIssue0104 {
                npc_type: pending.npc_type,
                detail: format!(
                    "no validated XDT-to-GLB route for server NPC {} type {}",
                    pending.npc_id, pending.npc_type
                ),
            });
            continue;
        };
        let grounding = NetworkNpcGrounding0104::from_server_height(definition.height_server_units);
        if current.is_some_and(|current| {
            current.npc_type == definition.npc_type
                && current.glb_path == definition.glb
                && current.table_scale == definition.table_scale
        }) {
            commands
                .entity(entity)
                .insert(grounding)
                .remove::<NetworkNpcVisualIssue0104>();
            continue;
        }
        if let Some(current) = current {
            commands.entity(current.visual_container).despawn();
        }

        let visual = spawn_network_npc_visual_0104(
            &mut commands,
            &asset_server,
            entity,
            definition,
            format!("network NPC {}", npc.npc_id),
        );
        let scene = visual.scene;
        if let Some(contract) = definition.collision_contract.as_ref() {
            let collider_glb = contract
                .collider_glb
                .clone()
                .unwrap_or_else(|| definition.glb.clone());
            commands
                .entity(scene)
                .insert(PendingNetworkNpcCollision0104 {
                    gameplay_root: entity,
                    npc_type: definition.npc_type,
                    glb_path: collider_glb,
                    contract: contract.clone(),
                })
                .observe(materialize_network_npc_collision_0104);
        }
        commands.entity(entity).insert((
            Name::new(format!("network NPC {} type {}", npc.npc_id, npc.npc_type)),
            Visibility::Inherited,
            grounding,
            visual,
            NetworkNpcAppearEffect0104::default(),
        ));
        commands
            .entity(entity)
            .remove::<NetworkNpcVisualIssue0104>();
    }
}
