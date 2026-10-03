//! Native, fail-closed loading for authored FusionFall world scenes.
//!
//! A world scene is a readable JSON graph. Props/characters may reference
//! ordinary Bevy GLB assets; terrain is always the native Gray16 heightmap
//! contract. All transforms are already in the shared native coordinate space:
//! no runtime Unity-axis conversion, origin shift, auto-centering, or
//! character-facing half-turn is permitted here.
//!
//! World discovery is driven by the unified `map/catalog.json`. Each map tile
//! owns its terrain, object placements, behaviour and native scene while all
//! reusable geometry and terrain layers live in shared map packages.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    error::Error,
    fmt, fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

use bevy::{
    asset::LoadState,
    camera::{
        primitives::{Aabb, MeshAabb},
        visibility::VisibilityRange,
    },
    gltf::{
        Gltf, GltfAssetLabel, GltfExtras, GltfMaterialExtras, GltfMaterialName, GltfMesh,
        GltfMeshExtras, GltfMeshName,
    },
    mesh::{Indices, MeshTag, VertexAttributeValues},
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures_lite::future},
    world_serialization::WorldInstanceReady,
};
use serde::Deserialize;

use crate::{
    coordinates::NATIVE_COORDINATE_CONTRACT_SCHEMA,
    entity_lifecycle::{NetworkNpc0104, NetworkNpcAppearance0104, NetworkRemotePc0104},
    legacy_environment::LegacyWaterSurface,
    legacy_model_material::{LegacyMaterialApplied, LegacyMaterialMetadataError},
    movement::{
        LEGACY_COLLISION_BELOW, LegacyCollisionMode, LegacyMovementSet, LegacyOrbitCamera,
        LegacyPlayerController, MovementIntentQueue,
    },
    native_terrain::{
        NATIVE_TERRAIN_ENVIRONMENT_SCHEMA, NativeHeightmapCollider, NativeTerrain,
        NativeTerrainEnvironment, NativeTerrainEnvironmentReference,
        NativeTerrainGameplayAttributes, NativeTerrainDetailStatus, install_native_terrain, spawn_pending_native_heightmap,
    },
    network_world_runtime::NetworkNpcGrounding0104,
    remote::{RemoteAnimation, RemoteAnimationState, interpolate_remote_players},
    terrain_ambience::{
        NativeTerrainAmbienceGrid, NativeTerrainAmbienceSample, NativeTerrainAppliedAmbience,
        apply_default_ambience,
    },
};

#[cfg(test)]
#[path = "world/streaming_tests.rs"]
mod streaming_tests;

#[cfg(test)]
#[path = "world/ground_triangle_index_tests.rs"]
mod ground_triangle_index_tests;

#[cfg(test)]
mod tests;


#[cfg(test)]
#[path = "world/rail_visibility_tests.rs"]
mod rail_visibility_tests;

#[cfg(test)]
#[path = "world/managed_visual_pass_tests.rs"]
mod managed_visual_pass_tests;

#[cfg(test)]
#[path = "world/location_presentation_tests.rs"]
mod location_presentation_tests;

mod constants;
mod assets_native_world_catalog;
mod assets_native_world_catalog_native_world_catalog;
mod input_resolve_authored_world_ground;
mod player_contact_shadow;
mod input_resolve_authored_wall_substep;
mod terrain;
mod codec;
mod containers;
mod models;
mod collision_authored_collider_from_mesh;
mod collision_collider_capsule_ground_contact_with_bounds;
mod collision_prepared_ground_collider;
mod validation;
mod types_native_world_scene;
mod types_native_world_plugin;
mod state;
mod operations_materialize_loaded_native_world_visual_asset;
mod operations_reveal_native_world_scenes;
mod operations_cook_convex_xy_prism;
mod materials;
mod entities;
mod animation;

pub use constants::{
    NATIVE_WORLD_SCENE_SCHEMA, LEGACY_NATIVE_WORLD_SCENE_SCHEMA, MAP_TILE_SCHEMA,
    FIRST_NATIVE_WORLD_SCENE, NATIVE_WORLD_SCENE_PATHS, WORLD_TILE_SIZE_NATIVE,
    LEGACY_DONG_UNLOAD_DISTANCE_NATIVE, LEGACY_WORLD_CAMERA_FAR_NATIVE,
    EXTENDED_WORLD_CAMERA_FAR_NATIVE, EXTENDED_DONG_UNLOAD_DISTANCE_NATIVE,
    NATIVE_WORLD_MAX_RESIDENT_TILES, AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
    AUTHORED_CHARACTER_CONTROLLER_RADIUS
};
use constants::{
    NATIVE_WORLD_MAX_IN_FLIGHT_VISUAL_ASSETS, NATIVE_WORLD_MAX_PENDING_COLLIDERS,
    NATIVE_WORLD_RANGE_GROUP_FINALIZATIONS_PER_ROOT, NATIVE_WORLD_RANGE_PER_RADIUS,
    PRIMARY_AUDITED_SMALLSTUFF_PRESENTATION_END_NATIVE, NATIVE_WORLD_RANGE_CAMERA_STEP,
    WORLD_BASIS, WORLD_UNIT_SCALE, WORLD_ORIGIN_POLICY, UNIT_QUATERNION_TOLERANCE,
    GROUND_EPSILON, GROUNDED_STEP_UP, AUTHORED_WALKABLE_MAX_TANGENT,
    AUTHORED_GROUND_SWEEP_TOLERANCE, REMOTE_GROUND_STEP_DOWN, REMOTE_GROUND_STEP_UP,
    AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH, AUTHORED_WALKABLE_MIN_UP_DOT,
    AUTHORED_MAX_CONTACT_PLANES, NATIVE_WORLD_RANGE_X_BITS, NATIVE_WORLD_RANGE_Z_BITS,
    NATIVE_WORLD_RANGE_Y_BITS, NATIVE_WORLD_RANGE_END_BITS, NATIVE_WORLD_RANGE_END_LEVELS
};
pub use assets_native_world_catalog::{
    NATIVE_WORLD_CATALOG_SCHEMA, LEGACY_NATIVE_WORLD_CATALOG_SCHEMA,
    NATIVE_WORLD_CATALOG_PATH, RUNTIME_WORLD_REGISTRY_SCHEMA, MAP_CATALOG_SCHEMA,
    RUNTIME_WORLD_REGISTRY_PATH, NativeWorldCatalog
};
use assets_native_world_catalog::{
    NativeWorldCatalogEnvironment, NativeWorldCatalogEntry, RuntimeWorldAssetReference,
    RuntimeWorldRegistryEntry, RuntimeWorldRegistry, MapCatalogArtifact, RuntimeMapCatalog,
    NativeWorldCatalogDocument
};
use assets_native_world_catalog_native_world_catalog::PendingNativeWorldVisualAsset;
#[cfg(test)]
use assets_native_world_catalog_native_world_catalog::build_catalog_indices;
pub use input_resolve_authored_world_ground::{
    LEGACY_DONG_LOAD_DISTANCE_NATIVE, EXTENDED_DONG_LOAD_DISTANCE_NATIVE,
    load_first_native_world_scene, load_native_world_scenes
};
use input_resolve_authored_world_ground::resolve_authored_world_ground;
#[cfg(test)]
use input_resolve_authored_world_ground::resolve_authored_controller_motion_with_bounds;
use input_resolve_authored_wall_substep::{
    resolve_authored_wall_motion_detailed_with_bounds,
    resolve_authored_downward_motion_detailed_with_bounds, resolve_authored_wall_motion,
    resolve_authored_camera_occlusion, read_runtime_world_reference, load_runtime_environment
};
#[cfg(test)]
use input_resolve_authored_wall_substep::collect_authored_motion_triangles;
use terrain::{
    NATIVE_WORLD_TERRAIN_LOADS_IN_FLIGHT, TERRAIN_RECOVERY_VERTICAL_LIMIT, NativeTerrainCache,
    load_native_world_terrain, PendingNativeWorldSceneTerrainLoad,
    sync_native_terrain_spatial_registry, authored_terrain_motion_patch,
    terrain_capsule_ground_contact
};
pub use terrain::{
    NativeTerrainSpatialLookup, NativeTerrainSpatialRegistryStatus,
    NativeTerrainSpatialRegistry, NativeTerrainRenderSerializedFields,
    NativeTerrainRenderContract, NativeTerrainSceneMapReference,
    NativeTerrainSceneTerrainData, NativeTerrainSceneGameObject, NativeTerrainResolvedPointer,
    NativeTerrainColliderContract, NativeTerrainSceneCollider,
    NativeTerrainSceneTransformLink, NativeTerrainSceneComponent, NativeTerrainSceneLinkage,
    NativeTerrainSceneCoordinateContract, NativeTerrainSidecarReference,
    NativeTerrainRawSidecarReference, NativeTerrainOwnerScriptEvidence,
    NativeTerrainOwnerComponent, NativeTerrainSceneInstance, NativeWorldTerrainInstance
};
use codec::{
    NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME, NATIVE_WORLD_VISUAL_FINALIZATIONS_PER_FRAME,
    NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME, NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME,
    NATIVE_WORLD_RANGE_ENTITY_VISITS_PER_FRAME,
    NATIVE_WORLD_RANGE_GROUP_FINALIZATIONS_PER_FRAME, AUTHORED_COLLIDER_COOKS_PER_FRAME,
    AUTHORED_CACHED_COLLIDER_MATERIALIZATIONS_PER_FRAME, AUTHORED_COLLIDER_VERTICES_PER_FRAME,
    AUTHORED_COLLIDER_INDICES_PER_FRAME, NativeWorldSceneUnloadFrame
};
use containers::{
    NATIVE_WORLD_OBJECT_FADE_BAND, NativeWorldObjectRangeMember, NativeWorldObjectRangeGroups,
    PendingNativeWorldObjectBounds, NativeWorldObjectRangeContract, NativeWorldObjectRangeReady,
    NativeWorldObjectRangeUnbounded, PendingNativeWorldObjectRangeTag,
    NativeWorldObjectRangeCameraCache, native_world_object_range_group_key,
    unpack_native_world_object_range, prepare_native_world_object_ranges,
    propagate_native_world_object_ranges_to_late_meshes, update_native_world_object_residency
};
#[cfg(test)]
use containers::{
    NativeWorldObjectRangeGroupKey, native_world_prefab_owner, pack_native_world_object_range,
    native_world_object_range
};
pub use containers::{
    NativeSerializedPointer, NativeWorldRootChainGameObject, NativeWorldSerializedTransform
};
pub(crate) use containers::NativeWorldDynamicObjectRange;
use models::{
    PRIMARY_ETC_TREE_07_FROND_MODEL_PATH, PRIMARY_ETC_TREE_07_TRUNK_MODEL_PATH,
    matches_primary_darktree_bridge_model_family, NativeWorldMeshAabbCache,
    LEGACY_ZERO_CONTRIBUTION_MODEL_ID, LEGACY_ZERO_CONTRIBUTION_MODEL_PATH,
    LEGACY_ZERO_CONTRIBUTION_MODEL_BLAKE3, is_canonical_native_world_water_model_path,
    materialize_authored_tri_mesh_colliders, validate_single_root_world_glb
};
pub use models::NativeWorldModel;
use collision_authored_collider_from_mesh::{
    AUTHORED_COLLISION_EPSILON, AUTHORED_COLLISION_CONTACT_TOLERANCE,
    AUTHORED_COLLISION_MAX_SUBSTEP, AUTHORED_COLLISION_MAX_SUBSTEPS, PendingAuthoredTriMeshCollider,
    RuntimeAttachedAuthoredModelCollider, RuntimeAuthoredColliderMotion, AuthoredColliderCooking,
    AuthoredColliderGeometry, AuthoredColliderGeometryCacheKey, AuthoredColliderGeometryCache,
    AuthoredColliderSpatialIndex, is_legacy_collision_helper_visual, native_world_collider_cooking,
    AuthoredColliderCookBudget, authored_collider_from_cached_geometry,
    finish_authored_collider_materialization, sync_authored_collider_world_bounds,
    sync_authored_collider_spatial_index, sync_runtime_attached_authored_collider_transforms,
    authored_collider_from_mesh, advance_network_npc_grounding, AuthoredCollisionTriangle
};
#[cfg(test)]
use collision_authored_collider_from_mesh::has_legacy_collision_helper_identity;
pub use collision_authored_collider_from_mesh::{
    NativeWorldColliderKind, NativeWorldCollider, SpawnedNativeWorldCollider,
    NativeWorldColliderStatus, AuthoredTriMeshCollider, AuthoredColliderWorldBounds
};
#[cfg(test)]
use collision_authored_collider_from_mesh::legacy_collision_flag_for_normal;
use collision_collider_capsule_ground_contact_with_bounds::{
    resolve_runtime_authored_collider_motion, authored_capsule_is_on_triangle_front_side,
    closest_points_segment_triangle, triangle_wall_hit, collider_capsule_ground_contact_with_bounds,
    collider_world_bounds, triangle_height_at_xz, PreparedGroundTriangle, GroundTriangleCell
};
#[cfg(test)]
use collision_collider_capsule_ground_contact_with_bounds::segment_triangle_hit;
#[cfg(test)]
use collision_collider_capsule_ground_contact_with_bounds::{
    collider_wall_penetration, collider_wall_hit, collider_world_bounds_overlap
};
pub(crate) use collision_collider_capsule_ground_contact_with_bounds::{
    collider_segment_hit_with_bounds, collider_ground_height
};
pub use collision_collider_capsule_ground_contact_with_bounds::{
    authored_collider_blocks_segment, authored_collider_blocks_segment_with_bounds,
    collider_ground_height_with_bounds
};
use collision_prepared_ground_collider::GroundTriangleCache;
#[cfg(test)]
use collision_prepared_ground_collider::PreparedGroundCollider;
pub use validation::NativeWorldSceneError;
use validation::{
    validate_name, validate_plain_or_prefixed_blake3, validate_root_chain, validate_scene_instance,
    validate_catalog_entry, validate_runtime_world_reference, validate_runtime_world_registry_entry,
    validate_runtime_scene_link, validate_catalog_scene_link, validate_blake3,
    validate_relative_asset_path
};
pub use types_native_world_scene::{
    NativeWorldCoordinateContract, NativeWorldProvenance, AuthoredWorldTransform,
    NativeWorldVisual, NativeWorldScope, NativeWorldRootChainNode, NativeWorldRootChain,
    NativeWorldScene, NativeWorldSceneEntity, NativeWorldSceneRoot,
    PendingNativeWorldSceneSpawn, PendingNativeWorldSceneUnload
};
use types_native_world_scene::{
    NativeWorldPresentationFootprint, NativeWorldNeighborWaterRoot,
    NativeWorldNeighborWaterVisual
};
use types_native_world_plugin::{
    NativeWorldRequestedScope, NativeWorldPrimaryFarPresentationGuard,
    NativeWorldPreparedRangeMember, NativeWorldPendingRangeGroup,
    PendingAuthoredPerimeterFootprint, AuthoredWallMotionResult, AuthoredCapsuleResponse,
    AuthoredControllerMotionResult, GroundInterval, GroundPreparationBudget
};
pub use types_native_world_plugin::{
    SpawnedNativeWorldVisual, NativeWorldGroundSupport, SpawnedNativeWorldScene,
    NativeWorldPlugin, NativeWorldSet, NativeWaterOcclusion
};
pub(crate) use types_native_world_plugin::{NativeWorldVisualSceneReady, AuthoredSegmentHit};
use state::{RuntimeMapTile, NativeWorldTerrainPresentation, verify_runtime_world_reference};
pub use state::{
    NativeWorldPresentationStatus, NativeWorldVisualPresentationStatus,
    NativeWorldBehaviourStatus, RuntimeManagedNativeWorldVisual, NativeWorldStreamingStatus,
    NativeWorldLocationPresentation
};
pub use operations_materialize_loaded_native_world_visual_asset::{
    select_native_world_scene, begin_native_world_scene_admission
};
use operations_materialize_loaded_native_world_visual_asset::{
    matches_primary_audited_smallstuff_presentation_identity, quantize_native_world_range,
    native_world_bounds_fit_center_range, native_world_pipeline_sentinel_range,
    native_world_range_physical_visibility, is_legacy_non_presenting_visual,
    native_world_water_surface, is_native_world_water_visual,
    materialize_loaded_native_world_visual_assets, begin_streamed_native_world_scene_loading,
    materialize_pending_native_world_scene_spawns, transformed_aabb_bounds,
    make_unsafe_native_world_range_groups_unbounded, mark_native_world_visual_scene_ready,
    native_world_descendant, native_world_ancestor_in_roots
};
#[cfg(test)]
use operations_materialize_loaded_native_world_visual_asset::is_legacy_zero_contribution_visual;
#[cfg(test)]
use operations_materialize_loaded_native_world_visual_asset::native_world_presentation_ready;
use operations_reveal_native_world_scenes::{
    reveal_native_world_scenes, stream_neighboring_water_surfaces,
    reveal_neighboring_water_surfaces, stream_native_world_dongs, queue_native_world_scene_unload,
    unload_native_world_scenes_incrementally, orient_native_water_surface_indices_upward
};
#[cfg(test)]
use operations_reveal_native_world_scenes::{
    neighboring_water_scenes, native_world_has_physical_admission_capacity,
    native_world_stream_eviction_tile, unload_native_world_scene_batch
};
pub use operations_reveal_native_world_scenes::{
    legacy_dong_squared_distance_native, native_dong_key
};
use operations_cook_convex_xy_prism::{
    cook_convex_xy_prism, carried_support_point, authored_player_ground_search_range,
    authored_down_contact_is_non_walkable, constrain_displacement_to_walkable_surface,
    authored_wall_spatial_query_bounds, retain_source_contact_normal,
    insert_authored_contact_normal, authored_capsule_callback_normal,
    authored_front_side_contact_direction, recover_authored_capsule_penetration_with_response,
    authored_capsule_overlap_hit_with_response,
    deepest_authored_capsule_penetration_with_response, closest_points_between_segments,
    capsule_face_root_height, closest_point_on_segment_xz, aabb_overlaps, verify_file_blake3,
    join_relative
};
#[cfg(test)]
use operations_cook_convex_xy_prism::deepest_authored_capsule_penetration;
use materials::{NativeWorldMaterialPresentationFailed, validate_render_contract};
pub use entities::{
    spawn_pending_authored_model_collider, spawn_pending_authored_model_perimeter_collider,
    spawn_native_world_scene
};
use entities::{
    spawn_native_world_scene_root, spawn_native_world_visual, spawn_native_world_collider,
    spawn_native_world_terrain
};
use animation::clip_displacement_against_contacts;
