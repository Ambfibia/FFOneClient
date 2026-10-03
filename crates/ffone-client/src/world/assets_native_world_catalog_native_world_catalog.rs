use super::*;

impl NativeWorldCatalog {
    pub(super) fn water_visual_indices(
        scenes: &[NativeWorldScene],
    ) -> HashMap<(NativeWorldScope, [i32; 2]), Box<[usize]>> {
        scenes
            .iter()
            .filter_map(|scene| {
                let indices = scene
                    .visuals
                    .iter()
                    .enumerate()
                    .filter_map(|(index, visual)| {
                        is_native_world_water_visual(scene, visual).then_some(index)
                    })
                    .collect::<Vec<_>>();
                (!indices.is_empty()).then_some((
                    (
                        scene.scope.unwrap_or(NativeWorldScope::WorldMap),
                        scene.tile,
                    ),
                    indices.into_boxed_slice(),
                ))
            })
            .collect()
    }

    pub(super) fn presentation_footprints(
        scenes: &[NativeWorldScene],
    ) -> HashMap<(NativeWorldScope, [i32; 2]), NativeWorldPresentationFootprint> {
        scenes
            .iter()
            .map(|scene| {
                let scope = scene.scope.unwrap_or(NativeWorldScope::WorldMap);
                (
                    (scope, scene.tile),
                    NativeWorldPresentationFootprint::from_scene(scene),
                )
            })
            .collect()
    }

    pub(super) fn presentation_squared_distance(
        &self,
        _scope: NativeWorldScope,
        native_position: Vec3,
        scene: &NativeWorldScene,
    ) -> Option<f32> {
        // Tutorial selection intentionally falls back to the ordinary world
        // grid when the publication has no tutorial-owned dong at this key.
        // The presentation footprint still belongs to the selected scene's
        // authored scope; looking it up through the requested scope would
        // discard overflow/backdrop centers on that fallback path.
        let scene_scope = scene.scope.unwrap_or(NativeWorldScope::WorldMap);
        self.presentation_footprints
            .get(&(scene_scope, scene.tile))
            .map_or_else(
                || legacy_dong_squared_distance_native(native_position, scene.tile),
                |footprint| footprint.squared_distance(native_position, scene.tile),
            )
    }

    pub fn open(asset_root: impl AsRef<Path>) -> Result<Self, NativeWorldSceneError> {
        let asset_root = asset_root.as_ref();
        let map_catalog_path = join_relative(asset_root, NATIVE_WORLD_CATALOG_PATH);
        if map_catalog_path.is_file() {
            return Self::open_map_catalog(asset_root, &map_catalog_path);
        }
        let runtime_registry_path = join_relative(asset_root, RUNTIME_WORLD_REGISTRY_PATH);
        if runtime_registry_path.is_file() {
            return Self::open_runtime_registry(asset_root, &runtime_registry_path);
        }
        let catalog_path = join_relative(asset_root, "world/catalog.json");
        let bytes = fs::read(&catalog_path).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "failed to read native world catalog {}: {error}",
                catalog_path.display()
            ))
        })?;
        let document: NativeWorldCatalogDocument =
            serde_json::from_slice(&bytes).map_err(|error| {
                NativeWorldSceneError::new(format!(
                    "invalid native world catalog {}: {error}",
                    catalog_path.display()
                ))
            })?;
        let mut scenes = Vec::new();
        let mut blocked = Vec::new();
        let mut world_ambience = NativeTerrainAmbienceGrid::default();
        let mut tutorial_ambience = NativeTerrainAmbienceGrid::default();
        match document {
            NativeWorldCatalogDocument::V2(document) => {
                if document.schema != NATIVE_WORLD_CATALOG_SCHEMA
                    || document.source_build.trim().is_empty()
                    || document.status.trim().is_empty()
                    || document.terrain_glb_allowed
                {
                    return Err(NativeWorldSceneError::new(
                        "v2 world catalog header is invalid or permits terrain GLB",
                    ));
                }
                blocked = document.blocked;
                for entry in document.entries {
                    validate_catalog_entry(&entry, true)?;
                    let environment = entry
                        .environment
                        .as_ref()
                        .map(|reference| load_catalog_environment(asset_root, reference, &entry))
                        .transpose()?;
                    if entry.placement_status == "blocked" {
                        if entry.scene.is_some() {
                            return Err(NativeWorldSceneError::new(
                                "blocked catalog placement must not invent a scene",
                            ));
                        }
                        continue;
                    }
                    if let Some(environment) = environment.as_ref() {
                        let registry = match entry.scope {
                            NativeWorldScope::WorldMap => &mut world_ambience,
                            NativeWorldScope::Tutorial => &mut tutorial_ambience,
                        };
                        registry.register(environment).map_err(|blocker| {
                            NativeWorldSceneError::new(format!(
                                "linked catalog environment {:?} is not runtime-registerable: status={}, ambienceStatus={}",
                                blocker.tile_id, blocker.status, blocker.ambience_status
                            ))
                        })?;
                    }
                    let relative = entry.scene.as_deref().ok_or_else(|| {
                        NativeWorldSceneError::new("linked catalog entry has no scene")
                    })?;
                    let scene_hash = entry.scene_blake3.as_deref().ok_or_else(|| {
                        NativeWorldSceneError::new("linked v2 catalog entry has no sceneBlake3")
                    })?;
                    let scene =
                        NativeWorldScene::open_metadata(asset_root, relative, Some(scene_hash))?;
                    validate_catalog_scene_link(&entry, &scene)?;
                    scenes.push(scene);
                }
            }
            NativeWorldCatalogDocument::V1(document) => {
                if document.schema != LEGACY_NATIVE_WORLD_CATALOG_SCHEMA
                    || document.terrain_glb_allowed
                    || document.runtime_migration_pending
                    || document.source_aliases_retained_in_runtime_assets
                    || document.source_build.trim().is_empty()
                    || document.status.trim().is_empty()
                {
                    return Err(NativeWorldSceneError::new(
                        "legacy world catalog header is incomplete",
                    ));
                }
                for entry in document.maps {
                    let scene = NativeWorldScene::open_metadata(
                        asset_root,
                        &entry.scene,
                        Some(&entry.scene_blake3),
                    )?;
                    if scene.native_terrain.as_ref().is_none_or(|terrain| {
                        terrain.path != entry.terrain_descriptor
                            || terrain.blake3 != entry.terrain_descriptor_blake3
                            || terrain.true_name != entry.true_terrain_data_name
                    }) {
                        return Err(NativeWorldSceneError::new(format!(
                            "legacy catalog entry {:?} disagrees with its scene",
                            entry.id
                        )));
                    }
                    scenes.push(scene);
                }
            }
        }
        let (world_by_dong_key, tutorial_by_dong_key) = build_catalog_indices(&scenes)?;
        let water_visual_indices = Self::water_visual_indices(&scenes);
        let presentation_footprints = Self::presentation_footprints(&scenes);
        Ok(Self {
            asset_root: asset_root.to_path_buf(),
            scenes,
            water_visual_indices,
            world_by_dong_key,
            tutorial_by_dong_key,
            presentation_footprints,
            blocked,
            terrain_cache: Arc::new(Mutex::new(NativeTerrainCache::default())),
            world_ambience,
            tutorial_ambience,
            runtime_behaviour_assets: HashMap::new(),
            runtime_object_assets: HashMap::new(),
        })
    }

    pub(super) fn open_runtime_registry(
        asset_root: &Path,
        registry_path: &Path,
    ) -> Result<Self, NativeWorldSceneError> {
        let bytes = fs::read(registry_path).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "failed to read runtime world registry {}: {error}",
                registry_path.display()
            ))
        })?;
        let document: RuntimeWorldRegistry = serde_json::from_slice(&bytes).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "invalid runtime world registry {}: {error}",
                registry_path.display()
            ))
        })?;
        if document.schema != RUNTIME_WORLD_REGISTRY_SCHEMA {
            return Err(NativeWorldSceneError::new(format!(
                "unsupported runtime world registry schema {:?}",
                document.schema
            )));
        }

        Self::open_runtime_entries(asset_root, document.entries)
    }

    pub(super) fn open_map_catalog(
        asset_root: &Path,
        catalog_path: &Path,
    ) -> Result<Self, NativeWorldSceneError> {
        let bytes = fs::read(catalog_path).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "failed to read unified map catalog {}: {error}",
                catalog_path.display()
            ))
        })?;
        let catalog: RuntimeMapCatalog = serde_json::from_slice(&bytes).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "invalid unified map catalog {}: {error}",
                catalog_path.display()
            ))
        })?;
        if catalog.schema != MAP_CATALOG_SCHEMA || catalog.tiles.is_empty() {
            return Err(NativeWorldSceneError::new(
                "unified map catalog has an invalid identity or no tiles",
            ));
        }

        let mut entries = Vec::with_capacity(catalog.tiles.len());
        let mut ids = HashSet::new();
        for tile_reference in catalog.tiles {
            if !ids.insert(tile_reference.tile_id.clone()) {
                return Err(NativeWorldSceneError::new(format!(
                    "unified map catalog duplicates tile {:?}",
                    tile_reference.tile_id
                )));
            }
            let manifest_reference = RuntimeWorldAssetReference {
                path: tile_reference.manifest.path,
                blake3: tile_reference.manifest.blake3,
            };
            let manifest_bytes =
                read_runtime_world_reference(asset_root, &manifest_reference, "map tile manifest")?;
            let manifest: RuntimeMapTile =
                serde_json::from_slice(&manifest_bytes).map_err(|error| {
                    NativeWorldSceneError::new(format!(
                        "invalid unified map tile manifest {:?}: {error}",
                        manifest_reference.path
                    ))
                })?;
            if manifest.schema != MAP_TILE_SCHEMA || manifest.id != tile_reference.tile_id {
                return Err(NativeWorldSceneError::new(format!(
                    "unified map tile manifest {:?} disagrees with the catalog",
                    manifest_reference.path
                )));
            }
            let environment = manifest
                .files
                .iter()
                .find(|artifact| {
                    artifact
                        .path
                        .ends_with("/terrain/environment/environment.json")
                })
                .map(|artifact| RuntimeWorldAssetReference {
                    path: artifact.path.clone(),
                    blake3: artifact.blake3.clone(),
                });
            for (authoritative, context) in [
                (&manifest.behaviour, "behaviour"),
                (&manifest.objects, "objects"),
            ] {
                let listed = manifest
                    .files
                    .iter()
                    .find(|artifact| artifact.path == authoritative.path)
                    .ok_or_else(|| {
                        NativeWorldSceneError::new(format!(
                            "unified map tile manifest {:?} omits its authoritative {context} artifact from files",
                            manifest.id
                        ))
                    })?;
                if listed.blake3 != authoritative.blake3 {
                    return Err(NativeWorldSceneError::new(format!(
                        "unified map tile manifest {:?} has contradictory {context} hashes",
                        manifest.id
                    )));
                }
            }
            let behaviour = Some(RuntimeWorldAssetReference {
                path: manifest.behaviour.path,
                blake3: manifest.behaviour.blake3,
            });
            let objects = Some(RuntimeWorldAssetReference {
                path: manifest.objects.path,
                blake3: manifest.objects.blake3,
            });
            entries.push(RuntimeWorldRegistryEntry {
                id: manifest.id,
                scope: NativeWorldScope::WorldMap,
                tile: manifest.grid,
                scene: RuntimeWorldAssetReference {
                    path: manifest.scene.path,
                    blake3: manifest.scene.blake3,
                },
                terrain: RuntimeWorldAssetReference {
                    path: manifest.terrain.path,
                    blake3: manifest.terrain.blake3,
                },
                environment,
                behaviour,
                objects,
            });
        }
        Self::open_runtime_entries(asset_root, entries)
    }

    pub(super) fn open_runtime_entries(
        asset_root: &Path,
        entries: Vec<RuntimeWorldRegistryEntry>,
    ) -> Result<Self, NativeWorldSceneError> {
        if entries.is_empty() {
            return Err(NativeWorldSceneError::new(
                "runtime world registry has no playable entries",
            ));
        }

        let mut scenes = Vec::with_capacity(entries.len());
        let mut world_ambience = NativeTerrainAmbienceGrid::default();
        let mut tutorial_ambience = NativeTerrainAmbienceGrid::default();
        let mut ids = HashSet::new();
        let mut runtime_behaviour_assets = HashMap::new();
        let mut runtime_object_assets = HashMap::new();
        for entry in entries {
            validate_runtime_world_registry_entry(&entry)?;
            if !ids.insert((entry.scope, entry.id.clone())) {
                return Err(NativeWorldSceneError::new(format!(
                    "duplicate runtime world id {:?} inside {:?} scope",
                    entry.id, entry.scope
                )));
            }
            verify_runtime_world_reference(asset_root, &entry.terrain, "terrain descriptor")?;
            if let Some(reference) = entry.behaviour.as_ref() {
                validate_runtime_world_reference(reference, "world behaviour")?;
                runtime_behaviour_assets.insert((entry.scope, entry.id.clone()), reference.clone());
            }
            if let Some(reference) = entry.objects.as_ref() {
                validate_runtime_world_reference(reference, "world object routes")?;
                runtime_object_assets.insert((entry.scope, entry.id.clone()), reference.clone());
            }
            let environment = entry
                .environment
                .as_ref()
                .map(|reference| load_runtime_environment(asset_root, reference, &entry))
                .transpose()?;
            if let Some(environment) = environment.as_ref() {
                let registry = match entry.scope {
                    NativeWorldScope::WorldMap => &mut world_ambience,
                    NativeWorldScope::Tutorial => &mut tutorial_ambience,
                };
                registry.register(environment).map_err(|blocker| {
                    NativeWorldSceneError::new(format!(
                        "runtime registry environment {:?} is not registerable: status={}, ambienceStatus={}",
                        blocker.tile_id, blocker.status, blocker.ambience_status
                    ))
                })?;
            }
            // `open_runtime_metadata` performs the authoritative path/hash
            // validation while decoding the document. Verifying the same
            // scene reference first read and hashed every (often megabyte-
            // sized) scene twice during startup.
            let scene = NativeWorldScene::open_runtime_metadata(
                asset_root,
                &entry.scene.path,
                &entry.scene.blake3,
            )?;
            validate_runtime_scene_link(&entry, &scene)?;
            scenes.push(scene);
        }
        let (world_by_dong_key, tutorial_by_dong_key) = build_catalog_indices(&scenes)?;
        let water_visual_indices = Self::water_visual_indices(&scenes);
        let presentation_footprints = Self::presentation_footprints(&scenes);
        Ok(Self {
            asset_root: asset_root.to_path_buf(),
            scenes,
            water_visual_indices,
            world_by_dong_key,
            tutorial_by_dong_key,
            presentation_footprints,
            blocked: Vec::new(),
            terrain_cache: Arc::new(Mutex::new(NativeTerrainCache::default())),
            world_ambience,
            tutorial_ambience,
            runtime_behaviour_assets,
            runtime_object_assets,
        })
    }

    #[must_use]
    pub fn scenes(&self) -> &[NativeWorldScene] {
        &self.scenes
    }

    #[must_use]
    pub fn blocked_count(&self) -> usize {
        self.blocked.len()
    }

    pub(crate) fn behaviour_asset(
        &self,
        scope: NativeWorldScope,
        scene_name: &str,
    ) -> Option<(PathBuf, String)> {
        self.runtime_behaviour_assets
            .get(&(scope, scene_name.to_owned()))
            .map(|reference| {
                (
                    join_relative(&self.asset_root, &reference.path),
                    reference.blake3.clone(),
                )
            })
    }

    pub(crate) fn object_asset(
        &self,
        scope: NativeWorldScope,
        scene_name: &str,
    ) -> Option<(PathBuf, String)> {
        self.runtime_object_assets
            .get(&(scope, scene_name.to_owned()))
            .map(|reference| {
                (
                    join_relative(&self.asset_root, &reference.path),
                    reference.blake3.clone(),
                )
            })
    }

    #[must_use]
    pub fn registered_ambience_cells(&self, tutorial: bool) -> usize {
        if tutorial {
            self.tutorial_ambience.registered_cell_count()
        } else {
            self.world_ambience.registered_cell_count()
        }
    }

    #[must_use]
    pub fn sample_ambience(
        &self,
        native_position: Vec3,
        tutorial: bool,
    ) -> NativeTerrainAmbienceSample {
        let registry = if tutorial {
            if self.tutorial_ambience.registered_cell_count() == 0 {
                &self.world_ambience
            } else {
                &self.tutorial_ambience
            }
        } else {
            &self.world_ambience
        };
        registry.sample_native_position(native_position.x, native_position.z)
    }

    #[must_use]
    pub fn applied_ambience(
        &self,
        native_position: Vec3,
        tutorial: bool,
    ) -> NativeTerrainAppliedAmbience {
        apply_default_ambience(self.sample_ambience(native_position, tutorial), tutorial)
    }

    #[must_use]
    pub fn cached_terrain_count(&self) -> usize {
        self.terrain_cache
            .lock()
            .map_or(0, |cache| cache.entries.len())
    }

    #[must_use]
    pub fn select_in_scope(
        &self,
        scope: NativeWorldScope,
        native_position: Vec3,
    ) -> Option<&NativeWorldScene> {
        let key = native_dong_key(native_position.x, native_position.z)?;
        let index = match scope {
            NativeWorldScope::WorldMap => self.world_by_dong_key.get(&key),
            NativeWorldScope::Tutorial => self
                .tutorial_by_dong_key
                .get(&key)
                .or_else(|| self.world_by_dong_key.get(&key)),
        }?;
        self.scenes.get(*index)
    }

    pub(super) fn scene_for_tile(&self, scope: NativeWorldScope, tile: [i32; 2]) -> Option<&NativeWorldScene> {
        let native_center = Vec3::new(
            -(tile[0] as f32 * WORLD_TILE_SIZE_NATIVE + WORLD_TILE_SIZE_NATIVE * 0.5),
            0.0,
            tile[1] as f32 * WORLD_TILE_SIZE_NATIVE + WORLD_TILE_SIZE_NATIVE * 0.5,
        );
        self.select_in_scope(scope, native_center)
            .filter(|scene| scene.tile == tile)
    }

    pub(super) fn scene_is_selected_for_scope(
        &self,
        scope: NativeWorldScope,
        scene: &NativeWorldScene,
    ) -> bool {
        self.scene_for_tile(scope, scene.tile)
            .is_some_and(|selected| std::ptr::eq(selected, scene))
    }

    /// Exact eventual load candidate used by legacy `DongLoader.UpdatePosition`.
    ///
    /// The old client starts one nearest load at a time. The native threshold
    /// extends 280 to 356 units to cover the 340-unit camera plus its 12-unit
    /// orbit offset and a four-unit admission margin. Published presentation
    /// overflow centers supplement the normal tile AABB for edge backdrops.
    #[must_use]
    pub fn next_legacy_stream_load(
        &self,
        scope: NativeWorldScope,
        native_position: Vec3,
        loaded_tiles: &HashSet<[i32; 2]>,
    ) -> Option<&NativeWorldScene> {
        if !native_position.x.is_finite() || !native_position.z.is_finite() {
            return None;
        }
        let mut best_distance =
            EXTENDED_DONG_LOAD_DISTANCE_NATIVE * EXTENDED_DONG_LOAD_DISTANCE_NATIVE;
        let mut best: Option<&NativeWorldScene> = None;
        // Most scenes are reached through their ordinary 512-unit AABB. A
        // handful of primary edge/backdrop renderers extend by more than one
        // grid cell, so scan the small metadata catalog instead of silently
        // excluding their owner from the legacy 3x3 iteration window.
        let candidates = self
            .scenes
            .iter()
            .filter(|scene| self.scene_is_selected_for_scope(scope, scene));
        for scene in candidates {
            if loaded_tiles.contains(&scene.tile) {
                continue;
            }
            let Some(distance) = self.presentation_squared_distance(scope, native_position, scene)
            else {
                continue;
            };
            // Resolve ties explicitly instead of allocating and sorting the
            // whole catalog each frame. Keep the strict load-distance edge.
            if distance < best_distance
                || (distance == best_distance
                    && best.is_some_and(|best| {
                        (scene.tile[1], scene.tile[0]) < (best.tile[1], best.tile[0])
                    }))
            {
                best_distance = distance;
                best = Some(scene);
            }
        }
        best
    }

    /// Complete set of tiles that the native `DongLoader` topology requires
    /// before `IsReadyForPlay` may become true at `native_position`.
    ///
    /// The same set remains the background streaming target after startup.
    /// It retains deterministic source order with the documented 356-unit
    /// horizon extension. Keeping this calculation next to
    /// [`Self::next_legacy_stream_load`] prevents the loading screen and the
    /// streamer from disagreeing at tile boundaries.
    #[must_use]
    pub fn legacy_stream_target_tiles(
        &self,
        scope: NativeWorldScope,
        native_position: Vec3,
    ) -> Vec<[i32; 2]> {
        if !native_position.x.is_finite() || !native_position.z.is_finite() {
            return Vec::new();
        }
        let maximum_distance =
            EXTENDED_DONG_LOAD_DISTANCE_NATIVE * EXTENDED_DONG_LOAD_DISTANCE_NATIVE;
        let mut targets = Vec::new();
        for scene in &self.scenes {
            if self.scene_is_selected_for_scope(scope, scene)
                && self
                    .presentation_squared_distance(scope, native_position, scene)
                    .is_some_and(|distance| distance < maximum_distance)
            {
                targets.push(scene.tile);
            }
        }
        targets.sort_by_key(|tile| (tile[1], tile[0]));
        targets
    }

    /// Default gameplay selection is world-map scoped. Tutorial uses its own
    /// registry, so equal legacy dong keys never collide across scopes.
    #[must_use]
    pub fn select(&self, native_position: Vec3) -> Option<&NativeWorldScene> {
        self.select_in_scope(NativeWorldScope::WorldMap, native_position)
    }

    pub fn load_terrain(
        &self,
        instance: &NativeWorldTerrainInstance,
    ) -> Result<Arc<NativeTerrain>, NativeWorldSceneError> {
        load_native_world_terrain(&self.asset_root, &self.terrain_cache, instance)
    }
}

/// A direct request for the only mesh in a validated static-world GLB.
///
/// The strong handle keeps the mesh asset and all of its material/texture
/// dependencies alive until it is attached to the placement entity.
#[derive(Component, Debug, Clone)]
pub(super) struct PendingNativeWorldVisualAsset {
    pub(super) gltf: Handle<Gltf>,
    pub(super) mesh: Handle<GltfMesh>,
}

pub(super) fn load_catalog_environment(
    asset_root: &Path,
    reference: &NativeWorldCatalogEnvironment,
    entry: &NativeWorldCatalogEntry,
) -> Result<NativeTerrainEnvironment, NativeWorldSceneError> {
    let path = join_relative(asset_root, &reference.path);
    let bytes = fs::read(&path).map_err(|error| {
        NativeWorldSceneError::new(format!(
            "failed to read native terrain environment {}: {error}",
            path.display()
        ))
    })?;
    let actual = blake3::hash(&bytes).to_hex().to_string();
    if actual != reference.blake3 {
        return Err(NativeWorldSceneError::new(format!(
            "native terrain environment hash mismatch for {}: expected {}, got {actual}",
            path.display(),
            reference.blake3
        )));
    }
    let environment: NativeTerrainEnvironment =
        serde_json::from_slice(&bytes).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "invalid native terrain environment {}: {error}",
                path.display()
            ))
        })?;
    let expected_scope = match entry.scope {
        NativeWorldScope::WorldMap => "worldMap",
        NativeWorldScope::Tutorial => "tutorial",
    };
    let expected_tile_id = entry
        .instance_id
        .strip_prefix("map_")
        .or_else(|| entry.instance_id.strip_prefix("tile_"))
        .ok_or_else(|| {
            NativeWorldSceneError::new("catalog environment instanceId has no map/tile prefix")
        })?;
    if environment.schema != reference.schema
        || environment.status != reference.status
        || environment.scope != expected_scope
        || environment.tile_id != expected_tile_id
    {
        return Err(NativeWorldSceneError::new(
            "catalog environment document identity disagrees with its entry/reference",
        ));
    }
    if entry.placement_status == "linked"
        && (environment.status != "complete"
            || environment.ambience.status != "exactSource"
            || environment.ambience.grid_coordinates != entry.tile)
    {
        return Err(NativeWorldSceneError::new(
            "linked catalog environment has a placement or DongColor coordinate contradiction",
        ));
    }
    Ok(environment)
}

pub(super) fn build_catalog_indices(
    scenes: &[NativeWorldScene],
) -> Result<(HashMap<i32, usize>, HashMap<i32, usize>), NativeWorldSceneError> {
    let mut world = HashMap::new();
    let mut tutorial = HashMap::new();
    for (index, scene) in scenes.iter().enumerate() {
        let origin = scene.terrain_origin().ok_or_else(|| {
            NativeWorldSceneError::new("catalog scene has no finite native terrain origin")
        })?;
        let key = native_dong_key(origin.x, origin.z).ok_or_else(|| {
            NativeWorldSceneError::new("catalog scene origin has no legacy dong key")
        })?;
        let registry = match scene.scope {
            Some(NativeWorldScope::WorldMap) => &mut world,
            Some(NativeWorldScope::Tutorial) => &mut tutorial,
            None => {
                return Err(NativeWorldSceneError::new(
                    "catalog scene has no resolved scope",
                ));
            }
        };
        if registry.insert(key, index).is_some() {
            return Err(NativeWorldSceneError::new(format!(
                "duplicate native terrain dong key {key} inside {:?} scope",
                scene.scope
            )));
        }
    }
    Ok((world, tutorial))
}
