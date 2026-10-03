use super::*;

#[derive(Debug, Deserialize)]
pub(super) struct TextBundle {
    pub(super) schema: String,
    pub(super) locale: String,
    pub(super) entries: BTreeMap<String, String>,
}

pub(super) fn open_bundle(path: &Path, expected_locale: &str) -> TextBundle {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let bundle: TextBundle = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    assert_eq!(bundle.schema, "ffone.text-bundle.v1", "{}", path.display());
    assert_eq!(bundle.locale, expected_locale, "{}", path.display());
    bundle
}
