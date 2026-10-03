use super::*;

/// Validated special-world packets awaiting the normal network bridge.
#[derive(Resource, Debug, Default)]
pub struct WorldGameplayIntentQueue {
    pub(super) pending: VecDeque<RegisteredGameplayRequest0104>,
}

impl WorldGameplayIntentQueue {
    pub fn push<T: WirePayload>(&mut self, packet_type: u32, payload: &T) -> bool {
        let Ok(request) = RegisteredGameplayRequest0104::new(packet_type, payload.encode()) else {
            return false;
        };
        self.pending.push_back(request);
        true
    }

    pub fn push_payload(&mut self, packet_type: u32, payload: Vec<u8>) -> bool {
        let Ok(request) = RegisteredGameplayRequest0104::new(packet_type, payload) else { return false; };
        self.pending.push_back(request);
        true
    }

    pub fn take_all(&mut self) -> VecDeque<RegisteredGameplayRequest0104> {
        std::mem::take(&mut self.pending)
    }
}

pub(super) fn world_ep_effect_command(
    root: Entity,
    owner_world_matrix: Option<&[[f64; 4]; 4]>,
    element: &serde_json::Value,
) -> Result<Option<TutorialEffectRuntimeCommand>, String> {
    let effect_id = element
        .get("particleIndex")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| "particle element has no integer particleIndex".to_owned())?;
    if effect_id == 0 {
        return Ok(None);
    }
    let effect_id = i32::try_from(effect_id)
        .map_err(|_| format!("particleIndex {effect_id} does not fit i32"))?;
    if !ffone_runtime_contracts::RETROBUTION_WORLD_EP_EFFECT_IDS.contains(&effect_id) {
        return Err(format!(
            "particleIndex {effect_id} is outside the exact world EP effect catalog"
        ));
    }
    let unity_position = json_vector3(element, "position")?;
    let unity_rotation = json_quaternion(element, "rotation")?;
    let element_scale = element
        .get("scale")
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| "particle element scale is not finite and positive".to_owned())?;
    let local = Transform::from_translation(Vec3::new(
        -unity_position.x,
        unity_position.y,
        unity_position.z,
    ))
    .with_rotation(Quat::from_xyzw(
        unity_rotation.x,
        -unity_rotation.y,
        -unity_rotation.z,
        unity_rotation.w,
    ))
    .with_scale(Vec3::splat(element_scale));
    let owner = owner_world_matrix
        .map(mat4_from_world_matrix)
        .unwrap_or(Mat4::IDENTITY);
    let world = owner * local.to_matrix();
    let (scale, rotation, position) = world.to_scale_rotation_translation();
    let absolute_scale = scale.abs();
    let minimum = absolute_scale.min_element();
    let maximum = absolute_scale.max_element();
    if !minimum.is_finite()
        || minimum <= f32::EPSILON
        || maximum - minimum > maximum.max(1.0) * 0.000_1
    {
        return Err(format!(
            "world EP effect {effect_id} has unsupported non-uniform world scale {scale:?}"
        ));
    }
    Ok(Some(TutorialEffectRuntimeCommand::Add {
        effect_id,
        placement: TutorialEffectPlacement::ExactEntityWorld {
            root_entity: root,
            position,
            rotation,
        },
        scale: (absolute_scale.x + absolute_scale.y + absolute_scale.z) / 3.0,
        tracked: false,
        name: None,
        destroy_after_seconds: None,
        source_line: 0,
    }))
}

pub(super) fn make_jumppad_request(
    position: Vec3,
    velocity: Vec3,
    yaw_degrees: f32,
    key_value: u8,
    launch_power: f32,
) -> PcJumppadRequest0104 {
    PcJumppadRequest0104 {
        client_time: (launch_power.max(0.0) * 100.0) as u64,
        position: ProtocolPosition::from_native(position).raw(),
        velocity: ProtocolScaledVelocity::from_native(velocity).raw(),
        angle: LegacyUnityHeadingDegrees::new(yaw_degrees)
            .to_protocol()
            .degrees(),
        key_value,
    }
}
