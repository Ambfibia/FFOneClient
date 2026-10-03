use super::*;

/// A manifest-verified native shared-rig catalog.
///
/// This resource is immutable and may be shared. All mutable loading, bone
/// lookup, skin rebinding, animation playback, and readiness state lives on
/// each [`NativePlayerRigInstance`] entity.
#[derive(Resource, Clone, Debug)]
pub struct NativePlayerRigCatalog {
    pub(super) asset_root: PathBuf,
    pub(super) contract: PlayerSharedRigContract,
    pub(super) table_extension_failures: BTreeMap<(u8, String), String>,
}

impl NativePlayerRigCatalog {
    /// Opens the published contract and verifies every code- or contract-owned
    /// GLB directly from the configured asset source.
    pub fn open(asset_root: impl Into<PathBuf>) -> Result<Self, String> {
        let asset_root = asset_root.into();
        let locator = AssetLocator::open(&asset_root)?;
        let contract_bytes = locator.read(PLAYER_SHARED_RIG_CONTRACT_PATH)?;
        let mut contract: PlayerSharedRigContract = serde_json::from_slice(&contract_bytes)
            .map_err(|error| {
                format!(
                    "invalid native player-rig contract {PLAYER_SHARED_RIG_CONTRACT_PATH}: {error}"
                )
            })?;
        validate_contract(&contract)?;
        append_vehicle_animation_catalog(&mut contract, &locator)?;
        append_native_animation_catalog(
            &mut contract,
            &locator,
            "characters/player/shared/action_animations.json",
            &[
                "stun",
                "stickdash",
                "rifledash",
                "rifletumbling",
                "rocketsomersault",
                "stickdodgeupper",
                "rifledodgeupper",
            ],
        )?;
        append_native_animation_catalog(&mut contract, &locator, "characters/player/shared/damage_animations.json", &["woundupper"])?;
        let item_catalog_bytes = locator.read(PLAYER_ITEM_SET_CATALOG_PATH)?;
        let item_catalog: PlayerItemModelCatalog = serde_json::from_slice(&item_catalog_bytes)
            .map_err(|error| {
                format!("invalid player item catalog {PLAYER_ITEM_SET_CATALOG_PATH}: {error}")
            })?;
        if item_catalog.schema != PLAYER_ITEM_SET_CATALOG_SCHEMA || item_catalog.models.is_empty() {
            return Err("player item catalog has an invalid identity".to_owned());
        }
        append_native_unequipped_parts(&mut contract, &locator, &item_catalog)?;
        let table_extension_failures =
            append_native_table_skinned_wearables(&mut contract, &locator, &item_catalog)?;
        append_native_hat_variants(&mut contract, &locator, &item_catalog)?;
        append_native_skinned_backs(&mut contract, &locator, &item_catalog)?;

        let mut required_models = BTreeSet::new();
        for gender in &contract.genders {
            required_models.insert(gender.skeleton_glb.as_str());
            for part in &gender.creator_parts {
                required_models.insert(part.glb.as_str());
            }
        }
        for path in required_models {
            locator.require_file(path)?;
        }

        Ok(Self {
            asset_root,
            contract,
            table_extension_failures,
        })
    }

    #[must_use]
    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }

    #[must_use]
    pub fn contract(&self) -> &PlayerSharedRigContract {
        &self.contract
    }

    pub fn gender(&self, gender: PlayerRigGender) -> Result<&PlayerGenderRigContract, String> {
        let matches = self
            .contract
            .genders
            .iter()
            .filter(|candidate| candidate.gender == gender)
            .collect::<Vec<_>>();
        let [resolved] = matches.as_slice() else {
            return Err(format!(
                "native player-rig contract resolves {gender:?} {} times",
                matches.len()
            ));
        };
        Ok(resolved)
    }

    pub fn default_creator_routes(&self, gender: PlayerRigGender) -> Result<Vec<String>, String> {
        Ok(self.gender(gender)?.default_creator_part_routes.clone())
    }

    pub fn part_by_exact_route(
        &self,
        gender: PlayerRigGender,
        exact_route: &str,
    ) -> Result<&PlayerRigPartContract, String> {
        let matches = self
            .gender(gender)?
            .creator_parts
            .iter()
            .filter(|part| part.exact_route == exact_route)
            .collect::<Vec<_>>();
        let [part] = matches.as_slice() else {
            if matches.is_empty()
                && let Some(error) = self
                    .table_extension_failures
                    .get(&(player_rig_gender_key(gender), exact_route.to_owned()))
            {
                return Err(format!(
                    "native player-rig route {exact_route:?} is not compatible with the {gender:?} shared skeleton: {error}"
                ));
            }
            return Err(format!(
                "native player-rig route {exact_route:?} has {} certified contracts for {gender:?}",
                matches.len()
            ));
        };
        Ok(part)
    }

    pub fn part_by_glb(
        &self,
        gender: PlayerRigGender,
        glb: &str,
    ) -> Result<&PlayerRigPartContract, String> {
        let gender_contract = self.gender(gender)?;
        let matches = gender_contract
            .creator_parts
            .iter()
            .filter(|part| part.glb == glb)
            .collect::<Vec<_>>();
        if let [part] = matches.as_slice() {
            return Ok(part);
        }
        // A documented missing-asset repair may publish one donor GLB under
        // both its native route and the retained requested route. GLB-only
        // callers cannot identify a repaired alias, but the immutable default
        // creator route remains unambiguous and is safe to recover.
        let default_matches = matches
            .iter()
            .copied()
            .filter(|part| {
                gender_contract
                    .default_creator_part_routes
                    .contains(&part.exact_route)
            })
            .collect::<Vec<_>>();
        if let [part] = default_matches.as_slice() {
            return Ok(part);
        }
        Err(format!(
            "native player-rig GLB {glb:?} has {} certified contracts for {gender:?}",
            matches.len()
        ))
    }

    pub fn exact_routes_for_glbs(
        &self,
        gender: PlayerRigGender,
        glbs: &[String],
    ) -> Result<Vec<String>, String> {
        glbs.iter()
            .map(|glb| {
                self.part_by_glb(gender, glb)
                    .map(|part| part.exact_route.clone())
            })
            .collect()
    }

    /// Adds one offline-audit-only skinned wearable by matching its exported
    /// GLB joint paths to the already verified actor skeleton.
    ///
    /// This deliberately does not claim legacy `ActorWearIndexTable` PathID
    /// parity: synthesized remaps carry zero source PathIDs and
    /// `exact_transform_index_parity = false`. It exists so broad HNPC audits
    /// can render table-owned player configurations without weakening the
    /// immutable production creator contract.
    pub fn register_diagnostic_skinned_part(
        &mut self,
        gender: PlayerRigGender,
        exact_route: impl Into<String>,
        true_name: impl Into<String>,
        glb: impl Into<String>,
        actor_skin_combiner_clothes_index: u8,
    ) -> Result<(), String> {
        let exact_route = exact_route.into();
        if self
            .gender(gender)?
            .creator_parts
            .iter()
            .any(|part| part.exact_route == exact_route)
        {
            return Ok(());
        }
        let true_name = true_name.into();
        let glb = glb.into();
        let locator = AssetLocator::open(&self.asset_root)?;
        let bytes = locator.read(&glb)?;
        let rig = self.gender(gender)?.clone();
        let mut skins = synthesize_native_skin_remaps(&bytes, &true_name, &rig)?;
        for skin in &mut skins {
            skin.exact_transform_index_parity = false;
        }
        let rig = self
            .contract
            .genders
            .iter_mut()
            .find(|candidate| candidate.gender == gender)
            .ok_or_else(|| format!("native player-rig contract has no {gender:?} rig"))?;
        rig.creator_parts.push(PlayerRigPartContract {
            exact_route,
            true_name,
            glb,
            actor_skin_combiner_clothes_index,
            skins,
        });
        Ok(())
    }

    /// Adds one table-owned HNPC wearable from the published production
    /// catalog. Unlike creator parts, several of these models have no
    /// recovered `ActorWearIndexTable` PathID contract. Their remap is derived
    /// from the published GLB joint paths against the already verified actor
    /// skeleton, and remains explicitly marked as non-exact source-index
    /// parity. The HNPC catalog owns the byte proof and records that bounded
    /// divergence from the clean client.
    #[allow(clippy::too_many_arguments)]
    pub fn register_hnpc_skinned_part(
        &mut self,
        gender: PlayerRigGender,
        exact_route: impl Into<String>,
        true_name: impl Into<String>,
        glb: impl Into<String>,
        expected_bytes: u64,
        expected_blake3: &str,
        actor_skin_combiner_clothes_index: u8,
    ) -> Result<(), String> {
        let exact_route = exact_route.into();
        let true_name = true_name.into();
        let glb = glb.into();
        if let Ok(existing) = self.part_by_exact_route(gender, &exact_route) {
            if existing.true_name == true_name
                && existing.glb == glb
                && existing.actor_skin_combiner_clothes_index == actor_skin_combiner_clothes_index
            {
                return Ok(());
            }
            return Err(format!(
                "HNPC route {exact_route:?} contradicts the certified {gender:?} player-rig part"
            ));
        }

        let locator = AssetLocator::open(&self.asset_root)?;
        let bytes = locator.read_verified(&glb, Some(expected_bytes), expected_blake3)?;
        let rig = self.gender(gender)?.clone();
        let mut skins = synthesize_native_skin_remaps(&bytes, &true_name, &rig)?;
        for skin in &mut skins {
            skin.exact_transform_index_parity = false;
        }
        let rig = self
            .contract
            .genders
            .iter_mut()
            .find(|candidate| candidate.gender == gender)
            .ok_or_else(|| format!("native player-rig contract has no {gender:?} rig"))?;
        rig.creator_parts.push(PlayerRigPartContract {
            exact_route,
            true_name,
            glb,
            actor_skin_combiner_clothes_index,
            skins,
        });
        Ok(())
    }
}

pub(super) const fn player_rig_gender_key(gender: PlayerRigGender) -> u8 {
    match gender {
        PlayerRigGender::Male => 0,
        PlayerRigGender::Female => 1,
    }
}

/// One independent native player-rig instance.
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigInstance {
    pub identity: String,
    pub generation: u64,
    pub gender: PlayerRigGender,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NativePlayerRigLoadingStage {
    #[default]
    SkeletonScene,
    BoneMap,
    ModularParts,
    Stand1,
}

/// Readiness is stored per rig root, so creator previews, slot portraits,
/// local-player visuals, and remote-player visuals cannot overwrite each
/// other's state.
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub enum NativePlayerRigStatus {
    Loading(NativePlayerRigLoadingStage),
    ReadyAnimated {
        actor_bones: usize,
        parts: usize,
        skin_palettes: usize,
        skinned_surfaces: usize,
    },
    Blocked(String),
}

impl Default for NativePlayerRigStatus {
    fn default() -> Self {
        Self::Loading(NativePlayerRigLoadingStage::SkeletonScene)
    }
}

impl NativePlayerRigStatus {
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::ReadyAnimated { .. })
    }

    #[must_use]
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigBoneEntity {
    pub actor_bone_index: u32,
    pub true_name: String,
    pub full_path: String,
    pub entity: Entity,
}

/// Exact contract-to-Bevy bone lookup for one instance.
///
/// Portrait code can wait for this component, then use
/// `unique_by_true_name("Bip01 Head")`. Gameplay/attachments should prefer
/// `by_full_path` whenever the exact full path is already known.
#[derive(Component, Clone, Debug)]
pub struct NativePlayerRigBones {
    pub(super) entries: Vec<NativePlayerRigBoneEntity>,
    pub(super) by_full_path: BTreeMap<String, Entity>,
    pub(super) by_true_name: BTreeMap<String, Vec<Entity>>,
}

impl NativePlayerRigBones {
    pub(crate) fn new(entries: Vec<NativePlayerRigBoneEntity>) -> Self {
        let mut by_full_path = BTreeMap::new();
        let mut by_true_name = BTreeMap::<String, Vec<Entity>>::new();
        for entry in &entries {
            by_full_path.insert(entry.full_path.clone(), entry.entity);
            by_true_name
                .entry(entry.true_name.clone())
                .or_default()
                .push(entry.entity);
        }
        Self {
            entries,
            by_full_path,
            by_true_name,
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[NativePlayerRigBoneEntity] {
        &self.entries
    }

    #[must_use]
    pub fn by_actor_bone_index(&self, index: u32) -> Option<Entity> {
        self.entries
            .get(index as usize)
            .filter(|entry| entry.actor_bone_index == index)
            .map(|entry| entry.entity)
    }

    #[must_use]
    pub fn by_full_path(&self, path: &str) -> Option<Entity> {
        self.by_full_path.get(path).copied()
    }

    #[must_use]
    pub fn unique_by_true_name(&self, name: &str) -> Option<Entity> {
        let matches = self.by_true_name.get(name)?;
        (matches.len() == 1).then_some(matches[0])
    }
}

/// Marker on the GLB scene that owns the animated shared skeleton.
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigSkeletonScene {
    pub rig_root: Entity,
    pub generation: u64,
}

/// Marker on each independently loaded modular-part scene.
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigPartScene {
    pub rig_root: Entity,
    pub generation: u64,
    pub exact_route: String,
    pub glb: String,
}

/// Published proof that this rig instance is looping its requested idle clip.
#[derive(Component, Clone, Debug)]
pub struct NativePlayerRigStand1Playback {
    pub animation_player: Entity,
    pub animation_graph: Handle<AnimationGraph>,
    pub animation_node: AnimationNodeIndex,
}

/// Runtime clip request for a fully assembled shared-player rig. This keeps
/// remote/world consumers on the same validated skeleton instead of spawning
/// a second animation-only avatar.
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigAnimationRequest {
    pub clip: String,
    pub revision: u64,
    pub repeat: bool,
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigAnimationApplied {
    pub clip: String,
    pub revision: u64,
    pub repeat: bool,
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct NativePlayerRigAnimationIssue(pub String);

/// Caller-controlled spawn contract.
pub struct NativePlayerRigSpawnRequest {
    pub identity: String,
    pub generation: u64,
    pub gender: PlayerRigGender,
    pub exact_part_routes: Vec<String>,
    pub parent: Option<Entity>,
    pub transform: Transform,
    pub visibility: Visibility,
    pub render_layers: RenderLayers,
    /// Exact named animation embedded in the shared-skeleton GLB.
    pub animation_name: String,
    /// Static skeleton proportions, sampled from the authored body clips.
    pub body_shape: Option<NativePlayerBodyShape>,
    /// `None` uses Bevy's default asset source. `Some("character")` produces
    /// `character://...`, matching the source registered by the current app.
    pub asset_source: Option<String>,
}

impl NativePlayerRigSpawnRequest {
    #[must_use]
    pub fn new(
        identity: impl Into<String>,
        generation: u64,
        gender: PlayerRigGender,
        render_layers: RenderLayers,
    ) -> Self {
        Self {
            identity: identity.into(),
            generation,
            gender,
            exact_part_routes: Vec::new(),
            parent: None,
            transform: Transform::IDENTITY,
            visibility: Visibility::Inherited,
            render_layers,
            animation_name: "stand1".to_owned(),
            body_shape: None,
            asset_source: None,
        }
    }

    pub fn use_default_creator_parts(
        mut self,
        catalog: &NativePlayerRigCatalog,
    ) -> Result<Self, String> {
        self.exact_part_routes = catalog.default_creator_routes(self.gender)?;
        Ok(self)
    }

    pub fn use_part_routes(
        mut self,
        catalog: &NativePlayerRigCatalog,
        exact_routes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, String> {
        self.exact_part_routes = exact_routes.into_iter().map(Into::into).collect();
        for route in &self.exact_part_routes {
            catalog.part_by_exact_route(self.gender, route)?;
        }
        Ok(self)
    }

    pub fn use_part_glbs(
        mut self,
        catalog: &NativePlayerRigCatalog,
        glbs: &[String],
    ) -> Result<Self, String> {
        self.exact_part_routes = catalog.exact_routes_for_glbs(self.gender, glbs)?;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnedNativePlayerRig {
    pub root: Entity,
    pub skeleton_scene: Entity,
    pub part_scenes: Vec<Entity>,
}

#[derive(Component)]
pub(super) struct NativePlayerRigRuntime {
    pub(super) gender: PlayerGenderRigContract,
    pub(super) skeleton_gltf: Handle<Gltf>,
    pub(super) skeleton_scene: Entity,
    pub(super) skeleton_scene_handle: Handle<WorldAsset>,
    pub(super) parts: Vec<NativePlayerRigPartRuntime>,
    pub(super) animation_name: String,
}

pub(super) struct NativePlayerRigPartRuntime {
    pub(super) root: Entity,
    pub(super) scene: Handle<WorldAsset>,
    pub(super) contract: PlayerRigPartContract,
}

#[derive(Component)]
pub(super) struct NativePlayerRigSceneReady;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePlayerRigPartsBound {
    pub parts: usize,
    pub skin_palettes: usize,
    pub skinned_surfaces: usize,
}

#[derive(Component)]
pub(super) struct NativePlayerRigStand1Graph {
    pub(super) graph: Handle<AnimationGraph>,
    pub(super) node: AnimationNodeIndex,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePlayerRigSkinBound {
    pub rig_root: Entity,
    pub generation: u64,
}

/// Strong handles shared by previews, portraits, and successive creator
/// appearances.
///
/// `AssetServer::load` de-duplicates an in-flight request, but an asset can be
/// evicted as soon as the last scene instance is despawned. Character creation
/// replaces modular parts frequently, so retaining these handles avoids
/// re-reading the same GLB and rebuilding the same animation graph whenever a
/// player revisits an option or moves between selection and creation.
#[derive(Resource, Default)]
pub struct NativePlayerRigAssetCache {
    pub(super) gltfs: HashMap<String, Handle<Gltf>>,
    pub(super) scenes: HashMap<String, Handle<WorldAsset>>,
    pub(super) images: HashMap<String, Handle<Image>>,
    pub(super) animation_graphs:
        HashMap<(AssetId<Gltf>, String), (Handle<AnimationGraph>, AnimationNodeIndex)>,
    pub(super) transition_graphs:
        HashMap<AssetId<Gltf>, (Handle<AnimationGraph>, HashMap<String, AnimationNodeIndex>)>,
}

impl NativePlayerRigAssetCache {
    /// Drops cache-owned strong handles at a state boundary. Live WorldAssetRoot,
    /// image/material, and AnimationPlayer components retain the assets they
    /// still use; entries that are no longer presented become evictable.
    pub fn release_cached_handles(&mut self) {
        self.gltfs.clear();
        self.scenes.clear();
        self.images.clear();
        self.animation_graphs.clear();
        self.transition_graphs.clear();
    }

    pub(super) fn gltf(&mut self, asset_server: &AssetServer, uri: String) -> Handle<Gltf> {
        self.gltfs
            .entry(uri.clone())
            .or_insert_with(|| asset_server.load(uri))
            .clone()
    }

    pub fn scene(&mut self, asset_server: &AssetServer, uri: String) -> Handle<WorldAsset> {
        self.scenes
            .entry(uri.clone())
            .or_insert_with(|| asset_server.load(GltfAssetLabel::Scene(0).from_asset(uri)))
            .clone()
    }

    pub fn retain_image(&mut self, path: String, handle: Handle<Image>) -> Handle<Image> {
        self.images.entry(path).or_insert(handle).clone()
    }

    pub fn preload_rig_parts(
        &mut self,
        asset_server: &AssetServer,
        catalog: &NativePlayerRigCatalog,
        gender: PlayerRigGender,
        exact_routes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<(), String> {
        let gender_contract = catalog.gender(gender)?;
        let skeleton_uri = gender_contract.skeleton_glb.clone();
        self.gltf(asset_server, skeleton_uri.clone());
        self.scene(asset_server, skeleton_uri);
        for route in exact_routes {
            let part = catalog.part_by_exact_route(gender, route.as_ref())?;
            self.scene(asset_server, part.glb.clone());
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn retained_gltfs(&self) -> usize {
        self.gltfs.len()
    }

    #[cfg(test)]
    pub(super) fn retained_scenes(&self) -> usize {
        self.scenes.len()
    }

    #[cfg(test)]
    pub(super) fn retained_handles_are_strong(&self) -> bool {
        self.gltfs.values().all(Handle::is_strong)
            && self.scenes.values().all(Handle::is_strong)
            && self.images.values().all(Handle::is_strong)
    }
}

pub struct NativePlayerSharedRigPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NativePlayerRigAnimationApply;

impl Plugin for NativePlayerSharedRigPlugin {
    fn build(&self, app: &mut App) {
        crate::native_gltf::install(app);
        app.init_resource::<NativePlayerRigAssetCache>()
            .add_systems(
                Update,
                (
                    prewarm_native_player_rig_animation_graphs,
                    monitor_native_player_rig_loads,
                    propagate_native_player_rig_render_layers,
                    resolve_native_player_rig_bones,
                    prepare_native_player_rig_stand1,
                    bind_native_player_rig_parts,
                    rebind_native_player_rig_material_companions,
                    fail_closed_native_player_rig_material_metadata,
                    play_native_player_rig_stand1,
                    body_shape::apply_body_shape,
                    apply_native_player_rig_animation_requests
                        .in_set(NativePlayerRigAnimationApply),
                    finalize_native_player_rig_status,
                )
                    .chain(),
            );
    }
}

pub(super) fn prewarm_native_player_rig_animation_graphs(
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let skeletons = asset_cache.gltfs.values().cloned().collect::<Vec<_>>();
    for skeleton in skeletons {
        let cache_key = (skeleton.id(), "stand1".to_owned());
        if asset_cache.animation_graphs.contains_key(&cache_key) {
            continue;
        }
        let Some(clip) = gltfs
            .get(&skeleton)
            .and_then(|gltf| gltf.named_animations.get("stand1"))
            .cloned()
        else {
            continue;
        };
        let (graph, node) = AnimationGraph::from_clip(clip);
        let graph = graphs.add(graph);
        asset_cache
            .animation_graphs
            .insert(cache_key, (graph, node));
    }
}

pub(super) fn mark_native_player_rig_scene_ready(event: On<WorldInstanceReady>, mut commands: Commands) {
    commands
        .entity(event.event().entity)
        .insert(NativePlayerRigSceneReady);
}

pub(super) fn monitor_native_player_rig_loads(
    asset_server: Res<AssetServer>,
    mut rigs: Query<(&NativePlayerRigRuntime, &mut NativePlayerRigStatus)>,
) {
    for (runtime, mut status) in &mut rigs {
        if status.is_blocked() {
            continue;
        }
        let failure = asset_load_failure(&asset_server, &runtime.skeleton_gltf)
            .or_else(|| asset_load_failure(&asset_server, &runtime.skeleton_scene_handle))
            .map(|error| format!("shared skeleton: {error}"))
            .or_else(|| {
                runtime.parts.iter().find_map(|part| {
                    asset_load_failure(&asset_server, &part.scene)
                        .map(|error| format!("part {:?}: {error}", part.contract.exact_route))
                })
            });
        if let Some(failure) = failure {
            *status = NativePlayerRigStatus::Blocked(failure);
        }
    }
}
