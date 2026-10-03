use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireNpcDefinition {
    pub(super) schema: String,
    pub(super) id: String,
    pub(super) network_id: u32,
    #[serde(default)]
    pub(super) replaces: Option<String>,
    #[serde(default)]
    pub(super) assets: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) data: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireNanoDefinition {
    pub(super) schema: String,
    pub(super) id: String,
    pub(super) network_id: u32,
    #[serde(default)]
    pub(super) replaces: Option<String>,
    #[serde(default)]
    pub(super) assets: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) data: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireItemDefinition {
    pub(super) schema: String,
    pub(super) id: String,
    pub(super) network_id: ItemNetworkId,
    #[serde(default)]
    pub(super) replaces: Option<String>,
    #[serde(default)]
    pub(super) assets: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) data: BTreeMap<String, JsonValue>,
}
