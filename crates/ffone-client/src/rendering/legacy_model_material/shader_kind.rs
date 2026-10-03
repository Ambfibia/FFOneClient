//! Exact legacy shader-name classification, fixed-function fog and extras errors.

use bevy::prelude::*;
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyShaderKind {
    SkinnedToon,
    SkinnedToonCullOffFallback,
    SkinnedToonCullOffCategory,
    SkinnedToonRim,
    SkinnedToonRimTransparent,
    SkinnedToonRimMatcap,
    SkinnedToonFlipped,
    RimEmissiveColoredTransparent,
    Toon,
    SkinDirectionalAlphaBlend,
    OpaqueNormal,
    AlphaBlendNormal,
    /// Exact fixed-function `normal_glow_blendSrcalphaInvsrcalpha` program.
    /// `_BumpMap` is an alpha glow mask, never a tangent-space normal map.
    AlphaBlendNormalGlow,
    AlphaBlendNormalVertexColorAd,
    AlphaBlendNormalCullOff,
    TransparentNormal,
    TransparentNormalCullOff,
    /// Exact fixed-function `normal_blendSrcalphaZero` program used by
    /// Past Pokey Oaks pool/house trim. ShaderLab leaves Cull and ZWrite at
    /// their defaults, so only the framebuffer blend differs from `normal`.
    SrcAlphaZeroBackfaceDepthWrite,
    SrcAlphaAdditiveBackface,
    SrcAlphaAdditiveBackfaceDepthWrite,
    SrcAlphaAdditiveTwoSided,
    SrcAlphaAdditiveTwoSidedVertexColorAd,
    ParticleSrcAlphaAdditiveTwoSided,
    ParticleOneMinusSrcAlphaAdditiveTwoSided,
    ParticleOneMinusDstColorAdditiveTwoSided,
    ParticleOneMinusDstColorAdditiveBackface,
    RotatingFlipbook,
    ScrollDistortAdditive,
    HologramSolidAdditive,
    AdditiveTwoSided,
    SrcAlphaAdditiveTestTwoSided,
    AdditiveTestTwoSidedQueue3011,
    AdditiveOneOneTwoSided,
    AdditiveOneOneTwoSidedVertexColorAd,
    AdditiveOneOneTwoSidedDepthWrite,
    AdditiveOneOneBackface,
    AdditiveOneOneBackfaceDepthWrite,
    TransparentCutoutDefaultCulling,
    TransparentCutoutZWriteOffDefaultCulling,
    TransparentCutoutTwoSided,
    FusionEffect,
    FusionMatterLightDir,
    DiffuseFade,
}

impl LegacyShaderKind {
    /// Match only a true source shader name. No suffix stripping or fuzzy
    /// matching is allowed: close names have different render state.
    pub fn classify_exact(name: &str) -> Result<Self, UnknownLegacyShaderName> {
        match name {
            "SkinnedToonShading_blendSrcalphaInvsrcalpha"
            | "SkinnedToonShading_blendSrcalphaInvsrcalpha e1" => Ok(Self::SkinnedToon),
            "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff" => {
                Ok(Self::SkinnedToonCullOffFallback)
            }
            "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_Rim" => Ok(Self::SkinnedToonRim),
            "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_Rim_transparent" => {
                Ok(Self::SkinnedToonRimTransparent)
            }
            "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_RimMatcap" => {
                Ok(Self::SkinnedToonRimMatcap)
            }
            "SkinnedToonShading_blendSrcalphaInvsrcalphaflipped" => Ok(Self::SkinnedToonFlipped),
            "Custom/RimEmissiveColoredTransparentTextureShaderNoScroll" => {
                Ok(Self::RimEmissiveColoredTransparent)
            }
            "ToonShading_blendSrcalphaInvsrcalpha" => Ok(Self::Toon),
            "Skin_DirLight_AmbLight_blendSrcalphaInvsrcalpha" => {
                Ok(Self::SkinDirectionalAlphaBlend)
            }
            "normal" => Ok(Self::OpaqueNormal),
            "normal_blendSrcalphaInvsrcalpha" => Ok(Self::AlphaBlendNormal),
            "normal_glow_blendSrcalphaInvsrcalpha" => Ok(Self::AlphaBlendNormalGlow),
            "normal_blendSrcalphaInvsrcalpha_vertexColorAD" => {
                Ok(Self::AlphaBlendNormalVertexColorAd)
            }
            "normal_blendSrcalphaInvsrcalpha_cullOff" => Ok(Self::AlphaBlendNormalCullOff),
            "normal_blendSrcalphaInvsrcalpha_zwriteOff" => Ok(Self::TransparentNormal),
            "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff" => {
                Ok(Self::TransparentNormalCullOff)
            }
            "normal_blendSrcalphaZero" => Ok(Self::SrcAlphaZeroBackfaceDepthWrite),
            "normal_blendSrcalphaOne_zwriteOff" => Ok(Self::SrcAlphaAdditiveBackface),
            "normal_blendSrcalphaOne" => Ok(Self::SrcAlphaAdditiveBackfaceDepthWrite),
            "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD" => {
                Ok(Self::SrcAlphaAdditiveTwoSidedVertexColorAd)
            }
            "normal_blendSrcalphaOne_zwriteOff_cullOff" => Ok(Self::SrcAlphaAdditiveTwoSided),
            "particle_blendSrcalphaOne_zwriteOff_cullOff"
            | "particle_blendSrcalphaOne_zwriteOff_cullOff_charcreation" => {
                Ok(Self::ParticleSrcAlphaAdditiveTwoSided)
            }
            "particle_blendInvsrcalphaOne_zwriteOff_cullOff" => {
                Ok(Self::ParticleOneMinusSrcAlphaAdditiveTwoSided)
            }
            "particle_blendInvdestcolorOne_zwriteOff_cullOff" => {
                Ok(Self::ParticleOneMinusDstColorAdditiveTwoSided)
            }
            "particle_blendInvdestcolorOne_zwriteOff" => {
                Ok(Self::ParticleOneMinusDstColorAdditiveBackface)
            }
            "Custom/VFX/RotatingFlipbook" => Ok(Self::RotatingFlipbook),
            "Custom/VFX/ScrollDistort_wMask_Gradient_Additive" => Ok(Self::ScrollDistortAdditive),
            "Custom/VFX/Unique/HologramSolid_Additive" => Ok(Self::HologramSolidAdditive),
            "normal_blendSrcalphaOneTest_cullOff" => Ok(Self::SrcAlphaAdditiveTestTwoSided),
            "normal_blendOneOneTest_cullOff" => Ok(Self::AdditiveTwoSided),
            "normal_blendOneOneTest_zwriteOff_cullOff" => Ok(Self::AdditiveTestTwoSidedQueue3011),
            "normal_blendOneOne_zwriteOff_cullOff" => Ok(Self::AdditiveOneOneTwoSided),
            "normal_blendOneOne_zwriteOff_cullOff_vertexColorAD" => {
                Ok(Self::AdditiveOneOneTwoSidedVertexColorAd)
            }
            "normal_blendOneOne_cullOff" => Ok(Self::AdditiveOneOneTwoSidedDepthWrite),
            "normal_blendOneOne_zwriteOff" => Ok(Self::AdditiveOneOneBackface),
            "normal_blendOneOne" => Ok(Self::AdditiveOneOneBackfaceDepthWrite),
            "normal_blendSrcalphaInvsrcalphaTest" => Ok(Self::TransparentCutoutDefaultCulling),
            "normal_blendSrcalphaInvsrcalphaTest_zwriteOff" => {
                Ok(Self::TransparentCutoutZWriteOffDefaultCulling)
            }
            "normal_blendSrcalphaInvsrcalphaTest_cullOff" => Ok(Self::TransparentCutoutTwoSided),
            "Skin_FusionEffect_blendSrcalphaInvsrcalpha"
            | "Skin_FusionEffect_blendSrcalphaInvsrcalpha_tutorial" => Ok(Self::FusionEffect),
            "SkinnedFusionMatterLightDir_blendSrcalphaInvsrcalpha" => {
                Ok(Self::FusionMatterLightDir)
            }
            "retro_diffuseFade" => Ok(Self::DiffuseFade),
            _ => Err(UnknownLegacyShaderName(name.to_owned())),
        }
    }

    pub const fn exact_name(self) -> &'static str {
        match self {
            Self::SkinnedToon => "SkinnedToonShading_blendSrcalphaInvsrcalpha",
            Self::SkinnedToonCullOffFallback => {
                "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff"
            }
            Self::SkinnedToonCullOffCategory => {
                "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff"
            }
            Self::SkinnedToonRim => "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_Rim",
            Self::SkinnedToonRimTransparent => {
                "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_Rim_transparent"
            }
            Self::SkinnedToonRimMatcap => {
                "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_RimMatcap"
            }
            Self::SkinnedToonFlipped => "SkinnedToonShading_blendSrcalphaInvsrcalphaflipped",
            Self::RimEmissiveColoredTransparent => {
                "Custom/RimEmissiveColoredTransparentTextureShaderNoScroll"
            }
            Self::Toon => "ToonShading_blendSrcalphaInvsrcalpha",
            Self::SkinDirectionalAlphaBlend => "Skin_DirLight_AmbLight_blendSrcalphaInvsrcalpha",
            Self::OpaqueNormal => "normal",
            Self::AlphaBlendNormal => "normal_blendSrcalphaInvsrcalpha",
            Self::AlphaBlendNormalGlow => "normal_glow_blendSrcalphaInvsrcalpha",
            Self::AlphaBlendNormalVertexColorAd => "normal_blendSrcalphaInvsrcalpha_vertexColorAD",
            Self::AlphaBlendNormalCullOff => "normal_blendSrcalphaInvsrcalpha_cullOff",
            Self::TransparentNormal => "normal_blendSrcalphaInvsrcalpha_zwriteOff",
            Self::TransparentNormalCullOff => "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff",
            Self::SrcAlphaZeroBackfaceDepthWrite => "normal_blendSrcalphaZero",
            Self::SrcAlphaAdditiveBackface => "normal_blendSrcalphaOne_zwriteOff",
            Self::SrcAlphaAdditiveBackfaceDepthWrite => "normal_blendSrcalphaOne",
            Self::SrcAlphaAdditiveTwoSidedVertexColorAd => {
                "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD"
            }
            Self::SrcAlphaAdditiveTwoSided => "normal_blendSrcalphaOne_zwriteOff_cullOff",
            Self::ParticleSrcAlphaAdditiveTwoSided => "particle_blendSrcalphaOne_zwriteOff_cullOff",
            Self::ParticleOneMinusSrcAlphaAdditiveTwoSided => {
                "particle_blendInvsrcalphaOne_zwriteOff_cullOff"
            }
            Self::ParticleOneMinusDstColorAdditiveTwoSided => {
                "particle_blendInvdestcolorOne_zwriteOff_cullOff"
            }
            Self::ParticleOneMinusDstColorAdditiveBackface => {
                "particle_blendInvdestcolorOne_zwriteOff"
            }
            Self::RotatingFlipbook => "Custom/VFX/RotatingFlipbook",
            Self::ScrollDistortAdditive => "Custom/VFX/ScrollDistort_wMask_Gradient_Additive",
            Self::HologramSolidAdditive => "Custom/VFX/Unique/HologramSolid_Additive",
            Self::SrcAlphaAdditiveTestTwoSided => "normal_blendSrcalphaOneTest_cullOff",
            Self::AdditiveTwoSided => "normal_blendOneOneTest_cullOff",
            Self::AdditiveTestTwoSidedQueue3011 => "normal_blendOneOneTest_zwriteOff_cullOff",
            Self::AdditiveOneOneTwoSided => "normal_blendOneOne_zwriteOff_cullOff",
            Self::AdditiveOneOneTwoSidedVertexColorAd => {
                "normal_blendOneOne_zwriteOff_cullOff_vertexColorAD"
            }
            Self::AdditiveOneOneTwoSidedDepthWrite => "normal_blendOneOne_cullOff",
            Self::AdditiveOneOneBackface => "normal_blendOneOne_zwriteOff",
            Self::AdditiveOneOneBackfaceDepthWrite => "normal_blendOneOne",
            Self::TransparentCutoutDefaultCulling => "normal_blendSrcalphaInvsrcalphaTest",
            Self::TransparentCutoutZWriteOffDefaultCulling => {
                "normal_blendSrcalphaInvsrcalphaTest_zwriteOff"
            }
            Self::TransparentCutoutTwoSided => "normal_blendSrcalphaInvsrcalphaTest_cullOff",
            Self::FusionEffect => "Skin_FusionEffect_blendSrcalphaInvsrcalpha",
            Self::FusionMatterLightDir => "SkinnedFusionMatterLightDir_blendSrcalphaInvsrcalpha",
            Self::DiffuseFade => "retro_diffuseFade",
        }
    }

    pub(super) const fn rim_mode(self) -> f32 {
        match self {
            Self::SkinnedToon
            | Self::SkinnedToonCullOffFallback
            | Self::SkinnedToonCullOffCategory
            | Self::SkinnedToonFlipped
            | Self::Toon => 1.0,
            Self::SkinnedToonRim | Self::SkinnedToonRimTransparent => 2.0,
            Self::FusionMatterLightDir => 3.0,
            _ => 0.0,
        }
    }

    pub(super) const fn shading_family(self) -> f32 {
        match self {
            Self::SrcAlphaAdditiveTwoSidedVertexColorAd => 9.0,
            Self::SkinnedToon
            | Self::SkinnedToonCullOffFallback
            | Self::SkinnedToonCullOffCategory
            | Self::SkinnedToonRim
            | Self::SkinnedToonRimTransparent
            | Self::SkinnedToonRimMatcap
            | Self::SkinnedToonFlipped
            | Self::RimEmissiveColoredTransparent
            | Self::Toon => 1.0,
            Self::FusionEffect => 2.0,
            Self::FusionMatterLightDir => 7.0,
            Self::HologramSolidAdditive => 8.0,
            Self::SkinDirectionalAlphaBlend => 3.0,
            // The infected-zone fence uses a black `_Color`, non-black
            // `_AmbColor`, Lighting On and additive One/One blending. Keep its
            // fixed-function ambient path distinct from the other audited
            // family-0 normal programs.
            Self::AdditiveOneOneTwoSided
            | Self::AdditiveOneOneTwoSidedVertexColorAd
            | Self::AdditiveOneOneTwoSidedDepthWrite
            | Self::AdditiveOneOneBackface
            | Self::AdditiveOneOneBackfaceDepthWrite => 4.0,
            Self::ParticleSrcAlphaAdditiveTwoSided
            | Self::ParticleOneMinusSrcAlphaAdditiveTwoSided
            | Self::ParticleOneMinusDstColorAdditiveTwoSided
            | Self::ParticleOneMinusDstColorAdditiveBackface => 5.0,
            // The exact `normal_blendSrcalphaInvsrcalpha` fixed-function
            // program has `Lighting On`. Its EPbarrier generator material has
            // a non-zero `_AmbColor`, so it must receive RenderSettings
            // ambient without enabling that term for unrelated normal/custom
            // shader families.
            Self::AlphaBlendNormal
            | Self::AlphaBlendNormalGlow
            | Self::SrcAlphaAdditiveTestTwoSided => 6.0,
            _ => 0.0,
        }
    }
}

pub(super) fn legacy_shader_uses_fixed_function_fog(name: &str) -> bool {
    // Exact clean-primary ShaderLab state. These programs either have no Fog
    // override (and therefore inherit RenderSettings) or, for the skinned toon
    // program, compile an explicit `oFog` output. Keep this as an allow-list:
    // visually similar programs can explicitly disable fog. In particular the
    // Darklands tree's `normal_blendOneOne_zwriteOff` and EPbarrier center's
    // `normal_blendOneOne_zwriteOff_cullOff` both declare `Fog { Mode Off }`.
    //
    // Acceptance evidence includes clean-primary World_shared_part4 Shader
    // PathIDs 1607 (vertex-color alpha blend), 1615 (alpha blend), 1616
    // (SrcAlpha/One), 1628 (two-pass cutout), Nano's compiled toon/skinned
    // toon programs, and Retro_shared's compiled cull-off toon program. Static
    // aliases remain exact; no suffix stripping occurs.
    matches!(
        name,
        "SkinnedToonShading_blendSrcalphaInvsrcalpha"
            | "SkinnedToonShading_blendSrcalphaInvsrcalpha e1"
            | "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff"
            | "Custom/SkinnedToonShading_blendSrcalphaInvsrcalpha_Rim_transparent"
            | "SkinnedFusionMatterLightDir_blendSrcalphaInvsrcalpha"
            | "ToonShading_blendSrcalphaInvsrcalpha"
            | "normal"
            | "RetroLit"
            | "normal_blendSrcalphaInvsrcalpha"
            | "normal_blendSrcalphaInvsrcalpha_vertexColorAD"
            | "normal_blendSrcalphaInvsrcalpha_cullOff"
            | "normal_blendSrcalphaInvsrcalpha_cullOff_vertexColorAD"
            | "normal_blendSrcalphaInvsrcalpha_zwriteOff"
            | "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff"
            | "normal_blendSrccolorInvsrcalpha_zwriteOff_cullOff"
            | "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff_vertexColorAD"
            | "normal_blendSrcalphaInvsrcalphaTest"
            | "normal_blendSrcalphaInvsrcalphaTest_zwriteOff"
            | "normal_blendSrcalphaInvsrcalphaTest_cullOff"
            | "normal_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD"
            | "normal_blendSrcalphaInvsrcalphaTest_zwriteOff_cullOff"
            | "normal_glow_blendSrcalphaInvsrcalpha"
            | "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff"
            | "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD"
            | "normal_transparentcutoutinfrontofWater"
            | "normal_blendSrcalphaInvsrccolor"
            | "normal_blendSrcalphaZero"
            | "normal_blendSrcalphaOne_zwriteOff_cullOff"
            | "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD"
            | "normal_blendSrcalphaOneTest_cullOff"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownLegacyShaderName(pub String);

impl fmt::Display for UnknownLegacyShaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported exact legacy shader name: {}", self.0)
    }
}

impl Error for UnknownLegacyShaderName {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyMaterialExtrasError(pub String);

impl fmt::Display for LegacyMaterialExtrasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for LegacyMaterialExtrasError {}
