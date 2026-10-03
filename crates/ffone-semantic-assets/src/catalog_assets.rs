use super::*;

pub const ROUTE_PLAN_SCHEMA: &str = "ffone.semantic-tree-route-plan.v1";

pub const WORLD_CATALOG_SCHEMA: &str = "ffone.semantic-world-catalog.v2";

pub const RUNTIME_WORLD_REGISTRY_SCHEMA: &str = "ffone.runtime-world.v1";

pub(super) const MANIFEST_SCHEMA: &str = "ffone.project-assets.v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetManifest {
    pub schema: String,
    pub protocol: u16,
    pub locale: String,
    pub source_pack: Value,
    pub files: Vec<AssetManifestEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetManifestEntry {
    pub source_path: String,
    pub path: String,
    pub kind: AssetKind,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Model,
    Texture,
    Audio,
    Font,
    Data,
    Shader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoutePlan {
    pub schema: String,
    pub source_build: String,
    pub policy: String,
    pub routes: Vec<SemanticRoute>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticRoute {
    pub source_path: String,
    pub destination_path: String,
    pub kind: AssetKind,
    pub category: SemanticCategory,
    pub ownership: OwnershipProof,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duplicate_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_reason: Option<String>,
    pub content: RouteContent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum RouteContent {
    CopyExact,
    RewriteJson {
        replacements: BTreeMap<String, String>,
    },
    GeneratedJson {
        value: Value,
    },
}

#[derive(Clone, Debug)]
pub(super) struct RenderedRoute {
    pub(super) route: SemanticRoute,
    pub(super) staged_path: PathBuf,
    pub(super) bytes: u64,
    pub(super) blake3: String,
    pub(super) glb_proof: Option<GlbProof>,
}

/// Publish a validated plan with staging, a lock, manifest snapshot checking, and rollback.
///
/// Sources are never moved or deleted. Existing destinations are accepted only if their bytes are
/// identical. New paths are removed again if any commit step fails.
pub fn publish_route_plan(
    asset_root: &Path,
    manifest_path: &Path,
    plan: &RoutePlan,
) -> Result<PublicationReport> {
    fs::create_dir_all(asset_root).map_err(|source| io_at(asset_root, source))?;
    let lock_path = asset_root.join(".semantic-tree.lock");
    let lock = TransactionLock::acquire(&lock_path)?;
    let manifest_before = read_bytes(manifest_path)?;
    let manifest_before_hash = blake3::hash(&manifest_before).to_hex().to_string();
    let mut manifest: AssetManifest =
        serde_json::from_slice(&manifest_before).map_err(|source| SemanticAssetError::Json {
            path: manifest_path.to_owned(),
            source,
        })?;

    let glb_proofs = validate_route_plan(asset_root, manifest_path, plan)?;
    let stage_root = asset_root.join(format!(
        ".semantic-tree-stage-{}-{}",
        std::process::id(),
        unique_stamp()
    ));
    fs::create_dir(&stage_root).map_err(|source| io_at(&stage_root, source))?;

    let result = (|| {
        let manifest_by_path: BTreeMap<String, AssetManifestEntry> = manifest
            .files
            .iter()
            .cloned()
            .map(|entry| (entry.path.clone(), entry))
            .collect();
        let proof_by_path: BTreeMap<&str, GlbProof> = glb_proofs
            .iter()
            .map(|proof| (proof.path.as_str(), proof.clone()))
            .collect();
        let mut rendered = Vec::new();

        for route in &plan.routes {
            let staged_path = stage_root.join(native_path(&route.destination_path));
            if let Some(parent) = staged_path.parent() {
                fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
            }
            render_route(asset_root, route, &staged_path)?;
            let (bytes, blake3) = hash_file(&staged_path)?;
            rendered.push(RenderedRoute {
                route: route.clone(),
                staged_path,
                bytes,
                blake3,
                glb_proof: proof_by_path.get(route.destination_path.as_str()).cloned(),
            });
        }

        if blake3::hash(&read_bytes(manifest_path)?).to_hex().as_str() != manifest_before_hash {
            return Err(SemanticAssetError::ManifestChanged);
        }

        let mut committed = Vec::new();
        let mut reused = Vec::new();
        let mut created_files = Vec::new();
        let install_result = (|| {
            for rendered_route in &rendered {
                let destination =
                    asset_root.join(native_path(&rendered_route.route.destination_path));
                if destination.exists() {
                    let (bytes, hash) = hash_file(&destination)?;
                    if bytes != rendered_route.bytes || hash != rendered_route.blake3 {
                        return Err(SemanticAssetError::Transaction(format!(
                            "destination exists with different bytes: {}",
                            rendered_route.route.destination_path
                        )));
                    }
                    reused.push(rendered_route.route.destination_path.clone());
                } else {
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                    }
                    fs::rename(&rendered_route.staged_path, &destination)
                        .map_err(|source| io_at(&destination, source))?;
                    created_files.push(destination);
                    committed.push(rendered_route.route.destination_path.clone());
                }

                let source_path = match rendered_route.route.content {
                    RouteContent::GeneratedJson { .. } => {
                        format!(
                            "semantic-generated/{}",
                            rendered_route.route.destination_path
                        )
                    }
                    _ => manifest_by_path
                        .get(&rendered_route.route.source_path)
                        .map(|entry| entry.source_path.clone())
                        .unwrap_or_else(|| rendered_route.route.source_path.clone()),
                };
                let entry = AssetManifestEntry {
                    source_path,
                    path: rendered_route.route.destination_path.clone(),
                    kind: rendered_route.route.kind,
                    bytes: rendered_route.bytes,
                    blake3: rendered_route.blake3.clone(),
                };
                match manifest
                    .files
                    .iter()
                    .position(|candidate| candidate.path == entry.path)
                {
                    Some(index) if manifest.files[index] != entry => {
                        return Err(SemanticAssetError::Transaction(format!(
                            "manifest already contains a different destination entry: {}",
                            entry.path
                        )));
                    }
                    Some(_) => {}
                    None => manifest.files.push(entry),
                }
            }

            for route in &rendered {
                let destination = asset_root.join(native_path(&route.route.destination_path));
                let (bytes, hash) = hash_file(&destination)?;
                if bytes != route.bytes || hash != route.blake3 {
                    return Err(SemanticAssetError::Transaction(format!(
                        "pre-manifest commit hash verification failed: {}",
                        route.route.destination_path
                    )));
                }
            }
            Ok(())
        })();
        if let Err(error) = install_result {
            rollback_created(&created_files);
            return Err(error);
        }

        manifest.files.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.source_path.cmp(&right.source_path))
        });
        let mut next_manifest = match serde_json::to_vec_pretty(&manifest) {
            Ok(bytes) => bytes,
            Err(source) => {
                rollback_created(&created_files);
                return Err(SemanticAssetError::Json {
                    path: manifest_path.to_owned(),
                    source,
                });
            }
        };
        next_manifest.push(b'\n');
        let next_path = stage_root.join("asset-manifest.next.json");
        if let Err(error) = write_bytes(&next_path, &next_manifest) {
            rollback_created(&created_files);
            return Err(error);
        }

        if let Err(error) = replace_manifest_transactionally(manifest_path, &next_path) {
            rollback_created(&created_files);
            return Err(error);
        }
        let final_manifest_blake3 = blake3::hash(&next_manifest).to_hex().to_string();

        Ok(PublicationReport {
            schema: "ffone.semantic-tree-publication.v1".to_owned(),
            status: "committed-hash-verified-source-retained".to_owned(),
            source_manifest_blake3: manifest_before_hash,
            final_manifest_blake3,
            source_store_retained: true,
            committed_files: committed,
            reused_identical_files: reused,
            glb_proofs: rendered
                .into_iter()
                .filter_map(|route| route.glb_proof)
                .collect(),
        })
    })();

    let _ = fs::remove_dir_all(&stage_root);
    drop(lock);
    result
}

pub(super) fn verify_manifest_entry(path: &Path, entry: &AssetManifestEntry) -> Result<()> {
    let (bytes, hash) = hash_file(path)?;
    if bytes != entry.bytes || hash != entry.blake3 {
        return Err(SemanticAssetError::Verification(format!(
            "{} differs from manifest: disk bytes={bytes}, blake3={hash}; manifest bytes={}, blake3={}",
            entry.path, entry.bytes, entry.blake3
        )));
    }
    Ok(())
}

pub(super) fn read_manifest(path: &Path) -> Result<AssetManifest> {
    let bytes = read_bytes(path)?;
    let manifest: AssetManifest =
        serde_json::from_slice(&bytes).map_err(|source| SemanticAssetError::Json {
            path: path.to_owned(),
            source,
        })?;
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(SemanticAssetError::Verification(format!(
            "expected {MANIFEST_SCHEMA}, got {}",
            manifest.schema
        )));
    }
    Ok(manifest)
}

pub(super) fn replace_manifest_transactionally(manifest_path: &Path, next_path: &Path) -> Result<()> {
    let backup_path = manifest_path.with_extension(format!(
        "json.semantic-tree-backup-{}-{}",
        std::process::id(),
        unique_stamp()
    ));
    fs::rename(manifest_path, &backup_path).map_err(|source| io_at(manifest_path, source))?;
    match fs::rename(next_path, manifest_path) {
        Ok(()) => {
            // A retained backup is harmless and preferable to reporting a failed transaction after
            // the new manifest has already become authoritative.
            let _ = fs::remove_file(&backup_path);
            Ok(())
        }
        Err(source) => {
            let _ = fs::rename(&backup_path, manifest_path);
            Err(io_at(manifest_path, source))
        }
    }
}

pub(super) fn native_path(path: &str) -> PathBuf {
    path.split('/').collect()
}

pub(super) fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            Component::Prefix(prefix) => Some(prefix.as_os_str().to_string_lossy().into_owned()),
            Component::RootDir => Some(String::new()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn classify_path(path: &str) -> SemanticCategory {
    if path.starts_with("characters/npc/") {
        SemanticCategory::Npc
    } else if path.starts_with("characters/mob/") {
        SemanticCategory::Mob
    } else if path.starts_with("characters/player/") {
        SemanticCategory::Player
    } else if path.starts_with("characters/nano/") {
        SemanticCategory::Nano
    } else if path.starts_with("world/")
        || path.starts_with("worlds/")
        || path.starts_with("models/world/")
    {
        SemanticCategory::World
    } else if path.starts_with("ui/") {
        SemanticCategory::Ui
    } else if path.starts_with("audio/") {
        SemanticCategory::Audio
    } else if path.starts_with("data/") {
        SemanticCategory::Data
    } else {
        SemanticCategory::Unresolved
    }
}

pub(super) fn is_semantic_path(path: &str, category: SemanticCategory) -> bool {
    match category {
        SemanticCategory::Npc
        | SemanticCategory::Mob
        | SemanticCategory::Player
        | SemanticCategory::Nano
        | SemanticCategory::Ui
        | SemanticCategory::Data => true,
        SemanticCategory::World => path.starts_with("world/"),
        SemanticCategory::Audio => {
            path.starts_with("audio/music/")
                || path.starts_with("audio/voice/")
                || path.starts_with("audio/sfx/")
                || path.starts_with("audio/ambient/")
        }
        SemanticCategory::Unresolved => false,
    }
}
