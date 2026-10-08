//! Native FusionFall terrain loading, verification, geometry and splat rendering.
//!
//! Terrain is deliberately not represented as glTF. The source of truth is a
//! readable `terrain.json`, a lossless Gray16 heightmap, linear RGBA weight
//! maps and semantic PNG albedo layers. Every file is verified before Bevy is
//! allowed to create either render geometry or grounded collision.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    fmt, fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex, OnceLock, Weak},
};

use bevy::{
    asset::{AssetPath, RenderAssetUsages, embedded_asset, embedded_path},
    camera::visibility::VisibilityRange,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, Extent3d, ShaderType, TextureDataOrder, TextureDimension, TextureFormat,
        TextureViewDescriptor, TextureViewDimension,
    },
    shader::ShaderRef,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use image::{ColorType, DynamicImage, ImageFormat};
use serde::Deserialize;
mod editing;

use crate::world::{NativeWorldColliderStatus, NativeWorldSceneEntity};

#[cfg(test)]
mod tests;

mod terrain_native_terrain_serialized_render_fields;
mod terrain_native_terrain;
mod terrain_native_heightmap_collider;
mod terrain_materialize_native_terrain_grass;
mod codec;
mod textures;
mod types;
mod validation;
mod input;
mod models;
mod operations;
mod materials;
mod entities;

pub use terrain_native_terrain_serialized_render_fields::{
    NATIVE_TERRAIN_SCHEMA, NATIVE_TERRAIN_ENVIRONMENT_SCHEMA, NATIVE_TERRAIN_MAX_LAYER_COUNT,
    NATIVE_TERRAIN_MAX_WEIGHT_MAP_COUNT, NATIVE_TERRAIN_HEIGHT_DENOMINATOR,
    NativeTerrainError, NativeTerrainBundleSource, NativeTerrainSource,
    NativeTerrainDimensions, NativeTerrainScale, NativeTerrainHeightmap,
    NativeTerrainVertexShift, NativeTerrainVertexShiftFlagBits,
    NativeTerrainVertexShiftEncoding, NativeTerrainGameplayAttributes,
    NativeTerrainHashedDocument, NativeTerrainScriptPointer, NativeTerrainDecompilerEvidence,
    NativeTerrainGameplayAttributesSource, NativeTerrainOrientation, NativeTerrainBounds,
    NativeTerrainCellIndices, NativeTerrainGeometryContract,
    NativeTerrainSceneInstanceContract, NativeTerrainPointerSource, NativeTerrainCanonicalUv,
    NativeTerrainWeightOrientation, NativeTerrainSourceEncodedMip, NativeTerrainTextureMip,
    NativeTerrainSerializedSamplerValue, NativeTerrainWrapMode,
    NativeTerrainColorSpaceContract, NativeTerrainAlphaContract,
    NativeTerrainSerializedTextureAuxiliary, NativeTerrainTextureSampler,
    NativeTerrainWeightMap, NativeTerrainAlbedoOrientation, NativeTerrainAlbedo,
    NativeTerrainTileSize, NativeTerrainLayerWeight, NativeTerrainLayer, NativeTerrainSplat,
    NativeTerrainOptionalLightmap, NativeTerrainDetailAssetClosure,
    NativeTerrainDetailTextureDocument, NativeTerrainDetailAndTrees,
    NativeTerrainEnvironmentReference, NativeTerrainAmbience,
    NativeTerrainDefaultAmbienceApplication, NativeTerrainAmbienceRegistration,
    NativeTerrainAmbienceZeroWeightFallback, NativeTerrainAmbienceLocalCoordinates,
    NativeTerrainAmbienceSampling, NativeTerrainRuntimeAmbienceContract,
    NativeTerrainSerializedRenderFields, NativeTerrainSerializedRenderBaseline,
    NativeTerrainEnvironmentDetail, NativeTerrainEnvironment, NativeTerrainDescriptor,
    NativeTerrainGeometry
};
use terrain_native_terrain_serialized_render_fields::{
    NATIVE_TERRAIN_MAX_IN_FLIGHT_PREPARATIONS, NATIVE_TERRAIN_MAX_IN_FLIGHT_GRASS_PREPARATIONS,
    NATIVE_TERRAIN_GRASS_MAX_INSTANCES_PER_CHUNK, NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES,
    legacy_terrain_vertex_light, NativeTerrainGrassLayer
};
#[cfg(test)]
use terrain_native_terrain_serialized_render_fields::{
    LEGACY_TERRAIN_VERTEX_AMBIENT, LEGACY_TERRAIN_SURFACE_TO_LIGHT
};
pub use terrain_native_terrain::{
    NativeTerrain, NATIVE_TERRAIN_ANISOTROPIC_TAPS, NATIVE_TERRAIN_ISOTROPIC_TAPS,
    NativeTerrainMaterialUniform, NativeTerrainMaterial, PendingNativeHeightmapTerrain,
    NativeHeightmapTerrain, NativeTerrainGameplayAttributesStatus,
    NativeTerrainGameplayAttributeSample, NativeTerrainLegacyInterpolatedHeightStatus,
    NativeTerrainLegacyInterpolatedHeightSample,
    NativeTerrainLegacyInterpolatedHeightRejection, LegacyTerrainInterpolatedHeightEvidence,
    LEGACY_TERRAIN_INTERPOLATED_HEIGHT_EVIDENCE, NativeTerrainDetailStatus,
    NativeTerrainTreeStatus, NativeHeightmapCollider
};
use terrain_native_terrain::{
    validate_splat_contract, PreparedNativeHeightmapTerrain, PreparedNativeTerrainGrassLayer,
    PreparedNativeTerrainGrassChunk, PendingNativeTerrainGrassPreparation,
    PendingNativeTerrainGrass, NATIVE_TERRAIN_SEGMENT_EPSILON
};
#[cfg(test)]
use terrain_native_terrain::load_heightmap;
pub(crate) use terrain_native_terrain::NativeTerrainSegmentHit;
use terrain_native_heightmap_collider::{
    prepare_native_terrain_grass, native_terrain_grass_preparation_slots
};
#[cfg(test)]
use terrain_native_heightmap_collider::native_heightmap_preparation_slots;
pub use terrain_native_heightmap_collider::{
    NativeTerrainVisualParityStatus, LegacyTerrainTextureMode, LegacyTerrainRenderMode,
    LegacyTerrainCompositorLayer, legacy_terrain_pass_composite, LegacyTerrainShaderEvidence,
    LEGACY_TERRAIN_SHADER_EVIDENCE
};
pub(crate) use terrain_native_heightmap_collider::install_native_terrain;

/// Installs native terrain presentation for standalone authoring/preview apps.
/// Gameplay installs the same systems through `NativeWorldPlugin`.
pub struct NativeTerrainPlugin;
impl Plugin for NativeTerrainPlugin {
    fn build(&self, app: &mut App) {
        install_native_terrain(app);
    }
}
use terrain_materialize_native_terrain_grass::{
    prepare_pending_native_terrain_grass, materialize_native_terrain_grass, terrain_material_uniform
};
#[cfg(test)]
use terrain_materialize_native_terrain_grass::{
    NativeTerrainGrassAdmissionBudget, native_terrain_detail_texture_asset_path
};
use codec::{
    NATIVE_TERRAIN_MATERIALIZATIONS_PER_FRAME, NATIVE_TERRAIN_GRASS_CHUNKS_PER_FRAME,
    NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME, VerifiedRgbaPayloadCacheKey, decode_png
};
use textures::{
    VerifiedRgbaImage, validate_texture_contract_metadata, load_rgba_image, load_texture_mip_files
};
#[cfg(test)]
use textures::load_verified_rgba_image;
#[cfg(test)]
use textures::verify_texture_mip_files;
use types::VerifiedGameplayAttributes;
use validation::{
    validate_descriptor, validate_environment, validate_pointer_source, validate_plain_blake3,
    validate_prefixed_blake3, validate_readable_name, validate_relative_path
};
use input::{load_environment, load_gameplay_attributes, load_grass_layers, read_file};
use models::{validate_vertex_shifts, finish_grass_mesh_chunk};
use operations::{
    build_geometry, materialize_native_heightmaps, build_grass_chunk_meshes, weight_array_image,
    layer_array_image, lightmap_image, u16_little_endian_bytes, verify_plain_hash,
    verify_prefixed_hash, join_relative, approximately_equal
};
use materials::legacy_realtime_blend_group;
pub use entities::spawn_pending_native_heightmap;
