use super::*;

/// The legacy movement update is discarded for stalls at or above this delta.
pub const LEGACY_MAX_FRAME_DELTA: f32 = 0.25;

/// Normal movement packet interval from `cnAvatarThirdPersonMove`.
pub const LEGACY_PACKET_SEND_INTERVAL: f32 = 0.3;

/// Hard lower bound between normal movement packets.
pub const LEGACY_PACKET_MIN_INTERVAL: f32 = 0.3;

/// Yaw change that can make a packet eligible (the 0.3 s minimum still wins).
pub const LEGACY_PACKET_ANGLE_THRESHOLD: f32 = 0.2;

pub(super) fn packet_is_due(controller: &LegacyPlayerController, direction_key: u8) -> bool {
    controller.packet_elapsed > LEGACY_PACKET_MIN_INTERVAL
        && (controller.last_direction_key != direction_key
            || controller.packet_elapsed > LEGACY_PACKET_SEND_INTERVAL
            || (controller.yaw_degrees - controller.last_packet_yaw).abs()
                >= LEGACY_PACKET_ANGLE_THRESHOLD)
}
