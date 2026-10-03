// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x21000018 => FrameKind0104::Fixed {
            size: LsChangeCharNameFailure0104::SIZE,
        },
        0x21000017 => FrameKind0104::Fixed {
            size: LsChangeCharNameSuccess0104::SIZE,
        },
        0x2100000a => FrameKind0104::Fixed {
            size: LsCharCreateFailure0104::SIZE,
        },
        0x21000009 => FrameKind0104::Fixed {
            size: LsCharCreateSuccess0104::SIZE,
        },
        0x2100000e => FrameKind0104::Fixed {
            size: LsCharDeleteFailure0104::SIZE,
        },
        0x2100000d => FrameKind0104::Fixed {
            size: LsCharDeleteSuccess0104::SIZE,
        },
        0x21000003 => FrameKind0104::Fixed {
            size: LsCharInfoReply0104::SIZE,
        },
        0x2100000c => FrameKind0104::Fixed {
            size: LsCharSelectFailure0104::SIZE,
        },
        0x2100000b => FrameKind0104::Fixed {
            size: LsCharSelectSuccess0104::SIZE,
        },
        0x21000006 => FrameKind0104::Fixed {
            size: LsCheckCharNameFailure0104::SIZE,
        },
        0x21000005 => FrameKind0104::Fixed {
            size: LsCheckCharNameSuccess0104::SIZE,
        },
        0x21000014 => FrameKind0104::Fixed {
            size: LsCheckNameListFailure0104::SIZE,
        },
        0x21000013 => FrameKind0104::Fixed {
            size: LsCheckNameListSuccess0104::SIZE,
        },
        0x21000002 => FrameKind0104::Fixed {
            size: LsLoginFailure0104::SIZE,
        },
        0x21000001 => FrameKind0104::Fixed {
            size: LsLoginSuccess0104::SIZE,
        },
        0x21000015 => FrameKind0104::Fixed {
            size: LsPcExitDuplicateReply0104::SIZE,
        },
        0x21000008 => FrameKind0104::Fixed {
            size: LsSaveCharNameFailure0104::SIZE,
        },
        0x21000007 => FrameKind0104::Fixed {
            size: LsSaveCharNameSuccess0104::SIZE,
        },
        0x21000019 => FrameKind0104::Fixed {
            size: LsShardListInfoSuccess0104::SIZE,
        },
        0x21000010 => FrameKind0104::Fixed {
            size: LsShardSelectFailure0104::SIZE,
        },
        0x2100000f => FrameKind0104::Fixed {
            size: LsShardSelectSuccess0104::SIZE,
        },
        0x21000012 => FrameKind0104::Fixed {
            size: LsVersionCheckFailure0104::SIZE,
        },
        0x21000011 => FrameKind0104::Fixed {
            size: LsVersionCheckSuccess0104::SIZE,
        },
        0x21000016 => FrameKind0104::Fixed {
            size: LsLiveCheck0104::SIZE,
        },
        _ => return None,
    })
}
