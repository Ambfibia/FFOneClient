// Generated descriptor range; do not edit by hand.
use super::*;

pub(super) fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    Some(match packet_id {
        0x31000011 => FrameKind0104::Fixed {
            size: AroundDelNpc0104::SIZE,
        },
        0x31000010 => FrameKind0104::Fixed {
            size: AroundDelPc0104::SIZE,
        },
        0x31000060 => FrameKind0104::Fixed {
            size: CharTimeBuffTimeOut0104::SIZE,
        },
        0x3100007f => FrameKind0104::Manual {
            header_size: CharTimeBuffTimeTick0104::SIZE,
        },
        0x31000000 => FrameKind0104::Fixed {
            size: Error0104::SIZE,
        },
        0x3100002a => FrameKind0104::Fixed {
            size: NanoActive0104::SIZE,
        },
        0x3100002c => FrameKind0104::Fixed {
            size: NanoSkillUse0104::SIZE,
        },
        0x3100002b => FrameKind0104::Manual {
            header_size: NanoSkillUseSuccess0104::SIZE,
        },
        0x3100000f => FrameKind0104::Fixed {
            size: NpcAround0104::SIZE,
        },
        0x31000016 => FrameKind0104::Counted {
            header_size: NpcAttackPcs0104::SIZE,
            count_offset: 4,
            count_width: 4,
            trailer_size: 24,
        },
        0x3100000b => FrameKind0104::Fixed {
            size: NpcEnter0104::SIZE,
        },
        0x3100000c => FrameKind0104::Fixed {
            size: NpcExit0104::SIZE,
        },
        0x3100000d => FrameKind0104::Fixed {
            size: NpcMove0104::SIZE,
        },
        0x3100000e => FrameKind0104::Fixed {
            size: NpcNew0104::SIZE,
        },
        0x31000025 => FrameKind0104::Fixed {
            size: NpcSkillCancel0104::SIZE,
        },
        0x31000024 => FrameKind0104::Counted {
            header_size: NpcSkillCorruptionHit0104::SIZE,
            count_offset: 20,
            count_width: 4,
            trailer_size: 40,
        },
        0x31000023 => FrameKind0104::Fixed {
            size: NpcSkillCorruptionReady0104::SIZE,
        },
        0x31000021 => FrameKind0104::Fixed {
            size: NpcSkillFire0104::SIZE,
        },
        0x31000022 => FrameKind0104::Manual {
            header_size: NpcSkillHit0104::SIZE,
        },
        0x31000020 => FrameKind0104::Fixed {
            size: NpcSkillReady0104::SIZE,
        },
        0x31000007 => FrameKind0104::Fixed {
            size: PcAround0104::SIZE,
        },
        0x31000015 => FrameKind0104::Counted {
            header_size: PcAttackNpcs0104::SIZE,
            count_offset: 4,
            count_width: 4,
            trailer_size: 24,
        },
        0x31000014 => FrameKind0104::Counted {
            header_size: PcAttackNpcsSuccess0104::SIZE,
            count_offset: 4,
            count_width: 4,
            trailer_size: 24,
        },
        0x3100001b => FrameKind0104::Fixed {
            size: PcEquipChange0104::SIZE,
        },
        0x31000006 => FrameKind0104::Fixed {
            size: PcExit0104::SIZE,
        },
        0x31000040 => FrameKind0104::Fixed {
            size: PcGrenadeStyleFire0104::SIZE,
        },
        0x31000041 => FrameKind0104::Counted {
            header_size: PcGrenadeStyleHit0104::SIZE,
            count_offset: 20,
            count_width: 4,
            trailer_size: 24,
        },
        0x3100003e => FrameKind0104::Fixed {
            size: PcGrenadeStyleReady0104::SIZE,
        },
        0x3100001a => FrameKind0104::Fixed {
            size: PcItemMoveSuccess0104::SIZE,
        },
        0x3100000a => FrameKind0104::Fixed {
            size: PcJump0104::SIZE,
        },
        0x31000075 => FrameKind0104::Fixed {
            size: PcJumppad0104::SIZE,
        },
        0x31000076 => FrameKind0104::Fixed {
            size: PcLauncher0104::SIZE,
        },
        0x31000008 => FrameKind0104::Fixed {
            size: PcMove0104::SIZE,
        },
        0x31000078 => FrameKind0104::Fixed {
            size: PcMoveplatform0104::SIZE,
        },
        0x31000003 => FrameKind0104::Fixed {
            size: PcNew0104::SIZE,
        },
        0x3100003c => FrameKind0104::Fixed {
            size: PcRocketStyleFire0104::SIZE,
        },
        0x3100003d => FrameKind0104::Fixed {
            size: PcRocketStyleHit0104::SIZE,
        },
        0x3100003a => FrameKind0104::Fixed {
            size: PcRocketStyleReady0104::SIZE,
        },
        0x31000079 => FrameKind0104::Fixed {
            size: PcSlope0104::SIZE,
        },
        0x3100007a => FrameKind0104::Fixed {
            size: PcStateChange0104::SIZE,
        },
        0x31000009 => FrameKind0104::Fixed {
            size: PcStop0104::SIZE,
        },
        0x31000077 => FrameKind0104::Fixed {
            size: PcZipline0104::SIZE,
        },
        0x31000068 => FrameKind0104::Fixed {
            size: AcceptMakeBuddyFailure0104::SIZE,
        },
        0x31000067 => FrameKind0104::Fixed {
            size: AcceptMakeBuddySuccess0104::SIZE,
        },
        0x31000032 => FrameKind0104::Fixed {
            size: ChargeNanoStaminaReply0104::SIZE,
        },
        0x31000070 => FrameKind0104::Fixed {
            size: GetBuddyStateFailure0104::SIZE,
        },
        0x3100006f => FrameKind0104::Fixed {
            size: GetBuddyStateSuccess0104::SIZE,
        },
        0x3100006e => FrameKind0104::Fixed {
            size: GetBuddyStyleFailure0104::SIZE,
        },
        0x3100006d => FrameKind0104::Fixed {
            size: GetBuddyStyleSuccess0104::SIZE,
        },
        0x3100007e => FrameKind0104::Fixed {
            size: ItemChestOpenFailure0104::SIZE,
        },
        0x3100007d => FrameKind0104::Fixed {
            size: ItemChestOpenSuccess0104::SIZE,
        },
        0x31000028 => FrameKind0104::Fixed {
            size: NanoActiveSuccess0104::SIZE,
        },
        0x31000026 => FrameKind0104::Fixed {
            size: NanoEquipSuccess0104::SIZE,
        },
        0x31000055 => FrameKind0104::Fixed {
            size: NanoTuneFailure0104::SIZE,
        },
        0x31000029 => FrameKind0104::Fixed {
            size: NanoTuneSuccess0104::SIZE,
        },
        0x31000027 => FrameKind0104::Fixed {
            size: NanoUnequipSuccess0104::SIZE,
        },
        0x31000059 => FrameKind0104::Fixed {
            size: PcBankCloseFailure0104::SIZE,
        },
        0x31000058 => FrameKind0104::Fixed {
            size: PcBankCloseSuccess0104::SIZE,
        },
        0x31000057 => FrameKind0104::Fixed {
            size: PcBankOpenFailure0104::SIZE,
        },
        0x31000056 => FrameKind0104::Fixed {
            size: PcBankOpenSuccess0104::SIZE,
        },
        0x31000064 => FrameKind0104::Fixed {
            size: PcBuddylistInfoFailure0104::SIZE,
        },
        0x31000063 => FrameKind0104::Counted {
            header_size: PcBuddylistInfoSuccess0104::SIZE,
            count_offset: 13,
            count_width: 1,
            trailer_size: 72,
        },
        0x31000001 => FrameKind0104::Fixed {
            size: PcEnterFailure0104::SIZE,
        },
        0x31000002 => FrameKind0104::Fixed {
            size: PcEnterSuccess0104::SIZE,
        },
        0x31000004 => FrameKind0104::Fixed {
            size: PcExitFailure0104::SIZE,
        },
        0x31000005 => FrameKind0104::Fixed {
            size: PcExitSuccess0104::SIZE,
        },
        0x31000062 => FrameKind0104::Fixed {
            size: PcGiveItemFailure0104::SIZE,
        },
        0x31000061 => FrameKind0104::Fixed {
            size: PcGiveItemSuccess0104::SIZE,
        },
        0x31000031 => FrameKind0104::Fixed {
            size: PcGotoSuccess0104::SIZE,
        },
        0x3100003f => FrameKind0104::Fixed {
            size: PcGrenadeStyleFireSuccess0104::SIZE,
        },
        0x31000039 => FrameKind0104::Fixed {
            size: PcItemDeleteSuccess0104::SIZE,
        },
        0x31000034 => FrameKind0104::Fixed {
            size: PcKillQuestNpcsSuccess0104::SIZE,
        },
        0x31000054 => FrameKind0104::Fixed {
            size: PcNanoCreateFailure0104::SIZE,
        },
        0x31000053 => FrameKind0104::Fixed {
            size: PcNanoCreateSuccess0104::SIZE,
        },
        0x31000017 => FrameKind0104::Fixed {
            size: PcRegenSuccess0104::SIZE,
        },
        0x3100003b => FrameKind0104::Fixed {
            size: PcRocketStyleFireSuccess0104::SIZE,
        },
        0x31000030 => FrameKind0104::Fixed {
            size: PcTaskContinueFailure0104::SIZE,
        },
        0x3100002f => FrameKind0104::Fixed {
            size: PcTaskContinueSuccess0104::SIZE,
        },
        0x3100001f => FrameKind0104::Fixed {
            size: PcTaskEndFailure0104::SIZE,
        },
        0x3100001e => FrameKind0104::Fixed {
            size: PcTaskEndSuccess0104::SIZE,
        },
        0x3100001d => FrameKind0104::Fixed {
            size: PcTaskStartFailure0104::SIZE,
        },
        0x3100001c => FrameKind0104::Fixed {
            size: PcTaskStartSuccess0104::SIZE,
        },
        0x3100002e => FrameKind0104::Fixed {
            size: PcTaskStopFailure0104::SIZE,
        },
        0x3100002d => FrameKind0104::Fixed {
            size: PcTaskStopSuccess0104::SIZE,
        },
        0x31000033 => FrameKind0104::Fixed {
            size: PcTickReply0104::SIZE,
        },
        0x31000051 => FrameKind0104::Fixed {
            size: PcTradeCashRegisterFailure0104::SIZE,
        },
        0x31000050 => FrameKind0104::Fixed {
            size: PcTradeCashRegisterSuccess0104::SIZE,
        },
        0x31000047 => FrameKind0104::Fixed {
            size: PcTradeConfirmReply0104::SIZE,
        },
        0x31000049 => FrameKind0104::Fixed {
            size: PcTradeConfirmAbortReply0104::SIZE,
        },
        0x31000048 => FrameKind0104::Fixed {
            size: PcTradeConfirmCancelReply0104::SIZE,
        },
        0x3100004b => FrameKind0104::Fixed {
            size: PcTradeConfirmFailure0104::SIZE,
        },
        0x3100004a => FrameKind0104::Fixed {
            size: PcTradeConfirmSuccess0104::SIZE,
        },
        0x31000052 => FrameKind0104::Fixed {
            size: PcTradeEmotesChatReply0104::SIZE,
        },
        0x3100004d => FrameKind0104::Fixed {
            size: PcTradeItemRegisterFailure0104::SIZE,
        },
        0x3100004c => FrameKind0104::Fixed {
            size: PcTradeItemRegisterSuccess0104::SIZE,
        },
        0x3100004f => FrameKind0104::Fixed {
            size: PcTradeItemUnregisterFailure0104::SIZE,
        },
        0x3100004e => FrameKind0104::Fixed {
            size: PcTradeItemUnregisterSuccess0104::SIZE,
        },
        0x31000042 => FrameKind0104::Fixed {
            size: PcTradeOfferReply0104::SIZE,
        },
        0x31000046 => FrameKind0104::Fixed {
            size: PcTradeOfferAbortReply0104::SIZE,
        },
        0x31000043 => FrameKind0104::Fixed {
            size: PcTradeOfferCancelReply0104::SIZE,
        },
        0x31000045 => FrameKind0104::Fixed {
            size: PcTradeOfferRefusalReply0104::SIZE,
        },
        0x31000044 => FrameKind0104::Fixed {
            size: PcTradeOfferSuccess0104::SIZE,
        },
        0x31000036 => FrameKind0104::Fixed {
            size: PcVendorItemBuyFailure0104::SIZE,
        },
        0x31000035 => FrameKind0104::Fixed {
            size: PcVendorItemBuySuccess0104::SIZE,
        },
        0x3100005f => FrameKind0104::Fixed {
            size: PcVendorItemRestoreBuyFailure0104::SIZE,
        },
        0x3100005e => FrameKind0104::Fixed {
            size: PcVendorItemRestoreBuySuccess0104::SIZE,
        },
        0x31000038 => FrameKind0104::Fixed {
            size: PcVendorItemSellFailure0104::SIZE,
        },
        0x31000037 => FrameKind0104::Fixed {
            size: PcVendorItemSellSuccess0104::SIZE,
        },
        0x3100005b => FrameKind0104::Fixed {
            size: PcVendorStartFailure0104::SIZE,
        },
        0x3100005a => FrameKind0104::Fixed {
            size: PcVendorStartSuccess0104::SIZE,
        },
        0x3100005d => FrameKind0104::Fixed {
            size: PcVendorTableUpdateFailure0104::SIZE,
        },
        0x3100005c => FrameKind0104::Fixed {
            size: PcVendorTableUpdateSuccess0104::SIZE,
        },
        0x31000074 => FrameKind0104::Fixed {
            size: RemoveBuddyFailure0104::SIZE,
        },
        0x31000073 => FrameKind0104::Fixed {
            size: RemoveBuddySuccess0104::SIZE,
        },
        0x31000066 => FrameKind0104::Fixed {
            size: RequestMakeBuddyFailure0104::SIZE,
        },
        0x3100007b => FrameKind0104::Fixed {
            size: RequestMakeBuddySuccessToAccepter0104::SIZE,
        },
        0x3100007c => FrameKind0104::Counted {
            header_size: RewardItemReply0104::SIZE,
            count_offset: 16,
            count_width: 1,
            trailer_size: 20,
        },
        0x3100006a => FrameKind0104::Fixed {
            size: SendBuddyFreechatMessageFailure0104::SIZE,
        },
        0x31000069 => FrameKind0104::Fixed {
            size: SendBuddyFreechatMessageSuccess0104::SIZE,
        },
        0x3100006c => FrameKind0104::Fixed {
            size: SendBuddyMenuchatMessageFailure0104::SIZE,
        },
        0x3100006b => FrameKind0104::Fixed {
            size: SendBuddyMenuchatMessageSuccess0104::SIZE,
        },
        0x31000013 => FrameKind0104::Fixed {
            size: SendFreechatMessageFailure0104::SIZE,
        },
        0x31000012 => FrameKind0104::Fixed {
            size: SendFreechatMessageSuccess0104::SIZE,
        },
        0x31000019 => FrameKind0104::Fixed {
            size: SendMenuchatMessageFailure0104::SIZE,
        },
        0x31000018 => FrameKind0104::Fixed {
            size: SendMenuchatMessageSuccess0104::SIZE,
        },
        0x31000072 => FrameKind0104::Fixed {
            size: SetBuddyBlockFailure0104::SIZE,
        },
        0x31000071 => FrameKind0104::Fixed {
            size: SetBuddyBlockSuccess0104::SIZE,
        },
        _ => return None,
    })
}
