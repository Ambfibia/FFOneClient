//! Numeric material parameters, texture sets and glTF texture bindings.

use super::material_uniforms::{LegacyModelMaterialUniform, LegacyOutlineUniform};
use super::metadata::legacy_static_glow_shader;
use super::render_plan::{LegacyBlendMode, LegacyModelRenderPlan, LegacyPassKind, LegacyPassPlan};
use super::shader_kind::{
    LegacyShaderKind, UnknownLegacyShaderName, legacy_shader_uses_fixed_function_fog,
};
use super::{LEGACY_MAIN_LIGHT_RAY_DIRECTION, LegacyModelMaterial, LegacyOutlineMaterial};
use bevy::prelude::*;
use ffone_skinned_model::{
    MaterialTextureSamplerBinding, NativeTextureMipLevel, TextureColorSpace, TextureMipProvenance,
};

/// Typed values copied from one native material record.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyModelMaterialParams {
    pub shader: LegacyShaderKind,
    /// Explicit native art direction; retains the validated surface render state.
    pub native_cel_shading: bool,
    /// Native ink pass, independent of the validated source surface shader.
    pub native_outline: bool,
    /// Black surface rim on rounded inset parts, using skinned normals.
    pub native_ink_rim: bool,
    pub base_color: LinearRgba,
    pub tint_color: LinearRgba,
    pub ambient_color: LinearRgba,
    pub emission: LinearRgba,
    /// Exact global or material-owned rim color. Ordinary toon programs are
    /// refreshed from the sampled area's sky color at runtime.
    pub rim_color: LinearRgba,
    pub rim_power: f32,
    pub rim_intensity: f32,
    pub shadow_strength: f32,
    pub emission_strength: f32,
    pub transparency: f32,
    pub uv_scale: Vec2,
    pub uv_offset: Vec2,
    pub uv_pivot: Vec2,
    pub uv_rotation_degrees: f32,
    pub light_direction_world: Vec3,
    pub alpha_cutoff: f32,
    pub outline_color: LinearRgba,
    pub outline_width: f32,
    pub fat_factor: f32,
    pub fusion_speed: f32,
    /// The static-world `normal_glow_*` fixed-function programs use
    /// `_BumpMap.a` to blend vertex lighting toward constant gray. Despite
    /// the serialized slot name, this is a glow mask, not a normal map.
    pub glow_mask: bool,
    /// Whether the exact ShaderLab program inherits the global fixed-function
    /// fog stage. This is source-program state, not a material heuristic.
    pub fixed_function_fog: bool,
}

impl LegacyModelMaterialParams {
    pub fn from_shader_name(name: &str) -> Result<Self, UnknownLegacyShaderName> {
        let mut params = Self::for_shader(LegacyShaderKind::classify_exact(name)?);
        params.glow_mask = legacy_static_glow_shader(name);
        params.fixed_function_fog = legacy_shader_uses_fixed_function_fog(name);
        Ok(params)
    }

    pub fn for_shader(shader: LegacyShaderKind) -> Self {
        let additive_fixed_function = matches!(
            shader,
            LegacyShaderKind::AdditiveOneOneTwoSided
                | LegacyShaderKind::AdditiveOneOneTwoSidedVertexColorAd
                | LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite
                | LegacyShaderKind::AdditiveOneOneBackface
                | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite
        );
        let particle_fixed_function = matches!(
            shader,
            LegacyShaderKind::ParticleSrcAlphaAdditiveTwoSided
                | LegacyShaderKind::ParticleOneMinusSrcAlphaAdditiveTwoSided
                | LegacyShaderKind::ParticleOneMinusDstColorAdditiveTwoSided
                | LegacyShaderKind::ParticleOneMinusDstColorAdditiveBackface
        );
        let fusion_matter = shader == LegacyShaderKind::FusionMatterLightDir;
        let transparent_rim = shader == LegacyShaderKind::SkinnedToonRimTransparent;
        Self {
            shader,
            base_color: if fusion_matter {
                LinearRgba::new(0.6, 0.6, 0.6, 1.0)
            } else if transparent_rim {
                LinearRgba::new(0.5, 0.5, 0.5, 1.0)
            } else {
                LinearRgba::WHITE
            },
            // The exact normal_blendOneOne ShaderLab family declares a white
            // `_AmbColor` static default. Unity applies it when a material did
            // not serialize an override, so black would erase untextured VFX.
            ambient_color: if fusion_matter {
                LinearRgba::new(1.0, 0.976, 0.208, 1.0)
            } else if additive_fixed_function {
                LinearRgba::WHITE
            } else {
                LinearRgba::BLACK
            },
            // The exact particle family declares `_TintColor = (.5,.5,.5,.5)`
            // and runs with Lighting Off. Saved material values override it.
            tint_color: if fusion_matter {
                LinearRgba::new(0.0, 1.0, 0.0, 1.0)
            } else if particle_fixed_function {
                LinearRgba::new(0.5, 0.5, 0.5, 0.5)
            } else {
                LinearRgba::WHITE
            },
            emission: if fusion_matter {
                LinearRgba::new(0.529, 0.576, 1.0, 1.0)
            } else if transparent_rim {
                LinearRgba::new(0.235, 0.235, 0.235, 0.0)
            } else {
                LinearRgba::BLACK
            },
            rim_color: if fusion_matter {
                LinearRgba::new(1.0, 0.976, 0.208, 1.0)
            } else {
                LinearRgba::WHITE
            },
            rim_power: if fusion_matter {
                2.0
            } else if transparent_rim {
                10.0
            } else {
                4.0
            },
            rim_intensity: if transparent_rim { 0.5 } else { 2.0 },
            shadow_strength: if fusion_matter { 0.65 } else { 0.0 },
            emission_strength: 1.0,
            transparency: if transparent_rim { 0.5 } else { 1.0 },
            uv_scale: Vec2::ONE,
            uv_offset: Vec2::ZERO,
            uv_pivot: Vec2::ZERO,
            uv_rotation_degrees: 0.0,
            light_direction_world: LEGACY_MAIN_LIGHT_RAY_DIRECTION,
            alpha_cutoff: 0.9,
            outline_color: LinearRgba::BLACK,
            outline_width: 0.005,
            fat_factor: 0.0,
            fusion_speed: 7.0,
            glow_mask: false,
            fixed_function_fog: false,
            native_cel_shading: false,
            native_outline: false,
            native_ink_rim: false,
        }
    }

    pub fn render_plan(&self) -> LegacyModelRenderPlan {
        let mut plan = LegacyModelRenderPlan::for_shader(self.shader);
        if self.native_outline && plan.outline().is_none() {
            let mut pass = plan.passes[0];
            pass.kind = LegacyPassKind::Outline;
            pass.render_mode.cull = super::render_plan::LegacyCullMode::Front;
            plan.passes.push(pass);
        }
        plan
    }

    /// Builds a non-outline material. The outline uses a separate vertex shader
    /// and therefore a separate companion entity/material.
    pub fn material_for_pass(
        &self,
        pass: LegacyPassPlan,
        textures: &LegacyModelTextures,
    ) -> Option<LegacyModelMaterial> {
        (pass.kind != LegacyPassKind::Outline).then(|| LegacyModelMaterial {
            uniform_owner: default(),
            sharing_state: default(),
            uniform: LegacyModelMaterialUniform {
                base_color: self.base_color,
                tint_color: self.tint_color,
                ambient_color: self.ambient_color,
                emission: self.emission,
                rim_color: self.rim_color,
                rim_effect: Vec4::new(
                    self.rim_power,
                    self.rim_intensity,
                    self.shader.rim_mode(),
                    self.shadow_strength,
                ),
                custom_effect: Vec4::new(
                    self.emission_strength,
                    self.transparency,
                    if self.shader == LegacyShaderKind::HologramSolidAdditive
                        && pass.render_mode.blend == LegacyBlendMode::SrcAlphaOne
                    {
                        1.0
                    } else {
                        0.0
                    },
                    if self.shader == LegacyShaderKind::HologramSolidAdditive {
                        self.alpha_cutoff
                    } else {
                        0.0
                    },
                ),
                uv_scale_offset: self
                    .uv_scale
                    .extend(self.uv_offset.x)
                    .extend(self.uv_offset.y),
                uv_pivot_rotation: self.uv_pivot.extend(self.uv_rotation_degrees).extend(0.0),
                uv_animation: Vec4::ZERO,
                light_direction_family: self
                    .light_direction_world
                    .extend(self.shader.shading_family()),
                alpha_effect: Vec4::new(
                    pass.alpha_test.cutoff(self.alpha_cutoff),
                    if pass.alpha_test.enabled() { 1.0 } else { 0.0 },
                    if pass.alpha_test.rejects_equality() {
                        1.0
                    } else {
                        0.0
                    },
                    if self.native_ink_rim { 2.0 } else if self.native_cel_shading { 1.0 } else { 0.0 },
                ),
                legacy_effect: Vec4::new(
                    self.fat_factor,
                    self.fusion_speed,
                    if self.glow_mask { 1.0 } else { 0.0 },
                    if !self.fixed_function_fog {
                        0.0
                    } else if pass.render_mode.blend.uses_additive_fog_color() {
                        // Unity's fixed-function fog contribution must fade an
                        // additive pass toward black. Fading toward the scene
                        // fog color injects that color into the framebuffer on
                        // every wave and made EPbarrier turn red.
                        2.0
                    } else {
                        1.0
                    },
                ),
            },
            base_texture: textures.base.clone(),
            toon_ramp: textures.toon_ramp.clone(),
            bump_texture: textures.bump.clone(),
            effect_map: textures.effect_map.clone(),
            render_mode: pass.render_mode,
            gpu_uv_animation: false,
            sort_bias: 0.0,
        })
    }

    pub fn outline_material(&self) -> Option<LegacyOutlineMaterial> {
        self.render_plan()
            .outline()
            .map(|pass| LegacyOutlineMaterial {
                uniform: LegacyOutlineUniform {
                    color: self.outline_color,
                    width_fat: Vec4::new(self.outline_width, self.fat_factor, 0.0, 0.0),
                    fog_params: Vec4::new(
                        if self.fixed_function_fog
                            && self.shader != LegacyShaderKind::FusionMatterLightDir
                        {
                            1.0
                        } else {
                            0.0
                        },
                        0.0,
                        0.0,
                        0.0,
                    ),
                },
                render_mode: pass.render_mode,
                sort_bias: 0.0,
            })
    }
}

#[derive(Debug, Clone, Default)]
pub struct LegacyModelTextures {
    pub base: Option<Handle<Image>>,
    pub toon_ramp: Option<Handle<Image>>,
    pub bump: Option<Handle<Image>>,
    pub effect_map: Option<Handle<Image>>,
    /// Exact null-slot `"white" {}` provenance. The corresponding optional
    /// image handle deliberately remains empty, so Bevy binds its implicit
    /// white image only when one of these flags made the slot complete.
    pub base_builtin_white: bool,
    pub toon_ramp_builtin_white: bool,
    pub bump_builtin_white: bool,
    pub effect_map_builtin_white: bool,
}

impl LegacyModelTextures {
    pub fn is_complete_for(&self, shader: LegacyShaderKind) -> bool {
        let has_base = self.base.is_some() || self.base_builtin_white;
        match shader {
            LegacyShaderKind::SkinnedToon
            | LegacyShaderKind::SkinnedToonCullOffFallback
            | LegacyShaderKind::SkinnedToonCullOffCategory
            | LegacyShaderKind::SkinnedToonRim
            | LegacyShaderKind::SkinnedToonRimTransparent
            | LegacyShaderKind::SkinnedToonRimMatcap
            | LegacyShaderKind::SkinnedToonFlipped
            | LegacyShaderKind::Toon => {
                has_base && (self.toon_ramp.is_some() || self.toon_ramp_builtin_white)
            }
            LegacyShaderKind::FusionEffect => {
                has_base
                    && (self.bump.is_some() || self.bump_builtin_white)
                    && (self.effect_map.is_some() || self.effect_map_builtin_white)
            }
            LegacyShaderKind::FusionMatterLightDir => has_base,
            LegacyShaderKind::AlphaBlendNormalGlow => {
                has_base && (self.bump.is_some() || self.bump_builtin_white)
            }
            _ => has_base,
        }
    }
}

/// A texture reference copied verbatim from `materials[*].extras.ffone`.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGltfTextureBinding {
    pub slot: String,
    pub texture_index: Option<usize>,
    pub source_name: Option<String>,
    pub uri: Option<String>,
    pub sampler: Option<MaterialTextureSamplerBinding>,
    pub mip_provenance: Option<TextureMipProvenance>,
    pub mip_levels: Option<Vec<NativeTextureMipLevel>>,
    pub color_space: TextureColorSpace,
    pub scale: Vec2,
    pub offset: Vec2,
    /// Exact source presence. Unity 4 ordinary m_TexEnvs serialize neither
    /// field; retaining `None` keeps authored zero distinct from absence.
    pub pivot: Option<Vec2>,
    pub rotation: Option<f32>,
}

impl LegacyGltfTextureBinding {
    /// Effective legacy default, applied only by runtime/shader consumption.
    pub fn effective_pivot(&self) -> Vec2 {
        self.pivot.unwrap_or(Vec2::ZERO)
    }

    /// Effective legacy default, applied only by runtime/shader consumption.
    pub fn effective_rotation(&self) -> f32 {
        self.rotation.unwrap_or(0.0)
    }
}
