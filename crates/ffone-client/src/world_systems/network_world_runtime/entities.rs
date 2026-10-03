use super::*;

/// Spawn the production modular HNPC appearance for world and editor consumers.
#[allow(clippy::too_many_arguments)]
pub fn spawn_network_hnpc_visual_0104(
    commands: &mut Commands,
    asset_server: &AssetServer,
    asset_cache: &mut NativePlayerRigAssetCache,
    hnpc: &HnpcRuntimeCatalog,
    npc_root: Entity,
    definition: &NetworkHnpcVisualDefinition0104,
    generation: u64,
) -> Result<NetworkHnpcVisual0104, String> {
    let look = hnpc
        .appearance(definition.appearance_index)
        .and_then(|appearance| appearance.look.clone())
        .ok_or_else(|| {
            format!(
                "HNPC appearance {} has no renderable parts",
                definition.appearance_index
            )
        })?;
    let skinned_parts = look
        .parts
        .iter()
        .filter(|part| part.uses_shared_skin())
        .cloned()
        .collect::<Vec<_>>();
    let mut request = NativePlayerRigSpawnRequest::new(
        look.identity.clone(),
        generation,
        look.gender,
        RenderLayers::layer(0),
    )
    .use_part_routes(
        hnpc.rig_catalog(),
        skinned_parts.iter().map(|part| part.exact_route.clone()),
    )
    .map_err(|error| format!("HNPC shared-rig request is invalid: {error}"))?;
    request.parent = Some(npc_root);
    request.transform = native_scene_container_transform(NativeSceneRole::CharacterGameplay);
    request.visibility = Visibility::Hidden;
    // HNPC idle belongs to its table, not the equipped player's weapon profile.
    // The animation owner selects the first table pose once the rig is ready.
    let spawned = spawn_native_player_rig(
        commands,
        asset_server,
        asset_cache,
        hnpc.rig_catalog(),
        request,
    )
    .map_err(|error| format!("HNPC shared rig failed to spawn: {error}"))?;
    for (part_root, part) in spawned.part_scenes.iter().copied().zip(&skinned_parts) {
        commands.entity(part_root).insert(NetworkHnpcPart0104 {
            rig_root: spawned.root,
            exact_route: part.exact_route.clone(),
            glb: part.glb.clone(),
        });
    }
    for part in look.parts.iter().filter(|part| !part.uses_shared_skin()) {
        let slot = match part.kind {
            NativePlayerPartKind::Hat => LegacyPlayerAttachmentSlot::Hat,
            NativePlayerPartKind::Glasses => LegacyPlayerAttachmentSlot::Glasses,
            NativePlayerPartKind::Back => LegacyPlayerAttachmentSlot::Back,
            NativePlayerPartKind::Weapon => LegacyPlayerAttachmentSlot::RightPistol,
            NativePlayerPartKind::Face
            | NativePlayerPartKind::Hair
            | NativePlayerPartKind::Shirt
            | NativePlayerPartKind::Pants
            | NativePlayerPartKind::Shoes
            | NativePlayerPartKind::Vehicle => continue,
        };
        let placement = standard_player_attachment_placement();
        let scene = asset_cache.scene(asset_server, part.glb.clone());
        commands.spawn((
            Name::new(format!("Network HNPC attachment: {}", part.exact_route)),
            NetworkHnpcPart0104 {
                rig_root: spawned.root,
                exact_route: part.exact_route.clone(),
                glb: part.glb.clone(),
            },
            PendingNetworkHnpcAttachment0104 {
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
    commands.entity(spawned.root).insert((
        NetworkHnpcRig0104 {
            npc_root,
            npc_type: definition.npc_type,
            animation_ends: hnpc.animation_ends(look.gender),
            animation_sounds: hnpc.animation_sounds(look.gender),
            look,
            idle_clips: definition.idle_clips.clone(),
        },
        NetworkHnpcRigAppearanceStatus0104::Loading,
    ));
    Ok(NetworkHnpcVisual0104 {
        npc_type: definition.npc_type,
        appearance_index: definition.appearance_index,
        rig_root: spawned.root,
        generation,
        walk_animation_speed: definition.walk_animation_speed,
        run_animation_speed: definition.run_animation_speed,
    })
}

/// Spawns the ordinary-NPC render subtree from a validated production catalog
/// definition. Both the live network world and native authoring previews use
/// this function so Scene0 selection, logical-root normalization, XDT scale,
/// GLTF ownership, and texture-variant metadata cannot drift apart.
pub fn spawn_network_npc_visual_0104(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
    definition: &NetworkNpcVisualDefinition0104,
    label: impl AsRef<str>,
) -> NetworkNpcVisual0104 {
    let label = label.as_ref();
    let gltf: Handle<Gltf> = asset_server.load(definition.glb.clone());
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(definition.glb.clone()));
    let spawned = spawn_legacy_character_scene(
        commands,
        parent,
        scene,
        definition.logical_name.clone(),
        LegacyCharacterRootPolicy::Npc {
            table_scale: definition.table_scale,
        },
    );
    commands
        .entity(spawned.visual_container)
        .insert(Name::new(format!(
            "{label} visual ({})",
            definition.logical_name
        )));
    commands.entity(spawned.scene).insert((
        Name::new(format!("{label} normalized Scene0")),
        LegacyCharacterSceneDeferredReveal,
    ));
    commands.entity(parent).remove::<(
        NetworkNpcVisualMaterialReady0104,
        NetworkNpcMaterialVisibilityBlocked0104,
    )>();
    NetworkNpcVisual0104 {
        npc_type: definition.npc_type,
        logical_name: definition.logical_name.clone(),
        glb_path: definition.glb.clone(),
        table_scale: definition.table_scale,
        visual_container: spawned.visual_container,
        scene: spawned.scene,
        gltf,
        main_texture: definition.main_texture.clone(),
        sub_texture: definition.sub_texture.clone(),
        walk_animation_speed: definition.walk_animation_speed,
        run_animation_speed: definition.run_animation_speed,
        animation_effect_events: definition.animation_effect_events.clone(),
        animation_sound_events: definition.animation_sound_events.clone(),
        animation_ends: definition.animation_ends.clone(),
        material_animation_clips: definition.material_animation_clips.clone(),
    }
}
