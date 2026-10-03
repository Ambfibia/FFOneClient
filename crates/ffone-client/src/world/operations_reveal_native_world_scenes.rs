use super::*;

pub(super) fn reveal_native_world_scenes(
    mut roots: Query<
        (
            Entity,
            &NativeWorldBehaviourStatus,
            &mut NativeWorldVisualPresentationStatus,
            &mut NativeWorldPresentationStatus,
            &mut Visibility,
        ),
        (
            With<NativeWorldSceneRoot>,
            Without<SpawnedNativeWorldVisual>,
            Without<PendingNativeWorldSceneSpawn>,
            Without<PendingNativeWorldSceneUnload>,
        ),
    >,
    mut visuals: Query<
        (
            Entity,
            Option<&NativeWorldVisualSceneReady>,
            Option<&RuntimeManagedNativeWorldVisual>,
            Option<&NativeWorldObjectRangeMember>,
            Option<&NativeWorldObjectRangeReady>,
            Option<&NativeWorldObjectRangeContract>,
            Option<&NativeWorldObjectRangeUnbounded>,
            Option<&NativeWorldMaterialPresentationFailed>,
            &mut Visibility,
        ),
        (
            With<SpawnedNativeWorldVisual>,
            Without<NativeWorldSceneRoot>,
        ),
    >,
    materials: Query<
        (
            Entity,
            Option<&LegacyMaterialApplied>,
            Option<&LegacyMaterialMetadataError>,
        ),
        With<GltfMaterialExtras>,
    >,
    rendered_meshes: Query<(Entity, Option<&MeshTag>), With<Mesh3d>>,
    range_members: Query<(), With<NativeWorldObjectRangeMember>>,
    unbounded_ranges: Query<(), With<NativeWorldObjectRangeUnbounded>>,
    colliders: Query<(Entity, &NativeWorldColliderStatus)>,
    animation_material_bindings: Query<(
        Entity,
        &crate::world_behaviour::WorldAnimationMaterialBindings,
    )>,
    parents: Query<&ChildOf>,
    terrains: Query<
        (
            Entity,
            &NativeWorldColliderStatus,
            Option<&NativeTerrainDetailStatus>,
        ),
        With<NativeWorldTerrainPresentation>,
    >,
    standard_meshes: Query<(Entity, &ChildOf, &MeshMaterial3d<StandardMaterial>)>,
    standard_materials: Res<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut streaming: ResMut<NativeWorldStreamingStatus>,
) {
    // A root cannot be visually complete until admission has removed the
    // pending-spawn component. The direct render path can then reveal exact
    // materials independently of the longer exact-triangle collider cook.
    let candidate_roots = roots
        .iter()
        .filter(|(_, behaviour, visual_presentation, presentation, _)| {
            (**visual_presentation == NativeWorldVisualPresentationStatus::Loading
                || **presentation == NativeWorldPresentationStatus::Loading)
                && matches!(behaviour, NativeWorldBehaviourStatus::Ready)
        })
        .map(|(entity, _, _, _, _)| entity)
        .collect::<HashSet<_>>();
    if candidate_roots.is_empty() {
        return;
    }

    // Aggregate each global query once. The old root-major implementation
    // repeated all visual/material/collider scans and ancestor walks for every
    // simultaneously loading tile, making its cost O(tiles * world entities).
    let mut visual_readiness = candidate_roots
        .iter()
        .copied()
        .map(|root| (root, true))
        .collect::<HashMap<_, _>>();
    // Terrain is prepared asynchronously after its placement was admitted.
    // Its collider becomes Ready before budgeted grass assembly finishes.
    for (entity, collider, details) in &terrains {
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        if !matches!(collider, NativeWorldColliderStatus::Ready { .. })
            || !matches!(
                details,
                Some(
                    NativeTerrainDetailStatus::Ready { .. }
                        | NativeTerrainDetailStatus::SourceContractPending
                )
            )
        {
            visual_readiness.insert(root, false);
        }
    }
    // Grass StandardMaterials do not carry GltfMaterialExtras. Their textures
    // must join the gate explicitly, including an actionable load failure.
    for (entity, parent, handle) in &standard_meshes {
        // Grass chunks are direct children of their owning terrain. Other
        // StandardMaterials retain the existing legacy fail-closed policy.
        if terrains.get(parent.parent()).is_err() {
            continue;
        }
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        let Some(material) = standard_materials.get(&handle.0) else {
            visual_readiness.insert(root, false);
            continue;
        };
        if let Some(texture) = &material.base_color_texture {
            if !asset_server.is_loaded_with_dependencies(texture.id()) {
                visual_readiness.insert(root, false);
                if let LoadState::Failed(error) = asset_server.load_state(texture.id()) {
                    streaming.blocker =
                        Some(format!("native terrain detail texture failed: {error}"));
                }
            }
        }
    }
    for (entity, ready, _, range_member, range_ready, _, _, _, _) in &mut visuals {
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        if ready.is_none() || (range_member.is_some() && range_ready.is_none()) {
            visual_readiness.insert(root, false);
        }
    }
    // Animated legacy surfaces are cloned and bound incrementally to keep
    // dense tiles within the per-frame admission budget. Keep the complete
    // tile hidden until those bindings are ready; otherwise the amortization
    // would trade a frame hitch for a visible static-to-animated pop.
    for (entity, bindings) in &animation_material_bindings {
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        if !bindings.is_ready() {
            visual_readiness.insert(root, false);
        }
    }
    // The GltfMesh subasset can become ready before exact legacy materials and
    // mip chains replace the loader fallback. A typed metadata error may let
    // the rest of the tile settle only after its owning visual is identified;
    // that visual remains fail-closed Hidden below. It must never expose the
    // StandardMaterial fallback that turns alpha/additive planes opaque.
    let mut failed_visuals = HashSet::new();
    for (entity, applied, error) in &materials {
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        if applied.is_none() && error.is_none() {
            visual_readiness.insert(root, false);
            continue;
        }
        if error.is_some() {
            let mut ancestor = entity;
            loop {
                if range_members.get(ancestor).is_ok() {
                    failed_visuals.insert(ancestor);
                    // The complete logical object must leave the custom
                    // tag/dither path before the tile settles. Direct meshes
                    // own this marker themselves; legacy WorldAssetRoot instances
                    // kept it on their parent.
                    if unbounded_ranges.get(ancestor).is_err() {
                        visual_readiness.insert(root, false);
                    }
                    break;
                }
                let Ok(parent) = parents.get(ancestor) else {
                    break;
                };
                ancestor = parent.parent();
            }
        }
    }
    // Direct map meshes receive their exact shared range tag before reveal.
    // Retain the equivalent protection for the few legacy WorldAssetRoot users:
    // a late descendant mesh must not flash without its object-complete range
    // contract. Permanently-unbounded objects deliberately carry no MeshTag.
    for (entity, mesh_tag) in &rendered_meshes {
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        if mesh_tag.is_some() {
            continue;
        }
        let mut ancestor = entity;
        loop {
            if range_members.get(ancestor).is_ok() {
                if unbounded_ranges.get(ancestor).is_err() {
                    visual_readiness.insert(root, false);
                }
                break;
            }
            let Ok(parent) = parents.get(ancestor) else {
                break;
            };
            ancestor = parent.parent();
        }
    }
    let visually_ready_roots = visual_readiness
        .into_iter()
        .filter_map(|(root, ready)| ready.then_some(root))
        .collect::<HashSet<_>>();

    let mut collision_readiness = candidate_roots
        .iter()
        .copied()
        .map(|root| (root, visually_ready_roots.contains(&root)))
        .collect::<HashMap<_, _>>();
    for (entity, status) in &colliders {
        let Some(root) = native_world_ancestor_in_roots(entity, &candidate_roots, &parents) else {
            continue;
        };
        if !matches!(status, NativeWorldColliderStatus::Ready { .. }) {
            collision_readiness.insert(root, false);
        }
    }

    for (root, _, mut visual_presentation, mut presentation, mut visibility) in &mut roots {
        if visually_ready_roots.contains(&root) {
            *visual_presentation = NativeWorldVisualPresentationStatus::Ready;
            *visibility = Visibility::Inherited;
        }
        if collision_readiness.get(&root).copied().unwrap_or(false) {
            *presentation = NativeWorldPresentationStatus::Ready;
        }
    }
    for (
        entity,
        _,
        runtime_managed,
        _,
        _,
        range_contract,
        unbounded_range,
        material_failed,
        mut visual_visibility,
    ) in &mut visuals
    {
        if native_world_ancestor_in_roots(entity, &visually_ready_roots, &parents).is_some() {
            *visual_visibility = match (
                failed_visuals.contains(&entity) || material_failed.is_some(),
                runtime_managed,
                range_contract,
                unbounded_range,
            ) {
                (true, _, _, _) => Visibility::Hidden,
                (_, Some(_), _, _) => Visibility::Hidden,
                (_, _, Some(contract), _) if contract.visible => Visibility::Inherited,
                (_, _, Some(_), _) => Visibility::Hidden,
                (_, _, None, Some(_)) => Visibility::Inherited,
                // Metadata-only collision helpers deliberately have neither
                // a range contract nor render geometry. Keep them hidden.
                (_, _, None, None) => Visibility::Hidden,
            };
        }
    }
}

/// Squared distance from a native Bevy position to a legacy Unity dong AABB.
///
/// Unity X is reflected into native X; Z and the original 512-unit bounds are
/// unchanged. This is the exact `DongLoader.GetSqrDistance` metric.
#[must_use]
pub fn legacy_dong_squared_distance_native(native_position: Vec3, tile: [i32; 2]) -> Option<f32> {
    if !native_position.x.is_finite()
        || !native_position.z.is_finite()
        || !(0..16).contains(&tile[0])
        || !(0..16).contains(&tile[1])
    {
        return None;
    }
    let unity_x = -native_position.x;
    let unity_z = native_position.z;
    let minimum_x = tile[0] as f32 * WORLD_TILE_SIZE_NATIVE;
    let maximum_x = (tile[0] + 1) as f32 * WORLD_TILE_SIZE_NATIVE;
    let minimum_z = tile[1] as f32 * WORLD_TILE_SIZE_NATIVE;
    let maximum_z = (tile[1] + 1) as f32 * WORLD_TILE_SIZE_NATIVE;
    let distance_x = if unity_x < minimum_x {
        minimum_x - unity_x
    } else if unity_x > maximum_x {
        unity_x - maximum_x
    } else {
        0.0
    };
    let distance_z = if unity_z < minimum_z {
        minimum_z - unity_z
    } else if unity_z > maximum_z {
        unity_z - maximum_z
    } else {
        0.0
    };
    Some(distance_x * distance_x + distance_z * distance_z)
}

pub(super) fn neighboring_water_scenes(
    catalog: &NativeWorldCatalog,
    scope: NativeWorldScope,
    native_position: Vec3,
) -> Vec<(&NativeWorldScene, &[usize])> {
    let Some(current) = catalog.select_in_scope(scope, native_position) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for z_offset in -1..=1 {
        for x_offset in -1..=1 {
            let tile = [current.tile[0] + x_offset, current.tile[1] + z_offset];
            let Some(scene) = catalog.scene_for_tile(scope, tile) else {
                continue;
            };
            let authored_scope = scene.scope.unwrap_or(NativeWorldScope::WorldMap);
            let Some(indices) = catalog
                .water_visual_indices
                .get(&(authored_scope, scene.tile))
            else {
                continue;
            };
            result.push((scene, indices.as_ref()));
        }
    }
    result
}

pub(super) fn stream_neighboring_water_surfaces(
    mut commands: Commands,
    catalog: Res<NativeWorldCatalog>,
    requested_scope: Option<Res<NativeWorldRequestedScope>>,
    asset_server: Res<AssetServer>,
    cameras: Query<&LegacyOrbitCamera>,
    transforms: Query<&Transform>,
    full_roots: Query<(
        &NativeWorldSceneRoot,
        &NativeWorldVisualPresentationStatus,
        Option<&PendingNativeWorldSceneUnload>,
    )>,
    water_roots: Query<(Entity, &NativeWorldNeighborWaterRoot)>,
    mut status: ResMut<NativeWorldStreamingStatus>,
) {
    let player_position = cameras
        .iter()
        .next()
        .and_then(|camera| transforms.get(camera.target).ok())
        .map(|transform| transform.translation);
    let scope = requested_scope.as_deref().map_or_else(
        || {
            full_roots
                .iter()
                .next()
                .map_or(NativeWorldScope::WorldMap, |(root, _, _)| {
                    root.selection_scope
                })
        },
        |requested| requested.0,
    );
    let desired = player_position.map_or_else(HashMap::new, |position| {
        neighboring_water_scenes(&catalog, scope, position)
            .into_iter()
            .map(|(scene, indices)| ((scope, scene.tile), (scene, indices)))
            .collect::<HashMap<_, _>>()
    });
    let fully_presented = full_roots
        .iter()
        .filter(|(root, presentation, unloading)| {
            root.selection_scope == scope
                && unloading.is_none()
                && **presentation == NativeWorldVisualPresentationStatus::Ready
        })
        .map(|(root, _, _)| (scope, root.tile))
        .collect::<HashSet<_>>();

    let mut existing = HashSet::new();
    for (entity, root) in &water_roots {
        let key = (root.selection_scope, root.tile);
        if !desired.contains_key(&key) || fully_presented.contains(&key) || !existing.insert(key) {
            commands.entity(entity).insert(Visibility::Hidden).despawn();
        }
    }

    for (key, (scene, indices)) in desired {
        if fully_presented.contains(&key) || existing.contains(&key) {
            continue;
        }
        let mut unbounded = HashSet::new();
        for &visual_index in indices {
            let visual = &scene.visuals[visual_index];
            unbounded.insert(native_world_object_range_group_key(
                visual_index,
                &visual.name,
            ));
        }
        let root = commands
            .spawn((
                Name::new(format!("{} neighbor water", scene.name)),
                NativeWorldNeighborWaterRoot {
                    tile: scene.tile,
                    selection_scope: scope,
                },
                NativeWorldObjectRangeGroups {
                    pending: HashMap::new(),
                    ready_to_finalize: VecDeque::new(),
                    unbounded,
                },
                match scene.root.try_to_bevy("neighbor water root") {
                    Ok(transform) => transform,
                    Err(error) => {
                        status.blocker = Some(format!(
                            "neighbor water root failed for {}: {error}",
                            scene.name
                        ));
                        continue;
                    }
                },
                Visibility::Inherited,
            ))
            .id();
        for &visual_index in indices {
            let visual = &scene.visuals[visual_index];
            match spawn_native_world_visual(
                &mut commands,
                &asset_server,
                scene,
                root,
                visual_index,
                visual,
            ) {
                Ok(entity) => {
                    commands
                        .entity(entity)
                        .insert(NativeWorldNeighborWaterVisual);
                }
                Err(error) => {
                    status.blocker = Some(format!(
                        "neighbor water visual failed for {}: {error}",
                        scene.name
                    ));
                    queue_native_world_scene_unload(&mut commands, root);
                    break;
                }
            }
        }
    }
}

pub(super) fn reveal_neighboring_water_surfaces(
    catalog: Res<NativeWorldCatalog>,
    requested_scope: Option<Res<NativeWorldRequestedScope>>,
    cameras: Query<&LegacyOrbitCamera>,
    transforms: Query<&Transform>,
    roots: Query<
        (&NativeWorldSceneRoot, &NativeWorldVisualPresentationStatus),
        Without<PendingNativeWorldSceneUnload>,
    >,
    water_roots: Query<&NativeWorldNeighborWaterRoot>,
    parents: Query<&ChildOf>,
    mut location: ResMut<NativeWorldLocationPresentation>,
    mut streaming: ResMut<NativeWorldStreamingStatus>,
    mut waters: Query<
        (
            Entity,
            Option<&NativeWorldVisualSceneReady>,
            Option<&LegacyMaterialApplied>,
            Option<&LegacyMaterialMetadataError>,
            &mut Visibility,
        ),
        With<NativeWorldNeighborWaterVisual>,
    >,
) {
    let position = cameras
        .iter()
        .next()
        .and_then(|camera| transforms.get(camera.target).ok())
        .map(|transform| transform.translation);
    let scope = requested_scope.as_deref().map_or_else(
        || {
            roots
                .iter()
                .next()
                .map_or(NativeWorldScope::WorldMap, |(root, _)| root.selection_scope)
        },
        |requested| requested.0,
    );
    let targets = position
        .map(|position| catalog.legacy_stream_target_tiles(scope, position))
        .unwrap_or_default();
    let presented = roots
        .iter()
        .filter(|(root, status)| {
            root.selection_scope == scope && **status == NativeWorldVisualPresentationStatus::Ready
        })
        .map(|(root, _)| root.tile)
        .collect::<HashSet<_>>();
    let world_ready = !targets.is_empty() && targets.iter().all(|tile| presented.contains(tile));
    let desired = position
        .map(|position| neighboring_water_scenes(&catalog, scope, position))
        .unwrap_or_default();
    let mut ready_counts = HashMap::<[i32; 2], usize>::new();
    for (entity, mesh_ready, material_ready, material_error, mut visibility) in &mut waters {
        let owner = parents
            .get(entity)
            .ok()
            .and_then(|parent| water_roots.get(parent.parent()).ok());
        let desired_owner = owner.is_some_and(|owner| {
            owner.selection_scope == scope
                && desired.iter().any(|(scene, _)| scene.tile == owner.tile)
                && !presented.contains(&owner.tile)
        });
        let ready = desired_owner
            && mesh_ready.is_some()
            && material_ready.is_some()
            && material_error.is_none();
        if desired_owner {
            if let Some(error) = material_error {
                streaming.blocker = Some(format!("neighboring water material failed: {}", error.0));
            }
        }
        if ready {
            *ready_counts.entry(owner.unwrap().tile).or_default() += 1;
        }
        let desired = if world_ready && ready {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
    location.scope = Some(scope);
    location.position = position;
    location.ready = world_ready
        && desired.iter().all(|(scene, indices)| {
            presented.contains(&scene.tile)
                || ready_counts.get(&scene.tile).copied().unwrap_or(0) == indices.len()
        });
}

pub(super) fn stream_native_world_dongs(
    mut commands: Commands,
    catalog: Res<NativeWorldCatalog>,
    requested_scope: Option<Res<NativeWorldRequestedScope>>,
    cameras: Query<&LegacyOrbitCamera>,
    transforms: Query<&Transform>,
    roots: Query<(Entity, &NativeWorldSceneRoot)>,
    unloading_roots: Query<(), With<PendingNativeWorldSceneUnload>>,
    terrain_loads: Query<(), With<PendingNativeWorldSceneTerrainLoad>>,
    collider_statuses: Query<&NativeWorldColliderStatus>,
    mut status: ResMut<NativeWorldStreamingStatus>,
) {
    let player_position = cameras
        .iter()
        .next()
        .and_then(|camera| transforms.get(camera.target).ok())
        .map(|transform| transform.translation);
    // Keep physical roots separate from active roots. Incremental unload can
    // retain thousands of entities for many frames; forgetting those roots
    // here allowed the same tile to be loaded a second time and let resident
    // memory/colliders exceed the absolute nine-tile bound.
    let mut physical_roots = roots.iter().collect::<Vec<_>>();
    physical_roots.sort_by_key(|(_, root)| (root.tile[1], root.tile[0]));
    let mut loaded = physical_roots
        .iter()
        .copied()
        .filter(|(entity, _)| unloading_roots.get(*entity).is_err())
        .collect::<Vec<_>>();
    loaded.sort_by_key(|(_, root)| (root.tile[1], root.tile[0]));
    status.resident_tiles = physical_roots.len();
    status.loading_colliders = collider_statuses
        .iter()
        .filter(|collider| matches!(collider, NativeWorldColliderStatus::Loading))
        .count();
    status.resident_budget_exceeded = physical_roots.len() > NATIVE_WORLD_MAX_RESIDENT_TILES;

    let Some(player_position) = player_position else {
        status.target_tiles = 0;
        if let Some((entity, _)) = loaded.first() {
            queue_native_world_scene_unload(&mut commands, *entity);
        }
        return;
    };
    let scope = requested_scope.as_deref().map_or_else(
        || {
            loaded
                .first()
                .map_or(NativeWorldScope::WorldMap, |(_, root)| root.selection_scope)
        },
        |requested| requested.0,
    );
    let target_tiles = catalog
        .legacy_stream_target_tiles(scope, player_position)
        .into_iter()
        .collect::<HashSet<_>>();
    status.target_tiles = target_tiles.len();

    // A gameplay-scope transition can briefly leave roots from the previous
    // slice alive while their incremental destruction runs. Remove active
    // foreign roots before readiness, collider errors or the tile-only target
    // set can protect them and consume the nine-tile admission budget.
    if let Some((entity, _)) = loaded
        .iter()
        .find(|(_, root)| root.selection_scope != scope)
    {
        queue_native_world_scene_unload(&mut commands, *entity);
        return;
    }

    if let Some((entity, _)) = loaded.iter().find(|(_, root)| {
        root.selection_scope != scope
            || catalog
                .scene_for_tile(root.selection_scope, root.tile)
                .and_then(|scene| {
                    catalog.presentation_squared_distance(
                        root.selection_scope,
                        player_position,
                        scene,
                    )
                })
                .is_some_and(|distance| {
                    distance
                        > EXTENDED_DONG_UNLOAD_DISTANCE_NATIVE
                            * EXTENDED_DONG_UNLOAD_DISTANCE_NATIVE
                })
    }) {
        queue_native_world_scene_unload(&mut commands, *entity);
        return;
    }
    if let Some(error) = collider_statuses
        .iter()
        .find_map(|collider| match collider {
            NativeWorldColliderStatus::Blocked(error) => Some(error.clone()),
            NativeWorldColliderStatus::Loading | NativeWorldColliderStatus::Ready { .. } => None,
        })
    {
        status.blocker = Some(format!("native dong collider blocked: {error}"));
        return;
    }
    if status.blocker.is_some() {
        return;
    }

    let loaded_tiles = physical_roots
        .iter()
        .filter(|(_, root)| root.selection_scope == scope)
        .map(|(_, root)| root.tile)
        .collect::<HashSet<_>>();
    let Some(scene) = catalog.next_legacy_stream_load(scope, player_position, &loaded_tiles) else {
        return;
    };
    // An unloading root still owns memory, colliders and behaviour entities.
    // Do not admit its replacement (or any additional tile) until physical
    // destruction has restored capacity.
    if !native_world_has_physical_admission_capacity(physical_roots.len()) {
        // The 64-unit unload hysteresis can retain one old non-target tile
        // while a new <=356 target is missing. At the hard nine-tile cap that
        // state otherwise deadlocks forever because the retained tile is not
        // yet beyond 420. Evict exactly one farthest non-target, then keep the
        // physical root counted until incremental destruction completes.
        if physical_roots.len() == loaded.len() {
            let eviction = native_world_stream_eviction_tile(
                loaded.iter().filter_map(|(_, root)| {
                    if root.selection_scope != scope {
                        return None;
                    }
                    let scene = catalog.scene_for_tile(root.selection_scope, root.tile)?;
                    let distance = catalog.presentation_squared_distance(
                        root.selection_scope,
                        player_position,
                        scene,
                    )?;
                    Some((root.tile, distance))
                }),
                &target_tiles,
            );
            if let Some(tile) = eviction
                && let Some((entity, _)) = loaded
                    .iter()
                    .find(|(_, root)| root.selection_scope == scope && root.tile == tile)
            {
                queue_native_world_scene_unload(&mut commands, *entity);
            }
        }
        return;
    }
    // Reserve the physical root before starting I/O. Two root-owned tasks
    // overlap decoding without exceeding the nine-root residency cap; their
    // main-thread terrain installation is still limited to one per frame.
    if scene.native_terrain.is_some()
        && terrain_loads.iter().len() >= NATIVE_WORLD_TERRAIN_LOADS_IN_FLIGHT
    {
        return;
    }
    if let Err(error) =
        begin_streamed_native_world_scene_loading(&mut commands, &catalog, scene, scope)
    {
        status.blocker = Some(format!(
            "native dong stream failed for {}: {error}",
            scene.name
        ));
    }
}

pub(super) fn native_world_has_physical_admission_capacity(physical_root_count: usize) -> bool {
    physical_root_count < NATIVE_WORLD_MAX_RESIDENT_TILES
}

/// Pick one stale retained tile when load/unload hysteresis temporarily makes
/// the union of the old and new target sets larger than the physical cap.
/// Current targets are never candidates. Equal distances use descending
/// source scan order for a deterministic single eviction.
pub(super) fn native_world_stream_eviction_tile(
    loaded: impl IntoIterator<Item = ([i32; 2], f32)>,
    target_tiles: &HashSet<[i32; 2]>,
) -> Option<[i32; 2]> {
    loaded
        .into_iter()
        .filter(|(tile, distance)| !target_tiles.contains(tile) && distance.is_finite())
        .max_by(|(left_tile, left_distance), (right_tile, right_distance)| {
            left_distance
                .total_cmp(right_distance)
                .then_with(|| (left_tile[1], left_tile[0]).cmp(&(right_tile[1], right_tile[0])))
        })
        .map(|(tile, _)| tile)
}

/// Native equivalent of the legacy `MapAttributeTable` dong key. The reflected
/// native X axis is converted back before C#-style truncation and integer `/512`.
#[must_use]
pub fn native_dong_key(world_x: f32, world_z: f32) -> Option<i32> {
    fn csharp_tile(value: f32) -> Option<i32> {
        if !value.is_finite() || value < i32::MIN as f32 || value > i32::MAX as f32 {
            return None;
        }
        Some((value.trunc() as i32) / WORLD_TILE_SIZE_NATIVE as i32)
    }
    let tile_x = csharp_tile(-world_x)?;
    let tile_z = csharp_tile(world_z)?;
    let packed = (((tile_x & 0xffff) as u32) << 16) | ((tile_z & 0xffff) as u32);
    Some(packed as i32)
}

// Streaming retirement executes the actual despawns inside the time budget.
// Timing only Commands construction would miss relationship hooks and asset
// handle releases, which are the expensive part of unloading a dense tile.
pub(super) fn queue_native_world_scene_unload(commands: &mut Commands, root: Entity) {
    commands
        .entity(root)
        .queue_silenced(|mut entity: EntityWorldMut| {
            entity.remove::<(
                PendingNativeWorldSceneSpawn,
                PendingNativeWorldSceneTerrainLoad,
            )>();
            // Several loading systems can fail on the same root in one frame.
            // Never restart an in-progress traversal or keep its terrain task alive.
            if !entity.contains::<PendingNativeWorldSceneUnload>() {
                entity.insert((PendingNativeWorldSceneUnload::default(), Visibility::Hidden));
            }
        });
}

pub(super) fn unload_native_world_scenes_incrementally(world: &mut World) {
    unload_native_world_scene_batch(
        world,
        NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME,
        Some(std::time::Duration::from_millis(1)),
    );
}

pub(super) fn unload_native_world_scene_batch(
    world: &mut World,
    entity_budget: usize,
    time_budget: Option<std::time::Duration>,
) {
    let started = std::time::Instant::now();
    let roots = world
        .query_filtered::<Entity, With<PendingNativeWorldSceneUnload>>()
        .iter(world)
        .collect::<Vec<_>>();
    let mut remaining = entity_budget;
    // Bound descent too: a pathological deep hierarchy must not spend a whole
    // frame walking ancestors before reaching the first leaf.
    let mut visits = entity_budget.saturating_mul(4);
    for root in roots {
        let Some(mut component) = world.get_mut::<PendingNativeWorldSceneUnload>(root) else {
            continue;
        };
        let mut unload = std::mem::take(&mut *component);
        if !unload.initialized {
            unload.traversal.push(NativeWorldSceneUnloadFrame::new(
                root,
                world.get::<Children>(root),
            ));
            unload.initialized = true;
        }
        for depth in 1..unload.traversal.len() {
            if world
                .get::<ChildOf>(unload.traversal[depth].entity)
                .is_none_or(|parent| parent.parent() != unload.traversal[depth - 1].entity)
            {
                unload.traversal.truncate(depth);
                break;
            }
        }
        while remaining > 0 && visits > 0 {
            // Always make progress, including on unusually slow machines.
            if remaining < entity_budget
                && time_budget.is_some_and(|limit| started.elapsed() >= limit)
            {
                break;
            }
            visits -= 1;
            let Some(frame) = unload.traversal.last_mut() else {
                break;
            };
            if let Some(&child) = frame.children.get(frame.next_child) {
                frame.next_child += 1;
                // A behaviour owner can remove/reparent descendants while a
                // hidden tile retires. A snapshot must not delete a new owner.
                if world
                    .get::<ChildOf>(child)
                    .is_some_and(|parent| parent.parent() == frame.entity)
                {
                    unload.traversal.push(NativeWorldSceneUnloadFrame::new(
                        child,
                        world.get::<Children>(child),
                    ));
                }
                continue;
            }
            let entity = frame.entity;
            // Include late children before retiring their parent; otherwise
            // recursive despawn could silently exceed the actual entity cap.
            if let Some(children) = world.get::<Children>(entity)
                && !children.is_empty()
            {
                *frame = NativeWorldSceneUnloadFrame::new(entity, Some(children));
                continue;
            }
            unload.traversal.pop();
            if world.despawn(entity) {
                remaining -= 1;
            }
        }
        if let Some(mut component) = world.get_mut::<PendingNativeWorldSceneUnload>(root) {
            *component = unload;
        }
        if remaining == 0
            || visits == 0
            || time_budget.is_some_and(|limit| started.elapsed() >= limit)
        {
            break;
        }
    }
}

pub(super) fn orient_native_water_surface_indices_upward(
    vertices: &[Vec3],
    indices: &mut [u32],
) -> Result<(), String> {
    let signed_up_area = indices.chunks_exact(3).try_fold(0.0_f64, |sum, triangle| {
        let a = vertices[triangle[0] as usize];
        let b = vertices[triangle[1] as usize];
        let c = vertices[triangle[2] as usize];
        let up_area = (b - a).cross(c - a).y;
        up_area
            .is_finite()
            .then_some(sum + f64::from(up_area))
            .ok_or_else(|| "authored water collider has non-finite triangle winding".to_owned())
    })?;
    if signed_up_area.abs() <= f64::EPSILON {
        return Err("authored water collider has no horizontal surface area".to_owned());
    }
    if signed_up_area < 0.0 {
        for triangle in indices.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
    }
    Ok(())
}
