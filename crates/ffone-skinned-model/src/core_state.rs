use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplerWrapMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}
