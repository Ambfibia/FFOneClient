use super::*;

/// Fully reconciles a release source tree and hashes its immutable payloads, returning an opaque
/// snapshot that can be consumed by [`package_loose_from_snapshot`].
///
/// Keeping the snapshot across the executable build lets the complete release
/// workflow avoid hashing the same source payloads a second time. Packaging
/// still performs a metadata reconciliation and rejects any source drift.
pub fn snapshot_release_source(
    asset_root: PathBuf,
) -> Result<(ReleaseSourceReport, ReleaseSourceSnapshot), String> {
    let graph = build_asset_graph(&asset_root, true)?;
    let report = ReleaseSourceReport {
        groups: graph.groups.len(),
        files: graph.files.len(),
        payload_bytes: graph.payload_bytes,
        root_blake3: graph.root_blake3.clone(),
    };
    let canonical_source_root = fs::canonicalize(&asset_root).map_err(|error| {
        format!(
            "cannot resolve release source {}: {error}",
            asset_root.display()
        )
    })?;
    let snapshot = ReleaseSourceSnapshot {
        canonical_source_root,
        graph,
    };
    Ok((report, snapshot))
}

pub fn package_loose(options: &PackageOptions) -> Result<PackageReport, String> {
    let graph = build_asset_graph(&options.source_root, true)?;
    package_asset_graph(options, &graph)
}

/// Packages a source tree that was fully verified by
/// [`snapshot_release_source`] earlier in the same release workflow.
///
/// A cheap metadata reconciliation and the manifest identity check make source
/// drift fail closed. The final release workflow must still fully verify the
/// packaged destination independently.
pub fn package_loose_from_snapshot(
    options: &PackageOptions,
    snapshot: &ReleaseSourceSnapshot,
) -> Result<PackageReport, String> {
    let canonical_source_root = fs::canonicalize(&options.source_root).map_err(|error| {
        format!(
            "cannot resolve release source {}: {error}",
            options.source_root.display()
        )
    })?;
    if canonical_source_root != snapshot.canonical_source_root {
        return Err(format!(
            "release source snapshot belongs to {}, not {}",
            snapshot.canonical_source_root.display(),
            canonical_source_root.display()
        ));
    }
    validate_domain_roots(&options.source_root, false)?;
    for entry in &snapshot.graph.files {
        if is_editable_asset(&entry.path) {
            continue;
        }
        let source = options.source_root.join(native_path(&entry.path));
        let metadata = regular_metadata(&source)?;
        if metadata.len() != entry.bytes || modified_nanos(&metadata)? != entry.modified_nanos {
            return Err(format!(
                "release source asset {:?} changed after full verification; restart the release build",
                entry.path
            ));
        }
    }
    let mut graph = snapshot.graph.clone();
    refresh_editable_metadata(&options.source_root, &mut graph)?;
    package_asset_graph(options, &graph)
}

/// Restores payloads which are absent from `asset_root` from a verified donor.
///
/// This operation deliberately has no overwrite path. Existing payloads are fully
/// verified and any mismatch aborts the preflight. Every missing payload must have
/// an identical manifest entry and a valid physical donor before the first target
/// payload is created. Each individual commit is atomic and no-clobbering.
pub fn restore_missing_assets(options: &RestoreOptions) -> Result<RestoreReport, String> {
    validate_regular_directory(&options.asset_root, "asset root")?;
    validate_regular_directory(&options.donor_root, "donor root")?;

    let (manifest, _) = read_manifest(&options.asset_root)?;
    let (donor_manifest, _) = read_manifest(&options.donor_root)?;
    validate_manifest(&manifest)?;
    validate_manifest(&donor_manifest)?;

    let donor_files = donor_manifest
        .files
        .iter()
        .map(|file| (file.path.to_lowercase(), file))
        .collect::<BTreeMap<_, _>>();
    let mut restore = Vec::<ProjectFile>::new();
    let mut already_valid_files = 0_usize;
    let mut already_valid_bytes = 0_u64;
    let mut missing_files = 0_usize;
    let mut missing_bytes = 0_u64;

    // Complete preflight first: this prevents a bad or incomplete donor from
    // producing a partially restored target tree.
    for expected in &manifest.files {
        let relative = validate_relative(&expected.path)?;
        let target = options.asset_root.join(&relative);
        match fs::symlink_metadata(&target) {
            Ok(_) => {
                verify_file(&target, expected, true).map_err(|error| {
                    format!(
                        "existing asset failed full verification; refusing to overwrite: {error}"
                    )
                })?;
                already_valid_files = already_valid_files
                    .checked_add(1)
                    .ok_or_else(|| "already-valid file count overflow".to_owned())?;
                already_valid_bytes = already_valid_bytes
                    .checked_add(expected.bytes)
                    .ok_or_else(|| "already-valid byte count overflow".to_owned())?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(donor_entry) = donor_files.get(&expected.path.to_lowercase()) else {
                    missing_files = missing_files
                        .checked_add(1)
                        .ok_or_else(|| "missing donor file count overflow".to_owned())?;
                    missing_bytes = missing_bytes
                        .checked_add(expected.bytes)
                        .ok_or_else(|| "missing donor byte count overflow".to_owned())?;
                    continue;
                };
                if *donor_entry != expected {
                    return Err(format!(
                        "donor manifest identity mismatch for {:?}: target={expected:?}, donor={donor_entry:?}",
                        expected.path
                    ));
                }
                let donor = options.donor_root.join(&relative);
                match fs::symlink_metadata(&donor) {
                    Ok(_) => {
                        verify_file(&donor, expected, true).map_err(|error| {
                            format!(
                                "donor asset failed full verification for {:?}: {error}",
                                expected.path
                            )
                        })?;
                        restore.push(expected.clone());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        missing_files = missing_files
                            .checked_add(1)
                            .ok_or_else(|| "missing donor file count overflow".to_owned())?;
                        missing_bytes = missing_bytes
                            .checked_add(expected.bytes)
                            .ok_or_else(|| "missing donor byte count overflow".to_owned())?;
                    }
                    Err(error) => {
                        return Err(format!("cannot inspect {}: {error}", donor.display()));
                    }
                }
            }
            Err(error) => return Err(format!("cannot inspect {}: {error}", target.display())),
        }
    }

    if missing_files != 0 {
        return Err(format!(
            "restore preflight failed: donor is missing {missing_files} files ({missing_bytes} bytes); restored=0 files (0 bytes), already-valid={already_valid_files} files ({already_valid_bytes} bytes)"
        ));
    }

    let mut restored_files = 0_usize;
    let mut restored_bytes = 0_u64;
    for expected in &restore {
        let relative = validate_relative(&expected.path)?;
        let donor = options.donor_root.join(&relative);
        let target = options.asset_root.join(&relative);
        ensure_regular_parent(&options.asset_root, &relative)?;
        restore_one_no_clobber(&donor, &target, expected)?;
        restored_files = restored_files
            .checked_add(1)
            .ok_or_else(|| "restored file count overflow".to_owned())?;
        restored_bytes = restored_bytes
            .checked_add(expected.bytes)
            .ok_or_else(|| "restored byte count overflow".to_owned())?;
    }

    Ok(RestoreReport {
        manifest_files: manifest.files.len(),
        restored_files,
        restored_bytes,
        already_valid_files,
        already_valid_bytes,
        missing_files,
        missing_bytes,
    })
}

/// Moves the flat legacy recovery payload out of the runtime asset tree.
///
/// Only direct `audio/*.ogg` and `models/*.glb` manifest entries are selected.
/// Semantic subdirectories are never archived. Every selected source is fully
/// verified, the external archive is completed first, and only then is the
/// runtime manifest reduced and the redundant source path removed.
pub fn archive_recovery_assets(
    options: &ArchiveRecoveryOptions,
) -> Result<ArchiveRecoveryReport, String> {
    let asset_root = fs::canonicalize(&options.asset_root).map_err(|error| {
        format!(
            "cannot resolve asset root {}: {error}",
            options.asset_root.display()
        )
    })?;
    if !asset_root.is_dir() {
        return Err(format!(
            "asset root is not a directory: {}",
            asset_root.display()
        ));
    }
    if options.archive_root.exists() {
        return Err(format!(
            "recovery archive already exists; refusing to overwrite {}",
            options.archive_root.display()
        ));
    }
    let archive_parent = options
        .archive_root
        .parent()
        .ok_or_else(|| {
            format!(
                "recovery archive has no parent: {}",
                options.archive_root.display()
            )
        })?
        .to_path_buf();
    fs::create_dir_all(&archive_parent).map_err(|error| {
        format!(
            "cannot create recovery archive parent {}: {error}",
            archive_parent.display()
        )
    })?;
    let archive_parent = fs::canonicalize(&archive_parent).map_err(|error| {
        format!(
            "cannot resolve recovery archive parent {}: {error}",
            archive_parent.display()
        )
    })?;
    let archive_name = options
        .archive_root
        .file_name()
        .ok_or_else(|| {
            format!(
                "recovery archive has no final name: {}",
                options.archive_root.display()
            )
        })?
        .to_owned();
    let archive_root = archive_parent.join(archive_name);
    if archive_root.starts_with(&asset_root) || asset_root.starts_with(&archive_root) {
        return Err(
            "recovery archive and runtime asset root must not contain each other".to_owned(),
        );
    }

    let (mut manifest, manifest_bytes) = read_manifest(&asset_root)?;
    validate_manifest(&manifest)?;
    let mut archived = manifest
        .files
        .iter()
        .filter(|entry| is_flat_recovery_payload(&entry.path))
        .cloned()
        .collect::<Vec<_>>();
    archived.sort_by(|left, right| left.path.cmp(&right.path));
    if archived.is_empty() {
        return Err("asset manifest contains no flat recovery payload to archive".to_owned());
    }
    let archived_paths = archived
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    let archived_bytes = checked_payload_bytes(&archived, "recovery archive")?;

    let stage = tempfile::Builder::new()
        .prefix(".ffone-recovery-archive-")
        .tempdir_in(&archive_parent)
        .map_err(|error| {
            format!(
                "cannot create recovery archive staging directory in {}: {error}",
                archive_parent.display()
            )
        })?;
    let mut hard_linked_files = 0_usize;
    let mut copied_files = 0_usize;
    for entry in &archived {
        let source = asset_root.join(validate_relative(&entry.path)?);
        verify_file(&source, entry, true)?;
        let target = stage.path().join(validate_relative(&entry.path)?);
        let parent = target
            .parent()
            .ok_or_else(|| format!("archive target has no parent: {}", target.display()))?;
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "cannot create recovery archive directory {}: {error}",
                parent.display()
            )
        })?;
        match fs::hard_link(&source, &target) {
            Ok(()) => hard_linked_files += 1,
            Err(_) => {
                let copied = fs::copy(&source, &target).map_err(|error| {
                    format!(
                        "cannot archive {} as {}: {error}",
                        source.display(),
                        target.display()
                    )
                })?;
                if copied != entry.bytes {
                    return Err(format!(
                        "short recovery copy for {:?}: expected {}, copied {copied}",
                        entry.path, entry.bytes
                    ));
                }
                copied_files += 1;
            }
        }
        verify_file(&target, entry, true)?;
    }
    let archive_index = RecoveryArchiveIndex {
        schema: "ffone.recovery-archive.v1",
        source_manifest_blake3: blake3::hash(&manifest_bytes).to_hex().to_string(),
        archived_files: archived.len(),
        archived_bytes,
        files: &archived,
    };
    fs::write(
        stage.path().join("archive-index.json"),
        pretty_json(&archive_index)?,
    )
    .map_err(|error| format!("cannot write recovery archive index: {error}"))?;
    let kept_stage = stage.keep();
    if let Err(error) = fs::rename(&kept_stage, &archive_root) {
        let _ = fs::remove_dir_all(&kept_stage);
        return Err(format!(
            "cannot publish recovery archive {}: {error}",
            archive_root.display()
        ));
    }

    manifest
        .files
        .retain(|entry| !archived_paths.contains(entry.path.as_str()));
    let next_manifest = pretty_json(&manifest)?;
    if let Err(error) = write_atomic(&asset_root.join(LEGACY_MANIFEST), &next_manifest) {
        return Err(format!(
            "recovery archive is safe at {}, but runtime manifest update failed: {error}",
            archive_root.display()
        ));
    }
    for entry in &archived {
        let source = asset_root.join(validate_relative(&entry.path)?);
        fs::remove_file(&source).map_err(|error| {
            format!(
                "recovery archive and manifest are committed, but redundant runtime file {} could not be removed: {error}",
                source.display()
            )
        })?;
    }
    remove_empty_directory_if_present(&asset_root.join("models").join("world"))?;
    remove_empty_directory_if_present(&asset_root.join("models"))?;

    Ok(ArchiveRecoveryReport {
        archived_files: archived.len(),
        archived_bytes,
        hard_linked_files,
        copied_files,
        remaining_manifest_files: manifest.files.len(),
    })
}

pub(super) fn remove_empty_directory_if_present(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!("cannot inspect {}: {error}", path.display()));
        }
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(format!(
            "expected an ordinary directory while cleaning {}",
            path.display()
        ));
    }
    if fs::read_dir(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?
        .next()
        .is_none()
    {
        fs::remove_dir(path)
            .map_err(|error| format!("cannot remove empty {}: {error}", path.display()))?;
    }
    Ok(())
}

pub(super) fn verify_file(path: &Path, expected: &ProjectFile, full: bool) -> Result<(), String> {
    let metadata = regular_metadata(path)?;
    if metadata.len() != expected.bytes {
        return Err(format!(
            "{} length mismatch: expected {}, found {}",
            path.display(),
            expected.bytes,
            metadata.len()
        ));
    }
    if full {
        let actual = hash_file(path)?;
        if actual != expected.blake3 {
            return Err(format!(
                "{} BLAKE3 mismatch: expected {}, found {actual}",
                path.display(),
                expected.blake3
            ));
        }
    }
    Ok(())
}

pub(super) fn verify_graph_file(path: &Path, expected: &graph::AssetGraphEntry) -> Result<(), String> {
    let metadata = regular_metadata(path)?;
    if metadata.len() != expected.bytes {
        return Err(format!(
            "{} length mismatch: expected {}, found {}",
            path.display(),
            expected.bytes,
            metadata.len()
        ));
    }
    if let Some(expected_hash) = &expected.blake3 {
        let actual = hash_file(path)?;
        if actual != *expected_hash {
            return Err(format!(
                "{} BLAKE3 mismatch: expected {}, found {actual}",
                path.display(),
                expected_hash
            ));
        }
    }
    Ok(())
}

pub(super) fn regular_metadata(path: &Path) -> Result<fs::Metadata, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(format!("{} is not a regular file", path.display()));
    }
    Ok(metadata)
}

pub(super) fn ensure_regular_parent(root: &Path, relative: &Path) -> Result<(), String> {
    validate_regular_directory(root, "asset root")?;
    let parent = relative
        .parent()
        .ok_or_else(|| format!("asset path {} has no parent", relative.display()))?;
    let mut current = root.to_path_buf();
    for component in parent.components() {
        let Component::Normal(segment) = component else {
            return Err(format!("unsafe asset parent path {}", relative.display()));
        };
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            }
            Ok(_) => {
                return Err(format!(
                    "asset parent {} is not a regular directory",
                    current.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match fs::create_dir(&current) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        return Err(format!("cannot create {}: {error}", current.display()));
                    }
                }
                validate_regular_directory(&current, "asset parent")?;
            }
            Err(error) => return Err(format!("cannot inspect {}: {error}", current.display())),
        }
    }
    Ok(())
}

pub(super) fn restore_one_no_clobber(
    donor: &Path,
    target: &Path,
    expected: &ProjectFile,
) -> Result<(), String> {
    // Re-verify immediately before copying so a donor modified after preflight
    // cannot silently enter the target.
    verify_file(donor, expected, true).map_err(|error| {
        format!(
            "donor asset changed after preflight for {:?}: {error}",
            expected.path
        )
    })?;
    match fs::symlink_metadata(target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => {
            return Err(format!(
                "target {} appeared after preflight; refusing to overwrite it",
                target.display()
            ));
        }
        Err(error) => return Err(format!("cannot inspect {}: {error}", target.display())),
    }

    let parent = target
        .parent()
        .ok_or_else(|| format!("{} has no parent", target.display()))?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("cannot stage {}: {error}", target.display()))?;
    copy_verified_payload(donor, expected, staged.as_file_mut())?;
    staged
        .as_file_mut()
        .sync_all()
        .map_err(|error| format!("cannot sync staged asset for {}: {error}", target.display()))?;
    verify_file(staged.path(), expected, true)?;
    let committed = staged.persist_noclobber(target).map_err(|error| {
        format!(
            "cannot atomically commit {} without overwriting: {}",
            target.display(),
            error.error
        )
    })?;
    drop(committed);
    verify_file(target, expected, true).map_err(|error| {
        format!(
            "restored target failed post-copy verification for {:?}: {error}",
            expected.path
        )
    })
}

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub(super) fn modified_nanos(metadata: &fs::Metadata) -> Result<u128, String> {
    metadata
        .modified()
        .map_err(|error| error.to_string())?
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| error.to_string())
}

pub(super) fn canonical_string(path: &Path) -> Result<String, String> {
    fs::canonicalize(path)
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| format!("cannot canonicalize {}: {error}", path.display()))
}

pub(super) fn replace_file(temporary: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_file(destination).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, destination).map_err(|error| {
        format!(
            "cannot commit {} to {}: {error}",
            temporary.display(),
            destination.display()
        )
    })
}

pub(super) fn temporary_sibling(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    path.with_file_name(format!(".{name}.ffone-{}", std::process::id()))
}

pub(super) fn remove_stale(root: &Path, allowed: &BTreeSet<PathBuf>) -> Result<usize, String> {
    let mut removed = 0;
    let mut pending = vec![root.to_path_buf()];
    let mut directories = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_symlink() {
                return Err(format!(
                    "package contains unsupported symlink {}",
                    path.display()
                ));
            }
            if kind.is_dir() {
                pending.push(path.clone());
                directories.push(path);
            } else if kind.is_file() {
                let relative = path.strip_prefix(root).map_err(|error| error.to_string())?;
                if !allowed.contains(relative) {
                    fs::remove_file(&path).map_err(|error| error.to_string())?;
                    removed += 1;
                }
            }
        }
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        if fs::read_dir(&directory)
            .map_err(|error| error.to_string())?
            .next()
            .is_none()
        {
            fs::remove_dir(directory).map_err(|error| error.to_string())?;
        }
    }
    Ok(removed)
}

pub(super) fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
