use super::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationClip {
    pub name: String,
    /// Effective playback duration: max of every present source duration.
    pub duration: f64,
    /// Raw serialized duration when Unity actually stored one.
    pub declared_duration: Option<f64>,
    /// Maximum time of every non-empty TRS key, or `None` for no keyed TRS.
    pub keyed_duration: Option<f64>,
    /// Maximum serialized event time, or `None` for no events.
    pub event_duration: Option<f64>,
    pub sample_rate: Option<f64>,
    pub wrap_mode: Option<i32>,
    pub looped: bool,
    pub channels: Vec<AnimationChannel>,
    pub metadata: AnimationMetadata,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationChannel {
    pub target_node: u32,
    /// Zero-based index in the corresponding serialized Unity curve array.
    pub source_index: u32,
    /// Exact Unity curve-array encoding provenance.
    pub source_encoding: EmptyTrsSourceEncoding,
    /// Number of serialized keys before collapsing identical same-time keys.
    pub source_key_count: u32,
    /// Source-key indices aligned one-to-one with `times` and runtime values.
    pub source_key_indices: Vec<u32>,
    /// Identical same-time source keys which cannot become distinct glTF
    /// sampler inputs. Canonical key identities live on the exact key arrays.
    pub duplicate_keys: Vec<DuplicateAnimationKey>,
    pub interpolation: Interpolation,
    pub times: Vec<f64>,
    pub values: TrackValues,
    /// Required for CUBICSPLINE; derivative values are not normalized.
    pub in_tangents: Option<TrackValues>,
    pub out_tangents: Option<TrackValues>,
    /// Exact legacy Keyframe tangentMode values. Empty means the source had none.
    pub tangent_modes: Vec<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationMetadata {
    pub float_curves: Vec<FloatCurve>,
    pub object_curves: Vec<ObjectReferenceCurve>,
    pub events: Vec<AnimationEvent>,
    /// Exact Unity TRS bindings whose serialized `m_Curve` contains no keys.
    ///
    /// Empty curves cannot become glTF animation channels, because a glTF
    /// sampler must contain at least one input/output value.  They remain
    /// first-class, digest-bound source metadata instead of being discarded.
    pub empty_trs_bindings: Vec<EmptyTrsBinding>,
    /// Serialized Unity TRS curves which are identical to one canonical
    /// runtime channel. glTF permits only one channel per node/property pair,
    /// so every duplicate identity and complete key sequence is retained here.
    pub duplicate_trs_bindings: Vec<DuplicateTrsBinding>,
    /// Explicit provenance for source curves whose serialized key times were
    /// non-strict and were recovered from one exact sibling curve.
    pub time_recoveries: Vec<AnimationTimeRecovery>,
    /// Explicit provenance for conflicting duplicate constant TRS bindings
    /// whose one canonical source was proven by exactly one sibling curve.
    /// The rejected serialized bindings remain complete and digest-bound.
    pub curve_recoveries: Vec<AnimationCurveRecovery>,
    /// Extraction records every binding that could not be represented exactly.
    /// Validation deliberately rejects any non-empty list.
    pub unsupported: Vec<UnsupportedAnimationBinding>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTimeRecovery {
    pub kind: EmptyTrsBindingKind,
    pub target_node: u32,
    /// Exact source Unity animation path, before hierarchy suffix resolution.
    pub target_path: String,
    pub source_encoding: EmptyTrsSourceEncoding,
    pub source_index: u32,
    pub reason: AnimationTimeRecoveryReason,
    pub original_times: Vec<f64>,
    pub recovered_times: Vec<f64>,
    pub source_sample_rate: Option<f64>,
    pub reference: AnimationTimeRecoveryReference,
    pub proof: AnimationTimeRecoveryProof,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationTimeRecoveryReason {
    NonStrictSourceTimesRecoveredFromExactSiblingCurve,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTimeRecoveryReference {
    pub asset: String,
    pub path_id: i64,
    pub clip_name: String,
    pub kind: EmptyTrsBindingKind,
    pub path: String,
    pub source_encoding: EmptyTrsSourceEncoding,
    pub source_index: u32,
    pub sample_rate: Option<f64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTimeRecoveryProof {
    pub exact_key_payload_excluding_time: bool,
    pub exact_path: bool,
    pub exact_sample_rate: bool,
    pub unique_recovered_time_vector_count: u64,
    pub matching_reference_count: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveRecovery {
    pub kind: EmptyTrsBindingKind,
    pub target_node: u32,
    /// Exact Unity animation path, before hierarchy suffix resolution.
    pub target_path: String,
    pub source_encoding: EmptyTrsSourceEncoding,
    pub reason: AnimationCurveRecoveryReason,
    pub source_sample_rate: Option<f64>,
    pub source: AnimationCurveRecoverySource,
    pub canonical: AnimationCurveRecoveryCanonicalTrack,
    pub rejected: Vec<AnimationCurveRecoveryTrack>,
    pub reference: AnimationCurveRecoveryReference,
    pub proof: AnimationCurveRecoveryProof,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationCurveRecoveryReason {
    ConflictingDuplicateConstantCurveResolvedFromExactSiblingCurve,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveRecoverySource {
    pub field: String,
    pub source_encoding: EmptyTrsSourceEncoding,
    pub source_indices: Vec<u32>,
    pub source_target_curve_count: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveRecoveryCanonicalTrack {
    pub path: String,
    pub interpolation: Interpolation,
    pub keys: DuplicateTrsKeys,
    pub duplicate_keys: Vec<DuplicateAnimationKey>,
    pub source_key_count: u32,
    pub source_index: u32,
    pub source_encoding: EmptyTrsSourceEncoding,
    /// Index in the canonical track array for this recovery kind.
    pub track_index: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveRecoveryTrack {
    pub path: String,
    pub interpolation: Interpolation,
    pub keys: DuplicateTrsKeys,
    pub duplicate_keys: Vec<DuplicateAnimationKey>,
    pub source_key_count: u32,
    pub source_index: u32,
    pub source_encoding: EmptyTrsSourceEncoding,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveRecoveryReference {
    // Accepted while reading older payloads; source object identity stays in
    // the Editor's evidence, not in newly serialized native model data.
    #[serde(default, skip_serializing)]
    pub asset: String,
    #[serde(default, skip_serializing)]
    pub path_id: i64,
    pub clip_name: String,
    pub field: String,
    pub kind: EmptyTrsBindingKind,
    pub path: String,
    pub source_encoding: EmptyTrsSourceEncoding,
    pub source_index: u32,
    pub sample_rate: Option<f64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationCurveRecoveryProof {
    pub exact_path: bool,
    pub exact_sample_rate: bool,
    pub all_source_curves_constant: bool,
    pub canonical_matches_reference: bool,
    pub unique_canonical_candidate_count: u64,
    pub matching_reference_count: u64,
    pub source_target_curve_count: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateAnimationKey {
    pub source_key_index: u32,
    pub canonical_key_index: u32,
    pub relation: DuplicateKeyRelation,
    pub key: ExactTrsKeyPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationBindingPointer {
    pub file_id: i32,
    pub path_id: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEvent {
    pub time: f64,
    pub function_name: String,
    pub string_parameter: String,
    pub float_parameter: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub float_parameter_provenance: Option<AnimationEventFloatParameterProvenance>,
    pub int_parameter: i32,
    pub object_parameter: Option<NativeAssetReference>,
    /// Exact serialized presence/identity of Unity's objectReferenceParameter.
    /// A stale non-zero fileId with pathId zero is still a typed null pointer.
    pub object_parameter_provenance: AnimationEventObjectParameterProvenance,
    pub message_options: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationEventFloatParameterProvenance {
    pub source_asset_format: u32,
    pub raw_float32_bits: String,
    pub interpretation: AnimationEventFloatParameterInterpretation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationEventFloatParameterInterpretation {
    LegacyUnusedNonfiniteNormalizedToZero,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "presence", rename_all = "kebab-case", deny_unknown_fields)]
pub enum AnimationEventObjectParameterProvenance {
    Missing {
        interpretation: MissingEventObjectParameterInterpretation,
    },
    SerializedPointer {
        #[serde(rename = "sourceAssetIndex")]
        source_asset_index: u32,
        #[serde(rename = "fileId")]
        file_id: i64,
        #[serde(rename = "pathId")]
        path_id: i64,
        interpretation: SerializedEventObjectParameterInterpretation,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnsupportedAnimationBinding {
    pub kind: String,
    pub target_path: String,
    pub property: String,
    pub reason: String,
}
