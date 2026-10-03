use super::*;

#[derive(Clone, Debug)]
pub struct AssetValidationOptions {
    pub asset_root: PathBuf,
    /// Hash every immutable payload and build the complete ownership graph.
    /// Editable audio/localization files remain path-validated but unhashed.
    /// Development checks keep this false and validate only domain roots and direct edges.
    pub full: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetValidationReport {
    pub domains: usize,
    pub references: usize,
    pub groups: usize,
    pub files: usize,
    pub payload_bytes: u64,
    pub root_blake3: Option<String>,
}

/// Validate the source-of-truth domain catalogs and, for release builds, the
/// complete physical dependency/ownership graph. No metadata is generated in
/// the asset tree.
pub fn validate_assets(options: &AssetValidationOptions) -> Result<AssetValidationReport, String> {
    if !options.full {
        let domains = validate_domain_roots(&options.asset_root, false)?;
        return Ok(AssetValidationReport {
            domains: domains.catalogs,
            references: domains.references,
            groups: 0,
            files: 0,
            payload_bytes: 0,
            root_blake3: None,
        });
    }
    let graph = build_asset_graph(&options.asset_root, true)?;
    Ok(AssetValidationReport {
        domains: graph.domain_catalogs,
        references: graph.direct_references,
        groups: graph.groups.len(),
        files: graph.files.len(),
        payload_bytes: graph.payload_bytes,
        root_blake3: Some(graph.root_blake3),
    })
}

pub(super) fn validate_manifest(manifest: &ProjectManifest) -> Result<(), String> {
    if manifest.schema != "ffone.project-assets.v1" {
        return Err(format!("unsupported asset schema {:?}", manifest.schema));
    }
    validate_file_entries(&manifest.files, "asset manifest")
}

pub(super) fn validate_file_entries(files: &[ProjectFile], description: &str) -> Result<(), String> {
    let mut folded = BTreeSet::new();
    for file in files {
        validate_relative(&file.path)?;
        if !folded.insert(file.path.to_lowercase()) {
            return Err(format!(
                "case-insensitive duplicate path {:?} in {description}",
                file.path
            ));
        }
        parse_hash(&file.blake3)?;
    }
    Ok(())
}

pub(super) fn validate_relative(value: &str) -> Result<PathBuf, String> {
    if value.is_empty()
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err(format!("invalid relative asset path {value:?}"));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe relative asset path {value:?}"));
    }
    Ok(native_path(value))
}

pub(super) fn validate_regular_directory(path: &Path, description: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {description} {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(format!(
            "{description} {} is not a regular directory",
            path.display()
        ));
    }
    Ok(())
}
