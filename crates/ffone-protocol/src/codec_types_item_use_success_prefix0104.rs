use super::*;

/// The official client starts at zero per TCP connection and wraps after 4095.
#[derive(Debug, Clone, Default)]
pub struct LegacyClientEncoder {
    pub(super) next_sequence: u16,
}

impl LegacyClientEncoder {
    pub const fn new() -> Self {
        Self { next_sequence: 0 }
    }

    pub const fn sequence(&self) -> u16 {
        self.next_sequence
    }

    pub fn encode(
        &mut self,
        packet_type: u32,
        payload: &[u8],
        key: u64,
    ) -> Result<Vec<u8>, FrameError> {
        let frame = encode_client_frame(packet_type, payload, key, self.next_sequence)?;
        self.next_sequence = (self.next_sequence + 1) & MAX_PACKET_FLAGS;
        Ok(frame)
    }

    pub fn reset(&mut self) {
        self.next_sequence = 0;
    }
}

/// Protocol-0104 `sQuickSlot` (`#pragma pack(2)`, 4 bytes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QuickSlotEntry0104 {
    pub item_type: i16,
    pub item_id: i16,
}

impl QuickSlotEntry0104 {
    pub const SIZE: usize = 4;

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        bytes[0..2].copy_from_slice(&self.item_type.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.item_id.to_le_bytes());
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item_type: read_i16(bytes, 0),
            item_id: read_i16(bytes, 2),
        }
    }
}

/// Protocol-0104 `sP_FE2CL_PC_QUICK_SLOT_INFO`
/// (`sQuickSlot[8]`, `#pragma pack(2)`, 32 bytes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QuickSlotInfo0104 {
    pub slots: [QuickSlotEntry0104; QUICK_SLOT_COUNT_0104],
}

impl WirePayload for QuickSlotInfo0104 {
    const SIZE: usize = QUICK_SLOT_COUNT_0104 * QuickSlotEntry0104::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        for (index, slot) in self.slots.iter().copied().enumerate() {
            let start = index * QuickSlotEntry0104::SIZE;
            slot.encode_into(&mut out[start..start + QuickSlotEntry0104::SIZE]);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            slots: std::array::from_fn(|index| {
                let start = index * QuickSlotEntry0104::SIZE;
                QuickSlotEntry0104::decode_exact(&bytes[start..start + QuickSlotEntry0104::SIZE])
            }),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_REGIST_QUICK_SLOT`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickSlotRegisterRequest0104 {
    pub slot_num: i32,
    pub item_type: i16,
    pub item_id: i16,
}

impl WirePayload for QuickSlotRegisterRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.slot_num);
        out[4..6].copy_from_slice(&self.item_type.to_le_bytes());
        out[6..8].copy_from_slice(&self.item_id.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            slot_num: read_i32(bytes, 0),
            item_type: read_i16(bytes, 4),
            item_id: read_i16(bytes, 6),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickSlotRegisterSuccess0104 {
    pub slot_num: i32,
    pub item_type: i16,
    pub item_id: i16,
}

impl WirePayload for QuickSlotRegisterSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        QuickSlotRegisterRequest0104 {
            slot_num: self.slot_num,
            item_type: self.item_type,
            item_id: self.item_id,
        }
        .encode()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        let request = QuickSlotRegisterRequest0104::decode(bytes)?;
        Ok(Self {
            slot_num: request.slot_num,
            item_type: request.item_type,
            item_id: request.item_id,
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickSlotRegisterFailure0104 {
    pub error_code: i32,
}

impl WirePayload for QuickSlotRegisterFailure0104 {
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

/// Protocol-0104 `sP_CL2FE_REQ_ITEM_USE` (`#pragma pack(4)`, 12 bytes).
///
/// Bytes 10..12 are ABI tail padding. Native outbound encoding always emits
/// them as zeroes, matching the zero-initialized managed struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUseRequest0104 {
    pub item_location: i32,
    pub slot_num: i32,
    pub nano_slot: i16,
}

impl WirePayload for ItemUseRequest0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.item_location);
        write_i32(&mut out, 4, self.slot_num);
        out[8..10].copy_from_slice(&self.nano_slot.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            item_location: read_i32(bytes, 0),
            slot_num: read_i32(bytes, 4),
            nano_slot: read_i16(bytes, 8),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_ITEM_CHEST_OPEN`
/// (`#pragma pack(4)`, 20 bytes).
///
/// The clean client sends this dedicated request for inventory item type 9;
/// chests are not opened through [`ItemUseRequest0104`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemChestOpenRequest0104 {
    pub item_location: i32,
    pub slot_num: i32,
    pub chest_item: ItemBase0104,
}

impl WirePayload for ItemChestOpenRequest0104 {
    const SIZE: usize = 20;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.item_location);
        write_i32(&mut out, 4, self.slot_num);
        self.chest_item.encode_into(&mut out[8..20]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            item_location: read_i32(bytes, 0),
            slot_num: read_i32(bytes, 4),
            chest_item: ItemBase0104::decode_exact(&bytes[8..20]),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_ITEM_CHEST_OPEN_SUCC`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemChestOpenSuccess0104 {
    pub slot_num: i32,
}

impl WirePayload for ItemChestOpenSuccess0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.slot_num.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            slot_num: read_i32(bytes, 0),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_ITEM_CHEST_OPEN_FAIL`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemChestOpenFailure0104 {
    pub slot_num: i32,
    pub error_code: i32,
}

impl WirePayload for ItemChestOpenFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.slot_num);
        write_i32(&mut out, 4, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            slot_num: read_i32(bytes, 0),
            error_code: read_i32(bytes, 4),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_ITEM_USE_FAIL`
/// (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUseFailure0104 {
    pub error_code: i32,
}

impl WirePayload for ItemUseFailure0104 {
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

/// Proven fixed 36-byte base of protocol-0104
/// `sP_FE2CL_REP_PC_ITEM_USE_SUCC`.
///
/// A positive `target_count` is followed by an `eST`-dependent skill-result
/// array. The clean client dispatches several different result structs, so
/// this prefix is intentionally not a `WirePayload` and is never treated as
/// the complete variable packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUseSuccessPrefix0104 {
    pub pc_id: i32,
    pub item_location: i32,
    pub slot_num: i32,
    pub remaining_item: ItemBase0104,
    pub skill_id: i16,
    pub pack_padding: [u8; 2],
    pub skill_type: i32,
    pub target_count: i32,
}

impl ItemUseSuccessPrefix0104 {
    pub const SIZE: usize = 36;

    pub fn encode_prefix(self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.item_location);
        write_i32(&mut out, 8, self.slot_num);
        self.remaining_item.encode_into(&mut out[12..24]);
        out[24..26].copy_from_slice(&self.skill_id.to_le_bytes());
        out[26..28].copy_from_slice(&self.pack_padding);
        write_i32(&mut out, 28, self.skill_type);
        write_i32(&mut out, 32, self.target_count);
        out
    }

    pub fn decode_prefix(payload: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(payload, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(payload, 0),
            item_location: read_i32(payload, 4),
            slot_num: read_i32(payload, 8),
            remaining_item: ItemBase0104::decode_exact(&payload[12..24]),
            skill_id: read_i16(payload, 24),
            pack_padding: payload[26..28].try_into().expect("fixed two-byte padding"),
            skill_type: read_i32(payload, 28),
            target_count: read_i32(payload, 32),
        })
    }
}

/// Proven fixed 16-byte base of protocol-0104 `sP_FE2CL_PC_ITEM_USE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUseBroadcastPrefix0104 {
    pub pc_id: i32,
    pub skill_id: i16,
    pub pack_padding: [u8; 2],
    pub skill_type: i32,
    pub target_count: i32,
}

impl ItemUseBroadcastPrefix0104 {
    pub const SIZE: usize = 16;

    pub fn encode_prefix(self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        out[4..6].copy_from_slice(&self.skill_id.to_le_bytes());
        out[6..8].copy_from_slice(&self.pack_padding);
        write_i32(&mut out, 8, self.skill_type);
        write_i32(&mut out, 12, self.target_count);
        out
    }

    pub fn decode_prefix(payload: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(payload, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(payload, 0),
            skill_id: read_i16(payload, 4),
            pack_padding: payload[6..8].try_into().expect("fixed two-byte padding"),
            skill_type: read_i32(payload, 8),
            target_count: read_i32(payload, 12),
        })
    }
}

/// A clean Retrobution `sSkillResult_Damage` record (`#pragma pack(4)`, 20
/// bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultDamage0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub protected: i32,
    pub damage: i32,
    pub hp: i32,
}

impl SkillResultDamage0104 {
    pub const SIZE: usize = 20;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            protected: read_i32(bytes, 8),
            damage: read_i32(bytes, 12),
            hp: read_i32(bytes, 16),
        }
    }
}

/// A clean Retrobution `sSkillResult_Heal_HP` record (`#pragma pack(4)`, 16
/// bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultHealHp0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub healed_hp: i32,
    pub hp: i32,
}

impl SkillResultHealHp0104 {
    pub const SIZE: usize = 16;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            healed_hp: read_i32(bytes, 8),
            hp: read_i32(bytes, 12),
        }
    }
}

/// A clean Retrobution `sSkillResult_Heal_Stamina` record (`#pragma pack(4)`,
/// 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultHealStamina0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub healed_nano_stamina: i16,
    pub nano: Nano0104,
}

impl SkillResultHealStamina0104 {
    pub const SIZE: usize = 16;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            healed_nano_stamina: read_i16(bytes, 8),
            nano: Nano0104::decode_exact(&bytes[10..16]),
        }
    }
}

/// A clean Retrobution `sSkillResult_Damage_N_Debuff` record
/// (`#pragma pack(4)`, 32 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultDamageDebuff0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub protected: i32,
    pub damage: i32,
    pub hp: i32,
    pub stamina: i16,
    /// ABI padding between `iStamina` and `bNanoDeactive`, retained losslessly.
    pub pack_padding: [u8; 2],
    pub nano_deactivated: i32,
    pub condition_bit_flag: i32,
}

impl SkillResultDamageDebuff0104 {
    pub const SIZE: usize = 32;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            protected: read_i32(bytes, 8),
            damage: read_i32(bytes, 12),
            hp: read_i32(bytes, 16),
            stamina: read_i16(bytes, 20),
            pack_padding: bytes[22..24].try_into().expect("fixed two-byte padding"),
            nano_deactivated: read_i32(bytes, 24),
            condition_bit_flag: read_i32(bytes, 28),
        }
    }
}

/// A clean Retrobution `sSkillResult_Buff` record (`#pragma pack(4)`, 16
/// bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultBuff0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub protected: i32,
    pub condition_bit_flag: i32,
}

impl SkillResultBuff0104 {
    pub const SIZE: usize = 16;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            protected: read_i32(bytes, 8),
            condition_bit_flag: read_i32(bytes, 12),
        }
    }
}

/// A clean Retrobution `sSkillResult_BatteryDrain` record
/// (`#pragma pack(4)`, 40 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultBatteryDrain0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub protected: i32,
    pub drained_weapon_battery: i32,
    pub weapon_battery: i32,
    pub drained_nano_battery: i32,
    pub nano_battery: i32,
    pub stamina: i16,
    /// ABI padding between `iStamina` and `bNanoDeactive`, retained losslessly.
    pub pack_padding: [u8; 2],
    pub nano_deactivated: i32,
    pub condition_bit_flag: i32,
}

impl SkillResultBatteryDrain0104 {
    pub const SIZE: usize = 40;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            protected: read_i32(bytes, 8),
            drained_weapon_battery: read_i32(bytes, 12),
            weapon_battery: read_i32(bytes, 16),
            drained_nano_battery: read_i32(bytes, 20),
            nano_battery: read_i32(bytes, 24),
            stamina: read_i16(bytes, 28),
            pack_padding: bytes[30..32].try_into().expect("fixed two-byte padding"),
            nano_deactivated: read_i32(bytes, 32),
            condition_bit_flag: read_i32(bytes, 36),
        }
    }
}

/// A clean Retrobution `sSkillResult_Move` record (`#pragma pack(4)`, 24
/// bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultMove0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub map_number: i32,
    pub position: [i32; 3],
}

impl SkillResultMove0104 {
    pub const SIZE: usize = 24;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            map_number: read_i32(bytes, 8),
            position: [
                read_i32(bytes, 12),
                read_i32(bytes, 16),
                read_i32(bytes, 20),
            ],
        }
    }
}

/// A clean Retrobution `sSkillResult_Resurrect` record (`#pragma pack(4)`, 12
/// bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillResultResurrect0104 {
    pub character_type: i32,
    pub character_id: i32,
    pub regenerated_hp: i32,
}

impl SkillResultResurrect0104 {
    pub const SIZE: usize = 12;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            character_type: read_i32(bytes, 0),
            character_id: read_i32(bytes, 4),
            regenerated_hp: read_i32(bytes, 8),
        }
    }
}

/// Strictly proven clean-client result tails for item-use success/broadcast
/// packets.
///
/// `Bloodsucking` is the one non-uniform layout: the clean client consumes one
/// leading `sSkillResult_Heal_HP`, then `iTargetCnt`
/// `sSkillResult_Damage` records. `None` is valid only for
/// `iTargetCnt == 0`; positive-count skill types for which the client does not
/// deserialize a record remain unsupported rather than being guessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUseSkillResults0104 {
    None,
    Damage(Vec<SkillResultDamage0104>),
    HealHp(Vec<SkillResultHealHp0104>),
    HealStamina(Vec<SkillResultHealStamina0104>),
    DamageDebuff(Vec<SkillResultDamageDebuff0104>),
    Buff(Vec<SkillResultBuff0104>),
    BatteryDrain(Vec<SkillResultBatteryDrain0104>),
    Resurrect(Vec<SkillResultResurrect0104>),
    Move(Vec<SkillResultMove0104>),
    Bloodsucking {
        self_heal: SkillResultHealHp0104,
        targets: Vec<SkillResultDamage0104>,
    },
}

impl ItemUseSkillResults0104 {
    #[must_use]
    pub fn target_count(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Damage(records) => records.len(),
            Self::HealHp(records) => records.len(),
            Self::HealStamina(records) => records.len(),
            Self::DamageDebuff(records) => records.len(),
            Self::Buff(records) => records.len(),
            Self::BatteryDrain(records) => records.len(),
            Self::Resurrect(records) => records.len(),
            Self::Move(records) => records.len(),
            Self::Bloodsucking { targets, .. } => targets.len(),
        }
    }
}

/// Fully validated protocol-0104 `sP_FE2CL_REP_PC_ITEM_USE_SUCC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemUseSuccessPacket0104 {
    pub(super) prefix: ItemUseSuccessPrefix0104,
    pub(super) results: ItemUseSkillResults0104,
}

impl ItemUseSuccessPacket0104 {
    pub fn decode(payload: &[u8]) -> Result<Self, ItemUseDecodeError0104> {
        let prefix = ItemUseSuccessPrefix0104::decode_prefix(payload)?;
        let results = decode_item_use_skill_results_0104(
            prefix.skill_type,
            prefix.target_count,
            ItemUseSuccessPrefix0104::SIZE,
            payload,
        )?;
        Ok(Self { prefix, results })
    }

    #[must_use]
    pub const fn prefix(&self) -> ItemUseSuccessPrefix0104 {
        self.prefix
    }

    #[must_use]
    pub const fn results(&self) -> &ItemUseSkillResults0104 {
        &self.results
    }
}

/// Fully validated protocol-0104 `sP_FE2CL_PC_ITEM_USE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemUseBroadcastPacket0104 {
    pub(super) prefix: ItemUseBroadcastPrefix0104,
    pub(super) results: ItemUseSkillResults0104,
}

impl ItemUseBroadcastPacket0104 {
    pub fn decode(payload: &[u8]) -> Result<Self, ItemUseDecodeError0104> {
        let prefix = ItemUseBroadcastPrefix0104::decode_prefix(payload)?;
        let results = decode_item_use_skill_results_0104(
            prefix.skill_type,
            prefix.target_count,
            ItemUseBroadcastPrefix0104::SIZE,
            payload,
        )?;
        Ok(Self { prefix, results })
    }

    #[must_use]
    pub const fn prefix(&self) -> ItemUseBroadcastPrefix0104 {
        self.prefix
    }

    #[must_use]
    pub const fn results(&self) -> &ItemUseSkillResults0104 {
        &self.results
    }
}

/// Server-to-client item-use packet closure. The original frame remains owned
/// by the net layer even when this typed view is produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUsePacket0104 {
    Failure(ItemUseFailure0104),
    Success(ItemUseSuccessPacket0104),
    Broadcast(ItemUseBroadcastPacket0104),
}

pub(super) trait ItemUseSkillResultRecord0104: Sized {
    const SIZE: usize;
    fn decode_exact(bytes: &[u8]) -> Self;
}

/// Fixed-layout packets in the QuickSlot/general-item-use foundation.
///
/// Item-use success and broadcast packets are deliberately absent from this
/// fixed-layout view. Use [`decode_item_use_packet_0104`] for the separate,
/// `eST`-dependent typed closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSlotPacket0104 {
    RegisterRequest(QuickSlotRegisterRequest0104),
    ItemUseRequest(ItemUseRequest0104),
    Info(QuickSlotInfo0104),
    RegisterFailure(QuickSlotRegisterFailure0104),
    RegisterSuccess(QuickSlotRegisterSuccess0104),
    ItemUseFailure(ItemUseFailure0104),
}
