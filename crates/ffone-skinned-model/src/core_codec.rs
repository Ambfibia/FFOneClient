use super::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ExactTrsKeyPayload {
    Vec3(ExactVec3KeyPayload),
    Quaternion(ExactQuaternionKeyPayload),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExactVec3KeyPayload {
    pub time: f64,
    pub value: [f64; 3],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_mode: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExactQuaternionKeyPayload {
    pub time: f64,
    pub value: [f64; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_mode: Option<i32>,
}
