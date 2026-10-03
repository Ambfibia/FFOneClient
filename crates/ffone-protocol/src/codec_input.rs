use super::*;

/// Protocol-0104 `sP_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY`
/// (`#pragma pack(2)`, 52 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFindNameRequest0104 {
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for BuddyFindNameRequest0104 {
    const SIZE: usize = 52;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.first_name);
        write_utf16(&mut out, 18, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC`
/// (`#pragma pack(4)`, 64 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFindNameSuccess0104 {
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
    pub pc_uid: i64,
    pub name_check_flag: i8,
}

impl WirePayload for BuddyFindNameSuccess0104 {
    const SIZE: usize = 64;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.first_name);
        write_utf16(&mut out, 18, &self.last_name);
        write_i64(&mut out, 52, self.pc_uid);
        out[60] = self.name_check_flag as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
            pc_uid: read_i64(bytes, 52),
            name_check_flag: bytes[60] as i8,
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL`
/// (`#pragma pack(4)`, 56 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFindNameFailure0104 {
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
    pub error_code: i32,
}

impl WirePayload for BuddyFindNameFailure0104 {
    const SIZE: usize = 56;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.first_name);
        write_utf16(&mut out, 18, &self.last_name);
        write_i32(&mut out, 52, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
            error_code: read_i32(bytes, 52),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY`
/// (`#pragma pack(4)`, 64 bytes). Clean Retrobution uses an `int32_t`
/// accept flag here, unlike the one-byte nearby-player accept packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFindNameAcceptRequest0104 {
    pub accept_flag: i32,
    pub buddy_pc_uid: i64,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for BuddyFindNameAcceptRequest0104 {
    const SIZE: usize = 64;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.accept_flag);
        write_i64(&mut out, 4, self.buddy_pc_uid);
        write_utf16(&mut out, 12, &self.first_name);
        write_utf16(&mut out, 30, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            accept_flag: read_i32(bytes, 0),
            buddy_pc_uid: read_i64(bytes, 4),
            first_name: read_utf16(bytes, 12),
            last_name: read_utf16(bytes, 30),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL`
/// (`#pragma pack(4)`, 68 bytes). There is no corresponding clean success
/// packet; accepted relationships arrive through the normal accept success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFindNameAcceptFailure0104 {
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
    pub pc_uid: i64,
    pub name_check_flag: i8,
    pub error_code: i32,
}

impl WirePayload for BuddyFindNameAcceptFailure0104 {
    const SIZE: usize = 68;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.first_name);
        write_utf16(&mut out, 18, &self.last_name);
        write_i64(&mut out, 52, self.pc_uid);
        out[60] = self.name_check_flag as u8;
        write_i32(&mut out, 64, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
            pc_uid: read_i64(bytes, 52),
            name_check_flag: bytes[60] as i8,
            error_code: read_i32(bytes, 64),
        })
    }
}

/// Lossless protocol-0104 `sPCLoadData2CL`.
#[derive(Clone, PartialEq, Eq)]
pub struct PcLoadData0104(pub(super) [u8; 2688]);

impl PcLoadData0104 {
    pub const SIZE: usize = 2688;
    pub const USER_LEVEL_OFFSET: usize = 0;
    pub const STYLE_OFFSET: usize = 4;
    pub const STYLE_FLAGS_OFFSET: usize = 80;
    pub const LEVEL_OFFSET: usize = 84;
    pub const MENTOR_OFFSET: usize = 86;
    pub const MENTOR_COUNT_OFFSET: usize = 88;
    pub const HP_OFFSET: usize = 92;
    pub const WEAPON_BATTERY_OFFSET: usize = 96;
    pub const NANO_BATTERY_OFFSET: usize = 100;
    pub const CANDY_OFFSET: usize = 104;
    pub const FUSION_MATTER_OFFSET: usize = 108;
    pub const SPECIAL_STATE_OFFSET: usize = 112;
    pub const MAP_NUMBER_OFFSET: usize = 116;
    pub const X_OFFSET: usize = 120;
    pub const Y_OFFSET: usize = 124;
    pub const Z_OFFSET: usize = 128;
    pub const ANGLE_OFFSET: usize = 132;
    pub const EQUIPMENT_OFFSET: usize = 136;
    pub const EQUIPMENT_COUNT: usize = 9;
    pub const INVENTORY_OFFSET: usize =
        Self::EQUIPMENT_OFFSET + Self::EQUIPMENT_COUNT * ItemBase0104::SIZE;
    pub const INVENTORY_COUNT: usize = 50;
    pub const QUEST_INVENTORY_OFFSET: usize =
        Self::INVENTORY_OFFSET + Self::INVENTORY_COUNT * ItemBase0104::SIZE;
    pub const QUEST_INVENTORY_COUNT: usize = 50;
    pub const NANO_BANK_OFFSET: usize =
        Self::QUEST_INVENTORY_OFFSET + Self::QUEST_INVENTORY_COUNT * ItemBase0104::SIZE;
    pub const NANO_BANK_COUNT: usize = 37;
    pub const NANO_SLOTS_OFFSET: usize =
        Self::NANO_BANK_OFFSET + Self::NANO_BANK_COUNT * Nano0104::SIZE;
    pub const NANO_SLOT_COUNT: usize = 3;
    pub const ACTIVE_NANO_SLOT_OFFSET: usize =
        Self::NANO_SLOTS_OFFSET + Self::NANO_SLOT_COUNT * size_of::<i16>();
    pub const CONDITION_BIT_FLAG_OFFSET: usize = 1676;
    pub const ADDITIONAL_TIME_BUFF_ID_OFFSET: usize = 1680;
    pub const INITIAL_TIME_BUFF_OFFSET: usize = 1684;
    pub const QUEST_FLAGS_OFFSET: usize = 1712;
    pub const QUEST_FLAG_COUNT: usize = 32;
    pub const REPEAT_QUEST_FLAGS_OFFSET: usize = 1968;
    pub const REPEAT_QUEST_FLAG_COUNT: usize = 8;
    pub const RUNNING_QUESTS_OFFSET: usize = 2032;
    pub const RUNNING_QUEST_COUNT: usize = 9;
    pub const CURRENT_MISSION_ID_OFFSET: usize = 2500;
    pub const WARP_LOCATION_FLAG_OFFSET: usize = 2504;
    pub const WYVERN_LOCATION_FLAGS_OFFSET: usize = 2508;
    pub const WYVERN_LOCATION_FLAG_COUNT: usize = 2;

    pub const fn zeroed() -> Self {
        Self([0; Self::SIZE])
    }

    pub const fn as_bytes(&self) -> &[u8; Self::SIZE] {
        &self.0
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8; Self::SIZE] {
        &mut self.0
    }

    pub fn hp(&self) -> i32 {
        read_i32(&self.0, Self::HP_OFFSET)
    }

    pub fn user_level(&self) -> i16 {
        read_i16(&self.0, Self::USER_LEVEL_OFFSET)
    }

    pub fn style(&self) -> PcStyle0104 {
        PcStyle0104::decode_exact(
            &self.0[Self::STYLE_OFFSET..Self::STYLE_OFFSET + PcStyle0104::SIZE],
        )
    }

    pub fn style_flags(&self) -> [i8; 3] {
        [
            self.0[Self::STYLE_FLAGS_OFFSET] as i8,
            self.0[Self::STYLE_FLAGS_OFFSET + 1] as i8,
            self.0[Self::STYLE_FLAGS_OFFSET + 2] as i8,
        ]
    }

    pub fn level(&self) -> i16 {
        read_i16(&self.0, Self::LEVEL_OFFSET)
    }

    pub fn mentor(&self) -> i16 {
        read_i16(&self.0, Self::MENTOR_OFFSET)
    }

    pub fn mentor_count(&self) -> i16 {
        read_i16(&self.0, Self::MENTOR_COUNT_OFFSET)
    }

    pub fn weapon_battery(&self) -> i32 {
        read_i32(&self.0, Self::WEAPON_BATTERY_OFFSET)
    }

    pub fn nano_battery(&self) -> i32 {
        read_i32(&self.0, Self::NANO_BATTERY_OFFSET)
    }

    pub fn candy(&self) -> i32 {
        read_i32(&self.0, Self::CANDY_OFFSET)
    }

    pub fn fusion_matter(&self) -> i32 {
        read_i32(&self.0, Self::FUSION_MATTER_OFFSET)
    }

    pub fn special_state(&self) -> i8 {
        self.0[Self::SPECIAL_STATE_OFFSET] as i8
    }

    pub fn map_number(&self) -> i32 {
        read_i32(&self.0, Self::MAP_NUMBER_OFFSET)
    }

    pub fn position(&self) -> [i32; 3] {
        [
            read_i32(&self.0, Self::X_OFFSET),
            read_i32(&self.0, Self::Y_OFFSET),
            read_i32(&self.0, Self::Z_OFFSET),
        ]
    }

    pub fn angle(&self) -> i32 {
        read_i32(&self.0, Self::ANGLE_OFFSET)
    }

    pub fn equipment(&self) -> [ItemBase0104; Self::EQUIPMENT_COUNT] {
        std::array::from_fn(|index| {
            let offset = Self::EQUIPMENT_OFFSET + index * ItemBase0104::SIZE;
            ItemBase0104::decode_exact(&self.0[offset..offset + ItemBase0104::SIZE])
        })
    }

    pub fn inventory(&self) -> [ItemBase0104; Self::INVENTORY_COUNT] {
        std::array::from_fn(|index| {
            let offset = Self::INVENTORY_OFFSET + index * ItemBase0104::SIZE;
            ItemBase0104::decode_exact(&self.0[offset..offset + ItemBase0104::SIZE])
        })
    }

    pub fn quest_inventory(&self) -> [ItemBase0104; Self::QUEST_INVENTORY_COUNT] {
        std::array::from_fn(|index| {
            let offset = Self::QUEST_INVENTORY_OFFSET + index * ItemBase0104::SIZE;
            ItemBase0104::decode_exact(&self.0[offset..offset + ItemBase0104::SIZE])
        })
    }

    pub fn nano_bank(&self) -> [Nano0104; Self::NANO_BANK_COUNT] {
        std::array::from_fn(|index| {
            let offset = Self::NANO_BANK_OFFSET + index * Nano0104::SIZE;
            Nano0104::decode_exact(&self.0[offset..offset + Nano0104::SIZE])
        })
    }

    pub fn nano_slots(&self) -> [i16; Self::NANO_SLOT_COUNT] {
        std::array::from_fn(|index| {
            read_i16(&self.0, Self::NANO_SLOTS_OFFSET + index * size_of::<i16>())
        })
    }

    pub fn active_nano_slot(&self) -> i16 {
        read_i16(&self.0, Self::ACTIVE_NANO_SLOT_OFFSET)
    }

    pub fn condition_bit_flag(&self) -> i32 {
        read_i32(&self.0, Self::CONDITION_BIT_FLAG_OFFSET)
    }

    pub fn additional_time_buff_id(&self) -> i32 {
        read_i32(&self.0, Self::ADDITIONAL_TIME_BUFF_ID_OFFSET)
    }

    pub fn initial_time_buff(&self) -> TimeBuff0104 {
        TimeBuff0104::decode(
            &self.0[Self::INITIAL_TIME_BUFF_OFFSET
                ..Self::INITIAL_TIME_BUFF_OFFSET + TimeBuff0104::SIZE],
        )
        .expect("fixed PC load TimeBuff slice")
    }

    /// Exact `aQuestFlag` mission-completion bitset. The clean client consumes
    /// only the first 16 words, but the wire contract contains all 32.
    pub fn quest_flags(&self) -> [i64; Self::QUEST_FLAG_COUNT] {
        std::array::from_fn(|index| read_i64(&self.0, Self::QUEST_FLAGS_OFFSET + index * 8))
    }

    pub fn repeat_quest_flags(&self) -> [i64; Self::REPEAT_QUEST_FLAG_COUNT] {
        std::array::from_fn(|index| read_i64(&self.0, Self::REPEAT_QUEST_FLAGS_OFFSET + index * 8))
    }

    pub fn running_quests(&self) -> [RunningQuest0104; Self::RUNNING_QUEST_COUNT] {
        std::array::from_fn(|index| {
            let offset = Self::RUNNING_QUESTS_OFFSET + index * RunningQuest0104::SIZE;
            RunningQuest0104::decode(&self.0[offset..offset + RunningQuest0104::SIZE])
                .expect("fixed PC load running-quest slice")
        })
    }

    pub fn current_mission_id(&self) -> i32 {
        read_i32(&self.0, Self::CURRENT_MISSION_ID_OFFSET)
    }

    /// Exact protocol-0104 `iWarpLocationFlag` used by the clean
    /// S.C.A.M.P.E.R. route-registration UI.
    pub fn warp_location_flag(&self) -> u32 {
        read_i32(&self.0, Self::WARP_LOCATION_FLAG_OFFSET) as u32
    }

    /// Exact protocol-0104 `aWyvernLocationFlag[2]` used by the clean Monkey
    /// Skyway route-registration UI. The wire values are signed `int64_t`,
    /// but the client consumes their bit patterns.
    pub fn wyvern_location_flags(&self) -> [u64; Self::WYVERN_LOCATION_FLAG_COUNT] {
        std::array::from_fn(|index| {
            read_i64(&self.0, Self::WYVERN_LOCATION_FLAGS_OFFSET + index * 8) as u64
        })
    }
}

impl Default for PcLoadData0104 {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl fmt::Debug for PcLoadData0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcLoadData0104")
            .field("style", &self.style())
            .field("style_flags", &self.style_flags())
            .field("level", &self.level())
            .field("hp", &self.hp())
            .field("candy", &self.candy())
            .field("fusion_matter", &self.fusion_matter())
            .field("condition_bit_flag", &self.condition_bit_flag())
            .field("map_number", &self.map_number())
            .field("position", &self.position())
            .field("angle", &self.angle())
            .finish_non_exhaustive()
    }
}

impl WirePayload for PcLoadData0104 {
    const SIZE: usize = 2688;

    fn encode(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self(bytes.try_into().expect("payload size checked")))
    }
}

pub(super) fn read_utf16<const N: usize>(bytes: &[u8], offset: usize) -> FixedUtf16<N> {
    let mut units = [0u16; N];
    for (index, unit) in units.iter_mut().enumerate() {
        let start = offset + index * 2;
        *unit = u16::from_le_bytes(bytes[start..start + 2].try_into().expect("fixed range"));
    }
    FixedUtf16::from_units(units)
}

pub(super) fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed range"))
}

pub(super) fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed range"))
}

pub(super) fn read_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("fixed range"))
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("fixed range"))
}

pub(super) fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed range"))
}
