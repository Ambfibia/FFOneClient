use std::collections::{BTreeMap, BTreeSet};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use sha2::{Digest, Sha256};

use crate::{
    AnimationChannel, AnimationCurveRecoveryCanonicalTrack, AnimationCurveRecoveryTrack,
    AnimationEventFloatParameterInterpretation, AnimationEventObjectParameterProvenance,
    AnimationTimeRecovery, DuplicateAnimationKey, DuplicateTrsBinding, DuplicateTrsKeys,
    DuplicateTrsRelation, EmptyTrsBindingKind, EmptyTrsSourceEncoding, ExactQuaternionKey,
    ExactQuaternionKeyPayload, ExactTrsKeyPayload, ExactVec3Key, ExactVec3KeyPayload, FloatCurve,
    Interpolation, MaterialAlphaReference, MaterialAlphaTestState, MaterialBlendFactor,
    MaterialBlendOperation, MaterialCompareFunction, MaterialOutlineState, ModelError,
    NativeAssetReference, NativeModel, PublishedMipPolicy, Result, SamplerMagFilter,
    SamplerMinFilter, SamplerWrapMode, SerializedCurveOverwriteRule,
    SerializedEventObjectParameterInterpretation, TrackValues, has_generated_identity, invalid,
    minimal_windows_png_filename, validate_logical_name, validate_true_asset_name,
    windows_png_filename_preserving_legacy_extension,
};

#[cfg(test)]
mod normal_orientation_tests;

#[cfg(test)]
mod shared_texture_uri_tests;

mod validation_validate_meshes;
mod validation_validate_materials_and_textures;
mod validation_validate_animations;
mod validation_validate_curve_recoveries;
mod collision;
mod textures;
mod operations;
mod codec;
mod models;

pub use validation_validate_meshes::validate;
pub(crate) use validation_validate_meshes::validate_primitive_normal_orientation;
use validation_validate_materials_and_textures::validate_materials_and_textures;
use validation_validate_animations::{
    validate_animations, validate_metadata_target, validate_duplicate_vec3_keys,
    validate_duplicate_quaternion_keys
};
use validation_validate_curve_recoveries::{
    validate_time_recoveries, validate_curve_recoveries, validate_channel, validate_float_curve,
    validate_times, validate_asset_reference
};
pub use collision::triangle_winding_opposes_normals;
use textures::{validate_texture_uri, texture_format_name, texture_mip_byte_length};
use operations::{
    is_sha256, same_optional_duration, same_duration, channel_kind, trs_source_rank,
    duplicate_vec3_keys_match_channel, duplicate_quaternion_keys_match_channel, exact_vec3_tangents,
    exact_quaternion_tangents, exact_tangent_modes, curve_source_field,
    interpolation_from_vec3_keys, interpolation_from_quaternion_keys, native_target_binding_count,
    curve_recovery_keys_last_time, same_optional_f64_bits, track_kind, valid_name, finite,
    normalized_quat
};
use codec::{
    channel_key_payload, ConstantCurvePayload, constant_channel_payload, exact_payload_time
};
use models::model_node_path;
