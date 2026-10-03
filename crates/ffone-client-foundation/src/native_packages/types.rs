use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRequirement {
    pub id: String,
    /// Exact version; ranges are intentionally outside the phase-one contract.
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NpcDefinition {
    pub schema: String,
    pub id: String,
    pub network_id: u32,
    pub replaces: Option<String>,
    pub assets: BTreeMap<String, SafeRelativePath>,
    /// Game-specific properties. Replacement never merges this map with an older leaf.
    pub data: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NanoDefinition {
    pub schema: String,
    pub id: String,
    pub network_id: u32,
    pub replaces: Option<String>,
    pub assets: BTreeMap<String, SafeRelativePath>,
    pub data: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemNetworkId {
    #[serde(rename = "type")]
    pub item_type: u32,
    pub number: u32,
}

impl fmt::Display for ItemNetworkId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.item_type, self.number)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemDefinition {
    pub schema: String,
    /// First wildcard in `items/*/*/item.ffdef.json`.
    pub category: String,
    pub id: String,
    pub network_id: ItemNetworkId,
    pub replaces: Option<String>,
    pub assets: BTreeMap<String, SafeRelativePath>,
    pub data: BTreeMap<String, JsonValue>,
}

/// A validated definition together with stable runtime and filesystem identities.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedDefinition<T> {
    pub(super) package_id: String,
    pub(super) canonical_id: String,
    pub(super) source_path: PathBuf,
    pub(super) leaf_root: PathBuf,
    pub(super) definition: T,
}

impl<T> LoadedDefinition<T> {
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn canonical_id(&self) -> &str {
        &self.canonical_id
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn definition(&self) -> &T {
        &self.definition
    }

    /// Resolves a validated asset path relative to the definition leaf directory.
    pub fn resolve_asset_path(&self, path: &SafeRelativePath) -> PathBuf {
        self.leaf_root.join(path.as_path())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPackage {
    pub(super) root: PathBuf,
    pub(super) manifest: NativePackageManifest,
}

impl LoadedPackage {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &NativePackageManifest {
        &self.manifest
    }
}

#[derive(Debug)]
pub(super) struct PackageCandidate {
    pub(super) root: PathBuf,
    pub(super) manifest_path: PathBuf,
    pub(super) manifest: NativePackageManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum NetworkKey {
    Npc(u32),
    Nano(u32),
    Item(ItemNetworkId),
}

impl NetworkKey {
    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::Npc(_) => "NPC",
            Self::Nano(_) => "Nano",
            Self::Item(_) => "item",
        }
    }
}

impl fmt::Display for NetworkKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Npc(id) | Self::Nano(id) => write!(formatter, "{id}"),
            Self::Item(id) => id.fmt(formatter),
        }
    }
}

#[derive(Debug)]
pub(super) enum PendingValue {
    Npc(NpcDefinition),
    Nano(NanoDefinition),
    Item(ItemDefinition),
}

#[derive(Debug)]
pub(super) struct PendingDefinition {
    pub(super) package_id: String,
    pub(super) canonical_id: String,
    pub(super) source_path: PathBuf,
    pub(super) leaf_root: PathBuf,
    pub(super) value: PendingValue,
}

impl PendingDefinition {
    pub(super) fn network_key(&self) -> NetworkKey {
        match &self.value {
            PendingValue::Npc(value) => NetworkKey::Npc(value.network_id),
            PendingValue::Nano(value) => NetworkKey::Nano(value.network_id),
            PendingValue::Item(value) => NetworkKey::Item(value.network_id),
        }
    }

    pub(super) fn replaces(&self) -> Option<&str> {
        match &self.value {
            PendingValue::Npc(value) => value.replaces.as_deref(),
            PendingValue::Nano(value) => value.replaces.as_deref(),
            PendingValue::Item(value) => value.replaces.as_deref(),
        }
    }
}

#[derive(Debug)]
pub(super) struct DefinitionIdentity {
    pub(super) package_id: String,
    pub(super) network_key: NetworkKey,
}

#[derive(Debug)]
pub(super) enum ActiveDefinition {
    Npc(LoadedDefinition<NpcDefinition>),
    Nano(LoadedDefinition<NanoDefinition>),
    Item(LoadedDefinition<ItemDefinition>),
}
