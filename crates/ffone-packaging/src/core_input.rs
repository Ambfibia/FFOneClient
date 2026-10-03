use super::*;

pub(super) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid {}: {error}", path.display()))
}

pub(super) fn parse_hash(value: &str) -> Result<(), String> {
    value
        .parse::<blake3::Hash>()
        .map(|_| ())
        .map_err(|error| format!("invalid BLAKE3 {value:?}: {error}"))
}
