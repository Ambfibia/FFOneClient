use super::*;

#[must_use]
pub fn select_native_world_scene(
    catalog: &NativeWorldCatalog,
    native_position: Vec3,
) -> Option<&NativeWorldScene> {
    catalog.select(native_position)
}

pub(super) fn matches_primary_audited_smallstuff_presentation_identity(
    legacy_layer: Option<i64>,
    model_path: &str,
) -> bool {
    legacy_layer == Some(12)
        && (matches_primary_darktree_bridge_model_family(model_path)
            || model_path == PRIMARY_ETC_TREE_07_FROND_MODEL_PATH
            || model_path == PRIMARY_ETC_TREE_07_TRUNK_MODEL_PATH)
}

pub(super) fn quantize_native_world_range(value: f32, minimum: f32, step: f32, bits: u32) -> Option<u32> {
    let maximum_code = (1_u32 << bits) - 1;
    if !value.is_finite() || value < minimum || value >= minimum + step * (maximum_code + 1) as f32
    {
        return None;
    }
    Some(((value - minimum) / step).floor() as u32)
}

/// A center-distance fade must not reach geometry inside the object itself.
/// Tile-spanning rails can extend beyond the capped radius even while the
/// camera is standing on their near end. Use the decoded center here so the
/// packing error cannot move that transition back inside the actual bounds.
pub(super) fn native_world_bounds_fit_center_range(minimum: Vec3, maximum: Vec3, tag: u32) -> bool {
    let (center, end) = unpack_native_world_object_range(tag);
    let farthest = (minimum - center).abs().max((maximum - center).abs());
    farthest.length() + NATIVE_WORLD_OBJECT_FADE_BAND < end
}

pub(super) fn native_world_pipeline_sentinel_range() -> VisibilityRange {
    VisibilityRange {
        start_margin: 0.0..0.0,
        // This non-abrupt, post-clip transition only enables Bevy's dither
        // shader variant. WGSL ignores its per-mesh LOD-table center and uses
        // the packed group center above instead.
        end_margin: EXTENDED_WORLD_CAMERA_FAR_NATIVE..(EXTENDED_WORLD_CAMERA_FAR_NATIVE + 1.0),
        use_aabb: true,
    }
}

pub(super) fn native_world_range_physical_visibility(
    presentation_ready: bool,
    contract_visible: bool,
    material_failed: bool,
    runtime_managed: bool,
) -> Visibility {
    if presentation_ready && contract_visible && !material_failed && !runtime_managed {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

pub(super) fn is_legacy_zero_contribution_visual(
    scene: &NativeWorldScene,
    visual: &NativeWorldVisual,
) -> bool {
    // Exact audited primary material
    // CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a:1847 is
    // Blend One One, ZWrite Off, has no MainTex and contributes black RGB.
    // It is mathematically invisible in Unity. Its StandardMaterial fallback
    // is instead an opaque black plane, so never request presentation for this
    // one frozen geometry identity. Requiring id, route and accepted bytes
    // avoids the unsafe historical practice of filtering objects named "NO".
    scene.model(&visual.model).is_some_and(|model| {
        model.id == LEGACY_ZERO_CONTRIBUTION_MODEL_ID
            && model.path == LEGACY_ZERO_CONTRIBUTION_MODEL_PATH
            && model.blake3 == LEGACY_ZERO_CONTRIBUTION_MODEL_BLAKE3
    })
}

pub(super) fn is_legacy_non_presenting_visual(scene: &NativeWorldScene, visual: &NativeWorldVisual) -> bool {
    is_legacy_collision_helper_visual(scene, visual)
        || is_legacy_zero_contribution_visual(scene, visual)
}

pub(super) fn native_world_water_surface(
    scene: &NativeWorldScene,
    visual: &NativeWorldVisual,
) -> Option<LegacyWaterSurface> {
    let model = scene.model(&visual.model)?;
    let infected = match model.root_name.as_str() {
        "ffWater" if is_canonical_native_world_water_model_path(&model.path, "ffwater") => false,
        "ffPoison" if is_canonical_native_world_water_model_path(&model.path, "ffpoison") => true,
        _ => return None,
    };
    Some(LegacyWaterSurface { infected })
}

pub(super) fn is_native_world_water_visual(scene: &NativeWorldScene, visual: &NativeWorldVisual) -> bool {
    native_world_water_surface(scene, visual).is_some()
}

pub(super) fn materialize_loaded_native_world_visual_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    meshes: Res<Assets<Mesh>>,
    mut aabb_cache: ResMut<NativeWorldMeshAabbCache>,
    group_roots: Query<&NativeWorldObjectRangeGroups>,
    unloading_roots: Query<(), With<PendingNativeWorldSceneUnload>>,
    mut status: ResMut<NativeWorldStreamingStatus>,
    pending_visuals: Query<(
        Entity,
        &PendingNativeWorldVisualAsset,
        &SpawnedNativeWorldVisual,
        &NativeWorldObjectRangeMember,
    )>,
) {
    let mut remaining = NATIVE_WORLD_VISUAL_FINALIZATIONS_PER_FRAME;
    let mut failed_roots = HashSet::new();
    for (entity, pending, visual, range_member) in &pending_visuals {
        if remaining == 0 {
            break;
        }
        if failed_roots.contains(&range_member.scene_root)
            || unloading_roots.contains(range_member.scene_root)
        {
            continue;
        }
        let Some(gltf_mesh) = gltf_meshes.get(&pending.mesh) else {
            if let LoadState::Failed(error) = asset_server.load_state(pending.mesh.id()) {
                status.blocker = Some(format!(
                    "native world visual {} failed to load: {error}",
                    visual.model_path
                ));
                failed_roots.insert(range_member.scene_root);
                queue_native_world_scene_unload(&mut commands, range_member.scene_root);
            }
            continue;
        };
        let Some(gltf) = gltfs.get(&pending.gltf) else {
            if let LoadState::Failed(error) = asset_server.load_state(pending.gltf.id()) {
                status.blocker = Some(format!(
                    "native world visual {} root failed to load: {error}",
                    visual.model_path
                ));
                failed_roots.insert(range_member.scene_root);
                queue_native_world_scene_unload(&mut commands, range_member.scene_root);
            }
            continue;
        };
        let Some(primitive) = (gltf_mesh.index == 0 && gltf_mesh.primitives.len() == 1)
            .then(|| &gltf_mesh.primitives[0])
        else {
            status.blocker = Some(format!(
                "native world visual {} violated the one-mesh/one-primitive fast-path contract",
                visual.model_path
            ));
            failed_roots.insert(range_member.scene_root);
            queue_native_world_scene_unload(&mut commands, range_member.scene_root);
            continue;
        };
        let Some(material) = primitive.material.clone() else {
            status.blocker = Some(format!(
                "native world visual {} has no material",
                visual.model_path
            ));
            failed_roots.insert(range_member.scene_root);
            queue_native_world_scene_unload(&mut commands, range_member.scene_root);
            continue;
        };
        let mesh_id = primitive.mesh.id();
        let aabb = if let Some(aabb) = aabb_cache.entries.get(&mesh_id).copied() {
            aabb
        } else {
            let Some(mesh) = meshes.get(&primitive.mesh) else {
                // GltfMesh publication and its primitive Mesh dependency can
                // become visible on adjacent asset events. Do not consume the
                // finalization budget until both are resident.
                if let LoadState::Failed(error) = asset_server.load_state(mesh_id) {
                    status.blocker = Some(format!(
                        "native world visual {} mesh failed to load: {error}",
                        visual.model_path
                    ));
                    failed_roots.insert(range_member.scene_root);
                    queue_native_world_scene_unload(&mut commands, range_member.scene_root);
                }
                continue;
            };
            let Some(aabb) = mesh.compute_aabb() else {
                status.blocker = Some(format!(
                    "native world visual {} has no finite Float32x3 POSITION bounds",
                    visual.model_path
                ));
                failed_roots.insert(range_member.scene_root);
                queue_native_world_scene_unload(&mut commands, range_member.scene_root);
                continue;
            };
            aabb_cache.entries.insert(mesh_id, aabb);
            aabb
        };
        remaining -= 1;
        let material_name = gltf
            .named_materials
            .iter()
            .find(|(_, handle)| handle.id() == material.id())
            .map(|(name, _)| name.to_string());

        let permanently_unbounded = group_roots
            .get(range_member.scene_root)
            .is_ok_and(|groups| groups.unbounded.contains(&range_member.group));
        // Bevy's PBR glTF extension publishes the immutable StandardMaterial
        // alongside GltfMaterial under the same label with a `/std` suffix.
        let Some(material_path) = material.path() else {
            status.blocker = Some(format!(
                "native world visual {} material has no asset path",
                visual.model_path
            ));
            failed_roots.insert(range_member.scene_root);
            queue_native_world_scene_unload(&mut commands, range_member.scene_root);
            continue;
        };
        let standard_material: Handle<StandardMaterial> =
            asset_server.load(material_path.clone().with_label(format!(
                "{}/std",
                material_path.label().unwrap_or("DefaultMaterial")
            )));
        let mut entity_commands = commands.entity(entity);
        entity_commands
            .try_remove::<PendingNativeWorldVisualAsset>()
            .try_insert((
                Mesh3d(primitive.mesh.clone()),
                MeshMaterial3d(standard_material),
                aabb,
                GltfMeshName(gltf_mesh.name.clone()),
                NativeWorldVisualSceneReady,
            ));
        if permanently_unbounded {
            entity_commands
                .try_insert((NativeWorldObjectRangeReady, NativeWorldObjectRangeUnbounded));
        } else {
            entity_commands.try_insert(PendingNativeWorldObjectBounds::default());
        }
        if let Some(extras) = &primitive.extras {
            entity_commands.try_insert(GltfExtras {
                value: extras.value.clone(),
            });
        }
        if let Some(extras) = &gltf_mesh.extras {
            entity_commands.try_insert(GltfMeshExtras {
                value: extras.value.clone(),
            });
        }
        if let Some(extras) = &primitive.material_extras {
            entity_commands.try_insert(GltfMaterialExtras {
                value: extras.value.clone(),
            });
        }
        if let Some(name) = material_name {
            entity_commands.try_insert(GltfMaterialName(name));
        }
    }
}

/// Starts the playable slice through the same bounded admission queue used by
/// neighboring streamed tiles.
///
/// The legacy entry path used to request every visual and collider in the
/// selected tile synchronously. Dense dongs can contain several thousand
/// placements, producing a single-frame CPU/asset-server burst before the
/// loading overlay had a chance to render. Terrain is decoded first so the
/// ground can materialize immediately; all GLB requests then share the normal
/// per-frame and in-flight budgets.
pub fn begin_native_world_scene_admission(
    commands: &mut Commands,
    catalog: &NativeWorldCatalog,
    scene: &NativeWorldScene,
    selection_scope: NativeWorldScope,
) -> Result<Entity, NativeWorldSceneError> {
    scene.validate_with_policy(false)?;
    if !catalog
        .scene_for_tile(selection_scope, scene.tile)
        .is_some_and(|selected| selected.name == scene.name && selected.scope == scene.scope)
    {
        return Err(NativeWorldSceneError::new(format!(
            "scene {:?} at {:?} is not selected by requested scope {selection_scope:?}",
            scene.name, scene.tile
        )));
    }
    commands.insert_resource(NativeWorldRequestedScope(selection_scope));
    begin_streamed_native_world_scene_loading(commands, catalog, scene, selection_scope)
}

pub(super) fn begin_streamed_native_world_scene_loading(
    commands: &mut Commands,
    catalog: &NativeWorldCatalog,
    scene: &NativeWorldScene,
    selection_scope: NativeWorldScope,
) -> Result<Entity, NativeWorldSceneError> {
    let already_loaded = scene
        .native_terrain
        .as_ref()
        .and_then(|instance| instance.loaded.clone());
    let root = begin_streamed_native_world_scene(
        commands,
        scene,
        already_loaded.clone(),
        selection_scope,
    )?;
    if let Some(instance) = scene.native_terrain.as_ref() {
        if already_loaded.is_none() {
            let asset_root = catalog.asset_root.clone();
            let terrain_cache = Arc::clone(&catalog.terrain_cache);
            let instance = instance.clone();
            let task = IoTaskPool::get().spawn(async move {
                load_native_world_terrain(&asset_root, &terrain_cache, &instance)
            });
            commands
                .entity(root)
                .try_insert(PendingNativeWorldSceneTerrainLoad { task });
        }
    }
    Ok(root)
}

pub(super) fn begin_streamed_native_world_scene(
    commands: &mut Commands,
    scene: &NativeWorldScene,
    terrain: Option<Arc<NativeTerrain>>,
    selection_scope: NativeWorldScope,
) -> Result<Entity, NativeWorldSceneError> {
    let root = spawn_native_world_scene_root(commands, scene, selection_scope)?;
    commands
        .entity(root)
        .insert(PendingNativeWorldSceneSpawn::new(terrain));
    Ok(root)
}

pub(super) fn materialize_pending_native_world_scene_spawns(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<NativeWorldCatalog>,
    mut status: ResMut<NativeWorldStreamingStatus>,
    pending_visuals: Query<(), With<PendingNativeWorldVisualAsset>>,
    pending_colliders: Query<(), With<PendingAuthoredTriMeshCollider>>,
    mut pending_roots: Query<(
        Entity,
        &NativeWorldSceneRoot,
        &mut PendingNativeWorldSceneSpawn,
        Option<&mut PendingNativeWorldSceneTerrainLoad>,
    )>,
) {
    // Several neighboring terrains may be resident while their much heavier
    // static scenes are still being admitted. Keep the documented budgets
    // global per frame rather than multiplying them by the number of roots.
    let mut remaining_visual_budget = NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME;
    let mut remaining_collider_budget = NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME;
    // Count actual outstanding requests across ALL roots, including roots
    // whose admission just completed. Ready objects and hierarchy changes do
    // not require a scan of thousands of already admitted children each frame.
    let mut remaining_in_flight_visual_slots =
        NATIVE_WORLD_MAX_IN_FLIGHT_VISUAL_ASSETS.saturating_sub(pending_visuals.iter().len());
    let mut remaining_pending_collider_slots =
        NATIVE_WORLD_MAX_PENDING_COLLIDERS.saturating_sub(pending_colliders.iter().len());
    let mut terrain_spawn_available = true;
    for (root, scene_root, mut pending, terrain_load) in &mut pending_roots {
        let Some(scene) = catalog.scenes.iter().find(|scene| {
            scene.name == scene_root.name
                && scene.tile == scene_root.tile
                && scene.scope == Some(scene_root.scope)
        }) else {
            status.blocker = Some(format!(
                "streamed tile {:?} ({}) disappeared from the native world catalog",
                scene_root.tile, scene_root.name
            ));
            queue_native_world_scene_unload(&mut commands, root);
            continue;
        };

        if !pending.terrain_spawned && scene.native_terrain.is_some() && pending.terrain.is_none() {
            let Some(mut terrain_load) = terrain_load else {
                status.blocker = Some(format!(
                    "native dong stream lost the terrain task for {}",
                    scene.name
                ));
                queue_native_world_scene_unload(&mut commands, root);
                continue;
            };
            let Some(result) = block_on(future::poll_once(&mut terrain_load.task)) else {
                continue;
            };
            match result {
                Ok(terrain) => {
                    pending.terrain = Some(terrain);
                    commands
                        .entity(root)
                        .remove::<PendingNativeWorldSceneTerrainLoad>();
                }
                Err(error) => {
                    status.blocker = Some(format!(
                        "native dong terrain failed for {}: {error}",
                        scene.name
                    ));
                    queue_native_world_scene_unload(&mut commands, root);
                    continue;
                }
            }
        }

        if !pending.terrain_spawned {
            if scene.native_terrain.is_some() {
                if !terrain_spawn_available {
                    continue;
                }
                terrain_spawn_available = false;
            }
            let terrain_result = match (&scene.native_terrain, pending.terrain.take()) {
                (Some(_), Some(terrain)) => {
                    spawn_native_world_terrain(&mut commands, scene, root, terrain).map(|_| ())
                }
                (Some(_), None) => Err(NativeWorldSceneError::new(
                    "streamed tile terrain finished without a decoded payload",
                )),
                (None, _) => Ok(()),
            };
            if let Err(error) = terrain_result {
                status.blocker = Some(format!(
                    "native dong stream failed for {}: {error}",
                    scene.name
                ));
                queue_native_world_scene_unload(&mut commands, root);
                continue;
            }
            pending.terrain_spawned = true;
        }

        // A per-frame request limit alone is insufficient: while GLBs are on
        // disk, pending requests can accumulate and all become ready together.
        // Bound direct-mesh finalization pressure independently, and apply the
        // equivalent cap to the uncooked collider queue.
        let visual_budget = remaining_in_flight_visual_slots.min(remaining_visual_budget);
        let collider_budget = remaining_pending_collider_slots.min(remaining_collider_budget);
        let (visuals, colliders) = pending.next_batch(
            scene.visuals.len(),
            scene.colliders.len(),
            visual_budget,
            collider_budget,
        );
        remaining_visual_budget = remaining_visual_budget.saturating_sub(visuals.len());
        remaining_collider_budget = remaining_collider_budget.saturating_sub(colliders.len());
        remaining_in_flight_visual_slots =
            remaining_in_flight_visual_slots.saturating_sub(visuals.len());
        remaining_pending_collider_slots =
            remaining_pending_collider_slots.saturating_sub(colliders.len());
        let spawn_result = (|| {
            let visual_start = visuals.start;
            for (offset, visual) in scene.visuals[visuals].iter().enumerate() {
                spawn_native_world_visual(
                    &mut commands,
                    &asset_server,
                    scene,
                    root,
                    visual_start + offset,
                    visual,
                )?;
            }
            for collider in &scene.colliders[colliders] {
                spawn_native_world_collider(&mut commands, &asset_server, scene, root, collider)?;
            }
            Ok::<(), NativeWorldSceneError>(())
        })();
        if let Err(error) = spawn_result {
            status.blocker = Some(format!(
                "native dong stream failed for {}: {error}",
                scene.name
            ));
            queue_native_world_scene_unload(&mut commands, root);
            continue;
        }

        if pending.is_complete(scene.visuals.len(), scene.colliders.len()) {
            commands
                .entity(root)
                .remove::<PendingNativeWorldSceneSpawn>();
        }
    }
}

pub(super) fn transformed_aabb_bounds(minimum: Vec3, maximum: Vec3, transform: Mat4) -> Option<(Vec3, Vec3)> {
    let mut output_minimum = Vec3::splat(f32::INFINITY);
    let mut output_maximum = Vec3::splat(f32::NEG_INFINITY);
    for x in [minimum.x, maximum.x] {
        for y in [minimum.y, maximum.y] {
            for z in [minimum.z, maximum.z] {
                let point = transform.transform_point3(Vec3::new(x, y, z));
                if !point.is_finite() {
                    return None;
                }
                output_minimum = output_minimum.min(point);
                output_maximum = output_maximum.max(point);
            }
        }
    }
    Some((output_minimum, output_maximum))
}

/// Scripted platforms, animations and billboards reparent authored visual
/// roots below moving pivots. Their one-time packed world center would become
/// stale immediately, so the complete logical group permanently falls back to
/// ordinary frustum + camera-far visibility. A metadata error likewise removes
/// the complete group from the tag path, but permanently hides only the visual
/// that owns the invalid fallback material. This is event-driven: settled
/// static objects pay no scan cost.
pub(super) fn make_unsafe_native_world_range_groups_unbounded(
    mut commands: Commands,
    dynamic_members: Query<&NativeWorldObjectRangeMember, Added<NativeWorldDynamicObjectRange>>,
    metadata_errors: Query<
        (Entity, &LegacyMaterialMetadataError),
        Added<LegacyMaterialMetadataError>,
    >,
    members: Query<(
        Entity,
        &NativeWorldObjectRangeMember,
        Option<&NativeWorldObjectRangeContract>,
        Option<&RuntimeManagedNativeWorldVisual>,
    )>,
    visuals: Query<&SpawnedNativeWorldVisual>,
    parents: Query<&ChildOf>,
    presentation_roots: Query<&NativeWorldVisualPresentationStatus, With<NativeWorldSceneRoot>>,
    mut group_roots: Query<&mut NativeWorldObjectRangeGroups>,
) {
    let mut dynamic_groups = dynamic_members
        .iter()
        .map(|member| (member.scene_root, member.group.clone()))
        .collect::<HashSet<_>>();
    let mut failed_visuals = HashSet::new();
    for (error_entity, error) in &metadata_errors {
        let mut ancestor = error_entity;
        loop {
            if let Ok((_, member, _, _)) = members.get(ancestor) {
                if let Ok(visual) = visuals.get(ancestor) {
                    warn!(
                        "native static visual material failed closed: model={} source={} error={}",
                        visual.model_path, visual.source_model_path, error.0
                    );
                }
                let group = (member.scene_root, member.group.clone());
                failed_visuals.insert(ancestor);
                dynamic_groups.insert(group);
                break;
            }
            let Ok(parent) = parents.get(ancestor) else {
                break;
            };
            ancestor = parent.parent();
        }
    }
    if dynamic_groups.is_empty() {
        return;
    }

    // Removing the pending aggregate is the permanent part of the fallback:
    // a range traversal that finishes after the reparent cannot recreate a
    // stale contract on a later frame.
    for (scene_root, key) in &dynamic_groups {
        if let Ok(mut groups) = group_roots.get_mut(*scene_root) {
            groups.pending.remove(key);
            groups.ready_to_finalize.retain(|queued| queued != key);
            groups.unbounded.insert(key.clone());
        }
    }

    for (visual_root, member, contract, runtime_managed) in &members {
        let group = (member.scene_root, member.group.clone());
        if !dynamic_groups.contains(&group) {
            continue;
        }
        // One material failure makes the shared range group unbounded, but
        // only the renderer that owns the bad metadata is fail-closed. Healthy
        // siblings of the same multipart prefab remain present, matching the
        // reveal pass's existing per-visual failure contract.
        let material_failed = failed_visuals.contains(&visual_root);
        let presentation_ready = presentation_roots
            .get(member.scene_root)
            .is_ok_and(|status| *status == NativeWorldVisualPresentationStatus::Ready);
        let mut visual_commands = commands.entity(visual_root);
        visual_commands
            .try_remove::<(
                PendingNativeWorldObjectBounds,
                NativeWorldObjectRangeContract,
            )>()
            .try_insert((
                NativeWorldObjectRangeReady,
                NativeWorldObjectRangeUnbounded,
                native_world_range_physical_visibility(
                    presentation_ready,
                    true,
                    material_failed,
                    runtime_managed.is_some(),
                ),
            ));
        if material_failed {
            visual_commands.try_insert(NativeWorldMaterialPresentationFailed);
        }

        // Every tagged source or material companion is retained in the
        // contract. If aggregation never completed there is nothing to clear;
        // late passes see Unbounded in the Added<Mesh3d> retry system.
        if let Some(contract) = contract {
            for entity in &contract.meshes {
                commands
                    .entity(*entity)
                    .try_remove::<(MeshTag, VisibilityRange, PendingNativeWorldObjectRangeTag)>()
                    .try_insert(native_world_range_physical_visibility(
                        // Only the root owns loading/gameplay visibility. Once
                        // unbounded, descendants have no residency update to
                        // clear a copied Hidden flag when the scene is revealed.
                        presentation_ready || *entity != visual_root,
                        true,
                        material_failed,
                        runtime_managed.is_some() && *entity == visual_root,
                    ));
            }
        }
    }
}

pub(super) fn mark_native_world_visual_scene_ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    visuals: Query<Option<&NativeWorldObjectRangeMember>, With<SpawnedNativeWorldVisual>>,
    group_roots: Query<&NativeWorldObjectRangeGroups>,
) {
    let entity = event.event().entity;
    if let Ok(range_member) = visuals.get(entity) {
        let mut entity_commands = commands.entity(entity);
        entity_commands.try_insert(NativeWorldVisualSceneReady);
        if let Some(range_member) = range_member {
            let permanently_unbounded = group_roots
                .get(range_member.scene_root)
                .is_ok_and(|groups| groups.unbounded.contains(&range_member.group));
            if permanently_unbounded {
                entity_commands
                    .try_insert((NativeWorldObjectRangeReady, NativeWorldObjectRangeUnbounded));
            } else {
                entity_commands.try_insert(PendingNativeWorldObjectBounds::default());
            }
        }
    }
}

pub(super) fn native_world_descendant(
    mut entity: Entity,
    ancestor: Entity,
    parents: &Query<&ChildOf>,
) -> bool {
    while let Ok(parent) = parents.get(entity) {
        entity = parent.parent();
        if entity == ancestor {
            return true;
        }
    }
    false
}

pub(super) fn native_world_ancestor_in_roots(
    mut entity: Entity,
    roots: &HashSet<Entity>,
    parents: &Query<&ChildOf>,
) -> Option<Entity> {
    if roots.contains(&entity) {
        return Some(entity);
    }
    while let Ok(parent) = parents.get(entity) {
        entity = parent.parent();
        if roots.contains(&entity) {
            return Some(entity);
        }
    }
    None
}

#[cfg(test)]
pub(super) fn native_world_presentation_ready<'a>(
    visual_readiness: impl IntoIterator<Item = bool>,
    material_readiness: impl IntoIterator<Item = bool>,
    collider_statuses: impl IntoIterator<Item = &'a NativeWorldColliderStatus>,
) -> bool {
    visual_readiness.into_iter().all(|ready| ready)
        && material_readiness.into_iter().all(|ready| ready)
        && collider_statuses
            .into_iter()
            .all(|status| matches!(status, NativeWorldColliderStatus::Ready { .. }))
}
