use super::*;

pub(super) fn collect_files_recursive(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut pending = vec![root.to_owned()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|source| io_at(&directory, source))? {
            let entry = entry.map_err(|source| io_at(&directory, source))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| io_at(&path, source))?;
            if file_type.is_symlink() {
                return Err(SemanticAssetError::Transaction(format!(
                    "symlink is forbidden inside native terrain destination: {}",
                    path.display()
                )));
            }
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

pub(super) fn collect_uris(value: &Value, uris: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "uri" {
                    if let Some(uri) = value.as_str() {
                        uris.push(uri.to_owned());
                    }
                } else {
                    collect_uris(value, uris);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_uris(value, uris);
            }
        }
        _ => {}
    }
}

pub(super) fn read_json(path: &Path) -> Result<Value> {
    let bytes = read_bytes(path)?;
    serde_json::from_slice(&bytes).map_err(|source| SemanticAssetError::Json {
        path: path.to_owned(),
        source,
    })
}

pub(super) fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| io_at(path, source))
}
