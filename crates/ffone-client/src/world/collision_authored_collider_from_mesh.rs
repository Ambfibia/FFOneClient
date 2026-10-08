use super::*;

// cos(50 degrees)
pub(super) const AUTHORED_COLLISION_EPSILON: f32 = 0.000_01;

pub(super) const AUTHORED_COLLISION_CONTACT_TOLERANCE: f32 = 0.000_1;

// The face sweep below is exact for triangle planes, while finite triangle
// edges/corners are recovered by capsule overlap. Keep each horizontal Move
// increment well inside the capsule radius so a complete edge-contact interval
// cannot be skipped between two overlap checks.
pub(super) const AUTHORED_COLLISION_MAX_SUBSTEP: f32 =
    (AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH) * 0.25;

pub(super) const AUTHORED_COLLISION_MAX_SUBSTEPS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeWorldColliderKind {
    TriangleMesh,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldCollider {
    pub name: String,
    pub kind: NativeWorldColliderKind,
    pub model: String,
    pub mesh: usize,
    pub primitive: usize,
    #[serde(default)]
    pub source_model_path: String,
    pub is_trigger: bool,
    #[serde(default = "collider_enabled_by_default")]
    pub enabled: bool,
    pub expected_vertex_count: usize,
    pub expected_index_count: usize,
    pub transform: AuthoredWorldTransform,
}
fn collider_enabled_by_default() -> bool { true }

/// Identifies the published model that owns a non-rendering world collider.
/// Behaviour scripts use this alongside [`SpawnedNativeWorldVisual`] so a
/// kinematic platform moves its exact renderer and triangle collision as one
/// authored object.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SpawnedNativeWorldCollider {
    pub model_path: String,
    pub source_model_path: String,
}

#[derive(Component, Debug, Clone)]
pub(super) struct PendingAuthoredTriMeshCollider {
    pub(super) source_mesh: Handle<Mesh>,
    pub(super) source_model_path: String,
    pub(super) is_trigger: bool,
    pub(super) expected_vertex_count: usize,
    pub(super) expected_index_count: usize,
    pub(super) perimeter_footprint: Option<PendingAuthoredPerimeterFootprint>,
    pub(super) cooking: AuthoredColliderCooking,
}

/// A collider attached to a runtime-owned actor/vehicle rather than a static
/// streamed scene. Its ancestor may move during Update, before Bevy's normal
/// PostUpdate transform propagation.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct RuntimeAttachedAuthoredModelCollider;

/// Previous and current world poses of a collider attached to a moving actor.
///
/// Static authored collision only needs the current [`GlobalTransform`]. A
/// kinematic NPC/vehicle can instead move *into* a stationary player between
/// two frames. Keeping both poses lets the character capsule sweep in the
/// collider's relative frame, matching the contact that Unity/PhysX produced
/// for the source kinematic Rigidbody without turning it into a free body.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct RuntimeAuthoredColliderMotion {
    pub(super) previous_world_from_local: Mat4,
    pub(super) current_world_from_local: Mat4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum AuthoredColliderCooking {
    ExactTriangleMesh,
    /// Normalizes a tagged ffWater/ffPoison sidecar to an upward-facing
    /// one-sided surface. Published water meshes contain both source-order
    /// and final-native-order variants, so a uniform winding reversal is not
    /// valid for this semantic family.
    NativeWaterSurface,
    /// Keeps the validated glTF triangle order for a collider-only sidecar
    /// attached below its retained visual hierarchy. Runtime NPC and vehicle
    /// sidecars are published in their final native handedness; restoring the
    /// static world's source order here would invert their roof and body faces.
    PublishedGltfTriangleMesh,
    /// Builds a continuous vertical shell from the source mesh's XY
    /// silhouette. This is reserved for authored perimeter blockers whose
    /// disconnected panel mesh leaves gameplay-sized holes between segments.
    /// The shell extends one source-height below the mesh so uneven terrain
    /// cannot carry a character capsule underneath its visible lower edge.
    ConvexXyPrism,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeWorldColliderStatus {
    Loading,
    Ready {
        vertex_count: usize,
        index_count: usize,
    },
    Blocked(String),
}

/// CPU-side triangle geometry built from the exact GLB primitive used by the
/// visible world. It remains independent of a specific physics crate.
#[derive(Component, Debug, Clone)]
pub struct AuthoredTriMeshCollider {
    pub(super) source_mesh: Handle<Mesh>,
    pub(super) source_model_path: String,
    pub(super) vertices: Arc<[Vec3]>,
    pub(super) indices: Arc<[u32]>,
    pub(super) local_min: Vec3,
    pub(super) local_max: Vec3,
    pub(super) is_trigger: bool,
}

#[derive(Debug, Clone)]
pub(super) struct AuthoredColliderGeometry {
    pub(super) vertices: Arc<[Vec3]>,
    pub(super) indices: Arc<[u32]>,
    pub(super) local_min: Vec3,
    pub(super) local_max: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct AuthoredColliderGeometryCacheKey {
    pub(super) source_mesh: AssetId<Mesh>,
    pub(super) cooking: AuthoredColliderCooking,
    pub(super) expected_vertex_count: usize,
    pub(super) expected_index_count: usize,
}

#[derive(Resource, Debug, Default)]
pub(super) struct AuthoredColliderGeometryCache {
    pub(super) entries: HashMap<AuthoredColliderGeometryCacheKey, AuthoredColliderGeometry>,
}

/// Cached world-space broad-phase bounds for one authored collider.
///
/// Dense world tiles contain hundreds of collider instances. Rebuilding the
/// eight transformed AABB corners in every player, remote-player, camera and
/// targeting query made that fixed cost multiply by every live actor. The
/// Static bounds are refreshed only when Bevy reports a changed global
/// transform. The small runtime-attached vehicle/NPC subset is refreshed after
/// its Update motion so exact triangle tests see the current frame's pose.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AuthoredColliderWorldBounds {
    pub(super) minimum: Vec3,
    pub(super) maximum: Vec3,
}

impl AuthoredColliderWorldBounds {
    /// Bounds used by the opt-in GM collision overlay; no collision mutation.
    pub fn debug_bounds(&self) -> (Vec3, Vec3) {
        (self.minimum, self.maximum)
    }
    pub(super) fn from_collider(collider: &AuthoredTriMeshCollider, global: &GlobalTransform) -> Option<Self> {
        collider_world_bounds(collider, global.to_matrix())
            .map(|(minimum, maximum)| Self { minimum, maximum })
    }

    pub(super) fn overlaps(&self, query_minimum: Vec3, query_maximum: Vec3) -> bool {
        aabb_overlaps(self.minimum, self.maximum, query_minimum, query_maximum)
    }
}

// Unity PhysX never linearly tested every MeshCollider in all nine resident
// dongs for each actor query. Keep the exact cooked triangles and placements,
// but recover that broad-phase property with a deterministic XZ grid. Static
// entries change only while tiles stream; the small moving-collider subset is
// reindexed after its current-frame transform is resolved.
pub(super) const AUTHORED_COLLIDER_SPATIAL_CELL_SIZE: f32 = 32.0;

pub(super) const AUTHORED_COLLIDER_MAX_INDEXED_CELLS: usize = 4_096;

#[derive(Resource, Debug, Default)]
pub(super) struct AuthoredColliderSpatialIndex {
    pub(super) cells: HashMap<IVec2, Vec<Entity>>,
    pub(super) memberships: HashMap<Entity, AuthoredColliderSpatialMembership>,
    pub(super) oversized: Vec<Entity>,
}

#[derive(Debug)]
pub(super) enum AuthoredColliderSpatialMembership {
    Cells(Vec<IVec2>),
    Oversized,
}

impl AuthoredColliderSpatialIndex {
    pub(super) fn cell(value: f32) -> i32 {
        (value / AUTHORED_COLLIDER_SPATIAL_CELL_SIZE).floor() as i32
    }

    pub(super) fn remove(&mut self, entity: Entity) {
        let Some(membership) = self.memberships.remove(&entity) else {
            return;
        };
        match membership {
            AuthoredColliderSpatialMembership::Cells(cells) => {
                for cell in cells {
                    let remove_cell = if let Some(entities) = self.cells.get_mut(&cell) {
                        entities.retain(|candidate| *candidate != entity);
                        entities.is_empty()
                    } else {
                        false
                    };
                    if remove_cell {
                        self.cells.remove(&cell);
                    }
                }
            }
            AuthoredColliderSpatialMembership::Oversized => {
                self.oversized.retain(|candidate| *candidate != entity);
            }
        }
    }

    pub(super) fn insert(&mut self, entity: Entity, bounds: &AuthoredColliderWorldBounds) {
        self.remove(entity);
        let minimum = IVec2::new(Self::cell(bounds.minimum.x), Self::cell(bounds.minimum.z));
        let maximum = IVec2::new(Self::cell(bounds.maximum.x), Self::cell(bounds.maximum.z));
        let width = i64::from(maximum.x) - i64::from(minimum.x) + 1;
        let height = i64::from(maximum.y) - i64::from(minimum.y) + 1;
        let cell_count = width.saturating_mul(height);
        if width <= 0
            || height <= 0
            || usize::try_from(cell_count)
                .map_or(true, |count| count > AUTHORED_COLLIDER_MAX_INDEXED_CELLS)
        {
            self.oversized.push(entity);
            self.memberships
                .insert(entity, AuthoredColliderSpatialMembership::Oversized);
            return;
        }
        let mut membership = Vec::with_capacity(cell_count as usize);
        for z in minimum.y..=maximum.y {
            for x in minimum.x..=maximum.x {
                let cell = IVec2::new(x, z);
                self.cells.entry(cell).or_default().push(entity);
                membership.push(cell);
            }
        }
        self.memberships
            .insert(entity, AuthoredColliderSpatialMembership::Cells(membership));
    }

    pub(super) fn candidates(&self, query_minimum: Vec3, query_maximum: Vec3, output: &mut Vec<Entity>) {
        output.clear();
        let minimum = IVec2::new(Self::cell(query_minimum.x), Self::cell(query_minimum.z));
        let maximum = IVec2::new(Self::cell(query_maximum.x), Self::cell(query_maximum.z));
        for z in minimum.y..=maximum.y {
            for x in minimum.x..=maximum.x {
                if let Some(entities) = self.cells.get(&IVec2::new(x, z)) {
                    output.extend_from_slice(entities);
                }
            }
        }
        output.extend_from_slice(&self.oversized);
        output.sort_unstable_by_key(|entity| entity.to_bits());
        output.dedup();
    }

    pub(super) fn point_candidates(&self, x: f32, z: f32, output: &mut Vec<Entity>) {
        self.candidates(Vec3::new(x, 0.0, z), Vec3::new(x, 0.0, z), output);
    }
}

impl AuthoredTriMeshCollider {
    #[must_use]
    pub fn source_mesh(&self) -> &Handle<Mesh> {
        &self.source_mesh
    }

    #[must_use]
    pub fn source_model_path(&self) -> &str {
        &self.source_model_path
    }

    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    #[must_use]
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    #[must_use]
    pub fn is_trigger(&self) -> bool {
        self.is_trigger
    }
}

pub(super) fn has_legacy_collision_helper_identity(
    visual: &NativeWorldVisual,
    published_model_path: Option<&str>,
) -> bool {
    let name = visual.name.to_ascii_lowercase();
    let semantic_name = name.contains("collision_alpha")
        && (name.starts_with("collision_") || name.contains("-collision_"));
    let published_path = published_model_path.is_some_and(|path| {
        path.to_ascii_lowercase()
            .contains("objects/collision/default_collision_alpha/")
    });
    semantic_name || published_path
}

pub(super) fn is_legacy_collision_helper_visual(scene: &NativeWorldScene, visual: &NativeWorldVisual) -> bool {
    has_legacy_collision_helper_identity(
        visual,
        scene.model(&visual.model).map(|model| model.path.as_str()),
    )
}

pub(super) fn native_world_collider_cooking(root_name: &str) -> AuthoredColliderCooking {
    match root_name {
        // Published water sidecars contain both source-order and final-native-
        // order variants. Normalize their horizontal surface upward so a
        // front-face-only character sweep stays in water contact and preserves
        // the terrain-attribute gate which owns infected-zone damage.
        "ffWater" | "ffPoison" => AuthoredColliderCooking::NativeWaterSurface,
        _ => AuthoredColliderCooking::ExactTriangleMesh,
    }
}

#[derive(Debug)]
pub(super) struct AuthoredColliderCookBudget {
    pub(super) remaining_count: usize,
    pub(super) remaining_vertices: usize,
    pub(super) remaining_indices: usize,
    pub(super) admitted: usize,
}

impl Default for AuthoredColliderCookBudget {
    fn default() -> Self {
        Self {
            remaining_count: AUTHORED_COLLIDER_COOKS_PER_FRAME,
            remaining_vertices: AUTHORED_COLLIDER_VERTICES_PER_FRAME,
            remaining_indices: AUTHORED_COLLIDER_INDICES_PER_FRAME,
            admitted: 0,
        }
    }
}

impl AuthoredColliderCookBudget {
    pub(super) fn admit(&mut self, vertices: usize, indices: usize) -> bool {
        if self.remaining_count == 0 {
            return false;
        }
        let fits_payload = vertices <= self.remaining_vertices && indices <= self.remaining_indices;
        if self.admitted > 0 && !fits_payload {
            return false;
        }
        self.remaining_count -= 1;
        self.remaining_vertices = self.remaining_vertices.saturating_sub(vertices);
        self.remaining_indices = self.remaining_indices.saturating_sub(indices);
        self.admitted += 1;
        true
    }
}

pub(super) fn authored_collider_from_cached_geometry(
    pending: &PendingAuthoredTriMeshCollider,
    geometry: &AuthoredColliderGeometry,
) -> AuthoredTriMeshCollider {
    AuthoredTriMeshCollider {
        source_mesh: pending.source_mesh.clone(),
        source_model_path: pending.source_model_path.clone(),
        vertices: geometry.vertices.clone(),
        indices: geometry.indices.clone(),
        local_min: geometry.local_min,
        local_max: geometry.local_max,
        is_trigger: pending.is_trigger,
    }
}

pub(super) fn finish_authored_collider_materialization(
    commands: &mut Commands,
    entity: Entity,
    global: &GlobalTransform,
    collider: AuthoredTriMeshCollider,
) {
    let Some(world_bounds) = AuthoredColliderWorldBounds::from_collider(&collider, global) else {
        commands
            .entity(entity)
            .remove::<PendingAuthoredTriMeshCollider>()
            .insert(NativeWorldColliderStatus::Blocked(
                "authored collider has non-finite world bounds".to_owned(),
            ));
        return;
    };
    let status = NativeWorldColliderStatus::Ready {
        vertex_count: collider.vertex_count(),
        index_count: collider.index_count(),
    };
    commands
        .entity(entity)
        .remove::<PendingAuthoredTriMeshCollider>()
        .insert((collider, world_bounds, status));
}

pub(super) fn sync_authored_collider_world_bounds(
    mut commands: Commands,
    changed: Query<
        (Entity, &GlobalTransform, &AuthoredTriMeshCollider),
        (
            Or<(Added<AuthoredTriMeshCollider>, Changed<GlobalTransform>)>,
            Without<RuntimeAttachedAuthoredModelCollider>,
        ),
    >,
) {
    for (entity, global, collider) in &changed {
        if let Some(bounds) = AuthoredColliderWorldBounds::from_collider(collider, global) {
            commands.entity(entity).insert(bounds);
        } else {
            commands
                .entity(entity)
                .remove::<AuthoredColliderWorldBounds>();
        }
    }
}

pub(super) fn sync_authored_collider_spatial_index(
    mut index: ResMut<AuthoredColliderSpatialIndex>,
    changed: Query<
        (Entity, &AuthoredColliderWorldBounds),
        Or<(
            Added<AuthoredColliderWorldBounds>,
            Changed<AuthoredColliderWorldBounds>,
        )>,
    >,
    mut removed: RemovedComponents<AuthoredColliderWorldBounds>,
) {
    for entity in removed.read() {
        index.remove(entity);
    }
    for (entity, bounds) in &changed {
        index.insert(entity, bounds);
    }
}

pub(super) fn sync_runtime_attached_authored_collider_transforms(
    mut commands: Commands,
    transforms: TransformHelper,
    mut colliders: Query<
        (
            Entity,
            &mut GlobalTransform,
            &AuthoredTriMeshCollider,
            Option<&mut RuntimeAuthoredColliderMotion>,
        ),
        // Scripted world colliders are authored scene entities, not runtime
        // model attachments. Their dynamic marker must opt them into the
        // same current-Update pose used for support carry and collision.
        Or<(
            With<RuntimeAttachedAuthoredModelCollider>,
            With<NativeWorldDynamicObjectRange>,
        )>,
    >,
) {
    for (entity, mut cached_global, collider, motion) in &mut colliders {
        let Ok(current_global) = transforms.compute_global_transform(entity) else {
            commands
                .entity(entity)
                .remove::<(AuthoredColliderWorldBounds, RuntimeAuthoredColliderMotion)>();
            continue;
        };
        // At the start of Update this is the pose propagated at the end of the
        // preceding frame, including the GLTF body's stand animation. The
        // helper independently composes the root motion written earlier in
        // this Update, so these are the two endpoints needed by a relative
        // kinematic sweep.
        let previous_world_from_local = cached_global.to_matrix();
        let current_world_from_local = current_global.to_matrix();
        *cached_global = current_global;
        if let Some(bounds) = AuthoredColliderWorldBounds::from_collider(collider, &current_global)
        {
            commands.entity(entity).insert(bounds);
        } else {
            commands
                .entity(entity)
                .remove::<AuthoredColliderWorldBounds>();
        }
        if let Some(mut motion) = motion {
            motion.previous_world_from_local = previous_world_from_local;
            motion.current_world_from_local = current_world_from_local;
        } else {
            // A newly cooked collider has no trustworthy preceding pose. Use
            // its first resolved pose for both endpoints so it cannot push an
            // avatar merely because asset materialization finished nearby.
            commands
                .entity(entity)
                .insert(RuntimeAuthoredColliderMotion {
                    previous_world_from_local: current_world_from_local,
                    current_world_from_local,
                });
        }
    }
}

pub(super) fn authored_collider_from_mesh(
    pending: &PendingAuthoredTriMeshCollider,
    mesh: &Mesh,
    footprint_mesh: Option<&Mesh>,
) -> Result<AuthoredTriMeshCollider, String> {
    let vertices = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values
            .iter()
            .map(|value| Vec3::from_array(*value))
            .collect::<Vec<_>>(),
        Some(_) => return Err("authored collider POSITION must be Float32x3".to_owned()),
        None => return Err("authored collider mesh has no POSITION attribute".to_owned()),
    };
    // Bevy generates flat normals for indexed glTF primitives that do not
    // author normals. That process deliberately expands every indexed corner
    // into a vertex and removes the index buffer. Collider-only GLBs use that
    // compact source representation, so accept this one exact loader
    // normalization while retaining the authored cardinality checks below.
    let bevy_flat_normal_expansion = mesh.indices().is_none()
        && vertices.len() == pending.expected_index_count
        && pending.expected_vertex_count <= pending.expected_index_count;
    if vertices.len() != pending.expected_vertex_count && !bevy_flat_normal_expansion {
        return Err(format!(
            "authored collider vertex count mismatch: expected {}, got {}",
            pending.expected_vertex_count,
            vertices.len()
        ));
    }
    if vertices.iter().any(|vertex| !vertex.is_finite()) {
        return Err("authored collider contains a non-finite vertex".to_owned());
    }

    let mut indices = match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|&value| u32::from(value)).collect(),
        Some(Indices::U32(values)) => values.clone(),
        None if bevy_flat_normal_expansion => (0..vertices.len() as u32).collect(),
        None => return Err("authored collider mesh has no triangle indices".to_owned()),
    };
    if indices.len() != pending.expected_index_count || !indices.len().is_multiple_of(3) {
        return Err(format!(
            "authored collider index count mismatch: expected {}, got {}",
            pending.expected_index_count,
            indices.len()
        ));
    }
    if indices
        .iter()
        .any(|&index| index as usize >= vertices.len())
    {
        return Err("authored collider index is outside the vertex array".to_owned());
    }

    let (vertices, indices) = match pending.cooking {
        AuthoredColliderCooking::ExactTriangleMesh => {
            // Published GLBs swap Unity's clockwise indices to glTF CCW for
            // GPU front-face culling. PhysX CharacterController sweeps use
            // the source MeshCollider winding and do not enable
            // eMESH_BOTH_SIDES, so restore that source order for collision.
            for triangle in indices.chunks_exact_mut(3) {
                triangle.swap(1, 2);
            }
            (vertices, indices)
        }
        AuthoredColliderCooking::NativeWaterSurface => {
            orient_native_water_surface_indices_upward(&vertices, &mut indices)?;
            (vertices, indices)
        }
        AuthoredColliderCooking::PublishedGltfTriangleMesh => (vertices, indices),
        AuthoredColliderCooking::ConvexXyPrism => {
            let footprint = pending.perimeter_footprint.as_ref().ok_or_else(|| {
                "authored perimeter collider has no visible footprint metadata".to_owned()
            })?;
            let footprint_mesh = footprint_mesh.ok_or_else(|| {
                "authored perimeter collider visible footprint is not loaded".to_owned()
            })?;
            if !footprint.local_scale.is_finite() || footprint.local_scale.min_element() <= 0.0 {
                return Err(
                    "authored perimeter footprint scale must be finite and positive".into(),
                );
            }
            let footprint_vertices = match footprint_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                Some(VertexAttributeValues::Float32x3(values)) => values,
                Some(_) => {
                    return Err(
                        "authored perimeter footprint POSITION must be Float32x3".to_owned()
                    );
                }
                None => {
                    return Err("authored perimeter footprint has no POSITION attribute".to_owned());
                }
            };
            if footprint_vertices.len() != footprint.expected_vertex_count
                || footprint_vertices
                    .iter()
                    .any(|vertex| !Vec3::from_array(*vertex).is_finite())
            {
                return Err(format!(
                    "authored perimeter footprint vertex count/data mismatch: expected {}, got {}",
                    footprint.expected_vertex_count,
                    footprint_vertices.len()
                ));
            }
            let footprint_index_count = match footprint_mesh.indices() {
                Some(Indices::U16(values)) => values.len(),
                Some(Indices::U32(values)) => values.len(),
                None => 0,
            };
            if footprint_index_count != footprint.expected_index_count {
                return Err(format!(
                    "authored perimeter footprint index count mismatch: expected {}, got {}",
                    footprint.expected_index_count, footprint_index_count
                ));
            }
            let footprint_points = footprint_vertices
                .iter()
                .map(|vertex| {
                    Vec2::new(
                        vertex[0] * footprint.local_scale.x,
                        vertex[1] * footprint.local_scale.y,
                    )
                })
                .collect::<Vec<_>>();
            cook_convex_xy_prism(&vertices, &footprint_points)?
        }
    };

    let (local_min, local_max) = vertices.iter().copied().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(minimum, maximum), vertex| (minimum.min(vertex), maximum.max(vertex)),
    );
    if !local_min.is_finite() || !local_max.is_finite() {
        return Err("authored collider has no finite local bounds".to_owned());
    }

    Ok(AuthoredTriMeshCollider {
        source_mesh: pending.source_mesh.clone(),
        source_model_path: pending.source_model_path.clone(),
        vertices: vertices.into(),
        indices: indices.into(),
        local_min,
        local_max,
        is_trigger: pending.is_trigger,
    })
}

#[must_use]
pub(super) fn advance_network_npc_grounding(
    current_y: f32,
    ground_y: Option<f32>,
    vertical_velocity: f32,
    delta_seconds: f32,
) -> (f32, f32) {
    const GRAVITY: f32 = 10.0;
    const TERMINAL_FALL_SPEED: f32 = -10.0;

    if !current_y.is_finite()
        || !vertical_velocity.is_finite()
        || !delta_seconds.is_finite()
        || delta_seconds <= 0.0
    {
        return (current_y, vertical_velocity);
    }
    if let Some(ground_y) = ground_y.filter(|height| height.is_finite()) {
        if ground_y >= current_y {
            return (ground_y, 0.0);
        }
        let next_velocity = (vertical_velocity - GRAVITY * delta_seconds).max(TERMINAL_FALL_SPEED);
        let next_y = current_y + next_velocity * delta_seconds;
        if next_y <= ground_y {
            return (ground_y, 0.0);
        }
        return (next_y, next_velocity);
    }

    let next_velocity = (vertical_velocity - GRAVITY * delta_seconds).max(TERMINAL_FALL_SPEED);
    (current_y + next_velocity * delta_seconds, next_velocity)
}

#[cfg(test)]
pub(super) fn legacy_collision_flag_for_normal(normal: Vec3) -> u8 {
    if normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT {
        LEGACY_COLLISION_BELOW
    } else if normal.y <= -AUTHORED_WALKABLE_MIN_UP_DOT {
        crate::movement::LEGACY_COLLISION_ABOVE
    } else {
        crate::movement::LEGACY_COLLISION_SIDES
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AuthoredCollisionTriangle {
    pub(super) a: Vec3,
    pub(super) b: Vec3,
    pub(super) c: Vec3,
    pub(super) normal: Vec3,
    pub(super) minimum: Vec3,
    pub(super) maximum: Vec3,
    pub(super) obstacle_maximum_y: f32,
}

impl AuthoredCollisionTriangle {
    pub(super) fn new(a: Vec3, b: Vec3, c: Vec3) -> Option<Self> {
        let normal = (b - a).cross(c - a).normalize_or_zero();
        if !a.is_finite()
            || !b.is_finite()
            || !c.is_finite()
            || !normal.is_finite()
            || normal == Vec3::ZERO
        {
            return None;
        }
        let minimum = a.min(b).min(c);
        let maximum = a.max(b).max(c);
        Some(Self {
            a,
            b,
            c,
            normal,
            minimum,
            maximum,
            obstacle_maximum_y: maximum.y,
        })
    }

    pub(super) fn with_obstacle_maximum_y(mut self, obstacle_maximum_y: f32) -> Self {
        self.obstacle_maximum_y = obstacle_maximum_y;
        self
    }

    pub(super) fn overlaps(self, minimum: Vec3, maximum: Vec3) -> bool {
        self.minimum.cmple(maximum).all() && self.maximum.cmpge(minimum).all()
    }
}
