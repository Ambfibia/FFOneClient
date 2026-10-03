//! Verified native project assets used by the stable runtime foundation.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use bevy::prelude::Resource;

/// Stable roots and root contracts understood by the native client.
///
/// These are routing rules, not an inventory. Domain catalogs own dynamic
/// mappings while code-owned UI modules keep their typed static paths.
pub const WORLD_CATALOG_PATH: &str = "map/catalog.json";
pub const TABLE_SET_PATH: &str = "data/tables/xdt.json";
pub const NPC_TEXTURE_CATALOG_PATH: &str = "data/tables/npc_texture_overrides.json";
pub const LOCALIZATION_CATALOG_PATH: &str = "localization/catalog.json";

pub const RUNTIME_ASSET_ROOTS: &[&str] = &[
    "_runtime",
    "audio",
    "characters",
    "data",
    "effects",
    "fonts",
    "icons",
    "localization",
    "map",
    "shaders",
    "ui",
];

/// Resolves stable logical asset paths against one physical asset source.
///
/// `AssetLocator` deliberately does not enumerate the complete tree. Runtime
/// systems resolve static paths from code and dynamic paths from their owning
/// domain catalogs. Full closure and orphan validation belongs to the release
/// graph validator, not client startup.
#[derive(Clone, Debug, Resource)]
pub struct AssetLocator {
    root: PathBuf,
}

impl AssetLocator {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, String> {
        let root = root.into();
        let metadata = fs::metadata(&root)
            .map_err(|error| format!("failed to open asset root {}: {error}", root.display()))?;
        if !metadata.is_dir() {
            return Err(format!("asset root is not a directory: {}", root.display()));
        }
        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, relative: &str) -> Result<PathBuf, String> {
        validate_relative_path(relative)?;
        Ok(join_asset_path(&self.root, relative))
    }

    pub fn read(&self, relative: &str) -> Result<Vec<u8>, String> {
        let path = self.path(relative)?;
        fs::read(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))
    }

    pub fn read_verified(
        &self,
        relative: &str,
        _expected_bytes: Option<u64>,
        _expected_blake3: &str,
    ) -> Result<Vec<u8>, String> {
        self.read(relative)
    }

    pub fn read_json<T: serde::de::DeserializeOwned>(&self, relative: &str) -> Result<T, String> {
        let path = self.path(relative)?;
        let bytes = fs::read(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid JSON {}: {error}", path.display()))
    }

    /// Named in-memory view of the server-compatible XDT and native extensions.
    pub fn read_table_set(&self) -> Result<serde_json::Value, String> {
        crate::xdt::from_slice(&self.read(TABLE_SET_PATH)?)
    }

    pub fn read_character_models<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_value(crate::asset_tables::character_models(&self.root)?)
            .map_err(|error| error.to_string())
    }

    pub fn read_verified_json<T: serde::de::DeserializeOwned>(
        &self,
        relative: &str,
        expected_bytes: Option<u64>,
        expected_blake3: &str,
    ) -> Result<T, String> {
        let bytes = self.read_verified(relative, expected_bytes, expected_blake3)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid JSON {relative:?}: {error}"))
    }

    pub fn require_file(&self, relative: &str) -> Result<PathBuf, String> {
        let path = self.path(relative)?;
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("missing required asset {}: {error}", path.display()))?;
        if !metadata.is_file() {
            return Err(format!("required asset is not a file: {}", path.display()));
        }
        Ok(path)
    }

    #[must_use]
    pub fn diagnostics(&self) -> AssetLocatorDiagnostics {
        AssetLocatorDiagnostics {
            root: fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone()),
            routing: "table-data-v1",
            domain_catalogs: [
                TABLE_SET_PATH,
                LOCALIZATION_CATALOG_PATH,
                WORLD_CATALOG_PATH,
            ],
        }
    }

    #[must_use]
    pub fn character_package_root(category: &str, id: &str) -> Result<String, String> {
        let directory = match category {
            "nano" => "nanos",
            "mob" => "mobs",
            "npc" => "npcs",
            "fusion" => "fusions",
            "player" | "tutorial" => category,
            _ => return Err(format!("unknown character category {category:?}")),
        };
        validate_identifier(id)?;
        Ok(format!("characters/{directory}/{id}"))
    }

    pub fn world_tile_root(scope: &str, id: &str) -> Result<String, String> {
        validate_identifier(id)?;
        match scope {
            "tutorial" | "worldMap" => {
                let canonical = id
                    .strip_prefix("tile_")
                    .map(|coordinates| format!("map_{coordinates}"))
                    .unwrap_or_else(|| id.to_owned());
                validate_identifier(&canonical)?;
                Ok(format!("map/tiles/{canonical}"))
            }
            _ => Err(format!("unknown world scope {scope:?}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssetLocatorDiagnostics {
    root: PathBuf,
    routing: &'static str,
    domain_catalogs: [&'static str; 3],
}

impl AssetLocatorDiagnostics {
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "assetRoot": self.root.display().to_string(),
            "routing": self.routing,
            "domainCatalogs": self.domain_catalogs,
            "globalManifest": false,
        }))?;
        json.push('\n');
        Ok(json)
    }
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.contains('\\') {
        return Err(format!("invalid project-asset path {path:?}"));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe project-asset path {path:?}"));
    }
    Ok(())
}

fn validate_identifier(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.starts_with('_')
        || id.ends_with('_')
        || id
            .bytes()
            .any(|byte| !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && byte != b'_')
    {
        return Err(format!("invalid semantic asset identifier {id:?}"));
    }
    Ok(())
}

fn join_asset_path(root: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

#[cfg(test)]
mod tests;
