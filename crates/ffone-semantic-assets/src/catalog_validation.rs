use super::*;

pub const TREE_AUDIT_SCHEMA: &str = "ffone.semantic-tree-audit.v1";

#[derive(Debug, Error)]
pub enum SemanticAssetError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid JSON at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid semantic route plan: {0}")]
    InvalidPlan(String),
    #[error("asset verification failed: {0}")]
    Verification(String),
    #[error("semantic publication lock already exists: {0}")]
    Locked(PathBuf),
    #[error("asset manifest changed during publication; no files were committed")]
    ManifestChanged,
    #[error("transaction failed: {0}")]
    Transaction(String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditCounts {
    pub manifest_entries: u64,
    pub verified_entries: u64,
    pub missing_entries: u64,
    pub byte_mismatch_entries: u64,
    pub hash_mismatch_entries: u64,
    pub duplicate_manifest_paths: u64,
    pub hashed_filename_entries: u64,
    pub semantic_entries: u64,
    pub legacy_or_unresolved_entries: u64,
    pub blocked_entries: u64,
    pub blocker_occurrences: u64,
    pub duplicate_name_candidate_groups: u64,
    pub duplicate_name_candidate_entries: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetTreeAudit {
    pub schema: String,
    pub scope: String,
    pub manifest_path: String,
    pub manifest_blake3: String,
    pub manifest_schema: String,
    pub production_assets_mutated: bool,
    pub source_store_retained: bool,
    pub counts: AuditCounts,
    pub by_category: BTreeMap<SemanticCategory, u64>,
    pub by_kind: BTreeMap<AssetKind, u64>,
    pub blockers: Vec<BlockerSummary>,
}

/// Produce an exact, compact audit of every manifest entry and its on-disk hash.
pub fn audit_asset_tree(asset_root: &Path, manifest_path: &Path) -> Result<AssetTreeAudit> {
    let manifest_bytes = read_bytes(manifest_path)?;
    let manifest_blake3 = blake3::hash(&manifest_bytes).to_hex().to_string();
    let manifest: AssetManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|source| SemanticAssetError::Json {
            path: manifest_path.to_owned(),
            source,
        })?;
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(SemanticAssetError::Verification(format!(
            "expected manifest schema {MANIFEST_SCHEMA}, got {}",
            manifest.schema
        )));
    }

    let mut counts = AuditCounts {
        manifest_entries: manifest.files.len() as u64,
        ..AuditCounts::default()
    };
    let mut by_category = BTreeMap::new();
    let mut by_kind = BTreeMap::new();
    let mut blockers: BTreeMap<String, (u64, Vec<String>)> = BTreeMap::new();
    let mut seen_paths = BTreeSet::new();
    let mut duplicate_candidates: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for entry in &manifest.files {
        *by_kind.entry(entry.kind).or_insert(0) += 1;
        let category = classify_path(&entry.path);
        *by_category.entry(category).or_insert(0) += 1;

        let mut entry_blocked = false;
        let mut add_blocker = |code: &str| {
            entry_blocked = true;
            let record = blockers.entry(code.to_owned()).or_default();
            record.0 += 1;
            if record.1.len() < 12 {
                record.1.push(entry.path.clone());
            }
        };

        if !seen_paths.insert(entry.path.clone()) {
            counts.duplicate_manifest_paths += 1;
            add_blocker("duplicate_manifest_path");
        }

        let disk_path = asset_root.join(native_path(&entry.path));
        match hash_file(&disk_path) {
            Ok((bytes, hash)) => {
                if bytes != entry.bytes {
                    counts.byte_mismatch_entries += 1;
                    add_blocker("manifest_byte_count_mismatch");
                } else if hash != entry.blake3 {
                    counts.hash_mismatch_entries += 1;
                    add_blocker("manifest_blake3_mismatch");
                } else {
                    counts.verified_entries += 1;
                }
            }
            Err(SemanticAssetError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                counts.missing_entries += 1;
                add_blocker("manifest_file_missing");
            }
            Err(error) => return Err(error),
        }

        let hashed = has_content_hash_filename(&entry.path);
        if hashed {
            counts.hashed_filename_entries += 1;
            add_blocker("hashed_filename_requires_ownership_route");
            let parent = entry
                .path
                .rsplit_once('/')
                .map(|(parent, _)| parent)
                .unwrap_or("");
            let readable = remove_content_hash(&entry.path);
            duplicate_candidates
                .entry((parent.to_owned(), readable))
                .or_default()
                .push(entry.path.clone());
        }

        if is_semantic_path(&entry.path, category) && !hashed {
            counts.semantic_entries += 1;
        } else {
            counts.legacy_or_unresolved_entries += 1;
        }

        if entry.path.starts_with("models/world/") || entry.path.starts_with("worlds/") {
            add_blocker("legacy_world_alias_retained_until_runtime_migration");
        } else if entry.path.starts_with("models/") {
            add_blocker("flat_model_store_ownership_unproven");
        } else if entry.path.starts_with("textures/") {
            add_blocker("flat_texture_store_ownership_unproven");
        } else if entry.path.starts_with("fonts/") {
            add_blocker("flat_font_store_usage_unproven");
        } else if entry.path.starts_with("audio/") && hashed {
            add_blocker("audio_owner_or_usage_unproven");
        } else if entry.path.starts_with("data/") && hashed {
            add_blocker("data_owner_or_schema_route_unproven");
        }

        if entry.path.starts_with("characters/") && entry.path.contains(".textures/") {
            add_blocker("semantic_character_sidecar_layout_noncanonical");
        }

        if entry_blocked {
            counts.blocked_entries += 1;
        }
    }

    for paths in duplicate_candidates
        .values()
        .filter(|paths| paths.len() > 1)
    {
        counts.duplicate_name_candidate_groups += 1;
        counts.duplicate_name_candidate_entries += paths.len() as u64;
        let record = blockers
            .entry("duplicate_readable_name_requires_provenance_variants".to_owned())
            .or_default();
        record.0 += paths.len() as u64;
        for path in paths.iter().take(12usize.saturating_sub(record.1.len())) {
            record.1.push(path.clone());
        }
    }

    counts.blocker_occurrences = blockers.values().map(|(count, _)| *count).sum();
    let blockers = blockers
        .into_iter()
        .map(|(code, (count, samples))| BlockerSummary {
            code,
            count,
            samples,
        })
        .collect();

    Ok(AssetTreeAudit {
        schema: TREE_AUDIT_SCHEMA.to_owned(),
        scope: "every asset-manifest entry; bytes and BLAKE3 verified against disk".to_owned(),
        manifest_path: slash_path(manifest_path),
        manifest_blake3,
        manifest_schema: manifest.schema,
        production_assets_mutated: false,
        source_store_retained: true,
        counts,
        by_category,
        by_kind,
        blockers,
    })
}

/// Validate ownership, names, destinations, source hashes, duplicate variants, and GLB URIs.
pub fn validate_route_plan(
    asset_root: &Path,
    manifest_path: &Path,
    plan: &RoutePlan,
) -> Result<Vec<GlbProof>> {
    if plan.schema != ROUTE_PLAN_SCHEMA {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "expected schema {ROUTE_PLAN_SCHEMA}, got {}",
            plan.schema
        )));
    }
    if plan.source_build.trim().is_empty() {
        return Err(SemanticAssetError::InvalidPlan(
            "sourceBuild cannot be empty".to_owned(),
        ));
    }

    let manifest = read_manifest(manifest_path)?;
    let manifest_by_path: BTreeMap<&str, &AssetManifestEntry> = manifest
        .files
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let mut destinations = BTreeSet::new();
    let mut declared_groups: BTreeMap<&str, Vec<&SemanticRoute>> = BTreeMap::new();
    let mut ownership_name_groups: BTreeMap<
        (SemanticCategory, AssetKind, String),
        Vec<&SemanticRoute>,
    > = BTreeMap::new();
    let route_by_source: BTreeMap<&str, &SemanticRoute> = plan
        .routes
        .iter()
        .filter(|route| !matches!(route.content, RouteContent::GeneratedJson { .. }))
        .map(|route| (route.source_path.as_str(), route))
        .collect();

    for route in &plan.routes {
        validate_relative_path(&route.destination_path)?;
        if has_content_hash_filename(&route.destination_path) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "published filename contains a content hash: {}",
                route.destination_path
            )));
        }
        let prefix = route.category.required_prefix().ok_or_else(|| {
            SemanticAssetError::InvalidPlan(format!(
                "route {} cannot publish to unresolved category",
                route.destination_path
            ))
        })?;
        if !route.destination_path.starts_with(prefix) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "{} must be under {prefix}",
                route.destination_path
            )));
        }
        if !destinations.insert(route.destination_path.as_str()) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "duplicate destination {}",
                route.destination_path
            )));
        }
        validate_ownership(&route.ownership, &plan.source_build)?;
        if matches!(route.content, RouteContent::CopyExact) {
            ownership_name_groups
                .entry((
                    route.category,
                    route.kind,
                    route.ownership.true_legacy_name.to_lowercase(),
                ))
                .or_default()
                .push(route);
        }

        if let Some(group) = route.duplicate_group.as_deref() {
            declared_groups.entry(group).or_default().push(route);
        }

        if !matches!(route.content, RouteContent::GeneratedJson { .. }) {
            validate_relative_path(&route.source_path)?;
            let entry = manifest_by_path
                .get(route.source_path.as_str())
                .ok_or_else(|| {
                    SemanticAssetError::Verification(format!(
                        "source is absent from manifest: {}",
                        route.source_path
                    ))
                })?;
            let source = asset_root.join(native_path(&route.source_path));
            verify_manifest_entry(&source, entry)?;
        }
    }

    for (group, routes) in declared_groups {
        if routes.len() < 2 {
            continue;
        }
        validate_variant_routes(&format!("declared duplicate group {group}"), &routes)?;
    }
    for ((category, kind, true_name), routes) in ownership_name_groups {
        if routes.len() < 2 {
            continue;
        }
        validate_variant_routes(
            &format!("duplicate true legacy name {category:?}/{kind:?}/{true_name}"),
            &routes,
        )?;
    }

    let mut glb_proofs = Vec::new();
    for route in plan
        .routes
        .iter()
        .filter(|route| route.kind == AssetKind::Model)
    {
        if !route
            .destination_path
            .to_ascii_lowercase()
            .ends_with(".glb")
        {
            continue;
        }
        if !matches!(route.content, RouteContent::CopyExact) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "GLB route must be copy_exact to preserve bones, skinning, animations, scale and origin: {}",
                route.destination_path
            )));
        }
        let source = asset_root.join(native_path(&route.source_path));
        let mut proof = inspect_glb(&source, &route.destination_path)?;
        proof.byte_exact_copy = true;
        proof.uri_safe = glb_uris_are_safe(route, &proof.external_uris, &route_by_source)?;
        if !proof.uri_safe {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "external GLB URI would break after routing {}: {:?}",
                route.destination_path, proof.external_uris
            )));
        }
        glb_proofs.push(proof);
    }

    Ok(glb_proofs)
}

pub(super) fn validate_batch_destination_root(scope: &str, destination_root: &str) -> Result<()> {
    let parts: Vec<&str> = destination_root.split('/').collect();
    let valid = match scope {
        "worldMap" => {
            parts.len() == 4
                && parts[0] == "world"
                && parts[1] == "maps"
                && parts[2].starts_with("map_")
                && parts[3] == "terrain"
        }
        "tutorial" => {
            parts.len() == 5
                && parts[0] == "world"
                && parts[1] == "tutorial"
                && parts[2] == "terrain"
                && parts[3] == "tiles"
                && parts[4].starts_with("tile_")
        }
        _ => false,
    };
    if !valid
        || parts.iter().any(|part| {
            !part
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        })
    {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "non-canonical {scope} native terrain destinationRoot {destination_root}"
        )));
    }
    Ok(())
}

pub(super) fn validate_ownership(proof: &OwnershipProof, source_build: &str) -> Result<()> {
    for (field, value) in [
        ("trueLegacyName", proof.true_legacy_name.as_str()),
        ("authority", proof.authority.as_str()),
        ("sourceBuild", proof.source_build.as_str()),
        ("sourceArchive", proof.source_archive.as_str()),
        ("sourceAsset", proof.source_asset.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "ownership {field} cannot be empty"
            )));
        }
    }
    if proof.source_build != source_build {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "ownership sourceBuild {} differs from plan {}",
            proof.source_build, source_build
        )));
    }
    Ok(())
}

pub(super) fn validate_variant_routes(context: &str, routes: &[&SemanticRoute]) -> Result<()> {
    for route in routes {
        if !contains_variant_directory(&route.destination_path)
            || route
                .variant_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
            || route.duplicate_group.as_deref().is_none_or(str::is_empty)
        {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "{context} must use distinct variants/variant_NN destinations and carry duplicateGroup plus variantReason"
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_relative_path(path: &str) -> Result<()> {
    if path.is_empty() || path.contains('\\') || Path::new(path).is_absolute() {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "path must be a non-empty forward-slash relative path: {path:?}"
        )));
    }
    if Path::new(path)
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "path traversal or non-normal component is forbidden: {path}"
        )));
    }
    Ok(())
}
