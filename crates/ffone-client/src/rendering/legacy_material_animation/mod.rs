//! Runtime sampling for exact Unity material float curves preserved in GLB extras.

use ffone_skinned_model::{AnimationMetadata, FloatCurve, Interpolation};
use serde_json::Value;

use crate::legacy_model_material::LegacyModelMaterial;

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyMaterialAnimationClip {
    pub name: String,
    pub duration: f32,
    pub looped: bool,
    pub float_curves: Vec<FloatCurve>,
}

pub fn parse_legacy_material_animation_clips(
    bytes: &[u8],
) -> Result<Vec<LegacyMaterialAnimationClip>, String> {
    let document = parse_glb_json(bytes)?;
    let animations = document
        .get("animations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut clips = Vec::new();
    for (index, animation) in animations.iter().enumerate() {
        let context = format!("animations[{index}]");
        let Some(non_trs) = animation.pointer("/extras/nonTrs").cloned() else {
            continue;
        };
        let name = animation
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| format!("{context}.name must be a non-empty string"))?;
        let duration = animation
            .pointer("/extras/duration")
            .and_then(Value::as_f64)
            .filter(|duration| duration.is_finite() && *duration >= 0.0)
            .ok_or_else(|| format!("{context}.extras.duration must be finite and non-negative"))?;
        if duration > f64::from(f32::MAX) {
            return Err(format!("{context}.extras.duration exceeds f32"));
        }
        let looped = animation
            .pointer("/extras/loop")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("{context}.extras.loop must be boolean"))?;
        let metadata: AnimationMetadata = serde_json::from_value(non_trs)
            .map_err(|error| format!("{context}.extras.nonTrs is invalid: {error}"))?;
        if metadata.float_curves.is_empty() {
            continue;
        }
        clips.push(LegacyMaterialAnimationClip {
            name: name.to_owned(),
            duration: duration as f32,
            looped,
            float_curves: metadata.float_curves,
        });
    }
    Ok(clips)
}

pub fn sample_legacy_float_curve(
    curve: &FloatCurve,
    time: f32,
    duration: f32,
    looped: bool,
) -> Option<f32> {
    if curve.times.is_empty() || curve.times.len() != curve.values.len() || !time.is_finite() {
        return None;
    }
    let mut time = f64::from(time.max(0.0));
    let duration = f64::from(duration);
    if looped && duration > 0.0 {
        time %= duration;
    }
    let first_time = *curve.times.first()?;
    let last_time = *curve.times.last()?;
    if time <= first_time {
        return finite_f32(*curve.values.first()?);
    }
    if time >= last_time {
        return finite_f32(*curve.values.last()?);
    }
    let right = curve.times.partition_point(|key_time| *key_time <= time);
    let left = right.checked_sub(1)?;
    let right = right.min(curve.times.len() - 1);
    let left_time = curve.times[left];
    let right_time = curve.times[right];
    let span = right_time - left_time;
    if !span.is_finite() || span <= 0.0 {
        return None;
    }
    let t = ((time - left_time) / span).clamp(0.0, 1.0);
    let left_value = curve.values[left];
    let right_value = curve.values[right];
    let value = match curve.interpolation {
        Interpolation::Step => left_value,
        Interpolation::Linear => left_value + (right_value - left_value) * t,
        Interpolation::CubicSpline => {
            let out_tangent = curve.out_tangents.as_ref()?.get(left).copied()?;
            let in_tangent = curve.in_tangents.as_ref()?.get(right).copied()?;
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            h00 * left_value
                + h10 * out_tangent * span
                + h01 * right_value
                + h11 * in_tangent * span
        }
    };
    finite_f32(value)
}

pub fn apply_legacy_material_float(
    material: &mut LegacyModelMaterial,
    property: &str,
    value: f32,
) -> bool {
    if !value.is_finite() {
        return false;
    }
    match property {
        "_TintColor.a" | "_Color.a" => material.uniform.base_color.alpha = value,
        "_MainTex.offset.x" => material.uniform.uv_scale_offset.z = value,
        "_MainTex.offset.y" => material.uniform.uv_scale_offset.w = value,
        _ => return false,
    }
    true
}

fn parse_glb_json(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err("semantic character is not a GLB 2.0 container".to_owned());
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().expect("checked GLB header"));
    let declared_length =
        u32::from_le_bytes(bytes[8..12].try_into().expect("checked GLB header")) as usize;
    if version != 2 || declared_length != bytes.len() {
        return Err(format!(
            "semantic character GLB header mismatch: version={version}, declared={declared_length}, actual={}",
            bytes.len()
        ));
    }
    let json_length =
        u32::from_le_bytes(bytes[12..16].try_into().expect("checked JSON chunk")) as usize;
    let json_type = u32::from_le_bytes(bytes[16..20].try_into().expect("checked JSON chunk"));
    let json_end = 20usize
        .checked_add(json_length)
        .ok_or_else(|| "semantic character GLB JSON length overflowed".to_owned())?;
    if json_type != 0x4e4f_534a || json_end > bytes.len() {
        return Err("semantic character GLB has no valid leading JSON chunk".to_owned());
    }
    serde_json::from_slice(&bytes[20..json_end])
        .map_err(|error| format!("semantic character GLB JSON is invalid: {error}"))
}

fn finite_f32(value: f64) -> Option<f32> {
    (value.is_finite() && value >= f64::from(f32::MIN) && value <= f64::from(f32::MAX))
        .then_some(value as f32)
}

#[cfg(test)]
mod tests;
