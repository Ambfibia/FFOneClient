//! Source-engine-independent representation of one complete logical model.
//!
//! One document owns its hierarchy, render primitives, skin palettes and clips.
//! The schema intentionally cannot express the broken legacy layout where every
//! Unity `Mesh` object was published as an unrelated model file.

mod glb;
mod gpu_evidence;
mod semantic_proof;
mod validate;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use glb::encode_glb;
pub use gpu_evidence::{
    AutomatedGpuStatus, GPU_EVIDENCE_SCHEMA, GPU_RENDER_PROFILE, GpuAnimationEvidence,
    GpuModelIdentity, GpuRuntimeEvidence, GpuScreenshotEvidence, LogicalModelGpuEvidence,
    LogicalModelGpuFacts, LogicalModelRuntimeSmoke, RUNTIME_SMOKE_SCHEMA, SourceOutlineMode,
    VisualParityClaim, gpu_evidence_relative_paths, gpu_model_facts_from_glb,
};
pub use semantic_proof::{
    SEMANTIC_ROUNDTRIP_PROOF_SCHEMA, SemanticDigests, SemanticRoundtripProof,
    prove_semantic_roundtrip, semantic_digests_from_glb, validate_glb_render_contract,
    validate_retrobution_fusion_eye_contract,
};
pub use validate::{triangle_winding_opposes_normals, validate};

mod core_models;
mod core_constants;
mod core_types;
mod core_operations;
mod core_materials;
mod core_textures;
mod core_state;
mod core_animation;
mod core_containers;
mod core_codec;
mod core_assets;
mod core_validation;

pub use core_models::{
    MODEL_SCHEMA, NativeModel, ModelNode, ModelMesh, ModelPrimitive, ModelSkin, ModelError,
    model_relative_path, minimal_windows_glb_filename
};
pub use core_constants::NATIVE_COORDINATE_CONTRACT_SCHEMA;
pub use core_types::{
    NativeCoordinateContract, PublishedPixelTransform, NativeSampler, SamplerMagFilter,
    SamplerMinFilter, Interpolation, TrackValues, EmptyTrsBinding, DuplicateTrsBinding,
    DuplicateTrsRelation, DuplicateTrsKeys, ExactVec3Key, ExactQuaternionKey,
    DuplicateKeyRelation, EmptyTrsBindingKind, EmptyTrsSourceEncoding, FloatCurve, Result
};
pub use core_operations::exact_native_coordinate_contract;
use core_operations::{is_false, minimal_windows_filename};
pub(crate) use core_operations::{has_generated_identity, invalid};
pub use core_materials::{
    NativeMaterial, MaterialColorProperty, MaterialFloatProperty, MaterialPass,
    MaterialBlendState, MaterialBlendFactor, MaterialBlendOperation, MaterialCullMode,
    MaterialCompareFunction, MaterialAlphaTestState, MaterialAlphaReference,
    MaterialOutlineState
};
pub use core_textures::{
    MaterialTextureBinding, DynamicTextureBinding, DynamicTexturePointer,
    ShaderLabTextureDefault, ShaderLabTextureDefaultProperty, MaterialTextureSamplerBinding,
    TextureColorSpace, NativeTexture, TextureMipProvenance, PublishedMipPolicy,
    TextureSourceMipLayout, NativeTextureMipLevel, minimal_windows_png_filename,
    windows_png_filename_preserving_legacy_extension
};
pub use core_state::SamplerWrapMode;
pub use core_animation::{
    AnimationClip, AnimationChannel, AnimationMetadata, AnimationTimeRecovery,
    AnimationTimeRecoveryReason, AnimationTimeRecoveryReference, AnimationTimeRecoveryProof,
    AnimationCurveRecovery, AnimationCurveRecoveryReason, AnimationCurveRecoverySource,
    AnimationCurveRecoveryCanonicalTrack, AnimationCurveRecoveryTrack,
    AnimationCurveRecoveryReference, AnimationCurveRecoveryProof, DuplicateAnimationKey,
    AnimationBindingPointer, AnimationEvent, AnimationEventFloatParameterProvenance,
    AnimationEventFloatParameterInterpretation, AnimationEventObjectParameterProvenance,
    UnsupportedAnimationBinding
};
pub use core_containers::{
    SerializedCurveOverwriteProof, SerializedCurveOverwriteRule, ObjectReferenceCurve,
    ObjectReferenceKey, MissingEventObjectParameterInterpretation,
    SerializedEventObjectParameterInterpretation
};
pub use core_codec::{ExactTrsKeyPayload, ExactVec3KeyPayload, ExactQuaternionKeyPayload};
pub use core_assets::NativeAssetReference;
pub(crate) use core_validation::{validate_logical_name, validate_true_asset_name};
use core_validation::validate_path_segment;
