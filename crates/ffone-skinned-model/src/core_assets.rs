use super::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeAssetReference {
    pub kind: String,
    pub name: String,
    pub uri: Option<String>,
}
