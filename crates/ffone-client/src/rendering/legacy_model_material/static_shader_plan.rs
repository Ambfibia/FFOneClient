//! Static-world shader plans and named static material values.

use super::render_plan::{
    LegacyAlphaTest, LegacyBlendMode, LegacyColorWriteMask, LegacyCullMode, LegacyModelRenderPlan,
    LegacyPassKind, LegacyPassPlan, LegacyRenderMode,
};
use super::shader_kind::{LegacyShaderKind, UnknownLegacyShaderName};
use bevy::prelude::*;
use serde_json::Value;

pub(super) fn legacy_static_shader_plan(
    name: &str,
) -> Result<(LegacyShaderKind, Vec<LegacyPassPlan>), UnknownLegacyShaderName> {
    let shader = match name {
        "RetroLit" => {
            // The material parser admits only the exact zero-specular,
            // unit-alpha contract. Reuse its identical ambient/diffuse color
            // program with the source opaque color-pass state, not alpha blend.
            return Ok((
                LegacyShaderKind::AlphaBlendNormal,
                LegacyModelRenderPlan::for_shader(LegacyShaderKind::OpaqueNormal).passes,
            ));
        }
        "normal_glow_blendSrcalphaInvsrcalpha" => LegacyShaderKind::AlphaBlendNormalGlow,
        "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff"
        | "normal_glow_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD" => {
            return Ok((
                LegacyShaderKind::TransparentCutoutTwoSided,
                vec![legacy_static_pass(
                    LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                    LegacyCullMode::Off,
                    true,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::GreaterMaterialCutoff,
                    3_000,
                )],
            ));
        }
        "normal_glow_blendOneOne_zwriteOff_vertexColorAD" => {
            return Ok((
                LegacyShaderKind::AdditiveOneOneTwoSidedVertexColorAd,
                vec![legacy_static_pass(
                    LegacyBlendMode::OneOne,
                    LegacyCullMode::Off,
                    false,
                    LegacyColorWriteMask::Rgba,
                    LegacyAlphaTest::Disabled,
                    3_001,
                )],
            ));
        }
        // Static-world v1 publishes no COLOR_0, so these variants reduce to
        // the corresponding non-vertex-color render plan.
        "normal_blendSrcalphaInvsrcalpha_vertexColorAD" => LegacyShaderKind::AlphaBlendNormal,
        "normal_blendSrcalphaInvsrcalpha_cullOff_vertexColorAD" => {
            LegacyShaderKind::AlphaBlendNormalCullOff
        }
        "normal_blendSrcalphaInvsrcalphaTest_cullOff_vertexColorAD" => {
            LegacyShaderKind::TransparentCutoutTwoSided
        }
        "normal_blendSrcalphaInvsrcalpha_zwriteOff_cullOff_vertexColorAD" => {
            LegacyShaderKind::TransparentNormalCullOff
        }
        "normal_blendOneOne_zwriteOff_cullOff_vertexColorAD" => {
            LegacyShaderKind::AdditiveOneOneTwoSided
        }
        "normal_blendSrcalphaOne_zwriteOff_cullOff_vertexColorAD" => {
            LegacyShaderKind::SrcAlphaAdditiveTwoSided
        }
        "normal_blendOneOneTest" => {
            return Ok((
                LegacyShaderKind::AdditiveTwoSided,
                vec![legacy_static_pass(
                    LegacyBlendMode::OneOne,
                    LegacyCullMode::Back,
                    false,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::GreaterMaterialCutoff,
                    3_000,
                )],
            ));
        }
        "normal_blendOneOne" | "normal_blendOneOne_vertexColorAD" => {
            return Ok((
                LegacyShaderKind::AdditiveOneOneBackface,
                vec![legacy_static_pass(
                    LegacyBlendMode::OneOne,
                    LegacyCullMode::Back,
                    true,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::Disabled,
                    3_000,
                )],
            ));
        }
        "normal_blendOneOne_zwriteOff" | "normal_blendOneOne_zwriteOff_vertexColorAD" => {
            return Ok((
                LegacyShaderKind::AdditiveOneOneBackface,
                vec![legacy_static_pass(
                    LegacyBlendMode::OneOne,
                    LegacyCullMode::Back,
                    false,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::Disabled,
                    3_011,
                )],
            ));
        }
        "normal_blendSrcalphaOne_zwriteOff" | "normal_blendSrcalphaOne_zwriteOff_vertexColorAD" => {
            return Ok((
                LegacyShaderKind::SrcAlphaAdditiveBackface,
                vec![legacy_static_pass(
                    LegacyBlendMode::SrcAlphaOne,
                    LegacyCullMode::Back,
                    false,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::Disabled,
                    3_010,
                )],
            ));
        }
        "normal_blendSrcalphaOne" | "normal_blendSrcalphaOne_vertexColorAD" => {
            return Ok((
                LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite,
                vec![legacy_static_pass(
                    LegacyBlendMode::SrcAlphaOne,
                    LegacyCullMode::Back,
                    true,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::Disabled,
                    3_000,
                )],
            ));
        }
        "normal_transparentcutoutinfrontofWater" => {
            return Ok((
                LegacyShaderKind::TransparentCutoutTwoSided,
                vec![
                    legacy_static_pass(
                        LegacyBlendMode::Replace,
                        LegacyCullMode::Off,
                        true,
                        LegacyColorWriteMask::Rgba,
                        LegacyAlphaTest::GreaterEqualLiteralNineTenths,
                        3_001,
                    ),
                    LegacyPassPlan {
                        kind: LegacyPassKind::TransparentColor,
                        ..legacy_static_pass(
                            LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                            LegacyCullMode::Off,
                            false,
                            LegacyColorWriteMask::Rgb,
                            LegacyAlphaTest::Disabled,
                            3_001,
                        )
                    },
                ],
            ));
        }
        "normal_blendSrccolorInvsrcalpha_zwriteOff_cullOff" => {
            // Ship vortex: the same lit texture/primary DOUBLE program as
            // AlphaBlendNormal, with source-color blending and RGB-only writes.
            return Ok((
                LegacyShaderKind::AlphaBlendNormal,
                vec![legacy_static_pass(
                    LegacyBlendMode::SrcColorOneMinusSrcAlpha,
                    LegacyCullMode::Off,
                    false,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::Disabled,
                    3_010,
                )],
            ));
        }
        "normal_blendSrcalphaInvsrccolor" => {
            return Ok((
                LegacyShaderKind::AlphaBlendNormal,
                vec![legacy_static_pass(
                    LegacyBlendMode::SrcAlphaOneMinusSrcColor,
                    LegacyCullMode::Back,
                    true,
                    LegacyColorWriteMask::Rgb,
                    LegacyAlphaTest::Disabled,
                    3_000,
                )],
            ));
        }
        "normal_blendSrcalphaInvsrcalphaTest_zwriteOff_cullOff" => {
            return Ok((
                LegacyShaderKind::OpaqueNormal,
                vec![legacy_static_pass(
                    LegacyBlendMode::Replace,
                    LegacyCullMode::Off,
                    false,
                    LegacyColorWriteMask::Rgba,
                    LegacyAlphaTest::GreaterMaterialCutoff,
                    2_900,
                )],
            ));
        }
        _ => LegacyShaderKind::classify_exact(name)?,
    };
    let plan = LegacyModelRenderPlan::for_shader(shader);
    if shader == LegacyShaderKind::FusionEffect
        || plan
            .passes
            .iter()
            .any(|pass| pass.kind == LegacyPassKind::Outline)
    {
        return Err(UnknownLegacyShaderName(format!(
            "{name} (non-static texture or outline contract is unsupported)"
        )));
    }
    Ok((shader, plan.passes))
}

pub(super) fn legacy_static_pass(
    blend: LegacyBlendMode,
    cull: LegacyCullMode,
    depth_write: bool,
    color_write: LegacyColorWriteMask,
    alpha_test: LegacyAlphaTest,
    source_queue: i32,
) -> LegacyPassPlan {
    LegacyPassPlan {
        kind: LegacyPassKind::Surface,
        render_mode: LegacyRenderMode {
            blend,
            cull,
            depth_write,
            color_write,
            alpha_cutout: alpha_test.enabled(),
            source_queue,
        },
        alpha_test,
    }
}

pub(super) fn static_vec2(value: Option<&Value>) -> Option<Vec2> {
    let value = value?.as_object()?;
    Some(Vec2::new(
        value.get("x")?.as_f64()? as f32,
        value.get("y")?.as_f64()? as f32,
    ))
}

pub(super) fn legacy_static_named_vec4(values: Option<&Value>, name: &str) -> Option<[f32; 4]> {
    let value = values?
        .as_array()?
        .iter()
        .find(|value| value.get("name").and_then(Value::as_str) == Some(name))?
        .get("value")?;
    if let Some(values) = value.as_array() {
        return Some([
            values.first()?.as_f64()? as f32,
            values.get(1)?.as_f64()? as f32,
            values.get(2)?.as_f64()? as f32,
            values.get(3)?.as_f64()? as f32,
        ]);
    }
    let value = value.as_object()?;
    Some([
        value.get("r")?.as_f64()? as f32,
        value.get("g")?.as_f64()? as f32,
        value.get("b")?.as_f64()? as f32,
        value.get("a")?.as_f64()? as f32,
    ])
}

pub(super) fn legacy_static_named_float(values: Option<&Value>, name: &str) -> Option<f32> {
    values?
        .as_array()?
        .iter()
        .find(|value| value.get("name").and_then(Value::as_str) == Some(name))?
        .get("value")?
        .as_f64()
        .map(|value| value as f32)
}
