//! Conversion of legacy render state into Bevy blend/cull/write state.

use super::render_plan::{LegacyBlendMode, LegacyColorWriteMask, LegacyCullMode, LegacyRenderMode};
use bevy::{
    prelude::*,
    render::render_resource::{
        BlendComponent, BlendFactor, BlendOperation, BlendState, ColorWrites, Face,
        RenderPipelineDescriptor,
    },
};
use serde_json::Value;

pub(super) fn named_vec4(values: Option<&Value>, name: &str) -> Option<[f32; 4]> {
    let entry = values?
        .as_array()?
        .iter()
        .find(|value| value.get("name").and_then(Value::as_str) == Some(name))?;
    let values = entry.get("value")?.as_array()?;
    Some([
        values.first()?.as_f64()? as f32,
        values.get(1)?.as_f64()? as f32,
        values.get(2)?.as_f64()? as f32,
        values.get(3)?.as_f64()? as f32,
    ])
}

pub(super) fn named_float(values: Option<&Value>, name: &str) -> Option<f32> {
    values?
        .as_array()?
        .iter()
        .find(|value| value.get("name").and_then(Value::as_str) == Some(name))?
        .get("value")?
        .as_f64()
        .map(|value| value as f32)
}

pub(super) fn linear_rgba(value: [f32; 4]) -> LinearRgba {
    LinearRgba::new(value[0], value[1], value[2], value[3])
}

pub(super) fn apply_render_mode(descriptor: &mut RenderPipelineDescriptor, mode: LegacyRenderMode) {
    descriptor.primitive.cull_mode = bevy_cull_mode(mode.cull);
    if let Some(depth) = descriptor.depth_stencil.as_mut() {
        depth.depth_write_enabled = Some(mode.depth_write);
    }
    if let Some(fragment) = descriptor.fragment.as_mut() {
        for target in fragment.targets.iter_mut().flatten() {
            target.blend = bevy_blend_state(mode.blend);
            target.write_mask = bevy_color_write_mask(mode.color_write);
        }
    }
}

pub(super) fn bevy_cull_mode(mode: LegacyCullMode) -> Option<Face> {
    match mode {
        LegacyCullMode::Off => None,
        LegacyCullMode::Front => Some(Face::Front),
        LegacyCullMode::Back => Some(Face::Back),
    }
}

pub(super) fn bevy_blend_state(mode: LegacyBlendMode) -> Option<BlendState> {
    match mode {
        LegacyBlendMode::Replace => None,
        // Unity ShaderLab's `Blend SrcAlpha OneMinusSrcAlpha` applies the
        // same factors to color and alpha. Bevy's `ALPHA_BLENDING` constant
        // instead uses `One` for source alpha, so spell out the audited state.
        LegacyBlendMode::SrcAlphaOneMinusSrcAlpha => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
        }),
        LegacyBlendMode::SrcColorOneMinusSrcAlpha => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::Src,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::Src,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
        }),
        LegacyBlendMode::SrcAlphaOneMinusSrcColor => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrc,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrc,
                operation: BlendOperation::Add,
            },
        }),
        // Clean-primary ShaderLab `Blend SrcAlpha Zero` uses the same factor
        // pair for color and alpha. Do not approximate this as ordinary alpha
        // blending: the destination must contribute nothing.
        LegacyBlendMode::SrcAlphaZero => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::Zero,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::Zero,
                operation: BlendOperation::Add,
            },
        }),
        LegacyBlendMode::SrcAlphaOne => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        }),
        LegacyBlendMode::OneMinusSrcAlphaOne => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::OneMinusSrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::OneMinusSrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        }),
        LegacyBlendMode::OneMinusDstColorOne => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::OneMinusDst,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::OneMinusDst,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        }),
        LegacyBlendMode::OneOne => Some(BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        }),
    }
}

pub(super) fn bevy_color_write_mask(mode: LegacyColorWriteMask) -> ColorWrites {
    match mode {
        LegacyColorWriteMask::Rgb => ColorWrites::RED | ColorWrites::GREEN | ColorWrites::BLUE,
        LegacyColorWriteMask::Rgba => ColorWrites::ALL,
    }
}
