//! FusionFall beta-20100104 TCP wire protocol.
//!
//! This crate models the wire ABI explicitly: little-endian values, fixed UTF-16 arrays, and
//! the original C `#pragma pack(4)` padding. Compatibility references are OpenFusion's
//! `CNProtocol.cpp`, `Defines.hpp`, and `structs/0104.hpp`.

#![forbid(unsafe_code)]

use std::{fmt, mem::size_of};

mod login_extended;
mod registry;
pub mod npc_map;
pub mod wire_0104;

pub use login_extended::{
    CharacterNameChangeFailure0104, CharacterNameChangeRequest0104, CharacterNameChangeSuccess0104,
    DuplicateExitReply0104, DuplicateExitRequest0104,
};
pub use registry::{
    OPENFUSION_SHARD_REQUESTS_0104, RegisteredGameplayRequest0104,
    RegisteredGameplayRequestError0104, RegisteredRequestLayout0104, RegisteredShardRequest0104,
    ShardRequestFamily0104, registered_request_layout_0104, registered_shard_request_0104,
};

/// IDs required by the first login-to-world vertical slice.
pub mod packet {
    pub const P_CL2LS_REQ_LOGIN: u32 = 0x1200_0001;
    pub const P_CL2LS_REQ_CHECK_CHAR_NAME: u32 = 0x1200_0002;
    pub const P_CL2LS_REQ_SAVE_CHAR_NAME: u32 = 0x1200_0003;
    pub const P_CL2LS_REQ_CHAR_CREATE: u32 = 0x1200_0004;
    pub const P_CL2LS_REQ_CHAR_SELECT: u32 = 0x1200_0005;
    pub const P_CL2LS_REQ_CHAR_DELETE: u32 = 0x1200_0006;
    pub const P_CL2LS_REQ_SAVE_CHAR_TUTOR: u32 = 0x1200_000a;
    pub const P_CL2LS_REQ_PC_EXIT_DUPLICATE: u32 = 0x1200_000b;
    pub const P_CL2LS_REP_LIVE_CHECK: u32 = 0x1200_000c;
    pub const P_CL2LS_REQ_CHANGE_CHAR_NAME: u32 = 0x1200_000d;

    pub const P_CL2FE_REQ_PC_ENTER: u32 = 0x1300_0001;
    pub const P_CL2FE_REQ_PC_EXIT: u32 = 0x1300_0002;
    pub const P_CL2FE_REQ_PC_MOVE: u32 = 0x1300_0003;
    pub const P_CL2FE_REQ_PC_STOP: u32 = 0x1300_0004;
    pub const P_CL2FE_REQ_PC_JUMP: u32 = 0x1300_0005;
    pub const P_CL2FE_REQ_PC_JUMPPAD: u32 = 0x1300_003d;
    pub const P_CL2FE_REQ_PC_LAUNCHER: u32 = 0x1300_003e;
    pub const P_CL2FE_REQ_PC_ZIPLINE: u32 = 0x1300_003f;
    pub const P_CL2FE_REQ_PC_MOVEPLATFORM: u32 = 0x1300_0040;
    pub const P_CL2FE_REQ_PC_SLOPE: u32 = 0x1300_0041;
    pub const P_CL2FE_REQ_PC_MOVETRANSPORTATION: u32 = 0x1300_0062;
    pub const P_CL2FE_REQ_BARKER: u32 = 0x1300_0065;
    pub const P_CL2FE_REQ_PC_ATTACK_NPCS: u32 = 0x1300_0006;
    pub const P_CL2FE_REQ_PC_FREECHAT: u32 = 0x1300_0007;
    pub const P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE: u32 = 0x1300_0008;
    pub const P_CL2FE_REQ_PC_REGEN: u32 = 0x1300_0009;
    pub const P_CL2FE_REQ_ITEM_MOVE: u32 = 0x1300_000a;
    pub const P_CL2FE_REQ_PC_TASK_START: u32 = 0x1300_000b;
    pub const P_CL2FE_REQ_PC_TASK_END: u32 = 0x1300_000c;
    pub const P_CL2FE_REQ_NANO_EQUIP: u32 = 0x1300_000d;
    pub const P_CL2FE_REQ_NANO_UNEQUIP: u32 = 0x1300_000e;
    pub const P_CL2FE_REQ_NANO_ACTIVE: u32 = 0x1300_000f;
    pub const P_CL2FE_REQ_NANO_TUNE: u32 = 0x1300_0010;
    pub const P_CL2FE_REQ_NANO_SKILL_USE: u32 = 0x1300_0011;
    pub const P_CL2FE_REQ_PC_TASK_STOP: u32 = 0x1300_0012;
    pub const P_CL2FE_REQ_PC_VENDOR_ITEM_BUY: u32 = 0x1300_0017;
    pub const P_CL2FE_REQ_PC_VENDOR_ITEM_SELL: u32 = 0x1300_0018;
    pub const P_CL2FE_REQ_PC_ITEM_DELETE: u32 = 0x1300_0019;
    pub const P_CL2FE_REQ_PC_BANK_OPEN: u32 = 0x1300_002e;
    pub const P_CL2FE_REQ_PC_BANK_CLOSE: u32 = 0x1300_002f;
    pub const P_CL2FE_REQ_PC_VENDOR_START: u32 = 0x1300_0030;
    pub const P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE: u32 = 0x1300_0031;
    pub const P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY: u32 = 0x1300_0032;
    pub const P_CL2FE_REQ_PC_COMBAT_BEGIN: u32 = 0x1300_0033;
    pub const P_CL2FE_REQ_PC_COMBAT_END: u32 = 0x1300_0034;
    pub const P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE: u32 = 0x1300_001c;
    pub const P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE: u32 = 0x1300_001f;
    pub const P_CL2FE_REQ_REQUEST_MAKE_BUDDY: u32 = 0x1300_0035;
    pub const P_CL2FE_REQ_ACCEPT_MAKE_BUDDY: u32 = 0x1300_0036;
    pub const P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE: u32 = 0x1300_0037;
    pub const P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE: u32 = 0x1300_0038;
    pub const P_CL2FE_REQ_SET_BUDDY_BLOCK: u32 = 0x1300_003a;
    pub const P_CL2FE_REQ_REMOVE_BUDDY: u32 = 0x1300_003b;
    pub const P_CL2FE_REQ_GET_BUDDY_STATE: u32 = 0x1300_003c;
    pub const P_CL2FE_REQ_ITEM_CHEST_OPEN: u32 = 0x1300_0047;
    pub const P_CL2FE_DOT_DAMAGE_ONOFF: u32 = 0x1300_0049;
    pub const P_CL2FE_REQ_PC_WARP_USE_NPC: u32 = 0x1300_004b;
    pub const P_CL2FE_REQ_PC_GROUP_LEAVE: u32 = 0x1300_004f;
    pub const P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT: u32 = 0x1300_0050;
    pub const P_CL2FE_REQ_PC_BUDDY_WARP: u32 = 0x1300_0051;
    pub const P_CL2FE_REQ_PC_CHANGE_MENTOR: u32 = 0x1300_0054;
    pub const P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY: u32 = 0x1300_004a;
    pub const P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE: u32 = 0x1300_0063;
    pub const P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE: u32 = 0x1300_0066;
    pub const P_CL2FE_GM_REQ_PC_SET_VALUE: u32 = 0x1300_006b;
    pub const P_CL2FE_REQ_ITEM_USE: u32 = 0x1300_0073;
    pub const P_CL2FE_REP_LIVE_CHECK: u32 = 0x1300_0075;
    pub const P_CL2FE_REQ_NPC_INTERACTION: u32 = 0x1300_0078;
    pub const P_CL2FE_DOT_HEAL_ONOFF: u32 = 0x1300_0079;
    pub const P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH: u32 = 0x1300_007a;
    pub const P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID: u32 = 0x1300_0083;
    pub const P_CL2FE_REQ_PC_LOADING_COMPLETE: u32 = 0x1300_008d;
    pub const P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY: u32 = 0x1300_008e;
    pub const P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY: u32 = 0x1300_008f;
    pub const P_CL2FE_REQ_PC_VEHICLE_ON: u32 = 0x1300_009f;
    pub const P_CL2FE_REQ_PC_VEHICLE_OFF: u32 = 0x1300_00a0;
    pub const P_CL2FE_REQ_PC_REGIST_QUICK_SLOT: u32 = 0x1300_00a1;
    pub const P_CL2FE_REQ_PC_DISASSEMBLE_ITEM: u32 = 0x1300_00a2;
    pub const P_CL2FE_REQ_PRESENT_NPC_TYPES: u32 = 0x1300_00a7;

    pub const P_LS2CL_REP_LOGIN_SUCC: u32 = 0x2100_0001;
    pub const P_LS2CL_REP_LOGIN_FAIL: u32 = 0x2100_0002;
    pub const P_LS2CL_REP_CHAR_INFO: u32 = 0x2100_0003;
    pub const P_LS2CL_REP_CHECK_CHAR_NAME_SUCC: u32 = 0x2100_0005;
    pub const P_LS2CL_REP_CHECK_CHAR_NAME_FAIL: u32 = 0x2100_0006;
    pub const P_LS2CL_REP_SAVE_CHAR_NAME_SUCC: u32 = 0x2100_0007;
    pub const P_LS2CL_REP_SAVE_CHAR_NAME_FAIL: u32 = 0x2100_0008;
    pub const P_LS2CL_REP_CHAR_CREATE_SUCC: u32 = 0x2100_0009;
    pub const P_LS2CL_REP_CHAR_CREATE_FAIL: u32 = 0x2100_000a;
    pub const P_LS2CL_REP_CHAR_DELETE_SUCC: u32 = 0x2100_000d;
    pub const P_LS2CL_REP_CHAR_DELETE_FAIL: u32 = 0x2100_000e;
    pub const P_LS2CL_REP_SHARD_SELECT_SUCC: u32 = 0x2100_000f;
    pub const P_LS2CL_REP_SHARD_SELECT_FAIL: u32 = 0x2100_0010;
    pub const P_LS2CL_REQ_LIVE_CHECK: u32 = 0x2100_0016;
    pub const P_LS2CL_REP_PC_EXIT_DUPLICATE: u32 = 0x2100_0015;
    pub const P_LS2CL_REP_CHANGE_CHAR_NAME_SUCC: u32 = 0x2100_0017;
    pub const P_LS2CL_REP_CHANGE_CHAR_NAME_FAIL: u32 = 0x2100_0018;

    pub const P_FE2CL_REP_PC_ENTER_FAIL: u32 = 0x3100_0001;
    pub const P_FE2CL_REP_PC_ENTER_SUCC: u32 = 0x3100_0002;
    pub const P_FE2CL_PC_NEW: u32 = 0x3100_0003;
    pub const P_FE2CL_REP_PC_EXIT_FAIL: u32 = 0x3100_0004;
    pub const P_FE2CL_REP_PC_EXIT_SUCC: u32 = 0x3100_0005;
    pub const P_FE2CL_PC_EXIT: u32 = 0x3100_0006;
    pub const P_FE2CL_PC_AROUND: u32 = 0x3100_0007;
    pub const P_FE2CL_PC_MOVE: u32 = 0x3100_0008;
    pub const P_FE2CL_PC_STOP: u32 = 0x3100_0009;
    pub const P_FE2CL_PC_JUMP: u32 = 0x3100_000a;
    pub const P_FE2CL_NPC_ENTER: u32 = 0x3100_000b;
    pub const P_FE2CL_NPC_EXIT: u32 = 0x3100_000c;
    pub const P_FE2CL_NPC_MOVE: u32 = 0x3100_000d;
    pub const P_FE2CL_NPC_NEW: u32 = 0x3100_000e;
    pub const P_FE2CL_NPC_AROUND: u32 = 0x3100_000f;
    pub const P_FE2CL_AROUND_DEL_PC: u32 = 0x3100_0010;
    pub const P_FE2CL_AROUND_DEL_NPC: u32 = 0x3100_0011;
    pub const P_FE2CL_REP_PC_FREECHAT_SUCC: u32 = 0x3100_0012;
    pub const P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC: u32 = 0x3100_0018;
    pub const P_FE2CL_PC_ATTACK_NPCS_SUCC: u32 = 0x3100_0014;
    pub const P_FE2CL_PC_ATTACK_NPCS: u32 = 0x3100_0015;
    pub const P_FE2CL_NPC_ATTACK_PCS: u32 = 0x3100_0016;
    pub const P_FE2CL_REP_PC_ROCKET_STYLE_FIRE_SUCC: u32 = 0x3100_003b;
    pub const P_FE2CL_PC_ROCKET_STYLE_FIRE: u32 = 0x3100_003c;
    pub const P_FE2CL_REP_PC_GRENADE_STYLE_FIRE_SUCC: u32 = 0x3100_003f;
    pub const P_FE2CL_PC_GRENADE_STYLE_FIRE: u32 = 0x3100_0040;
    pub const P_FE2CL_NPC_SKILL_READY: u32 = 0x3100_0020;
    pub const P_FE2CL_NPC_SKILL_FIRE: u32 = 0x3100_0021;
    pub const P_FE2CL_NPC_SKILL_HIT: u32 = 0x3100_0022;
    pub const P_FE2CL_NPC_SKILL_CORRUPTION_READY: u32 = 0x3100_0023;
    pub const P_FE2CL_NPC_SKILL_CORRUPTION_HIT: u32 = 0x3100_0024;
    pub const P_FE2CL_NPC_SKILL_CANCEL: u32 = 0x3100_0025;
    pub const P_FE2CL_REP_BARKER: u32 = 0x3100_00b9;
    pub const P_FE2CL_REP_NANO_ACTIVE_SUCC: u32 = 0x3100_0028;
    pub const P_FE2CL_NANO_SKILL_USE_SUCC: u32 = 0x3100_002b;
    pub const P_FE2CL_NANO_SKILL_USE: u32 = 0x3100_002c;
    pub const P_FE2CL_REP_PC_REGEN_SUCC: u32 = 0x3100_0017;
    pub const P_FE2CL_PC_ITEM_MOVE_SUCC: u32 = 0x3100_001a;
    pub const P_FE2CL_PC_EQUIP_CHANGE: u32 = 0x3100_001b;
    pub const P_FE2CL_REP_PC_TASK_START_SUCC: u32 = 0x3100_001c;
    pub const P_FE2CL_REP_PC_TASK_START_FAIL: u32 = 0x3100_001d;
    pub const P_FE2CL_REP_PC_TASK_END_SUCC: u32 = 0x3100_001e;
    pub const P_FE2CL_REP_PC_TASK_END_FAIL: u32 = 0x3100_001f;
    pub const P_FE2CL_REP_PC_VENDOR_ITEM_BUY_SUCC: u32 = 0x3100_0035;
    pub const P_FE2CL_REP_PC_VENDOR_ITEM_BUY_FAIL: u32 = 0x3100_0036;
    pub const P_FE2CL_REP_PC_VENDOR_ITEM_SELL_SUCC: u32 = 0x3100_0037;
    pub const P_FE2CL_REP_PC_VENDOR_ITEM_SELL_FAIL: u32 = 0x3100_0038;
    pub const P_FE2CL_REP_PC_ITEM_DELETE_SUCC: u32 = 0x3100_0039;
    pub const P_FE2CL_REP_NANO_TUNE_SUCC: u32 = 0x3100_0029;
    pub const P_FE2CL_REP_NANO_TUNE_FAIL: u32 = 0x3100_0055;
    pub const P_FE2CL_REP_PC_TASK_STOP_SUCC: u32 = 0x3100_002d;
    pub const P_FE2CL_REP_PC_TASK_STOP_FAIL: u32 = 0x3100_002e;
    pub const P_FE2CL_REP_PC_GOTO_SUCC: u32 = 0x3100_0031;
    pub const P_FE2CL_REP_PC_TICK: u32 = 0x3100_0033;
    pub const P_FE2CL_REP_PC_KILL_QUEST_NPCS_SUCC: u32 = 0x3100_0034;
    pub const P_FE2CL_REP_PC_NANO_CREATE_SUCC: u32 = 0x3100_0053;
    pub const P_FE2CL_REP_PC_NANO_CREATE_FAIL: u32 = 0x3100_0054;
    pub const P_FE2CL_REP_PC_BANK_OPEN_SUCC: u32 = 0x3100_0056;
    pub const P_FE2CL_REP_PC_BANK_OPEN_FAIL: u32 = 0x3100_0057;
    pub const P_FE2CL_REP_PC_BANK_CLOSE_SUCC: u32 = 0x3100_0058;
    pub const P_FE2CL_REP_PC_BANK_CLOSE_FAIL: u32 = 0x3100_0059;
    pub const P_FE2CL_REP_PC_VENDOR_START_SUCC: u32 = 0x3100_005a;
    pub const P_FE2CL_REP_PC_VENDOR_START_FAIL: u32 = 0x3100_005b;
    pub const P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC: u32 = 0x3100_005c;
    pub const P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_FAIL: u32 = 0x3100_005d;
    pub const P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_SUCC: u32 = 0x3100_005e;
    pub const P_FE2CL_REP_PC_VENDOR_ITEM_RESTORE_BUY_FAIL: u32 = 0x3100_005f;
    pub const P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC: u32 = 0x3100_0063;
    pub const P_FE2CL_REP_PC_BUDDYLIST_INFO_FAIL: u32 = 0x3100_0064;
    /// The clean Retrobution `csDefines` value. OpenFusion's generated 0104
    /// enum currently substitutes `0x7fff_ffff`, which is not the client ABI.
    pub const P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC: u32 = 0x8300_0065;
    pub const P_FE2CL_REP_REQUEST_MAKE_BUDDY_FAIL: u32 = 0x3100_0066;
    pub const P_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC: u32 = 0x3100_0067;
    pub const P_FE2CL_REP_ACCEPT_MAKE_BUDDY_FAIL: u32 = 0x3100_0068;
    pub const P_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC: u32 = 0x3100_0069;
    pub const P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC: u32 = 0x3100_006b;
    pub const P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT: u32 = 0x3100_0092;
    pub const P_FE2CL_REP_GET_BUDDY_STATE_SUCC: u32 = 0x3100_006f;
    pub const P_FE2CL_REP_GET_BUDDY_STATE_FAIL: u32 = 0x3100_0070;
    pub const P_FE2CL_REP_SET_BUDDY_BLOCK_SUCC: u32 = 0x3100_0071;
    pub const P_FE2CL_REP_SET_BUDDY_BLOCK_FAIL: u32 = 0x3100_0072;
    pub const P_FE2CL_REP_REMOVE_BUDDY_SUCC: u32 = 0x3100_0073;
    pub const P_FE2CL_REP_REMOVE_BUDDY_FAIL: u32 = 0x3100_0074;
    pub const P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER: u32 = 0x3100_007b;
    pub const P_FE2CL_REP_REWARD_ITEM: u32 = 0x3100_007c;
    pub const P_FE2CL_REP_ITEM_CHEST_OPEN_SUCC: u32 = 0x3100_007d;
    pub const P_FE2CL_REP_ITEM_CHEST_OPEN_FAIL: u32 = 0x3100_007e;
    pub const P_FE2CL_CHAR_TIME_BUFF_TIME_OUT: u32 = 0x3100_0060;
    pub const P_FE2CL_REP_PC_CHANGE_MENTOR_SUCC: u32 = 0x3100_0093;
    pub const P_FE2CL_CHAR_TIME_BUFF_TIME_TICK: u32 = 0x3100_007f;
    pub const P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_SUCC: u32 = 0x3100_0080;
    pub const P_FE2CL_REP_PC_VENDOR_BATTERY_BUY_FAIL: u32 = 0x3100_0081;
    pub const P_FE2CL_PC_GROUP_JOIN: u32 = 0x3100_0089;
    pub const P_FE2CL_PC_GROUP_JOIN_SUCC: u32 = 0x3100_008b;
    pub const P_FE2CL_PC_GROUP_LEAVE: u32 = 0x3100_008c;
    pub const P_FE2CL_PC_GROUP_LEAVE_SUCC: u32 = 0x3100_008e;
    pub const P_FE2CL_PC_GROUP_MEMBER_INFO: u32 = 0x3100_008f;
    pub const P_FE2CL_REP_PC_WARP_USE_NPC_SUCC: u32 = 0x3100_0090;
    pub const P_FE2CL_REP_PC_WARP_USE_NPC_FAIL: u32 = 0x3100_0091;
    pub const P_FE2CL_REP_PC_CHANGE_MENTOR_FAIL: u32 = 0x3100_0094;
    pub const P_FE2CL_REP_PC_SET_CURRENT_MISSION_ID: u32 = 0x3100_00ef;
    pub const P_FE2CL_PC_REGEN: u32 = 0x3100_0099;
    pub const P_FE2CL_TRANSPORTATION_ENTER: u32 = 0x3100_009b;
    pub const P_FE2CL_TRANSPORTATION_EXIT: u32 = 0x3100_009c;
    pub const P_FE2CL_TRANSPORTATION_MOVE: u32 = 0x3100_009d;
    pub const P_FE2CL_TRANSPORTATION_NEW: u32 = 0x3100_009e;
    pub const P_FE2CL_TRANSPORTATION_AROUND: u32 = 0x3100_009f;
    pub const P_FE2CL_AROUND_DEL_TRANSPORTATION: u32 = 0x3100_00a0;
    pub const P_FE2CL_SHINY_ENTER: u32 = 0x3100_00ad;
    pub const P_FE2CL_SHINY_EXIT: u32 = 0x3100_00ae;
    pub const P_FE2CL_SHINY_NEW: u32 = 0x3100_00af;
    pub const P_FE2CL_SHINY_AROUND: u32 = 0x3100_00b0;
    pub const P_FE2CL_AROUND_DEL_SHINY: u32 = 0x3100_00b1;
    pub const P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC: u32 = 0x3100_00b5;
    pub const P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC: u32 = 0x3100_00ba;
    pub const P_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC: u32 = 0x3100_00c3;
    pub const P_FE2CL_PC_SPECIAL_STATE_CHANGE: u32 = 0x3100_00c4;
    pub const P_FE2CL_GM_REP_PC_SET_VALUE: u32 = 0x3100_00c5;
    pub const P_FE2CL_REP_PC_BUDDY_WARP_FAIL: u32 = 0x3100_00c9;
    pub const P_FE2CL_REQ_LIVE_CHECK: u32 = 0x3100_00d0;
    pub const P_FE2CL_PC_MOTD_LOGIN: u32 = 0x3100_00d1;
    pub const P_FE2CL_REP_PC_ITEM_USE_FAIL: u32 = 0x3100_00d2;
    pub const P_FE2CL_REP_PC_ITEM_USE_SUCC: u32 = 0x3100_00d3;
    pub const P_FE2CL_PC_ITEM_USE: u32 = 0x3100_00d4;
    pub const P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC: u32 = 0x3100_00db;
    pub const P_FE2CL_PC_BUFF_UPDATE: u32 = 0x3100_00df;
    pub const P_FE2CL_PC_SUDDEN_DEAD: u32 = 0x3100_00ed;
    pub const P_FE2CL_REP_NPC_GROUP_INVITE_SUCC: u32 = 0x3100_00f1;
    pub const P_FE2CL_REP_NPC_GROUP_KICK_SUCC: u32 = 0x3100_00f3;
    pub const P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC: u32 = 0x3100_00f9;
    pub const P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC: u32 = 0x3100_00fe;
    pub const P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL: u32 = 0x3100_00ff;
    pub const P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL: u32 = 0x3100_0100;
    pub const P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC: u32 = 0x3100_0101;
    pub const P_FE2CL_REP_PC_CHANGE_LEVEL_SUCC: u32 = 0x3100_0105;
    pub const P_FE2CL_PC_CASH_BUFF_UPDATE: u32 = 0x3100_0118;
    pub const P_FE2CL_PC_VEHICLE_ON_SUCC: u32 = 0x3100_0122;
    pub const P_FE2CL_PC_VEHICLE_ON_FAIL: u32 = 0x3100_0123;
    pub const P_FE2CL_PC_VEHICLE_OFF_SUCC: u32 = 0x3100_0124;
    pub const P_FE2CL_PC_VEHICLE_OFF_FAIL: u32 = 0x3100_0125;
    pub const P_FE2CL_PC_QUICK_SLOT_INFO: u32 = 0x3100_0126;
    pub const P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL: u32 = 0x3100_0127;
    pub const P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC: u32 = 0x3100_0128;
    pub const P_FE2CL_REP_PC_DISASSEMBLE_ITEM_SUCC: u32 = 0x3100_012a;
    pub const P_FE2CL_REP_PC_DISASSEMBLE_ITEM_FAIL: u32 = 0x3100_012b;
    pub const P_FE2CL_REP_NANO_BOOK_SUBSET: u32 = 0x3100_0134;
    pub const P_FE2CL_REP_PRESENT_NPC_TYPES: u32 = 0x3100_0137;

    include!("packet_ids_wire_0104.rs");
}

macro_rules! impl_item_use_skill_result_record {
    ($($record:ty),+ $(,)?) => {
        $(
            impl ItemUseSkillResultRecord0104 for $record {
                const SIZE: usize = <$record>::SIZE;

                fn decode_exact(bytes: &[u8]) -> Self {
                    <$record>::decode_exact(bytes)
                }
            }
        )+
    };
}

impl_item_use_skill_result_record!(
    SkillResultDamage0104,
    SkillResultHealHp0104,
    SkillResultHealStamina0104,
    SkillResultDamageDebuff0104,
    SkillResultBuff0104,
    SkillResultBatteryDrain0104,
    SkillResultResurrect0104,
    SkillResultMove0104,
);

macro_rules! buddy_uid_slot_payload_0104 {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub buddy_pc_uid: i64,
            pub buddy_slot: i8,
        }

        impl WirePayload for $name {
            const SIZE: usize = 12;

            fn encode(&self) -> Vec<u8> {
                let mut out = vec![0; Self::SIZE];
                write_i64(&mut out, 0, self.buddy_pc_uid);
                out[8] = self.buddy_slot as u8;
                out
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self {
                    buddy_pc_uid: read_i64(bytes, 0),
                    buddy_slot: bytes[8] as i8,
                })
            }
        }
    };
}

buddy_uid_slot_payload_0104!(
    BuddySetBlockRequest0104,
    "Protocol-0104 `sP_CL2FE_REQ_SET_BUDDY_BLOCK`."
);
buddy_uid_slot_payload_0104!(
    BuddyRemoveRequest0104,
    "Protocol-0104 `sP_CL2FE_REQ_REMOVE_BUDDY`."
);

buddy_uid_slot_payload_0104!(
    BuddyWarpRequest0104,
    "Protocol-0104 `sP_CL2FE_REQ_PC_BUDDY_WARP`. The clean source calls byte 8 \
     `iSlotNum`; the native API uses the same buddy-slot semantics as block/remove."
);

macro_rules! buddy_error_payload_0104 {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub error_code: i32,
        }

        impl WirePayload for $name {
            const SIZE: usize = 4;

            fn encode(&self) -> Vec<u8> {
                self.error_code.to_le_bytes().to_vec()
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self {
                    error_code: read_i32(bytes, 0),
                })
            }
        }
    };
}

buddy_error_payload_0104!(
    BuddyListFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_PC_BUDDYLIST_INFO_FAIL`."
);
buddy_error_payload_0104!(
    BuddyStateFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_GET_BUDDY_STATE_FAIL`."
);

macro_rules! buddy_peer_failure_payload_0104 {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub buddy_id: i32,
            pub buddy_pc_uid: i64,
            pub error_code: i32,
        }

        impl WirePayload for $name {
            const SIZE: usize = 16;

            fn encode(&self) -> Vec<u8> {
                let mut out = vec![0; Self::SIZE];
                write_i32(&mut out, 0, self.buddy_id);
                write_i64(&mut out, 4, self.buddy_pc_uid);
                write_i32(&mut out, 12, self.error_code);
                out
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self {
                    buddy_id: read_i32(bytes, 0),
                    buddy_pc_uid: read_i64(bytes, 4),
                    error_code: read_i32(bytes, 12),
                })
            }
        }
    };
}

buddy_peer_failure_payload_0104!(
    BuddyMakeFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_REQUEST_MAKE_BUDDY_FAIL`."
);
buddy_peer_failure_payload_0104!(
    BuddyAcceptFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_ACCEPT_MAKE_BUDDY_FAIL`."
);

macro_rules! buddy_uid_slot_success_payload_0104 {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub buddy_pc_uid: i64,
            pub buddy_slot: i8,
        }

        impl WirePayload for $name {
            const SIZE: usize = 12;

            fn encode(&self) -> Vec<u8> {
                let mut out = vec![0; Self::SIZE];
                write_i64(&mut out, 0, self.buddy_pc_uid);
                out[8] = self.buddy_slot as u8;
                out
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self {
                    buddy_pc_uid: read_i64(bytes, 0),
                    buddy_slot: bytes[8] as i8,
                })
            }
        }
    };
}

buddy_uid_slot_success_payload_0104!(
    BuddyBlockSuccess0104,
    "Protocol-0104 `sP_FE2CL_REP_SET_BUDDY_BLOCK_SUCC`."
);
buddy_uid_slot_success_payload_0104!(
    BuddyRemoveSuccess0104,
    "Protocol-0104 `sP_FE2CL_REP_REMOVE_BUDDY_SUCC`."
);

macro_rules! buddy_uid_error_payload_0104 {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub buddy_pc_uid: i64,
            pub error_code: i32,
        }

        impl WirePayload for $name {
            const SIZE: usize = 12;

            fn encode(&self) -> Vec<u8> {
                let mut out = vec![0; Self::SIZE];
                write_i64(&mut out, 0, self.buddy_pc_uid);
                write_i32(&mut out, 8, self.error_code);
                out
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self {
                    buddy_pc_uid: read_i64(bytes, 0),
                    error_code: read_i32(bytes, 8),
                })
            }
        }
    };
}

buddy_uid_error_payload_0104!(
    BuddyBlockFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_SET_BUDDY_BLOCK_FAIL`."
);
buddy_uid_error_payload_0104!(
    BuddyRemoveFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_REMOVE_BUDDY_FAIL`."
);
buddy_uid_error_payload_0104!(
    BuddyWarpFailure0104,
    "Protocol-0104 `sP_FE2CL_REP_PC_BUDDY_WARP_FAIL`."
);

macro_rules! i32_payload {
    ($name:ident, $field:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub $field: i32,
        }

        impl WirePayload for $name {
            const SIZE: usize = 4;

            fn encode(&self) -> Vec<u8> {
                self.$field.to_le_bytes().to_vec()
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self {
                    $field: read_i32(bytes, 0),
                })
            }
        }
    };
}

i32_payload!(ShardSelectFailure, error_code);
i32_payload!(CharacterNameCheckFailure0104, error_code);
i32_payload!(CharacterNameSaveFailure0104, error_code);
i32_payload!(CharacterCreateFailure0104, error_code);
i32_payload!(CharacterDeleteFailure0104, error_code);
i32_payload!(PcEnterFailure, error_code);
i32_payload!(PcLoadingCompleteRequest, pc_id);
i32_payload!(PcLoadingCompleteSuccess, pc_id);

macro_rules! vehicle_toggle_request_0104 {
    ($name:ident, $packet:literal) => {
        #[doc = concat!("Exact protocol-0104 `", $packet, "` (`#pragma pack(1)`, 1 byte).")]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name {
            pub unused: u8,
        }

        impl WirePayload for $name {
            const SIZE: usize = 1;

            fn encode(&self) -> Vec<u8> {
                vec![self.unused]
            }

            fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
                require_size(bytes, Self::SIZE)?;
                Ok(Self { unused: bytes[0] })
            }
        }
    };
}

vehicle_toggle_request_0104!(PcVehicleOnRequest0104, "sP_CL2FE_REQ_PC_VEHICLE_ON");
vehicle_toggle_request_0104!(PcVehicleOffRequest0104, "sP_CL2FE_REQ_PC_VEHICLE_OFF");

#[cfg(test)]
mod tests;

#[cfg(test)]
mod healing_tick_regression;

#[cfg(test)]
mod warhead_hit_tests;

mod codec_constants;
mod codec_codec_decode_buddy_lifecycle_packet;
mod codec_codec_decode_nano_skill_result;
mod codec_codec_fixed_payload_size;
mod codec_operations;
mod codec_validation;
mod codec_types_item_use_success_prefix0104;
mod codec_types_buddy_list_info0104;
mod codec_types_character_info0104;
mod codec_types_pc_zipline_request0104;
mod codec_types_nano_skill_use_success0104;
mod codec_types_pc_bank_open_success0104;
mod codec_types_reward_item_reply0104;
mod codec_types_present_npc_types_reply0104;
mod codec_types_npc_skill_hit0104;
mod codec_types_npc_skill_corruption_hit0104;
mod codec_input;
mod codec_state;
mod codec_commands;
mod codec_output;
mod codec_systems;

pub use codec_constants::{
    PROTOCOL_VERSION, DEFAULT_KEY_BYTES, DEFAULT_KEY, MAX_BODY_SIZE_0104,
    QUICK_SLOT_COUNT_0104, GM_SET_VALUE_SPEED_0104, BUDDY_LIST_CAPACITY_0104,
    CHARACTER_EQUIP_SLOT_COUNT_0104, NANO_TUNE_ITEM_SLOT_COUNT_0104,
    NANO_SKILL_RESULT_MAX_TARGETS_0104, VENDOR_TABLE_ITEM_COUNT_0104, BANK_SLOT_COUNT_0104
};
pub use codec_codec_decode_buddy_lifecycle_packet::{
    PACKET_TYPE_MASK, PACKET_FLAGS_MASK, PACKET_FLAGS_SHIFT, MAX_PACKET_FLAGS, FrameError,
    DecodedFrame, packet_checksum, encode_server_frame, encode_client_frame, decode_frame,
    decode_server_frame, decode_client_frame, FrameBuffer, PayloadError, WirePayload,
    ItemUseDecodeError0104, decode_item_use_packet_0104, decode_quick_slot_packet_0104,
    decode_skill_buff_packet_0104, decode_freechat_packet_0104, decode_menu_chat_packet_0104,
    decode_avatar_emote_chat_0104, decode_server_message_0104, decode_gm_set_value_reply_0104,
    decode_buddy_lifecycle_packet_0104, decode_buddy_freechat_packet_0104,
    decode_buddy_menu_chat_packet_0104, decode_all_group_freechat_packet_0104,
    decode_all_group_menu_chat_packet_0104, decode_special_state_change_0104
};
use codec_codec_decode_buddy_lifecycle_packet::decode_item_use_skill_results_0104;
pub use codec_codec_decode_nano_skill_result::{
    decode_nano_active_success_0104, NanoSkillUseDecodeError0104,
    decode_nano_skill_use_packet_0104, decode_nano_skill_use_success_0104,
    decode_nano_tune_packet_0104, decode_vendor_packet_0104, decode_pc_bank_reply_0104,
    decode_inventory_packet_0104, decode_pc_nano_create_packet_0104,
    decode_pc_regen_packet_0104, decode_pc_exit_packet_0104, CountedPayloadError0104,
    PresentNpcTypesDecodeError0104, decode_present_npc_types_packet_0104,
    decode_group_packet_0104, decode_pc_warhead_fire_packet_0104, decode_pc_warhead_hit_0104,
    decode_npc_skill_signal_0104, decode_npc_skill_authority_0104,
    NpcSkillAuthorityDecodeError0104
};
use codec_codec_decode_nano_skill_result::{
    decode_nano_skill_target_0104, decode_nano_skill_result_0104
};
pub use codec_codec_fixed_payload_size::{
    NpcCombatDecodeError0104, decode_npc_combat_packet_0104, AroundDecodeError,
    decode_initial_around_0104, decode_pc_around_0104, decode_npc_around_0104,
    decode_transportation_around_0104, decode_shiny_around_0104, fixed_payload_size
};
use codec_codec_fixed_payload_size::{
    counted_payload_len, allocate_counted_payload, decode_counted_i32, encode_attack_results,
    encode_attack_results_after, decode_attack_results, decode_attack_results_after,
    OPENFUSION_PAYLOAD_CAPACITY_0104
};
pub use codec_operations::{
    derive_key, derive_login_e_key, derive_frontend_key, derive_shard_e_key, encrypt_in_place,
    decrypt_in_place, nano_skill_result_size_0104, npc_skill_result_size_0104,
    is_client_to_server_0104
};
use codec_operations::checked_npc_skill_target_count;
#[cfg(test)]
use codec_codec_fixed_payload_size::checked_around_payload_len;
use codec_validation::{
    validate_body_len, validate_buddy_list_value, validate_buddy_list_end,
    require_counted_header, require_size, require_prefix
};
pub use codec_validation::{CharacterNameCheckRequest0104, CharacterNameCheckSuccess0104};
pub use codec_types_item_use_success_prefix0104::{
    LegacyClientEncoder, QuickSlotEntry0104, QuickSlotInfo0104, QuickSlotRegisterRequest0104,
    QuickSlotRegisterSuccess0104, QuickSlotRegisterFailure0104, ItemUseRequest0104,
    ItemChestOpenRequest0104, ItemChestOpenSuccess0104, ItemChestOpenFailure0104,
    ItemUseFailure0104, ItemUseSuccessPrefix0104, ItemUseBroadcastPrefix0104,
    SkillResultDamage0104, SkillResultHealHp0104, SkillResultHealStamina0104,
    SkillResultDamageDebuff0104, SkillResultBuff0104, SkillResultBatteryDrain0104,
    SkillResultMove0104, SkillResultResurrect0104, ItemUseSkillResults0104,
    ItemUseSuccessPacket0104, ItemUseBroadcastPacket0104, ItemUsePacket0104,
    QuickSlotPacket0104
};
use codec_types_item_use_success_prefix0104::ItemUseSkillResultRecord0104;
pub use codec_types_buddy_list_info0104::{
    EnvironmentDotToggle0104, PcTick0104, TimeBuffDotDamageTick0104, TimeBuffHealTick0104,
    TimeBuff0104, PcBuffUpdate0104, PcCashBuffUpdate0104, CharTimeBuffTimeout0104,
    SkillBuffPacket0104, FixedUtf16, FreeChatRequest0104, FreeChatSuccess0104,
    FreeChatPacket0104, MenuChatRequest0104, MenuChatSuccess0104, MenuChatPacket0104,
    AvatarEmoteChat0104, ServerMessage0104, GmSetValueRequest0104, GmSetValueReply0104,
    BuddyBaseInfo0104, BuddyListInfoPrefix0104, BuddyListInfo0104, BuddyMakeRequest0104,
    BuddyAcceptRequest0104
};
pub use codec_types_character_info0104::{
    BuddyMakeSuccess0104, BuddyAcceptSuccess0104, BuddyIncomingRequest0104,
    BuddyWarpOtherShardSuccess0104, BuddyWarpSameShardSuccess0104, BuddyLifecyclePacket0104,
    BuddyFreeChatRequest0104, BuddyFreeChatSuccess0104, BuddyFreeChatPacket0104,
    BuddyMenuChatRequest0104, BuddyMenuChatSuccess0104, BuddyMenuChatPacket0104,
    GroupLeaveRequest0104, AllGroupFreeChatRequest0104, AllGroupFreeChatSuccess0104,
    AllGroupFreeChatPacket0104, AllGroupMenuChatRequest0104, AllGroupMenuChatSuccess0104,
    AllGroupMenuChatPacket0104, LoginSuccess, LoginFailure, CharacterInfo0104,
    CharacterStyle0104, EquippedItem0104, CharacterEquipSlot0104, CharacterDeleteRequest0104,
    CharacterDeleteSuccess0104, ShardSelectSuccess, PcMoveRequest0104
};
pub use codec_types_pc_zipline_request0104::{
    PcStopRequest0104, PcJumpRequest0104, PcJumppadRequest0104, PcLauncherRequest0104,
    PcMovePlatformRequest0104, PcMoveTransportationRequest0104, PcSlopeRequest0104,
    PcZiplineRequest0104, PcMove0104, PcStop0104, PcJump0104, RunningQuest0104,
    PcEnterSuccess, PcStyle0104, PcStyle2Flags0104, OnItem0104, OnItemIndex0104,
    CharacterCreateRequest0104
};
pub use codec_types_nano_skill_use_success0104::{
    CharacterCreateSuccess0104, ItemBase0104, NanoEquipRequest0104, NanoUnequipRequest0104,
    NanoActiveRequest0104, NanoActiveSuccess0104, NanoSkillUseSuccessPrefix0104,
    NanoSkillTarget0104, NanoSkillDamageResult0104, NanoSkillHealHpResult0104,
    NanoSkillDamageDebuffResult0104, NanoSkillBuffResult0104, NanoSkillBatteryDrainResult0104,
    NanoSkillMoveResult0104, NanoSkillResurrectResult0104, NanoSkillLeechResult0104,
    NanoSkillResult0104, NanoSkillUseSuccess0104, NanoSkillUseDelivery0104,
    NanoSkillUsePacket0104, NanoSkillUseRequest0104, NanoTuneRequest0104, NanoTuneSuccess0104,
    NanoTuneFailure0104, NanoTunePacket0104, ItemVendor0104, VendorItemBuyRequest0104,
    VendorItemSellRequest0104
};
pub use codec_types_pc_bank_open_success0104::{
    PcItemDeleteRequest0104, VendorStartRequest0104, VendorItemRestoreBuyRequest0104,
    VendorBatteryBuyRequest0104, PcDisassembleItemRequest0104, VendorItemBuySuccess0104,
    VendorFailure0104, VendorItemSellSuccess0104, PcItemDeleteSuccess0104,
    VendorStartSuccess0104, VendorItemRestoreBuySuccess0104, VendorBatteryBuySuccess0104,
    PcDisassembleItemSuccess0104, PcDisassembleItemFailure0104, VendorPacket0104,
    ItemMoveRequest0104, PcBankOpenRequest0104, PcBankCloseRequest0104, PcBankOpenSuccess0104,
    PcBankFailure0104, PcBankCloseSuccess0104, PcBankReply0104, ItemMoveSuccessPacket0104,
    EquipChangePacket0104, Nano0104, PcNanoCreateSuccess0104
};
pub use codec_types_reward_item_reply0104::{
    PcNanoCreateFailure0104, PcNanoCreatePacket0104, PcAppearance0104, PcNew0104,
    PcChangeMentorRequest0104, PcChangeMentorSuccess0104, PcChangeMentorFailure0104,
    PcWarpUseNpcRequest0104, PcWarpUseNpcSuccess0104, PcWarpUseNpcFailure0104,
    PcGotoSuccess0104, PcTaskStartRequest0104, PcTaskEndRequest0104, PcTaskStartSuccess0104,
    PcTaskFailure0104, PcTaskEndSuccess0104, PcKillQuestNpcsSuccess0104,
    PcSetCurrentMissionId0104, PcTaskStopRequest0104, PcTaskStopSuccess0104,
    PcTaskStopFailure0104, ItemReward0104, RewardItemReply0104, NpcInteractionRequest0104,
    PcRegenRequest0104
};
pub use codec_types_present_npc_types_reply0104::{
    PcRegenData0104, PcRegenSuccess0104, PcRegen0104, PcSuddenDead0104, PcRegenPacket0104,
    PcExitRequest0104, PcExitFailure0104, PcExitSuccess0104, PcExitPacket0104, PcExit0104,
    NpcAppearance0104, NpcEnter0104, NpcExit0104, NpcMove0104, NpcNew0104, AttackResult0104,
    PresentNpcTypesRequest0104, PresentNpcTypesReply0104, PresentNpcTypesPacket0104,
    GroupPcMemberInfo0104, GroupNpcMemberInfo0104, GroupRoster0104, GroupPacket0104,
    PcAttackNpcsRequest0104, PcRocketStyleFireRequest0104, PcGrenadeStyleFireRequest0104
};
pub use codec_types_npc_skill_hit0104::{
    PcBullet0104, PcRocketStyleFireSuccess0104, PcRocketStyleFire0104,
    PcGrenadeStyleFireSuccess0104, PcGrenadeStyleFire0104, PcWarheadFirePacket0104,
    AroundDelNpc0104, AroundDelPc0104, PcAttackNpcsSuccess0104, PcAttackNpcs0104,
    PcAttackCharsTarget0104, PcAttackCharsRequest0104, PcAttackCharsSuccess0104,
    PcAttackChars0104, NpcAttackChars0104, CharacterAttackCharacters0104, NpcAttackPcs0104,
    NpcBarker0104, NpcBarkerRequest0104, NpcSkillSignal0104, NpcSkillSignalKind0104,
    NpcSkillHitPrefix0104, NpcSkillHit0104
};
pub use codec_types_npc_skill_corruption_hit0104::{
    NpcSkillCorruptionHitPrefix0104, NpcSkillCorruptionResult0104, NpcSkillCorruptionHit0104,
    NpcSkillAuthorityPacket0104, NpcCombatPacket0104, TransportationAppearance0104,
    TransportationExit0104, TransportationMove0104, AroundDelTransportation0104,
    ShinyAppearance0104, ShinyExit0104, AroundDelShiny0104, InitialAroundPacket0104
};
pub use codec_input::{
    BuddyFindNameRequest0104, BuddyFindNameSuccess0104, BuddyFindNameFailure0104,
    BuddyFindNameAcceptRequest0104, BuddyFindNameAcceptFailure0104, PcLoadData0104
};
use codec_input::{read_utf16, read_i16, read_i32, read_i64, read_u64, read_f32};
pub use codec_state::{
    BuddyStateRequest0104, BuddyStateSuccess0104, SpecialStateChange0104, InventoryPacket0104,
    PcSpecialStateSwitchRequest0104, PcCombatStateRequest0104
};
pub use codec_commands::{LoginRequest, CharacterSelectRequest, PcEnterRequest};
pub use codec_output::{
    CharacterNameSaveRequest0104, CharacterNameSaveSuccess0104,
    CharacterTutorialSaveRequest0104
};
use codec_output::{write_utf16, write_i32, write_i64, write_u64, write_f32};
pub use codec_systems::{VendorTableUpdateRequest0104, VendorTableUpdateSuccess0104};
