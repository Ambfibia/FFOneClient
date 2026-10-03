use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SerializedCurveOverwriteProof {
    pub rule: SerializedCurveOverwriteRule,
    // Read compatibility for older payloads and the Editor's source validator.
    // Publication lineage is never serialized into new native model data.
    #[serde(default, skip_serializing)]
    pub asset: String,
    #[serde(default, skip_serializing)]
    pub path_id: i64,
    pub clip_name: String,
    pub exact_target_path: bool,
    pub exact_source_array_order: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SerializedCurveOverwriteRule {
    LaterSerializedBindingOverwritesEarlier,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObjectReferenceCurve {
    pub target_node: u32,
    pub property: String,
    pub keys: Vec<ObjectReferenceKey>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectReferenceKey {
    pub time: f64,
    pub value: Option<NativeAssetReference>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingEventObjectParameterInterpretation {
    Missing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SerializedEventObjectParameterInterpretation {
    NullPathId,
    NonNullUnresolved,
    /// The exact non-null pointer is retained as source provenance, while the
    /// legacy event contract consumes another typed argument (currently the
    /// string parameter of `sound` events) and does not expose an object to the
    /// native runtime.
    NonNullMetadataOnly,
    /// The exact legacy pointer is retained as provenance for a callback whose
    /// clean-primary event contract does not consume an object argument.
    NonNullLegacyUnused,
}
