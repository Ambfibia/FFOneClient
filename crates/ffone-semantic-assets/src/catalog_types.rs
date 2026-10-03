use super::*;

pub type Result<T> = std::result::Result<T, SemanticAssetError>;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCategory {
    Npc,
    Mob,
    Player,
    Nano,
    World,
    Ui,
    Audio,
    Data,
    Unresolved,
}

impl SemanticCategory {
    pub(super) fn required_prefix(self) -> Option<&'static str> {
        match self {
            Self::Npc => Some("characters/npc/"),
            Self::Mob => Some("characters/mob/"),
            Self::Player => Some("characters/player/"),
            Self::Nano => Some("characters/nano/"),
            Self::World => Some("world/"),
            Self::Ui => Some("ui/"),
            Self::Audio => Some("audio/"),
            Self::Data => Some("data/"),
            Self::Unresolved => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnershipProof {
    pub true_legacy_name: String,
    pub authority: String,
    pub source_build: String,
    pub source_archive: String,
    pub source_asset: String,
    #[serde(default)]
    pub source_object_ids: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BlockerSummary {
    pub code: String,
    pub count: u64,
    pub samples: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicationReport {
    pub schema: String,
    pub status: String,
    pub source_manifest_blake3: String,
    pub final_manifest_blake3: String,
    pub source_store_retained: bool,
    pub committed_files: Vec<String>,
    pub reused_identical_files: Vec<String>,
    pub glb_proofs: Vec<GlbProof>,
}

pub(super) struct TransactionLock {
    pub(super) path: PathBuf,
}

impl TransactionLock {
    pub(super) fn acquire(path: &Path) -> Result<Self> {
        match OpenOptions::new().create_new(true).write(true).open(path) {
            Ok(mut file) => {
                writeln!(file, "pid={}", std::process::id())
                    .map_err(|source| io_at(path, source))?;
                Ok(Self {
                    path: path.to_owned(),
                })
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(SemanticAssetError::Locked(path.to_owned()))
            }
            Err(source) => Err(io_at(path, source)),
        }
    }
}

impl Drop for TransactionLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
