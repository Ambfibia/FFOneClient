use super::*;

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}

pub(super) fn same_optional_duration(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => same_duration(left, right),
        (None, None) => true,
        _ => false,
    }
}

pub(super) fn same_duration(left: f64, right: f64) -> bool {
    let tolerance = 1.0e-12 * left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= tolerance
}

pub(super) fn channel_kind(channel: &AnimationChannel) -> EmptyTrsBindingKind {
    match &channel.values {
        TrackValues::Translation(_) => EmptyTrsBindingKind::Translation,
        TrackValues::Rotation(_) => EmptyTrsBindingKind::Rotation,
        TrackValues::Scale(_) => EmptyTrsBindingKind::Scale,
    }
}

pub(super) fn trs_source_rank(kind: EmptyTrsBindingKind, encoding: EmptyTrsSourceEncoding) -> Option<u8> {
    match (kind, encoding) {
        (EmptyTrsBindingKind::Translation, EmptyTrsSourceEncoding::Plain) => Some(0),
        (EmptyTrsBindingKind::Rotation, EmptyTrsSourceEncoding::Plain) => Some(1),
        (EmptyTrsBindingKind::Rotation, EmptyTrsSourceEncoding::Compressed) => Some(2),
        (EmptyTrsBindingKind::Scale, EmptyTrsSourceEncoding::Plain) => Some(3),
        (_, EmptyTrsSourceEncoding::Compressed) => None,
    }
}

pub(super) fn duplicate_vec3_keys_match_channel(
    keys: &[ExactVec3Key],
    values: &[[f64; 3]],
    canonical: &AnimationChannel,
    context: &str,
) -> Result<bool> {
    Ok(keys
        .iter()
        .map(|key| key.source_key_index)
        .eq(canonical.source_key_indices.iter().copied())
        && keys
            .iter()
            .map(|key| key.time)
            .eq(canonical.times.iter().copied())
        && keys.iter().map(|key| key.value).eq(values.iter().copied())
        && exact_vec3_tangents(keys, true, context)?
            == canonical.in_tangents.as_ref().map(|values| match values {
                TrackValues::Translation(values) | TrackValues::Scale(values) => values.clone(),
                TrackValues::Rotation(_) => unreachable!("validated canonical vec3 channel"),
            })
        && exact_vec3_tangents(keys, false, context)?
            == canonical.out_tangents.as_ref().map(|values| match values {
                TrackValues::Translation(values) | TrackValues::Scale(values) => values.clone(),
                TrackValues::Rotation(_) => unreachable!("validated canonical vec3 channel"),
            })
        && exact_tangent_modes(keys.iter().map(|key| key.tangent_mode), context)?
            == canonical.tangent_modes)
}

pub(super) fn duplicate_quaternion_keys_match_channel(
    keys: &[ExactQuaternionKey],
    values: &[[f64; 4]],
    canonical: &AnimationChannel,
    context: &str,
) -> Result<bool> {
    Ok(keys
        .iter()
        .map(|key| key.source_key_index)
        .eq(canonical.source_key_indices.iter().copied())
        && keys
            .iter()
            .map(|key| key.time)
            .eq(canonical.times.iter().copied())
        && keys.iter().map(|key| key.value).eq(values.iter().copied())
        && exact_quaternion_tangents(keys, true, context)?
            == canonical.in_tangents.as_ref().map(|values| match values {
                TrackValues::Rotation(values) => values.clone(),
                TrackValues::Translation(_) | TrackValues::Scale(_) => {
                    unreachable!("validated canonical quaternion channel")
                }
            })
        && exact_quaternion_tangents(keys, false, context)?
            == canonical.out_tangents.as_ref().map(|values| match values {
                TrackValues::Rotation(values) => values.clone(),
                TrackValues::Translation(_) | TrackValues::Scale(_) => {
                    unreachable!("validated canonical quaternion channel")
                }
            })
        && exact_tangent_modes(keys.iter().map(|key| key.tangent_mode), context)?
            == canonical.tangent_modes)
}

pub(super) fn exact_vec3_tangents(
    keys: &[ExactVec3Key],
    input: bool,
    context: &str,
) -> Result<Option<Vec<[f64; 3]>>> {
    exact_optional_values(
        keys.iter().map(|key| {
            if input {
                key.in_tangent
            } else {
                key.out_tangent
            }
        }),
        context,
        "vec3 tangent",
    )
}

pub(super) fn exact_quaternion_tangents(
    keys: &[ExactQuaternionKey],
    input: bool,
    context: &str,
) -> Result<Option<Vec<[f64; 4]>>> {
    exact_optional_values(
        keys.iter().map(|key| {
            if input {
                key.in_tangent
            } else {
                key.out_tangent
            }
        }),
        context,
        "quaternion tangent",
    )
}

pub(super) fn exact_optional_values<T: Copy>(
    values: impl IntoIterator<Item = Option<T>>,
    context: &str,
    label: &str,
) -> Result<Option<Vec<T>>> {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    if values.iter().any(Option::is_none) {
        return invalid(format!("{context} only partially preserves {label}s"));
    }
    Ok(Some(values.into_iter().flatten().collect()))
}

pub(super) fn exact_tangent_modes(
    values: impl IntoIterator<Item = Option<i32>>,
    context: &str,
) -> Result<Vec<i32>> {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.iter().all(Option::is_none) {
        return Ok(Vec::new());
    }
    if values.iter().any(Option::is_none) {
        return invalid(format!(
            "{context} only partially preserves duplicate tangentMode"
        ));
    }
    Ok(values.into_iter().flatten().collect())
}

pub(super) fn curve_source_field(kind: EmptyTrsBindingKind) -> &'static str {
    match kind {
        EmptyTrsBindingKind::Translation => "m_PositionCurves",
        EmptyTrsBindingKind::Rotation => "m_RotationCurves",
        EmptyTrsBindingKind::Scale => "m_ScaleCurves",
    }
}

pub(super) fn interpolation_from_vec3_keys(keys: &[ExactVec3Key], context: &str) -> Result<Interpolation> {
    let in_tangents = exact_vec3_tangents(keys, true, context)?;
    let out_tangents = exact_vec3_tangents(keys, false, context)?;
    match (in_tangents.is_some(), out_tangents.is_some()) {
        (true, true) => Ok(Interpolation::CubicSpline),
        (false, false) => Ok(Interpolation::Linear),
        _ => invalid(format!("{context} only partially preserves tangents")),
    }
}

pub(super) fn interpolation_from_quaternion_keys(
    keys: &[ExactQuaternionKey],
    context: &str,
) -> Result<Interpolation> {
    let in_tangents = exact_quaternion_tangents(keys, true, context)?;
    let out_tangents = exact_quaternion_tangents(keys, false, context)?;
    match (in_tangents.is_some(), out_tangents.is_some()) {
        (true, true) => Ok(Interpolation::CubicSpline),
        (false, false) => Ok(Interpolation::Linear),
        _ => invalid(format!("{context} only partially preserves tangents")),
    }
}

pub(super) fn native_target_binding_count(
    clip: &crate::AnimationClip,
    kind: EmptyTrsBindingKind,
    target_node: u32,
) -> usize {
    clip.channels
        .iter()
        .filter(|channel| channel_kind(channel) == kind && channel.target_node == target_node)
        .count()
        + clip
            .metadata
            .empty_trs_bindings
            .iter()
            .filter(|binding| binding.kind == kind && binding.target_node == target_node)
            .count()
        + clip
            .metadata
            .duplicate_trs_bindings
            .iter()
            .filter(|binding| binding.kind == kind && binding.target_node == target_node)
            .count()
        + clip
            .metadata
            .curve_recoveries
            .iter()
            .filter(|recovery| recovery.kind == kind && recovery.target_node == target_node)
            .map(|recovery| recovery.rejected.len())
            .sum::<usize>()
}

pub(super) fn curve_recovery_keys_last_time(keys: &DuplicateTrsKeys) -> Option<f64> {
    match keys {
        DuplicateTrsKeys::Vec3(keys) => keys.last().map(|key| key.time),
        DuplicateTrsKeys::Quaternion(keys) => keys.last().map(|key| key.time),
    }
}

pub(super) fn same_optional_f64_bits(left: Option<f64>, right: Option<f64>) -> bool {
    left.map(f64::to_bits) == right.map(f64::to_bits)
}

pub(super) fn track_kind(values: &TrackValues) -> u8 {
    match values {
        TrackValues::Translation(_) => 0,
        TrackValues::Rotation(_) => 1,
        TrackValues::Scale(_) => 2,
    }
}

pub(super) fn valid_name(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() || value.len() > 1_024 || value.chars().any(char::is_control) {
        return invalid(format!("{label} name is empty or invalid"));
    }
    Ok(())
}

pub(super) fn finite<'a>(label: &str, values: impl IntoIterator<Item = &'a f64>) -> Result<()> {
    for value in values {
        if !value.is_finite() {
            return invalid(format!("{label} contains a non-finite value"));
        }
        if !(*value as f32).is_finite() {
            return invalid(format!("{label} contains a value not representable as f32"));
        }
    }
    Ok(())
}

pub(super) fn normalized_quat(value: &[f64; 4], label: &str) -> Result<()> {
    let length = value
        .iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt();
    if (length - 1.0).abs() > 1.0e-4 {
        return invalid(format!("{label} is not normalized"));
    }
    Ok(())
}
