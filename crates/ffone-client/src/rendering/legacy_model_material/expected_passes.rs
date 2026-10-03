//! Independent canonical pass expectations that cross-check parsed source passes.

use super::params::LegacyModelMaterialParams;
use super::shader_kind::{LegacyMaterialExtrasError, LegacyShaderKind};
use bevy::prelude::*;
use ffone_skinned_model::{
    MaterialAlphaReference, MaterialAlphaTestState, MaterialBlendFactor, MaterialBlendOperation,
    MaterialBlendState, MaterialCompareFunction, MaterialCullMode, MaterialOutlineState,
    MaterialPass,
};

pub(super) fn canonical_blend(
    enabled: bool,
    source: MaterialBlendFactor,
    destination: MaterialBlendFactor,
) -> MaterialBlendState {
    MaterialBlendState {
        enabled,
        source_color: source,
        destination_color: destination,
        color_operation: MaterialBlendOperation::Add,
        source_alpha: source,
        destination_alpha: destination,
        alpha_operation: MaterialBlendOperation::Add,
    }
}

/// Retrobution contains compiled skinned-toon programs whose declared names
/// alone do not determine BASE culling. Their serialized pass state is the
/// lossless discriminator: the common/fallback programs cull back faces,
/// while the audited character category programs render BASE two-sided. Keep
/// the exact names closed and use typed source passes instead of guessing by
/// model.
pub(super) fn refine_exact_shader_kind(
    declared_name: &str,
    classified: LegacyShaderKind,
    source_passes: &[MaterialPass],
) -> LegacyShaderKind {
    if ((declared_name == "SkinnedToonShading_blendSrcalphaInvsrcalpha_cullOff"
        && classified == LegacyShaderKind::SkinnedToonCullOffFallback)
        || (declared_name == "SkinnedToonShading_blendSrcalphaInvsrcalpha e1"
            && classified == LegacyShaderKind::SkinnedToon))
        && source_passes.len() == 2
        && source_passes[0].name.as_deref() == Some("BASE")
        && source_passes[0].cull == MaterialCullMode::Off
        && source_passes[1].name.as_deref() == Some("OUTLINE")
        && source_passes[1].cull == MaterialCullMode::Front
    {
        LegacyShaderKind::SkinnedToonCullOffCategory
    } else {
        classified
    }
}

pub(super) fn canonical_expected_passes(params: &LegacyModelMaterialParams) -> Vec<MaterialPass> {
    let replace = || canonical_blend(false, MaterialBlendFactor::One, MaterialBlendFactor::Zero);
    let alpha = || {
        canonical_blend(
            true,
            MaterialBlendFactor::SourceAlpha,
            MaterialBlendFactor::OneMinusSourceAlpha,
        )
    };
    let disabled_outline = || MaterialOutlineState::Disabled;
    let disabled_alpha = || MaterialAlphaTestState::Disabled;
    let pass = |name: Option<&str>,
                blend: MaterialBlendState,
                cull: MaterialCullMode,
                z_write: bool,
                alpha_test: MaterialAlphaTestState,
                color_mask: u8,
                outline: MaterialOutlineState| MaterialPass {
        name: name.map(str::to_owned),
        blend,
        cull,
        z_write,
        z_test: MaterialCompareFunction::LessEqual,
        alpha_test,
        color_mask,
        outline,
    };

    match params.shader {
        LegacyShaderKind::SkinnedToon
        | LegacyShaderKind::SkinnedToonCullOffFallback
        | LegacyShaderKind::SkinnedToonCullOffCategory
        | LegacyShaderKind::SkinnedToonRim
        | LegacyShaderKind::SkinnedToonRimTransparent
        | LegacyShaderKind::SkinnedToonRimMatcap
        | LegacyShaderKind::SkinnedToonFlipped
        | LegacyShaderKind::RimEmissiveColoredTransparent
        | LegacyShaderKind::Toon => vec![
            pass(
                if params.shader == LegacyShaderKind::RimEmissiveColoredTransparent {
                    None
                } else {
                    Some("BASE")
                },
                alpha(),
                if params.shader == LegacyShaderKind::SkinnedToonCullOffCategory {
                    MaterialCullMode::Off
                } else {
                    MaterialCullMode::Back
                },
                true,
                disabled_alpha(),
                0b1111,
                disabled_outline(),
            ),
            pass(
                Some("OUTLINE"),
                alpha(),
                if params.shader == LegacyShaderKind::SkinnedToonFlipped {
                    MaterialCullMode::Back
                } else {
                    MaterialCullMode::Front
                },
                true,
                disabled_alpha(),
                0b1111,
                MaterialOutlineState::WorldSpace {
                    width: f64::from(params.outline_width),
                    color: [
                        f64::from(params.outline_color.red),
                        f64::from(params.outline_color.green),
                        f64::from(params.outline_color.blue),
                        f64::from(params.outline_color.alpha),
                    ],
                },
            ),
        ],
        LegacyShaderKind::OpaqueNormal => vec![pass(
            None,
            replace(),
            MaterialCullMode::Back,
            true,
            disabled_alpha(),
            0b1111,
            disabled_outline(),
        )],
        LegacyShaderKind::AlphaBlendNormal
        | LegacyShaderKind::AlphaBlendNormalVertexColorAd
        | LegacyShaderKind::AlphaBlendNormalCullOff => {
            vec![pass(
                None,
                alpha(),
                if params.shader == LegacyShaderKind::AlphaBlendNormalCullOff {
                    MaterialCullMode::Off
                } else {
                    MaterialCullMode::Back
                },
                true,
                disabled_alpha(),
                0b1111,
                disabled_outline(),
            )]
        }
        LegacyShaderKind::AlphaBlendNormalGlow => vec![pass(
            None,
            alpha(),
            MaterialCullMode::Back,
            true,
            disabled_alpha(),
            // The exact glow program writes RGB only; framebuffer alpha is
            // deliberately preserved by `ColorMask RGB`.
            0b0111,
            disabled_outline(),
        )],
        LegacyShaderKind::SkinDirectionalAlphaBlend => vec![pass(
            Some("BASE"),
            alpha(),
            MaterialCullMode::Back,
            true,
            disabled_alpha(),
            0b1111,
            disabled_outline(),
        )],
        LegacyShaderKind::TransparentNormal | LegacyShaderKind::TransparentNormalCullOff => {
            vec![pass(
                None,
                alpha(),
                if params.shader == LegacyShaderKind::TransparentNormalCullOff {
                    MaterialCullMode::Off
                } else {
                    MaterialCullMode::Back
                },
                false,
                disabled_alpha(),
                if params.shader == LegacyShaderKind::TransparentNormalCullOff {
                    0b0111
                } else {
                    0b1111
                },
                disabled_outline(),
            )]
        }
        LegacyShaderKind::SrcAlphaZeroBackfaceDepthWrite => vec![pass(
            None,
            canonical_blend(
                true,
                MaterialBlendFactor::SourceAlpha,
                MaterialBlendFactor::Zero,
            ),
            MaterialCullMode::Back,
            true,
            disabled_alpha(),
            0b1111,
            disabled_outline(),
        )],
        LegacyShaderKind::SrcAlphaAdditiveBackface
        | LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite
        | LegacyShaderKind::SrcAlphaAdditiveTwoSided
        | LegacyShaderKind::SrcAlphaAdditiveTwoSidedVertexColorAd => vec![pass(
            None,
            canonical_blend(
                true,
                MaterialBlendFactor::SourceAlpha,
                MaterialBlendFactor::One,
            ),
            if matches!(
                params.shader,
                LegacyShaderKind::SrcAlphaAdditiveTwoSided
                    | LegacyShaderKind::SrcAlphaAdditiveTwoSidedVertexColorAd
            ) {
                MaterialCullMode::Off
            } else {
                MaterialCullMode::Back
            },
            params.shader == LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite,
            disabled_alpha(),
            0b0111,
            disabled_outline(),
        )],
        LegacyShaderKind::ParticleSrcAlphaAdditiveTwoSided => vec![pass(
            None,
            canonical_blend(
                true,
                MaterialBlendFactor::SourceAlpha,
                MaterialBlendFactor::One,
            ),
            MaterialCullMode::Off,
            false,
            MaterialAlphaTestState::Enabled {
                compare: MaterialCompareFunction::Greater,
                reference: MaterialAlphaReference::Literal { value: 0.01 },
            },
            0b0111,
            disabled_outline(),
        )],
        LegacyShaderKind::ParticleOneMinusSrcAlphaAdditiveTwoSided
        | LegacyShaderKind::ParticleOneMinusDstColorAdditiveTwoSided
        | LegacyShaderKind::ParticleOneMinusDstColorAdditiveBackface => vec![pass(
            None,
            canonical_blend(
                true,
                if params.shader == LegacyShaderKind::ParticleOneMinusSrcAlphaAdditiveTwoSided {
                    MaterialBlendFactor::OneMinusSourceAlpha
                } else {
                    MaterialBlendFactor::OneMinusDestinationColor
                },
                MaterialBlendFactor::One,
            ),
            if params.shader == LegacyShaderKind::ParticleOneMinusDstColorAdditiveBackface {
                MaterialCullMode::Back
            } else {
                MaterialCullMode::Off
            },
            false,
            MaterialAlphaTestState::Enabled {
                compare: MaterialCompareFunction::Greater,
                reference: MaterialAlphaReference::Literal { value: 0.01 },
            },
            0b0111,
            disabled_outline(),
        )],
        LegacyShaderKind::RotatingFlipbook | LegacyShaderKind::ScrollDistortAdditive => vec![pass(
            Some("BASE"),
            canonical_blend(
                true,
                MaterialBlendFactor::SourceAlpha,
                if params.shader == LegacyShaderKind::RotatingFlipbook {
                    MaterialBlendFactor::OneMinusSourceAlpha
                } else {
                    MaterialBlendFactor::One
                },
            ),
            MaterialCullMode::Off,
            false,
            disabled_alpha(),
            0b1111,
            disabled_outline(),
        )],
        LegacyShaderKind::HologramSolidAdditive => vec![
            pass(
                Some("BASE"),
                canonical_blend(
                    true,
                    MaterialBlendFactor::SourceAlpha,
                    MaterialBlendFactor::OneMinusSourceAlpha,
                ),
                MaterialCullMode::Off,
                true,
                disabled_alpha(),
                0b1111,
                disabled_outline(),
            ),
            pass(
                Some("OVERLAYS"),
                canonical_blend(
                    true,
                    MaterialBlendFactor::SourceAlpha,
                    MaterialBlendFactor::One,
                ),
                MaterialCullMode::Off,
                true,
                disabled_alpha(),
                0b1111,
                disabled_outline(),
            ),
        ],
        LegacyShaderKind::AdditiveTwoSided
        | LegacyShaderKind::AdditiveTestTwoSidedQueue3011
        | LegacyShaderKind::SrcAlphaAdditiveTestTwoSided => {
            vec![pass(
                None,
                canonical_blend(
                    true,
                    if params.shader == LegacyShaderKind::SrcAlphaAdditiveTestTwoSided {
                        MaterialBlendFactor::SourceAlpha
                    } else {
                        MaterialBlendFactor::One
                    },
                    MaterialBlendFactor::One,
                ),
                MaterialCullMode::Off,
                params.shader == LegacyShaderKind::SrcAlphaAdditiveTestTwoSided,
                MaterialAlphaTestState::Enabled {
                    compare: MaterialCompareFunction::Greater,
                    reference: MaterialAlphaReference::FloatProperty {
                        name: "_Cutoff".to_owned(),
                        resolved_value: f64::from(params.alpha_cutoff),
                    },
                },
                0b0111,
                disabled_outline(),
            )]
        }
        LegacyShaderKind::AdditiveOneOneTwoSided
        | LegacyShaderKind::AdditiveOneOneTwoSidedVertexColorAd
        | LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite
        | LegacyShaderKind::AdditiveOneOneBackface
        | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite => {
            vec![pass(
                None,
                canonical_blend(true, MaterialBlendFactor::One, MaterialBlendFactor::One),
                if matches!(
                    params.shader,
                    LegacyShaderKind::AdditiveOneOneBackface
                        | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite
                ) {
                    MaterialCullMode::Back
                } else {
                    MaterialCullMode::Off
                },
                matches!(
                    params.shader,
                    LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite
                        | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite
                ),
                disabled_alpha(),
                0b0111,
                disabled_outline(),
            )]
        }
        LegacyShaderKind::TransparentCutoutDefaultCulling
        | LegacyShaderKind::TransparentCutoutTwoSided => {
            let cull = if params.shader == LegacyShaderKind::TransparentCutoutTwoSided {
                MaterialCullMode::Off
            } else {
                MaterialCullMode::Back
            };
            vec![
                pass(
                    None,
                    replace(),
                    cull,
                    true,
                    MaterialAlphaTestState::Enabled {
                        compare: MaterialCompareFunction::GreaterEqual,
                        reference: MaterialAlphaReference::Literal { value: 0.9 },
                    },
                    0b1111,
                    disabled_outline(),
                ),
                pass(
                    None,
                    alpha(),
                    cull,
                    false,
                    disabled_alpha(),
                    0b0111,
                    disabled_outline(),
                ),
            ]
        }
        LegacyShaderKind::TransparentCutoutZWriteOffDefaultCulling => vec![pass(
            None,
            replace(),
            MaterialCullMode::Back,
            false,
            MaterialAlphaTestState::Enabled {
                compare: MaterialCompareFunction::Greater,
                reference: MaterialAlphaReference::FloatProperty {
                    name: "_Cutoff".to_owned(),
                    resolved_value: f64::from(params.alpha_cutoff),
                },
            },
            0b1111,
            disabled_outline(),
        )],
        LegacyShaderKind::FusionEffect => vec![pass(
            Some("BASE"),
            alpha(),
            MaterialCullMode::Back,
            true,
            disabled_alpha(),
            0b0111,
            disabled_outline(),
        )],
        LegacyShaderKind::FusionMatterLightDir => vec![
            pass(
                Some("BASE"),
                alpha(),
                MaterialCullMode::Back,
                true,
                disabled_alpha(),
                0b1111,
                disabled_outline(),
            ),
            pass(
                Some("OUTLINE"),
                alpha(),
                MaterialCullMode::Front,
                true,
                disabled_alpha(),
                0b1111,
                MaterialOutlineState::WorldSpace {
                    width: f64::from(params.outline_width),
                    color: [
                        f64::from(params.outline_color.red),
                        f64::from(params.outline_color.green),
                        f64::from(params.outline_color.blue),
                        f64::from(params.outline_color.alpha),
                    ],
                },
            ),
        ],
        LegacyShaderKind::DiffuseFade => vec![pass(
            Some("BASE"),
            alpha(),
            MaterialCullMode::Back,
            true,
            disabled_alpha(),
            0b0111,
            disabled_outline(),
        )],
    }
}

pub(super) fn validate_typed_source_passes(
    params: &LegacyModelMaterialParams,
    actual: &[MaterialPass],
) -> Result<(), LegacyMaterialExtrasError> {
    let expected = canonical_expected_passes(params);
    if actual.len() != expected.len() {
        return Err(LegacyMaterialExtrasError(format!(
            "typed passes for {} contain {} passes, expected {}",
            params.shader.exact_name(),
            actual.len(),
            expected.len()
        )));
    }
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        if actual.name != expected.name
            || actual.blend != expected.blend
            || actual.cull != expected.cull
            || actual.z_write != expected.z_write
            || actual.z_test != expected.z_test
            || actual.color_mask != expected.color_mask
            || !alpha_tests_match(&actual.alpha_test, &expected.alpha_test)
            || !outlines_match(&actual.outline, &expected.outline)
        {
            return Err(LegacyMaterialExtrasError(format!(
                "typed pass {index} for {} contradicts the canonical pass: actual {actual:?}, expected {expected:?}",
                params.shader.exact_name()
            )));
        }
    }
    Ok(())
}

pub(super) fn alpha_tests_match(actual: &MaterialAlphaTestState, expected: &MaterialAlphaTestState) -> bool {
    match (actual, expected) {
        (MaterialAlphaTestState::Disabled, MaterialAlphaTestState::Disabled) => true,
        (
            MaterialAlphaTestState::Enabled {
                compare: actual_compare,
                reference: actual_reference,
            },
            MaterialAlphaTestState::Enabled {
                compare: expected_compare,
                reference: expected_reference,
            },
        ) if actual_compare == expected_compare => match (actual_reference, expected_reference) {
            (
                MaterialAlphaReference::Literal { value: actual },
                MaterialAlphaReference::Literal { value: expected },
            ) => floats_match(*actual, *expected),
            (
                MaterialAlphaReference::FloatProperty {
                    name: actual_name,
                    resolved_value: actual,
                },
                MaterialAlphaReference::FloatProperty {
                    name: expected_name,
                    resolved_value: expected,
                },
            ) => actual_name == expected_name && floats_match(*actual, *expected),
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn outlines_match(actual: &MaterialOutlineState, expected: &MaterialOutlineState) -> bool {
    match (actual, expected) {
        (MaterialOutlineState::Disabled, MaterialOutlineState::Disabled) => true,
        (
            MaterialOutlineState::WorldSpace {
                width: actual_width,
                color: actual_color,
            },
            MaterialOutlineState::WorldSpace {
                width: expected_width,
                color: expected_color,
            },
        ) => {
            floats_match(*actual_width, *expected_width)
                && actual_color
                    .iter()
                    .zip(expected_color)
                    .all(|(actual, expected)| floats_match(*actual, *expected))
        }
        _ => false,
    }
}

pub(super) fn floats_match(actual: f64, expected: f64) -> bool {
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= 1.0e-6
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LegacyMaterialMetadataError(pub String);
