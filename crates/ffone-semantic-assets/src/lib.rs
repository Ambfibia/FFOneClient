//! Transactional publication of human-readable, ownership-proven native assets.
//!
//! This crate only reads files that are already present in the Bevy project asset tree. It never
//! reads Unity bundles and never deletes the flat, content-addressed import store. A semantic route
//! must carry ownership evidence, must use the true legacy name, and must pass byte/hash and GLB URI
//! checks before any destination or manifest is changed.

#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

#[cfg(test)]
mod tests;

mod catalog_assets;
mod catalog_validation;
mod catalog_constants;
mod catalog_terrain_publish_native_terrain_batch;
mod catalog_terrain_collect_native_terrain_batch_files;
mod catalog_terrain_validate_native_terrain_reconciliation_plan;
mod catalog_types;
mod catalog_operations;
mod catalog_models;
mod catalog_codec;
mod catalog_state;
mod catalog_input;
mod catalog_output;
mod catalog_materials;

pub use catalog_assets::{
    ROUTE_PLAN_SCHEMA, WORLD_CATALOG_SCHEMA, RUNTIME_WORLD_REGISTRY_SCHEMA, AssetManifest,
    AssetManifestEntry, AssetKind, RoutePlan, SemanticRoute, RouteContent, publish_route_plan
};
use catalog_assets::{
    MANIFEST_SCHEMA, RenderedRoute, verify_manifest_entry, read_manifest,
    replace_manifest_transactionally, native_path, slash_path, classify_path, is_semantic_path
};
pub use catalog_validation::{
    TREE_AUDIT_SCHEMA, SemanticAssetError, AuditCounts, AssetTreeAudit, audit_asset_tree,
    validate_route_plan
};
use catalog_validation::{
    validate_batch_destination_root, validate_ownership, validate_relative_path
};
pub use catalog_constants::NATIVE_WORLD_SCENE_SCHEMA;
pub use catalog_terrain_publish_native_terrain_batch::{
    NATIVE_TERRAIN_PUBLICATION_PLAN_SCHEMA, NativeTerrainReconciliationReport,
    NativeTerrainPublicationPlan, NativeTerrainPublicationEntry,
    NativeTerrainPublicationBlocker, NativeTerrainBatchPublicationReport,
    reconcile_retro_world_native_terrain, publish_native_terrain_batch
};
use catalog_terrain_publish_native_terrain_batch::{
    NativeTerrainBatchFile, NativeTerrainBatchFileSource
};
use catalog_terrain_collect_native_terrain_batch_files::collect_native_terrain_batch_files;
#[cfg(test)]
use catalog_terrain_collect_native_terrain_batch_files::native_terrain_display_name;
use catalog_terrain_validate_native_terrain_reconciliation_plan::{
    insert_native_terrain_batch_file, validate_native_terrain_reconciliation_plan
};
pub use catalog_types::{
    Result, SemanticCategory, OwnershipProof, BlockerSummary, PublicationReport
};
use catalog_types::TransactionLock;
use catalog_operations::{
    io_at, normalize_blake3, restore_quarantined, rollback_replaced,
    remove_empty_legacy_world_directories, json_array_len, hash_file, rewrite_json_strings,
    rollback_created, unique_stamp, has_content_hash_filename, remove_content_hash,
    contains_variant_directory, required_str, required_i64
};
pub use catalog_operations::{ensure_legacy_world_publication_allowed, build_retro_world_plan};
pub use catalog_models::GlbProof;
use catalog_models::{glb_uris_are_safe, inspect_glb};
pub use catalog_codec::NativeTerrainPublicationPayload;
use catalog_state::RETRO_WORLD_LEGACY_RUNTIME_PATHS;
use catalog_input::{collect_files_recursive, collect_uris, read_json, read_bytes};
pub use catalog_output::write_json;
use catalog_output::write_bytes;
use catalog_materials::render_route;
