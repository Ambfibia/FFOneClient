use super::*;

pub(super) fn read_json<T: serde::de::DeserializeOwned>(
    root: &Path,
    relative: &str,
) -> Result<T, TutorialEffectLibraryError> {
    let path = safe_asset_path(root, relative)?;
    reject_symlink(&path, relative)?;
    let bytes = fs::read(&path).map_err(|error| {
        TutorialEffectLibraryError(format!("could not read {}: {error}", path.display()))
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        TutorialEffectLibraryError(format!("invalid JSON at {}: {error}", path.display()))
    })
}

pub(super) fn read_verified(
    root: &Path,
    relative: &str,
    _expected_bytes: u64,
    _expected_blake3: &str,
) -> Result<Vec<u8>, TutorialEffectLibraryError> {
    let path = safe_asset_path(root, relative)?;
    reject_symlink(&path, relative)?;
    let bytes = fs::read(&path).map_err(|error| {
        TutorialEffectLibraryError(format!("could not read {}: {error}", path.display()))
    })?;
    Ok(bytes)
}
