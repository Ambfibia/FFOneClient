// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x83000065 => FrameKind0104::Fixed {
            size: RequestMakeBuddySuccess0104::SIZE,
        },
        _ => return None,
    })
}
