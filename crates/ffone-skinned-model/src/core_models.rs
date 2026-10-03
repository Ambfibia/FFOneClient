use super::*;

pub const MODEL_SCHEMA: &str = "ffone.skinned-model.v1";

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeModel {
    pub schema: String,
    /// Exact logical root `m_Name`; never a content hash or PathID fallback.
    pub name: String,
    pub native_coordinate_contract: NativeCoordinateContract,
    pub roots: Vec<u32>,
    pub nodes: Vec<ModelNode>,
    pub meshes: Vec<ModelMesh>,
    pub skins: Vec<ModelSkin>,
    /// Exact, ordered legacy materials used by this logical model.
    pub materials: Vec<NativeMaterial>,
    /// External, human-viewable PNG sources used by `materials`.
    pub textures: Vec<NativeTexture>,
    /// Exact legacy sampler state translated to standard glTF sampler state.
    pub samplers: Vec<NativeSampler>,
    pub animations: Vec<AnimationClip>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelNode {
    pub name: String,
    /// Exact Unity `GameObject.m_Name` when duplicate sibling names require a
    /// deterministic native disambiguator in glTF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_name: Option<String>,
    /// One-based serialized sibling occurrence used only with `legacy_name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_sibling_ordinal: Option<u32>,
    pub parent: Option<u32>,
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
    pub mesh: Option<u32>,
    pub skin: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelMesh {
    pub name: String,
    /// Explicit flattened legacy renderer order. This is independent of the
    /// source Mesh object/table iteration order and is consumed together with
    /// each material's exact ShaderLab render queue.
    pub renderer_order: u16,
    pub primitives: Vec<ModelPrimitive>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPrimitive {
    /// Real index into `NativeModel::materials`, emitted as glTF `material`.
    pub material: Option<u32>,
    /// Exact source material-slot `m_Name`, retained as provenance.
    pub material_slot: Option<String>,
    pub positions: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub uvs: Vec<[f64; 2]>,
    /// Palette-local indices matching the owning `ModelSkin::joints` order.
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f64; 4]>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSkin {
    pub name: String,
    pub skeleton_root: u32,
    /// Node indices in the exact order used by primitive JOINTS_0 values.
    pub joints: Vec<u32>,
    /// Authoritative mesh `m_BindPose`, one matrix per renderer `m_Bones` entry.
    pub inverse_bind_matrices: Vec<[[f64; 4]; 4]>,
}

#[derive(Debug, Error, PartialEq)]
pub enum ModelError {
    #[error("invalid native model: {0}")]
    Invalid(String),
    #[error("GLB is too large: {0}")]
    Overflow(&'static str),
}

/// Canonical final layout: one minimally sanitized `<legacy_name>.glb` per true root.
pub fn model_relative_path(
    family: &str,
    semantic_directories: &[&str],
    logical_name: &str,
) -> Result<PathBuf> {
    validate_path_segment("family", family)?;
    let mut path = PathBuf::from("models");
    path.push(family);
    for segment in semantic_directories {
        validate_path_segment("semantic directory", segment)?;
        path.push(segment);
    }
    path.push(minimal_windows_glb_filename(logical_name)?);
    Ok(path)
}

/// Produces the minimally changed Windows-safe filename for an exact legacy name.
///
/// No transliteration, slug, PathID or hash is introduced.
pub fn minimal_windows_glb_filename(logical_name: &str) -> Result<String> {
    validate_logical_name(logical_name)?;
    minimal_windows_filename(logical_name, "glb")
}
