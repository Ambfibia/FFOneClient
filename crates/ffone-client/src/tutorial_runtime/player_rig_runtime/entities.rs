use super::*;

/// Spawn the selected account character below the existing gameplay controller.
/// A failure returns before a fallback is hidden or removed.
#[allow(clippy::too_many_arguments)]
pub fn spawn_tutorial_selected_player_rig(
    commands: &mut Commands,
    asset_server: &AssetServer,
    asset_cache: &mut NativePlayerRigAssetCache,
    catalog: &NativePlayerRigCatalog,
    weapon_animation_catalog: &PlayerWeaponAnimationCatalog,
    data: &CharacterCreationData,
    controller_root: Entity,
    character: &CharacterSummary,
    generation: u64,
    render_layers: RenderLayers,
    fallback_visual: Option<Entity>,
    tutorial_stand_semantics: bool,
) -> Result<SpawnedTutorialSelectedPlayerRig, String> {
    let look = data
        .resolve_character_summary(character)
        .map_err(|error| format!("selected tutorial player look: {error}"))?;
    look.validate()
        .map_err(|error| format!("selected tutorial player look: {error}"))?;
    let gender = look.gender;
    let hand_item_id = character.equipment[CharacterEquipSlot0104::Hand as usize].item_id;
    let initial_weapon_item_id = (hand_item_id != 0).then_some(hand_item_id);
    let catalog_weapon_profile = initial_weapon_item_id
        .and_then(|item_id| weapon_animation_catalog.profile_for_item(item_id));
    if look.weapon_animation_profile != catalog_weapon_profile {
        return Err(format!(
            "selected tutorial player weapon animation profile disagrees between avatar and XDT catalogs: avatar={:?}, xdt={catalog_weapon_profile:?}",
            look.weapon_animation_profile
        ));
    }
    let initial_weapon_profile = look.weapon_animation_profile;
    let gender_contract = catalog.gender(gender)?;
    let capabilities = TutorialPlayerRigCapabilities::from_contract(gender, &gender_contract.clips)
        .map_err(|error| format!("selected tutorial player animation contract: {error}"))?;
    let clip_indices = resolve_required_tutorial_player_clips(&capabilities, gender)?;

    let skinned_parts = look
        .parts
        .iter()
        .filter(|part| part.uses_shared_skin())
        .cloned()
        .collect::<Vec<_>>();
    let exact_part_routes = skinned_parts
        .iter()
        .map(|part| part.exact_route.clone())
        .collect::<Vec<_>>();
    let mut request = NativePlayerRigSpawnRequest::new(
        look.identity.clone(),
        generation,
        gender,
        render_layers.clone(),
    )
    .use_part_routes(catalog, exact_part_routes)?;
    request.parent = Some(controller_root);
    request.transform = native_scene_container_transform(NativeSceneRole::CharacterGameplay);
    request.visibility = Visibility::Hidden;
    request.animation_name = TutorialPlayerClip::Stand1.name().to_owned();

    let mut weapon_models = BTreeMap::new();
    let weapon_item_ids = TUTORIAL_WEAPON_IDS
        .into_iter()
        .chain(initial_weapon_item_id)
        .collect::<BTreeSet<_>>();
    for item_id in weapon_item_ids {
        let part = data
            .resolve_weapon_attachment(item_id as u32, gender)
            .map_err(|error| format!("tutorial weapon {item_id}: {error}"))?;
        weapon_models.insert(item_id, part);
    }
    // Every fallible catalog/data operation above completes before any entity
    // is spawned. A rejected selection therefore cannot disturb the existing
    // Dexter fallback or leave a partial hidden rig behind.
    let skeleton_gltf = asset_server.load::<Gltf>(gender_contract.skeleton_glb.clone());
    let spawned = spawn_native_player_rig(commands, asset_server, asset_cache, catalog, request)?;
    for (part_root, part) in spawned.part_scenes.iter().copied().zip(&skinned_parts) {
        commands
            .entity(part_root)
            .insert(TutorialPlayerAppearancePart {
                rig_root: spawned.root,
                kind: part.kind,
                exact_route: part.exact_route.clone(),
                glb: part.glb.clone(),
            });
    }
    for part in look
        .parts
        .iter()
        .filter(|part| !part.uses_shared_skin() && part.kind != NativePlayerPartKind::Weapon)
    {
        let Some(slot) = tutorial_player_appearance_attachment_slot(part.kind) else {
            continue;
        };
        let placement = standard_player_attachment_placement();
        let scene = asset_cache.scene(asset_server, part.glb.clone());
        commands.spawn((
            Name::new(format!("Selected player attachment: {}", part.exact_route)),
            TutorialPlayerAppearancePart {
                rig_root: spawned.root,
                kind: part.kind,
                exact_route: part.exact_route.clone(),
                glb: part.glb.clone(),
            },
            PendingTutorialPlayerAppearanceAttachment {
                rig_root: spawned.root,
                socket_full_path: player_attachment_socket_full_path(gender, slot),
                socket_local_scale_override: placement.socket_local_scale_override,
            },
            ChildOf(spawned.root),
            WorldAssetRoot(scene),
            placement.item_local,
            Visibility::Inherited,
            render_layers.clone(),
        ));
    }
    commands.entity(spawned.root).insert((
        TutorialSelectedPlayerRig {
            controller_root,
            gender,
            look,
            consumer: TutorialPlayerPresentationConsumer::new(capabilities),
            clip_indices,
            skeleton_gltf,
            weapon_models,
            attached_weapon: None,
            attached_weapon_item_id: None,
            render_layers,
            tutorial_stand_semantics,
            initial_weapon_profile,
        },
        TutorialSelectedPlayerRigActive,
        TutorialSelectedPlayerRigStatus::Loading,
        TutorialPlayerFallbackVisual {
            entity: fallback_visual,
        },
    ));
    Ok(SpawnedTutorialSelectedPlayerRig {
        rig_root: spawned.root,
        skeleton_scene: spawned.skeleton_scene,
    })
}
