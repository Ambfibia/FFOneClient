use super::*;

pub(super) fn collect_relative_files(
    asset_root: &std::path::Path,
    relative_root: &str,
) -> TutorialMissionContentResult<BTreeSet<String>> {
    let root = asset_root.join(relative_root);
    let mut files = BTreeSet::new();
    let mut pending = vec![root.clone()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| TutorialMissionContentError::Io {
            path: directory.display().to_string(),
            detail: error.to_string(),
        })? {
            let entry = entry.map_err(|error| TutorialMissionContentError::Io {
                path: directory.display().to_string(),
                detail: error.to_string(),
            })?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| TutorialMissionContentError::Io {
                    path: path.display().to_string(),
                    detail: error.to_string(),
                })?;
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() {
                let relative = path
                    .strip_prefix(asset_root)
                    .map_err(|error| invalid(error.to_string()))?
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(relative);
            }
        }
    }
    Ok(files)
}
