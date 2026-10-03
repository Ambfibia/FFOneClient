// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x13000049 => FrameKind0104::Fixed {
            size: DotDamageOnoff0104::SIZE,
        },
        0x13000079 => FrameKind0104::Fixed {
            size: DotHealOnoff0104::SIZE,
        },
        0x1300006c => FrameKind0104::Fixed {
            size: GmKickPlayerRequest0104::SIZE,
        },
        0x1300006f => FrameKind0104::Fixed {
            size: GmPcAnnounceRequest0104::SIZE,
        },
        0x1300006e => FrameKind0104::Fixed {
            size: GmPcLocationRequest0104::SIZE,
        },
        0x13000072 => FrameKind0104::Fixed {
            size: GmPcMotdRegisterRequest0104::SIZE,
        },
        0x1300006b => FrameKind0104::Fixed {
            size: GmPcSetValueRequest0104::SIZE,
        },
        0x1300006a => FrameKind0104::Fixed {
            size: GmPcSpecialStateSwitchRequest0104::SIZE,
        },
        0x1300006d => FrameKind0104::Fixed {
            size: GmTargetPcTeleportRequest0104::SIZE,
        },
        0x13000075 => FrameKind0104::Fixed {
            size: LiveCheckReply0104::SIZE,
        },
        0x13000036 => FrameKind0104::Fixed {
            size: AcceptMakeBuddyRequest0104::SIZE,
        },
        0x13000065 => FrameKind0104::Fixed {
            size: BarkerRequest0104::SIZE,
        },
        0x13000015 => FrameKind0104::Fixed {
            size: ChargeNanoStaminaRequest0104::SIZE,
        },
        0x1300005e => FrameKind0104::Fixed {
            size: EpGetRingRequest0104::SIZE,
        },
        0x1300005d => FrameKind0104::Fixed {
            size: EpRaceCancelRequest0104::SIZE,
        },
        0x1300005c => FrameKind0104::Fixed {
            size: EpRaceEndRequest0104::SIZE,
        },
        0x1300005b => FrameKind0104::Fixed {
            size: EpRaceStartRequest0104::SIZE,
        },
        0x13000059 => FrameKind0104::Fixed {
            size: EpRankGetDetailRequest0104::SIZE,
        },
        0x13000058 => FrameKind0104::Fixed {
            size: EpRankGetListRequest0104::SIZE,
        },
        0x1300005a => FrameKind0104::Fixed {
            size: EpRankGetPcInfoRequest0104::SIZE,
        },
        0x13000055 => FrameKind0104::Fixed {
            size: GetBuddyLocationRequest0104::SIZE,
        },
        0x1300003c => FrameKind0104::Fixed {
            size: GetBuddyStateRequest0104::SIZE,
        },
        0x13000039 => FrameKind0104::Fixed {
            size: GetBuddyStyleRequest0104::SIZE,
        },
        0x13000053 => FrameKind0104::Fixed {
            size: GetGroupStyleRequest0104::SIZE,
        },
        0x13000052 => FrameKind0104::Fixed {
            size: GetMemberStyleRequest0104::SIZE,
        },
        0x1300005f => FrameKind0104::Fixed {
            size: ImChangeSwitchStatusRequest0104::SIZE,
        },
        0x13000047 => FrameKind0104::Fixed {
            size: ItemChestOpenRequest0104::SIZE,
        },
        0x1300000a => FrameKind0104::Fixed {
            size: ItemMoveRequest0104::SIZE,
        },
        0x13000073 => FrameKind0104::Fixed {
            size: ItemUseRequest0104::SIZE,
        },
        0x1300000f => FrameKind0104::Fixed {
            size: NanoActiveRequest0104::SIZE,
        },
        0x1300000d => FrameKind0104::Fixed {
            size: NanoEquipRequest0104::SIZE,
        },
        0x13000011 => FrameKind0104::Counted {
            header_size: NanoSkillUseRequest0104::SIZE,
            count_offset: 16,
            count_width: 4,
            trailer_size: 4,
        },
        0x13000010 => FrameKind0104::Fixed {
            size: NanoTuneRequest0104::SIZE,
        },
        0x1300000e => FrameKind0104::Fixed {
            size: NanoUnequipRequest0104::SIZE,
        },
        0x13000056 => FrameKind0104::Fixed {
            size: NpcGroupSummonRequest0104::SIZE,
        },
        0x13000078 => FrameKind0104::Fixed {
            size: NpcInteractionRequest0104::SIZE,
        },
        0x13000045 => FrameKind0104::Fixed {
            size: NpcSummonRequest0104::SIZE,
        },
        0x13000046 => FrameKind0104::Fixed {
            size: NpcUnsummonRequest0104::SIZE,
        },
        0x13000006 => FrameKind0104::Counted {
            header_size: PcAttackNpcsRequest0104::SIZE,
            count_offset: 0,
            count_width: 4,
            trailer_size: 4,
        },
        0x13000050 => FrameKind0104::Fixed {
            size: PcAvatarEmotesChatRequest0104::SIZE,
        },
        0x1300002f => FrameKind0104::Fixed {
            size: PcBankCloseRequest0104::SIZE,
        },
        0x1300002e => FrameKind0104::Fixed {
            size: PcBankOpenRequest0104::SIZE,
        },
        0x13000051 => FrameKind0104::Fixed {
            size: PcBuddyWarpRequest0104::SIZE,
        },
        0x13000054 => FrameKind0104::Fixed {
            size: PcChangeMentorRequest0104::SIZE,
        },
        0x13000033 => FrameKind0104::Fixed {
            size: PcCombatBeginRequest0104::SIZE,
        },
        0x13000034 => FrameKind0104::Fixed {
            size: PcCombatEndRequest0104::SIZE,
        },
        0x1300007e => FrameKind0104::Fixed {
            size: PcDeleteEmailRequest0104::SIZE,
        },
        0x1300007b => FrameKind0104::Fixed {
            size: PcEmailUpdateCheckRequest0104::SIZE,
        },
        0x13000001 => FrameKind0104::Fixed {
            size: PcEnterRequest0104::SIZE,
        },
        0x13000002 => FrameKind0104::Fixed {
            size: PcExitRequest0104::SIZE,
        },
        0x1300001a => FrameKind0104::Fixed {
            size: PcGiveItemRequest0104::SIZE,
        },
        0x13000044 => FrameKind0104::Fixed {
            size: PcGiveNanoRequest0104::SIZE,
        },
        0x13000048 => FrameKind0104::Fixed {
            size: PcGiveNanoSkillRequest0104::SIZE,
        },
        0x13000014 => FrameKind0104::Fixed {
            size: PcGotoRequest0104::SIZE,
        },
        0x1300001f => FrameKind0104::Fixed {
            size: PcGrenadeStyleFireRequest0104::SIZE,
        },
        0x13000020 => FrameKind0104::Fixed {
            size: PcGrenadeStyleHitRequest0104::SIZE,
        },
        0x1300001e => FrameKind0104::Fixed {
            size: PcGrenadeStyleReadyRequest0104::SIZE,
        },
        0x1300004c => FrameKind0104::Fixed {
            size: PcGroupInviteRequest0104::SIZE,
        },
        0x1300004d => FrameKind0104::Fixed {
            size: PcGroupInviteRefuseRequest0104::SIZE,
        },
        0x1300004e => FrameKind0104::Fixed {
            size: PcGroupJoinRequest0104::SIZE,
        },
        0x1300004f => FrameKind0104::Fixed {
            size: PcGroupLeaveRequest0104::SIZE,
        },
        0x13000019 => FrameKind0104::Fixed {
            size: PcItemDeleteRequest0104::SIZE,
        },
        0x13000005 => FrameKind0104::Fixed {
            size: PcJumpRequest0104::SIZE,
        },
        0x1300003d => FrameKind0104::Fixed {
            size: PcJumppadRequest0104::SIZE,
        },
        0x13000016 => FrameKind0104::Fixed {
            size: PcKillQuestNpcsRequest0104::SIZE,
        },
        0x1300003e => FrameKind0104::Fixed {
            size: PcLauncherRequest0104::SIZE,
        },
        0x13000043 => FrameKind0104::Fixed {
            size: PcMapWarpRequest0104::SIZE,
        },
        0x13000076 => FrameKind0104::Fixed {
            size: PcMissionCompleteRequest0104::SIZE,
        },
        0x13000003 => FrameKind0104::Fixed {
            size: PcMoveRequest0104::SIZE,
        },
        0x13000040 => FrameKind0104::Fixed {
            size: PcMoveplatformRequest0104::SIZE,
        },
        0x13000062 => FrameKind0104::Fixed {
            size: PcMovetransportationRequest0104::SIZE,
        },
        0x13000021 => FrameKind0104::Fixed {
            size: PcNanoCreateRequest0104::SIZE,
        },
        0x1300007c => FrameKind0104::Fixed {
            size: PcReadEmailRequest0104::SIZE,
        },
        0x1300007d => FrameKind0104::Fixed {
            size: PcRecvEmailPageListRequest0104::SIZE,
        },
        0x13000009 => FrameKind0104::Fixed {
            size: PcRegenRequest0104::SIZE,
        },
        0x1300001c => FrameKind0104::Fixed {
            size: PcRocketStyleFireRequest0104::SIZE,
        },
        0x1300001d => FrameKind0104::Counted {
            header_size: PcRocketStyleHitRequest0104::SIZE,
            count_offset: 16,
            count_width: 4,
            trailer_size: 8,
        },
        0x1300001b => FrameKind0104::Fixed {
            size: PcRocketStyleReadyRequest0104::SIZE,
        },
        0x1300007f => FrameKind0104::Fixed {
            size: PcSendEmailRequest0104::SIZE,
        },
        0x13000041 => FrameKind0104::Fixed {
            size: PcSlopeRequest0104::SIZE,
        },
        0x1300007a => FrameKind0104::Fixed {
            size: PcSpecialStateSwitchRequest0104::SIZE,
        },
        0x13000042 => FrameKind0104::Fixed {
            size: PcStateChangeRequest0104::SIZE,
        },
        0x13000004 => FrameKind0104::Fixed {
            size: PcStopRequest0104::SIZE,
        },
        0x13000077 => FrameKind0104::Fixed {
            size: PcTaskCompleteRequest0104::SIZE,
        },
        0x13000013 => FrameKind0104::Fixed {
            size: PcTaskContinueRequest0104::SIZE,
        },
        0x1300000c => FrameKind0104::Fixed {
            size: PcTaskEndRequest0104::SIZE,
        },
        0x1300000b => FrameKind0104::Fixed {
            size: PcTaskStartRequest0104::SIZE,
        },
        0x13000012 => FrameKind0104::Fixed {
            size: PcTaskStopRequest0104::SIZE,
        },
        0x1300002c => FrameKind0104::Fixed {
            size: PcTradeCashRegisterRequest0104::SIZE,
        },
        0x13000027 => FrameKind0104::Fixed {
            size: PcTradeConfirmRequest0104::SIZE,
        },
        0x13000029 => FrameKind0104::Fixed {
            size: PcTradeConfirmAbortRequest0104::SIZE,
        },
        0x13000028 => FrameKind0104::Fixed {
            size: PcTradeConfirmCancelRequest0104::SIZE,
        },
        0x1300002d => FrameKind0104::Fixed {
            size: PcTradeEmotesChatRequest0104::SIZE,
        },
        0x1300002a => FrameKind0104::Fixed {
            size: PcTradeItemRegisterRequest0104::SIZE,
        },
        0x1300002b => FrameKind0104::Fixed {
            size: PcTradeItemUnregisterRequest0104::SIZE,
        },
        0x13000022 => FrameKind0104::Fixed {
            size: PcTradeOfferRequest0104::SIZE,
        },
        0x13000026 => FrameKind0104::Fixed {
            size: PcTradeOfferAbortRequest0104::SIZE,
        },
        0x13000024 => FrameKind0104::Fixed {
            size: PcTradeOfferAcceptRequest0104::SIZE,
        },
        0x13000023 => FrameKind0104::Fixed {
            size: PcTradeOfferCancelRequest0104::SIZE,
        },
        0x13000025 => FrameKind0104::Fixed {
            size: PcTradeOfferRefusalRequest0104::SIZE,
        },
        0x1300004a => FrameKind0104::Fixed {
            size: PcVendorBatteryBuyRequest0104::SIZE,
        },
        0x13000017 => FrameKind0104::Fixed {
            size: PcVendorItemBuyRequest0104::SIZE,
        },
        0x13000032 => FrameKind0104::Fixed {
            size: PcVendorItemRestoreBuyRequest0104::SIZE,
        },
        0x13000018 => FrameKind0104::Fixed {
            size: PcVendorItemSellRequest0104::SIZE,
        },
        0x13000030 => FrameKind0104::Fixed {
            size: PcVendorStartRequest0104::SIZE,
        },
        0x13000031 => FrameKind0104::Fixed {
            size: PcVendorTableUpdateRequest0104::SIZE,
        },
        0x13000057 => FrameKind0104::Fixed {
            size: PcWarpToPcRequest0104::SIZE,
        },
        0x1300004b => FrameKind0104::Fixed {
            size: PcWarpUseNpcRequest0104::SIZE,
        },
        0x13000069 => FrameKind0104::Fixed {
            size: PcWarpUseTransportationRequest0104::SIZE,
        },
        0x1300003f => FrameKind0104::Fixed {
            size: PcZiplineRequest0104::SIZE,
        },
        0x13000071 => FrameKind0104::Fixed {
            size: RegistRxcomRequest0104::SIZE,
        },
        0x13000068 => FrameKind0104::Fixed {
            size: RegistTransportationLocationRequest0104::SIZE,
        },
        0x1300003b => FrameKind0104::Fixed {
            size: RemoveBuddyRequest0104::SIZE,
        },
        0x13000035 => FrameKind0104::Fixed {
            size: RequestMakeBuddyRequest0104::SIZE,
        },
        0x13000063 => FrameKind0104::Fixed {
            size: SendAllGroupFreechatMessageRequest0104::SIZE,
        },
        0x13000066 => FrameKind0104::Fixed {
            size: SendAllGroupMenuchatMessageRequest0104::SIZE,
        },
        0x13000064 => FrameKind0104::Fixed {
            size: SendAnyGroupFreechatMessageRequest0104::SIZE,
        },
        0x13000067 => FrameKind0104::Fixed {
            size: SendAnyGroupMenuchatMessageRequest0104::SIZE,
        },
        0x13000037 => FrameKind0104::Fixed {
            size: SendBuddyFreechatMessageRequest0104::SIZE,
        },
        0x13000038 => FrameKind0104::Fixed {
            size: SendBuddyMenuchatMessageRequest0104::SIZE,
        },
        0x13000007 => FrameKind0104::Fixed {
            size: SendFreechatMessageRequest0104::SIZE,
        },
        0x13000008 => FrameKind0104::Fixed {
            size: SendMenuchatMessageRequest0104::SIZE,
        },
        0x1300003a => FrameKind0104::Fixed {
            size: SetBuddyBlockRequest0104::SIZE,
        },
        0x13000070 => FrameKind0104::Fixed {
            size: SetPcBlockRequest0104::SIZE,
        },
        0x13000060 => FrameKind0104::Fixed {
            size: ShinyPickupRequest0104::SIZE,
        },
        0x13000061 => FrameKind0104::Fixed {
            size: ShinySummonRequest0104::SIZE,
        },
        0x13000074 => FrameKind0104::Fixed {
            size: WarpUseRecallRequest0104::SIZE,
        },
        _ => return None,
    })
}
