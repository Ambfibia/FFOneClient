use super::*;

pub(super) fn dependency_order(
    candidates: &BTreeMap<String, PackageCandidate>,
) -> Result<Vec<String>, PackageRegistryError> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut dependents = BTreeMap::<String, BTreeSet<String>>::new();

    for (id, candidate) in candidates {
        indegree.insert(id.clone(), candidate.manifest.requires.len());
        for requirement in &candidate.manifest.requires {
            let Some(required) = candidates.get(&requirement.id) else {
                return Err(PackageRegistryError::MissingDependency {
                    package: id.clone(),
                    dependency: requirement.id.clone(),
                });
            };
            if required.manifest.version != requirement.version {
                return Err(PackageRegistryError::DependencyVersionMismatch {
                    package: id.clone(),
                    dependency: requirement.id.clone(),
                    required: requirement.version.clone(),
                    found: required.manifest.version.clone(),
                });
            }
            dependents
                .entry(requirement.id.clone())
                .or_default()
                .insert(id.clone());
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(candidates.len());

    while let Some(id) = ready.pop_first() {
        order.push(id.clone());
        if let Some(children) = dependents.get(&id) {
            for child in children {
                let count = indegree
                    .get_mut(child)
                    .expect("dependent package has an indegree entry");
                *count -= 1;
                if *count == 0 {
                    ready.insert(child.clone());
                }
            }
        }
    }

    if order.len() != candidates.len() {
        let packages = indegree
            .into_iter()
            .filter_map(|(id, count)| (count != 0).then_some(id))
            .collect();
        return Err(PackageRegistryError::DependencyCycle { packages });
    }
    Ok(order)
}

pub(super) fn dependency_closures(
    candidates: &BTreeMap<String, PackageCandidate>,
    order: &[String],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut closures = BTreeMap::<String, BTreeSet<String>>::new();
    for id in order {
        let mut closure = BTreeSet::new();
        for requirement in &candidates[id].manifest.requires {
            closure.insert(requirement.id.clone());
            if let Some(inherited) = closures.get(&requirement.id) {
                closure.extend(inherited.iter().cloned());
            }
        }
        closures.insert(id.clone(), closure);
    }
    closures
}

pub(super) fn sorted_directories(root: &Path) -> Result<Vec<PathBuf>, PackageRegistryError> {
    let entries = fs::read_dir(root).map_err(|source| PackageRegistryError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let mut directories = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| PackageRegistryError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| PackageRegistryError::Io {
                path: path.clone(),
                source,
            })?;
        if file_type.is_symlink() {
            return Err(PackageRegistryError::SymlinkNotAllowed { path });
        }
        if file_type.is_dir() {
            directories.push(path);
        }
    }
    directories.sort();
    Ok(directories)
}

pub(super) fn optional_sorted_directories(root: &Path) -> Result<Vec<PathBuf>, PackageRegistryError> {
    match fs::symlink_metadata(root) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                Err(PackageRegistryError::SymlinkNotAllowed {
                    path: root.to_path_buf(),
                })
            } else if metadata.is_dir() {
                sorted_directories(root)
            } else {
                Err(PackageRegistryError::MissingDefinitionFile {
                    path: root.to_path_buf(),
                })
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(source) => Err(PackageRegistryError::Io {
            path: root.to_path_buf(),
            source,
        }),
    }
}

pub(super) fn checked_definition_file(
    leaf_root: &Path,
    filename: &str,
) -> Result<PathBuf, PackageRegistryError> {
    let path = leaf_root.join(filename);
    let metadata = fs::symlink_metadata(&path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            PackageRegistryError::MissingDefinitionFile { path: path.clone() }
        } else {
            PackageRegistryError::Io {
                path: path.clone(),
                source,
            }
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err(PackageRegistryError::SymlinkNotAllowed { path });
    }
    if !metadata.is_file() {
        return Err(PackageRegistryError::MissingDefinitionFile { path });
    }
    Ok(path)
}
