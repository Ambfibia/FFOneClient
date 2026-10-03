use super::*;

pub(super) fn copy_atomic(source: &Path, destination: &Path) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| format!("{} has no parent", destination.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = temporary_sibling(destination);
    fs::copy(source, &temporary).map_err(|error| {
        format!(
            "cannot copy {} to {}: {error}",
            source.display(),
            temporary.display()
        )
    })?;
    replace_file(&temporary, destination)
}

pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = temporary_sibling(path);
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    replace_file(&temporary, path)
}
