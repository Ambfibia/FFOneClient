use super::*;

pub const PACKAGE_MANIFEST_FILE: &str = "package.ffpkg.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePackageManifest {
    pub schema: String,
    pub id: String,
    pub version: String,
    pub protocol: String,
    #[serde(default)]
    pub requires: Vec<PackageRequirement>,
}

/// A validated path resolved relative to the definition leaf that owns it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SafeRelativePath(pub(super) String);

impl SafeRelativePath {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl fmt::Display for SafeRelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Fully validated active package registry.
#[derive(Debug)]
pub struct NativePackageRegistry {
    pub(super) packages: Vec<LoadedPackage>,
    pub(super) npcs: Vec<LoadedDefinition<NpcDefinition>>,
    pub(super) nanos: Vec<LoadedDefinition<NanoDefinition>>,
    pub(super) items: Vec<LoadedDefinition<ItemDefinition>>,
    pub(super) npc_by_id: BTreeMap<String, usize>,
    pub(super) nano_by_id: BTreeMap<String, usize>,
    pub(super) item_by_id: BTreeMap<String, usize>,
    pub(super) replacements: BTreeMap<String, String>,
}

impl NativePackageRegistry {
    pub fn packages(&self) -> &[LoadedPackage] {
        &self.packages
    }

    pub fn npcs(&self) -> &[LoadedDefinition<NpcDefinition>] {
        &self.npcs
    }

    pub fn nanos(&self) -> &[LoadedDefinition<NanoDefinition>] {
        &self.nanos
    }

    pub fn items(&self) -> &[LoadedDefinition<ItemDefinition>] {
        &self.items
    }

    /// Resolves an active canonical ID or an older canonical ID replaced by an active leaf.
    pub fn active_canonical_id<'a>(&'a self, canonical_id: &'a str) -> Option<&'a str> {
        let mut current = canonical_id;
        while let Some(next) = self.replacements.get(current) {
            current = next;
        }
        if self.npc_by_id.contains_key(current)
            || self.nano_by_id.contains_key(current)
            || self.item_by_id.contains_key(current)
        {
            Some(current)
        } else {
            None
        }
    }

    pub fn npc(&self, canonical_id: &str) -> Option<&LoadedDefinition<NpcDefinition>> {
        let active = self.active_canonical_id(canonical_id)?;
        self.npc_by_id.get(active).map(|index| &self.npcs[*index])
    }

    pub fn nano(&self, canonical_id: &str) -> Option<&LoadedDefinition<NanoDefinition>> {
        let active = self.active_canonical_id(canonical_id)?;
        self.nano_by_id.get(active).map(|index| &self.nanos[*index])
    }

    pub fn item(&self, canonical_id: &str) -> Option<&LoadedDefinition<ItemDefinition>> {
        let active = self.active_canonical_id(canonical_id)?;
        self.item_by_id.get(active).map(|index| &self.items[*index])
    }

    /// Immediate old-to-new canonical mappings. Replacement documents replace whole leaves.
    pub fn replacements(&self) -> &BTreeMap<String, String> {
        &self.replacements
    }
}

pub(super) fn path_file_name(path: &Path) -> Result<&str, PackageRegistryError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| PackageRegistryError::NonUtf8Path {
            path: path.to_path_buf(),
        })
}

pub(super) fn build_registry(
    packages: Vec<LoadedPackage>,
    pending: Vec<PendingDefinition>,
    dependency_closures: &BTreeMap<String, BTreeSet<String>>,
) -> Result<NativePackageRegistry, PackageRegistryError> {
    let mut all = BTreeMap::<String, DefinitionIdentity>::new();
    let mut all_sources = BTreeMap::<String, PathBuf>::new();

    for definition in &pending {
        if let Some(first) = all_sources.get(&definition.canonical_id) {
            return Err(PackageRegistryError::DuplicateCanonicalId {
                id: definition.canonical_id.clone(),
                first: first.clone(),
                second: definition.source_path.clone(),
            });
        }
        all_sources.insert(
            definition.canonical_id.clone(),
            definition.source_path.clone(),
        );
        all.insert(
            definition.canonical_id.clone(),
            DefinitionIdentity {
                package_id: definition.package_id.clone(),
                network_key: definition.network_key(),
            },
        );
    }

    let mut active_by_id = BTreeMap::<String, usize>::new();
    let mut active_by_network = BTreeMap::<NetworkKey, String>::new();
    let mut active = Vec::<Option<ActiveDefinition>>::new();
    let mut replacements = BTreeMap::<String, String>::new();

    for definition in pending {
        let network_key = definition.network_key();
        if let Some(target_id) = definition.replaces().map(str::to_owned) {
            let target = all.get(&target_id).ok_or_else(|| {
                PackageRegistryError::MissingReplacementTarget {
                    replacement: definition.canonical_id.clone(),
                    target: target_id.clone(),
                }
            })?;
            let dependencies = &dependency_closures[&definition.package_id];
            if !dependencies.contains(&target.package_id) {
                return Err(PackageRegistryError::ReplacementNotDependency {
                    replacement: definition.canonical_id.clone(),
                    target_package: target.package_id.clone(),
                });
            }
            if std::mem::discriminant(&network_key) != std::mem::discriminant(&target.network_key) {
                return Err(PackageRegistryError::ReplacementKindMismatch {
                    replacement: definition.canonical_id.clone(),
                    target: target_id,
                });
            }
            if network_key != target.network_key {
                return Err(PackageRegistryError::ReplacementNetworkMismatch {
                    replacement: definition.canonical_id.clone(),
                    target: target_id,
                    expected: target.network_key.to_string(),
                    found: network_key.to_string(),
                });
            }
            let target_slot = active_by_id.remove(&target_id).ok_or_else(|| {
                PackageRegistryError::ReplacementTargetInactive {
                    replacement: definition.canonical_id.clone(),
                    target: target_id.clone(),
                }
            })?;
            active[target_slot] = None;
            active_by_network.remove(&network_key);
            replacements.insert(target_id, definition.canonical_id.clone());
        } else if let Some(first) = active_by_network.get(&network_key) {
            return Err(PackageRegistryError::DuplicateNetworkId {
                kind: network_key.kind(),
                key: network_key.to_string(),
                first: first.clone(),
                second: definition.canonical_id,
            });
        }

        let canonical_id = definition.canonical_id.clone();
        let loaded = match definition.value {
            PendingValue::Npc(value) => ActiveDefinition::Npc(LoadedDefinition {
                package_id: definition.package_id.clone(),
                canonical_id: canonical_id.clone(),
                source_path: definition.source_path.clone(),
                leaf_root: definition.leaf_root,
                definition: value,
            }),
            PendingValue::Nano(value) => ActiveDefinition::Nano(LoadedDefinition {
                package_id: definition.package_id.clone(),
                canonical_id: canonical_id.clone(),
                source_path: definition.source_path.clone(),
                leaf_root: definition.leaf_root,
                definition: value,
            }),
            PendingValue::Item(value) => ActiveDefinition::Item(LoadedDefinition {
                package_id: definition.package_id.clone(),
                canonical_id: canonical_id.clone(),
                source_path: definition.source_path.clone(),
                leaf_root: definition.leaf_root,
                definition: value,
            }),
        };
        let slot = active.len();
        active.push(Some(loaded));
        active_by_id.insert(canonical_id.clone(), slot);
        active_by_network.insert(network_key.clone(), canonical_id.clone());
    }

    let mut npcs = Vec::new();
    let mut nanos = Vec::new();
    let mut items = Vec::new();
    for definition in active.into_iter().flatten() {
        match definition {
            ActiveDefinition::Npc(value) => npcs.push(value),
            ActiveDefinition::Nano(value) => nanos.push(value),
            ActiveDefinition::Item(value) => items.push(value),
        }
    }
    let npc_by_id = definition_index(&npcs);
    let nano_by_id = definition_index(&nanos);
    let item_by_id = definition_index(&items);
    Ok(NativePackageRegistry {
        packages,
        npcs,
        nanos,
        items,
        npc_by_id,
        nano_by_id,
        item_by_id,
        replacements,
    })
}

pub(super) fn definition_index<T>(definitions: &[LoadedDefinition<T>]) -> BTreeMap<String, usize> {
    definitions
        .iter()
        .enumerate()
        .map(|(index, definition)| (definition.canonical_id.clone(), index))
        .collect()
}
