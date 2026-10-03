use std::{error::Error, fmt};

/// Source family for one registered request in the paired 0104 shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShardRequestFamily0104 {
    Buddies,
    BuiltinCommands,
    Chat,
    Combat,
    Eggs,
    Email,
    Groups,
    Items,
    Missions,
    Nanos,
    NpcManager,
    PlayerManager,
    PlayerMovement,
    Racing,
    Trading,
    Transport,
    Vendors,
}

/// One request that the paired 0104 shard actually handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisteredShardRequest0104 {
    pub packet_type: u32,
    pub name: &'static str,
    pub family: ShardRequestFamily0104,
}

impl RegisteredShardRequest0104 {
    pub const fn new(packet_type: u32, name: &'static str, family: ShardRequestFamily0104) -> Self {
        Self {
            packet_type,
            name,
            family,
        }
    }
}

macro_rules! request {
    ($id:literal, $name:ident, $family:ident) => {
        RegisteredShardRequest0104::new($id, stringify!($name), ShardRequestFamily0104::$family)
    };
}

/// Complete, checked client-to-shard request inventory for OpenFusion 0104.
///
/// Having a protocol descriptor is intentionally insufficient: this list is
/// includes the 116 base registrations in `docs/protocol-0104-inventory.md`
/// and fifteen GM/store extensions documented in `docs/admin-commands.md`.
pub const OPENFUSION_SHARD_REQUESTS_0104: [RegisteredShardRequest0104; 131] = [
    request!(0x1300_0091, P_CL2FE_PC_STREETSTALL_REQ_READY, BuiltinCommands),
    request!(0x1300_0092, P_CL2FE_PC_STREETSTALL_REQ_CANCEL, BuiltinCommands),
    request!(0x1300_0093, P_CL2FE_PC_STREETSTALL_REQ_REGIST_ITEM, BuiltinCommands),
    request!(0x1300_0094, P_CL2FE_PC_STREETSTALL_REQ_UNREGIST_ITEM, BuiltinCommands),
    request!(0x1300_0095, P_CL2FE_PC_STREETSTALL_REQ_SALE_START, BuiltinCommands),
    request!(0x1300_0096, P_CL2FE_PC_STREETSTALL_REQ_ITEM_LIST, BuiltinCommands),
    request!(0x1300_0097, P_CL2FE_PC_STREETSTALL_REQ_ITEM_BUY, BuiltinCommands),
    request!(0x1300_0072, P_CL2FE_GM_REQ_PC_MOTD_REGISTER, BuiltinCommands),
    request!(0x1300_0056, P_CL2FE_REQ_NPC_GROUP_SUMMON, NpcManager),
    request!(0x1300_0061, P_CL2FE_REQ_SHINY_SUMMON, Eggs),
    request!(0x1300_0076, P_CL2FE_REQ_PC_MISSION_COMPLETE, Missions),
    request!(0x1300_0077, P_CL2FE_REQ_PC_TASK_COMPLETE, Missions),
    request!(0x1300_008a, P_CL2FE_REQ_CHANNEL_INFO, BuiltinCommands),
    request!(0x1300_008b, P_CL2FE_REQ_PC_CHANNEL_NUM, BuiltinCommands),
    request!(0x1300_008c, P_CL2FE_REQ_PC_WARP_CHANNEL, BuiltinCommands),
    request!(0x1300_0036, P_CL2FE_REQ_ACCEPT_MAKE_BUDDY, Buddies),
    request!(0x1300_003c, P_CL2FE_REQ_GET_BUDDY_STATE, Buddies),
    request!(0x1300_0051, P_CL2FE_REQ_PC_BUDDY_WARP, Buddies),
    request!(0x1300_008f, P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY, Buddies),
    request!(0x1300_008e, P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY, Buddies),
    request!(0x1300_003b, P_CL2FE_REQ_REMOVE_BUDDY, Buddies),
    request!(0x1300_0035, P_CL2FE_REQ_REQUEST_MAKE_BUDDY, Buddies),
    request!(0x1300_003a, P_CL2FE_REQ_SET_BUDDY_BLOCK, Buddies),
    request!(0x1300_0070, P_CL2FE_REQ_SET_PC_BLOCK, Buddies),
    request!(0x1300_006c, P_CL2FE_GM_REQ_KICK_PLAYER, BuiltinCommands),
    request!(0x1300_006e, P_CL2FE_GM_REQ_PC_LOCATION, BuiltinCommands),
    request!(0x1300_006b, P_CL2FE_GM_REQ_PC_SET_VALUE, BuiltinCommands),
    request!(
        0x1300_006a,
        P_CL2FE_GM_REQ_PC_SPECIAL_STATE_SWITCH,
        BuiltinCommands
    ),
    request!(0x1300_00a3, P_CL2FE_GM_REQ_REWARD_RATE, BuiltinCommands),
    request!(
        0x1300_0082,
        P_CL2FE_GM_REQ_TARGET_PC_SPECIAL_STATE_ONOFF,
        BuiltinCommands
    ),
    request!(
        0x1300_006d,
        P_CL2FE_GM_REQ_TARGET_PC_TELEPORT,
        BuiltinCommands
    ),
    request!(0x1300_001a, P_CL2FE_REQ_PC_GIVE_ITEM, BuiltinCommands),
    request!(0x1300_0044, P_CL2FE_REQ_PC_GIVE_NANO, BuiltinCommands),
    request!(0x1300_0014, P_CL2FE_REQ_PC_GOTO, BuiltinCommands),
    request!(0x1300_0057, P_CL2FE_REQ_PC_WARP_TO_PC, BuiltinCommands),
    request!(0x1300_006f, P_CL2FE_GM_REQ_PC_ANNOUNCE, Chat),
    request!(0x1300_0050, P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT, Chat),
    request!(0x1300_002d, P_CL2FE_REQ_PC_TRADE_EMOTES_CHAT, Chat),
    request!(
        0x1300_0063,
        P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE,
        Chat
    ),
    request!(
        0x1300_0066,
        P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE,
        Chat
    ),
    request!(0x1300_0037, P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE, Chat),
    request!(0x1300_0038, P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE, Chat),
    request!(0x1300_0007, P_CL2FE_REQ_SEND_FREECHAT_MESSAGE, Chat),
    request!(0x1300_0008, P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE, Chat),
    request!(0x1300_0049, P_CL2FE_DOT_DAMAGE_ONOFF, Combat),
    request!(0x1300_0090, P_CL2FE_REQ_PC_ATTACK_CHARs, Combat),
    request!(0x1300_0006, P_CL2FE_REQ_PC_ATTACK_NPCs, Combat),
    request!(0x1300_0033, P_CL2FE_REQ_PC_COMBAT_BEGIN, Combat),
    request!(0x1300_0034, P_CL2FE_REQ_PC_COMBAT_END, Combat),
    request!(0x1300_001f, P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE, Combat),
    request!(0x1300_001c, P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE, Combat),
    request!(0x1300_001d, P_CL2FE_REQ_PC_ROCKET_STYLE_HIT, Combat),
    request!(0x1300_0060, P_CL2FE_REQ_SHINY_PICKUP, Eggs),
    request!(0x1300_007e, P_CL2FE_REQ_PC_DELETE_EMAIL, Email),
    request!(0x1300_007b, P_CL2FE_REQ_PC_EMAIL_UPDATE_CHECK, Email),
    request!(0x1300_007c, P_CL2FE_REQ_PC_READ_EMAIL, Email),
    request!(0x1300_0081, P_CL2FE_REQ_PC_RECV_EMAIL_CANDY, Email),
    request!(0x1300_0080, P_CL2FE_REQ_PC_RECV_EMAIL_ITEM, Email),
    request!(0x1300_0089, P_CL2FE_REQ_PC_RECV_EMAIL_ITEM_ALL, Email),
    request!(0x1300_007d, P_CL2FE_REQ_PC_RECV_EMAIL_PAGE_LIST, Email),
    request!(0x1300_007f, P_CL2FE_REQ_PC_SEND_EMAIL, Email),
    request!(0x1300_004c, P_CL2FE_REQ_PC_GROUP_INVITE, Groups),
    request!(0x1300_004d, P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE, Groups),
    request!(0x1300_004e, P_CL2FE_REQ_PC_GROUP_JOIN, Groups),
    request!(0x1300_004f, P_CL2FE_REQ_PC_GROUP_LEAVE, Groups),
    request!(0x1300_0047, P_CL2FE_REQ_ITEM_CHEST_OPEN, Items),
    request!(0x1300_000a, P_CL2FE_REQ_ITEM_MOVE, Items),
    request!(0x1300_0073, P_CL2FE_REQ_ITEM_USE, Items),
    request!(0x1300_002e, P_CL2FE_REQ_PC_BANK_OPEN, Items),
    request!(0x1300_0019, P_CL2FE_REQ_PC_ITEM_DELETE, Items),
    request!(0x1300_0083, P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID, Missions),
    request!(0x1300_000c, P_CL2FE_REQ_PC_TASK_END, Missions),
    request!(0x1300_000b, P_CL2FE_REQ_PC_TASK_START, Missions),
    request!(0x1300_0012, P_CL2FE_REQ_PC_TASK_STOP, Missions),
    request!(0x1300_0015, P_CL2FE_REQ_CHARGE_NANO_STAMINA, Nanos),
    request!(0x1300_000f, P_CL2FE_REQ_NANO_ACTIVE, Nanos),
    request!(0x1300_000d, P_CL2FE_REQ_NANO_EQUIP, Nanos),
    request!(0x1300_0011, P_CL2FE_REQ_NANO_SKILL_USE, Nanos),
    request!(0x1300_0010, P_CL2FE_REQ_NANO_TUNE, Nanos),
    request!(0x1300_000e, P_CL2FE_REQ_NANO_UNEQUIP, Nanos),
    request!(0x1300_0048, P_CL2FE_REQ_PC_GIVE_NANO_SKILL, Nanos),
    request!(0x1300_0071, P_CL2FE_REQ_REGIST_RXCOM, Nanos),
    request!(0x1300_0074, P_CL2FE_REQ_WARP_USE_RECALL, Nanos),
    request!(0x1300_0065, P_CL2FE_REQ_BARKER, NpcManager),
    request!(0x1300_0045, P_CL2FE_REQ_NPC_SUMMON, NpcManager),
    request!(0x1300_0046, P_CL2FE_REQ_NPC_UNSUMMON, NpcManager),
    request!(0x1300_0088, P_CL2FE_REQ_PC_TIME_TO_GO_WARP, NpcManager),
    request!(0x1300_004b, P_CL2FE_REQ_PC_WARP_USE_NPC, NpcManager),
    request!(0x1300_00a7, P_CL2FE_REQ_PRESENT_NPC_TYPES, NpcManager),
    request!(0x1300_0075, P_CL2FE_REP_LIVE_CHECK, PlayerManager),
    request!(0x1300_0054, P_CL2FE_REQ_PC_CHANGE_MENTOR, PlayerManager),
    request!(0x1300_0001, P_CL2FE_REQ_PC_ENTER, PlayerManager),
    request!(0x1300_0002, P_CL2FE_REQ_PC_EXIT, PlayerManager),
    request!(
        0x1300_0086,
        P_CL2FE_REQ_PC_FIRST_USE_FLAG_SET,
        PlayerManager
    ),
    request!(0x1300_008d, P_CL2FE_REQ_PC_LOADING_COMPLETE, PlayerManager),
    request!(0x1300_0009, P_CL2FE_REQ_PC_REGEN, PlayerManager),
    request!(
        0x1300_007a,
        P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH,
        PlayerManager
    ),
    request!(0x1300_00a0, P_CL2FE_REQ_PC_VEHICLE_OFF, PlayerManager),
    request!(0x1300_009f, P_CL2FE_REQ_PC_VEHICLE_ON, PlayerManager),
    request!(0x1300_0005, P_CL2FE_REQ_PC_JUMP, PlayerMovement),
    request!(0x1300_003d, P_CL2FE_REQ_PC_JUMPPAD, PlayerMovement),
    request!(0x1300_003e, P_CL2FE_REQ_PC_LAUNCHER, PlayerMovement),
    request!(0x1300_0003, P_CL2FE_REQ_PC_MOVE, PlayerMovement),
    request!(0x1300_0040, P_CL2FE_REQ_PC_MOVEPLATFORM, PlayerMovement),
    request!(
        0x1300_0062,
        P_CL2FE_REQ_PC_MOVETRANSPORTATION,
        PlayerMovement
    ),
    request!(0x1300_0041, P_CL2FE_REQ_PC_SLOPE, PlayerMovement),
    request!(0x1300_0004, P_CL2FE_REQ_PC_STOP, PlayerMovement),
    request!(0x1300_003f, P_CL2FE_REQ_PC_ZIPLINE, PlayerMovement),
    request!(0x1300_005e, P_CL2FE_REQ_EP_GET_RING, Racing),
    request!(0x1300_005d, P_CL2FE_REQ_EP_RACE_CANCEL, Racing),
    request!(0x1300_005c, P_CL2FE_REQ_EP_RACE_END, Racing),
    request!(0x1300_005b, P_CL2FE_REQ_EP_RACE_START, Racing),
    request!(0x1300_002c, P_CL2FE_REQ_PC_TRADE_CASH_REGISTER, Trading),
    request!(0x1300_0027, P_CL2FE_REQ_PC_TRADE_CONFIRM, Trading),
    request!(0x1300_0028, P_CL2FE_REQ_PC_TRADE_CONFIRM_CANCEL, Trading),
    request!(0x1300_002a, P_CL2FE_REQ_PC_TRADE_ITEM_REGISTER, Trading),
    request!(0x1300_002b, P_CL2FE_REQ_PC_TRADE_ITEM_UNREGISTER, Trading),
    request!(0x1300_0022, P_CL2FE_REQ_PC_TRADE_OFFER, Trading),
    request!(0x1300_0026, P_CL2FE_REQ_PC_TRADE_OFFER_ABORT, Trading),
    request!(0x1300_0024, P_CL2FE_REQ_PC_TRADE_OFFER_ACCEPT, Trading),
    request!(0x1300_0023, P_CL2FE_REQ_PC_TRADE_OFFER_CANCEL, Trading),
    request!(0x1300_0025, P_CL2FE_REQ_PC_TRADE_OFFER_REFUSAL, Trading),
    request!(
        0x1300_0069,
        P_CL2FE_REQ_PC_WARP_USE_TRANSPORTATION,
        Transport
    ),
    request!(
        0x1300_0068,
        P_CL2FE_REQ_REGIST_TRANSPORTATION_LOCATION,
        Transport
    ),
    request!(0x1300_0098, P_CL2FE_REQ_PC_ITEM_COMBINATION, Vendors),
    request!(0x1300_004a, P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY, Vendors),
    request!(0x1300_0017, P_CL2FE_REQ_PC_VENDOR_ITEM_BUY, Vendors),
    request!(0x1300_0032, P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY, Vendors),
    request!(0x1300_0018, P_CL2FE_REQ_PC_VENDOR_ITEM_SELL, Vendors),
    request!(0x1300_0030, P_CL2FE_REQ_PC_VENDOR_START, Vendors),
    request!(0x1300_0031, P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE, Vendors),
];

/// Native RustyFusion handlers absent from the preserved OpenFusion inventory.
/// See `RustyFusion/src/server/shard/mod.rs` and `RustyFusion/src/barber.rs`.
const RUSTYFUSION_SHARD_REQUESTS_0104: [RegisteredShardRequest0104; 2] = [
    request!(0x1300_00a5, P_CL2FE_REQ_PC_BARBER_OPEN, NpcManager),
    request!(0x1300_00a6, P_CL2FE_REQ_PC_BARBER_CONFIRM, NpcManager),
];

pub fn registered_shard_request_0104(
    packet_type: u32,
) -> Option<&'static RegisteredShardRequest0104> {
    OPENFUSION_SHARD_REQUESTS_0104
        .iter()
        .chain(RUSTYFUSION_SHARD_REQUESTS_0104.iter())
        .find(|request| request.packet_type == packet_type)
}

/// Exact protocol-0104 body shape accepted by one registered request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisteredRequestLayout0104 {
    Fixed {
        size: usize,
    },
    Counted {
        base_size: usize,
        count_offset: usize,
        element_size: usize,
        maximum_count: usize,
    },
}

impl RegisteredRequestLayout0104 {
    pub const fn fixed_size(self) -> Option<usize> {
        match self {
            Self::Fixed { size } => Some(size),
            Self::Counted { .. } => None,
        }
    }

    fn validate(
        self,
        packet_type: u32,
        payload: &[u8],
    ) -> Result<(), RegisteredGameplayRequestError0104> {
        match self {
            Self::Fixed { size } => {
                if payload.len() != size {
                    return Err(RegisteredGameplayRequestError0104::WrongPayloadSize {
                        packet_type,
                        expected: size,
                        actual: payload.len(),
                    });
                }
            }
            Self::Counted {
                base_size,
                count_offset,
                element_size,
                maximum_count,
            } => {
                if payload.len() < base_size {
                    return Err(RegisteredGameplayRequestError0104::PayloadTooShort {
                        packet_type,
                        minimum: base_size,
                        actual: payload.len(),
                    });
                }
                let count = i32::from_le_bytes(
                    payload[count_offset..count_offset + 4]
                        .try_into()
                        .expect("layout count is inside fixed base"),
                );
                if count < 0 {
                    return Err(RegisteredGameplayRequestError0104::NegativeCount {
                        packet_type,
                        count,
                    });
                }
                let count = count as usize;
                if count > maximum_count {
                    return Err(RegisteredGameplayRequestError0104::CountTooLarge {
                        packet_type,
                        count,
                        maximum: maximum_count,
                    });
                }
                let expected = base_size + count * element_size;
                if payload.len() != expected {
                    return Err(RegisteredGameplayRequestError0104::WrongPayloadSize {
                        packet_type,
                        expected,
                        actual: payload.len(),
                    });
                }
            }
        }
        Ok(())
    }
}

const fn fixed(size: usize) -> RegisteredRequestLayout0104 {
    RegisteredRequestLayout0104::Fixed { size }
}

const fn counted(
    base_size: usize,
    count_offset: usize,
    element_size: usize,
    maximum_count: usize,
) -> RegisteredRequestLayout0104 {
    RegisteredRequestLayout0104::Counted {
        base_size,
        count_offset,
        element_size,
        maximum_count,
    }
}

/// Complete request-body ABI for the registered OpenFusion handlers.
///
/// Fixed sizes come from the pinned `structs/0104.hpp` assertions. The four
/// variable layouts come from the corresponding `validInVarPacket` calls in
/// `Combat.cpp` and `Nanos.cpp`.
pub const fn registered_request_layout_0104(
    packet_type: u32,
) -> Option<RegisteredRequestLayout0104> {
    Some(match packet_type {
        0x1300_0091 | 0x1300_0092 | 0x1300_0094 | 0x1300_0095 | 0x1300_0096 => fixed(4),
        0x1300_0093 => fixed(24),
        0x1300_0097 => fixed(12),
        // Paired server GM extensions, using the existing 0104 wire ABI.
        0x1300_0072 => fixed(1026),
        0x1300_0056 | 0x1300_0076 | 0x1300_0077 => fixed(4),
        0x1300_0061 => fixed(16),
        0x1300_008a | 0x1300_008b => fixed(1),
        0x1300_008c => fixed(8),
        // Buddies
        0x1300_0036 => fixed(16),
        0x1300_003c => fixed(1),
        0x1300_0051 => fixed(12),
        0x1300_008f => fixed(64),
        0x1300_008e => fixed(52),
        0x1300_003b | 0x1300_003a => fixed(12),
        0x1300_0035 => fixed(12),
        0x1300_0070 => fixed(12),

        // BuiltinCommands
        0x1300_006c => fixed(76),
        0x1300_006e => fixed(72),
        0x1300_006b => fixed(12),
        0x1300_006a => fixed(8),
        0x1300_00a3 => fixed(16),
        0x1300_0082 => fixed(80),
        0x1300_006d => fixed(172),
        0x1300_001a => fixed(24),
        0x1300_0044 => fixed(2),
        0x1300_0014 => fixed(12),
        0x1300_0057 => fixed(8),

        // Chat
        0x1300_006f => fixed(1032),
        0x1300_0050 => fixed(8),
        0x1300_002d => fixed(276),
        0x1300_0063 | 0x1300_0066 | 0x1300_0007 | 0x1300_0008 => fixed(260),
        0x1300_0037 | 0x1300_0038 => fixed(272),

        // Combat
        0x1300_0049 => fixed(4),
        0x1300_0090 => counted(4, 0, 8, 510),
        0x1300_0006 => counted(4, 0, 4, 3),
        0x1300_0033 | 0x1300_0034 => fixed(4),
        0x1300_001f => fixed(16),
        0x1300_001c => fixed(28),
        0x1300_001d => counted(20, 16, 8, 508),

        // Eggs
        0x1300_0060 => fixed(4),

        // Email
        0x1300_007e => fixed(40),
        0x1300_007b => fixed(1),
        0x1300_007c => fixed(8),
        0x1300_0081 => fixed(8),
        0x1300_0080 => fixed(16),
        0x1300_0089 => fixed(8),
        0x1300_007d => fixed(1),
        0x1300_007f => fixed(1164),

        // Groups
        0x1300_004c | 0x1300_004d | 0x1300_004e => fixed(4),
        0x1300_004f => fixed(1),

        // Items
        0x1300_0047 => fixed(20),
        0x1300_000a => fixed(16),
        0x1300_0073 => fixed(12),
        0x1300_002e => fixed(8),
        0x1300_0019 => fixed(8),

        // Missions
        0x1300_0083 => fixed(4),
        0x1300_000c => fixed(16),
        0x1300_000b => fixed(12),
        0x1300_0012 => fixed(4),

        // Nanos
        0x1300_0015 => fixed(4),
        0x1300_000f => fixed(2),
        0x1300_000d => fixed(4),
        0x1300_0011 => counted(20, 16, 4, 1017),
        0x1300_0010 => fixed(44),
        0x1300_000e => fixed(2),
        0x1300_0048 => fixed(4),
        0x1300_0071 | 0x1300_0074 => fixed(4),

        // NPCManager
        0x1300_00a5 => fixed(4),
        0x1300_00a6 => fixed(76),
        0x1300_0065 => fixed(8),
        0x1300_0045 => fixed(8),
        0x1300_0046 => fixed(4),
        0x1300_0088 => fixed(32),
        0x1300_004b => fixed(24),
        0x1300_00a7 => fixed(8),

        // PlayerManager
        0x1300_0075 => fixed(4),
        0x1300_0054 => fixed(2),
        0x1300_0001 => fixed(80),
        0x1300_0002 => fixed(4),
        0x1300_0086 | 0x1300_008d => fixed(4),
        0x1300_0009 => fixed(12),
        0x1300_007a => fixed(8),
        0x1300_00a0 | 0x1300_009f => fixed(1),

        // PlayerMovement
        0x1300_0005 | 0x1300_0003 => fixed(44),
        0x1300_003d | 0x1300_003e => fixed(40),
        0x1300_0040 => fixed(64),
        0x1300_0062 => fixed(60),
        0x1300_0041 => fixed(48),
        0x1300_0004 => fixed(20),
        0x1300_003f => fixed(76),

        // Racing
        0x1300_005e | 0x1300_005d => fixed(4),
        0x1300_005c => fixed(8),
        0x1300_005b => fixed(12),

        // Trading
        0x1300_002c => fixed(16),
        0x1300_0027 | 0x1300_0028 | 0x1300_0022 | 0x1300_0024 | 0x1300_0023 | 0x1300_0025 => {
            fixed(12)
        }
        0x1300_002a | 0x1300_002b => fixed(28),
        0x1300_0026 => fixed(16),

        // Transport
        0x1300_0069 => fixed(16),
        0x1300_0068 => fixed(12),

        // Vendors
        0x1300_0098 => fixed(16),
        0x1300_004a => fixed(24),
        0x1300_0017 | 0x1300_0032 => fixed(28),
        0x1300_0018 | 0x1300_0030 | 0x1300_0031 => fixed(8),
        _ => return None,
    })
}

/// Lossless request envelope for feature codecs that live above `ffone-protocol`.
///
/// Construction proves that the paired RustyFusion shard registers the ID
/// and that the body matches the handler's exact fixed or counted ABI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredGameplayRequest0104 {
    packet_type: u32,
    payload: Vec<u8>,
}

impl RegisteredGameplayRequest0104 {
    pub const MAX_PAYLOAD_SIZE: usize = 4096 - 2 * size_of::<i32>();

    pub fn new(
        packet_type: u32,
        payload: Vec<u8>,
    ) -> Result<Self, RegisteredGameplayRequestError0104> {
        if registered_shard_request_0104(packet_type).is_none() {
            return Err(RegisteredGameplayRequestError0104::UnregisteredPacket { packet_type });
        }
        if payload.len() > Self::MAX_PAYLOAD_SIZE {
            return Err(RegisteredGameplayRequestError0104::PayloadTooLarge {
                packet_type,
                actual: payload.len(),
                maximum: Self::MAX_PAYLOAD_SIZE,
            });
        }
        registered_request_layout_0104(packet_type)
            .expect("every registered request has an ABI layout")
            .validate(packet_type, &payload)?;
        Ok(Self {
            packet_type,
            payload,
        })
    }

    pub const fn packet_type(&self) -> u32 {
        self.packet_type
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_parts(self) -> (u32, Vec<u8>) {
        (self.packet_type, self.payload)
    }

    pub fn registration(&self) -> &'static RegisteredShardRequest0104 {
        registered_shard_request_0104(self.packet_type)
            .expect("registered request construction invariant")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisteredGameplayRequestError0104 {
    UnregisteredPacket {
        packet_type: u32,
    },
    PayloadTooLarge {
        packet_type: u32,
        actual: usize,
        maximum: usize,
    },
    PayloadTooShort {
        packet_type: u32,
        minimum: usize,
        actual: usize,
    },
    WrongPayloadSize {
        packet_type: u32,
        expected: usize,
        actual: usize,
    },
    NegativeCount {
        packet_type: u32,
        count: i32,
    },
    CountTooLarge {
        packet_type: u32,
        count: usize,
        maximum: usize,
    },
}

impl fmt::Display for RegisteredGameplayRequestError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnregisteredPacket { packet_type } => write!(
                formatter,
                "packet {packet_type:#010x} is not registered by the pinned OpenFusion shard"
            ),
            Self::PayloadTooLarge {
                packet_type,
                actual,
                maximum,
            } => write!(
                formatter,
                "packet {packet_type:#010x} payload has {actual} bytes; OpenFusion accepts at most {maximum}"
            ),
            Self::PayloadTooShort {
                packet_type,
                minimum,
                actual,
            } => write!(
                formatter,
                "packet {packet_type:#010x} counted payload needs at least {minimum} bytes, got {actual}"
            ),
            Self::WrongPayloadSize {
                packet_type,
                expected,
                actual,
            } => write!(
                formatter,
                "packet {packet_type:#010x} payload must be exactly {expected} bytes, got {actual}"
            ),
            Self::NegativeCount { packet_type, count } => write!(
                formatter,
                "packet {packet_type:#010x} has negative trailer count {count}"
            ),
            Self::CountTooLarge {
                packet_type,
                count,
                maximum,
            } => write!(
                formatter,
                "packet {packet_type:#010x} trailer count {count} exceeds {maximum}"
            ),
        }
    }
}

impl Error for RegisteredGameplayRequestError0104 {}

#[cfg(test)]
mod tests;
