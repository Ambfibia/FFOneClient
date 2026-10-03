use super::*;

pub const NATIVE_WORLD_CATALOG_SCHEMA: &str = "ffone.semantic-world-catalog.v2";

pub const LEGACY_NATIVE_WORLD_CATALOG_SCHEMA: &str = "ffone.semantic-world-catalog.v1";

pub const NATIVE_WORLD_CATALOG_PATH: &str = "map/catalog.json";

pub const RUNTIME_WORLD_REGISTRY_SCHEMA: &str = "ffone.runtime-world.v1";

pub const MAP_CATALOG_SCHEMA: &str = "ffone.map-catalog.v1";

/// Compatibility fallback for asset trees created before the unified map catalog.
pub const RUNTIME_WORLD_REGISTRY_PATH: &str = "_runtime/world.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct NativeWorldCatalogBlocker {
    pub(super) scope: String,
    pub(super) instance_id: String,
    pub(super) stage: String,
    pub(super) code: String,
    pub(super) message: String,
    pub(super) placement_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct NativeWorldCatalogEnvironment {
    pub(super) schema: String,
    pub(super) status: String,
    pub(super) path: String,
    pub(super) blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct NativeWorldCatalogEntry {
    pub(super) scope: NativeWorldScope,
    pub(super) instance_id: String,
    pub(super) tile: [i32; 2],
    pub(super) placement_status: String,
    pub(super) scene: Option<String>,
    #[serde(default)]
    pub(super) scene_blake3: Option<String>,
    pub(super) terrain_descriptor: String,
    pub(super) terrain_descriptor_blake3: String,
    pub(super) provenance: String,
    #[serde(default)]
    pub(super) environment: Option<NativeWorldCatalogEnvironment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct NativeWorldCatalogV2 {
    pub(super) schema: String,
    pub(super) source_build: String,
    pub(super) status: String,
    pub(super) terrain_glb_allowed: bool,
    pub(super) entries: Vec<NativeWorldCatalogEntry>,
    pub(super) blocked: Vec<NativeWorldCatalogBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RuntimeWorldAssetReference {
    pub(super) path: String,
    pub(super) blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RuntimeWorldRegistryEntry {
    pub(super) id: String,
    pub(super) scope: NativeWorldScope,
    pub(super) tile: [i32; 2],
    pub(super) scene: RuntimeWorldAssetReference,
    pub(super) terrain: RuntimeWorldAssetReference,
    #[serde(default)]
    pub(super) environment: Option<RuntimeWorldAssetReference>,
    #[serde(default)]
    pub(super) behaviour: Option<RuntimeWorldAssetReference>,
    #[serde(default)]
    pub(super) objects: Option<RuntimeWorldAssetReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RuntimeWorldRegistry {
    pub(super) schema: String,
    pub(super) entries: Vec<RuntimeWorldRegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MapCatalogArtifact {
    pub(super) path: String,
    pub(super) blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MapCatalogTileReference {
    pub(super) tile_id: String,
    pub(super) manifest: MapCatalogArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RuntimeMapCatalog {
    pub(super) schema: String,
    pub(super) tiles: Vec<MapCatalogTileReference>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct LegacyNativeWorldCatalogMap {
    pub(super) coordinate_contract: NativeWorldCoordinateContract,
    pub(super) id: String,
    pub(super) legacy_tile_name: String,
    pub(super) native_terrain_instance: NativeWorldTerrainInstance,
    pub(super) payload_files: Vec<String>,
    pub(super) provenance: String,
    pub(super) root_transform: AuthoredWorldTransform,
    pub(super) scene: String,
    pub(super) scene_blake3: String,
    pub(super) terrain_descriptor: String,
    pub(super) terrain_descriptor_blake3: String,
    pub(super) true_terrain_data_name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct NativeWorldCatalogV1 {
    pub(super) maps: Vec<LegacyNativeWorldCatalogMap>,
    pub(super) runtime_migration_pending: bool,
    pub(super) schema: String,
    pub(super) source_aliases_retained_in_runtime_assets: bool,
    pub(super) source_build: String,
    pub(super) status: String,
    pub(super) terrain_glb_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub(super) enum NativeWorldCatalogDocument {
    V2(NativeWorldCatalogV2),
    V1(NativeWorldCatalogV1),
}

/// Catalog metadata is loaded eagerly, but all Gray16/PNG/mip payload remains
/// lazy and is bounded by a nine-tile LRU cache.
#[derive(Debug, Clone, Resource)]
pub struct NativeWorldCatalog {
    pub(super) asset_root: PathBuf,
    pub(super) scenes: Vec<NativeWorldScene>,
    pub(super) water_visual_indices: HashMap<(NativeWorldScope, [i32; 2]), Box<[usize]>>,
    pub(super) world_by_dong_key: HashMap<i32, usize>,
    pub(super) tutorial_by_dong_key: HashMap<i32, usize>,
    pub(super) presentation_footprints:
        HashMap<(NativeWorldScope, [i32; 2]), NativeWorldPresentationFootprint>,
    pub(super) blocked: Vec<NativeWorldCatalogBlocker>,
    pub(super) terrain_cache: Arc<Mutex<NativeTerrainCache>>,
    pub(super) world_ambience: NativeTerrainAmbienceGrid,
    pub(super) tutorial_ambience: NativeTerrainAmbienceGrid,
    pub(super) runtime_behaviour_assets: HashMap<(NativeWorldScope, String), RuntimeWorldAssetReference>,
    pub(super) runtime_object_assets: HashMap<(NativeWorldScope, String), RuntimeWorldAssetReference>,
}
