use super::*;

impl WirePayload for CharacterCreateRequest0104 {
    const SIZE: usize = 100;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.style.encode_into(&mut out[0..PcStyle0104::SIZE]);
        self.equipped
            .encode_into(&mut out[76..76 + OnItem0104::SIZE]);
        self.selected_indices
            .encode_into(&mut out[90..90 + OnItemIndex0104::SIZE]);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            style: PcStyle0104::decode_exact(&bytes[0..76]),
            equipped: OnItem0104::decode_exact(&bytes[76..90]),
            selected_indices: OnItemIndex0104::decode_exact(&bytes[90..100]),
        })
    }
}

/// Exact protocol-0104 `sP_LS2CL_REP_CHAR_CREATE_SUCC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterCreateSuccess0104 {
    pub level: i16,
    pub style: PcStyle0104,
    pub style2: PcStyle2Flags0104,
    pub equipped: OnItem0104,
}

impl WirePayload for CharacterCreateSuccess0104 {
    const SIZE: usize = 100;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.level.to_le_bytes());
        // Bytes 2..4 align nested sPCStyle to pack(4).
        self.style.encode_into(&mut out[4..80]);
        self.style2.encode_into(&mut out[80..83]);
        // Byte 83 aligns pack(2) sOnItem.
        self.equipped.encode_into(&mut out[84..98]);
        // Bytes 98..100 are outer pack(4) tail padding.
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            level: read_i16(bytes, 0),
            style: PcStyle0104::decode_exact(&bytes[4..80]),
            style2: PcStyle2Flags0104::decode_exact(&bytes[80..83]),
            equipped: OnItem0104::decode_exact(&bytes[84..98]),
        })
    }
}

/// Protocol-0104 `sItemBase` used by player appearance equipment slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBase0104 {
    pub item_type: i16,
    pub item_id: i16,
    pub option: i32,
    pub time_limit: i32,
}

impl ItemBase0104 {
    pub const SIZE: usize = 12;

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item_type: read_i16(bytes, 0),
            item_id: read_i16(bytes, 2),
            option: read_i32(bytes, 4),
            time_limit: read_i32(bytes, 8),
        }
    }

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        bytes[0..2].copy_from_slice(&self.item_type.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.item_id.to_le_bytes());
        write_i32(bytes, 4, self.option);
        write_i32(bytes, 8, self.time_limit);
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_NANO_EQUIP`
/// (`#pragma pack(2)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoEquipRequest0104 {
    pub nano_id: i16,
    pub nano_slot: i16,
}

impl WirePayload for NanoEquipRequest0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.nano_id.to_le_bytes());
        out[2..4].copy_from_slice(&self.nano_slot.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            nano_id: read_i16(bytes, 0),
            nano_slot: read_i16(bytes, 2),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_NANO_UNEQUIP`
/// (`#pragma pack(2)`, 2 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoUnequipRequest0104 {
    pub nano_slot: i16,
}

impl WirePayload for NanoUnequipRequest0104 {
    const SIZE: usize = 2;

    fn encode(&self) -> Vec<u8> {
        self.nano_slot.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            nano_slot: read_i16(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_NANO_ACTIVE`
/// (`#pragma pack(2)`, 2 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoActiveRequest0104 {
    pub nano_slot: i16,
}

impl WirePayload for NanoActiveRequest0104 {
    const SIZE: usize = 2;

    fn encode(&self) -> Vec<u8> {
        self.nano_slot.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            nano_slot: read_i16(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_NANO_ACTIVE_SUCC`
/// (`#pragma pack(4)`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoActiveSuccess0104 {
    pub active_nano_slot: i16,
    /// ABI padding between `iActiveNanoSlotNum` and `eCSTB___Add`, retained
    /// losslessly instead of assuming that a foreign shard zeroes it.
    pub pack_padding: [u8; 2],
    pub condition_status_add: i32,
}

impl WirePayload for NanoActiveSuccess0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.active_nano_slot.to_le_bytes());
        out[2..4].copy_from_slice(&self.pack_padding);
        write_i32(&mut out, 4, self.condition_status_add);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        let active_nano_slot = read_i16(bytes, 0);
        if !(-1..=2).contains(&active_nano_slot) {
            return Err(PayloadError::ValueOutOfRange {
                field: "active nano slot",
                value: i32::from(active_nano_slot),
                minimum: -1,
                maximum: 2,
            });
        }
        Ok(Self {
            active_nano_slot,
            pack_padding: bytes[2..4].try_into().expect("fixed two-byte padding"),
            condition_status_add: read_i32(bytes, 4),
        })
    }
}

/// Exact 36-byte common prefix of protocol-0104
/// `sP_FE2CL_NANO_SKILL_USE_SUCC`. Its result tail is `eST`-dependent and is
/// not safe to commit until [`NanoSkillUseSuccess0104::decode`] validates the
/// complete payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillUseSuccessPrefix0104 {
    pub pc_id: i32,
    pub bullet_id: i8,
    /// ABI padding between `iBulletID` and `iSkillID`.
    pub pack_padding: [u8; 1],
    pub skill_id: i16,
    pub arg1: i32,
    pub arg2: i32,
    pub arg3: i32,
    pub nano_deactivated: i32,
    pub nano_id: i16,
    pub nano_stamina: i16,
    pub skill_type: i32,
    pub target_count: i32,
}

impl NanoSkillUseSuccessPrefix0104 {
    pub const SIZE: usize = 36;

    #[must_use]
    pub fn encode_prefix(self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        out[4] = self.bullet_id as u8;
        out[5] = self.pack_padding[0];
        out[6..8].copy_from_slice(&self.skill_id.to_le_bytes());
        write_i32(&mut out, 8, self.arg1);
        write_i32(&mut out, 12, self.arg2);
        write_i32(&mut out, 16, self.arg3);
        write_i32(&mut out, 20, self.nano_deactivated);
        out[24..26].copy_from_slice(&self.nano_id.to_le_bytes());
        out[26..28].copy_from_slice(&self.nano_stamina.to_le_bytes());
        write_i32(&mut out, 28, self.skill_type);
        write_i32(&mut out, 32, self.target_count);
        out
    }

    pub(super) fn decode_prefix(payload: &[u8]) -> Result<Self, PayloadError> {
        require_prefix(payload, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(payload, 0),
            bullet_id: payload[4] as i8,
            pack_padding: [payload[5]],
            skill_id: read_i16(payload, 6),
            arg1: read_i32(payload, 8),
            arg2: read_i32(payload, 12),
            arg3: read_i32(payload, 16),
            nano_deactivated: read_i32(payload, 20),
            nano_id: read_i16(payload, 24),
            nano_stamina: read_i16(payload, 26),
            skill_type: read_i32(payload, 28),
            target_count: read_i32(payload, 32),
        })
    }
}

/// Raw protocol-0104 combatant identity used by every OpenFusion Nano result.
/// `eCT` is intentionally retained as an integer here; the world projection
/// validates the only server-proven values (PC=1, NPC=2, MOB=4) before an
/// entity can be mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillTarget0104 {
    pub entity_type: i32,
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillDamageResult0104 {
    pub target: NanoSkillTarget0104,
    pub protected: i32,
    pub damage: i32,
    /// Authoritative absolute post-damage HP.
    pub hp: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillHealHpResult0104 {
    pub target: NanoSkillTarget0104,
    pub healed_hp: i32,
    /// Authoritative absolute post-heal HP.
    pub hp: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillDamageDebuffResult0104 {
    pub target: NanoSkillTarget0104,
    pub protected: i32,
    pub damage: i32,
    /// Authoritative absolute target HP after the handler.
    pub hp: i32,
    pub nano_stamina: i16,
    /// `#pragma pack(4)` alignment between `int16_t iStamina` and the next
    /// `int32_t`; retained so the typed representation stays lossless.
    pub pack_padding: [u8; 2],
    pub nano_deactivated: i32,
    /// Authoritative complete condition bitfield after the handler.
    pub condition_bit_flag: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillBuffResult0104 {
    pub target: NanoSkillTarget0104,
    pub protected: i32,
    /// Authoritative complete condition bitfield after the handler.
    pub condition_bit_flag: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillBatteryDrainResult0104 {
    pub target: NanoSkillTarget0104,
    pub protected: i32,
    pub drained_weapon_battery: i32,
    /// Authoritative absolute post-drain weapon battery.
    pub weapon_battery: i32,
    pub drained_nano_battery: i32,
    /// Authoritative absolute post-drain Nano battery.
    pub nano_battery: i32,
    pub nano_stamina: i16,
    /// `#pragma pack(4)` alignment before `bNanoDeactive`.
    pub pack_padding: [u8; 2],
    pub nano_deactivated: i32,
    /// Authoritative complete condition bitfield after the handler.
    pub condition_bit_flag: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillMoveResult0104 {
    pub target: NanoSkillTarget0104,
    pub map_number: i32,
    /// Authoritative server-space destination.
    pub position: [i32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillResurrectResult0104 {
    pub target: NanoSkillTarget0104,
    /// OpenFusion serializes the target's absolute current HP in `iRegenHP`.
    pub hp: i32,
}

/// OpenFusion's custom 36-byte `sSkillResult_Leech`: a 16-byte heal record
/// followed by a 20-byte damage record. The server copies the damage target's
/// `eCT/iID` into `heal.target`, even though `heal.hp` is the caster's HP; the
/// world projection deliberately keys that HP by the packet prefix's PC ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoSkillLeechResult0104 {
    pub heal: NanoSkillHealHpResult0104,
    pub damage: NanoSkillDamageResult0104,
}

/// Typed, lossless result families proven by OpenFusion's
/// `Abilities::handleSkill` switch for protocol 0104.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NanoSkillResult0104 {
    Damage(NanoSkillDamageResult0104),
    HealHp(NanoSkillHealHpResult0104),
    DamageDebuff(NanoSkillDamageDebuffResult0104),
    Buff(NanoSkillBuffResult0104),
    BatteryDrain(NanoSkillBatteryDrainResult0104),
    Move(NanoSkillMoveResult0104),
    Resurrect(NanoSkillResurrectResult0104),
    Leech(NanoSkillLeechResult0104),
}

impl NanoSkillResult0104 {
    /// The result's server-authored target. For Leech this is the damaged
    /// target; caster healing is identified by the surrounding packet prefix.
    #[must_use]
    pub const fn target(self) -> NanoSkillTarget0104 {
        match self {
            Self::Damage(result) => result.target,
            Self::HealHp(result) => result.target,
            Self::DamageDebuff(result) => result.target,
            Self::Buff(result) => result.target,
            Self::BatteryDrain(result) => result.target,
            Self::Move(result) => result.target,
            Self::Resurrect(result) => result.target,
            Self::Leech(result) => result.damage.target,
        }
    }
}

/// Fully validated protocol-0104 `sP_FE2CL_NANO_SKILL_USE_SUCC` wire body.
/// The identical `sP_FE2CL_NANO_SKILL_USE` broadcast uses the same decoder.
/// Both lossless raw bytes and typed `eST`-specific results are retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NanoSkillUseSuccess0104 {
    pub(super) prefix: NanoSkillUseSuccessPrefix0104,
    pub(super) result_bytes: Vec<u8>,
    pub(super) results: Vec<NanoSkillResult0104>,
}

impl NanoSkillUseSuccess0104 {
    pub const MAX_TARGETS: usize = NANO_SKILL_RESULT_MAX_TARGETS_0104;

    pub fn decode(payload: &[u8]) -> Result<Self, NanoSkillUseDecodeError0104> {
        let prefix = NanoSkillUseSuccessPrefix0104::decode_prefix(payload)?;
        if prefix.target_count < 0 {
            return Err(NanoSkillUseDecodeError0104::NegativeTargetCount {
                target_count: prefix.target_count,
            });
        }
        if prefix.target_count == 0 {
            return Err(NanoSkillUseDecodeError0104::EmptyTargetResults);
        }
        let target_count =
            usize::try_from(prefix.target_count).expect("non-negative i32 target count fits usize");
        if target_count > Self::MAX_TARGETS {
            return Err(NanoSkillUseDecodeError0104::TargetCountTooLarge {
                target_count: prefix.target_count,
                maximum: Self::MAX_TARGETS,
            });
        }
        let record_size = nano_skill_result_size_0104(prefix.skill_type).ok_or(
            NanoSkillUseDecodeError0104::UnsupportedSkillType {
                skill_type: prefix.skill_type,
            },
        )?;
        let expected = target_count
            .checked_mul(record_size)
            .and_then(|tail_size| NanoSkillUseSuccessPrefix0104::SIZE.checked_add(tail_size))
            .ok_or(NanoSkillUseDecodeError0104::PayloadSizeOverflow {
                record_size,
                target_count: prefix.target_count,
            })?;
        require_size(payload, expected)?;
        let result_bytes = payload[NanoSkillUseSuccessPrefix0104::SIZE..].to_vec();
        let results = result_bytes
            .chunks_exact(record_size)
            .map(|record| decode_nano_skill_result_0104(prefix.skill_type, record))
            .collect::<Result<Vec<_>, _>>()?;
        debug_assert_eq!(results.len(), target_count);
        Ok(Self {
            prefix,
            result_bytes,
            results,
        })
    }

    #[must_use]
    pub const fn prefix(&self) -> NanoSkillUseSuccessPrefix0104 {
        self.prefix
    }

    #[must_use]
    pub fn result_bytes(&self) -> &[u8] {
        &self.result_bytes
    }

    /// Typed records in authoritative server application order.
    #[must_use]
    pub fn results(&self) -> &[NanoSkillResult0104] {
        &self.results
    }

    #[must_use]
    pub fn result_record_size(&self) -> usize {
        nano_skill_result_size_0104(self.prefix.skill_type)
            .expect("validated Nano-skill success retains a supported eST")
    }
}

/// Which authoritative OpenFusion delivery carried a Nano result body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NanoSkillUseDelivery0104 {
    /// Caster-only `P_FE2CL_NANO_SKILL_USE_SUCC`.
    LocalSuccess,
    /// Viewer-facing `P_FE2CL_NANO_SKILL_USE` with the identical body/tail.
    RemoteUse,
}

/// A typed Nano result together with the packet route which delivered it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NanoSkillUsePacket0104 {
    pub delivery: NanoSkillUseDelivery0104,
    pub result: NanoSkillUseSuccess0104,
}

/// Exact protocol-0104 `sP_CL2FE_REQ_NANO_SKILL_USE` header followed by
/// `iTargetCnt` `int32_t` target IDs. The three bytes after `iBulletID` are
/// the `#pragma pack(4)` alignment gap from the clean ABI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NanoSkillUseRequest0104 {
    pub bullet_id: i8,
    pub arg1: i32,
    pub arg2: i32,
    pub arg3: i32,
    pub target_ids: Vec<i32>,
}

impl NanoSkillUseRequest0104 {
    pub const HEADER_SIZE: usize = 20;
    /// OpenFusion's 4,088-byte request-payload capacity leaves room for this
    /// many `int32_t` targets after the 20-byte fixed header.
    pub const MAX_TARGETS: usize =
        (OPENFUSION_PAYLOAD_CAPACITY_0104 - Self::HEADER_SIZE) / size_of::<i32>();

    pub fn encode(&self) -> Result<Vec<u8>, CountedPayloadError0104> {
        let mut out = allocate_counted_payload(
            self.target_ids.len(),
            Self::HEADER_SIZE,
            size_of::<i32>(),
            Some(Self::MAX_TARGETS),
        )?;
        out[0] = self.bullet_id as u8;
        write_i32(&mut out, 4, self.arg1);
        write_i32(&mut out, 8, self.arg2);
        write_i32(&mut out, 12, self.arg3);
        write_i32(&mut out, 16, self.target_ids.len() as i32);
        for (index, target_id) in self.target_ids.iter().copied().enumerate() {
            write_i32(&mut out, Self::HEADER_SIZE + index * 4, target_id);
        }
        Ok(out)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CountedPayloadError0104> {
        let target_ids =
            decode_counted_i32(payload, Self::HEADER_SIZE, 16, Some(Self::MAX_TARGETS))?;
        Ok(Self {
            bullet_id: payload[0] as i8,
            arg1: read_i32(payload, 4),
            arg2: read_i32(payload, 8),
            arg3: read_i32(payload, 12),
            target_ids,
        })
    }
}

/// Exact clean-Retrobution protocol-0104 `sP_CL2FE_REQ_NANO_TUNE`
/// (`Pack = 4`, 44 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoTuneRequest0104 {
    pub nano_id: i16,
    pub tune_id: i16,
    pub needed_item_slots: [i32; NANO_TUNE_ITEM_SLOT_COUNT_0104],
}

impl WirePayload for NanoTuneRequest0104 {
    const SIZE: usize = 44;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.nano_id.to_le_bytes());
        out[2..4].copy_from_slice(&self.tune_id.to_le_bytes());
        for (index, slot) in self.needed_item_slots.iter().copied().enumerate() {
            write_i32(&mut out, 4 + index * 4, slot);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            nano_id: read_i16(bytes, 0),
            tune_id: read_i16(bytes, 2),
            needed_item_slots: std::array::from_fn(|index| read_i32(bytes, 4 + index * 4)),
        })
    }
}

/// Exact clean-Retrobution protocol-0104 `sP_FE2CL_REP_NANO_TUNE_SUCC`
/// (`Pack = 4`, 168 bytes). Every authoritative inventory slot and item is
/// retained; callers must not synthesize these values from the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoTuneSuccess0104 {
    pub nano_id: i16,
    pub skill_id: i16,
    pub fusion_matter: i32,
    pub item_slots: [i32; NANO_TUNE_ITEM_SLOT_COUNT_0104],
    pub items: [ItemBase0104; NANO_TUNE_ITEM_SLOT_COUNT_0104],
}

impl WirePayload for NanoTuneSuccess0104 {
    const SIZE: usize = 168;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..2].copy_from_slice(&self.nano_id.to_le_bytes());
        out[2..4].copy_from_slice(&self.skill_id.to_le_bytes());
        write_i32(&mut out, 4, self.fusion_matter);
        for (index, slot) in self.item_slots.iter().copied().enumerate() {
            write_i32(&mut out, 8 + index * 4, slot);
        }
        for (index, item) in self.items.iter().copied().enumerate() {
            let start = 48 + index * ItemBase0104::SIZE;
            item.encode_into(&mut out[start..start + ItemBase0104::SIZE]);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            nano_id: read_i16(bytes, 0),
            skill_id: read_i16(bytes, 2),
            fusion_matter: read_i32(bytes, 4),
            item_slots: std::array::from_fn(|index| read_i32(bytes, 8 + index * 4)),
            items: std::array::from_fn(|index| {
                let start = 48 + index * ItemBase0104::SIZE;
                ItemBase0104::decode_exact(&bytes[start..start + ItemBase0104::SIZE])
            }),
        })
    }
}

/// Exact clean-Retrobution protocol-0104 `sP_FE2CL_REP_NANO_TUNE_FAIL`
/// (`Pack = 4`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoTuneFailure0104 {
    pub pc_id: i32,
    pub error_code: i32,
}

impl WirePayload for NanoTuneFailure0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.pc_id);
        write_i32(&mut out, 4, self.error_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_id: read_i32(bytes, 0),
            error_code: read_i32(bytes, 4),
        })
    }
}

/// Strictly decoded clean-Retrobution Nano tune reply family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NanoTunePacket0104 {
    Success(NanoTuneSuccess0104),
    Failure(NanoTuneFailure0104),
}

/// Exact protocol-0104 `sItemVendor`
/// (`#pragma pack(4)`, 24 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemVendor0104 {
    pub vendor_id: i32,
    pub buy_cost: f32,
    pub item: ItemBase0104,
    pub sort_num: i32,
}

impl ItemVendor0104 {
    pub const SIZE: usize = 24;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            vendor_id: read_i32(bytes, 0),
            buy_cost: read_f32(bytes, 4),
            item: ItemBase0104::decode_exact(&bytes[8..20]),
            sort_num: read_i32(bytes, 20),
        }
    }

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        write_i32(bytes, 0, self.vendor_id);
        write_f32(bytes, 4, self.buy_cost);
        self.item.encode_into(&mut bytes[8..20]);
        write_i32(bytes, 20, self.sort_num);
    }
}

impl WirePayload for ItemVendor0104 {
    const SIZE: usize = 24;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.encode_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_VENDOR_ITEM_BUY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorItemBuyRequest0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
    pub list_id: i8,
    pub item: ItemBase0104,
    pub inventory_slot: i32,
}

impl WirePayload for VendorItemBuyRequest0104 {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.vendor_id);
        out[8] = self.list_id as u8;
        // Bytes 9..12 are pack(4) padding.
        self.item.encode_into(&mut out[12..24]);
        write_i32(&mut out, 24, self.inventory_slot);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            vendor_id: read_i32(bytes, 4),
            list_id: bytes[8] as i8,
            item: ItemBase0104::decode_exact(&bytes[12..24]),
            inventory_slot: read_i32(bytes, 24),
        })
    }
}

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_VENDOR_ITEM_SELL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorItemSellRequest0104 {
    pub inventory_slot: i32,
    pub item_count: i32,
}
