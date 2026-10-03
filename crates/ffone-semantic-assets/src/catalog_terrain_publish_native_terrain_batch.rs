use super::*;

pub const NATIVE_TERRAIN_PUBLICATION_PLAN_SCHEMA: &str = "ffone.native-terrain-publication-plan.v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTerrainReconciliationReport {
    pub schema: String,
    pub status: String,
    pub source_manifest_blake3: String,
    pub final_manifest_blake3: String,
    pub published_files: Vec<String>,
    pub removed_manifest_paths: Vec<String>,
    pub removed_legacy_runtime_files: Vec<String>,
    pub terrain_glb_retained: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTerrainPublicationPlan {
    pub schema: String,
    pub status: String,
    pub source_output_root: String,
    pub entries: Vec<NativeTerrainPublicationEntry>,
    pub blocked: Vec<NativeTerrainPublicationBlocker>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTerrainPublicationEntry {
    pub scope: String,
    pub instance_id: String,
    pub source_root: String,
    pub destination_root: String,
    pub terrain_document: String,
    pub terrain_document_blake3: String,
    pub heightmap_path: String,
    pub heightmap_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_instance_document: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_instance_document_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_document: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_document_blake3: Option<String>,
    pub placement_status: String,
    pub source_bundle: String,
    #[serde(default)]
    pub payloads: Vec<NativeTerrainPublicationPayload>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTerrainPublicationBlocker {
    pub scope: String,
    pub instance_id: String,
    pub stage: String,
    pub code: String,
    pub message: String,
    pub placement_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTerrainBatchPublicationReport {
    pub schema: String,
    pub status: String,
    pub source_plan: String,
    pub source_plan_blake3: String,
    pub source_manifest_blake3: String,
    pub final_manifest_blake3: String,
    pub entries_published: usize,
    pub blocked_placements_retained_as_data: usize,
    pub published_files: Vec<String>,
    pub replaced_files: Vec<String>,
    pub removed_orphan_files: Vec<String>,
}

#[derive(Clone, Debug)]
pub(super) struct NativeTerrainBatchFile {
    pub(super) source: NativeTerrainBatchFileSource,
    pub(super) destination_path: String,
    pub(super) kind: AssetKind,
    pub(super) expected_blake3: String,
}

#[derive(Clone, Debug)]
pub(super) enum NativeTerrainBatchFileSource {
    File(PathBuf),
    Generated(Vec<u8>),
}

/// Atomically makes the two proven retro world tiles authoritative in the project manifest.
///
/// Unlike [`publish_route_plan`], this bootstrap reconciliation deliberately accepts canonical
/// files that are newer than (or absent from) the old manifest. Every planned file is staged and
/// re-hashed, generated provenance/catalog JSON replaces its previous version with rollback, and
/// the exact legacy terrain-GLB/runtime-alias allowlist is quarantined before the manifest swap.
/// No path outside that fixed allowlist can be deleted by this operation.
pub fn reconcile_retro_world_native_terrain(
    asset_root: &Path,
    manifest_path: &Path,
) -> Result<NativeTerrainReconciliationReport> {
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
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(SemanticAssetError::Verification(format!(
            "expected manifest schema {MANIFEST_SCHEMA}, got {}",
            manifest.schema
        )));
    }

    let plan = build_retro_world_plan(asset_root)?;
    validate_native_terrain_reconciliation_plan(asset_root, &plan)?;
    let stage_root = asset_root.join(format!(
        ".semantic-tree-stage-{}-{}",
        std::process::id(),
        unique_stamp()
    ));
    fs::create_dir(&stage_root).map_err(|source| io_at(&stage_root, source))?;

    let result = (|| {
        let mut rendered = Vec::new();
        for route in &plan.routes {
            let staged_path = stage_root
                .join("rendered")
                .join(native_path(&route.destination_path));
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
                glb_proof: None,
            });
        }

        if blake3::hash(&read_bytes(manifest_path)?).to_hex().as_str() != manifest_before_hash {
            return Err(SemanticAssetError::ManifestChanged);
        }

        let route_paths: BTreeSet<&str> = rendered
            .iter()
            .map(|route| route.route.destination_path.as_str())
            .collect();
        let legacy_paths: BTreeSet<&str> = RETRO_WORLD_LEGACY_RUNTIME_PATHS.into_iter().collect();
        let removed_manifest_paths: Vec<String> = manifest
            .files
            .iter()
            .filter(|entry| legacy_paths.contains(entry.path.as_str()))
            .map(|entry| entry.path.clone())
            .collect();
        manifest.files.retain(|entry| {
            !route_paths.contains(entry.path.as_str())
                && !legacy_paths.contains(entry.path.as_str())
        });
        for route in &rendered {
            let source_path = match route.route.content {
                RouteContent::GeneratedJson { .. } => {
                    format!("semantic-generated/{}", route.route.destination_path)
                }
                _ => route.route.source_path.clone(),
            };
            manifest.files.push(AssetManifestEntry {
                source_path,
                path: route.route.destination_path.clone(),
                kind: route.route.kind,
                bytes: route.bytes,
                blake3: route.blake3.clone(),
            });
        }
        manifest.files.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.source_path.cmp(&right.source_path))
        });
        if manifest
            .files
            .windows(2)
            .any(|pair| pair[0].path == pair[1].path)
        {
            return Err(SemanticAssetError::Transaction(
                "native terrain reconciliation would create duplicate manifest paths".to_owned(),
            ));
        }
        if manifest
            .files
            .iter()
            .any(|entry| entry.path.to_ascii_lowercase().ends_with("terrain.glb"))
        {
            return Err(SemanticAssetError::Transaction(
                "native terrain reconciliation retained a terrain.glb manifest entry".to_owned(),
            ));
        }
        let mut next_manifest =
            serde_json::to_vec_pretty(&manifest).map_err(|source| SemanticAssetError::Json {
                path: manifest_path.to_owned(),
                source,
            })?;
        next_manifest.push(b'\n');
        let next_path = stage_root.join("asset-manifest.next.json");
        write_bytes(&next_path, &next_manifest)?;

        let mut installed_generated: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
        let mut quarantined_legacy: Vec<(PathBuf, PathBuf)> = Vec::new();
        let install_result = (|| {
            for route in &rendered {
                if !matches!(route.route.content, RouteContent::GeneratedJson { .. }) {
                    continue;
                }
                let destination = asset_root.join(native_path(&route.route.destination_path));
                let backup = if destination.exists() {
                    let backup = stage_root
                        .join("replaced")
                        .join(native_path(&route.route.destination_path));
                    if let Some(parent) = backup.parent() {
                        fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                    }
                    fs::rename(&destination, &backup)
                        .map_err(|source| io_at(&destination, source))?;
                    Some(backup)
                } else {
                    None
                };
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                }
                if let Err(source) = fs::rename(&route.staged_path, &destination) {
                    if let Some(backup) = &backup {
                        let _ = fs::rename(backup, &destination);
                    }
                    return Err(io_at(&destination, source));
                }
                installed_generated.push((destination, backup));
            }

            for legacy_path in RETRO_WORLD_LEGACY_RUNTIME_PATHS {
                let original = asset_root.join(native_path(legacy_path));
                if !original.exists() {
                    continue;
                }
                if !original.is_file() {
                    return Err(SemanticAssetError::Transaction(format!(
                        "legacy runtime allowlist path is not a file: {legacy_path}"
                    )));
                }
                let quarantine = stage_root.join("quarantine").join(native_path(legacy_path));
                if let Some(parent) = quarantine.parent() {
                    fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                }
                fs::rename(&original, &quarantine).map_err(|source| io_at(&original, source))?;
                quarantined_legacy.push((original, quarantine));
            }

            for route in &rendered {
                let destination = asset_root.join(native_path(&route.route.destination_path));
                let (bytes, hash) = hash_file(&destination)?;
                if bytes != route.bytes || hash != route.blake3 {
                    return Err(SemanticAssetError::Transaction(format!(
                        "pre-manifest native terrain hash verification failed: {}",
                        route.route.destination_path
                    )));
                }
            }
            if blake3::hash(&read_bytes(manifest_path)?).to_hex().as_str() != manifest_before_hash {
                return Err(SemanticAssetError::ManifestChanged);
            }
            replace_manifest_transactionally(manifest_path, &next_path)
        })();
        if let Err(error) = install_result {
            restore_quarantined(&quarantined_legacy);
            rollback_replaced(&installed_generated);
            return Err(error);
        }

        let final_manifest_blake3 = blake3::hash(&next_manifest).to_hex().to_string();
        Ok(NativeTerrainReconciliationReport {
            schema: "ffone.native-terrain-reconciliation.v1".to_owned(),
            status: "committed-native-heightmaps-legacy-terrain-glb-removed".to_owned(),
            source_manifest_blake3: manifest_before_hash,
            final_manifest_blake3,
            published_files: rendered
                .iter()
                .map(|route| route.route.destination_path.clone())
                .collect(),
            removed_manifest_paths,
            removed_legacy_runtime_files: quarantined_legacy
                .iter()
                .map(|(path, _)| {
                    path.strip_prefix(asset_root)
                        .map(slash_path)
                        .unwrap_or_else(|_| slash_path(path))
                })
                .collect(),
            terrain_glb_retained: false,
        })
    })();

    let _ = fs::remove_dir_all(&stage_root);
    remove_empty_legacy_world_directories(asset_root);
    drop(lock);
    result
}

/// Publish an exporter-owned native-terrain batch without requiring scene placement to be known.
///
/// Every entry publishes its complete TerrainData payload closure. A `blocked` placement remains a
/// typed blocker, but its verified heightmap/weights/layers/gameplay-attributes are still committed.
/// Destination roots are restricted to canonical world-map and tutorial terrain directories.
pub fn publish_native_terrain_batch(
    asset_root: &Path,
    manifest_path: &Path,
    publication_plan_path: &Path,
) -> Result<NativeTerrainBatchPublicationReport> {
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
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(SemanticAssetError::Verification(format!(
            "expected manifest schema {MANIFEST_SCHEMA}, got {}",
            manifest.schema
        )));
    }
    let plan_bytes = read_bytes(publication_plan_path)?;
    let plan_hash = blake3::hash(&plan_bytes).to_hex().to_string();
    let plan: NativeTerrainPublicationPlan =
        serde_json::from_slice(&plan_bytes).map_err(|source| SemanticAssetError::Json {
            path: publication_plan_path.to_owned(),
            source,
        })?;
    if plan.schema != NATIVE_TERRAIN_PUBLICATION_PLAN_SCHEMA {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "expected schema {NATIVE_TERRAIN_PUBLICATION_PLAN_SCHEMA}, got {}",
            plan.schema
        )));
    }
    if plan.entries.is_empty() {
        return Err(SemanticAssetError::InvalidPlan(
            "native terrain publication plan has no exported entries".to_owned(),
        ));
    }
    let plan_root = publication_plan_path.parent().ok_or_else(|| {
        SemanticAssetError::InvalidPlan("publication plan has no parent directory".to_owned())
    })?;
    let (files, destination_roots) = collect_native_terrain_batch_files(plan_root, &plan)?;
    let owned_scene_paths = plan
        .entries
        .iter()
        .map(native_terrain_owned_scene_path)
        .collect::<Result<BTreeSet<_>>>()?;
    let stage_root = asset_root.join(format!(
        ".semantic-tree-stage-{}-{}",
        std::process::id(),
        unique_stamp()
    ));
    fs::create_dir(&stage_root).map_err(|source| io_at(&stage_root, source))?;

    let result = (|| {
        let mut staged_files = Vec::new();
        for file in files.values() {
            let staged = stage_root
                .join("rendered")
                .join(native_path(&file.destination_path));
            if let Some(parent) = staged.parent() {
                fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
            }
            match &file.source {
                NativeTerrainBatchFileSource::File(source_path) => {
                    fs::copy(source_path, &staged).map_err(|source| io_at(&staged, source))?;
                }
                NativeTerrainBatchFileSource::Generated(bytes) => {
                    write_bytes(&staged, bytes)?;
                }
            }
            let (bytes, hash) = hash_file(&staged)?;
            if hash != file.expected_blake3 {
                let source_label = match &file.source {
                    NativeTerrainBatchFileSource::File(path) => path.display().to_string(),
                    NativeTerrainBatchFileSource::Generated(_) => {
                        format!("generated:{}", file.destination_path)
                    }
                };
                return Err(SemanticAssetError::Verification(format!(
                    "batch source hash changed while staging {}: expected {}, got {}",
                    source_label, file.expected_blake3, hash
                )));
            }
            staged_files.push((
                file.clone(),
                staged,
                AssetManifestEntry {
                    source_path: format!("native-terrain-export/{}", file.destination_path),
                    path: file.destination_path.clone(),
                    kind: file.kind,
                    bytes,
                    blake3: hash,
                },
            ));
        }
        if blake3::hash(&read_bytes(manifest_path)?).to_hex().as_str() != manifest_before_hash {
            return Err(SemanticAssetError::ManifestChanged);
        }

        let planned_paths: BTreeSet<String> = files.keys().cloned().collect();
        manifest.files.retain(|entry| {
            !planned_paths.contains(&entry.path)
                && !owned_scene_paths.contains(&entry.path)
                && !destination_roots
                    .iter()
                    .any(|root| entry.path == *root || entry.path.starts_with(&format!("{root}/")))
        });
        manifest
            .files
            .extend(staged_files.iter().map(|(_, _, entry)| entry.clone()));
        manifest.files.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.source_path.cmp(&right.source_path))
        });
        if manifest
            .files
            .windows(2)
            .any(|pair| pair[0].path == pair[1].path)
        {
            return Err(SemanticAssetError::Transaction(
                "native terrain batch would create duplicate manifest paths".to_owned(),
            ));
        }
        if manifest
            .files
            .iter()
            .any(|entry| entry.path.to_ascii_lowercase().ends_with("terrain.glb"))
        {
            return Err(SemanticAssetError::Transaction(
                "native terrain batch cannot coexist with a Terrain.glb manifest entry".to_owned(),
            ));
        }
        let mut next_manifest =
            serde_json::to_vec_pretty(&manifest).map_err(|source| SemanticAssetError::Json {
                path: manifest_path.to_owned(),
                source,
            })?;
        next_manifest.push(b'\n');
        let next_path = stage_root.join("asset-manifest.next.json");
        write_bytes(&next_path, &next_manifest)?;

        let mut installed: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
        let mut quarantined: Vec<(PathBuf, PathBuf)> = Vec::new();
        let install_result = (|| {
            for (file, staged, _) in &staged_files {
                let destination = asset_root.join(native_path(&file.destination_path));
                if destination.exists() {
                    let (_, existing_hash) = hash_file(&destination)?;
                    if existing_hash == file.expected_blake3 {
                        continue;
                    }
                    let backup = stage_root
                        .join("replaced")
                        .join(native_path(&file.destination_path));
                    if let Some(parent) = backup.parent() {
                        fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                    }
                    fs::rename(&destination, &backup)
                        .map_err(|source| io_at(&destination, source))?;
                    if let Err(source) = fs::rename(staged, &destination) {
                        let _ = fs::rename(&backup, &destination);
                        return Err(io_at(&destination, source));
                    }
                    installed.push((destination, Some(backup)));
                } else {
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                    }
                    fs::rename(staged, &destination)
                        .map_err(|source| io_at(&destination, source))?;
                    installed.push((destination, None));
                }
            }

            for root in &destination_roots {
                let disk_root = asset_root.join(native_path(root));
                for orphan in collect_files_recursive(&disk_root)? {
                    let relative =
                        orphan
                            .strip_prefix(asset_root)
                            .map(slash_path)
                            .map_err(|_| {
                                SemanticAssetError::Transaction(format!(
                                    "batch orphan escaped asset root: {}",
                                    orphan.display()
                                ))
                            })?;
                    if planned_paths.contains(&relative) {
                        continue;
                    }
                    let quarantine = stage_root.join("quarantine").join(native_path(&relative));
                    if let Some(parent) = quarantine.parent() {
                        fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                    }
                    fs::rename(&orphan, &quarantine).map_err(|source| io_at(&orphan, source))?;
                    quarantined.push((orphan, quarantine));
                }
            }
            for relative in &owned_scene_paths {
                if planned_paths.contains(relative) {
                    continue;
                }
                let orphan = asset_root.join(native_path(relative));
                if !orphan.is_file() {
                    continue;
                }
                let quarantine = stage_root.join("quarantine").join(native_path(relative));
                if let Some(parent) = quarantine.parent() {
                    fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
                }
                fs::rename(&orphan, &quarantine).map_err(|source| io_at(&orphan, source))?;
                quarantined.push((orphan, quarantine));
            }

            for file in files.values() {
                let destination = asset_root.join(native_path(&file.destination_path));
                let (_, hash) = hash_file(&destination)?;
                if hash != file.expected_blake3 {
                    return Err(SemanticAssetError::Transaction(format!(
                        "batch destination hash mismatch after install: {}",
                        file.destination_path
                    )));
                }
            }
            if blake3::hash(&read_bytes(manifest_path)?).to_hex().as_str() != manifest_before_hash {
                return Err(SemanticAssetError::ManifestChanged);
            }
            replace_manifest_transactionally(manifest_path, &next_path)
        })();
        if let Err(error) = install_result {
            restore_quarantined(&quarantined);
            rollback_replaced(&installed);
            return Err(error);
        }

        Ok(NativeTerrainBatchPublicationReport {
            schema: "ffone.native-terrain-batch-publication.v1".to_owned(),
            status: "committed-complete-terrain-data-placement-blockers-preserved".to_owned(),
            source_plan: slash_path(publication_plan_path),
            source_plan_blake3: plan_hash,
            source_manifest_blake3: manifest_before_hash,
            final_manifest_blake3: blake3::hash(&next_manifest).to_hex().to_string(),
            entries_published: plan.entries.len(),
            blocked_placements_retained_as_data: plan
                .entries
                .iter()
                .filter(|entry| entry.placement_status == "blocked")
                .count(),
            published_files: files.keys().cloned().collect(),
            replaced_files: installed
                .iter()
                .filter_map(|(path, backup)| backup.as_ref().map(|_| path))
                .filter_map(|path| path.strip_prefix(asset_root).ok())
                .map(slash_path)
                .collect(),
            removed_orphan_files: quarantined
                .iter()
                .filter_map(|(path, _)| path.strip_prefix(asset_root).ok())
                .map(slash_path)
                .collect(),
        })
    })();

    let _ = fs::remove_dir_all(&stage_root);
    drop(lock);
    result
}

pub(super) fn native_terrain_owned_scene_path(entry: &NativeTerrainPublicationEntry) -> Result<String> {
    let destination_root = entry
        .destination_root
        .strip_prefix("assets/game/")
        .ok_or_else(|| {
            SemanticAssetError::InvalidPlan(format!(
                "{} destinationRoot must begin assets/game/",
                entry.instance_id
            ))
        })?;
    validate_relative_path(destination_root)?;
    match entry.scope.as_str() {
        "worldMap" => destination_root
            .strip_suffix("/terrain")
            .map(|map_root| format!("{map_root}/scene.json"))
            .ok_or_else(|| {
                SemanticAssetError::InvalidPlan(format!(
                    "{} worldMap destination does not end in /terrain",
                    entry.instance_id
                ))
            }),
        "tutorial" => Ok(format!("{destination_root}/scene.json")),
        other => Err(SemanticAssetError::InvalidPlan(format!(
            "unsupported native terrain scope {other}"
        ))),
    }
}
