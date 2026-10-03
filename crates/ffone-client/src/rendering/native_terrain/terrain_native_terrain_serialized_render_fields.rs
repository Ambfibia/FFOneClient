use super::*;

pub const NATIVE_TERRAIN_SCHEMA: &str = "ffone.native-terrain.v1";

pub const NATIVE_TERRAIN_ENVIRONMENT_SCHEMA: &str = "ffone.native-terrain-environment.v1";

pub const NATIVE_TERRAIN_MAX_LAYER_COUNT: usize = 17;

pub const NATIVE_TERRAIN_MAX_WEIGHT_MAP_COUNT: usize = 5;

pub const NATIVE_TERRAIN_HEIGHT_DENOMINATOR: u16 = 32_767;

pub(super) const LEGACY_TERRAIN_VERTEX_AMBIENT: f32 = 0.2;

// Terrain conversion is CPU-heavy (full mip-chain copies plus up to twenty
// authored grass meshes). Bound both worker pressure and main-thread asset
// admission so crossing a dong boundary cannot consume every compute worker or
// upload several complete tiles in one frame.
pub(super) const NATIVE_TERRAIN_MAX_IN_FLIGHT_PREPARATIONS: usize = 1;

pub(super) const NATIVE_TERRAIN_MAX_IN_FLIGHT_GRASS_PREPARATIONS: usize = 1;

pub(super) const NATIVE_TERRAIN_GRASS_VERTICES_PER_INSTANCE: usize = 8;

pub(super) const NATIVE_TERRAIN_GRASS_MAX_INSTANCES_PER_CHUNK: usize =
    NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME / NATIVE_TERRAIN_GRASS_VERTICES_PER_INSTANCE;

// Primary TerrainData uses 64x64 detail patches with 8x8 density samples per
// patch. Group sixteen source patches (128 world metres on the published 512m
// terrains) into one render chunk. This reduces the complete publication from
// 11,007 render entities at 64 samples to 4,247 while preserving every density
// instance and still letting Bevy reject distant parts of a resident tile. The
// hard per-chunk/per-frame vertex cap below splits unusually dense regions, and
// texture grouping retains the old client's per-atlas batching rather than
// multiplying materials by every serialized prototype.
pub(super) const NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES: usize = 128;

// Retrobution `mainData` path 28 is the single white directional light used
// by the legacy Terrain VertexLit variants. Unity stores the light's forward
// ray direction; this is the reflected native surface-to-light direction.
pub(super) const LEGACY_TERRAIN_SURFACE_TO_LIGHT: Vec3 = Vec3::new(0.893_700_85, 0.309_017_06, 0.325_280_46);

pub(super) fn legacy_terrain_vertex_light(normal: Vec3) -> Vec4 {
    let diffuse = normal.dot(LEGACY_TERRAIN_SURFACE_TO_LIGHT).max(0.0);
    let channel = (LEGACY_TERRAIN_VERTEX_AMBIENT + diffuse).clamp(0.0, 1.0);
    Vec4::new(channel, channel, channel, 1.0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeTerrainError {
    pub(super) message: String,
}

impl NativeTerrainError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NativeTerrainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for NativeTerrainError {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainBundleSource {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSource {
    pub bundle: NativeTerrainBundleSource,
    pub asset_name: String,
    pub terrain_data_path_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDimensions {
    pub width: u32,
    pub height: u32,
    pub sample_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainScale {
    pub sample_spacing_x: f64,
    pub height_scale: f64,
    pub sample_spacing_z: f64,
    pub extent_x: f64,
    pub extent_z: f64,
    pub height_normalization_denominator: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainHeightmap {
    pub path: String,
    pub png_color_type: String,
    pub raw_encoding: String,
    pub png_sample_byte_order: String,
    pub raw_hash_byte_order: String,
    pub pixel_order: String,
    pub raw_min: u16,
    pub raw_max: u16,
    pub source_order_raw_blake3: String,
    pub canonical_order_raw_blake3: String,
    pub png_blake3: String,
    #[serde(default)]
    pub vertex_shifts: Vec<NativeTerrainVertexShift>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainVertexShift {
    pub flags: u8,
    pub column: u32,
    pub row: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainVertexShiftFlagBits {
    pub negative_source_x: u8,
    pub positive_source_x: u8,
    pub negative_source_z: u8,
    pub positive_source_z: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainVertexShiftEncoding {
    pub source_field: String,
    pub source_coordinates: String,
    pub source_index: String,
    pub canonical_index: String,
    pub flag_bits: NativeTerrainVertexShiftFlagBits,
    pub position_formula: String,
    pub uv_formula: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainGameplayAttributes {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub sample_count: usize,
    pub png_color_type: String,
    pub raw_encoding: String,
    pub pixel_order: String,
    pub raw_path: String,
    pub raw_blake3: String,
    pub png_blake3: String,
    #[serde(default)]
    pub raw_parsed_document: Option<NativeTerrainHashedDocument>,
    #[serde(default)]
    pub source: Option<NativeTerrainGameplayAttributesSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainHashedDocument {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainScriptPointer {
    pub file_id: i32,
    pub path_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDecompilerEvidence {
    pub workspace_path: String,
    pub sha256: String,
    pub index_formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainGameplayAttributesSource {
    pub pointer_file_id: i32,
    pub pointer_path_id: i64,
    pub resolved_asset_name: String,
    pub resolved_path_id: i64,
    pub script_pointer: Option<NativeTerrainScriptPointer>,
    pub script_true_name: String,
    pub decompiled_evidence: NativeTerrainDecompilerEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainOrientation {
    pub source_height_layout: String,
    pub source_height_index: String,
    pub canonical_height_layout: String,
    pub canonical_height_index: String,
    pub canonical_columns: String,
    pub canonical_rows: String,
    pub weight_map_transform: String,
    pub runtime_requires_unity_orientation_fixup: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainCellIndices {
    pub i00: String,
    pub i10: String,
    pub i01: String,
    pub i11: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainGeometryContract {
    pub local_origin: [f64; 3],
    pub column_step: [f64; 3],
    pub row_step: [f64; 3],
    pub height_axis: [f64; 3],
    pub vertex_formula: String,
    #[serde(default)]
    pub vertex_shift_encoding: Option<NativeTerrainVertexShiftEncoding>,
    pub local_bounds: NativeTerrainBounds,
    pub front_face: String,
    pub cell_indices: NativeTerrainCellIndices,
    pub cell_triangle_order: String,
    pub geometric_front_normal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSceneInstanceContract {
    pub status: String,
    pub terrain_data_local_geometry_only: bool,
    pub required_for_world_placement: bool,
    pub required_source: String,
    pub application_order: String,
    pub height_formula: String,
    pub translation_is_not_baked_into_heightmap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainPointerSource {
    pub pointer_file_id: i32,
    pub pointer_path_id: i64,
    pub resolved_asset_name: String,
    pub resolved_path_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainCanonicalUv {
    pub u: String,
    pub v: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainWeightOrientation {
    pub source_decoded_rows: String,
    pub canonical_transform: String,
    pub canonical_uv: NativeTerrainCanonicalUv,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSourceEncodedMip {
    pub path: String,
    pub byte_length: usize,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainTextureMip {
    pub level: usize,
    pub width: u32,
    pub height: u32,
    pub path: String,
    pub png_color_type: String,
    pub canonical_transform: String,
    pub canonical_rgba_blake3: String,
    pub png_blake3: String,
    #[serde(default)]
    pub source_encoded: Option<NativeTerrainSourceEncodedMip>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSerializedSamplerValue {
    pub serialized_value: i64,
    pub unity_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainWrapMode {
    pub serialized_value: i64,
    pub unity_name: String,
    pub u: String,
    pub v: String,
    pub axis_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainColorSpaceContract {
    pub serialized_status: String,
    pub runtime_interpretation_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAlphaContract {
    pub serialized_status: String,
    pub runtime_contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSerializedTextureAuxiliary {
    pub complete_image_size: Option<i64>,
    pub image_count: Option<i64>,
    pub limit: Option<i64>,
    pub touchable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainTextureSampler {
    pub source_schema: String,
    pub texture_format: i64,
    pub mip_map: bool,
    pub mip_count: usize,
    pub mip_count_source: String,
    pub filter_mode: NativeTerrainSerializedSamplerValue,
    pub wrap_mode: NativeTerrainWrapMode,
    pub aniso_level: i64,
    pub mip_bias: f64,
    pub usage_color_space: String,
    pub color_space_flag: NativeTerrainColorSpaceContract,
    pub alpha_is_transparency_flag: NativeTerrainAlphaContract,
    pub serialized_auxiliary: NativeTerrainSerializedTextureAuxiliary,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainWeightMap {
    pub index: usize,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub color_space: String,
    #[serde(default)]
    pub source: Option<NativeTerrainPointerSource>,
    pub orientation: NativeTerrainWeightOrientation,
    pub canonical_rgba_blake3: String,
    pub png_blake3: String,
    #[serde(default)]
    pub sampler: Option<NativeTerrainTextureSampler>,
    #[serde(default)]
    pub mips: Vec<NativeTerrainTextureMip>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAlbedoOrientation {
    pub source_decoded_rows: String,
    pub canonical_transform: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAlbedo {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub color_space: String,
    #[serde(default)]
    pub source: Option<NativeTerrainPointerSource>,
    pub orientation: NativeTerrainAlbedoOrientation,
    pub canonical_rgba_blake3: String,
    pub png_blake3: String,
    #[serde(default)]
    pub sampler: Option<NativeTerrainTextureSampler>,
    #[serde(default)]
    pub mips: Vec<NativeTerrainTextureMip>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainTileSize {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainLayerWeight {
    pub map_index: usize,
    pub map_path: String,
    pub channel_index: usize,
    pub channel: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainLayer {
    pub index: usize,
    pub true_texture_name: String,
    pub mode: i64,
    #[serde(default)]
    pub mode_source: Option<String>,
    #[serde(default)]
    pub mode_evidence: Option<serde_json::Value>,
    pub tile_size: NativeTerrainTileSize,
    pub albedo: NativeTerrainAlbedo,
    pub weight: NativeTerrainLayerWeight,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSplat {
    pub resolution: u32,
    #[serde(default)]
    pub base_map_resolution: Option<u32>,
    pub weight_maps: Vec<NativeTerrainWeightMap>,
    pub layers: Vec<NativeTerrainLayer>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainOptionalLightmap {
    pub status: String,
    #[serde(default)]
    pub source_pointer: Option<NativeTerrainScriptPointer>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub color_space: Option<String>,
    #[serde(default)]
    pub source: Option<NativeTerrainPointerSource>,
    #[serde(default)]
    pub canonical_transform: Option<String>,
    #[serde(default)]
    pub canonical_rgba_blake3: Option<String>,
    #[serde(default)]
    pub png_blake3: Option<String>,
    #[serde(default)]
    pub sampler: Option<NativeTerrainTextureSampler>,
    #[serde(default)]
    pub mips: Option<Vec<NativeTerrainTextureMip>>,
    #[serde(default)]
    pub runtime_selection: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDetailAssetClosure {
    pub status: String,
    pub blocked_count: usize,
    pub blocked_graph_root_count: usize,
    pub blocked_preload_pointer_count: usize,
    #[serde(default)]
    pub prototype_texture_reference_count: Option<usize>,
    #[serde(default)]
    pub unique_resolved_texture_count: Option<usize>,
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDetailTextureDocument {
    pub true_texture_name: String,
    pub resolved_asset_name: String,
    pub resolved_path_id: i64,
    pub serialized_object_raw_blake3: String,
    pub document_path: String,
    pub document_blake3: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDetailAndTrees {
    pub status: String,
    #[serde(default)]
    pub asset_closure: Option<NativeTerrainDetailAssetClosure>,
    #[serde(default)]
    pub raw_document: Option<NativeTerrainHashedDocument>,
    #[serde(default)]
    pub patch_grid: Option<serde_json::Value>,
    #[serde(default)]
    pub waving_grass: Option<serde_json::Value>,
    #[serde(default)]
    pub prototypes: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub textures: Option<Vec<NativeTerrainDetailTextureDocument>>,
    #[serde(default)]
    pub random_rotations: Option<serde_json::Value>,
    #[serde(default)]
    pub preload_texture_atlas_data: Option<serde_json::Value>,
    #[serde(default)]
    pub trees: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainEnvironmentReference {
    pub schema: String,
    pub status: String,
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAmbience {
    pub status: String,
    pub grid_coordinates: [i32; 2],
    pub fog_depth: f64,
    pub fog_color: [f64; 4],
    pub sky_color: [f64; 4],
    pub light_color: [f64; 4],
    #[serde(default)]
    pub source_object: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDefaultAmbienceApplication {
    pub fog_alpha_after_scope_blend: f64,
    pub fog_color: String,
    pub fog_density: String,
    pub fog_enabled: String,
    pub light_color: String,
    pub source: String,
    pub tutorial_blend_applies_to_this_scope: bool,
    pub tutorial_fog_color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAmbienceRegistration {
    pub grid_index_formula: String,
    pub grid_width: usize,
    pub operation: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAmbienceZeroWeightFallback {
    pub fog_depth: f64,
    pub fog_color: [f64; 4],
    pub sky_color: [f64; 4],
    pub light_color: [f64; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAmbienceLocalCoordinates {
    pub x: String,
    pub y: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainAmbienceSampling {
    pub fraction_remap: String,
    pub local_coordinates: NativeTerrainAmbienceLocalCoordinates,
    pub missing_tile_policy: String,
    pub neighbor_selection: String,
    pub normalization: String,
    pub source: String,
    pub weights: [String; 4],
    pub zero_weight_fallback: NativeTerrainAmbienceZeroWeightFallback,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainRuntimeAmbienceContract {
    pub default_ambience_application: NativeTerrainDefaultAmbienceApplication,
    pub registration: NativeTerrainAmbienceRegistration,
    pub runtime_policy: String,
    pub sampling: NativeTerrainAmbienceSampling,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTerrainSerializedRenderFields {
    #[serde(rename = "m_CastShadows")]
    pub cast_shadows: i64,
    #[serde(rename = "m_DebugDrawMainCamera")]
    pub debug_draw_main_camera: i64,
    #[serde(rename = "m_DetailObjectDistance")]
    pub detail_object_distance: f64,
    #[serde(rename = "m_Enabled")]
    pub enabled: bool,
    #[serde(rename = "m_HeightmapMaximumLOD")]
    pub heightmap_maximum_lod: i64,
    #[serde(rename = "m_HeightmapPixelError")]
    pub heightmap_pixel_error: f64,
    #[serde(rename = "m_RenderMode")]
    pub render_mode: i64,
    #[serde(rename = "m_SplatMapDistance")]
    pub splat_map_distance: f64,
    #[serde(rename = "m_TreeBillboardDistance")]
    pub tree_billboard_distance: f64,
    #[serde(rename = "m_TreeCrossFadeLength")]
    pub tree_cross_fade_length: f64,
    #[serde(rename = "m_TreeDistance")]
    pub tree_distance: f64,
    #[serde(rename = "m_TreeMaximumFullLODCount")]
    pub tree_maximum_full_lod_count: i64,
    #[serde(rename = "m_UseLightmap")]
    pub use_lightmap: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainSerializedRenderBaseline {
    pub selection: String,
    pub serialized_fields: NativeTerrainSerializedRenderFields,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainEnvironmentDetail {
    pub runtime_option_bindings: serde_json::Value,
    pub serialized_terrain_render_baseline: NativeTerrainSerializedRenderBaseline,
    #[serde(default)]
    pub source_object: Option<serde_json::Value>,
    pub status: String,
    #[serde(default)]
    pub terrain_renderer_source_object: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainEnvironment {
    pub ambience: NativeTerrainAmbience,
    #[serde(default)]
    pub map_scene: Option<serde_json::Value>,
    #[serde(default)]
    pub placement_audit: Option<serde_json::Value>,
    pub runtime_ambience_contract: NativeTerrainRuntimeAmbienceContract,
    pub schema: String,
    pub scope: String,
    #[serde(default)]
    pub source_code_evidence: Vec<serde_json::Value>,
    pub status: String,
    pub terrain_detail: NativeTerrainEnvironmentDetail,
    pub tile_id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeTerrainDescriptor {
    pub schema: String,
    pub true_name: String,
    #[serde(default)]
    pub source: Option<NativeTerrainSource>,
    pub dimensions: NativeTerrainDimensions,
    pub scale: NativeTerrainScale,
    pub heightmap: NativeTerrainHeightmap,
    pub orientation: NativeTerrainOrientation,
    pub native_geometry: NativeTerrainGeometryContract,
    #[serde(default)]
    pub scene_instance: Option<NativeTerrainSceneInstanceContract>,
    pub splat: NativeTerrainSplat,
    #[serde(default)]
    pub lightmap: Option<NativeTerrainOptionalLightmap>,
    #[serde(default)]
    pub detail_and_trees: Option<NativeTerrainDetailAndTrees>,
    #[serde(default)]
    pub gameplay_attributes: Option<NativeTerrainGameplayAttributes>,
    #[serde(default)]
    pub environment: Option<NativeTerrainEnvironmentReference>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NativeTerrainGrassLayer {
    pub(super) density_width: u32,
    pub(super) density_height: u32,
    pub(super) density: Arc<[u8]>,
    pub(super) texture_path: String,
    pub(super) healthy_color: [f32; 4],
    pub(super) dry_color: [f32; 4],
    pub(super) minimum_width: f32,
    pub(super) maximum_width: f32,
    pub(super) minimum_height: f32,
    pub(super) maximum_height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTerrainGeometry {
    pub(super) positions: Arc<[[f32; 3]]>,
    pub(super) normals: Arc<[[f32; 3]]>,
    pub(super) uvs: Arc<[[f32; 2]]>,
    pub(super) indices: Arc<[u32]>,
}
