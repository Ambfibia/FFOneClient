// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x130000a3 => FrameKind0104::Fixed {
            size: GmRewardRateRequest0104::SIZE,
        },
        0x13000099 => FrameKind0104::Fixed {
            size: GmSetPcSkillRequest0104::SIZE,
        },
        0x13000082 => FrameKind0104::Fixed {
            size: GmTargetPcSpecialStateOnoffRequest0104::SIZE,
        },
        0x13000092 => FrameKind0104::Fixed {
            size: PcStreetstallCancelRequest0104::SIZE,
        },
        0x13000097 => FrameKind0104::Fixed {
            size: PcStreetstallItemBuyRequest0104::SIZE,
        },
        0x13000096 => FrameKind0104::Fixed {
            size: PcStreetstallItemListRequest0104::SIZE,
        },
        0x13000091 => FrameKind0104::Fixed {
            size: PcStreetstallReadyRequest0104::SIZE,
        },
        0x13000093 => FrameKind0104::Fixed {
            size: PcStreetstallRegistItemRequest0104::SIZE,
        },
        0x13000095 => FrameKind0104::Fixed {
            size: PcStreetstallSaleStartRequest0104::SIZE,
        },
        0x13000094 => FrameKind0104::Fixed {
            size: PcStreetstallUnregistItemRequest0104::SIZE,
        },
        0x1300008a => FrameKind0104::Fixed {
            size: ChannelInfoRequest0104::SIZE,
        },
        0x13000084 => FrameKind0104::Fixed {
            size: NpcGroupInviteRequest0104::SIZE,
        },
        0x13000085 => FrameKind0104::Fixed {
            size: NpcGroupKickRequest0104::SIZE,
        },
        0x13000090 => FrameKind0104::Counted {
            header_size: PcAttackCharsRequest0104::SIZE,
            count_offset: 0,
            count_width: 4,
            trailer_size: 8,
        },
        0x1300009e => FrameKind0104::Fixed {
            size: PcBeltRequest0104::SIZE,
        },
        0x1300008b => FrameKind0104::Fixed {
            size: PcChannelNumRequest0104::SIZE,
        },
        0x130000a2 => FrameKind0104::Fixed {
            size: PcDisassembleItemRequest0104::SIZE,
        },
        0x1300008f => FrameKind0104::Fixed {
            size: PcFindNameAcceptBuddyRequest0104::SIZE,
        },
        0x1300008e => FrameKind0104::Fixed {
            size: PcFindNameMakeBuddyRequest0104::SIZE,
        },
        0x13000086 => FrameKind0104::Fixed {
            size: PcFirstUseFlagSetRequest0104::SIZE,
        },
        0x13000098 => FrameKind0104::Fixed {
            size: PcItemCombinationRequest0104::SIZE,
        },
        0x130000a4 => FrameKind0104::Fixed {
            size: PcItemEnchantRequest0104::SIZE,
        },
        0x1300008d => FrameKind0104::Fixed {
            size: PcLoadingCompleteRequest0104::SIZE,
        },
        0x13000081 => FrameKind0104::Fixed {
            size: PcRecvEmailCandyRequest0104::SIZE,
        },
        0x13000080 => FrameKind0104::Fixed {
            size: PcRecvEmailItemRequest0104::SIZE,
        },
        0x13000089 => FrameKind0104::Fixed {
            size: PcRecvEmailItemAllRequest0104::SIZE,
        },
        0x130000a1 => FrameKind0104::Fixed {
            size: PcRegistQuickSlotRequest0104::SIZE,
        },
        0x1300009d => FrameKind0104::Fixed {
            size: PcRopeRequest0104::SIZE,
        },
        0x13000083 => FrameKind0104::Fixed {
            size: PcSetCurrentMissionIdRequest0104::SIZE,
        },
        0x1300009a => FrameKind0104::Fixed {
            size: PcSkillAddRequest0104::SIZE,
        },
        0x1300009b => FrameKind0104::Fixed {
            size: PcSkillDelRequest0104::SIZE,
        },
        0x1300009c => FrameKind0104::Fixed {
            size: PcSkillUseRequest0104::SIZE,
        },
        0x13000088 => FrameKind0104::Fixed {
            size: PcTimeToGoWarpRequest0104::SIZE,
        },
        0x13000087 => FrameKind0104::Fixed {
            size: PcTransportWarpRequest0104::SIZE,
        },
        0x130000a0 => FrameKind0104::Fixed {
            size: PcVehicleOffRequest0104::SIZE,
        },
        0x1300009f => FrameKind0104::Fixed {
            size: PcVehicleOnRequest0104::SIZE,
        },
        0x1300008c => FrameKind0104::Fixed {
            size: PcWarpChannelRequest0104::SIZE,
        },
        _ => return None,
    })
}
