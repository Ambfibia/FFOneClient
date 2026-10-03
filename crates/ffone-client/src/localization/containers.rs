use super::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TextBundle {
    pub(super) schema: String,
    pub(super) locale: String,
    pub(super) entries: BTreeMap<String, String>,
}
