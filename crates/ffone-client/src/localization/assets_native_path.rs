use super::*;

pub(super) fn native_path(path: &str) -> Result<PathBuf, String> {
    let parsed = Path::new(path);
    if path.is_empty()
        || path.contains('\\')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        || parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe localization path {path:?}"));
    }
    Ok(path.split('/').collect())
}
