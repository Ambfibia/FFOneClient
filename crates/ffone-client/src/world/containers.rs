use super::*;

pub(super) const NATIVE_WORLD_RANGE_ENTITY_VISITS_PER_OBJECT: usize = 32;

pub(super) const NATIVE_WORLD_MIN_OBJECT_RANGE: f32 = 75.0;

pub(super) const NATIVE_WORLD_OBJECT_FADE_BAND: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeSerializedPointer {
    pub file_id: i64,
    pub path_id: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldRootChainGameObject {
    pub asset_name: String,
    pub path_id: i64,
    pub true_name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldSerializedTransform {
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum NativeWorldObjectRangeGroupKey {
    /// One legacy prefab can publish many sibling renderer roots. They must
    /// share one center/range or buildings and effects still tear apart.
    Prefab(Arc<str>),
    /// Ordinary BuildPlayer renderers are independent objects even when their
    /// authored transforms happen to coincide.
    Visual(usize),
}

#[derive(Component, Debug, Clone)]
pub(super) struct NativeWorldObjectRangeMember {
    pub(super) scene_root: Entity,
    pub(super) group: NativeWorldObjectRangeGroupKey,
}

/// Per-tile admission state. Completed groups are removed immediately, so a
/// resident tile does not retain every temporary traversal or mesh list twice.
#[derive(Component, Debug)]
pub(super) struct NativeWorldObjectRangeGroups {
    pub(super) pending: HashMap<NativeWorldObjectRangeGroupKey, NativeWorldPendingRangeGroup>,
    pub(super) ready_to_finalize: VecDeque<NativeWorldObjectRangeGroupKey>,
    pub(super) unbounded: HashSet<NativeWorldObjectRangeGroupKey>,
}

impl NativeWorldObjectRangeGroups {
    pub(super) fn from_scene(scene: &NativeWorldScene) -> Self {
        let mut pending = HashMap::<_, NativeWorldPendingRangeGroup>::new();
        let mut unbounded = HashSet::new();
        for (visual_index, visual) in scene.visuals.iter().enumerate() {
            if is_legacy_non_presenting_visual(scene, visual) {
                continue;
            }
            let group = native_world_object_range_group_key(visual_index, &visual.name);
            if is_native_world_water_visual(scene, visual) {
                // Unity renders a water mesh from its actual renderer bounds.
                // The native adaptive range is center-based, which can hide a
                // neighboring 512-unit surface while its near edge is still
                // in front of the camera. Loaded water therefore keeps normal
                // Bevy AABB/frustum/far-plane culling and skips only that
                // extra center-distance contract.
                unbounded.insert(group);
                continue;
            }
            pending
                .entry(group)
                .and_modify(|entry| entry.expected_members += 1)
                .or_insert_with(|| NativeWorldPendingRangeGroup {
                    expected_members: 1,
                    members: Vec::new(),
                });
        }
        Self {
            pending,
            ready_to_finalize: VecDeque::new(),
            unbounded,
        }
    }
}

#[derive(Component, Debug, Default)]
pub(super) struct PendingNativeWorldObjectBounds {
    pub(super) traversal: VecDeque<Entity>,
    pub(super) initialized: bool,
    pub(super) minimum: Option<Vec3>,
    pub(super) maximum: Option<Vec3>,
    pub(super) meshes: Vec<Entity>,
}

/// Shared contract copied to every sibling visual root in one logical object.
/// Its Mesh3d list is member-local so CPU residency changes touch only actual
/// renderables and preserve any scripted visibility on the visual root.
#[derive(Component, Debug, Clone)]
pub(super) struct NativeWorldObjectRangeContract {
    pub(super) tag: u32,
    pub(super) meshes: Vec<Entity>,
    pub(super) visible: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldObjectRangeReady;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldObjectRangeUnbounded;

/// Sparse signal from behaviour materialization that an authored visual is
/// actually driven at runtime (billboard, platform or active TRS animation).
/// Static hierarchy organization must not opt almost half the map out of the
/// adaptive range budget merely because it also reparents renderer roots.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeWorldDynamicObjectRange;

/// A material/outline pass appeared before its logical object's aggregation
/// completed. Retain it for retry; `Added<Mesh3d>` alone is a one-frame edge.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PendingNativeWorldObjectRangeTag;

#[derive(Resource, Debug, Default)]
pub(super) struct NativeWorldObjectRangeCameraCache {
    pub(super) last_position: Option<Vec3>,
}

pub(super) fn native_world_prefab_owner(name: &str) -> Option<&str> {
    let owner_start = name.find("[prefab:")? + "[prefab:".len();
    let owner_tail = &name[owner_start..];
    let owner_end = owner_tail.find(':')?;
    let owner = &owner_tail[..owner_end];
    (!owner.is_empty()).then_some(owner)
}

pub(super) fn native_world_object_range_group_key(
    visual_index: usize,
    name: &str,
) -> NativeWorldObjectRangeGroupKey {
    native_world_prefab_owner(name).map_or(
        NativeWorldObjectRangeGroupKey::Visual(visual_index),
        |owner| NativeWorldObjectRangeGroupKey::Prefab(Arc::from(owner)),
    )
}

/// Packs the exact center used by both CPU residency and WGSL fade. Published
/// centers fit X [-8189,-191], Z [256,8179], Y [-17993,597]. Cell-center
/// decoding therefore has <=8/8/10-unit axis error (<=15.1 units in 3D),
/// safely inside the 32-unit transition band.
pub(super) fn pack_native_world_object_range(center: Vec3, end: f32) -> Option<u32> {
    let x = quantize_native_world_range(center.x, -8192.0, 16.0, NATIVE_WORLD_RANGE_X_BITS)?;
    let z = quantize_native_world_range(center.z, 0.0, 16.0, NATIVE_WORLD_RANGE_Z_BITS)?;
    let y = quantize_native_world_range(center.y, -18000.0, 20.0, NATIVE_WORLD_RANGE_Y_BITS)?;
    if !end.is_finite() {
        return None;
    }
    let end_code = (((end.clamp(
        NATIVE_WORLD_MIN_OBJECT_RANGE,
        EXTENDED_WORLD_CAMERA_FAR_NATIVE,
    ) - NATIVE_WORLD_MIN_OBJECT_RANGE)
        / (EXTENDED_WORLD_CAMERA_FAR_NATIVE - NATIVE_WORLD_MIN_OBJECT_RANGE)
        * NATIVE_WORLD_RANGE_END_LEVELS)
        .round() as u32)
        .min((1_u32 << NATIVE_WORLD_RANGE_END_BITS) - 1);
    Some(
        (x << (NATIVE_WORLD_RANGE_Z_BITS
            + NATIVE_WORLD_RANGE_Y_BITS
            + NATIVE_WORLD_RANGE_END_BITS))
            | (z << (NATIVE_WORLD_RANGE_Y_BITS + NATIVE_WORLD_RANGE_END_BITS))
            | (y << NATIVE_WORLD_RANGE_END_BITS)
            | end_code,
    )
}

pub(super) fn unpack_native_world_object_range(tag: u32) -> (Vec3, f32) {
    let end_mask = (1_u32 << NATIVE_WORLD_RANGE_END_BITS) - 1;
    let y_mask = (1_u32 << NATIVE_WORLD_RANGE_Y_BITS) - 1;
    let z_mask = (1_u32 << NATIVE_WORLD_RANGE_Z_BITS) - 1;
    let end_code = tag & end_mask;
    let y = (tag >> NATIVE_WORLD_RANGE_END_BITS) & y_mask;
    let z = (tag >> (NATIVE_WORLD_RANGE_END_BITS + NATIVE_WORLD_RANGE_Y_BITS)) & z_mask;
    let x = tag
        >> (NATIVE_WORLD_RANGE_END_BITS + NATIVE_WORLD_RANGE_Y_BITS + NATIVE_WORLD_RANGE_Z_BITS);
    let center = Vec3::new(
        -8192.0 + (x as f32 + 0.5) * 16.0,
        -18000.0 + (y as f32 + 0.5) * 20.0,
        (z as f32 + 0.5) * 16.0,
    );
    let end = NATIVE_WORLD_MIN_OBJECT_RANGE
        + end_code as f32 / NATIVE_WORLD_RANGE_END_LEVELS
            * (EXTENDED_WORLD_CAMERA_FAR_NATIVE - NATIVE_WORLD_MIN_OBJECT_RANGE);
    (center, end)
}

pub(super) fn native_world_object_range(radius: f32) -> f32 {
    (radius * NATIVE_WORLD_RANGE_PER_RADIUS).clamp(
        NATIVE_WORLD_MIN_OBJECT_RANGE,
        EXTENDED_WORLD_CAMERA_FAR_NATIVE,
    )
}

pub(super) fn prepare_native_world_object_ranges(
    mut commands: Commands,
    transforms: TransformHelper,
    hierarchy: Query<(Option<&Children>, Option<&Mesh3d>, Option<&Aabb>)>,
    cameras: Query<Entity, (With<Camera3d>, With<LegacyOrbitCamera>)>,
    presentation_roots: Query<&NativeWorldVisualPresentationStatus, With<NativeWorldSceneRoot>>,
    unloading_roots: Query<(), With<PendingNativeWorldSceneUnload>>,
    mut pending_members: Query<(
        Entity,
        &NativeWorldObjectRangeMember,
        &mut PendingNativeWorldObjectBounds,
    )>,
    mut group_roots: Query<(Entity, &mut NativeWorldObjectRangeGroups)>,
    member_states: Query<
        (
            Option<&NativeWorldObjectRangeUnbounded>,
            Option<&NativeWorldDynamicObjectRange>,
            Option<&RuntimeManagedNativeWorldVisual>,
        ),
        With<NativeWorldObjectRangeMember>,
    >,
    primary_far_guards: Query<(), With<NativeWorldPrimaryFarPresentationGuard>>,
    metadata_errors: Query<(), With<LegacyMaterialMetadataError>>,
) {
    let mut completed = Vec::new();
    let mut remaining = NATIVE_WORLD_RANGE_ENTITY_VISITS_PER_FRAME;
    for (visual_root, member, mut pending) in &mut pending_members {
        if unloading_roots.get(member.scene_root).is_ok() {
            commands
                .entity(visual_root)
                .try_remove::<PendingNativeWorldObjectBounds>();
            continue;
        }
        if group_roots
            .get_mut(member.scene_root)
            .is_ok_and(|(_, groups)| groups.unbounded.contains(&member.group))
        {
            commands
                .entity(visual_root)
                .try_remove::<PendingNativeWorldObjectBounds>()
                .try_insert((NativeWorldObjectRangeReady, NativeWorldObjectRangeUnbounded));
            continue;
        }
        if remaining == 0 {
            break;
        }
        if !pending.initialized {
            pending.traversal.push_back(visual_root);
            pending.initialized = true;
        }
        let Ok(visual_global) = transforms.compute_global_transform(visual_root) else {
            continue;
        };
        let visual_from_world = visual_global.to_matrix().inverse();
        let mut object_budget = NATIVE_WORLD_RANGE_ENTITY_VISITS_PER_OBJECT.min(remaining);
        let mut waiting_for_bounds = false;
        while object_budget > 0 {
            let Some(entity) = pending.traversal.pop_front() else {
                break;
            };
            object_budget -= 1;
            remaining -= 1;
            let Ok((children, mesh, aabb)) = hierarchy.get(entity) else {
                continue;
            };
            if mesh.is_some() {
                let (Some(aabb), Ok(mesh_global)) =
                    (aabb, transforms.compute_global_transform(entity))
                else {
                    pending.traversal.push_front(entity);
                    waiting_for_bounds = true;
                    break;
                };
                let mesh_to_visual = visual_from_world * mesh_global.to_matrix();
                let local_minimum = Vec3::from(aabb.center - aabb.half_extents);
                let local_maximum = Vec3::from(aabb.center + aabb.half_extents);
                let Some((minimum, maximum)) =
                    transformed_aabb_bounds(local_minimum, local_maximum, mesh_to_visual)
                else {
                    pending.traversal.push_front(entity);
                    waiting_for_bounds = true;
                    break;
                };
                pending.minimum = Some(pending.minimum.map_or(minimum, |value| value.min(minimum)));
                pending.maximum = Some(pending.maximum.map_or(maximum, |value| value.max(maximum)));
                pending.meshes.push(entity);
            }
            if let Some(children) = children {
                pending.traversal.extend(children.iter());
            }
        }
        if !waiting_for_bounds && pending.traversal.is_empty() {
            let minimum = pending.minimum.unwrap_or(Vec3::ZERO);
            let maximum = pending.maximum.unwrap_or(Vec3::ZERO);
            completed.push((
                member.scene_root,
                member.group.clone(),
                NativeWorldPreparedRangeMember {
                    visual_root,
                    local_minimum: minimum,
                    local_maximum: maximum,
                    meshes: std::mem::take(&mut pending.meshes),
                },
            ));
        }
    }

    let camera_position = cameras
        .iter()
        .next()
        .and_then(|camera| transforms.compute_global_transform(camera).ok())
        .map(|global| global.translation());
    for (scene_root, key, member) in completed {
        commands
            .entity(member.visual_root)
            .try_remove::<PendingNativeWorldObjectBounds>();
        let Ok((_, mut groups)) = group_roots.get_mut(scene_root) else {
            continue;
        };
        let Some(group) = groups.pending.get_mut(&key) else {
            continue;
        };
        if group
            .members
            .iter()
            .any(|existing| existing.visual_root == member.visual_root)
        {
            continue;
        }
        group.members.push(member);
        if group.members.len() == group.expected_members {
            groups.ready_to_finalize.push_back(key);
        }
    }

    // Finalization is a persistent phase, separate from the traversal that
    // produced each member's local bounds. Transform propagation can lag
    // WorldInstanceReady by a frame; retaining a complete group here makes
    // that ordinary race retry instead of hiding its tile forever. The queue
    // also avoids rescanning every incomplete group in every resident tile.
    let mut finalizations_remaining = NATIVE_WORLD_RANGE_GROUP_FINALIZATIONS_PER_FRAME;
    for (scene_root, mut groups) in &mut group_roots {
        if finalizations_remaining == 0 {
            break;
        }
        // Only retry entries that were queued before this root's pass. A
        // missing Transform is placed at the back for a future Update rather
        // than spinning inside this one.
        let attempts = groups
            .ready_to_finalize
            .len()
            .min(NATIVE_WORLD_RANGE_GROUP_FINALIZATIONS_PER_ROOT)
            .min(finalizations_remaining);
        for _ in 0..attempts {
            finalizations_remaining -= 1;
            let Some(key) = groups.ready_to_finalize.pop_front() else {
                break;
            };
            let Some(group) = groups.pending.remove(&key) else {
                continue;
            };

            // Explicitly runtime-driven members already have a moving world
            // center. A source material metadata failure likewise retains
            // Bevy's fallback shader, which cannot consume this tag contract.
            let unsafe_group = group.members.iter().any(|member| {
                member_states
                    .get(member.visual_root)
                    .map_or(true, |(unbounded, dynamic, _)| {
                        unbounded.is_some() || dynamic.is_some()
                    })
                    || member
                        .meshes
                        .iter()
                        .any(|mesh| metadata_errors.get(*mesh).is_ok())
            });
            if unsafe_group {
                groups.unbounded.insert(key);
                for member in group.members {
                    commands
                        .entity(member.visual_root)
                        .try_insert((NativeWorldObjectRangeReady, NativeWorldObjectRangeUnbounded));
                }
                continue;
            }

            let mut world_minimum = Vec3::splat(f32::INFINITY);
            let mut world_maximum = Vec3::splat(f32::NEG_INFINITY);
            let mut transform_ready = true;
            for member in &group.members {
                let Ok(global) = transforms.compute_global_transform(member.visual_root) else {
                    transform_ready = false;
                    break;
                };
                let Some((minimum, maximum)) = transformed_aabb_bounds(
                    member.local_minimum,
                    member.local_maximum,
                    global.to_matrix(),
                ) else {
                    transform_ready = false;
                    break;
                };
                world_minimum = world_minimum.min(minimum);
                world_maximum = world_maximum.max(maximum);
            }
            if !transform_ready {
                groups.pending.insert(key.clone(), group);
                groups.ready_to_finalize.push_back(key);
                continue;
            }
            let center = (world_minimum + world_maximum) * 0.5;
            let radius = (world_maximum - world_minimum).length() * 0.5;
            let Some(tag) =
                pack_native_world_object_range(center, native_world_object_range(radius)).filter(
                    |tag| native_world_bounds_fit_center_range(world_minimum, world_maximum, *tag),
                )
            else {
                // Out-of-envelope centers and objects that contain their own
                // fade boundary retain ordinary mesh-AABB/frustum/far culling.
                // In particular, City Station rails span more than 340 units
                // from their center; center culling hides their walkable ends.
                groups.unbounded.insert(key);
                for member in group.members {
                    commands
                        .entity(member.visual_root)
                        .try_insert((NativeWorldObjectRangeReady, NativeWorldObjectRangeUnbounded));
                }
                continue;
            };
            let (decoded_center, decoded_end) = unpack_native_world_object_range(tag);
            let presentation_ready = presentation_roots
                .get(scene_root)
                .is_ok_and(|status| *status == NativeWorldVisualPresentationStatus::Ready);
            for member in group.members {
                let runtime_managed = member_states
                    .get(member.visual_root)
                    .is_ok_and(|(_, _, runtime_managed)| runtime_managed.is_some());
                let presentation_end = if primary_far_guards.get(member.visual_root).is_ok() {
                    PRIMARY_AUDITED_SMALLSTUFF_PRESENTATION_END_NATIVE
                } else {
                    decoded_end
                };
                let visible = camera_position
                    .is_none_or(|camera| camera.distance(decoded_center) < presentation_end);
                // Loading and gameplay gate the root. Its passes inherit that
                // gate even if reveal happens without another residency sample.
                for mesh in &member.meshes {
                    commands.entity(*mesh).try_insert((
                        MeshTag(tag),
                        native_world_pipeline_sentinel_range(),
                        native_world_range_physical_visibility(
                            presentation_ready || *mesh != member.visual_root,
                            visible,
                            false,
                            runtime_managed && *mesh == member.visual_root,
                        ),
                    ));
                }
                commands.entity(member.visual_root).try_insert((
                    NativeWorldObjectRangeContract {
                        tag,
                        meshes: member.meshes,
                        visible,
                    },
                    NativeWorldObjectRangeReady,
                ));
            }
        }
    }
}

/// Outline companions and other late material passes are spawned after the
/// source scene is ready. Give them the exact same tag/sentinel without
/// touching their real AABB, so Bevy frustum culling remains per mesh.
pub(super) fn propagate_native_world_object_ranges_to_late_meshes(
    mut commands: Commands,
    meshes: Query<
        Entity,
        (
            With<Mesh3d>,
            Or<(Added<Mesh3d>, With<PendingNativeWorldObjectRangeTag>)>,
        ),
    >,
    parents: Query<&ChildOf>,
    mut contracts: Query<&mut NativeWorldObjectRangeContract>,
    range_members: Query<(), With<NativeWorldObjectRangeMember>>,
    unbounded: Query<(), With<NativeWorldObjectRangeUnbounded>>,
    material_failures: Query<(), With<NativeWorldMaterialPresentationFailed>>,
) {
    for mesh_entity in &meshes {
        let mut ancestor = mesh_entity;
        while let Ok(parent) = parents.get(ancestor) {
            ancestor = parent.parent();
            if let Ok(mut contract) = contracts.get_mut(ancestor) {
                if !contract.meshes.contains(&mesh_entity) {
                    contract.meshes.push(mesh_entity);
                }
                commands.entity(mesh_entity).try_insert((
                    MeshTag(contract.tag),
                    native_world_pipeline_sentinel_range(),
                    // The gameplay owner toggles the visual root. Descendant
                    // passes must inherit that gate, not retain a second hidden
                    // flag after a race pod (or another managed visual) appears.
                    if contract.visible {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ));
                commands
                    .entity(mesh_entity)
                    .try_remove::<PendingNativeWorldObjectRangeTag>();
                break;
            }
            if range_members.get(ancestor).is_ok() {
                if unbounded.get(ancestor).is_ok() {
                    commands
                        .entity(mesh_entity)
                        .try_remove::<(MeshTag, VisibilityRange, PendingNativeWorldObjectRangeTag)>(
                        )
                        .try_insert(
                            if material_failures.get(ancestor).is_ok() {
                                Visibility::Hidden
                            } else {
                                Visibility::Inherited
                            },
                        );
                } else {
                    commands
                        .entity(mesh_entity)
                        .try_insert(PendingNativeWorldObjectRangeTag);
                }
                break;
            }
        }
    }
}

pub(super) fn update_native_world_object_residency(
    mut commands: Commands,
    transforms: TransformHelper,
    cameras: Query<Entity, (With<Camera3d>, With<LegacyOrbitCamera>)>,
    presentation_roots: Query<&NativeWorldVisualPresentationStatus, With<NativeWorldSceneRoot>>,
    unloading_roots: Query<(), With<PendingNativeWorldSceneUnload>>,
    primary_far_guards: Query<(), With<NativeWorldPrimaryFarPresentationGuard>>,
    mesh_visibilities: Query<&Visibility>,
    mut cache: ResMut<NativeWorldObjectRangeCameraCache>,
    mut contracts: ParamSet<(
        Query<Entity, Added<NativeWorldObjectRangeContract>>,
        Query<(
            Entity,
            &NativeWorldObjectRangeMember,
            Option<&NativeWorldMaterialPresentationFailed>,
            Option<&RuntimeManagedNativeWorldVisual>,
            &mut NativeWorldObjectRangeContract,
        )>,
    )>,
) {
    let Some(camera_position) = cameras
        .iter()
        .next()
        .and_then(|camera| transforms.compute_global_transform(camera).ok())
        .map(|global| global.translation())
    else {
        return;
    };
    let moved = cache.last_position.is_none_or(|previous| {
        previous.distance_squared(camera_position)
            >= NATIVE_WORLD_RANGE_CAMERA_STEP * NATIVE_WORLD_RANGE_CAMERA_STEP
    });
    let added = contracts.p0().iter().collect::<Vec<_>>();
    if !moved && added.is_empty() {
        return;
    }
    if moved {
        cache.last_position = Some(camera_position);
    }
    let targets = (!moved).then_some(added.into_iter().collect::<HashSet<_>>());
    for (entity, member, material_failed, runtime_managed, mut contract) in &mut contracts.p1() {
        // Incremental unload keeps the root and part of its hierarchy alive
        // across several Updates. Do not spend the residency budget updating
        // renderers that are already committed to destruction.
        if unloading_roots.get(member.scene_root).is_ok() {
            continue;
        }
        if targets
            .as_ref()
            .is_some_and(|targets| !targets.contains(&entity))
        {
            continue;
        }
        let (center, adaptive_end) = unpack_native_world_object_range(contract.tag);
        let end = if primary_far_guards.get(entity).is_ok() {
            PRIMARY_AUDITED_SMALLSTUFF_PRESENTATION_END_NATIVE
        } else {
            adaptive_end
        };
        let visible = camera_position.distance(center) < end;
        if visible != contract.visible {
            contract.visible = visible;
        }
        let presentation_ready = presentation_roots
            .get(member.scene_root)
            .is_ok_and(|status| *status == NativeWorldVisualPresentationStatus::Ready);
        for mesh in &contract.meshes {
            let desired_visibility = native_world_range_physical_visibility(
                presentation_ready || *mesh != entity,
                visible,
                material_failed.is_some(),
                runtime_managed.is_some() && *mesh == entity,
            );
            // Dense nine-tile neighborhoods contain thousands of contracts.
            // Re-inserting the same value at every camera residency sample marked
            // every renderer Changed<Visibility>, forcing Bevy's propagation,
            // extraction and render preparation to revisit the complete
            // resident set. Queue only a real state transition instead.
            //
            // A world transition may already have queued this mesh for
            // recursive despawn. Keep the existence-tolerant deferred command:
            // an entity removed before apply is a stale cancelled residency
            // target, not a fatal invariant. The read-only query also avoids
            // globally serializing this system with unrelated visibility
            // readers in UI, effects and animation.
            if mesh_visibilities
                .get(*mesh)
                .is_ok_and(|current| *current == desired_visibility)
            {
                continue;
            }
            commands.entity(*mesh).try_insert(desired_visibility);
        }
    }
}
