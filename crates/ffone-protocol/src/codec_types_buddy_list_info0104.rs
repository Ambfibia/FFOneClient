use super::*;

/// Protocol-0104 `sP_CL2FE_DOT_DAMAGE_ONOFF` and
/// `sP_CL2FE_DOT_HEAL_ONOFF` share this exact pack(4) body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentDotToggle0104 {
    pub enabled: bool,
}

impl WirePayload for EnvironmentDotToggle0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        i32::from(self.enabled).to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            enabled: read_i32(bytes, 0) != 0,
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_TICK` (pack(4), 32 bytes).
///
/// The three serialized `sNano` records retain their equipped-slot order.
/// Protocol-0104 does not include the active slot in this tick, so callers
/// must reconcile activity from `P_FE2CL_REP_NANO_ACTIVE_SUCC` separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcTick0104 {
    pub hp: i32,
    pub remaining: [u8; 28],
}

impl PcTick0104 {
    pub const NANO_SLOT_COUNT: usize = 3;
    pub(super) const NANO_TAIL_OFFSET: usize = 0;
    pub(super) const NANO_BATTERY_TAIL_OFFSET: usize = 20;
    pub(super) const RESET_MISSION_TAIL_OFFSET: usize = 24;

    /// Exact equipped-slot `sNano` records at body offsets 4, 10 and 16.
    #[must_use]
    pub fn nanos(&self) -> [Nano0104; Self::NANO_SLOT_COUNT] {
        std::array::from_fn(|index| {
            let start = Self::NANO_TAIL_OFFSET + index * Nano0104::SIZE;
            Nano0104::decode_exact(&self.remaining[start..start + Nano0104::SIZE])
        })
    }

    /// Exact `iBatteryN` at body offset 24 (tail offset 20).
    #[must_use]
    pub fn nano_battery(&self) -> i32 {
        read_i32(&self.remaining, Self::NANO_BATTERY_TAIL_OFFSET)
    }

    /// Exact `bResetMissionFlag` at body offset 28 (tail offset 24).
    #[must_use]
    pub fn reset_mission_flag(&self) -> i32 {
        read_i32(&self.remaining, Self::RESET_MISSION_TAIL_OFFSET)
    }
}

impl WirePayload for PcTick0104 {
    const SIZE: usize = 32;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.hp);
        out[4..].copy_from_slice(&self.remaining);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            hp: read_i32(bytes, 0),
            remaining: bytes[4..].try_into().expect("fixed 28-byte tail"),
        })
    }
}

/// OpenFusion's infection result appended to
/// `sP_FE2CL_CHAR_TIME_BUFF_TIME_TICK`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBuffDotDamageTick0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub time_buff_id: i16,
    pub result_character_type: i32,
    pub result_character_id: i32,
    pub protected: bool,
    pub damage: i32,
    pub hp: i32,
    pub stamina: i16,
    pub nano_deactivated: bool,
    pub condition_bit_flag: i32,
}

impl WirePayload for TimeBuffDotDamageTick0104 {
    const SIZE: usize = 44;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.character_type);
        write_i32(&mut out, 4, self.character_id);
        out[8..10].copy_from_slice(&self.time_buff_id.to_le_bytes());
        write_i32(&mut out, 12, self.result_character_type);
        write_i32(&mut out, 16, self.result_character_id);
        write_i32(&mut out, 20, i32::from(self.protected));
        write_i32(&mut out, 24, self.damage);
        write_i32(&mut out, 28, self.hp);
        out[32..34].copy_from_slice(&self.stamina.to_le_bytes());
        write_i32(&mut out, 36, i32::from(self.nano_deactivated));
        write_i32(&mut out, 40, self.condition_bit_flag);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            time_buff_id: i16::from_le_bytes(bytes[8..10].try_into().expect("fixed i16")),
            result_character_type: read_i32(bytes, 12),
            result_character_id: read_i32(bytes, 16),
            protected: read_i32(bytes, 20) != 0,
            damage: read_i32(bytes, 24),
            hp: read_i32(bytes, 28),
            stamina: i16::from_le_bytes(bytes[32..34].try_into().expect("fixed i16")),
            nano_deactivated: read_i32(bytes, 36) != 0,
            condition_bit_flag: read_i32(bytes, 40),
        })
    }
}

/// Timed healing (buff 24): 12-byte header followed by sSkillResult_Heal_HP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBuffHealTick0104 { pub character_type: i32, pub character_id: i32, pub healed_hp: i32, pub hp: i32 }

impl WirePayload for TimeBuffHealTick0104 {
    const SIZE: usize = 28;
    fn encode(&self) -> Vec<u8> {
        let mut out=vec![0;Self::SIZE];
        write_i32(&mut out,0,self.character_type);write_i32(&mut out,4,self.character_id);
        out[8..10].copy_from_slice(&24i16.to_le_bytes());
        write_i32(&mut out,12,self.character_type);write_i32(&mut out,16,self.character_id);
        write_i32(&mut out,20,self.healed_hp);write_i32(&mut out,24,self.hp);out
    }
    fn decode(bytes:&[u8])->Result<Self,PayloadError> {
        require_size(bytes,Self::SIZE)?;
        let character_type=read_i32(bytes,0);let character_id=read_i32(bytes,4);
        for (field,value,minimum,maximum) in [
            ("healing buff",i32::from(read_i16(bytes,8)),24,24),
            ("healing result type",read_i32(bytes,12),character_type,character_type),
            ("healing result owner",read_i32(bytes,16),character_id,character_id),
            ("healed HP",read_i32(bytes,20),0,i32::MAX),
            ("healing HP",read_i32(bytes,24),0,i32::MAX),
        ] { if value<minimum || value>maximum {return Err(PayloadError::ValueOutOfRange{field,value,minimum,maximum});} }
        Ok(Self {character_type,character_id,healed_hp:read_i32(bytes,20),hp:read_i32(bytes,24)})
    }
}

/// Protocol-0104 `sTimeBuff` (`#pragma pack(4)`, 28 bytes).
///
/// Retrobution uses the same record inside the initial PC load block and both
/// regular and cash buff update packets. The two time fields are server-owned
/// millisecond values and are retained without normalizing their units.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeBuff0104 {
    pub time_limit: u64,
    pub time_duration: u64,
    pub time_repeat: i32,
    pub value: i32,
    pub confirm_number: i32,
}

impl WirePayload for TimeBuff0104 {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.time_limit);
        write_u64(&mut out, 8, self.time_duration);
        write_i32(&mut out, 16, self.time_repeat);
        write_i32(&mut out, 20, self.value);
        write_i32(&mut out, 24, self.confirm_number);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            time_limit: read_u64(bytes, 0),
            time_duration: read_u64(bytes, 8),
            time_repeat: read_i32(bytes, 16),
            value: read_i32(bytes, 20),
            confirm_number: read_i32(bytes, 24),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_PC_BUFF_UPDATE` (pack(4), 44 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcBuffUpdate0104 {
    pub buff_id: i32,
    pub update_kind: i32,
    pub buff_type: i32,
    pub time_buff: TimeBuff0104,
    pub condition_bit_flag: i32,
}

impl WirePayload for PcBuffUpdate0104 {
    const SIZE: usize = 44;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.buff_id);
        write_i32(&mut out, 4, self.update_kind);
        write_i32(&mut out, 8, self.buff_type);
        out[12..40].copy_from_slice(&self.time_buff.encode());
        write_i32(&mut out, 40, self.condition_bit_flag);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            buff_id: read_i32(bytes, 0),
            update_kind: read_i32(bytes, 4),
            buff_type: read_i32(bytes, 8),
            time_buff: TimeBuff0104::decode(&bytes[12..40])?,
            condition_bit_flag: read_i32(bytes, 40),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_PC_CASH_BUFF_UPDATE` (pack(4), 40 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcCashBuffUpdate0104 {
    pub buff_id: i32,
    pub update_kind: i32,
    pub time_buff: TimeBuff0104,
    pub condition_bit_flag: i32,
}

impl WirePayload for PcCashBuffUpdate0104 {
    const SIZE: usize = 40;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.buff_id);
        write_i32(&mut out, 4, self.update_kind);
        out[8..36].copy_from_slice(&self.time_buff.encode());
        write_i32(&mut out, 36, self.condition_bit_flag);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            buff_id: read_i32(bytes, 0),
            update_kind: read_i32(bytes, 4),
            time_buff: TimeBuff0104::decode(&bytes[8..36])?,
            condition_bit_flag: read_i32(bytes, 36),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_CHAR_TIME_BUFF_TIME_OUT` (pack(4), 12 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharTimeBuffTimeout0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub condition_bit_flag: i32,
}

impl WirePayload for CharTimeBuffTimeout0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.character_type);
        write_i32(&mut out, 4, self.character_id);
        write_i32(&mut out, 8, self.condition_bit_flag);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            condition_bit_flag: read_i32(bytes, 8),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillBuffPacket0104 {
    Pc(PcBuffUpdate0104),
    Cash(PcCashBuffUpdate0104),
    Timeout(CharTimeBuffTimeout0104),
}

#[derive(Clone, PartialEq, Eq)]
pub struct FixedUtf16<const N: usize>(pub(super) [u16; N]);

impl<const N: usize> FixedUtf16<N> {
    pub const fn zeroed() -> Self {
        Self([0; N])
    }

    pub const fn from_units(units: [u16; N]) -> Self {
        Self(units)
    }

    pub fn from_str(value: &str) -> Result<Self, PayloadError> {
        let encoded = value.encode_utf16().collect::<Vec<_>>();
        if encoded.len() >= N {
            return Err(PayloadError::Utf16TooLong {
                capacity: N,
                actual: encoded.len(),
            });
        }
        let mut units = [0u16; N];
        units[..encoded.len()].copy_from_slice(&encoded);
        Ok(Self(units))
    }

    pub const fn as_units(&self) -> &[u16; N] {
        &self.0
    }

    pub fn to_string_lossy(&self) -> String {
        let length = self.0.iter().position(|unit| *unit == 0).unwrap_or(N);
        String::from_utf16_lossy(&self.0[..length])
    }
}

impl<const N: usize> Default for FixedUtf16<N> {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl<const N: usize> fmt::Debug for FixedUtf16<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FixedUtf16")
            .field(&self.to_string_lossy())
            .finish()
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_SEND_FREECHAT_MESSAGE`
/// (`char16_t[128]` followed by one pack(4) `int32_t`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeChatRequest0104 {
    pub message: FixedUtf16<128>,
    pub emote_code: i32,
}

impl WirePayload for FreeChatRequest0104 {
    const SIZE: usize = 260;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.message);
        write_i32(&mut out, 256, self.emote_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            message: read_utf16(bytes, 0),
            emote_code: read_i32(bytes, 256),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_SEND_FREECHAT_MESSAGE_SUCC`
/// (`int32_t`, `char16_t[128]`, `int32_t`, pack(4)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeChatSuccess0104 {
    pub pc_id: i32,
    pub message: FixedUtf16<128>,
    pub emote_code: i32,
}

impl WirePayload for FreeChatSuccess0104 {
    const SIZE: usize = 264;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_utf16(&mut out, 4, &self.message);
        write_i32(&mut out, 260, self.emote_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            message: read_utf16(bytes, 4),
            emote_code: read_i32(bytes, 260),
        })
    }
}

/// The confirmed normal FreeChat request/broadcast family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FreeChatPacket0104 {
    Request(FreeChatRequest0104),
    Success(FreeChatSuccess0104),
}

/// Protocol-0104 normal MenuChat uses the same packed fields as FreeChat,
/// but distinct packet IDs and server-side moderation semantics.
pub type MenuChatRequest0104 = FreeChatRequest0104;

pub type MenuChatSuccess0104 = FreeChatSuccess0104;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuChatPacket0104 {
    Request(MenuChatRequest0104),
    Success(MenuChatSuccess0104),
}

/// Shared request/reply ABI for clean `AvatarEmote(int)` chat broadcasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvatarEmoteChat0104 {
    pub pc_id: i32,
    pub emote_code: i32,
}

impl WirePayload for AvatarEmoteChat0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.emote_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            emote_code: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_PC_MOTD_LOGIN`
/// (`int8_t`, one alignment byte, `char16_t[512]`, `#pragma pack(2)`).
///
/// Despite the legacy packet name, OpenFusion also uses this packet for
/// server-authored chat-command output after the player has entered a shard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMessage0104 {
    pub message_type: i8,
    pub message: FixedUtf16<512>,
}

impl WirePayload for ServerMessage0104 {
    const SIZE: usize = 1026;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0] = self.message_type as u8;
        // Byte 1 is the pack(2) alignment gap before the UTF-16 buffer.
        write_utf16(&mut out, 2, &self.message);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            message_type: bytes[0] as i8,
            message: read_utf16(bytes, 2),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_GM_REQ_PC_SET_VALUE` (`#pragma pack(4)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmSetValueRequest0104 {
    pub pc_id: i32,
    pub value_type: i32,
    pub value: i32,
}

impl GmSetValueRequest0104 {
    #[must_use]
    pub const fn speed(pc_id: i32, speed: i32) -> Self {
        Self {
            pc_id,
            value_type: GM_SET_VALUE_SPEED_0104,
            value: speed,
        }
    }
}

impl WirePayload for GmSetValueRequest0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.value_type);
        write_i32(&mut out, 8, self.value);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            value_type: read_i32(bytes, 4),
            value: read_i32(bytes, 8),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_GM_REP_PC_SET_VALUE` (`#pragma pack(4)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmSetValueReply0104 {
    pub pc_id: i32,
    pub value_type: i32,
    pub value: i32,
}

impl WirePayload for GmSetValueReply0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.value_type);
        write_i32(&mut out, 8, self.value);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            value_type: read_i32(bytes, 4),
            value: read_i32(bytes, 8),
        })
    }
}

/// Protocol-0104 `sBuddyBaseInfo` (`#pragma pack(4)`, 72 bytes).
///
/// The wire record has one alignment byte before `szFirstName` and two tail
/// padding bytes. Semantic decoding ignores those bytes; lossless callers keep
/// the original `DecodedFrame` alongside the typed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyBaseInfo0104 {
    pub pc_id: i32,
    pub pc_uid: i64,
    pub blocked: i8,
    pub free_chat: i8,
    pub pc_state: i8,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
    pub gender: i8,
    pub name_check_flag: i8,
}

impl WirePayload for BuddyBaseInfo0104 {
    const SIZE: usize = 72;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i64(&mut out, 4, self.pc_uid);
        out[12] = self.blocked as u8;
        out[13] = self.free_chat as u8;
        out[14] = self.pc_state as u8;
        write_utf16(&mut out, 16, &self.first_name);
        write_utf16(&mut out, 34, &self.last_name);
        out[68] = self.gender as u8;
        out[69] = self.name_check_flag as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            pc_uid: read_i64(bytes, 4),
            blocked: bytes[12] as i8,
            free_chat: bytes[13] as i8,
            pc_state: bytes[14] as i8,
            first_name: read_utf16(bytes, 16),
            last_name: read_utf16(bytes, 34),
            gender: bytes[68] as i8,
            name_check_flag: bytes[69] as i8,
        })
    }
}

/// Proven fixed header of variable
/// `sP_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC`.
///
/// `buddy_count` `sBuddyBaseInfo` records follow this 16-byte prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuddyListInfoPrefix0104 {
    pub pc_id: i32,
    pub pc_uid: i64,
    pub list_num: i8,
    pub buddy_count: i8,
    pub pack_padding: [u8; 2],
}

impl BuddyListInfoPrefix0104 {
    pub const SIZE: usize = 16;

    pub fn encode_prefix(self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i64(&mut out, 4, self.pc_uid);
        out[12] = self.list_num as u8;
        out[13] = self.buddy_count as u8;
        out[14..16].copy_from_slice(&self.pack_padding);
        out
    }

    pub fn decode_prefix(payload: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(payload, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(payload, 0),
            pc_uid: read_i64(payload, 4),
            list_num: payload[12] as i8,
            buddy_count: payload[13] as i8,
            pack_padding: payload[14..16].try_into().expect("fixed two-byte padding"),
        })
    }
}

/// Complete variable protocol-0104
/// `sP_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC`.
///
/// The clean manager writes each decoded record to
/// `m_sBuddyList[list_num + index]`, whose capacity is exactly 50. Decoding
/// therefore rejects negative/out-of-range indices, any slice crossing slot
/// 50, and every payload whose length is not exactly
/// `16 + buddy_count * 72`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyListInfo0104 {
    pub pc_id: i32,
    pub pc_uid: i64,
    pub list_num: i8,
    pub pack_padding: [u8; 2],
    pub buddies: Vec<BuddyBaseInfo0104>,
}

impl BuddyListInfo0104 {
    pub const HEADER_SIZE: usize = BuddyListInfoPrefix0104::SIZE;
    pub const ENTRY_SIZE: usize = BuddyBaseInfo0104::SIZE;

    pub fn buddy_count(&self) -> usize {
        self.buddies.len()
    }

    pub fn encode(&self) -> Result<Vec<u8>, PayloadError> {
        let list_num = validate_buddy_list_value("buddy list_num", i32::from(self.list_num))?;
        let buddy_count = validate_buddy_list_value(
            "buddy count",
            i32::try_from(self.buddies.len()).unwrap_or(i32::MAX),
        )?;
        validate_buddy_list_end(list_num, buddy_count)?;

        let mut out = BuddyListInfoPrefix0104 {
            pc_id: self.pc_id,
            pc_uid: self.pc_uid,
            list_num: self.list_num,
            buddy_count: buddy_count as i8,
            pack_padding: self.pack_padding,
        }
        .encode_prefix();
        out.reserve(buddy_count * Self::ENTRY_SIZE);
        for buddy in &self.buddies {
            out.extend_from_slice(&buddy.encode());
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, PayloadError> {
        let prefix = BuddyListInfoPrefix0104::decode_prefix(payload)?;
        let list_num = validate_buddy_list_value("buddy list_num", i32::from(prefix.list_num))?;
        let buddy_count = validate_buddy_list_value("buddy count", i32::from(prefix.buddy_count))?;
        validate_buddy_list_end(list_num, buddy_count)?;

        let expected = Self::HEADER_SIZE + buddy_count * Self::ENTRY_SIZE;
        require_size(payload, expected)?;
        let buddies = (0..buddy_count)
            .map(|index| {
                let start = Self::HEADER_SIZE + index * Self::ENTRY_SIZE;
                BuddyBaseInfo0104::decode(&payload[start..start + Self::ENTRY_SIZE])
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            pc_id: prefix.pc_id,
            pc_uid: prefix.pc_uid,
            list_num: prefix.list_num,
            pack_padding: prefix.pack_padding,
            buddies,
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_REQUEST_MAKE_BUDDY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuddyMakeRequest0104 {
    pub buddy_id: i32,
    pub buddy_pc_uid: i64,
}

impl WirePayload for BuddyMakeRequest0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.buddy_id);
        write_i64(&mut out, 4, self.buddy_pc_uid);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            buddy_id: read_i32(bytes, 0),
            buddy_pc_uid: read_i64(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_ACCEPT_MAKE_BUDDY`.
///
/// Bytes 1..4 are pack(4) alignment padding and are zeroed on outbound encode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuddyAcceptRequest0104 {
    pub accept_flag: i8,
    pub buddy_id: i32,
    pub buddy_pc_uid: i64,
}
