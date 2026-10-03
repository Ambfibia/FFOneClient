//! Value-level proof that the deterministic GLB writer retained model semantics.
//!
//! This deliberately decodes the emitted GLB from its JSON and BIN chunks. It
//! does not reuse the writer's accessor bookkeeping and does not accept feature
//! counts as evidence that same-sized arrays contain the same values.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Interpolation, ModelError, NativeModel, Result, TrackValues};

#[cfg(test)]
mod tests;

mod constants;
mod types;
mod operations;
mod materials;
mod validation;
mod animation;
mod models;
mod output;
mod input;

pub use constants::SEMANTIC_ROUNDTRIP_PROOF_SCHEMA;
use constants::{COVERED_SCOPES, EXCLUDED_SCOPES};
pub use types::{SemanticDigests, SemanticRoundtripProof};
use types::{CanonicalJsonNumber, SemanticSections, Canonical, Accessors};
pub use operations::prove_semantic_roundtrip;
use operations::{
    vec3_f64, first_json_difference, interpolation_tag, source_encoding_tag, required_array,
    optional_root_array, required_array_value, optional_array, indexed, required_string,
    required_bool, required_u32, optional_u32, value_u32, value_i32, optional_i32, required_f64,
    optional_f64, required_usize, optional_usize, component_size, checked_add, usize_from_u32,
    sha256, invalid
};
pub use materials::validate_glb_render_contract;
pub use validation::validate_retrobution_fusion_eye_contract;
use animation::{
    animation_metadata_mismatch_detail, animation_metadata_from_model,
    animation_metadata_from_glb
};
pub use models::semantic_digests_from_glb;
use models::{
    hierarchy_from_model, geometry_from_model, skins_from_model, animations_from_model,
    materials_from_model, ParsedGlb, hierarchy_from_glb, geometry_from_glb, skins_from_glb,
    animations_from_glb, materials_from_glb
};
use output::{
    write_f64_vectors_as_f32, write_track, write_optional_track, write_canonical_json,
    write_track_rows, write_json_f32_array, write_accessor_f32, write_optional_accessor_f32,
    write_indices, write_optional_joints
};
use input::{read_u32, read_array};
