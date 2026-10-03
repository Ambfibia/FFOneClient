//! Runtime discovery and validation for editable native content packages.
//!
//! This module deliberately knows nothing about Unity containers, extraction caches, hashes, or
//! cooked artifacts. A package is an ordinary direct child directory with a
//! [`PACKAGE_MANIFEST_FILE`] manifest and editable JSON definition leaves.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value as JsonValue;

#[cfg(test)]
mod tests;

mod assets;
mod constants;
mod types;
mod validation;
mod codec;
mod input;
mod operations;

pub use assets::{
    PACKAGE_MANIFEST_FILE, NativePackageManifest, SafeRelativePath, NativePackageRegistry
};
use assets::{path_file_name, build_registry};
pub use constants::{
    PACKAGE_SCHEMA, NPC_DEFINITION_SCHEMA, NANO_DEFINITION_SCHEMA, ITEM_DEFINITION_SCHEMA
};
pub use types::{
    PackageRequirement, NpcDefinition, NanoDefinition, ItemNetworkId, ItemDefinition,
    LoadedDefinition, LoadedPackage
};
use types::{
    PackageCandidate, NetworkKey, PendingValue, PendingDefinition, DefinitionIdentity,
    ActiveDefinition
};
pub use validation::PackageRegistryError;
use validation::{validate_schema, validate_identifier, validate_text, validate_assets};
use codec::{WireNpcDefinition, WireNanoDefinition, WireItemDefinition};
pub use input::discover_native_packages;
use operations::{
    dependency_order, dependency_closures, sorted_directories, optional_sorted_directories,
    checked_definition_file
};
