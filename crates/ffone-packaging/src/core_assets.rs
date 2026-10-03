use super::*;

pub(super) const LEGACY_MANIFEST: &str = "asset-manifest.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct ProjectManifest {
    pub(super) schema: String,
    pub(super) protocol: u32,
    pub(super) locale: String,
    pub(super) source_pack: serde_json::Value,
    pub(super) files: Vec<ProjectFile>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecoveryArchiveIndex<'a> {
    pub(super) schema: &'static str,
    pub(super) source_manifest_blake3: String,
    pub(super) archived_files: usize,
    pub(super) archived_bytes: u64,
    pub(super) files: &'a [ProjectFile],
}

pub(super) fn package_asset_graph(
    options: &PackageOptions,
    graph: &AssetGraph,
) -> Result<PackageReport, String> {
    let destination = options.release_root.join("assets/game");
    fs::create_dir_all(&destination).map_err(|error| {
        format!(
            "cannot create package root {}: {error}",
            destination.display()
        )
    })?;

    let receipt_path = options.release_root.join(RECEIPT);
    let invalid_receipt = options.release_root.join(format!("{RECEIPT}.invalid"));
    if receipt_path.exists() {
        if invalid_receipt.exists() {
            fs::remove_file(&invalid_receipt).map_err(|error| error.to_string())?;
        }
        fs::rename(&receipt_path, &invalid_receipt).map_err(|error| {
            format!(
                "cannot invalidate old receipt {}: {error}",
                receipt_path.display()
            )
        })?;
    }

    let cache_path = options.release_root.join(CACHE);
    let old_cache = read_json::<PackageCache>(&cache_path).unwrap_or_default();
    let source_root = canonical_string(&options.source_root)?;
    let mut new_cache = PackageCache {
        schema: "ffone.package-cache.v4".to_owned(),
        source_root: source_root.clone(),
        files: BTreeMap::new(),
    };
    let cache_compatible =
        old_cache.schema == new_cache.schema && old_cache.source_root == source_root;

    let mut allowed = BTreeSet::new();
    let mut copied = 0;
    let mut reused = 0;
    for file in &graph.files {
        let relative = validate_relative(&file.path)?;
        allowed.insert(relative.clone());
        let source = options.source_root.join(&relative);
        let target = destination.join(&relative);
        let source_meta = regular_metadata(&source)?;
        if source_meta.len() != file.bytes {
            return Err(format!("source length mismatch for {:?}", file.path));
        }
        let source_modified = modified_nanos(&source_meta)?;
        let cached = cache_compatible
            .then(|| old_cache.files.get(&file.path))
            .flatten();
        let target_meta = fs::metadata(&target).ok().filter(|meta| meta.is_file());
        let target_modified = target_meta
            .as_ref()
            .map(modified_nanos)
            .transpose()?
            .unwrap_or(0);
        let fast_reuse = !is_editable_asset(&file.path)
            && cached.is_some_and(|cached| {
                cached.source_modified_nanos == source_modified
                    && cached.destination_modified_nanos == target_modified
            });
        if fast_reuse {
            reused += 1;
        } else {
            verify_graph_file(&source, file)?;
            let target_valid =
                !is_editable_asset(&file.path) && verify_graph_file(&target, file).is_ok();
            if !target_valid {
                copy_atomic(&source, &target)?;
                verify_graph_file(&target, file)?;
                copied += 1;
            } else {
                reused += 1;
            }
        }
        let destination_meta = regular_metadata(&target)?;
        new_cache.files.insert(
            file.path.clone(),
            CachedFile {
                source_modified_nanos: source_modified,
                destination_modified_nanos: modified_nanos(&destination_meta)?,
            },
        );
    }

    let removed = remove_stale(&destination, &allowed)?;
    write_atomic(&cache_path, &pretty_json(&new_cache)?)?;

    let groups = graph
        .groups
        .iter()
        .map(|(id, files)| ReleaseGroupSummary {
            id: id.clone(),
            file_count: files.len(),
        })
        .collect();
    let receipt = ReleaseReceipt {
        schema: "ffone.release-assets.v4",
        tool_version: env!("CARGO_PKG_VERSION"),
        packaging_mode: "loose-editable",
        asset_root: "assets/game",
        file_count: graph.files.len(),
        groups,
    };
    write_atomic(&receipt_path, &pretty_json(&receipt)?)?;
    if invalid_receipt.exists() {
        fs::remove_file(invalid_receipt).map_err(|error| error.to_string())?;
    }
    Ok(PackageReport {
        files: receipt.file_count,
        payload_bytes: graph.payload_bytes,
        copied,
        reused,
        removed,
    })
}

pub fn verify_asset_root(options: &VerifyOptions) -> Result<VerifyReport, String> {
    let graph = build_asset_graph(&options.asset_root, options.full)?;
    Ok(VerifyReport {
        files: graph.files.len(),
        payload_bytes: graph.payload_bytes,
        full: options.full,
    })
}

pub(super) fn read_manifest(root: &Path) -> Result<(ProjectManifest, Vec<u8>), String> {
    let path = root.join(LEGACY_MANIFEST);
    let bytes =
        fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let manifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid {}: {error}", path.display()))?;
    Ok((manifest, bytes))
}

pub(super) fn native_path(value: &str) -> PathBuf {
    value.split('/').collect()
}
