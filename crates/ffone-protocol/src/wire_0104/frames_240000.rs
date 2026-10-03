// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x12000009 => FrameKind0104::Fixed {
            size: LsCheckNameList0104::SIZE,
        },
        0x1200000c => FrameKind0104::Fixed {
            size: LsLiveCheckReply0104::SIZE,
        },
        0x1200000d => FrameKind0104::Fixed {
            size: LsChangeCharNameRequest0104::SIZE,
        },
        0x12000004 => FrameKind0104::Fixed {
            size: LsCharCreateRequest0104::SIZE,
        },
        0x12000006 => FrameKind0104::Fixed {
            size: LsCharDeleteRequest0104::SIZE,
        },
        0x12000005 => FrameKind0104::Fixed {
            size: LsCharSelectRequest0104::SIZE,
        },
        0x12000002 => FrameKind0104::Fixed {
            size: LsCheckCharNameRequest0104::SIZE,
        },
        0x12000001 => FrameKind0104::Fixed {
            size: LsLoginRequest0104::SIZE,
        },
        0x1200000b => FrameKind0104::Fixed {
            size: LsPcExitDuplicateRequest0104::SIZE,
        },
        0x12000003 => FrameKind0104::Fixed {
            size: LsSaveCharNameRequest0104::SIZE,
        },
        0x1200000a => FrameKind0104::Fixed {
            size: LsSaveCharTutorRequest0104::SIZE,
        },
        0x1200000e => FrameKind0104::Fixed {
            size: LsServerSelectRequest0104::SIZE,
        },
        0x12000008 => FrameKind0104::Fixed {
            size: LsShardListInfoRequest0104::SIZE,
        },
        0x12000007 => FrameKind0104::Fixed {
            size: LsShardSelectRequest0104::SIZE,
        },
        _ => return None,
    })
}
