use super::*;

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|source| SemanticAssetError::Json {
            path: path.to_owned(),
            source,
        })?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
}

pub(super) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| io_at(parent, source))?;
    }
    let mut file = File::create(path).map_err(|source| io_at(path, source))?;
    file.write_all(bytes)
        .map_err(|source| io_at(path, source))?;
    file.sync_all().map_err(|source| io_at(path, source))
}
