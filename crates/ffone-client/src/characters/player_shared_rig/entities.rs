use super::*;

/// Spawns one fully independent shared-rig instance below the caller-selected
/// parent and render layer.
pub fn spawn_native_player_rig(
    commands: &mut Commands,
    asset_server: &AssetServer,
    asset_cache: &mut NativePlayerRigAssetCache,
    catalog: &NativePlayerRigCatalog,
    request: NativePlayerRigSpawnRequest,
) -> Result<SpawnedNativePlayerRig, String> {
    if request.identity.trim().is_empty() {
        return Err("native player-rig identity is empty".to_owned());
    }
    if request
        .asset_source
        .as_ref()
        .is_some_and(|source| source.is_empty() || source.contains("://"))
    {
        return Err("native player-rig asset source must be a bare registered name".to_owned());
    }
    let gender = catalog.gender(request.gender)?.clone();
    let mut route_set = BTreeSet::new();
    let mut clothes_slots = BTreeMap::new();
    let mut parts = Vec::with_capacity(request.exact_part_routes.len());
    for route in &request.exact_part_routes {
        if !route_set.insert(route.as_str()) {
            return Err(format!(
                "native player-rig request repeats part route {route:?}"
            ));
        }
        let matches = gender
            .creator_parts
            .iter()
            .filter(|part| part.exact_route == *route)
            .cloned()
            .collect::<Vec<_>>();
        let [part] = matches.as_slice() else {
            return Err(format!(
                "native player-rig route {route:?} has {} certified contracts for {:?}",
                matches.len(),
                request.gender
            ));
        };
        if let Some(previous_route) = clothes_slots.insert(
            part.actor_skin_combiner_clothes_index,
            part.exact_route.clone(),
        ) {
            return Err(format!(
                "native player-rig request maps routes {previous_route:?} and {:?} to ActorSkinCombiner clothes slot {}",
                part.exact_route, part.actor_skin_combiner_clothes_index
            ));
        }
        parts.push(part.clone());
    }

    let mut root_commands = commands.spawn((
        Name::new(format!("Native player rig: {}", request.identity)),
        NativePlayerRigInstance {
            identity: request.identity,
            generation: request.generation,
            gender: request.gender,
        },
        NativePlayerRigStatus::default(),
        request.transform,
        request.visibility,
        request.render_layers.clone(),
    ));
    if let Some(parent) = request.parent {
        root_commands.insert(ChildOf(parent));
    }
    if let Some(shape) = request.body_shape {
        root_commands.insert(shape);
    }
    let root = root_commands.id();

    let skeleton_uri = asset_uri(request.asset_source.as_deref(), &gender.skeleton_glb);
    let skeleton_gltf = asset_cache.gltf(asset_server, skeleton_uri.clone());
    let skeleton_scene_handle = asset_cache.scene(asset_server, skeleton_uri);
    let skeleton_scene = commands
        .spawn((
            Name::new(format!("{:?} shared player skeleton", request.gender)),
            NativePlayerRigSkeletonScene {
                rig_root: root,
                generation: request.generation,
            },
            ChildOf(root),
            WorldAssetRoot(skeleton_scene_handle.clone()),
            Transform::IDENTITY,
            Visibility::Inherited,
            request.render_layers.clone(),
        ))
        .observe(mark_native_player_rig_scene_ready)
        .id();

    let mut part_scenes = Vec::with_capacity(parts.len());
    let mut runtime_parts = Vec::with_capacity(parts.len());
    for part in parts {
        let uri = asset_uri(request.asset_source.as_deref(), &part.glb);
        let scene = asset_cache.scene(asset_server, uri);
        let part_root = commands
            .spawn((
                Name::new(format!("Native player part: {}", part.exact_route)),
                NativePlayerRigPartScene {
                    rig_root: root,
                    generation: request.generation,
                    exact_route: part.exact_route.clone(),
                    glb: part.glb.clone(),
                },
                ChildOf(root),
                WorldAssetRoot(scene.clone()),
                Transform::IDENTITY,
                Visibility::Inherited,
                request.render_layers.clone(),
            ))
            .observe(mark_native_player_rig_scene_ready)
            .id();
        part_scenes.push(part_root);
        runtime_parts.push(NativePlayerRigPartRuntime {
            root: part_root,
            scene,
            contract: part,
        });
    }

    commands.entity(root).insert(NativePlayerRigRuntime {
        gender,
        skeleton_gltf,
        skeleton_scene,
        skeleton_scene_handle,
        parts: runtime_parts,
        animation_name: request.animation_name,
    });

    Ok(SpawnedNativePlayerRig {
        root,
        skeleton_scene,
        part_scenes,
    })
}
