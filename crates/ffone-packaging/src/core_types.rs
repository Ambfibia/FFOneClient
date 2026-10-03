use super::*;

#[derive(Clone, Debug)]
pub struct PackageOptions {
    pub source_root: PathBuf,
    pub release_root: PathBuf,
}

#[derive(Debug)]
pub struct ReleaseSourceSnapshot {
    pub(super) canonical_source_root: PathBuf,
    pub(super) graph: AssetGraph,
}

#[derive(Clone, Debug)]
pub struct VerifyOptions {
    pub asset_root: PathBuf,
    pub full: bool,
}

#[derive(Clone, Debug)]
pub struct RestoreOptions {
    pub asset_root: PathBuf,
    pub donor_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ArchiveRecoveryOptions {
    pub asset_root: PathBuf,
    pub archive_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseSourceReport {
    pub groups: usize,
    pub files: usize,
    pub payload_bytes: u64,
    pub root_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageReport {
    pub files: usize,
    pub payload_bytes: u64,
    pub copied: usize,
    pub reused: usize,
    pub removed: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyReport {
    pub files: usize,
    pub payload_bytes: u64,
    pub full: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreReport {
    pub manifest_files: usize,
    pub restored_files: usize,
    pub restored_bytes: u64,
    pub already_valid_files: usize,
    pub already_valid_bytes: u64,
    pub missing_files: usize,
    pub missing_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveRecoveryReport {
    pub archived_files: usize,
    pub archived_bytes: u64,
    pub hard_linked_files: usize,
    pub copied_files: usize,
    pub remaining_manifest_files: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PackageCache {
    pub(super) schema: String,
    pub(super) source_root: String,
    pub(super) files: BTreeMap<String, CachedFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CachedFile {
    pub(super) source_modified_nanos: u128,
    pub(super) destination_modified_nanos: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReleaseReceipt {
    pub(super) schema: &'static str,
    pub(super) tool_version: &'static str,
    pub(super) packaging_mode: &'static str,
    pub(super) asset_root: &'static str,
    pub(super) file_count: usize,
    pub(super) groups: Vec<ReleaseGroupSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReleaseGroupSummary {
    pub(super) id: String,
    pub(super) file_count: usize,
}
