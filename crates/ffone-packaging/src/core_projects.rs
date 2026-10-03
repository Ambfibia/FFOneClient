use super::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct ProjectFile {
    pub(super) source_path: String,
    pub(super) path: String,
    pub(super) kind: String,
    pub(super) bytes: u64,
    pub(super) blake3: String,
}
