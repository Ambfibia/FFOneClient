use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeCoordinateContract {
    pub schema: String,
    pub published_space: String,
    pub position: String,
    pub normal: String,
    pub translation: String,
    pub rotation: String,
    pub scale: String,
    pub uv: String,
    pub inverse_bind_matrix: String,
    pub source_triangle_winding: String,
    pub published_triangle_winding: String,
    pub winding_conversion_owner: String,
    pub unit_scale: String,
    pub origin_policy: String,
    pub auto_centered: bool,
    pub auto_scaled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum PublishedPixelTransform {
    #[serde(rename = "vertical-flip-only-for-png-top-left-origin")]
    VerticalFlipOnlyForPngTopLeftOrigin,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeSampler {
    pub name: String,
    pub mag_filter: SamplerMagFilter,
    pub min_filter: SamplerMinFilter,
    pub wrap_s: SamplerWrapMode,
    pub wrap_t: SamplerWrapMode,
    /// Exact raw legacy Texture2D sampler provenance.
    pub legacy_filter_mode: i32,
    pub legacy_wrap_mode: i32,
    pub anisotropy_level: u32,
    pub mip_map_bias: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplerMagFilter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplerMinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Interpolation {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "path", content = "values", rename_all = "lowercase")]
pub enum TrackValues {
    Translation(Vec<[f64; 3]>),
    Rotation(Vec<[f64; 4]>),
    Scale(Vec<[f64; 3]>),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmptyTrsBinding {
    pub kind: EmptyTrsBindingKind,
    pub target_node: u32,
    /// Exact source Unity animation path, before hierarchy suffix resolution.
    pub target_path: String,
    /// Zero-based index in the corresponding serialized Unity curve array.
    pub source_index: u32,
    pub source_encoding: EmptyTrsSourceEncoding,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateTrsBinding {
    pub kind: EmptyTrsBindingKind,
    pub target_node: u32,
    /// Exact source Unity animation path, before hierarchy suffix resolution.
    pub target_path: String,
    pub source_index: u32,
    pub source_encoding: EmptyTrsSourceEncoding,
    pub relation: DuplicateTrsRelation,
    /// Index in the canonical track array for this `kind`, not the combined
    /// glTF channel array.
    pub canonical_track_index: u32,
    pub canonical_source_index: u32,
    pub canonical_source_encoding: EmptyTrsSourceEncoding,
    pub source_key_count: u32,
    pub duplicate_keys: Vec<DuplicateAnimationKey>,
    pub keys: DuplicateTrsKeys,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_proof: Option<SerializedCurveOverwriteProof>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateTrsRelation {
    Identical,
    SerializedLastWriteWins,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DuplicateTrsKeys {
    Vec3(Vec<ExactVec3Key>),
    Quaternion(Vec<ExactQuaternionKey>),
}

impl DuplicateTrsKeys {
    pub fn len(&self) -> usize {
        match self {
            Self::Vec3(keys) => keys.len(),
            Self::Quaternion(keys) => keys.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExactVec3Key {
    pub source_key_index: u32,
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
pub struct ExactQuaternionKey {
    pub source_key_index: u32,
    pub time: f64,
    pub value: [f64; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_mode: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DuplicateKeyRelation {
    IdenticalSameTime,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EmptyTrsBindingKind {
    Translation,
    Rotation,
    Scale,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EmptyTrsSourceEncoding {
    Plain,
    Compressed,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FloatCurve {
    pub target_node: u32,
    /// Exact Unity animation path before hierarchy suffix resolution.
    pub target_path: String,
    /// Zero-based index in the serialized `m_FloatCurves` array.
    pub source_index: u32,
    /// Number of serialized keys. Float curves with duplicate times are
    /// rejected until they have typed duplicate-key provenance.
    pub source_key_count: u32,
    /// Source-key indices aligned one-to-one with `times` and `values`.
    pub source_key_indices: Vec<u32>,
    pub property: String,
    /// Unity component class bound by the serialized EditorCurveBinding.
    pub class_id: i32,
    /// Exact serialized script PPtr. Built-in component bindings use a typed
    /// null pointer (`path_id == 0`).
    pub script: AnimationBindingPointer,
    /// Exact Unity AnimationCurve infinity modes.
    pub pre_infinity: i32,
    pub post_infinity: i32,
    pub interpolation: Interpolation,
    pub times: Vec<f64>,
    pub values: Vec<f64>,
    pub in_tangents: Option<Vec<f64>>,
    pub out_tangents: Option<Vec<f64>>,
    pub tangent_modes: Vec<i32>,
}

pub type Result<T> = std::result::Result<T, ModelError>;
