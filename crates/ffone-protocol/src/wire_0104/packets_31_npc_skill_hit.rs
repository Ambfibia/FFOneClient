// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_NPC_SKILL_HIT` (`#pragma pack(4)`, 28 bytes, packet ID `0x31000022`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSkillHit0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i16,
    /// `iValue1` at offset 8.
    pub value1: i32,
    /// `iValue2` at offset 12.
    pub value2: i32,
    /// `iValue3` at offset 16.
    pub value3: i32,
    /// `eST` at offset 20.
    pub e_st: i32,
    /// `iTargetCnt` at offset 24.
    pub target_cnt: i32,
}

impl NpcSkillHit0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.value1.to_le_bytes());
        write_prim(out, 12, &self.value2.to_le_bytes());
        write_prim(out, 16, &self.value3.to_le_bytes());
        write_prim(out, 20, &self.e_st.to_le_bytes());
        write_prim(out, 24, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 4)),
            value1: i32::from_le_bytes(read_array(bytes, 8)),
            value2: i32::from_le_bytes(read_array(bytes, 12)),
            value3: i32::from_le_bytes(read_array(bytes, 16)),
            e_st: i32::from_le_bytes(read_array(bytes, 20)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for NpcSkillHit0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NPC_SKILL_CORRUPTION_READY` (`#pragma pack(4)`, 20 bytes, packet ID `0x31000023`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSkillCorruptionReady0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i16,
    /// `iStyle` at offset 6.
    pub style: i16,
    /// `iValue1` at offset 8.
    pub value1: i32,
    /// `iValue2` at offset 12.
    pub value2: i32,
    /// `iValue3` at offset 16.
    pub value3: i32,
}

impl NpcSkillCorruptionReady0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 6, &self.style.to_le_bytes());
        write_prim(out, 8, &self.value1.to_le_bytes());
        write_prim(out, 12, &self.value2.to_le_bytes());
        write_prim(out, 16, &self.value3.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 4)),
            style: i16::from_le_bytes(read_array(bytes, 6)),
            value1: i32::from_le_bytes(read_array(bytes, 8)),
            value2: i32::from_le_bytes(read_array(bytes, 12)),
            value3: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for NpcSkillCorruptionReady0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NPC_SKILL_CORRUPTION_HIT` (`#pragma pack(4)`, 24 bytes, packet ID `0x31000024`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSkillCorruptionHit0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i16,
    /// `iStyle` at offset 6.
    pub style: i16,
    /// `iValue1` at offset 8.
    pub value1: i32,
    /// `iValue2` at offset 12.
    pub value2: i32,
    /// `iValue3` at offset 16.
    pub value3: i32,
    /// `iTargetCnt` at offset 20.
    pub target_cnt: i32,
}

impl NpcSkillCorruptionHit0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 6, &self.style.to_le_bytes());
        write_prim(out, 8, &self.value1.to_le_bytes());
        write_prim(out, 12, &self.value2.to_le_bytes());
        write_prim(out, 16, &self.value3.to_le_bytes());
        write_prim(out, 20, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 4)),
            style: i16::from_le_bytes(read_array(bytes, 6)),
            value1: i32::from_le_bytes(read_array(bytes, 8)),
            value2: i32::from_le_bytes(read_array(bytes, 12)),
            value3: i32::from_le_bytes(read_array(bytes, 16)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for NpcSkillCorruptionHit0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NPC_SKILL_CANCEL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000025`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSkillCancel0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
}

impl NpcSkillCancel0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for NpcSkillCancel0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_NANO_EQUIP_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000026`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoEquipSuccess0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
    /// `iNanoSlotNum` at offset 2.
    pub nano_slot_num: i16,
    /// `bNanoDeactive` at offset 4.
    pub nano_deactive: i32,
}

impl NanoEquipSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
        write_prim(out, 2, &self.nano_slot_num.to_le_bytes());
        write_prim(out, 4, &self.nano_deactive.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
            nano_slot_num: i16::from_le_bytes(read_array(bytes, 2)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NanoEquipSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_NANO_UNEQUIP_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000027`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoUnequipSuccess0104 {
    /// `iNanoSlotNum` at offset 0.
    pub nano_slot_num: i16,
    /// `bNanoDeactive` at offset 4.
    pub nano_deactive: i32,
}

impl NanoUnequipSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_slot_num.to_le_bytes());
        write_prim(out, 4, &self.nano_deactive.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_slot_num: i16::from_le_bytes(read_array(bytes, 0)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NanoUnequipSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_NANO_ACTIVE_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x31000028`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoActiveSuccess0104 {
    /// `iActiveNanoSlotNum` at offset 0.
    pub active_nano_slot_num: i16,
    /// `eCSTB___Add` at offset 4.
    pub cstb_add: i32,
}

impl NanoActiveSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.active_nano_slot_num.to_le_bytes());
        write_prim(out, 4, &self.cstb_add.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            active_nano_slot_num: i16::from_le_bytes(read_array(bytes, 0)),
            cstb_add: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NanoActiveSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_NANO_TUNE_SUCC` (`#pragma pack(4)`, 168 bytes, packet ID `0x31000029`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoTuneSuccess0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
    /// `iSkillID` at offset 2.
    pub skill_id: i16,
    /// `iPC_FusionMatter` at offset 4.
    pub pc_fusion_matter: i32,
    /// `aiItemSlotNum` at offset 8.
    pub ai_item_slot_num: [i32; 10],
    /// `aItem` at offset 48.
    pub item: [ItemBase0104; 10],
}

impl NanoTuneSuccess0104 {
    pub const SIZE: usize = 168;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
        write_prim(out, 2, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.pc_fusion_matter.to_le_bytes());
        for (index, value) in self.ai_item_slot_num.iter().enumerate() {
            write_prim(out, 8 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.item.iter().enumerate() {
            let start = 48 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 2)),
            pc_fusion_matter: i32::from_le_bytes(read_array(bytes, 4)),
            ai_item_slot_num: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 8 + index * 4))
            }),
            item: std::array::from_fn(|index| {
                let start = 48 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
        }
    }
}

impl WirePayload for NanoTuneSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NANO_ACTIVE` (`#pragma pack(4)`, 20 bytes, packet ID `0x3100002a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoActive0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `Nano` at offset 4.
    pub nano: Nano0104,
    /// `iConditionBitFlag` at offset 12.
    pub condition_bit_flag: i32,
    /// `eCSTB___Add` at offset 16.
    pub cstb_add: i32,
}

impl NanoActive0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        self.nano.write_into(&mut out[4..10]);
        write_prim(out, 12, &self.condition_bit_flag.to_le_bytes());
        write_prim(out, 16, &self.cstb_add.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            nano: Nano0104::read_from(&bytes[4..10]),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 12)),
            cstb_add: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for NanoActive0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NANO_SKILL_USE_SUCC` (`#pragma pack(4)`, 36 bytes, packet ID `0x3100002b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoSkillUseSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iBulletID` at offset 4.
    pub bullet_id: i8,
    /// `iSkillID` at offset 6.
    pub skill_id: i16,
    /// `iArg1` at offset 8.
    pub arg1: i32,
    /// `iArg2` at offset 12.
    pub arg2: i32,
    /// `iArg3` at offset 16.
    pub arg3: i32,
    /// `bNanoDeactive` at offset 20.
    pub nano_deactive: i32,
    /// `iNanoID` at offset 24.
    pub nano_id: i16,
    /// `iNanoStamina` at offset 26.
    pub nano_stamina: i16,
    /// `eST` at offset 28.
    pub e_st: i32,
    /// `iTargetCnt` at offset 32.
    pub target_cnt: i32,
}

impl NanoSkillUseSuccess0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.bullet_id.to_le_bytes());
        write_prim(out, 6, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.arg1.to_le_bytes());
        write_prim(out, 12, &self.arg2.to_le_bytes());
        write_prim(out, 16, &self.arg3.to_le_bytes());
        write_prim(out, 20, &self.nano_deactive.to_le_bytes());
        write_prim(out, 24, &self.nano_id.to_le_bytes());
        write_prim(out, 26, &self.nano_stamina.to_le_bytes());
        write_prim(out, 28, &self.e_st.to_le_bytes());
        write_prim(out, 32, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 4)),
            skill_id: i16::from_le_bytes(read_array(bytes, 6)),
            arg1: i32::from_le_bytes(read_array(bytes, 8)),
            arg2: i32::from_le_bytes(read_array(bytes, 12)),
            arg3: i32::from_le_bytes(read_array(bytes, 16)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 20)),
            nano_id: i16::from_le_bytes(read_array(bytes, 24)),
            nano_stamina: i16::from_le_bytes(read_array(bytes, 26)),
            e_st: i32::from_le_bytes(read_array(bytes, 28)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for NanoSkillUseSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_NANO_SKILL_USE` (`#pragma pack(4)`, 36 bytes, packet ID `0x3100002c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoSkillUse0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iBulletID` at offset 4.
    pub bullet_id: i8,
    /// `iSkillID` at offset 6.
    pub skill_id: i16,
    /// `iArg1` at offset 8.
    pub arg1: i32,
    /// `iArg2` at offset 12.
    pub arg2: i32,
    /// `iArg3` at offset 16.
    pub arg3: i32,
    /// `bNanoDeactive` at offset 20.
    pub nano_deactive: i32,
    /// `iNanoID` at offset 24.
    pub nano_id: i16,
    /// `iNanoStamina` at offset 26.
    pub nano_stamina: i16,
    /// `eST` at offset 28.
    pub e_st: i32,
    /// `iTargetCnt` at offset 32.
    pub target_cnt: i32,
}

impl NanoSkillUse0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.bullet_id.to_le_bytes());
        write_prim(out, 6, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.arg1.to_le_bytes());
        write_prim(out, 12, &self.arg2.to_le_bytes());
        write_prim(out, 16, &self.arg3.to_le_bytes());
        write_prim(out, 20, &self.nano_deactive.to_le_bytes());
        write_prim(out, 24, &self.nano_id.to_le_bytes());
        write_prim(out, 26, &self.nano_stamina.to_le_bytes());
        write_prim(out, 28, &self.e_st.to_le_bytes());
        write_prim(out, 32, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 4)),
            skill_id: i16::from_le_bytes(read_array(bytes, 6)),
            arg1: i32::from_le_bytes(read_array(bytes, 8)),
            arg2: i32::from_le_bytes(read_array(bytes, 12)),
            arg3: i32::from_le_bytes(read_array(bytes, 16)),
            nano_deactive: i32::from_le_bytes(read_array(bytes, 20)),
            nano_id: i16::from_le_bytes(read_array(bytes, 24)),
            nano_stamina: i16::from_le_bytes(read_array(bytes, 26)),
            e_st: i32::from_le_bytes(read_array(bytes, 28)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for NanoSkillUse0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_STOP_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100002d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskStopSuccess0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
}

impl PcTaskStopSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcTaskStopSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_STOP_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100002e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskStopFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcTaskStopFailure0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcTaskStopFailure0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_CONTINUE_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x3100002f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskContinueSuccess0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
}

impl PcTaskContinueSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.task_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            task_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcTaskContinueSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_TASK_CONTINUE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000030`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskContinueFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcTaskContinueFailure0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcTaskContinueFailure0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}

/// `sP_FE2CL_REP_PC_GOTO_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000031`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGotoSuccess0104 {
    /// `iX` at offset 0.
    pub x: i32,
    /// `iY` at offset 4.
    pub y: i32,
    /// `iZ` at offset 8.
    pub z: i32,
}

impl PcGotoSuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.x.to_le_bytes());
        write_prim(out, 4, &self.y.to_le_bytes());
        write_prim(out, 8, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            x: i32::from_le_bytes(read_array(bytes, 0)),
            y: i32::from_le_bytes(read_array(bytes, 4)),
            z: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcGotoSuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}
