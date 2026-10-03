//! Native Bevy rendering for FusionFall's legacy model materials.
//!
//! Toon outlines and transparent cutouts are genuinely multi-pass. A single
//! Bevy `Material` cannot draw a skinned mesh twice, so the API exposes the
//! source pass plan and companion-entity markers instead of pretending that a
//! one-pass `StandardMaterial` is equivalent.


use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, embedded_asset, embedded_path, io::Reader},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

mod static_exact_mips;
use static_exact_mips::LegacyStaticExactMipManifest;
mod gpu_uniforms;
mod shared_assets;
pub(crate) use gpu_uniforms::NativeMaterialUniformUpdates;
pub(crate) use shared_assets::ModelMaterialSharing;
pub use shared_assets::make_legacy_material_unique;
#[cfg(test)]
mod mip_cache_tests;
mod static_world;
use shared_assets::maintain_shared_static_assets;
pub use shared_assets::{
    StaticAssetSharing, StaticAssetSharingStatistics, static_asset_sharing_statistics,
};
use static_world::apply_legacy_static_world_materials;

mod apply;
mod exact_mip_cache;
mod exact_mip_chain;
mod expected_passes;
mod extras_parse;
mod material_uniforms;
mod metadata;
mod params;
mod pipeline_state;
mod render_plan;
mod replacement_textures;
mod samplers;
mod shader_kind;
mod sort_order;
mod static_material_cache;
mod static_shader_plan;
mod sync;
mod texture_resolve;
mod validation;
pub use apply::LegacyMaterialApplied;
use apply::{
    apply_cached_legacy_static_material, apply_legacy_material_renderer_order,
    apply_legacy_model_materials, apply_legacy_water_materials,
    apply_static_world_base_color_fallback, discover_legacy_material_extras,
    discover_legacy_material_renderer_order, image_is_fully_opaque,
};
pub use exact_mip_cache::ExactMipChainApplied;
use exact_mip_cache::ExactMipChainCache;
use exact_mip_chain::resolve_exact_mip_chain;
pub use expected_passes::LegacyMaterialMetadataError;
use extras_parse::LegacyMaterialExtrasParseCache;
use material_uniforms::{LEGACY_WATER_RENDER_MODE, LegacyWaterMaterialUniform};
pub use material_uniforms::{
    LegacyModelMaterialUniform, LegacyModelPipelineKey, LegacyOutlineCompanion,
    LegacyOutlinePipelineKey, LegacyOutlineSource, LegacyOutlineUniform,
};
pub use metadata::PendingLegacyModelMaterial;
pub(crate) use metadata::PendingLegacyStaticWorldMaterial;
pub use params::{LegacyGltfTextureBinding, LegacyModelMaterialParams, LegacyModelTextures};
use pipeline_state::apply_render_mode;
pub use render_plan::{
    LegacyAlphaTest, LegacyBlendMode, LegacyColorWriteMask, LegacyCullMode, LegacyModelRenderPlan,
    LegacyPassKind, LegacyPassPlan, LegacyRenderMode,
};
pub use replacement_textures::{
    LegacyNpcTextureRole, legacy_npc_texture_role, load_character_runtime_texture_with_contract,
    load_legacy_main_texture_replacement, load_legacy_main_texture_replacement_with_contract,
    load_legacy_main_texture_replacement_with_sampler, native_npc_table_texture_writable,
};
use samplers::exact_sampler_descriptor;
pub use samplers::{LEGACY_TEXTURE_ANISOTROPY_DISABLED, LEGACY_TEXTURE_ANISOTROPY_ENABLED};
pub use shader_kind::{LegacyMaterialExtrasError, LegacyShaderKind, UnknownLegacyShaderName};
pub(crate) use sort_order::legacy_render_queue_sort_bias;
use sort_order::legacy_static_world_sort_bias;
pub use sort_order::{
    LegacyMaterialPassCompanion, LegacyMaterialRendererOrder, LegacyMaterialSortOrderApplied,
};
pub(crate) use static_material_cache::LegacyStaticMaterialCache;
use static_material_cache::{CachedLegacyStaticMaterial, LegacyStaticMaterialCacheKey};
use sync::{
    disable_static_frustum_culling_for_skinned_meshes, sync_legacy_outline_visibility,
    sync_legacy_sky_rimlight, sync_legacy_texture_anisotropy,
};
use validation::{
    exact_f32, exact_vec2_f32, is_safe_relative_png_uri, validate_exact_mip_metadata,
};

// Retrobution `mainData` path 28 is the single white directional light.
// Unity stores the light's forward ray direction; native world space reflects
// Unity X, so this is the exact ray direction consumed by the legacy shaders.
const LEGACY_MAIN_LIGHT_RAY_DIRECTION: Vec3 =
    Vec3::new(-0.893_700_85, -0.309_017_06, -0.325_280_46);

#[derive(Asset, TypePath, Debug, Clone)]
#[type_path = "ffone_client::legacy_model_material"]
struct ExactMipPngBytes {
    bytes: Vec<u8>,
}

#[derive(Default, TypePath)]
#[type_path = "ffone_client::legacy_model_material"]
struct ExactMipPngBytesLoader;

impl AssetLoader for ExactMipPngBytesLoader {
    type Asset = ExactMipPngBytes;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ExactMipPngBytes { bytes })
    }
}

#[derive(Asset, TypePath, Debug, Clone, PartialEq)]
#[type_path = "ffone_client::legacy_model_material"]
pub struct LegacyModelMaterial {
    uniform_owner: gpu_uniforms::UniformOwner,
    sharing_state: shared_assets::MaterialSharingState,
    pub uniform: LegacyModelMaterialUniform,
    pub base_texture: Option<Handle<Image>>,
    pub toon_ramp: Option<Handle<Image>>,
    pub bump_texture: Option<Handle<Image>>,
    pub effect_map: Option<Handle<Image>>,
    pub render_mode: LegacyRenderMode,
    /// Enables the dedicated vertex-shader variant for source UV controllers.
    pub gpu_uv_animation: bool,
    /// Instance-local transparent-phase order. This preserves the authored
    /// Legacy ShaderLab queue/renderer/pass sequence without moving geometry.
    pub sort_bias: f32,
}

impl LegacyModelMaterial {
    pub(crate) fn is_shared_immutable(&self) -> bool {
        matches!(
            self.sharing_state,
            shared_assets::MaterialSharingState::Shared
        )
    }

    pub(crate) fn is_instance_private(&self) -> bool {
        matches!(
            self.sharing_state,
            shared_assets::MaterialSharingState::Private
        )
    }
}

impl Material for LegacyModelMaterial {
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn vertex_shader() -> ShaderRef {
        embedded_shader_ref("legacy_model_base.wgsl")
    }

    fn fragment_shader() -> ShaderRef {
        embedded_shader_ref("legacy_model_base.wgsl")
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.render_mode.blend {
            LegacyBlendMode::Replace if self.render_mode.alpha_cutout => {
                AlphaMode::Mask(self.uniform.alpha_effect.x)
            }
            LegacyBlendMode::Replace => AlphaMode::Opaque,
            LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
            | LegacyBlendMode::SrcAlphaOneMinusSrcColor
            | LegacyBlendMode::SrcColorOneMinusSrcAlpha
            | LegacyBlendMode::SrcAlphaZero
            | LegacyBlendMode::SrcAlphaOne
            | LegacyBlendMode::OneMinusSrcAlphaOne
            | LegacyBlendMode::OneMinusDstColorOne
            | LegacyBlendMode::OneOne => AlphaMode::Blend,
        }
    }

    fn depth_bias(&self) -> f32 {
        self.sort_bias
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        apply_render_mode(descriptor, key.bind_group_data.render_mode);
        if key.bind_group_data.gpu_uv_animation {
            descriptor
                .vertex
                .shader_defs
                .push("LEGACY_GPU_UV_ANIMATION".into());
        }
        Ok(())
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[type_path = "ffone_client::legacy_model_material"]
struct LegacyWaterMaterial {
    #[uniform(0)]
    uniform: LegacyWaterMaterialUniform,
    /// Exact `_ReflectiveColor` one-dimensional gradient.
    #[texture(1)]
    #[sampler(2)]
    reflective_gradient: Option<Handle<Image>>,
    /// Exact scrolling `_BumpMap`.
    #[texture(3)]
    #[sampler(4)]
    bump_map: Option<Handle<Image>>,
    /// Exact `_Fresnel` map retained for the source contract and lower shader
    /// tiers even though the audited reflective pass derives Fresnel from
    /// view direction and the bump sum.
    #[texture(5)]
    #[sampler(6)]
    fresnel_map: Option<Handle<Image>>,
}

impl Material for LegacyWaterMaterial {
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn vertex_shader() -> ShaderRef {
        embedded_shader_ref("legacy_water.wgsl")
    }

    fn fragment_shader() -> ShaderRef {
        embedded_shader_ref("legacy_water.wgsl")
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        apply_render_mode(descriptor, LEGACY_WATER_RENDER_MODE);
        Ok(())
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(LegacyOutlinePipelineKey)]
#[type_path = "ffone_client::legacy_model_material"]
pub struct LegacyOutlineMaterial {
    #[uniform(0)]
    pub uniform: LegacyOutlineUniform,
    pub render_mode: LegacyRenderMode,
    /// Instance-local transparent-phase order, paired with the source
    /// renderer's exact flattened legacy index.
    pub sort_bias: f32,
}

impl Material for LegacyOutlineMaterial {
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn vertex_shader() -> ShaderRef {
        embedded_shader_ref("legacy_model_outline.wgsl")
    }

    fn fragment_shader() -> ShaderRef {
        embedded_shader_ref("legacy_model_outline.wgsl")
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        self.sort_bias
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        apply_render_mode(descriptor, key.bind_group_data.render_mode);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LegacyModelMaterialPlugin;

impl Plugin for LegacyModelMaterialPlugin {
    fn build(&self, app: &mut App) {
        crate::native_gltf::install(app);
        let static_exact_mips =
            LegacyStaticExactMipManifest::from_published().unwrap_or_else(|error| {
                panic!("published legacy static exact-mip manifest is invalid: {error}")
            });
        embedded_asset!(app, "legacy_model_base.wgsl");
        embedded_asset!(app, "legacy_model_outline.wgsl");
        embedded_asset!(app, "legacy_water.wgsl");
        app.insert_resource(static_exact_mips)
            .init_asset::<ExactMipPngBytes>()
            .init_asset_loader::<ExactMipPngBytesLoader>()
            .init_resource::<ExactMipChainCache>()
            .init_resource::<LegacyStaticMaterialCache>()
            .init_resource::<StaticAssetSharing>()
            .init_resource::<ModelMaterialSharing>()
            .init_resource::<LegacyMaterialExtrasParseCache>()
            .add_plugins((
                MaterialPlugin::<LegacyModelMaterial>::default(),
                MaterialPlugin::<LegacyOutlineMaterial>::default(),
                MaterialPlugin::<LegacyWaterMaterial>::default(),
            ))
            .add_systems(
                Update,
                (
                    // Runs before any admission so a texture loaded this frame
                    // already carries the committed anisotropy option.
                    sync_legacy_texture_anisotropy,
                    disable_static_frustum_culling_for_skinned_meshes,
                    discover_legacy_material_extras,
                    discover_legacy_material_renderer_order,
                    maintain_shared_static_assets,
                    apply_legacy_water_materials,
                    apply_legacy_static_world_materials,
                    apply_legacy_model_materials,
                    apply_legacy_material_renderer_order,
                    sync_legacy_sky_rimlight,
                    sync_legacy_outline_visibility,
                )
                    .chain(),
            );
        gpu_uniforms::install(app);
    }
}

fn embedded_shader_ref(filename: &'static str) -> ShaderRef {
    let path = match filename {
        "legacy_model_base.wgsl" => embedded_path!("legacy_model_base.wgsl"),
        "legacy_model_outline.wgsl" => embedded_path!("legacy_model_outline.wgsl"),
        "legacy_water.wgsl" => embedded_path!("legacy_water.wgsl"),
        _ => unreachable!("native shader set is closed"),
    };
    AssetPath::from_path_buf(path)
        .with_source("embedded")
        .into()
}

#[cfg(test)]
mod tests;
