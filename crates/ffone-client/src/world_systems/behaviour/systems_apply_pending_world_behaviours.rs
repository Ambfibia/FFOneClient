use super::*;

/// Apply the behaviour document of every tile that has been spawned but not
/// yet given its behaviour.
///
/// This reacts to the spawned scene root instead of being called from the
/// world-slice spawn, so streaming, tutorial entry and world entry all get the
/// same treatment without threading another parameter through those systems.
/// A tile without a readable document still renders; the failure is reported
/// once and the root is marked so it is not retried every frame.
pub fn apply_pending_world_behaviours(
    mut commands: Commands,
    behaviour_root: Option<Res<NativeWorldBehaviourRoot>>,
    mut effect_runtime: Option<ResMut<TutorialEffectRuntime>>,
    catalog: Res<crate::world::NativeWorldCatalog>,
    mut roots: Query<
        (
            Entity,
            &crate::world::NativeWorldSceneRoot,
            &Children,
            Option<&mut PendingWorldBehaviourLoad>,
        ),
        (
            Without<WorldBehavioursApplied>,
            Without<PendingWorldBehaviourSpawn>,
            Without<PendingNativeWorldSceneSpawn>,
            Without<PendingNativeWorldSceneUnload>,
        ),
    >,
    visuals: Query<(&SpawnedNativeWorldVisual, Option<&Name>, &GlobalTransform)>,
    colliders: Query<(&SpawnedNativeWorldCollider, Option<&Name>, &GlobalTransform)>,
) {
    let Some(behaviour_root) = behaviour_root else {
        return;
    };
    for (root, scene_root, children, pending_load) in &mut roots {
        let Some(mut pending_load) = pending_load else {
            let asset_root = behaviour_root.0.clone();
            let scope = scene_root.scope;
            let scene_name = scene_root.name.clone();
            let behaviour_asset = catalog
                .behaviour_asset(scope, &scene_name)
                .map(|(path, blake3)| (path, blake3));
            let object_asset = catalog
                .object_asset(scope, &scene_name)
                .map(|(path, blake3)| (path, blake3));
            let task = IoTaskPool::get().spawn(async move {
                let mut document = if let Some((path, expected_blake3)) = behaviour_asset {
                    let bytes = read_hashed_world_behaviour_asset(
                        &path,
                        &expected_blake3,
                        "world behaviour",
                    )?;
                    parse_native_world_behaviours(&bytes, &path.display().to_string(), &scene_name)
                        .map_err(|error| error.to_string())?
                } else {
                    load_native_world_behaviours(&asset_root, scope, &scene_name)
                        .map_err(|error| error.to_string())?
                };
                let object_source_routes = if let Some((path, expected_blake3)) = object_asset {
                    let bytes = read_hashed_world_behaviour_asset(
                        &path,
                        &expected_blake3,
                        "world object routes",
                    )?;
                    parse_native_world_object_source_routes(&bytes, &path.display().to_string())?
                } else {
                    load_native_world_object_source_routes(&asset_root, &scene_name)?
                };
                let animation_clips = take_world_animation_clips(&mut document);
                Ok(LoadedNativeWorldBehaviour {
                    document,
                    animation_clips,
                    object_source_routes,
                })
            });
            commands
                .entity(root)
                .insert(PendingWorldBehaviourLoad(Some(task)));
            continue;
        };
        let task = pending_load
            .0
            .as_mut()
            .expect("pending world behaviour load retains its task");
        let Some(load_result) = block_on(future::poll_once(task)) else {
            continue;
        };
        pending_load.0.take();
        commands.entity(root).remove::<PendingWorldBehaviourLoad>();

        let Some(scene) = catalog
            .scenes()
            .iter()
            .find(|scene| scene.name == scene_root.name)
        else {
            IoTaskPool::get()
                .spawn(async move { drop(load_result) })
                .detach();
            continue;
        };
        let path_to_model = scene
            .models
            .iter()
            .map(|model| (model.path.as_str(), model.id.as_str()))
            .collect::<HashMap<_, _>>();
        let object_source_routes = load_result
            .as_ref()
            .ok()
            .map(|loaded| &loaded.object_source_routes);
        let mut entities_by_model = HashMap::<String, Vec<Entity>>::new();
        let mut entities_by_source_node = HashMap::<String, Vec<Entity>>::new();
        let mut legacy_sources = Vec::<(String, Option<String>, Entity)>::new();
        let mut authored_model_worlds = HashMap::<Entity, Mat4>::new();
        for child in children.iter() {
            if let Ok((visual, name, world)) = visuals.get(child)
                && let Some(model) = path_to_model.get(visual.model_path.as_str())
            {
                authored_model_worlds.insert(child, world.to_matrix());
                entities_by_model
                    .entry((*model).to_owned())
                    .or_default()
                    .push(child);
                let source_node = name
                    .and_then(organized_world_source_node)
                    .map(str::to_owned);
                if let Some(source_node) = source_node.as_ref() {
                    entities_by_source_node
                        .entry(source_node.clone())
                        .or_default()
                        .push(child);
                }
                if let Some(legacy_id) = legacy_static_model_id(&visual.source_model_path) {
                    legacy_sources.push((legacy_id, source_node, child));
                }
            }
            if let Ok((collider, name, world)) = colliders.get(child)
                && let Some(model) = path_to_model.get(collider.model_path.as_str())
            {
                authored_model_worlds.insert(child, world.to_matrix());
                entities_by_model
                    .entry((*model).to_owned())
                    .or_default()
                    .push(child);
                let source_node = name
                    .and_then(organized_world_source_node)
                    .map(str::to_owned);
                if let Some(source_node) = source_node.as_ref() {
                    entities_by_source_node
                        .entry(source_node.clone())
                        .or_default()
                        .push(child);
                }
                let source_model_path = (!collider.source_model_path.is_empty())
                    .then_some(collider.source_model_path.as_str())
                    .or_else(|| {
                        let source_node = source_node.as_ref()?;
                        object_source_routes?
                            .by_node_and_geometry
                            .get(&(source_node.clone(), (*model).to_owned()))
                            .map(String::as_str)
                    });
                if let Some(legacy_id) = source_model_path.and_then(legacy_static_model_id) {
                    legacy_sources.push((legacy_id, source_node, child));
                }
            }
        }
        // A behaviour record commonly lists one visual payload id and one
        // collider payload id from the same source node.  The organizer keeps
        // the original visual route and the shared source node on both new
        // entities.  Binding the visual id to the complete node group restores
        // the renderer and triangle collider as one moving gameplay object.
        for (legacy_id, source_node, source_entity) in legacy_sources {
            let binding = source_node
                .as_ref()
                .and_then(|source_node| entities_by_source_node.get(source_node))
                .cloned()
                .unwrap_or_else(|| vec![source_entity]);
            entities_by_model
                .entry(legacy_id)
                .or_default()
                .extend(binding);
        }
        for entities in entities_by_model.values_mut() {
            entities.sort_unstable();
            entities.dedup();
        }

        match load_result {
            Ok(loaded) => {
                if let Err(error) = validate_world_effect_nif_animation_bindings(
                    &loaded.document,
                    &entities_by_model,
                ) {
                    warn!(
                        tile = %scene_root.name,
                        "native world NIF animation bindings unavailable: {error}"
                    );
                    commands.entity(root).insert((
                        WorldBehavioursApplied {
                            document: scene_root.name.clone(),
                            billboards: 0,
                            visibility_switches: 0,
                            effect_emitters: 0,
                            animations: 0,
                            triggers: 0,
                            waypoints: 0,
                            trigger_volumes: 0,
                            rigid_bodies: 0,
                            blockers: 1,
                        },
                        NativeWorldBehaviourStatus::Blocked(error),
                    ));
                    continue;
                }
                let document = Arc::new(loaded.document);
                // EP elements share the same bounded ambient queue as named
                // serialized emitters. Enqueueing the complete tile here is
                // cheap; compilation and ECS materialization remain capped by
                // the effect runtime and never outrank gameplay VFX.
                let native_particle_effects = effect_runtime
                    .as_deref_mut()
                    .map(|runtime| enqueue_world_ep_effects(root, &document, runtime))
                    .unwrap_or_default();
                let native_scripted_effects = effect_runtime
                    .is_some()
                    .then(|| world_scripted_effect_count(&document))
                    .unwrap_or_default();
                commands
                    .entity(root)
                    .insert(PendingWorldBehaviourSpawn::new(
                        document,
                        loaded.animation_clips,
                        entities_by_model,
                        authored_model_worlds,
                        native_particle_effects,
                        native_scripted_effects,
                    ));
            }
            Err(error) => {
                warn!(
                    tile = %scene_root.name,
                    "native world behaviours unavailable: {error}"
                );
                commands.entity(root).insert((
                    WorldBehavioursApplied {
                        document: scene_root.name.clone(),
                        billboards: 0,
                        visibility_switches: 0,
                        effect_emitters: 0,
                        animations: 0,
                        triggers: 0,
                        waypoints: 0,
                        trigger_volumes: 0,
                        rigid_bodies: 0,
                        blockers: 0,
                    },
                    NativeWorldBehaviourStatus::Blocked(error),
                ));
            }
        }
    }
}
