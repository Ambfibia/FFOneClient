use super::*;

pub(super) const NATIVE_TERRAIN_CACHE_CAPACITY: usize = 9;

pub(super) const NATIVE_WORLD_TERRAIN_LOADS_IN_FLIGHT: usize = 2;

pub(super) const TERRAIN_RECOVERY_VERTICAL_LIMIT: f32 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTerrainSpatialLookup {
    Found(Entity),
    Missing,
    DuplicateKey { dong_key: i32, entities: usize },
    NonFinite,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTerrainSpatialRegistryStatus {
    Ready { dong_key: i32 },
    DuplicateKey { dong_key: i32, entities: usize },
}

#[derive(Resource, Debug, Default)]
pub struct NativeTerrainSpatialRegistry {
    pub(super) key_entities: HashMap<i32, HashSet<Entity>>,
    pub(super) entity_keys: HashMap<Entity, i32>,
}

impl NativeTerrainSpatialRegistry {
    #[must_use]
    pub fn lookup(&self, world_x: f32, world_z: f32) -> NativeTerrainSpatialLookup {
        let Some(dong_key) = native_dong_key(world_x, world_z) else {
            return NativeTerrainSpatialLookup::NonFinite;
        };
        match self.key_entities.get(&dong_key) {
            None => NativeTerrainSpatialLookup::Missing,
            Some(entities) if entities.len() == 1 => {
                NativeTerrainSpatialLookup::Found(*entities.iter().next().unwrap())
            }
            Some(entities) => NativeTerrainSpatialLookup::DuplicateKey {
                dong_key,
                entities: entities.len(),
            },
        }
    }

    pub(super) fn remove_entity(&mut self, entity: Entity) -> Option<i32> {
        let key = self.entity_keys.remove(&entity)?;
        if let Some(entities) = self.key_entities.get_mut(&key) {
            entities.remove(&entity);
            if entities.is_empty() {
                self.key_entities.remove(&key);
            }
        }
        Some(key)
    }

    pub(super) fn insert_entity(&mut self, entity: Entity, key: i32) {
        self.remove_entity(entity);
        self.entity_keys.insert(entity, key);
        self.key_entities.entry(key).or_default().insert(entity);
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainRenderSerializedFields {
    #[serde(rename = "m_CastShadows")]
    pub m_cast_shadows: i64,
    #[serde(rename = "m_HeightmapMaximumLOD")]
    pub m_heightmap_maximum_lod: i64,
    #[serde(rename = "m_HeightmapPixelError")]
    pub m_heightmap_pixel_error: f64,
    #[serde(rename = "m_RenderMode")]
    pub m_render_mode: i64,
    #[serde(rename = "m_SplatMapDistance")]
    pub m_splat_map_distance: f64,
    #[serde(rename = "m_DetailObjectDistance")]
    pub m_detail_object_distance: f64,
    #[serde(rename = "m_TreeBillboardDistance")]
    pub m_tree_billboard_distance: f64,
    #[serde(rename = "m_TreeCrossFadeLength")]
    pub m_tree_cross_fade_length: f64,
    #[serde(rename = "m_TreeDistance")]
    pub m_tree_distance: f64,
    #[serde(rename = "m_TreeMaximumFullLODCount")]
    pub m_tree_maximum_full_lod_count: i64,
    #[serde(rename = "m_UseLightmap")]
    pub m_use_lightmap: i64,
    #[serde(default, rename = "m_BasemapDistance")]
    pub m_basemap_distance: Option<f64>,
    #[serde(default, rename = "m_DebugDrawMainCamera")]
    pub m_debug_draw_main_camera: Option<i64>,
    #[serde(default, rename = "m_Enabled")]
    pub m_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainRenderContract {
    pub selection: String,
    pub serialized_fields: NativeTerrainRenderSerializedFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneMapReference {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneTerrainData {
    pub true_name: String,
    pub source_asset_name: String,
    pub source_path_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneGameObject {
    pub asset_name: String,
    pub path_id: i64,
    pub true_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainResolvedPointer {
    pub file_id: i64,
    pub path_id: i64,
    pub resolved_asset_name: String,
    pub resolved_path_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainColliderContract {
    pub terrain_data_pointer: NativeSerializedPointer,
    pub create_tree_colliders: bool,
    pub is_trigger: bool,
    pub material_pointer: NativeSerializedPointer,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneCollider {
    pub asset_name: String,
    pub path_id: i64,
    pub contract: NativeTerrainColliderContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneTransformLink {
    pub asset_name: String,
    pub path_id: i64,
    pub parent_transform_path_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneComponent {
    pub asset_name: String,
    pub path_id: i64,
    pub terrain_data_pointer: NativeSerializedPointer,
    pub render_contract: NativeTerrainRenderContract,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneLinkage {
    pub terrain_collider: NativeTerrainSceneCollider,
    pub terrain_data_pointer: NativeTerrainResolvedPointer,
    pub game_object: NativeTerrainSceneGameObject,
    pub transform: NativeTerrainSceneTransformLink,
    pub terrain_component: NativeTerrainSceneComponent,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneCoordinateContract {
    pub space: String,
    pub basis: String,
    pub unit_scale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSidecarReference {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainRawSidecarReference {
    pub path: String,
    pub byte_length: usize,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainOwnerScriptEvidence {
    pub fact: String,
    pub sha256: String,
    pub workspace_path: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainOwnerComponent {
    pub asset_name: String,
    pub class_id: i64,
    pub component_index: usize,
    pub component_pointer: NativeSerializedPointer,
    pub parsed_data: NativeTerrainSidecarReference,
    pub path_id: i64,
    pub raw_data: NativeTerrainRawSidecarReference,
    pub role: String,
    pub script_pointer: Option<NativeSerializedPointer>,
    pub script_true_name: Option<String>,
    pub type_id: i64,
    pub type_name: String,
    #[serde(default)]
    pub script_evidence: Option<NativeTerrainOwnerScriptEvidence>,
    #[serde(default)]
    pub gameplay_attributes: Option<NativeTerrainGameplayAttributes>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneInstance {
    pub schema: String,
    pub tile_id: String,
    pub terrain_data: NativeTerrainSceneTerrainData,
    pub map_scene: NativeTerrainSceneMapReference,
    #[serde(default)]
    pub environment: Option<NativeTerrainEnvironmentReference>,
    pub linkage: NativeTerrainSceneLinkage,
    pub owner_components: Vec<NativeTerrainOwnerComponent>,
    pub coordinate_contract: NativeTerrainSceneCoordinateContract,
    pub local_transform: AuthoredWorldTransform,
    pub root_chain: NativeWorldRootChain,
    pub map_tile_root_transform_path_id: i64,
    pub root_chain_blake3: String,
    pub terrain_data_local_geometry_only: bool,
    pub transform_is_not_baked_into_heightmap: bool,
    pub application_order: String,
}

/// Scene-side placement of TerrainData-local native heightmap geometry.
///
/// The TerrainData exporter cannot know the owning scene hierarchy, so this
/// record deliberately keeps the exact TerrainCollider/GameObject/Transform
/// identity and transform separate. In particular, owner Y=-300 is never
/// baked into Gray16 height samples.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldTerrainInstance {
    pub name: String,
    pub path: String,
    pub blake3: String,
    pub true_name: String,
    #[serde(default)]
    pub source_game_object_true_name: Option<String>,
    pub terrain_collider_path_id: i64,
    pub terrain_game_object_path_id: i64,
    pub terrain_transform_path_id: i64,
    #[serde(default)]
    pub terrain_data_path_id: Option<i64>,
    #[serde(default)]
    pub parent_transform_path_id: Option<i64>,
    #[serde(default)]
    pub parent_root_transform_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_component_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_render_contract: Option<NativeTerrainRenderContract>,
    #[serde(default)]
    pub scene_instance_path: Option<String>,
    #[serde(default)]
    pub scene_instance_blake3: Option<String>,
    #[serde(default)]
    pub(super) environment: Option<NativeWorldCatalogEnvironment>,
    pub transform: AuthoredWorldTransform,
    #[serde(default)]
    pub root_chain: Option<NativeWorldRootChain>,
    #[serde(skip)]
    pub(super) loaded: Option<Arc<NativeTerrain>>,
    #[serde(skip)]
    pub(super) scene_instance: Option<Arc<NativeTerrainSceneInstance>>,
}

#[derive(Debug, Default)]
pub(super) struct NativeTerrainCache {
    pub(super) entries: HashMap<String, Arc<NativeTerrain>>,
    pub(super) least_recently_used: VecDeque<String>,
}

impl NativeTerrainCache {
    pub(super) fn get(&mut self, path: &str) -> Option<Arc<NativeTerrain>> {
        let terrain = self.entries.get(path)?.clone();
        self.least_recently_used.retain(|entry| entry != path);
        self.least_recently_used.push_back(path.to_owned());
        Some(terrain)
    }

    pub(super) fn insert(&mut self, path: String, terrain: Arc<NativeTerrain>) {
        self.entries.insert(path.clone(), terrain);
        self.least_recently_used.retain(|entry| entry != &path);
        self.least_recently_used.push_back(path);
        while self.entries.len() > NATIVE_TERRAIN_CACHE_CAPACITY {
            let Some(evicted) = self.least_recently_used.pop_front() else {
                break;
            };
            self.entries.remove(&evicted);
        }
    }
}

pub(super) fn load_native_world_terrain(
    asset_root: &Path,
    terrain_cache: &Arc<Mutex<NativeTerrainCache>>,
    instance: &NativeWorldTerrainInstance,
) -> Result<Arc<NativeTerrain>, NativeWorldSceneError> {
    if let Some(terrain) = terrain_cache
        .lock()
        .map_err(|_| NativeWorldSceneError::new("native terrain cache mutex was poisoned"))?
        .get(&instance.path)
    {
        return Ok(terrain);
    }

    // PNG/Gray16 reads, hash validation, decoding and heightfield generation
    // are deliberately performed without holding the shared LRU mutex. The
    // streaming path runs this function on IoTaskPool, while startup can still
    // use the same fail-closed loader synchronously behind its loading screen.
    let terrain = Arc::new(
        NativeTerrain::open_with_authoritative_environment(
            asset_root,
            &instance.path,
            &instance.blake3,
            instance
                .environment
                .as_ref()
                .map(|reference| reference.blake3.as_str()),
        )
        .map_err(|error| {
            NativeWorldSceneError::new(format!(
                "native terrain {:?} failed lazy validation: {error}",
                instance.path
            ))
        })?,
    );
    if terrain.descriptor().true_name != instance.true_name {
        return Err(NativeWorldSceneError::new(format!(
            "scene terrain trueName {:?} differs from terrain.json {:?}",
            instance.true_name,
            terrain.descriptor().true_name
        )));
    }

    let mut cache = terrain_cache
        .lock()
        .map_err(|_| NativeWorldSceneError::new("native terrain cache mutex was poisoned"))?;
    // Another caller may have completed the same request while this payload
    // was being decoded. Reuse its canonical Arc and do not perturb the LRU
    // with a duplicate allocation.
    if let Some(cached) = cache.get(&instance.path) {
        return Ok(cached);
    }
    cache.insert(instance.path.clone(), terrain.clone());
    Ok(terrain)
}

/// Root-owned background decode/verification for initial and neighboring tiles.
/// Retirement drops the task immediately instead of retaining cancelled work.
#[derive(Component)]
pub(super) struct PendingNativeWorldSceneTerrainLoad {
    pub(super) task: Task<Result<Arc<NativeTerrain>, NativeWorldSceneError>>,
}

pub(super) fn sync_native_terrain_spatial_registry(
    mut commands: Commands,
    mut registry: ResMut<NativeTerrainSpatialRegistry>,
    changed: Query<
        (Entity, &GlobalTransform),
        (
            With<NativeHeightmapCollider>,
            Or<(Added<NativeHeightmapCollider>, Changed<GlobalTransform>)>,
        ),
    >,
    mut removed: RemovedComponents<NativeHeightmapCollider>,
) {
    let mut touched = HashSet::new();
    for entity in removed.read() {
        commands
            .entity(entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.remove::<NativeTerrainSpatialRegistryStatus>();
            });
        if let Some(key) = registry.remove_entity(entity) {
            touched.insert(key);
        }
    }
    for (entity, global) in &changed {
        commands
            .entity(entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.remove::<NativeTerrainSpatialRegistryStatus>();
            });
        if let Some(old_key) = registry.remove_entity(entity) {
            touched.insert(old_key);
        }
        let translation = global.translation();
        let Some(key) = native_dong_key(translation.x, translation.z) else {
            continue;
        };
        registry.insert_entity(entity, key);
        touched.insert(key);
    }
    for key in touched {
        let Some(entities) = registry.key_entities.get(&key) else {
            continue;
        };
        let count = entities.len();
        for &entity in entities {
            let status = if count == 1 {
                NativeTerrainSpatialRegistryStatus::Ready { dong_key: key }
            } else {
                NativeTerrainSpatialRegistryStatus::DuplicateKey {
                    dong_key: key,
                    entities: count,
                }
            };
            commands
                .entity(entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.insert(status);
                });
        }
    }
}

pub(super) fn authored_terrain_motion_patch<'a>(
    heightmaps: impl IntoIterator<Item = (&'a GlobalTransform, &'a NativeHeightmapCollider)>,
    minimum: Vec3,
    maximum: Vec3,
) -> Option<(AuthoredTriMeshCollider, AuthoredColliderWorldBounds)> {
    let mut vertices = Vec::new();
    for (global, heightmap) in heightmaps {
        heightmap.visit_collision_triangles(global, minimum, maximum, |triangle| {
            vertices.extend(triangle);
        });
    }
    if vertices.is_empty() {
        return None;
    }
    let local_min = vertices
        .iter()
        .copied()
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let local_max = vertices
        .iter()
        .copied()
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let indices = (0..vertices.len() as u32).collect::<Vec<_>>().into();
    Some((
        AuthoredTriMeshCollider {
            source_mesh: Handle::default(),
            source_model_path: String::new(),
            vertices: vertices.into(),
            indices,
            local_min,
            local_max,
            is_trigger: false,
        },
        AuthoredColliderWorldBounds {
            minimum: local_min,
            maximum: local_max,
        },
    ))
}

pub(super) fn terrain_capsule_ground_contact(
    heightmap: &NativeHeightmapCollider,
    global: &GlobalTransform,
    x: f32,
    z: f32,
    minimum_y: f32,
    maximum_y: f32,
) -> Option<(f32, Vec3)> {
    let sweep_radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let vertical_padding =
        sweep_radius * AUTHORED_WALKABLE_MAX_TANGENT + AUTHORED_COLLISION_CONTACT_TOLERANCE;
    let mut query_x = x;
    let mut query_z = z;
    let mut highest: Option<(f32, Vec3)> = None;

    // The lower-sphere tangent point is displaced downhill from the vertical
    // root ray. Near a TerrainCollider cell diagonal it can belong to the
    // neighbouring rendered triangle, so follow that point for a bounded
    // number of exact O(1) cell queries. Taking the upper envelope handles a
    // convex terrain bend without falling through either owner triangle.
    for _ in 0..4 {
        let Some(hit) = heightmap.ground_contact(
            global,
            query_x,
            query_z,
            minimum_y - vertical_padding,
            maximum_y + vertical_padding,
        ) else {
            break;
        };
        let mut normal = hit.normal.normalize_or_zero();
        if normal.y < 0.0 {
            normal = -normal;
        }
        if normal == Vec3::ZERO {
            break;
        }
        let plane_height_at_root =
            hit.point.y - (normal.x * (x - hit.point.x) + normal.z * (z - hit.point.z)) / normal.y;
        let root_height =
            capsule_face_root_height(plane_height_at_root, normal).unwrap_or(plane_height_at_root);
        if root_height >= minimum_y - GROUND_EPSILON
            && root_height <= maximum_y + GROUND_EPSILON
            && highest.is_none_or(|(current, _)| root_height > current)
        {
            highest = Some((root_height, normal));
        }
        if normal.y < AUTHORED_WALKABLE_MIN_UP_DOT {
            break;
        }

        let lower_sphere_center =
            Vec3::new(x, root_height + AUTHORED_CHARACTER_CONTROLLER_RADIUS, z);
        let face_distance = normal.dot(lower_sphere_center - hit.point);
        let face_point = lower_sphere_center - normal * face_distance;
        if Vec2::new(face_point.x - query_x, face_point.z - query_z).length_squared()
            <= AUTHORED_COLLISION_EPSILON.powi(2)
        {
            break;
        }
        query_x = face_point.x;
        query_z = face_point.z;
    }
    highest
}
