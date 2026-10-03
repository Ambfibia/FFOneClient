//! Instance-local native player rig assembly.
//!
//! The immutable contract and every GLB are published by
//! `ffone-asset-pipeline::publish_player_shared_rigs`. This runtime never opens
//! Unity data. It spawns one shared actor skeleton, rebinds each modular part
//! through the proven `transformIndicesM/F` palette, and loops the real
//! `stand1` clip embedded in the shared-skeleton GLB.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use bevy::{
    asset::{Asset, LoadState, RecursiveDependencyLoadState},
    camera::visibility::RenderLayers,
    gltf::{Gltf, GltfAssetLabel},
    mesh::skinning::SkinnedMesh,
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use ffone_runtime_contracts::{
    AvatarItemCategory, CHARACTER_CREATION_AVATAR_ITEMS_PATH,
    CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA, CharacterAppearanceCategory,
    CharacterCreationAssetReference, CharacterCreationAvatarItems, NativeLookupStatus,
    PLAYER_SHARED_RIG_CONTRACT_PATH, PLAYER_SHARED_RIG_SCHEMA, PlayerGenderRigContract,
    PlayerRigGender, PlayerRigPartContract, PlayerRigSkinRemap, PlayerSharedRigContract,
};
use serde::Deserialize;

use crate::{
    assets::AssetLocator,
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyMaterialPassCompanion, LegacyMaterialRendererOrder,
        LegacyMaterialSortOrderApplied, PendingLegacyModelMaterial,
    },
};

mod body_shape;
pub use body_shape::{NativePlayerBodyShape, NativePlayerBodyShapePlayback};

#[cfg(test)]
#[path = "performance_tests.rs"]
mod performance_tests;

use crate::scene_hierarchy::is_descendant_of;

#[cfg(test)]
mod tests;

mod animation_native_player_rig_catalog;
mod animation_bind_native_player_rig_parts;
mod animation_append_native_animation_catalog;
mod assets_asset_load_failure;
mod models;
mod constants;
mod operations;
mod validation;
mod materials;
mod entities;

pub use animation_native_player_rig_catalog::{
    NativePlayerRigCatalog, NativePlayerRigInstance, NativePlayerRigLoadingStage,
    NativePlayerRigStatus, NativePlayerRigBoneEntity, NativePlayerRigBones,
    NativePlayerRigSkeletonScene, NativePlayerRigPartScene, NativePlayerRigStand1Playback,
    NativePlayerRigAnimationRequest, NativePlayerRigAnimationApplied,
    NativePlayerRigAnimationIssue, NativePlayerRigSpawnRequest, SpawnedNativePlayerRig,
    NativePlayerRigPartsBound, NativePlayerRigSkinBound, NativePlayerRigAssetCache,
    NativePlayerSharedRigPlugin
};
use animation_native_player_rig_catalog::{
    player_rig_gender_key, NativePlayerRigRuntime, NativePlayerRigPartRuntime,
    NativePlayerRigSceneReady, NativePlayerRigStand1Graph, mark_native_player_rig_scene_ready
};
pub(crate) use animation_native_player_rig_catalog::NativePlayerRigAnimationApply;
use animation_bind_native_player_rig_parts::{
    resolve_native_player_rig_bones, prepare_native_player_rig_stand1, bind_native_player_rig_parts,
    play_native_player_rig_stand1, apply_native_player_rig_animation_requests,
    finalize_native_player_rig_status, RigDescendantScratch, RigQueryIndex
};
use animation_append_native_animation_catalog::{
    append_vehicle_animation_catalog, append_native_animation_catalog
};
use assets_asset_load_failure::{
    PLAYER_ITEM_SET_CATALOG_PATH, PLAYER_ITEM_SET_CATALOG_SCHEMA, SKINNED_BACK_CLOTHES_INDEX,
    actor_skin_combiner_clothes_index, asset_uri, asset_load_failure, flattened_renderer_index
};
use models::{PlayerItemModelCatalog, gltf_node_path};
use constants::UNEQUIPPED_PARTS;
use operations::{
    append_native_unequipped_parts, append_native_table_skinned_wearables,
    append_native_hat_variants, append_native_skinned_backs, synthesize_native_skin_remaps,
    exact_renderer_remap, owning_scene_root
};
use validation::validate_contract;
pub use materials::NativePlayerRigAnimationBlend;
use materials::{
    propagate_native_player_rig_render_layers, rebind_native_player_rig_material_companions,
    fail_closed_native_player_rig_material_metadata
};
#[cfg(test)]
use materials::NativePlayerRigMaterialMetadataGrace;
pub use entities::spawn_native_player_rig;
