//! GPU uniform layouts, pipeline keys and outline companions of the native materials.

use super::render_plan::{LegacyBlendMode, LegacyColorWriteMask, LegacyCullMode, LegacyRenderMode};
use super::{LegacyModelMaterial, LegacyOutlineMaterial};
use bevy::{prelude::*, render::render_resource::ShaderType};

#[derive(Debug, Clone, Copy, ShaderType, PartialEq)]
pub struct LegacyModelMaterialUniform {
    pub base_color: LinearRgba,
    pub tint_color: LinearRgba,
    pub ambient_color: LinearRgba,
    pub emission: LinearRgba,
    pub rim_color: LinearRgba,
    /// x=power, y=intensity, z=mode (1=sky, 2=fixed, 3=Fusion Matter),
    /// w=Fusion Matter shadow strength.
    pub rim_effect: Vec4,
    /// x=emission strength, y=transparency, zw=reserved.
    pub custom_effect: Vec4,
    pub uv_scale_offset: Vec4,
    /// xy=texture pivot, z=clockwise Unity degrees, w=GPU-time wrap period.
    pub uv_pivot_rotation: Vec4,
    /// xy=UV offset velocity, z=clockwise degrees/second, w=GPU start time.
    pub uv_animation: Vec4,
    pub light_direction_family: Vec4,
    /// x=cutoff, y=cutout enabled, z=reject equality, w=native cel (1), ink rim (2).
    pub alpha_effect: Vec4,
    /// x=`_FatFactor`, y=`_Speed`, z=glow-mask flag, w=fixed fog mode
    /// (0=off, 1=scene color, 2=additive black).
    pub legacy_effect: Vec4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegacyModelPipelineKey {
    pub(super) render_mode: LegacyRenderMode,
    pub(super) gpu_uv_animation: bool,
}

impl From<&LegacyModelMaterial> for LegacyModelPipelineKey {
    fn from(material: &LegacyModelMaterial) -> Self {
        Self {
            render_mode: material.render_mode,
            gpu_uv_animation: material.gpu_uv_animation,
        }
    }
}

#[derive(Debug, Clone, Copy, ShaderType, PartialEq)]
pub(super) struct LegacyWaterMaterialUniform {
    pub(super) color: LinearRgba,
    pub(super) wave_speed: Vec4,
    pub(super) refr_color: LinearRgba,
    pub(super) horizon_color: LinearRgba,
    pub(super) foam_color: LinearRgba,
    /// x=_WaveScale, y=_ReflDistort, z=_RefrDistort, w=ffPoison marker.
    pub(super) water_params: Vec4,
}

pub(super) const LEGACY_WATER_RENDER_MODE: LegacyRenderMode = LegacyRenderMode {
    blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
    cull: LegacyCullMode::Back,
    depth_write: true,
    color_write: LegacyColorWriteMask::Rgb,
    alpha_cutout: false,
    source_queue: 3_002,
};

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct LegacyOutlineUniform {
    pub color: LinearRgba,
    /// x=outline width, y=legacy fat factor.
    pub width_fat: Vec4,
    /// x=whether the exact source outline pass inherits fixed-function fog.
    pub fog_params: Vec4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegacyOutlinePipelineKey {
    pub(super) render_mode: LegacyRenderMode,
}

impl From<&LegacyOutlineMaterial> for LegacyOutlinePipelineKey {
    fn from(material: &LegacyOutlineMaterial) -> Self {
        Self {
            render_mode: material.render_mode,
        }
    }
}

/// Source marker: the scene loader must create a companion skinned entity.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LegacyOutlineSource {
    pub width: f32,
    pub color: LinearRgba,
}

/// Companion marker. It must share the source mesh, hierarchy and joint palette.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyOutlineCompanion {
    pub source_mesh_entity: Entity,
}
