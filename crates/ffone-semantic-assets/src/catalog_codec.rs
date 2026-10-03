use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTerrainPublicationPayload {
    pub path: String,
    pub blake3: String,
    pub kind: AssetKind,
    pub role: String,
}
