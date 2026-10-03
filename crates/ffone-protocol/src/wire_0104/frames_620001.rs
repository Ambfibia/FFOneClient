// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x310000c2 => FrameKind0104::Fixed {
            size: AnnounceMsg0104::SIZE,
        },
        0x310000b1 => FrameKind0104::Fixed {
            size: AroundDelShiny0104::SIZE,
        },
        0x310000a0 => FrameKind0104::Fixed {
            size: AroundDelTransportation0104::SIZE,
        },
        0x31000085 => FrameKind0104::Fixed {
            size: CharacterAttackCharacters0104::SIZE,
        },
        0x310000c6 => FrameKind0104::Fixed {
            size: GmPcChangeValue0104::SIZE,
        },
        0x310000c8 => FrameKind0104::Fixed {
            size: GmPcAnnounceReply0104::SIZE,
        },
        0x310000c7 => FrameKind0104::Fixed {
            size: GmPcLocationReply0104::SIZE,
        },
        0x310000c5 => FrameKind0104::Fixed {
            size: GmPcSetValueReply0104::SIZE,
        },
        0x3100009a => FrameKind0104::Fixed {
            size: InstanceMapInfo0104::SIZE,
        },
        0x31000084 => FrameKind0104::Fixed {
            size: NpcBulletStyleHit0104::SIZE,
        },
        0x31000083 => FrameKind0104::Fixed {
            size: NpcGrenadeStyleFire0104::SIZE,
        },
        0x31000082 => FrameKind0104::Fixed {
            size: NpcRocketStyleFire0104::SIZE,
        },
        0x310000da => FrameKind0104::Fixed {
            size: PcBroomstickMove0104::SIZE,
        },
        0x310000df => FrameKind0104::Fixed {
            size: PcBuffUpdate0104::SIZE,
        },
        0x310000f4 => FrameKind0104::Fixed {
            size: PcEvent0104::SIZE,
        },
        0x31000086 => FrameKind0104::Fixed {
            size: PcGroupInvite0104::SIZE,
        },
        0x31000087 => FrameKind0104::Fixed {
            size: PcGroupInviteFailure0104::SIZE,
        },
        0x31000088 => FrameKind0104::Fixed {
            size: PcGroupInviteRefuse0104::SIZE,
        },
        0x31000089 => FrameKind0104::Manual {
            header_size: PcGroupJoin0104::SIZE,
        },
        0x3100008a => FrameKind0104::Fixed {
            size: PcGroupJoinFailure0104::SIZE,
        },
        0x3100008b => FrameKind0104::Fixed {
            size: PcGroupJoinSuccess0104::SIZE,
        },
        0x3100008c => FrameKind0104::Manual {
            header_size: PcGroupLeave0104::SIZE,
        },
        0x3100008d => FrameKind0104::Fixed {
            size: PcGroupLeaveFailure0104::SIZE,
        },
        0x3100008e => FrameKind0104::Fixed {
            size: PcGroupLeaveSuccess0104::SIZE,
        },
        0x3100008f => FrameKind0104::Manual {
            header_size: PcGroupMemberInfo0104::SIZE,
        },
        0x310000cf => FrameKind0104::Fixed {
            size: PcInvenFullMsg0104::SIZE,
        },
        0x310000d4 => FrameKind0104::Fixed {
            size: PcItemUse0104::SIZE,
        },
        0x310000d1 => FrameKind0104::Fixed {
            size: PcMotdLogin0104::SIZE,
        },
        0x310000b4 => FrameKind0104::Fixed {
            size: PcMovetransportation0104::SIZE,
        },
        0x31000099 => FrameKind0104::Fixed {
            size: PcRegen0104::SIZE,
        },
        0x310000d9 => FrameKind0104::Fixed {
            size: PcRiding0104::SIZE,
        },
        0x310000c4 => FrameKind0104::Fixed {
            size: PcSpecialStateChange0104::SIZE,
        },
        0x310000ed => FrameKind0104::Fixed {
            size: PcSuddenDead0104::SIZE,
        },
        0x310000b9 => FrameKind0104::Fixed {
            size: BarkerReply0104::SIZE,
        },
        0x310000fa => FrameKind0104::Fixed {
            size: ChannelInfoReply0104::SIZE,
        },
        0x310000ab => FrameKind0104::Fixed {
            size: EpGetRingFailure0104::SIZE,
        },
        0x310000aa => FrameKind0104::Fixed {
            size: EpGetRingSuccess0104::SIZE,
        },
        0x310000a9 => FrameKind0104::Fixed {
            size: EpRaceCancelFailure0104::SIZE,
        },
        0x310000a8 => FrameKind0104::Fixed {
            size: EpRaceCancelSuccess0104::SIZE,
        },
        0x310000a7 => FrameKind0104::Fixed {
            size: EpRaceEndFailure0104::SIZE,
        },
        0x310000a6 => FrameKind0104::Fixed {
            size: EpRaceEndSuccess0104::SIZE,
        },
        0x310000a5 => FrameKind0104::Fixed {
            size: EpRaceStartFailure0104::SIZE,
        },
        0x310000a4 => FrameKind0104::Fixed {
            size: EpRaceStartSuccess0104::SIZE,
        },
        0x310000a2 => FrameKind0104::Fixed {
            size: EpRankDetailReply0104::SIZE,
        },
        0x310000a1 => FrameKind0104::Fixed {
            size: EpRankListReply0104::SIZE,
        },
        0x310000a3 => FrameKind0104::Fixed {
            size: EpRankPcInfoReply0104::SIZE,
        },
        0x310000d6 => FrameKind0104::Fixed {
            size: GetBuddyLocationFailure0104::SIZE,
        },
        0x310000d5 => FrameKind0104::Fixed {
            size: GetBuddyLocationSuccess0104::SIZE,
        },
        0x31000097 => FrameKind0104::Fixed {
            size: GetGroupStyleFailure0104::SIZE,
        },
        0x31000098 => FrameKind0104::Fixed {
            size: GetGroupStyleSuccess0104::SIZE,
        },
        0x31000095 => FrameKind0104::Fixed {
            size: GetMemberStyleFailure0104::SIZE,
        },
        0x31000096 => FrameKind0104::Fixed {
            size: GetMemberStyleSuccess0104::SIZE,
        },
        0x310000ee => FrameKind0104::Fixed {
            size: GmTargetPcSpecialStateOnoffSuccess0104::SIZE,
        },
        0x310000ac => FrameKind0104::Fixed {
            size: ImChangeSwitchStatusReply0104::SIZE,
        },
        0x310000f0 => FrameKind0104::Fixed {
            size: NpcGroupInviteFailure0104::SIZE,
        },
        0x310000f1 => FrameKind0104::Fixed {
            size: NpcGroupInviteSuccess0104::SIZE,
        },
        0x310000f2 => FrameKind0104::Fixed {
            size: NpcGroupKickFailure0104::SIZE,
        },
        0x310000f3 => FrameKind0104::Fixed {
            size: NpcGroupKickSuccess0104::SIZE,
        },
        0x31000092 => FrameKind0104::Fixed {
            size: PcAvatarEmotesChatReply0104::SIZE,
        },
        0x310000c9 => FrameKind0104::Fixed {
            size: PcBuddyWarpFailure0104::SIZE,
        },
        0x310000db => FrameKind0104::Fixed {
            size: PcBuddyWarpOtherShardSuccess0104::SIZE,
        },
        0x310000ca => FrameKind0104::Fixed {
            size: PcChangeLevelReply0104::SIZE,
        },
        0x31000094 => FrameKind0104::Fixed {
            size: PcChangeMentorFailure0104::SIZE,
        },
        0x31000093 => FrameKind0104::Fixed {
            size: PcChangeMentorSuccess0104::SIZE,
        },
        0x310000fb => FrameKind0104::Fixed {
            size: PcChannelNumReply0104::SIZE,
        },
        0x310000e6 => FrameKind0104::Fixed {
            size: PcDeleteEmailFailure0104::SIZE,
        },
        0x310000e5 => FrameKind0104::Fixed {
            size: PcDeleteEmailSuccess0104::SIZE,
        },
        0x310000dd => FrameKind0104::Fixed {
            size: PcExitDuplicateReply0104::SIZE,
        },
        0x310000ff => FrameKind0104::Fixed {
            size: PcFindNameMakeBuddyFailure0104::SIZE,
        },
        0x310000fe => FrameKind0104::Fixed {
            size: PcFindNameMakeBuddySuccess0104::SIZE,
        },
        0x310000d2 => FrameKind0104::Fixed {
            size: PcItemUseFailure0104::SIZE,
        },
        0x310000d3 => FrameKind0104::Counted {
            header_size: PcItemUseSuccess0104::SIZE,
            count_offset: 32,
            count_width: 4,
            trailer_size: 16,
        },
        0x310000f9 => FrameKind0104::Fixed {
            size: PcLoadingCompleteSuccess0104::SIZE,
        },
        0x310000de => FrameKind0104::Fixed {
            size: PcMissionCompleteSuccess0104::SIZE,
        },
        0x310000e0 => FrameKind0104::Fixed {
            size: PcNewEmailReply0104::SIZE,
        },
        0x310000e2 => FrameKind0104::Fixed {
            size: PcReadEmailFailure0104::SIZE,
        },
        0x310000e1 => FrameKind0104::Fixed {
            size: PcReadEmailSuccess0104::SIZE,
        },
        0x310000ec => FrameKind0104::Fixed {
            size: PcRecvEmailCandyFailure0104::SIZE,
        },
        0x310000eb => FrameKind0104::Fixed {
            size: PcRecvEmailCandySuccess0104::SIZE,
        },
        0x310000f8 => FrameKind0104::Fixed {
            size: PcRecvEmailItemAllFailure0104::SIZE,
        },
        0x310000f7 => FrameKind0104::Fixed {
            size: PcRecvEmailItemAllSuccess0104::SIZE,
        },
        0x310000ea => FrameKind0104::Fixed {
            size: PcRecvEmailItemFailure0104::SIZE,
        },
        0x310000e9 => FrameKind0104::Fixed {
            size: PcRecvEmailItemSuccess0104::SIZE,
        },
        0x310000e4 => FrameKind0104::Fixed {
            size: PcRecvEmailPageListFailure0104::SIZE,
        },
        0x310000e3 => FrameKind0104::Fixed {
            size: PcRecvEmailPageListSuccess0104::SIZE,
        },
        0x310000be => FrameKind0104::Fixed {
            size: PcRegistTransportationLocationFailure0104::SIZE,
        },
        0x310000bf => FrameKind0104::Fixed {
            size: PcRegistTransportationLocationSuccess0104::SIZE,
        },
        0x310000d7 => FrameKind0104::Fixed {
            size: PcRidingFailure0104::SIZE,
        },
        0x310000d8 => FrameKind0104::Fixed {
            size: PcRidingSuccess0104::SIZE,
        },
        0x310000e8 => FrameKind0104::Fixed {
            size: PcSendEmailFailure0104::SIZE,
        },
        0x310000e7 => FrameKind0104::Fixed {
            size: PcSendEmailSuccess0104::SIZE,
        },
        0x310000ef => FrameKind0104::Fixed {
            size: PcSetCurrentMissionIdReply0104::SIZE,
        },
        0x310000c3 => FrameKind0104::Fixed {
            size: PcSpecialStateSwitchSuccess0104::SIZE,
        },
        0x310000f6 => FrameKind0104::Fixed {
            size: PcTradeEmotesChatFailure0104::SIZE,
        },
        0x310000f5 => FrameKind0104::Fixed {
            size: PcTransportWarpSuccess0104::SIZE,
        },
        0x31000081 => FrameKind0104::Fixed {
            size: PcVendorBatteryBuyFailure0104::SIZE,
        },
        0x31000080 => FrameKind0104::Fixed {
            size: PcVendorBatteryBuySuccess0104::SIZE,
        },
        0x310000fc => FrameKind0104::Fixed {
            size: PcWarpChannelFailure0104::SIZE,
        },
        0x310000fd => FrameKind0104::Fixed {
            size: PcWarpChannelSuccess0104::SIZE,
        },
        0x31000091 => FrameKind0104::Fixed {
            size: PcWarpUseNpcFailure0104::SIZE,
        },
        0x31000090 => FrameKind0104::Fixed {
            size: PcWarpUseNpcSuccess0104::SIZE,
        },
        0x310000c0 => FrameKind0104::Fixed {
            size: PcWarpUseTransportationFailure0104::SIZE,
        },
        0x310000c1 => FrameKind0104::Fixed {
            size: PcWarpUseTransportationSuccess0104::SIZE,
        },
        0x310000cd => FrameKind0104::Fixed {
            size: RegistRxcomReply0104::SIZE,
        },
        0x310000ce => FrameKind0104::Fixed {
            size: RegistRxcomFailure0104::SIZE,
        },
        0x310000b6 => FrameKind0104::Fixed {
            size: SendAllGroupFreechatMessageFailure0104::SIZE,
        },
        0x310000b5 => FrameKind0104::Fixed {
            size: SendAllGroupFreechatMessageSuccess0104::SIZE,
        },
        0x310000bb => FrameKind0104::Fixed {
            size: SendAllGroupMenuchatMessageFailure0104::SIZE,
        },
        0x310000ba => FrameKind0104::Fixed {
            size: SendAllGroupMenuchatMessageSuccess0104::SIZE,
        },
        0x310000b8 => FrameKind0104::Fixed {
            size: SendAnyGroupFreechatMessageFailure0104::SIZE,
        },
        0x310000b7 => FrameKind0104::Fixed {
            size: SendAnyGroupFreechatMessageSuccess0104::SIZE,
        },
        0x310000bd => FrameKind0104::Fixed {
            size: SendAnyGroupMenuchatMessageFailure0104::SIZE,
        },
        0x310000bc => FrameKind0104::Fixed {
            size: SendAnyGroupMenuchatMessageSuccess0104::SIZE,
        },
        0x310000cc => FrameKind0104::Fixed {
            size: SetPcBlockFailure0104::SIZE,
        },
        0x310000cb => FrameKind0104::Fixed {
            size: SetPcBlockSuccess0104::SIZE,
        },
        0x310000b2 => FrameKind0104::Fixed {
            size: ShinyPickupFailure0104::SIZE,
        },
        0x310000b3 => FrameKind0104::Fixed {
            size: ShinyPickupSuccess0104::SIZE,
        },
        0x310000dc => FrameKind0104::Fixed {
            size: WarpUseRecallFailure0104::SIZE,
        },
        0x310000d0 => FrameKind0104::Fixed {
            size: LiveCheck0104::SIZE,
        },
        0x310000b0 => FrameKind0104::Fixed {
            size: ShinyAround0104::SIZE,
        },
        0x310000ad => FrameKind0104::Fixed {
            size: ShinyEnter0104::SIZE,
        },
        0x310000ae => FrameKind0104::Fixed {
            size: ShinyExit0104::SIZE,
        },
        0x310000af => FrameKind0104::Fixed {
            size: ShinyNew0104::SIZE,
        },
        0x3100009f => FrameKind0104::Fixed {
            size: TransportationAround0104::SIZE,
        },
        0x3100009b => FrameKind0104::Fixed {
            size: TransportationEnter0104::SIZE,
        },
        0x3100009c => FrameKind0104::Fixed {
            size: TransportationExit0104::SIZE,
        },
        0x3100009d => FrameKind0104::Fixed {
            size: TransportationMove0104::SIZE,
        },
        0x3100009e => FrameKind0104::Fixed {
            size: TransportationNew0104::SIZE,
        },
        _ => return None,
    })
}
