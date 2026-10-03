use super::*;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldCoordinateContract {
    pub schema: String,
    pub space: String,
    pub basis: String,
    pub unit_scale: String,
    pub origin_policy: String,
    pub gameplay_facing_rotation_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldProvenance {
    pub source_build: String,
    pub source_archive: String,
    pub source_archive_blake3: String,
    pub source_asset: String,
    pub root_transform_path_id: i64,
    /// Optional legacy mesh provenance. Native heightmap scenes omit these
    /// fields rather than publishing zero as a serialized-object identity.
    #[serde(default)]
    pub terrain_game_object_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_transform_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_mesh_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_mesh_filter_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_mesh_renderer_path_id: Option<i64>,
    #[serde(default)]
    pub terrain_mesh_collider_path_id: Option<i64>,
}

/// Parent-relative transform serialized in native Bevy axes.
///
/// Quaternion order is `[x, y, z, w]`. Scale remains authored and may be
/// uniform or non-uniform, but a world node may never be collapsed or mirrored.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthoredWorldTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl AuthoredWorldTransform {
    pub fn try_to_bevy(self, context: &str) -> Result<Transform, NativeWorldSceneError> {
        let translation = Vec3::from_array(self.translation);
        let scale = Vec3::from_array(self.scale);
        let rotation = Quat::from_array(self.rotation);
        let rotation_length_squared = rotation.length_squared();
        if !translation.is_finite()
            || !scale.is_finite()
            || !rotation.is_finite()
            || (rotation_length_squared - 1.0).abs() > UNIT_QUATERNION_TOLERANCE
        {
            return Err(NativeWorldSceneError::new(format!(
                "{context} has a non-finite transform or non-unit quaternion"
            )));
        }
        if scale.min_element() <= 0.0 {
            return Err(NativeWorldSceneError::new(format!(
                "{context} has a zero or negative authored scale"
            )));
        }
        Ok(Transform {
            translation,
            rotation,
            scale,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldVisual {
    pub name: String,
    pub model: String,
    pub scene: usize,
    #[serde(default)]
    pub source_model_path: String,
    #[serde(default)]
    pub legacy_layer: Option<i64>,
    pub transform: AuthoredWorldTransform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeWorldScope {
    WorldMap,
    Tutorial,
}

impl NativeWorldScope {
    pub(super) fn canonical_scene_prefix(self) -> &'static str {
        match self {
            Self::WorldMap => "world/maps/",
            Self::Tutorial => "world/tutorial/terrain/tiles/",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldRootChainNode {
    pub asset_name: String,
    pub transform_path_id: i64,
    pub game_object: NativeWorldRootChainGameObject,
    pub unity_local_transform: NativeWorldSerializedTransform,
    pub native_local_transform: AuthoredWorldTransform,
    pub parent_pointer: Option<NativeSerializedPointer>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldRootChain {
    pub order: String,
    pub includes_owner_transform: bool,
    pub composition_order_transform_path_ids: Vec<i64>,
    pub nodes: Vec<NativeWorldRootChainNode>,
    pub map_tile_root_transform_path_id: i64,
    pub canonical_json_blake3: String,
}

/// A complete, validated native scene graph. It is a Bevy resource so the
/// application can load it once before opening the window and spawn it when
/// OpenFusion supplies a position inside this tile.
#[derive(Debug, Clone, PartialEq, Resource, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldScene {
    pub schema: String,
    pub name: String,
    #[serde(default)]
    pub scope: Option<NativeWorldScope>,
    pub tile: [i32; 2],
    pub coverage: String,
    pub coordinate_contract: NativeWorldCoordinateContract,
    #[serde(default)]
    pub provenance: Option<NativeWorldProvenance>,
    pub root: AuthoredWorldTransform,
    pub models: Vec<NativeWorldModel>,
    pub visuals: Vec<NativeWorldVisual>,
    pub colliders: Vec<NativeWorldCollider>,
    #[serde(default)]
    pub native_terrain: Option<NativeWorldTerrainInstance>,
}

impl NativeWorldScene {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, NativeWorldSceneError> {
        Self::from_json_slice_with_policy(bytes, true)
    }

    pub(super) fn from_json_slice_with_policy(
        bytes: &[u8],
        require_scene_instance: bool,
    ) -> Result<Self, NativeWorldSceneError> {
        let scene: Self = serde_json::from_slice(bytes).map_err(|error| {
            NativeWorldSceneError::new(format!("invalid native world JSON: {error}"))
        })?;
        scene.validate_with_policy(require_scene_instance)?;
        Ok(scene)
    }

    /// Opens a scene below an arbitrary native project-asset root and verifies
    /// the exact GLB hashes recorded by the JSON graph.
    pub fn open(
        asset_root: impl AsRef<Path>,
        relative_scene: &str,
    ) -> Result<Self, NativeWorldSceneError> {
        Self::open_internal(asset_root.as_ref(), relative_scene, None, true, true)
    }

    pub(super) fn open_metadata(
        asset_root: &Path,
        relative_scene: &str,
        expected_scene_blake3: Option<&str>,
    ) -> Result<Self, NativeWorldSceneError> {
        Self::open_internal(
            asset_root,
            relative_scene,
            expected_scene_blake3,
            false,
            true,
        )
    }

    pub(super) fn open_runtime_metadata(
        asset_root: &Path,
        relative_scene: &str,
        expected_scene_blake3: &str,
    ) -> Result<Self, NativeWorldSceneError> {
        Self::open_internal(
            asset_root,
            relative_scene,
            Some(expected_scene_blake3),
            false,
            false,
        )
    }

    pub(super) fn open_internal(
        asset_root: &Path,
        relative_scene: &str,
        expected_scene_blake3: Option<&str>,
        materialize_payload: bool,
        require_scene_instance: bool,
    ) -> Result<Self, NativeWorldSceneError> {
        validate_relative_asset_path(relative_scene, "json")?;
        let canonical_map =
            relative_scene.starts_with("map/tiles/map_") && relative_scene.ends_with("/scene.json");
        let canonical_world_map =
            relative_scene.starts_with("world/maps/") && relative_scene.ends_with("/scene.json");
        let canonical_tutorial = relative_scene.starts_with("world/tutorial/terrain/tiles/")
            && relative_scene.ends_with("/scene.json");
        if !canonical_map && !canonical_world_map && !canonical_tutorial {
            return Err(NativeWorldSceneError::new(
                "native world scene must use a canonical unified map path",
            ));
        }
        let scene_path = join_relative(asset_root, relative_scene);
        let bytes = fs::read(&scene_path).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "failed to read native world scene {}: {error}",
                scene_path.display()
            ))
        })?;
        if let Some(expected) = expected_scene_blake3 {
            verify_file_blake3(&bytes, expected, &scene_path)?;
        }
        let mut scene = Self::from_json_slice_with_policy(&bytes, require_scene_instance)?;
        let inferred_scope = if canonical_map || canonical_world_map {
            NativeWorldScope::WorldMap
        } else {
            NativeWorldScope::Tutorial
        };
        if scene.scope.is_some_and(|scope| scope != inferred_scope) {
            return Err(NativeWorldSceneError::new(format!(
                "scene scope disagrees with canonical path {relative_scene:?}"
            )));
        }
        if scene.scope.is_none() && scene.schema == NATIVE_WORLD_SCENE_SCHEMA {
            return Err(NativeWorldSceneError::new(
                "v2 native world scene is missing its explicit scope",
            ));
        }
        if scene.scope.is_none() {
            scene.scope = Some(inferred_scope);
        }
        if require_scene_instance {
            scene.verify_scene_instance_metadata(asset_root)?;
        }
        if materialize_payload {
            scene.verify_model_files(asset_root)?;
            scene.materialize_native_terrain(asset_root)?;
        }
        Ok(scene)
    }

    pub(super) fn materialize_native_terrain(
        &mut self,
        asset_root: &Path,
    ) -> Result<(), NativeWorldSceneError> {
        if let Some(instance) = self.native_terrain.as_mut() {
            let terrain = NativeTerrain::open_with_authoritative_environment(
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
                    "native terrain {:?} failed validation: {error}",
                    instance.path
                ))
            })?;
            if terrain.descriptor().true_name != instance.true_name {
                return Err(NativeWorldSceneError::new(format!(
                    "scene terrain trueName {:?} differs from terrain.json {:?}",
                    instance.true_name,
                    terrain.descriptor().true_name
                )));
            }
            instance.loaded = Some(Arc::new(terrain));
        }
        Ok(())
    }

    pub(super) fn verify_scene_instance_metadata(
        &mut self,
        asset_root: &Path,
    ) -> Result<(), NativeWorldSceneError> {
        let Some(instance) = self.native_terrain.as_mut() else {
            return Ok(());
        };
        let (Some(relative), Some(expected_hash)) = (
            instance.scene_instance_path.as_deref(),
            instance.scene_instance_blake3.as_deref(),
        ) else {
            if self.schema == NATIVE_WORLD_SCENE_SCHEMA {
                return Err(NativeWorldSceneError::new(
                    "v2 terrain scene is missing its scene-instance path/hash",
                ));
            }
            return Ok(());
        };
        validate_relative_asset_path(relative, "json")?;
        let expected_prefix = instance
            .path
            .strip_suffix("terrain.json")
            .ok_or_else(|| NativeWorldSceneError::new("invalid terrain descriptor suffix"))?;
        if relative != format!("{expected_prefix}scene-instance.json") {
            return Err(NativeWorldSceneError::new(
                "scene-instance is not adjacent to its exact terrain descriptor",
            ));
        }
        let path = join_relative(asset_root, relative);
        let bytes = fs::read(&path).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "failed to read native terrain scene-instance {}: {error}",
                path.display()
            ))
        })?;
        verify_file_blake3(&bytes, expected_hash, &path)?;
        let document: NativeTerrainSceneInstance =
            serde_json::from_slice(&bytes).map_err(|error| {
                NativeWorldSceneError::new(format!(
                    "invalid native terrain scene-instance {}: {error}",
                    path.display()
                ))
            })?;
        validate_scene_instance(instance, &document)?;
        instance.scene_instance = Some(Arc::new(document));
        Ok(())
    }

    pub fn validate(&self) -> Result<(), NativeWorldSceneError> {
        self.validate_with_policy(true)
    }

    pub(super) fn validate_with_policy(
        &self,
        require_scene_instance: bool,
    ) -> Result<(), NativeWorldSceneError> {
        if !matches!(
            self.schema.as_str(),
            NATIVE_WORLD_SCENE_SCHEMA | LEGACY_NATIVE_WORLD_SCENE_SCHEMA
        ) {
            return Err(NativeWorldSceneError::new(format!(
                "unsupported native world schema {:?}",
                self.schema
            )));
        }
        validate_name(&self.name, "scene name")?;
        validate_name(&self.coverage, "scene coverage")?;
        self.validate_coordinate_contract()?;
        if require_scene_instance && self.provenance.is_none() {
            return Err(NativeWorldSceneError::new(
                "authored native world scene is missing conversion provenance",
            ));
        }

        let root = self.root.try_to_bevy("world root")?;
        if self.schema == NATIVE_WORLD_SCENE_SCHEMA && root != Transform::IDENTITY {
            return Err(NativeWorldSceneError::new(format!(
                "v2 native world scene outer root must be identity; placement belongs to rootChain"
            )));
        }

        if self.native_terrain.is_none()
            && (self.models.is_empty() || self.visuals.is_empty() || self.colliders.is_empty())
        {
            return Err(NativeWorldSceneError::new(
                "native world scene requires either a native heightmap terrain or at least one model, visual, and collider",
            ));
        }
        let mut model_ids = BTreeSet::new();
        let mut models = BTreeMap::new();
        for model in &self.models {
            validate_name(&model.id, "model id")?;
            validate_relative_asset_path(&model.path, "glb")?;
            if !model.path.starts_with("objects/") && !model.path.starts_with("models/") {
                return Err(NativeWorldSceneError::new(format!(
                    "world model {:?} must live inside a map object package",
                    model.path
                )));
            }
            validate_blake3(&model.blake3, &model.path)?;
            validate_name(&model.root_name, "model root name")?;
            if !model_ids.insert(model.id.as_str()) {
                return Err(NativeWorldSceneError::new(format!(
                    "duplicate native world model id {:?}",
                    model.id
                )));
            }
            models.insert(model.id.as_str(), model);
        }

        let mut visual_names = BTreeSet::new();
        for visual in &self.visuals {
            validate_name(&visual.name, "visual name")?;
            if !visual_names.insert(visual.name.as_str()) {
                return Err(NativeWorldSceneError::new(format!(
                    "duplicate native world visual name {:?}",
                    visual.name
                )));
            }
            if !models.contains_key(visual.model.as_str()) {
                return Err(NativeWorldSceneError::new(format!(
                    "visual {:?} references unknown model {:?}",
                    visual.name, visual.model
                )));
            }
            visual
                .transform
                .try_to_bevy(&format!("visual {:?}", visual.name))?;
        }

        let mut collider_names = BTreeSet::new();
        for collider in &self.colliders {
            validate_name(&collider.name, "collider name")?;
            if !collider_names.insert(collider.name.as_str()) {
                return Err(NativeWorldSceneError::new(format!(
                    "duplicate native world collider name {:?}",
                    collider.name
                )));
            }
            if !models.contains_key(collider.model.as_str()) {
                return Err(NativeWorldSceneError::new(format!(
                    "collider {:?} references unknown model {:?}",
                    collider.name, collider.model
                )));
            }
            if collider.expected_vertex_count == 0
                || collider.expected_index_count == 0
                || !collider.expected_index_count.is_multiple_of(3)
            {
                return Err(NativeWorldSceneError::new(format!(
                    "collider {:?} has invalid expected triangle cardinality",
                    collider.name
                )));
            }
            collider
                .transform
                .try_to_bevy(&format!("collider {:?}", collider.name))?;
        }
        if let Some(terrain) = &self.native_terrain {
            self.validate_native_terrain_instance(terrain, require_scene_instance)?;
        }
        Ok(())
    }

    pub(super) fn validate_native_terrain_instance(
        &self,
        terrain: &NativeWorldTerrainInstance,
        require_scene_instance: bool,
    ) -> Result<(), NativeWorldSceneError> {
        validate_name(&terrain.name, "native terrain name")?;
        validate_name(&terrain.true_name, "native terrain trueName")?;
        validate_relative_asset_path(&terrain.path, "json")?;
        let canonical_map = terrain.path.starts_with("map/tiles/map_")
            && terrain.path.ends_with("/terrain/terrain.json");
        let canonical_world = terrain.path.starts_with("world/maps/")
            && terrain.path.ends_with("/terrain/terrain.json");
        let canonical_tutorial = terrain.path.starts_with("world/tutorial/terrain/tiles/")
            && terrain.path.ends_with("/terrain.json");
        if !canonical_map && !canonical_world && !canonical_tutorial {
            return Err(NativeWorldSceneError::new(
                "native terrain descriptor must use a canonical unified map path",
            ));
        }
        validate_blake3(&terrain.blake3, &terrain.path)?;
        if terrain.terrain_collider_path_id <= 0
            || terrain.terrain_game_object_path_id <= 0
            || terrain.terrain_transform_path_id <= 0
        {
            return Err(NativeWorldSceneError::new(
                "native terrain scene hierarchy has invalid serialized-object ids",
            ));
        }
        terrain.transform.try_to_bevy("native terrain owner")?;
        if self.schema == NATIVE_WORLD_SCENE_SCHEMA {
            if terrain.terrain_data_path_id.is_none_or(|value| value <= 0)
                || terrain.source_game_object_true_name.is_none()
                || terrain
                    .terrain_component_path_id
                    .is_none_or(|value| value <= 0)
                || terrain.terrain_render_contract.is_none()
                || terrain.root_chain.is_none()
                || (require_scene_instance
                    && (terrain.scene_instance_path.is_none()
                        || terrain.scene_instance_blake3.is_none()))
                || terrain.scene_instance_path.is_some() != terrain.scene_instance_blake3.is_some()
            {
                return Err(NativeWorldSceneError::new(
                    "v2 native terrain placement contract is incomplete",
                ));
            }
            if let Some(hash) = terrain.scene_instance_blake3.as_deref() {
                validate_plain_or_prefixed_blake3(hash, "scene-instance")?;
            }
        }
        if let Some(environment) = &terrain.environment {
            validate_relative_asset_path(&environment.path, "json")?;
            validate_blake3(&environment.blake3, "scene terrain environment")?;
            let expected_environment = terrain
                .path
                .rsplit_once('/')
                .map(|(parent, _)| format!("{parent}/environment/environment.json"))
                .ok_or_else(|| {
                    NativeWorldSceneError::new("scene terrain descriptor has no parent path")
                })?;
            if environment.schema != NATIVE_TERRAIN_ENVIRONMENT_SCHEMA
                || environment.path != expected_environment
                || environment.status != "complete"
            {
                return Err(NativeWorldSceneError::new(
                    "linked scene terrain environment reference is invalid",
                ));
            }
        }
        if let Some(root_chain) = &terrain.root_chain {
            let expected_root_transform_path_id = self
                .provenance
                .as_ref()
                .map(|provenance| provenance.root_transform_path_id)
                .unwrap_or(root_chain.map_tile_root_transform_path_id);
            validate_root_chain(
                root_chain,
                terrain.parent_transform_path_id,
                expected_root_transform_path_id,
            )?;
        } else if self.provenance.as_ref().is_none_or(|provenance| {
            terrain
                .parent_root_transform_path_id
                .is_none_or(|value| value <= 0 || value != provenance.root_transform_path_id)
        }) {
            return Err(NativeWorldSceneError::new(
                "legacy native terrain root identity is missing or mismatched",
            ));
        }
        if let Some(render) = &terrain.terrain_render_contract {
            validate_render_contract(render)?;
        }
        let world_matrix = self.terrain_world_matrix(terrain)?;
        let world_up = world_matrix.transform_vector3(Vec3::Y);
        if !world_up.is_finite() || world_up.length_squared() <= f32::EPSILON {
            return Err(NativeWorldSceneError::new(
                "native regular-grid terrain has a singular/non-finite +Y axis",
            ));
        }
        let world_up = world_up.normalize();
        if !world_up.abs_diff_eq(Vec3::Y, 0.000_001) {
            return Err(NativeWorldSceneError::new(
                "native regular-grid terrain hierarchy may yaw/scale but may not tilt its +Y height axis",
            ));
        }
        if self.models.iter().any(|model| {
            model.path.to_ascii_lowercase().contains("terrain.glb")
                || model.id.eq_ignore_ascii_case("terrain")
        }) {
            return Err(NativeWorldSceneError::new(
                "native heightmap terrain scene must not retain a terrain GLB model",
            ));
        }
        Ok(())
    }

    pub(super) fn terrain_world_matrix(
        &self,
        terrain: &NativeWorldTerrainInstance,
    ) -> Result<Mat4, NativeWorldSceneError> {
        let mut matrix = self.root.try_to_bevy("world root")?.to_matrix();
        if let Some(root_chain) = &terrain.root_chain {
            for node in root_chain.nodes.iter().rev() {
                matrix *= node
                    .native_local_transform
                    .try_to_bevy("native terrain rootChain node")?
                    .to_matrix();
            }
        }
        matrix *= terrain
            .transform
            .try_to_bevy("native terrain owner")?
            .to_matrix();
        if !matrix.is_finite() {
            return Err(NativeWorldSceneError::new(
                "native terrain global matrix is non-finite",
            ));
        }
        Ok(matrix)
    }

    pub(super) fn terrain_origin(&self) -> Option<Vec3> {
        let terrain = self.native_terrain.as_ref()?;
        Some(
            self.terrain_world_matrix(terrain)
                .ok()?
                .transform_point3(Vec3::ZERO),
        )
    }

    pub(super) fn validate_coordinate_contract(&self) -> Result<(), NativeWorldSceneError> {
        let contract = &self.coordinate_contract;
        if contract.schema != NATIVE_COORDINATE_CONTRACT_SCHEMA
            || contract.space != "native"
            || contract.basis != WORLD_BASIS
            || contract.unit_scale != WORLD_UNIT_SCALE
            || contract.origin_policy != WORLD_ORIGIN_POLICY
            || contract.gameplay_facing_rotation_applied
        {
            return Err(NativeWorldSceneError::new(
                "native world coordinate contract permits an axis, origin, scale, or gameplay-facing mismatch",
            ));
        }
        Ok(())
    }

    pub(super) fn verify_model_files(&self, asset_root: &Path) -> Result<(), NativeWorldSceneError> {
        for model in &self.models {
            let path = join_relative(asset_root, &model.path);
            let bytes = fs::read(&path).map_err(|error| {
                NativeWorldSceneError::new(format!(
                    "failed to read native world model {}: {error}",
                    path.display()
                ))
            })?;
            let actual = blake3::hash(&bytes).to_hex().to_string();
            if actual != model.blake3 {
                return Err(NativeWorldSceneError::new(format!(
                    "native world model hash mismatch for {}: expected {}, got {actual}",
                    path.display(),
                    model.blake3
                )));
            }
            validate_single_root_world_glb(&bytes, &model.root_name, &model.path)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn contains_native_xz(&self, position: Vec3) -> bool {
        let Some(origin) = self.terrain_origin() else {
            return false;
        };
        native_dong_key(origin.x, origin.z) == native_dong_key(position.x, position.z)
    }

    pub(super) fn model(&self, id: &str) -> Option<&NativeWorldModel> {
        self.models.iter().find(|model| model.id == id)
    }
}

/// Authored renderers are normally owned by the 512-unit tile AABB. A small
/// set of primary edge/backdrop placements deliberately lives outside that
/// box, so streaming also retains their published centers. This is derived
/// once from already-loaded scene metadata; no GLB or runtime hierarchy scan
/// is involved.
#[derive(Debug, Clone, Default)]
pub(super) struct NativeWorldPresentationFootprint {
    pub(super) overflow_centers: Box<[Vec2]>,
}

impl NativeWorldPresentationFootprint {
    pub(super) fn from_scene(scene: &NativeWorldScene) -> Self {
        let overflow_centers = scene
            .visuals
            .iter()
            .filter(|visual| !is_legacy_non_presenting_visual(scene, visual))
            .filter_map(|visual| {
                let center = Vec3::from_array(visual.transform.translation);
                (legacy_dong_squared_distance_native(center, scene.tile)? > 0.0)
                    .then_some(Vec2::new(center.x, center.z))
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { overflow_centers }
    }

    pub(super) fn squared_distance(&self, native_position: Vec3, tile: [i32; 2]) -> Option<f32> {
        let mut distance = legacy_dong_squared_distance_native(native_position, tile)?;
        let position = native_position.xz();
        for center in &self.overflow_centers {
            distance = distance.min(position.distance_squared(*center));
        }
        Some(distance)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeWorldSceneEntity;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct NativeWorldSceneRoot {
    pub name: String,
    pub tile: [i32; 2],
    /// Scope that owns the published scene, behaviour and environment data.
    pub scope: NativeWorldScope,
    /// Scope requested by the current gameplay slice. Tutorial deliberately
    /// falls back to world-owned dongs when no tutorial override exists, so
    /// this cannot be reconstructed from `scope`.
    pub selection_scope: NativeWorldScope,
}

/// Presentation-only owner for an exact published ffWater visual from a
/// neighboring dong whose full scene is outside the ordinary load radius.
/// It is deliberately not a [`NativeWorldSceneRoot`], so it cannot consume a
/// resident tile slot or admit that tile's terrain/objects/behaviours.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldNeighborWaterRoot {
    pub(super) tile: [i32; 2],
    pub(super) selection_scope: NativeWorldScope,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldNeighborWaterVisual;

/// A streamed tile whose Bevy asset requests are being admitted gradually.
///
/// This is public only so the sibling behaviour runtime can wait for the full
/// static hierarchy before binding source-node ownership. Fields remain private
/// because stream admission is owned exclusively by [`NativeWorldPlugin`].
#[derive(Component)]
pub struct PendingNativeWorldSceneSpawn {
    pub(super) next_visual: usize,
    pub(super) next_collider: usize,
    pub(super) terrain: Option<Arc<NativeTerrain>>,
    pub(super) terrain_spawned: bool,
}

/// Hidden streamed tile whose complete scene/collider hierarchy is being
/// released in post-order over several frames.
///
/// A count of root children is not a useful destruction budget: each GLTF
/// scene child can own hundreds of descendants, and Bevy recursively despawns
/// that entire subtree. The retained traversal snapshots let the unload pass
/// remove actual entities, leaves first, without invalidating child indices
/// between frames.
#[derive(Component, Debug, Default)]
pub struct PendingNativeWorldSceneUnload {
    pub(super) traversal: Vec<NativeWorldSceneUnloadFrame>,
    pub(super) initialized: bool,
}
