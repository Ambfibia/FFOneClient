use super::*;

pub fn publish_world_platform_packets(
    time: Res<Time>,
    mut clock: ResMut<WorldSurfacePacketClock>,
    players: Query<(
        &Transform,
        &crate::movement::LegacyPlayerController,
        &crate::world::NativeWorldGroundSupport,
    )>,
    parents: Query<&ChildOf>,
    triggers: Query<(&WorldTrigger, &GlobalTransform)>,
    mut gameplay: ResMut<WorldGameplayIntentQueue>,
) {
    let Ok((transform, controller, support)) = players.single() else {
        clock.0 = crate::movement::LEGACY_PACKET_SEND_INTERVAL;
        return;
    };
    let mut current = support.collider;
    let mut platform = None;
    for _ in 0..8 {
        if let Ok(candidate) = triggers.get(current)
            && candidate.0.kind == WorldTriggerKind::Platform
        {
            platform = Some(candidate);
            break;
        }
        let Ok(parent) = parents.get(current) else {
            break;
        };
        current = parent.parent();
    }
    let Some((trigger, global)) = platform else {
        clock.0 = crate::movement::LEGACY_PACKET_SEND_INTERVAL;
        return;
    };
    clock.0 += time.delta_secs().max(0.0);
    if clock.0 < crate::movement::LEGACY_PACKET_SEND_INTERVAL {
        return;
    }
    let local_position = transform.translation - global.translation();
    let request = PcMovePlatformRequest0104 {
        client_time: 0,
        local_position: ProtocolPosition::from_native(local_position).raw(),
        position: ProtocolPosition::from_native(transform.translation).raw(),
        velocity: ProtocolMoveVelocity::from_native(controller.velocity).raw(),
        down: i32::from(controller.velocity.y < 0.0),
        platform_id: trigger.object_id as i32,
        angle: LegacyUnityHeadingDegrees::new(controller.yaw_degrees)
            .to_protocol()
            .degrees(),
        key_value: controller.current_direction_key(),
        speed: controller.run_speed_server_units,
    };
    let _ = gameplay.push(packet::P_CL2FE_REQ_PC_MOVEPLATFORM, &request);
    clock.0 = 0.0;
}
