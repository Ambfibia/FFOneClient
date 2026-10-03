use std::path::PathBuf;

use ffone_skinned_model::{
    AnimationBindingPointer, AnimationChannel, AnimationClip, AnimationEvent, AnimationMetadata,
    EmptyTrsSourceEncoding, FloatCurve, Interpolation, MODEL_SCHEMA, MaterialAlphaReference,
    MaterialAlphaTestState, MaterialBlendFactor, MaterialBlendOperation, MaterialBlendState,
    MaterialColorProperty, MaterialCompareFunction, MaterialCullMode, MaterialFloatProperty,
    MaterialOutlineState, MaterialPass, MaterialTextureBinding, MaterialTextureSamplerBinding,
    ModelMesh, ModelNode, ModelPrimitive, ModelSkin, NativeAssetReference, NativeMaterial,
    NativeModel, NativeSampler, NativeTexture, NativeTextureMipLevel, ObjectReferenceCurve,
    ObjectReferenceKey, PublishedMipPolicy, PublishedPixelTransform, SamplerMagFilter,
    SamplerMinFilter, SamplerWrapMode, ShaderLabTextureDefault, ShaderLabTextureDefaultProperty,
    TextureColorSpace, TextureMipProvenance, TextureSourceMipLayout, TrackValues,
    UnsupportedAnimationBinding, encode_glb, exact_native_coordinate_contract,
    minimal_windows_png_filename, model_relative_path, validate,
};
use serde_json::{Value, json};

mod operations;
mod models;
mod animation;
mod assets;
mod output;
mod codec;
mod textures;
mod materials;

use operations::{identity, rex, materialized_rex, accessor_f32};
use models::glb_json;
