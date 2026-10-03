// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x3100012c => FrameKind0104::Fixed {
            size: GmRewardRateSuccess0104::SIZE,
        },
        0x31000104 => FrameKind0104::Fixed {
            size: NpcAttackChars0104::SIZE,
        },
        0x31000103 => FrameKind0104::Counted {
            header_size: PcAttackChars0104::SIZE,
            count_offset: 4,
            count_width: 4,
            trailer_size: 24,
        },
        0x31000102 => FrameKind0104::Counted {
            header_size: PcAttackCharsSuccess0104::SIZE,
            count_offset: 4,
            count_width: 4,
            trailer_size: 24,
        },
        0x31000121 => FrameKind0104::Fixed {
            size: PcBelt0104::SIZE,
        },
        0x31000118 => FrameKind0104::Fixed {
            size: PcCashBuffUpdate0104::SIZE,
        },
        0x31000129 => FrameKind0104::Counted {
            header_size: PcDeleteTimeLimitItem0104::SIZE,
            count_offset: 0,
            count_width: 4,
            trailer_size: 8,
        },
        0x31000126 => FrameKind0104::Fixed {
            size: PcQuickSlotInfo0104::SIZE,
        },
        0x31000120 => FrameKind0104::Fixed {
            size: PcRope0104::SIZE,
        },
        0x3100011f => FrameKind0104::Fixed {
            size: PcSkillUse0104::SIZE,
        },
        0x3100010a => FrameKind0104::Fixed {
            size: PcStreetstallCancelFailure0104::SIZE,
        },
        0x31000109 => FrameKind0104::Fixed {
            size: PcStreetstallCancelSuccess0104::SIZE,
        },
        0x31000115 => FrameKind0104::Fixed {
            size: PcStreetstallItemBuyFailure0104::SIZE,
        },
        0x31000113 => FrameKind0104::Fixed {
            size: PcStreetstallItemBuySuccessBuyer0104::SIZE,
        },
        0x31000114 => FrameKind0104::Fixed {
            size: PcStreetstallItemBuySuccessSeller0104::SIZE,
        },
        0x31000111 => FrameKind0104::Fixed {
            size: PcStreetstallItemListReply0104::SIZE,
        },
        0x31000112 => FrameKind0104::Fixed {
            size: PcStreetstallItemListFailure0104::SIZE,
        },
        0x31000108 => FrameKind0104::Fixed {
            size: PcStreetstallReadyFailure0104::SIZE,
        },
        0x31000107 => FrameKind0104::Fixed {
            size: PcStreetstallReadySuccess0104::SIZE,
        },
        0x3100010c => FrameKind0104::Fixed {
            size: PcStreetstallRegistItemFailure0104::SIZE,
        },
        0x3100010b => FrameKind0104::Fixed {
            size: PcStreetstallRegistItemSuccess0104::SIZE,
        },
        0x31000110 => FrameKind0104::Fixed {
            size: PcStreetstallSaleStartFailure0104::SIZE,
        },
        0x3100010f => FrameKind0104::Fixed {
            size: PcStreetstallSaleStartSuccess0104::SIZE,
        },
        0x3100010e => FrameKind0104::Fixed {
            size: PcStreetstallUnregistItemFailure0104::SIZE,
        },
        0x3100010d => FrameKind0104::Fixed {
            size: PcStreetstallUnregistItemSuccess0104::SIZE,
        },
        0x31000125 => FrameKind0104::Fixed {
            size: PcVehicleOffFailure0104::SIZE,
        },
        0x31000124 => FrameKind0104::Fixed {
            size: PcVehicleOffSuccess0104::SIZE,
        },
        0x31000123 => FrameKind0104::Fixed {
            size: PcVehicleOnFailure0104::SIZE,
        },
        0x31000122 => FrameKind0104::Fixed {
            size: PcVehicleOnSuccess0104::SIZE,
        },
        0x31000134 => FrameKind0104::Fixed {
            size: NanoBookSubsetReply0104::SIZE,
        },
        0x31000101 => FrameKind0104::Fixed {
            size: PcBuddyWarpSameShardSuccess0104::SIZE,
        },
        0x31000105 => FrameKind0104::Fixed {
            size: PcChangeLevelSuccess0104::SIZE,
        },
        0x3100012b => FrameKind0104::Fixed {
            size: PcDisassembleItemFailure0104::SIZE,
        },
        0x3100012a => FrameKind0104::Fixed {
            size: PcDisassembleItemSuccess0104::SIZE,
        },
        0x31000100 => FrameKind0104::Fixed {
            size: PcFindNameAcceptBuddyFailure0104::SIZE,
        },
        0x31000117 => FrameKind0104::Fixed {
            size: PcItemCombinationFailure0104::SIZE,
        },
        0x31000116 => FrameKind0104::Fixed {
            size: PcItemCombinationSuccess0104::SIZE,
        },
        0x3100012e => FrameKind0104::Fixed {
            size: PcItemEnchantFailure0104::SIZE,
        },
        0x3100012d => FrameKind0104::Fixed {
            size: PcItemEnchantSuccess0104::SIZE,
        },
        0x31000106 => FrameKind0104::Fixed {
            size: PcNanoCreateReply0104::SIZE,
        },
        0x31000127 => FrameKind0104::Fixed {
            size: PcRegistQuickSlotFailure0104::SIZE,
        },
        0x31000128 => FrameKind0104::Fixed {
            size: PcRegistQuickSlotSuccess0104::SIZE,
        },
        0x3100011a => FrameKind0104::Fixed {
            size: PcSkillAddFailure0104::SIZE,
        },
        0x31000119 => FrameKind0104::Fixed {
            size: PcSkillAddSuccess0104::SIZE,
        },
        0x3100011c => FrameKind0104::Fixed {
            size: PcSkillDelFailure0104::SIZE,
        },
        0x3100011b => FrameKind0104::Fixed {
            size: PcSkillDelSuccess0104::SIZE,
        },
        0x3100011e => FrameKind0104::Fixed {
            size: PcSkillUseFailure0104::SIZE,
        },
        0x3100011d => FrameKind0104::Fixed {
            size: PcSkillUseSuccess0104::SIZE,
        },
        _ => return None,
    })
}
