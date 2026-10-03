use super::*;

#[derive(Clone)]
pub(super) struct BundleFile {
    pub(super) document: Value,
    pub(super) entries: BTreeMap<String, String>,
    pub(super) bytes: Vec<u8>,
}

pub(super) fn read_bundle(root: &Path, locale: &str) -> Result<BundleFile, String> {
    let bytes =
        fs::read(root.join(format!("localization/{locale}.json"))).map_err(|e| e.to_string())?;
    let document: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if document["schema"] != "ffone.text-bundle.v1" || document["locale"] != locale {
        return Err(format!("Invalid bundle: {locale}"));
    }
    let entries = serde_json::from_value(document["entries"].clone()).map_err(|e| e.to_string())?;
    Ok(BundleFile {
        document,
        entries,
        bytes,
    })
}
