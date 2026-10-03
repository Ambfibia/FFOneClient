use super::*;
#[test]
fn every_packet_id_matches_the_clean_csdefines_value() {
    assert_eq!(packet::P_CL2LS_REQ_LOGIN, 0x12000001);
    assert_eq!(packet::P_CL2LS_REQ_CHECK_CHAR_NAME, 0x12000002);
    assert_eq!(packet::P_CL2LS_REQ_SAVE_CHAR_NAME, 0x12000003);
    assert_eq!(packet::P_CL2LS_REQ_CHAR_CREATE, 0x12000004);
    assert_eq!(packet::P_CL2LS_REQ_CHAR_SELECT, 0x12000005);
    assert_eq!(packet::P_CL2LS_REQ_CHAR_DELETE, 0x12000006);
    assert_eq!(packet::P_CL2LS_REQ_SHARD_SELECT, 0x12000007);
    assert_eq!(packet::P_CL2LS_REQ_SHARD_LIST_INFO, 0x12000008);
    assert_eq!(packet::P_CL2LS_CHECK_NAME_LIST, 0x12000009);
    assert_eq!(packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR, 0x1200000a);
    assert_eq!(packet::P_CL2LS_REQ_PC_EXIT_DUPLICATE, 0x1200000b);
    assert_eq!(packet::P_CL2LS_REP_LIVE_CHECK, 0x1200000c);
    assert_eq!(packet::P_CL2LS_REQ_CHANGE_CHAR_NAME, 0x1200000d);
    assert_eq!(packet::P_CL2LS_REQ_SERVER_SELECT, 0x1200000e);
    assert_eq!(packet::P_CL2FE_REQ_PC_ENTER, 0x13000001);
    assert_eq!(packet::P_CL2FE_REQ_PC_EXIT, 0x13000002);
    assert_eq!(packet::P_CL2FE_REQ_PC_MOVE, 0x13000003);
    assert_eq!(packet::P_CL2FE_REQ_PC_STOP, 0x13000004);
    assert_eq!(packet::P_CL2FE_REQ_PC_JUMP, 0x13000005);
    assert_eq!(packet::P_CL2FE_REQ_PC_ATTACK_NPCS, 0x13000006);
    assert_eq!(packet::P_CL2FE_REQ_SEND_FREECHAT_MESSAGE, 0x13000007);
    assert_eq!(packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE, 0x13000008);
    assert_eq!(packet::P_CL2FE_REQ_PC_REGEN, 0x13000009);
    assert_eq!(packet::P_CL2FE_REQ_ITEM_MOVE, 0x1300000a);
    assert_eq!(packet::P_CL2FE_REQ_PC_TASK_START, 0x1300000b);
    assert_eq!(packet::P_CL2FE_REQ_PC_TASK_END, 0x1300000c);
    assert_eq!(packet::P_CL2FE_REQ_NANO_EQUIP, 0x1300000d);
    assert_eq!(packet::P_CL2FE_REQ_NANO_UNEQUIP, 0x1300000e);
    assert_eq!(packet::P_CL2FE_REQ_NANO_ACTIVE, 0x1300000f);
    assert_eq!(packet::P_CL2FE_REQ_NANO_TUNE, 0x13000010);
    assert_eq!(packet::P_CL2FE_REQ_NANO_SKILL_USE, 0x13000011);
    assert_eq!(packet::P_CL2FE_REQ_PC_TASK_STOP, 0x13000012);
    assert_eq!(packet::P_CL2FE_REQ_PC_TASK_CONTINUE, 0x13000013);
    assert_eq!(packet::P_CL2FE_REQ_PC_GOTO, 0x13000014);
    assert_eq!(packet::P_CL2FE_REQ_CHARGE_NANO_STAMINA, 0x13000015);
    assert_eq!(packet::P_CL2FE_REQ_PC_KILL_QUEST_NPCS, 0x13000016);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_BUY, 0x13000017);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_SELL, 0x13000018);
    assert_eq!(packet::P_CL2FE_REQ_PC_ITEM_DELETE, 0x13000019);
    assert_eq!(packet::P_CL2FE_REQ_PC_GIVE_ITEM, 0x1300001a);
    assert_eq!(packet::P_CL2FE_REQ_PC_ROCKET_STYLE_READY, 0x1300001b);
    assert_eq!(packet::P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE, 0x1300001c);
    assert_eq!(packet::P_CL2FE_REQ_PC_ROCKET_STYLE_HIT, 0x1300001d);
    assert_eq!(packet::P_CL2FE_REQ_PC_GRENADE_STYLE_READY, 0x1300001e);
    assert_eq!(packet::P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE, 0x1300001f);
    assert_eq!(packet::P_CL2FE_REQ_PC_GRENADE_STYLE_HIT, 0x13000020);
    assert_eq!(packet::P_CL2FE_REQ_PC_NANO_CREATE, 0x13000021);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_OFFER, 0x13000022);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_OFFER_CANCEL, 0x13000023);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_OFFER_ACCEPT, 0x13000024);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_OFFER_REFUSAL, 0x13000025);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_OFFER_ABORT, 0x13000026);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_CONFIRM, 0x13000027);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_CONFIRM_CANCEL, 0x13000028);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_CONFIRM_ABORT, 0x13000029);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_ITEM_REGISTER, 0x1300002a);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_ITEM_UNREGISTER, 0x1300002b);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_CASH_REGISTER, 0x1300002c);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRADE_EMOTES_CHAT, 0x1300002d);
    assert_eq!(packet::P_CL2FE_REQ_PC_BANK_OPEN, 0x1300002e);
    assert_eq!(packet::P_CL2FE_REQ_PC_BANK_CLOSE, 0x1300002f);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_START, 0x13000030);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE, 0x13000031);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY, 0x13000032);
    assert_eq!(packet::P_CL2FE_REQ_PC_COMBAT_BEGIN, 0x13000033);
    assert_eq!(packet::P_CL2FE_REQ_PC_COMBAT_END, 0x13000034);
    assert_eq!(packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY, 0x13000035);
    assert_eq!(packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY, 0x13000036);
    assert_eq!(packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE, 0x13000037);
    assert_eq!(packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE, 0x13000038);
    assert_eq!(packet::P_CL2FE_REQ_GET_BUDDY_STYLE, 0x13000039);
    assert_eq!(packet::P_CL2FE_REQ_SET_BUDDY_BLOCK, 0x1300003a);
    assert_eq!(packet::P_CL2FE_REQ_REMOVE_BUDDY, 0x1300003b);
    assert_eq!(packet::P_CL2FE_REQ_GET_BUDDY_STATE, 0x1300003c);
    assert_eq!(packet::P_CL2FE_REQ_PC_JUMPPAD, 0x1300003d);
    assert_eq!(packet::P_CL2FE_REQ_PC_LAUNCHER, 0x1300003e);
    assert_eq!(packet::P_CL2FE_REQ_PC_ZIPLINE, 0x1300003f);
    assert_eq!(packet::P_CL2FE_REQ_PC_MOVEPLATFORM, 0x13000040);
    assert_eq!(packet::P_CL2FE_REQ_PC_SLOPE, 0x13000041);
    assert_eq!(packet::P_CL2FE_REQ_PC_STATE_CHANGE, 0x13000042);
    assert_eq!(packet::P_CL2FE_REQ_PC_MAP_WARP, 0x13000043);
    assert_eq!(packet::P_CL2FE_REQ_PC_GIVE_NANO, 0x13000044);
    assert_eq!(packet::P_CL2FE_REQ_NPC_SUMMON, 0x13000045);
    assert_eq!(packet::P_CL2FE_REQ_NPC_UNSUMMON, 0x13000046);
    assert_eq!(packet::P_CL2FE_REQ_ITEM_CHEST_OPEN, 0x13000047);
    assert_eq!(packet::P_CL2FE_REQ_PC_GIVE_NANO_SKILL, 0x13000048);
    assert_eq!(packet::P_CL2FE_DOT_DAMAGE_ONOFF, 0x13000049);
    assert_eq!(packet::P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY, 0x1300004a);
    assert_eq!(packet::P_CL2FE_REQ_PC_WARP_USE_NPC, 0x1300004b);
    assert_eq!(packet::P_CL2FE_REQ_PC_GROUP_INVITE, 0x1300004c);
    assert_eq!(packet::P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE, 0x1300004d);
    assert_eq!(packet::P_CL2FE_REQ_PC_GROUP_JOIN, 0x1300004e);
    assert_eq!(packet::P_CL2FE_REQ_PC_GROUP_LEAVE, 0x1300004f);
    assert_eq!(packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT, 0x13000050);
    assert_eq!(packet::P_CL2FE_REQ_PC_BUDDY_WARP, 0x13000051);
    assert_eq!(packet::P_CL2FE_REQ_GET_MEMBER_STYLE, 0x13000052);
    assert_eq!(packet::P_CL2FE_REQ_GET_GROUP_STYLE, 0x13000053);
    assert_eq!(packet::P_CL2FE_REQ_PC_CHANGE_MENTOR, 0x13000054);
    assert_eq!(packet::P_CL2FE_REQ_GET_BUDDY_LOCATION, 0x13000055);
    assert_eq!(packet::P_CL2FE_REQ_NPC_GROUP_SUMMON, 0x13000056);
    assert_eq!(packet::P_CL2FE_REQ_PC_WARP_TO_PC, 0x13000057);
    assert_eq!(packet::P_CL2FE_REQ_EP_RANK_GET_LIST, 0x13000058);
    assert_eq!(packet::P_CL2FE_REQ_EP_RANK_GET_DETAIL, 0x13000059);
    assert_eq!(packet::P_CL2FE_REQ_EP_RANK_GET_PC_INFO, 0x1300005a);
    assert_eq!(packet::P_CL2FE_REQ_EP_RACE_START, 0x1300005b);
    assert_eq!(packet::P_CL2FE_REQ_EP_RACE_END, 0x1300005c);
    assert_eq!(packet::P_CL2FE_REQ_EP_RACE_CANCEL, 0x1300005d);
    assert_eq!(packet::P_CL2FE_REQ_EP_GET_RING, 0x1300005e);
    assert_eq!(packet::P_CL2FE_REQ_IM_CHANGE_SWITCH_STATUS, 0x1300005f);
    assert_eq!(packet::P_CL2FE_REQ_SHINY_PICKUP, 0x13000060);
    assert_eq!(packet::P_CL2FE_REQ_SHINY_SUMMON, 0x13000061);
    assert_eq!(packet::P_CL2FE_REQ_PC_MOVETRANSPORTATION, 0x13000062);
    assert_eq!(
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE,
        0x13000063
    );
    assert_eq!(
        packet::P_CL2FE_REQ_SEND_ANY_GROUP_FREECHAT_MESSAGE,
        0x13000064
    );
    assert_eq!(packet::P_CL2FE_REQ_BARKER, 0x13000065);
    assert_eq!(
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE,
        0x13000066
    );
    assert_eq!(
        packet::P_CL2FE_REQ_SEND_ANY_GROUP_MENUCHAT_MESSAGE,
        0x13000067
    );
    assert_eq!(
        packet::P_CL2FE_REQ_REGIST_TRANSPORTATION_LOCATION,
        0x13000068
    );
    assert_eq!(packet::P_CL2FE_REQ_PC_WARP_USE_TRANSPORTATION, 0x13000069);
    assert_eq!(packet::P_CL2FE_GM_REQ_PC_SPECIAL_STATE_SWITCH, 0x1300006a);
    assert_eq!(packet::P_CL2FE_GM_REQ_PC_SET_VALUE, 0x1300006b);
    assert_eq!(packet::P_CL2FE_GM_REQ_KICK_PLAYER, 0x1300006c);
    assert_eq!(packet::P_CL2FE_GM_REQ_TARGET_PC_TELEPORT, 0x1300006d);
    assert_eq!(packet::P_CL2FE_GM_REQ_PC_LOCATION, 0x1300006e);
    assert_eq!(packet::P_CL2FE_GM_REQ_PC_ANNOUNCE, 0x1300006f);
    assert_eq!(packet::P_CL2FE_REQ_SET_PC_BLOCK, 0x13000070);
    assert_eq!(packet::P_CL2FE_REQ_REGIST_RXCOM, 0x13000071);
    assert_eq!(packet::P_CL2FE_GM_REQ_PC_MOTD_REGISTER, 0x13000072);
    assert_eq!(packet::P_CL2FE_REQ_ITEM_USE, 0x13000073);
    assert_eq!(packet::P_CL2FE_REQ_WARP_USE_RECALL, 0x13000074);
    assert_eq!(packet::P_CL2FE_REP_LIVE_CHECK, 0x13000075);
    assert_eq!(packet::P_CL2FE_REQ_PC_MISSION_COMPLETE, 0x13000076);
    assert_eq!(packet::P_CL2FE_REQ_PC_TASK_COMPLETE, 0x13000077);
    assert_eq!(packet::P_CL2FE_REQ_NPC_INTERACTION, 0x13000078);
    assert_eq!(packet::P_CL2FE_DOT_HEAL_ONOFF, 0x13000079);
    assert_eq!(packet::P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH, 0x1300007a);
    assert_eq!(packet::P_CL2FE_REQ_PC_EMAIL_UPDATE_CHECK, 0x1300007b);
    assert_eq!(packet::P_CL2FE_REQ_PC_READ_EMAIL, 0x1300007c);
    assert_eq!(packet::P_CL2FE_REQ_PC_RECV_EMAIL_PAGE_LIST, 0x1300007d);
    assert_eq!(packet::P_CL2FE_REQ_PC_DELETE_EMAIL, 0x1300007e);
    assert_eq!(packet::P_CL2FE_REQ_PC_SEND_EMAIL, 0x1300007f);
    assert_eq!(packet::P_CL2FE_REQ_PC_RECV_EMAIL_ITEM, 0x13000080);
    assert_eq!(packet::P_CL2FE_REQ_PC_RECV_EMAIL_CANDY, 0x13000081);
    assert_eq!(
        packet::P_CL2FE_GM_REQ_TARGET_PC_SPECIAL_STATE_ONOFF,
        0x13000082
    );
    assert_eq!(packet::P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID, 0x13000083);
    assert_eq!(packet::P_CL2FE_REQ_NPC_GROUP_INVITE, 0x13000084);
    assert_eq!(packet::P_CL2FE_REQ_NPC_GROUP_KICK, 0x13000085);
    assert_eq!(packet::P_CL2FE_REQ_PC_FIRST_USE_FLAG_SET, 0x13000086);
    assert_eq!(packet::P_CL2FE_REQ_PC_TRANSPORT_WARP, 0x13000087);
    assert_eq!(packet::P_CL2FE_REQ_PC_TIME_TO_GO_WARP, 0x13000088);
    assert_eq!(packet::P_CL2FE_REQ_PC_RECV_EMAIL_ITEM_ALL, 0x13000089);
    assert_eq!(packet::P_CL2FE_REQ_CHANNEL_INFO, 0x1300008a);
    assert_eq!(packet::P_CL2FE_REQ_PC_CHANNEL_NUM, 0x1300008b);
    assert_eq!(packet::P_CL2FE_REQ_PC_WARP_CHANNEL, 0x1300008c);
    assert_eq!(packet::P_CL2FE_REQ_PC_LOADING_COMPLETE, 0x1300008d);
    assert_eq!(packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY, 0x1300008e);
    assert_eq!(packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY, 0x1300008f);
    assert_eq!(packet::P_CL2FE_REQ_PC_ATTACK_CHARS, 0x13000090);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_READY, 0x13000091);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_CANCEL, 0x13000092);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_REGIST_ITEM, 0x13000093);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_UNREGIST_ITEM, 0x13000094);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_SALE_START, 0x13000095);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_ITEM_LIST, 0x13000096);
    assert_eq!(packet::P_CL2FE_PC_STREETSTALL_REQ_ITEM_BUY, 0x13000097);
    assert_eq!(packet::P_CL2FE_REQ_PC_ITEM_COMBINATION, 0x13000098);
    assert_eq!(packet::P_CL2FE_GM_REQ_SET_PC_SKILL, 0x13000099);
    assert_eq!(packet::P_CL2FE_REQ_PC_SKILL_ADD, 0x1300009a);
    assert_eq!(packet::P_CL2FE_REQ_PC_SKILL_DEL, 0x1300009b);
    assert_eq!(packet::P_CL2FE_REQ_PC_SKILL_USE, 0x1300009c);
    assert_eq!(packet::P_CL2FE_REQ_PC_ROPE, 0x1300009d);
    assert_eq!(packet::P_CL2FE_REQ_PC_BELT, 0x1300009e);
    assert_eq!(packet::P_CL2FE_REQ_PC_VEHICLE_ON, 0x1300009f);
    assert_eq!(packet::P_CL2FE_REQ_PC_VEHICLE_OFF, 0x130000a0);
    assert_eq!(packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT, 0x130000a1);
    assert_eq!(packet::P_CL2FE_REQ_PC_DISASSEMBLE_ITEM, 0x130000a2);
    assert_eq!(packet::P_CL2FE_GM_REQ_REWARD_RATE, 0x130000a3);
    assert_eq!(packet::P_CL2FE_REQ_PC_ITEM_ENCHANT, 0x130000a4);
    assert_eq!(packet::P_CL2FE_REQ_PC_BARBER_OPEN, 0x130000a5);
    assert_eq!(packet::P_CL2FE_REQ_PC_BARBER_CONFIRM, 0x130000a6);
    assert_eq!(packet::P_CL2FE_REQ_PRESENT_NPC_TYPES, 0x130000a7);
    assert_eq!(packet::P_LS2CL_REP_LOGIN_SUCC, 0x21000001);
    assert_eq!(packet::P_LS2CL_REP_LOGIN_FAIL, 0x21000002);
    assert_eq!(packet::P_LS2CL_REP_CHAR_INFO, 0x21000003);
    assert_eq!(packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC, 0x21000005);
    assert_eq!(packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL, 0x21000006);
    assert_eq!(packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC, 0x21000007);
    assert_eq!(packet::P_LS2CL_REP_SAVE_CHAR_NAME_FAIL, 0x21000008);
    assert_eq!(packet::P_LS2CL_REP_CHAR_CREATE_SUCC, 0x21000009);
    assert_eq!(packet::P_LS2CL_REP_CHAR_CREATE_FAIL, 0x2100000a);
    assert_eq!(packet::P_LS2CL_REP_CHAR_SELECT_SUCC, 0x2100000b);
    assert_eq!(packet::P_LS2CL_REP_CHAR_SELECT_FAIL, 0x2100000c);
    assert_eq!(packet::P_LS2CL_REP_CHAR_DELETE_SUCC, 0x2100000d);
    assert_eq!(packet::P_LS2CL_REP_CHAR_DELETE_FAIL, 0x2100000e);
    assert_eq!(packet::P_LS2CL_REP_SHARD_SELECT_SUCC, 0x2100000f);
    assert_eq!(packet::P_LS2CL_REP_SHARD_SELECT_FAIL, 0x21000010);
    assert_eq!(packet::P_LS2CL_REP_VERSION_CHECK_SUCC, 0x21000011);
    assert_eq!(packet::P_LS2CL_REP_VERSION_CHECK_FAIL, 0x21000012);
    assert_eq!(packet::P_LS2CL_REP_CHECK_NAME_LIST_SUCC, 0x21000013);
    assert_eq!(packet::P_LS2CL_REP_CHECK_NAME_LIST_FAIL, 0x21000014);
    assert_eq!(packet::P_LS2CL_REP_PC_EXIT_DUPLICATE, 0x21000015);
    assert_eq!(packet::P_LS2CL_REQ_LIVE_CHECK, 0x21000016);
    assert_eq!(packet::P_LS2CL_REP_CHANGE_CHAR_NAME_SUCC, 0x21000017);
    assert_eq!(packet::P_LS2CL_REP_CHANGE_CHAR_NAME_FAIL, 0x21000018);
    assert_eq!(packet::P_LS2CL_REP_SHARD_LIST_INFO_SUCC, 0x21000019);
    assert_eq!(packet::P_FE2CL_ERROR, 0x31000000);
    assert_eq!(packet::P_FE2CL_REP_PC_ENTER_FAIL, 0x31000001);
    assert_eq!(packet::P_FE2CL_REP_PC_ENTER_SUCC, 0x31000002);
    assert_eq!(packet::P_FE2CL_PC_NEW, 0x31000003);
    assert_eq!(packet::P_FE2CL_REP_PC_EXIT_FAIL, 0x31000004);
    assert_eq!(packet::P_FE2CL_REP_PC_EXIT_SUCC, 0x31000005);
    assert_eq!(packet::P_FE2CL_PC_EXIT, 0x31000006);
    assert_eq!(packet::P_FE2CL_PC_AROUND, 0x31000007);
    assert_eq!(packet::P_FE2CL_PC_MOVE, 0x31000008);
    assert_eq!(packet::P_FE2CL_PC_STOP, 0x31000009);
    assert_eq!(packet::P_FE2CL_PC_JUMP, 0x3100000a);
    assert_eq!(packet::P_FE2CL_NPC_ENTER, 0x3100000b);
    assert_eq!(packet::P_FE2CL_NPC_EXIT, 0x3100000c);
    assert_eq!(packet::P_FE2CL_NPC_MOVE, 0x3100000d);
    assert_eq!(packet::P_FE2CL_NPC_NEW, 0x3100000e);
    assert_eq!(packet::P_FE2CL_NPC_AROUND, 0x3100000f);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_PC, 0x31000010);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_NPC, 0x31000011);
    assert_eq!(packet::P_FE2CL_REP_SEND_FREECHAT_MESSAGE_SUCC, 0x31000012);
    assert_eq!(packet::P_FE2CL_REP_SEND_FREECHAT_MESSAGE_FAIL, 0x31000013);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_NPCS_SUCC, 0x31000014);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_NPCS, 0x31000015);
    assert_eq!(packet::P_FE2CL_NPC_ATTACK_PCS, 0x31000016);
    assert_eq!(packet::P_FE2CL_REP_PC_REGEN_SUCC, 0x31000017);
    assert_eq!(packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC, 0x31000018);
    assert_eq!(packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_FAIL, 0x31000019);
    assert_eq!(packet::P_FE2CL_PC_ITEM_MOVE_SUCC, 0x3100001a);
    assert_eq!(packet::P_FE2CL_PC_EQUIP_CHANGE, 0x3100001b);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_START_SUCC, 0x3100001c);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_START_FAIL, 0x3100001d);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_END_SUCC, 0x3100001e);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_END_FAIL, 0x3100001f);
    assert_eq!(packet::P_FE2CL_NPC_SKILL_READY, 0x31000020);
    assert_eq!(packet::P_FE2CL_NPC_SKILL_FIRE, 0x31000021);
    assert_eq!(packet::P_FE2CL_NPC_SKILL_HIT, 0x31000022);
    assert_eq!(packet::P_FE2CL_NPC_SKILL_CORRUPTION_READY, 0x31000023);
    assert_eq!(packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT, 0x31000024);
    assert_eq!(packet::P_FE2CL_NPC_SKILL_CANCEL, 0x31000025);
    assert_eq!(packet::P_FE2CL_REP_NANO_EQUIP_SUCC, 0x31000026);
    assert_eq!(packet::P_FE2CL_REP_NANO_UNEQUIP_SUCC, 0x31000027);
    assert_eq!(packet::P_FE2CL_REP_NANO_ACTIVE_SUCC, 0x31000028);
    assert_eq!(packet::P_FE2CL_REP_NANO_TUNE_SUCC, 0x31000029);
    assert_eq!(packet::P_FE2CL_NANO_ACTIVE, 0x3100002a);
    assert_eq!(packet::P_FE2CL_NANO_SKILL_USE_SUCC, 0x3100002b);
    assert_eq!(packet::P_FE2CL_NANO_SKILL_USE, 0x3100002c);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_STOP_SUCC, 0x3100002d);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_STOP_FAIL, 0x3100002e);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_CONTINUE_SUCC, 0x3100002f);
    assert_eq!(packet::P_FE2CL_REP_PC_TASK_CONTINUE_FAIL, 0x31000030);
    assert_eq!(packet::P_FE2CL_REP_PC_GOTO_SUCC, 0x31000031);
    assert_eq!(packet::P_FE2CL_REP_CHARGE_NANO_STAMINA, 0x31000032);
    assert_eq!(packet::P_FE2CL_REP_PC_TICK, 0x31000033);
    assert_eq!(packet::P_FE2CL_REP_PC_KILL_QUEST_NPCS_SUCC, 0x31000034);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC, 0x31000035);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL, 0x31000036);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC, 0x31000037);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL, 0x31000038);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC, 0x31000039);
    assert_eq!(packet::P_FE2CL_PC_ROCKET_STYLE_READY, 0x3100003a);
    assert_eq!(packet::P_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC, 0x3100003b);
    assert_eq!(packet::P_FE2CL_PC_ROCKET_STYLE_FIRE, 0x3100003c);
    assert_eq!(packet::P_FE2CL_PC_ROCKET_STYLE_HIT, 0x3100003d);
    assert_eq!(packet::P_FE2CL_PC_GRENADE_STYLE_READY, 0x3100003e);
    assert_eq!(packet::P_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC, 0x3100003f);
    assert_eq!(packet::P_FE2CL_PC_GRENADE_STYLE_FIRE, 0x31000040);
    assert_eq!(packet::P_FE2CL_PC_GRENADE_STYLE_HIT, 0x31000041);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_OFFER, 0x31000042);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_OFFER_CANCEL, 0x31000043);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_OFFER_SUCC, 0x31000044);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_OFFER_REFUSAL, 0x31000045);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_OFFER_ABORT, 0x31000046);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CONFIRM, 0x31000047);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CONFIRM_CANCEL, 0x31000048);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CONFIRM_ABORT, 0x31000049);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CONFIRM_SUCC, 0x3100004a);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CONFIRM_FAIL, 0x3100004b);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_ITEM_REGISTER_SUCC, 0x3100004c);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_ITEM_REGISTER_FAIL, 0x3100004d);
    assert_eq!(
        packet::P_FE2CL_REP_PC_TRADE_ITEM_UNREGISTER_SUCC,
        0x3100004e
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_TRADE_ITEM_UNREGISTER_FAIL,
        0x3100004f
    );
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CASH_REGISTER_SUCC, 0x31000050);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_CASH_REGISTER_FAIL, 0x31000051);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_EMOTES_CHAT, 0x31000052);
    assert_eq!(packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC, 0x31000053);
    assert_eq!(packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL, 0x31000054);
    assert_eq!(packet::P_FE2CL_REP_NANO_TUNE_FAIL, 0x31000055);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC, 0x31000056);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_OPEN_FAIL, 0x31000057);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_CLOSE_SUCC, 0x31000058);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_CLOSE_FAIL, 0x31000059);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_START_SUCC, 0x3100005a);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_START_FAIL, 0x3100005b);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC, 0x3100005c);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL, 0x3100005d);
    assert_eq!(
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC,
        0x3100005e
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL,
        0x3100005f
    );
    assert_eq!(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT, 0x31000060);
    assert_eq!(packet::P_FE2CL_REP_PC_GIVE_ITEM_SUCC, 0x31000061);
    assert_eq!(packet::P_FE2CL_REP_PC_GIVE_ITEM_FAIL, 0x31000062);
    assert_eq!(packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC, 0x31000063);
    assert_eq!(packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_FAIL, 0x31000064);
    assert_eq!(packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_FAIL, 0x31000066);
    assert_eq!(packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC, 0x31000067);
    assert_eq!(packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_FAIL, 0x31000068);
    assert_eq!(
        packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC,
        0x31000069
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_FAIL,
        0x3100006a
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC,
        0x3100006b
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_FAIL,
        0x3100006c
    );
    assert_eq!(packet::P_FE2CL_REP_GET_BUDDY_STYLE_SUCC, 0x3100006d);
    assert_eq!(packet::P_FE2CL_REP_GET_BUDDY_STYLE_FAIL, 0x3100006e);
    assert_eq!(packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC, 0x3100006f);
    assert_eq!(packet::P_FE2CL_REP_GET_BUDDY_STATE_FAIL, 0x31000070);
    assert_eq!(packet::P_FE2CL_REP_SET_BUDDY_BLOCK_SUCC, 0x31000071);
    assert_eq!(packet::P_FE2CL_REP_SET_BUDDY_BLOCK_FAIL, 0x31000072);
    assert_eq!(packet::P_FE2CL_REP_REMOVE_BUDDY_SUCC, 0x31000073);
    assert_eq!(packet::P_FE2CL_REP_REMOVE_BUDDY_FAIL, 0x31000074);
    assert_eq!(packet::P_FE2CL_PC_JUMPPAD, 0x31000075);
    assert_eq!(packet::P_FE2CL_PC_LAUNCHER, 0x31000076);
    assert_eq!(packet::P_FE2CL_PC_ZIPLINE, 0x31000077);
    assert_eq!(packet::P_FE2CL_PC_MOVEPLATFORM, 0x31000078);
    assert_eq!(packet::P_FE2CL_PC_SLOPE, 0x31000079);
    assert_eq!(packet::P_FE2CL_PC_STATE_CHANGE, 0x3100007a);
    assert_eq!(
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER,
        0x3100007b
    );
    assert_eq!(packet::P_FE2CL_REP_REWARD_ITEM, 0x3100007c);
    assert_eq!(packet::P_FE2CL_REP_ITEM_CHEST_OPEN_SUCC, 0x3100007d);
    assert_eq!(packet::P_FE2CL_REP_ITEM_CHEST_OPEN_FAIL, 0x3100007e);
    assert_eq!(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK, 0x3100007f);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC, 0x31000080);
    assert_eq!(packet::P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL, 0x31000081);
    assert_eq!(packet::P_FE2CL_NPC_ROCKET_STYLE_FIRE, 0x31000082);
    assert_eq!(packet::P_FE2CL_NPC_GRENADE_STYLE_FIRE, 0x31000083);
    assert_eq!(packet::P_FE2CL_NPC_BULLET_STYLE_HIT, 0x31000084);
    assert_eq!(packet::P_FE2CL_CHARACTER_ATTACK_CHARACTERS, 0x31000085);
    assert_eq!(packet::P_FE2CL_PC_GROUP_INVITE, 0x31000086);
    assert_eq!(packet::P_FE2CL_PC_GROUP_INVITE_FAIL, 0x31000087);
    assert_eq!(packet::P_FE2CL_PC_GROUP_INVITE_REFUSE, 0x31000088);
    assert_eq!(packet::P_FE2CL_PC_GROUP_JOIN, 0x31000089);
    assert_eq!(packet::P_FE2CL_PC_GROUP_JOIN_FAIL, 0x3100008a);
    assert_eq!(packet::P_FE2CL_PC_GROUP_JOIN_SUCC, 0x3100008b);
    assert_eq!(packet::P_FE2CL_PC_GROUP_LEAVE, 0x3100008c);
    assert_eq!(packet::P_FE2CL_PC_GROUP_LEAVE_FAIL, 0x3100008d);
    assert_eq!(packet::P_FE2CL_PC_GROUP_LEAVE_SUCC, 0x3100008e);
    assert_eq!(packet::P_FE2CL_PC_GROUP_MEMBER_INFO, 0x3100008f);
    assert_eq!(packet::P_FE2CL_REP_PC_WARP_USE_NPC_SUCC, 0x31000090);
    assert_eq!(packet::P_FE2CL_REP_PC_WARP_USE_NPC_FAIL, 0x31000091);
    assert_eq!(packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT, 0x31000092);
    assert_eq!(packet::P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC, 0x31000093);
    assert_eq!(packet::P_FE2CL_REP_PC_CHANGE_MENTOR_FAIL, 0x31000094);
    assert_eq!(packet::P_FE2CL_REP_GET_MEMBER_STYLE_FAIL, 0x31000095);
    assert_eq!(packet::P_FE2CL_REP_GET_MEMBER_STYLE_SUCC, 0x31000096);
    assert_eq!(packet::P_FE2CL_REP_GET_GROUP_STYLE_FAIL, 0x31000097);
    assert_eq!(packet::P_FE2CL_REP_GET_GROUP_STYLE_SUCC, 0x31000098);
    assert_eq!(packet::P_FE2CL_PC_REGEN, 0x31000099);
    assert_eq!(packet::P_FE2CL_INSTANCE_MAP_INFO, 0x3100009a);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_ENTER, 0x3100009b);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_EXIT, 0x3100009c);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_MOVE, 0x3100009d);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_NEW, 0x3100009e);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_AROUND, 0x3100009f);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_TRANSPORTATION, 0x310000a0);
    assert_eq!(packet::P_FE2CL_REP_EP_RANK_LIST, 0x310000a1);
    assert_eq!(packet::P_FE2CL_REP_EP_RANK_DETAIL, 0x310000a2);
    assert_eq!(packet::P_FE2CL_REP_EP_RANK_PC_INFO, 0x310000a3);
    assert_eq!(packet::P_FE2CL_REP_EP_RACE_START_SUCC, 0x310000a4);
    assert_eq!(packet::P_FE2CL_REP_EP_RACE_START_FAIL, 0x310000a5);
    assert_eq!(packet::P_FE2CL_REP_EP_RACE_END_SUCC, 0x310000a6);
    assert_eq!(packet::P_FE2CL_REP_EP_RACE_END_FAIL, 0x310000a7);
    assert_eq!(packet::P_FE2CL_REP_EP_RACE_CANCEL_SUCC, 0x310000a8);
    assert_eq!(packet::P_FE2CL_REP_EP_RACE_CANCEL_FAIL, 0x310000a9);
    assert_eq!(packet::P_FE2CL_REP_EP_GET_RING_SUCC, 0x310000aa);
    assert_eq!(packet::P_FE2CL_REP_EP_GET_RING_FAIL, 0x310000ab);
    assert_eq!(packet::P_FE2CL_REP_IM_CHANGE_SWITCH_STATUS, 0x310000ac);
    assert_eq!(packet::P_FE2CL_SHINY_ENTER, 0x310000ad);
    assert_eq!(packet::P_FE2CL_SHINY_EXIT, 0x310000ae);
    assert_eq!(packet::P_FE2CL_SHINY_NEW, 0x310000af);
    assert_eq!(packet::P_FE2CL_SHINY_AROUND, 0x310000b0);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_SHINY, 0x310000b1);
    assert_eq!(packet::P_FE2CL_REP_SHINY_PICKUP_FAIL, 0x310000b2);
    assert_eq!(packet::P_FE2CL_REP_SHINY_PICKUP_SUCC, 0x310000b3);
    assert_eq!(packet::P_FE2CL_PC_MOVETRANSPORTATION, 0x310000b4);
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC,
        0x310000b5
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_FAIL,
        0x310000b6
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ANY_GROUP_FREECHAT_MESSAGE_SUCC,
        0x310000b7
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ANY_GROUP_FREECHAT_MESSAGE_FAIL,
        0x310000b8
    );
    assert_eq!(packet::P_FE2CL_REP_BARKER, 0x310000b9);
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC,
        0x310000ba
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_FAIL,
        0x310000bb
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ANY_GROUP_MENUCHAT_MESSAGE_SUCC,
        0x310000bc
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ANY_GROUP_MENUCHAT_MESSAGE_FAIL,
        0x310000bd
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_REGIST_TRANSPORTATION_LOCATION_FAIL,
        0x310000be
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_REGIST_TRANSPORTATION_LOCATION_SUCC,
        0x310000bf
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_WARP_USE_TRANSPORTATION_FAIL,
        0x310000c0
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_WARP_USE_TRANSPORTATION_SUCC,
        0x310000c1
    );
    assert_eq!(packet::P_FE2CL_ANNOUNCE_MSG, 0x310000c2);
    assert_eq!(packet::P_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC, 0x310000c3);
    assert_eq!(packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE, 0x310000c4);
    assert_eq!(packet::P_FE2CL_GM_REP_PC_SET_VALUE, 0x310000c5);
    assert_eq!(packet::P_FE2CL_GM_PC_CHANGE_VALUE, 0x310000c6);
    assert_eq!(packet::P_FE2CL_GM_REP_PC_LOCATION, 0x310000c7);
    assert_eq!(packet::P_FE2CL_GM_REP_PC_ANNOUNCE, 0x310000c8);
    assert_eq!(packet::P_FE2CL_REP_PC_BUDDY_WARP_FAIL, 0x310000c9);
    assert_eq!(packet::P_FE2CL_REP_PC_CHANGE_LEVEL, 0x310000ca);
    assert_eq!(packet::P_FE2CL_REP_SET_PC_BLOCK_SUCC, 0x310000cb);
    assert_eq!(packet::P_FE2CL_REP_SET_PC_BLOCK_FAIL, 0x310000cc);
    assert_eq!(packet::P_FE2CL_REP_REGIST_RXCOM, 0x310000cd);
    assert_eq!(packet::P_FE2CL_REP_REGIST_RXCOM_FAIL, 0x310000ce);
    assert_eq!(packet::P_FE2CL_PC_INVEN_FULL_MSG, 0x310000cf);
    assert_eq!(packet::P_FE2CL_REQ_LIVE_CHECK, 0x310000d0);
    assert_eq!(packet::P_FE2CL_PC_MOTD_LOGIN, 0x310000d1);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_USE_FAIL, 0x310000d2);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_USE_SUCC, 0x310000d3);
    assert_eq!(packet::P_FE2CL_PC_ITEM_USE, 0x310000d4);
    assert_eq!(packet::P_FE2CL_REP_GET_BUDDY_LOCATION_SUCC, 0x310000d5);
    assert_eq!(packet::P_FE2CL_REP_GET_BUDDY_LOCATION_FAIL, 0x310000d6);
    assert_eq!(packet::P_FE2CL_REP_PC_RIDING_FAIL, 0x310000d7);
    assert_eq!(packet::P_FE2CL_REP_PC_RIDING_SUCC, 0x310000d8);
    assert_eq!(packet::P_FE2CL_PC_RIDING, 0x310000d9);
    assert_eq!(packet::P_FE2CL_PC_BROOMSTICK_MOVE, 0x310000da);
    assert_eq!(
        packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC,
        0x310000db
    );
    assert_eq!(packet::P_FE2CL_REP_WARP_USE_RECALL_FAIL, 0x310000dc);
    assert_eq!(packet::P_FE2CL_REP_PC_EXIT_DUPLICATE, 0x310000dd);
    assert_eq!(packet::P_FE2CL_REP_PC_MISSION_COMPLETE_SUCC, 0x310000de);
    assert_eq!(packet::P_FE2CL_PC_BUFF_UPDATE, 0x310000df);
    assert_eq!(packet::P_FE2CL_REP_PC_NEW_EMAIL, 0x310000e0);
    assert_eq!(packet::P_FE2CL_REP_PC_READ_EMAIL_SUCC, 0x310000e1);
    assert_eq!(packet::P_FE2CL_REP_PC_READ_EMAIL_FAIL, 0x310000e2);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_PAGE_LIST_SUCC, 0x310000e3);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_PAGE_LIST_FAIL, 0x310000e4);
    assert_eq!(packet::P_FE2CL_REP_PC_DELETE_EMAIL_SUCC, 0x310000e5);
    assert_eq!(packet::P_FE2CL_REP_PC_DELETE_EMAIL_FAIL, 0x310000e6);
    assert_eq!(packet::P_FE2CL_REP_PC_SEND_EMAIL_SUCC, 0x310000e7);
    assert_eq!(packet::P_FE2CL_REP_PC_SEND_EMAIL_FAIL, 0x310000e8);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_ITEM_SUCC, 0x310000e9);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_ITEM_FAIL, 0x310000ea);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_CANDY_SUCC, 0x310000eb);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_CANDY_FAIL, 0x310000ec);
    assert_eq!(packet::P_FE2CL_PC_SUDDEN_DEAD, 0x310000ed);
    assert_eq!(
        packet::P_FE2CL_REP_GM_REQ_TARGET_PC_SPECIAL_STATE_ONOFF_SUCC,
        0x310000ee
    );
    assert_eq!(packet::P_FE2CL_REP_PC_SET_CURRENT_MISSION_ID, 0x310000ef);
    assert_eq!(packet::P_FE2CL_REP_NPC_GROUP_INVITE_FAIL, 0x310000f0);
    assert_eq!(packet::P_FE2CL_REP_NPC_GROUP_INVITE_SUCC, 0x310000f1);
    assert_eq!(packet::P_FE2CL_REP_NPC_GROUP_KICK_FAIL, 0x310000f2);
    assert_eq!(packet::P_FE2CL_REP_NPC_GROUP_KICK_SUCC, 0x310000f3);
    assert_eq!(packet::P_FE2CL_PC_EVENT, 0x310000f4);
    assert_eq!(packet::P_FE2CL_REP_PC_TRANSPORT_WARP_SUCC, 0x310000f5);
    assert_eq!(packet::P_FE2CL_REP_PC_TRADE_EMOTES_CHAT_FAIL, 0x310000f6);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_ITEM_ALL_SUCC, 0x310000f7);
    assert_eq!(packet::P_FE2CL_REP_PC_RECV_EMAIL_ITEM_ALL_FAIL, 0x310000f8);
    assert_eq!(packet::P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC, 0x310000f9);
    assert_eq!(packet::P_FE2CL_REP_CHANNEL_INFO, 0x310000fa);
    assert_eq!(packet::P_FE2CL_REP_PC_CHANNEL_NUM, 0x310000fb);
    assert_eq!(packet::P_FE2CL_REP_PC_WARP_CHANNEL_FAIL, 0x310000fc);
    assert_eq!(packet::P_FE2CL_REP_PC_WARP_CHANNEL_SUCC, 0x310000fd);
    assert_eq!(packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC, 0x310000fe);
    assert_eq!(packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL, 0x310000ff);
    assert_eq!(
        packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL,
        0x31000100
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC,
        0x31000101
    );
    assert_eq!(packet::P_FE2CL_PC_ATTACK_CHARS_SUCC, 0x31000102);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_CHARS, 0x31000103);
    assert_eq!(packet::P_FE2CL_NPC_ATTACK_CHARS, 0x31000104);
    assert_eq!(packet::P_FE2CL_REP_PC_CHANGE_LEVEL_SUCC, 0x31000105);
    assert_eq!(packet::P_FE2CL_REP_PC_NANO_CREATE, 0x31000106);
    assert_eq!(packet::P_FE2CL_PC_STREETSTALL_REP_READY_SUCC, 0x31000107);
    assert_eq!(packet::P_FE2CL_PC_STREETSTALL_REP_READY_FAIL, 0x31000108);
    assert_eq!(packet::P_FE2CL_PC_STREETSTALL_REP_CANCEL_SUCC, 0x31000109);
    assert_eq!(packet::P_FE2CL_PC_STREETSTALL_REP_CANCEL_FAIL, 0x3100010a);
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_REGIST_ITEM_SUCC,
        0x3100010b
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_REGIST_ITEM_FAIL,
        0x3100010c
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_UNREGIST_ITEM_SUCC,
        0x3100010d
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_UNREGIST_ITEM_FAIL,
        0x3100010e
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_SALE_START_SUCC,
        0x3100010f
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_SALE_START_FAIL,
        0x31000110
    );
    assert_eq!(packet::P_FE2CL_PC_STREETSTALL_REP_ITEM_LIST, 0x31000111);
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_ITEM_LIST_FAIL,
        0x31000112
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_ITEM_BUY_SUCC_BUYER,
        0x31000113
    );
    assert_eq!(
        packet::P_FE2CL_PC_STREETSTALL_REP_ITEM_BUY_SUCC_SELLER,
        0x31000114
    );
    assert_eq!(packet::P_FE2CL_PC_STREETSTALL_REP_ITEM_BUY_FAIL, 0x31000115);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_COMBINATION_SUCC, 0x31000116);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_COMBINATION_FAIL, 0x31000117);
    assert_eq!(packet::P_FE2CL_PC_CASH_BUFF_UPDATE, 0x31000118);
    assert_eq!(packet::P_FE2CL_REP_PC_SKILL_ADD_SUCC, 0x31000119);
    assert_eq!(packet::P_FE2CL_REP_PC_SKILL_ADD_FAIL, 0x3100011a);
    assert_eq!(packet::P_FE2CL_REP_PC_SKILL_DEL_SUCC, 0x3100011b);
    assert_eq!(packet::P_FE2CL_REP_PC_SKILL_DEL_FAIL, 0x3100011c);
    assert_eq!(packet::P_FE2CL_REP_PC_SKILL_USE_SUCC, 0x3100011d);
    assert_eq!(packet::P_FE2CL_REP_PC_SKILL_USE_FAIL, 0x3100011e);
    assert_eq!(packet::P_FE2CL_PC_SKILL_USE, 0x3100011f);
    assert_eq!(packet::P_FE2CL_PC_ROPE, 0x31000120);
    assert_eq!(packet::P_FE2CL_PC_BELT, 0x31000121);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_ON_SUCC, 0x31000122);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_ON_FAIL, 0x31000123);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_OFF_SUCC, 0x31000124);
    assert_eq!(packet::P_FE2CL_PC_VEHICLE_OFF_FAIL, 0x31000125);
    assert_eq!(packet::P_FE2CL_PC_QUICK_SLOT_INFO, 0x31000126);
    assert_eq!(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL, 0x31000127);
    assert_eq!(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC, 0x31000128);
    assert_eq!(packet::P_FE2CL_PC_DELETE_TIME_LIMIT_ITEM, 0x31000129);
    assert_eq!(packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC, 0x3100012a);
    assert_eq!(packet::P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL, 0x3100012b);
    assert_eq!(packet::P_FE2CL_GM_REP_REWARD_RATE_SUCC, 0x3100012c);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_ENCHANT_SUCC, 0x3100012d);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_ENCHANT_FAIL, 0x3100012e);
    assert_eq!(packet::P_FE2CL_REP_NANO_BOOK_SUBSET, 0x31000134);
    assert_eq!(packet::P_FE2CL_NPC_CUSTOM_ATTACK_PCS, 0x31000135);
    assert_eq!(packet::P_FE2CL_NPC_SELF_EFFECT, 0x31000136);
    assert_eq!(packet::P_FE2CL_REP_PRESENT_NPC_TYPES, 0x31000137);
    assert_eq!(packet::P_FE2CL_REP_PC_BARBER_OPEN_SUCC, 0x31000138);
    assert_eq!(packet::P_FE2CL_REP_PC_BARBER_CONFIRM, 0x31000139);
    assert_eq!(packet::P_FE2CL_PC_STYLE_CHANGE, 0x3100013a);
    assert_eq!(packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC, 0x83000065);
    assert_eq!(PACKET_IDS_0104.len(), 515);
}

#[test]
fn frame_kinds_follow_the_openfusion_descriptor_table() {
    let mut fixed = 0;
    let mut counted = 0;
    let mut manual = 0;
    for (_, id) in PACKET_IDS_0104 {
        match frame_kind(*id) {
            Some(FrameKind0104::Fixed { size }) => {
                fixed += 1;
                assert_eq!(fixed_frame_size(*id), Some(size));
                assert_eq!(declared_struct_size(*id), Some(size));
            }
            Some(FrameKind0104::Counted {
                header_size,
                count_offset,
                count_width,
                trailer_size,
            }) => {
                counted += 1;
                assert!(matches!(count_width, 1 | 2 | 4));
                assert!(count_offset + count_width <= header_size);
                assert!(trailer_size > 0);
                assert_eq!(fixed_frame_size(*id), None);
            }
            Some(FrameKind0104::Manual { header_size }) => {
                manual += 1;
                assert_eq!(declared_struct_size(*id), Some(header_size));
                assert_eq!(fixed_frame_size(*id), None);
            }
            None => {}
        }
    }
    assert_eq!((fixed, counted, manual), (485, 15, 6));
}

#[test]
fn hand_written_sizes_agree_with_mirror() {
    for (name, id) in PACKET_IDS_0104 {
        let Some(fixed) = fixed_payload_size(*id) else {
            continue;
        };
        let declared = declared_struct_size(*id).unwrap();
        if fixed == declared {
            continue;
        }
        // A hand-written codec may follow the pinned server only where the
        // clean client and OpenFusion are known to disagree.
        let openfusion = OPENFUSION_SIZE_DIVERGENCES_0104
            .iter()
            .find(|(_, divergent_id, _, _)| divergent_id == id)
            .map(|(_, _, _, openfusion)| *openfusion);
        assert_eq!(
            Some(fixed),
            openfusion,
            "{name}: fixed size {fixed} matches neither the clean client ({declared}) nor a documented OpenFusion divergence"
        );
    }
}
