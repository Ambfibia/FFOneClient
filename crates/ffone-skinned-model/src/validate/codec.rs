use super::*;

pub(super) fn channel_key_payload(channel: &AnimationChannel, index: usize) -> Option<ExactTrsKeyPayload> {
    let time = *channel.times.get(index)?;
    let tangent_mode = if channel.tangent_modes.is_empty() {
        None
    } else {
        Some(*channel.tangent_modes.get(index)?)
    };
    match &channel.values {
        TrackValues::Translation(values) | TrackValues::Scale(values) => {
            let value = *values.get(index)?;
            let in_tangent = match channel.in_tangents.as_ref() {
                Some(TrackValues::Translation(values) | TrackValues::Scale(values)) => {
                    Some(*values.get(index)?)
                }
                None => None,
                Some(TrackValues::Rotation(_)) => return None,
            };
            let out_tangent = match channel.out_tangents.as_ref() {
                Some(TrackValues::Translation(values) | TrackValues::Scale(values)) => {
                    Some(*values.get(index)?)
                }
                None => None,
                Some(TrackValues::Rotation(_)) => return None,
            };
            Some(ExactTrsKeyPayload::Vec3(ExactVec3KeyPayload {
                time,
                value,
                in_tangent,
                out_tangent,
                tangent_mode,
            }))
        }
        TrackValues::Rotation(values) => {
            let value = *values.get(index)?;
            let in_tangent = match channel.in_tangents.as_ref() {
                Some(TrackValues::Rotation(values)) => Some(*values.get(index)?),
                None => None,
                Some(TrackValues::Translation(_) | TrackValues::Scale(_)) => return None,
            };
            let out_tangent = match channel.out_tangents.as_ref() {
                Some(TrackValues::Rotation(values)) => Some(*values.get(index)?),
                None => None,
                Some(TrackValues::Translation(_) | TrackValues::Scale(_)) => return None,
            };
            Some(ExactTrsKeyPayload::Quaternion(ExactQuaternionKeyPayload {
                time,
                value,
                in_tangent,
                out_tangent,
                tangent_mode,
            }))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ConstantCurvePayload {
    Vec3 {
        value: [f64; 3],
        in_tangent: Option<[f64; 3]>,
        out_tangent: Option<[f64; 3]>,
        tangent_mode: Option<i32>,
    },
    Quaternion {
        value: [f64; 4],
        in_tangent: Option<[f64; 4]>,
        out_tangent: Option<[f64; 4]>,
        tangent_mode: Option<i32>,
    },
}

pub(super) fn constant_channel_payload(channel: &AnimationChannel) -> Option<ConstantCurvePayload> {
    if channel.times.is_empty()
        || !channel.duplicate_keys.is_empty()
        || channel.source_key_count as usize != channel.times.len()
        || channel
            .source_key_indices
            .iter()
            .enumerate()
            .any(|(index, source)| *source as usize != index)
    {
        return None;
    }
    let mut payload = None;
    for index in 0..channel.times.len() {
        let current = match channel_key_payload(channel, index)? {
            ExactTrsKeyPayload::Vec3(key) => ConstantCurvePayload::Vec3 {
                value: key.value,
                in_tangent: key.in_tangent,
                out_tangent: key.out_tangent,
                tangent_mode: key.tangent_mode,
            },
            ExactTrsKeyPayload::Quaternion(key) => ConstantCurvePayload::Quaternion {
                value: key.value,
                in_tangent: key.in_tangent,
                out_tangent: key.out_tangent,
                tangent_mode: key.tangent_mode,
            },
        };
        if payload
            .as_ref()
            .is_some_and(|previous| previous != &current)
        {
            return None;
        }
        payload.get_or_insert(current);
    }
    payload
}

pub(super) fn exact_payload_time(payload: &ExactTrsKeyPayload) -> f64 {
    match payload {
        ExactTrsKeyPayload::Vec3(key) => key.time,
        ExactTrsKeyPayload::Quaternion(key) => key.time,
    }
}
