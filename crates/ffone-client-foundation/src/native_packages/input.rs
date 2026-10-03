use super::*;

/// Discovers package roots only among direct child directories, validates them, and returns a
/// dependency-first registry. Independent ready packages are ordered lexicographically by ID.
pub fn discover_native_packages(
    packages_root: impl AsRef<Path>,
) -> Result<NativePackageRegistry, PackageRegistryError> {
    let packages_root = packages_root.as_ref();
    let candidates = discover_candidates(packages_root)?;
    let order = dependency_order(&candidates)?;
    let dependency_closures = dependency_closures(&candidates, &order);

    let packages = order
        .iter()
        .map(|id| {
            let candidate = &candidates[id];
            LoadedPackage {
                root: candidate.root.clone(),
                manifest: candidate.manifest.clone(),
            }
        })
        .collect::<Vec<_>>();

    let mut pending = Vec::new();
    for package_id in &order {
        pending.extend(load_package_definitions(&candidates[package_id])?);
    }

    build_registry(packages, pending, &dependency_closures)
}

pub(super) fn discover_candidates(
    packages_root: &Path,
) -> Result<BTreeMap<String, PackageCandidate>, PackageRegistryError> {
    let mut candidates = BTreeMap::new();
    for root in sorted_directories(packages_root)? {
        let manifest_path = root.join(PACKAGE_MANIFEST_FILE);
        match fs::symlink_metadata(&manifest_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(PackageRegistryError::SymlinkNotAllowed {
                        path: manifest_path,
                    });
                }
                if !metadata.is_file() {
                    return Err(PackageRegistryError::MissingDefinitionFile {
                        path: manifest_path,
                    });
                }
            }
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(PackageRegistryError::Io {
                    path: manifest_path,
                    source,
                });
            }
        }

        let manifest: NativePackageManifest = read_json(&manifest_path)?;
        validate_schema(&manifest_path, &manifest.schema, PACKAGE_SCHEMA)?;
        validate_identifier(&manifest_path, "package id", &manifest.id)?;
        validate_text(&manifest_path, "version", &manifest.version)?;
        validate_text(&manifest_path, "protocol", &manifest.protocol)?;

        let mut requirement_ids = BTreeSet::new();
        for requirement in &manifest.requires {
            validate_identifier(&manifest_path, "required package id", &requirement.id)?;
            validate_text(&manifest_path, "required version", &requirement.version)?;
            if !requirement_ids.insert(requirement.id.clone()) {
                return Err(PackageRegistryError::DuplicateRequirement {
                    package: manifest.id.clone(),
                    dependency: requirement.id.clone(),
                });
            }
        }

        let id = manifest.id.clone();
        let candidate = PackageCandidate {
            root,
            manifest_path: manifest_path.clone(),
            manifest,
        };
        if let Some(first) = candidates.insert(id.clone(), candidate) {
            return Err(PackageRegistryError::DuplicatePackageId {
                id,
                first: first.manifest_path,
                second: manifest_path,
            });
        }
    }
    Ok(candidates)
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, PackageRegistryError> {
    let bytes = fs::read(path).map_err(|source| PackageRegistryError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| PackageRegistryError::Json {
        path: path.to_path_buf(),
        source,
    })
}

pub(super) fn load_package_definitions(
    package: &PackageCandidate,
) -> Result<Vec<PendingDefinition>, PackageRegistryError> {
    let mut definitions = Vec::new();

    for leaf_root in optional_sorted_directories(&package.root.join("npcs"))? {
        let source_path = checked_definition_file(&leaf_root, "npc.ffdef.json")?;
        let wire: WireNpcDefinition = read_json(&source_path)?;
        validate_schema(&source_path, &wire.schema, NPC_DEFINITION_SCHEMA)?;
        validate_identifier(&source_path, "NPC id", &wire.id)?;
        let assets = validate_assets(&source_path, wire.assets)?;
        let canonical_id = format!("{}:npc/{}", package.manifest.id, wire.id);
        definitions.push(PendingDefinition {
            package_id: package.manifest.id.clone(),
            canonical_id,
            source_path,
            leaf_root,
            value: PendingValue::Npc(NpcDefinition {
                schema: wire.schema,
                id: wire.id,
                network_id: wire.network_id,
                replaces: wire.replaces,
                assets,
                data: wire.data,
            }),
        });
    }

    for leaf_root in optional_sorted_directories(&package.root.join("nanos"))? {
        let source_path = checked_definition_file(&leaf_root, "nano.ffdef.json")?;
        let wire: WireNanoDefinition = read_json(&source_path)?;
        validate_schema(&source_path, &wire.schema, NANO_DEFINITION_SCHEMA)?;
        validate_identifier(&source_path, "Nano id", &wire.id)?;
        let assets = validate_assets(&source_path, wire.assets)?;
        let canonical_id = format!("{}:nano/{}", package.manifest.id, wire.id);
        definitions.push(PendingDefinition {
            package_id: package.manifest.id.clone(),
            canonical_id,
            source_path,
            leaf_root,
            value: PendingValue::Nano(NanoDefinition {
                schema: wire.schema,
                id: wire.id,
                network_id: wire.network_id,
                replaces: wire.replaces,
                assets,
                data: wire.data,
            }),
        });
    }

    for category_root in optional_sorted_directories(&package.root.join("items"))? {
        let category = path_file_name(&category_root)?.to_owned();
        validate_identifier(&category_root, "item category", &category)?;
        for leaf_root in sorted_directories(&category_root)? {
            let source_path = checked_definition_file(&leaf_root, "item.ffdef.json")?;
            let wire: WireItemDefinition = read_json(&source_path)?;
            validate_schema(&source_path, &wire.schema, ITEM_DEFINITION_SCHEMA)?;
            validate_identifier(&source_path, "item id", &wire.id)?;
            let assets = validate_assets(&source_path, wire.assets)?;
            let canonical_id = format!("{}:item/{}/{}", package.manifest.id, category, wire.id);
            definitions.push(PendingDefinition {
                package_id: package.manifest.id.clone(),
                canonical_id,
                source_path,
                leaf_root,
                value: PendingValue::Item(ItemDefinition {
                    schema: wire.schema,
                    category: category.clone(),
                    id: wire.id,
                    network_id: wire.network_id,
                    replaces: wire.replaces,
                    assets,
                    data: wire.data,
                }),
            });
        }
    }

    definitions.sort_by(|left, right| left.source_path.cmp(&right.source_path));
    Ok(definitions)
}
