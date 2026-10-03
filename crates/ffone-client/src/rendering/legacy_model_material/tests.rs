use super::apply::{
    apply_cached_legacy_static_material, apply_static_world_base_color_fallback,
    image_is_fully_opaque, renderer_index_from_mesh_extras,
};
use super::exact_mip_chain::{
    assemble_exact_mip_image, exact_interpretation_digest, sha256_hex, validate_loaded_image,
    validate_loaded_mip_level,
};
use super::expected_passes::{
    canonical_blend, canonical_expected_passes, refine_exact_shader_kind,
};
use super::metadata::PendingLegacyWaterMaterial;
use super::pipeline_state::{bevy_blend_state, bevy_color_write_mask, bevy_cull_mode};
use super::replacement_textures::{
    validate_character_runtime_texture_contract, validate_runtime_replacement_contract,
};
use super::samplers::{
    LEGACY_TEXTURE_ANISOTROPY, apply_native_anisotropy, exact_sampler_descriptor,
    validate_sampler_descriptor,
};
use super::sort_order::{
    LEGACY_MATERIAL_PASS_STRIDE, legacy_material_sort_bias, legacy_static_world_sort_bias,
};
use super::static_shader_plan::legacy_static_shader_plan;
use super::sync::{
    legacy_outline_visibility, sync_legacy_sky_rimlight_assets, try_insert_streamed_entity,
};
use super::texture_resolve::apply_unassigned_shader_texture_defaults;
use super::validation::{is_safe_relative_png_uri, validate_runtime_texture_binding};
use crate::legacy_model_material::*;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::camera::visibility::VisibilityRange;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImageAddressMode;
use bevy::image::ImageFilterMode;
use bevy::image::ImageSampler;
use bevy::image::ImageSamplerDescriptor;
use bevy::mesh::MeshTag;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::render::render_resource::BlendComponent;
use bevy::render::render_resource::BlendFactor;
use bevy::render::render_resource::BlendOperation;
use bevy::render::render_resource::BlendState;
use bevy::render::render_resource::ColorWrites;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use ffone_runtime_contracts::CharacterRuntimeTextureContract;
use ffone_skinned_model::MaterialBlendFactor;
use ffone_skinned_model::MaterialTextureBinding;
use ffone_skinned_model::NativeSampler;
use ffone_skinned_model::SamplerMagFilter;
use ffone_skinned_model::SamplerMinFilter;
use ffone_skinned_model::SamplerWrapMode;
use ffone_skinned_model::ShaderLabTextureDefault;
use ffone_skinned_model::ShaderLabTextureDefaultProperty;
use ffone_skinned_model::TextureColorSpace;
use serde_json::Value;
use std::sync::atomic::Ordering;

mod types;
mod materials_exact_shader_names_classify_without_fuzzy_su;
mod materials_exact_equipment_blocker_material_provenance;
mod systems;
mod operations_production_epbarrier_static_adapter_preserve;
mod operations_anisotropic_filtering_only_raises_the_clamp;
mod models;
mod native_nano_outlines;
mod state;
mod assets;
mod textures;
mod validation_exact_source_mips_validate_hashe;
mod npc_texture_interpretation;

use types::StreamingSafeInsertMarker;
use operations_production_epbarrier_static_adapter_preserve::{
    production_static_world_pending, test_assigned_binding, test_null_binding
};
use models::production_model_pending;
use state::runtime_binding_from_typed;
use textures::test_runtime_texture_contract;
