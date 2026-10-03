//! Immutable evidence that one exact logical-model GLB completed the native
//! Bevy load/material/skin/animation/render path.
//!
//! The contract intentionally does not claim 1:1 visual parity. A screenshot
//! SHA-256 only makes the exact reviewed pixels content-addressable.

use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ModelError, Result};

pub const GPU_EVIDENCE_SCHEMA: &str = "ffone.logical-model-gpu-evidence.v1";
pub const GPU_RENDER_PROFILE: &str = "ffone.logical-model-gpu-preview.v1";
pub const RUNTIME_SMOKE_SCHEMA: &str = "ffone.logical-model-runtime-smoke.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutomatedGpuStatus {
    Passed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VisualParityClaim {
    NotAsserted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceOutlineMode {
    Source,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogicalModelGpuEvidence {
    pub schema: String,
    pub status: AutomatedGpuStatus,
    pub render_profile: String,
    pub visual_parity: VisualParityClaim,
    pub model: GpuModelIdentity,
    pub animation: GpuAnimationEvidence,
    pub runtime: GpuRuntimeEvidence,
    pub screenshot: GpuScreenshotEvidence,
}

/// Immutable proof that Bevy loaded the complete GLTF dependency closure and
/// evaluated an exact named animation through a real scene AnimationPlayer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogicalModelRuntimeSmoke {
    pub schema: String,
    pub status: AutomatedGpuStatus,
    pub render_profile: String,
    pub model: GpuModelIdentity,
    pub gltf_loaded_with_dependencies: bool,
    pub exact_animation_names: Vec<String>,
    pub selected_exact_name: Option<String>,
    pub scene_ready: bool,
    pub animation_players: u64,
    pub animation_graph_handles: u64,
    pub sampled_players: u64,
    pub animation_evaluation_frames: u64,
    pub frames: u64,
    pub material_errors: u64,
    pub shader_errors: u64,
    pub gpu_evidence_relative_json: String,
    pub gpu_evidence_sha256: String,
    pub screenshot_relative_png: String,
    pub screenshot_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GpuModelIdentity {
    /// Candidate-root-relative, slash-normalized GLB path.
    pub relative_glb: String,
    /// Exact sole-root legacy `m_Name`.
    pub true_name: String,
    pub glb_byte_length: u64,
    pub glb_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GpuAnimationEvidence {
    /// Standard glTF clips only. Metadata-only clips remain structural proof.
    pub standard_clips_loaded: u64,
    /// Exact clip `m_Name`; never an overloaded numeric index.
    pub selected_exact_name: Option<String>,
    /// 500,000 means a fixed sample at 50% of the exact clip duration.
    pub sample_normalized_ppm: Option<u32>,
    pub animation_players: u64,
    pub sampled_players: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GpuRuntimeEvidence {
    pub scene_ready: bool,
    pub outline_mode: SourceOutlineMode,
    pub mesh_parts: u64,
    pub skinned_mesh_parts: u64,
    pub skin_joint_references: u64,
    pub resolved_skin_joint_references: u64,
    pub inverse_bind_matrices: u64,
    pub materials_applied: u64,
    pub legacy_pass_companions: u64,
    pub outline_pass_companions: u64,
    pub assigned_texture_bindings: u64,
    pub exact_mip_markers: u64,
    pub exact_mip_chains: u64,
    pub exact_mip_levels: u64,
    pub material_errors: u64,
    pub shader_errors: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GpuScreenshotEvidence {
    /// Evidence-root-relative, slash-normalized PNG path.
    pub relative_png: String,
    pub byte_length: u64,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub foreground_pixels: u64,
}

/// Deterministic runtime expectations independently decoded from the GLB.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalModelGpuFacts {
    pub true_name: String,
    pub standard_animation_names: Vec<String>,
    pub mesh_parts: u64,
    pub skinned_mesh_parts: u64,
    pub skin_joint_references: u64,
    pub inverse_bind_matrices: u64,
    pub materials_applied: u64,
    pub legacy_pass_companions: u64,
    pub outline_pass_companions: u64,
    pub assigned_texture_bindings: u64,
    pub exact_mip_markers: u64,
    pub exact_mip_chains: u64,
    pub exact_mip_levels: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct MaterialFacts {
    companions: u64,
    outline_companions: u64,
    assigned_texture_bindings: u64,
    exact_mip_chains: u64,
    exact_mip_levels: u64,
}

/// Returns deterministic sidecars for one candidate-relative GLB.
pub fn gpu_evidence_relative_paths(relative_glb: &str) -> Result<(PathBuf, PathBuf)> {
    let normalized = relative_glb.replace('\\', "/");
    let path = Path::new(&normalized);
    if normalized.is_empty()
        || path.is_absolute()
        || path.extension().and_then(|value| value.to_str()) != Some("glb")
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return invalid("GPU evidence requires a safe relative lowercase .glb path");
    }
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ModelError::Invalid("GPU evidence GLB has no UTF-8 stem".to_owned()))?;
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    Ok((
        parent.join(format!("{stem}.gpu.json")),
        parent.join(format!("{stem}.gpu.png")),
    ))
}

/// Strictly derives the values that the Bevy scene must instantiate. This is
/// intentionally independent from the evidence JSON being audited.
pub fn gpu_model_facts_from_glb(bytes: &[u8]) -> Result<LogicalModelGpuFacts> {
    let document = parse_glb_document(bytes)?;
    let nodes = required_array(&document, "nodes")?;
    let meshes = required_array(&document, "meshes")?;
    let skins = optional_array(&document, "skins")?;
    let materials = optional_array(&document, "materials")?;
    let accessors = required_array(&document, "accessors")?;

    let true_name = required_string(document.pointer("/extras/logicalModelName"), "logical name")?;
    let scene_index = document.get("scene").and_then(Value::as_u64).unwrap_or(0);
    let scenes = required_array(&document, "scenes")?;
    let scene = index(scenes, scene_index, "selected scene")?;
    let roots = scene
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| ModelError::Invalid("GPU facts scene has no root list".to_owned()))?;
    if roots.len() != 1 {
        return invalid("GPU facts require exactly one logical scene root");
    }
    if required_string(scene.get("name"), "scene name")? != true_name {
        return invalid("GPU facts scene name differs from exact logical name");
    }
    let root = index(nodes, required_u64(&roots[0], "root node")?, "root node")?;
    if required_string(root.get("name"), "root node name")? != true_name {
        return invalid("GPU facts root node name differs from exact logical name");
    }

    let mut animation_names = Vec::new();
    let mut unique_animation_names = BTreeSet::new();
    for animation in optional_array(&document, "animations")? {
        let name = required_string(animation.get("name"), "standard animation name")?.to_owned();
        if !unique_animation_names.insert(name.clone()) {
            return invalid("GPU facts reject duplicate exact standard animation names");
        }
        animation_names.push(name);
    }

    crate::validate_glb_render_contract(bytes)?;
    let mut material_facts = Vec::with_capacity(materials.len());
    for material in materials {
        let ffone = material.pointer("/extras/ffone").ok_or_else(|| {
            ModelError::Invalid("GPU facts material has no extras.ffone".to_owned())
        })?;
        let passes = ffone
            .get("passes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ModelError::Invalid("GPU facts material has no exact passes".to_owned())
            })?;
        if passes.is_empty() {
            return invalid("GPU facts material has an empty exact pass list");
        }
        let mut facts = MaterialFacts {
            companions: u64::try_from(passes.len() - 1)
                .map_err(|_| ModelError::Overflow("GPU companion pass count"))?,
            ..MaterialFacts::default()
        };
        for pass in passes.iter().skip(1) {
            if pass
                .pointer("/outline/mode")
                .and_then(Value::as_str)
                .is_some_and(|mode| mode != "disabled")
            {
                checked_inc(&mut facts.outline_companions, "GPU outline companion count")?;
            }
        }
        let bindings = ffone
            .get("textureBindings")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ModelError::Invalid("GPU facts material has no exact textureBindings".to_owned())
            })?;
        for binding in bindings {
            if binding
                .get("ignoredStaleShaderBinding")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                continue;
            }
            if binding.get("texture").is_none_or(Value::is_null) {
                continue;
            }
            required_u64(
                binding.get("texture").expect("checked non-null texture"),
                "texture binding index",
            )?;
            checked_inc(
                &mut facts.assigned_texture_bindings,
                "GPU assigned texture binding count",
            )?;
            let policy = required_string(
                binding.pointer("/mipProvenance/publishedPolicy"),
                "published mip policy",
            )?;
            let levels = binding
                .get("mipLevels")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ModelError::Invalid("GPU facts assigned texture has no mipLevels".to_owned())
                })?;
            match policy {
                "baseLevelOnly" if levels.len() == 1 => {}
                "exactSourceLevels" if !levels.is_empty() => {
                    checked_inc(&mut facts.exact_mip_chains, "GPU exact mip chain count")?;
                    checked_add(
                        &mut facts.exact_mip_levels,
                        u64::try_from(levels.len())
                            .map_err(|_| ModelError::Overflow("GPU exact mip level count"))?,
                        "GPU exact mip level count",
                    )?;
                }
                _ => return invalid("GPU facts found an incoherent published mip policy"),
            }
        }
        material_facts.push(facts);
    }

    let mut skin_palettes = Vec::with_capacity(skins.len());
    for skin in skins {
        let joints = skin
            .get("joints")
            .and_then(Value::as_array)
            .ok_or_else(|| ModelError::Invalid("GPU facts skin has no joint palette".to_owned()))?;
        if joints.is_empty() {
            return invalid("GPU facts skin has an empty joint palette");
        }
        let accessor_index = required_u64(
            skin.get("inverseBindMatrices").ok_or_else(|| {
                ModelError::Invalid("GPU facts skin has no inverseBindMatrices".to_owned())
            })?,
            "inverse bind accessor",
        )?;
        let accessor = index(accessors, accessor_index, "inverse bind accessor")?;
        let inverse_count = required_u64(
            accessor.get("count").ok_or_else(|| {
                ModelError::Invalid("GPU facts inverse bind accessor has no count".to_owned())
            })?,
            "inverse bind count",
        )?;
        let joint_count = u64::try_from(joints.len())
            .map_err(|_| ModelError::Overflow("GPU skin joint count"))?;
        if inverse_count != joint_count {
            return invalid("GPU facts skin joint/inverse-bind cardinality differs");
        }
        skin_palettes.push((joint_count, inverse_count));
    }

    let mut facts = LogicalModelGpuFacts {
        true_name: true_name.to_owned(),
        standard_animation_names: animation_names,
        mesh_parts: 0,
        skinned_mesh_parts: 0,
        skin_joint_references: 0,
        inverse_bind_matrices: 0,
        materials_applied: 0,
        legacy_pass_companions: 0,
        outline_pass_companions: 0,
        assigned_texture_bindings: 0,
        exact_mip_markers: 0,
        exact_mip_chains: 0,
        exact_mip_levels: 0,
    };

    for node in nodes {
        let Some(mesh_index) = optional_u64(node.get("mesh"), "node mesh")? else {
            continue;
        };
        let mesh = index(meshes, mesh_index, "node mesh")?;
        let primitives = mesh
            .get("primitives")
            .and_then(Value::as_array)
            .ok_or_else(|| ModelError::Invalid("GPU facts mesh has no primitives".to_owned()))?;
        if primitives.is_empty() {
            return invalid("GPU facts mesh has an empty primitive list");
        }
        let primitive_count = u64::try_from(primitives.len())
            .map_err(|_| ModelError::Overflow("GPU mesh part count"))?;
        checked_add(
            &mut facts.mesh_parts,
            primitive_count,
            "GPU mesh part count",
        )?;

        if let Some(skin_index) = optional_u64(node.get("skin"), "node skin")? {
            let (joints, inverse) = *skin_palettes
                .get(usize_index(skin_index, "node skin")?)
                .ok_or_else(|| ModelError::Invalid("GPU facts node skin is invalid".to_owned()))?;
            checked_add(
                &mut facts.skinned_mesh_parts,
                primitive_count,
                "GPU skinned mesh part count",
            )?;
            checked_add(
                &mut facts.skin_joint_references,
                joints
                    .checked_mul(primitive_count)
                    .ok_or(ModelError::Overflow("GPU skin joint references"))?,
                "GPU skin joint references",
            )?;
            checked_add(
                &mut facts.inverse_bind_matrices,
                inverse
                    .checked_mul(primitive_count)
                    .ok_or(ModelError::Overflow("GPU inverse bind matrices"))?,
                "GPU inverse bind matrices",
            )?;
        }

        for primitive in primitives {
            let Some(material_index) = optional_u64(primitive.get("material"), "material index")?
            else {
                continue;
            };
            let material = *material_facts
                .get(usize_index(material_index, "material index")?)
                .ok_or_else(|| {
                    ModelError::Invalid("GPU facts primitive material is invalid".to_owned())
                })?;
            checked_inc(
                &mut facts.materials_applied,
                "GPU material application count",
            )?;
            checked_inc(&mut facts.exact_mip_markers, "GPU exact mip marker count")?;
            checked_add(
                &mut facts.legacy_pass_companions,
                material.companions,
                "GPU companion pass count",
            )?;
            checked_add(
                &mut facts.outline_pass_companions,
                material.outline_companions,
                "GPU outline companion count",
            )?;
            checked_add(
                &mut facts.assigned_texture_bindings,
                material.assigned_texture_bindings,
                "GPU assigned texture binding count",
            )?;
            checked_add(
                &mut facts.exact_mip_chains,
                material.exact_mip_chains,
                "GPU exact mip chain count",
            )?;
            checked_add(
                &mut facts.exact_mip_levels,
                material.exact_mip_levels,
                "GPU exact mip level count",
            )?;
        }
    }
    Ok(facts)
}

fn parse_glb_document(bytes: &[u8]) -> Result<Value> {
    if bytes.len() < 20 || bytes.get(..4) != Some(b"glTF") {
        return invalid("GPU facts cannot parse invalid GLB header");
    }
    if read_u32(bytes, 4)? != 2
        || usize_index(u64::from(read_u32(bytes, 8)?), "GLB byte length")? != bytes.len()
    {
        return invalid("GPU facts require exact GLB 2.0 byte length");
    }
    let json_len = usize_index(u64::from(read_u32(bytes, 12)?), "GLB JSON length")?;
    if read_u32(bytes, 16)? != 0x4e4f_534a {
        return invalid("GPU facts GLB has no leading JSON chunk");
    }
    let json_end = 20usize
        .checked_add(json_len)
        .ok_or(ModelError::Overflow("GPU facts JSON end"))?;
    let bin_header_end = json_end
        .checked_add(8)
        .ok_or(ModelError::Overflow("GPU facts BIN header"))?;
    if bin_header_end > bytes.len() || read_u32(bytes, json_end + 4)? != 0x004e_4942 {
        return invalid("GPU facts GLB has no BIN chunk after JSON");
    }
    let bin_len = usize_index(u64::from(read_u32(bytes, json_end)?), "GLB BIN length")?;
    if bin_header_end
        .checked_add(bin_len)
        .ok_or(ModelError::Overflow("GPU facts BIN end"))?
        != bytes.len()
    {
        return invalid("GPU facts GLB has trailing or truncated BIN bytes");
    }
    serde_json::from_slice(&bytes[20..json_end])
        .map_err(|error| ModelError::Invalid(format!("GPU facts cannot parse GLB JSON: {error}")))
}

fn required_array<'a>(document: &'a Value, name: &str) -> Result<&'a [Value]> {
    document
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| ModelError::Invalid(format!("GPU facts document has no {name} array")))
}

fn optional_array<'a>(document: &'a Value, name: &str) -> Result<&'a [Value]> {
    match document.get(name) {
        None => Ok(&[]),
        Some(value) => value
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| ModelError::Invalid(format!("GPU facts {name} is not an array"))),
    }
}

fn required_string<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ModelError::Invalid(format!("GPU facts has no exact {label}")))
}

fn required_u64(value: &Value, label: &str) -> Result<u64> {
    value
        .as_u64()
        .ok_or_else(|| ModelError::Invalid(format!("GPU facts has invalid {label}")))
}

fn optional_u64(value: Option<&Value>, label: &str) -> Result<Option<u64>> {
    value.map(|value| required_u64(value, label)).transpose()
}

fn index<'a>(values: &'a [Value], index: u64, label: &str) -> Result<&'a Value> {
    values
        .get(usize_index(index, label)?)
        .ok_or_else(|| ModelError::Invalid(format!("GPU facts {label} is out of range")))
}

fn usize_index(value: u64, label: &str) -> Result<usize> {
    usize::try_from(value)
        .map_err(|_| ModelError::Invalid(format!("GPU facts {label} is not addressable")))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| ModelError::Invalid("GPU facts GLB header is truncated".to_owned()))
}

fn checked_inc(target: &mut u64, label: &'static str) -> Result<()> {
    checked_add(target, 1, label)
}

fn checked_add(target: &mut u64, value: u64, label: &'static str) -> Result<()> {
    *target = target
        .checked_add(value)
        .ok_or(ModelError::Overflow(label))?;
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(ModelError::Invalid(message.into()))
}

#[cfg(test)]
mod tests;
