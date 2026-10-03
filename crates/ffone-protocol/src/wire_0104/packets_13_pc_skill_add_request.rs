// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_PC_SKILL_ADD` (`#pragma pack(4)`, 20 bytes, packet ID `0x1300009a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillAddRequest0104 {
    /// `iSkillSlotNum` at offset 0.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
    /// `iSkillItemInvenSlotNum` at offset 8.
    pub skill_item_inven_slot_num: i32,
    /// `iPreSkillSlotNum` at offset 12.
    pub pre_skill_slot_num: i32,
    /// `iPreSkillID` at offset 16.
    pub pre_skill_id: i32,
}

impl PcSkillAddRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.skill_item_inven_slot_num.to_le_bytes());
        write_prim(out, 12, &self.pre_skill_slot_num.to_le_bytes());
        write_prim(out, 16, &self.pre_skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
            skill_item_inven_slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            pre_skill_slot_num: i32::from_le_bytes(read_array(bytes, 12)),
            pre_skill_id: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcSkillAddRequest0104 {
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

/// `sP_CL2FE_REQ_PC_SKILL_DEL` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300009b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillDelRequest0104 {
    /// `iSkillSlotNum` at offset 0.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
}

impl PcSkillDelRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcSkillDelRequest0104 {
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

/// `sP_CL2FE_REQ_PC_SKILL_USE` (`#pragma pack(4)`, 60 bytes, packet ID `0x1300009c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSkillUseRequest0104 {
    /// `iSkillSlotNum` at offset 0.
    pub skill_slot_num: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i32,
    /// `iMoveFlag` at offset 8.
    pub move_flag: i32,
    /// `iFromX` at offset 12.
    pub from_x: i32,
    /// `iFromY` at offset 16.
    pub from_y: i32,
    /// `iFromZ` at offset 20.
    pub from_z: i32,
    /// `iToX` at offset 24.
    pub to_x: i32,
    /// `iToY` at offset 28.
    pub to_y: i32,
    /// `iToZ` at offset 32.
    pub to_z: i32,
    /// `iMainTargetType` at offset 36.
    pub main_target_type: i32,
    /// `iMainTargetID` at offset 40.
    pub main_target_id: i32,
    /// `iTargetLocationX` at offset 44.
    pub target_location_x: i32,
    /// `iTargetLocationY` at offset 48.
    pub target_location_y: i32,
    /// `iTargetLocationZ` at offset 52.
    pub target_location_z: i32,
    /// `iTargetCount` at offset 56.
    pub target_count: i32,
}

impl PcSkillUseRequest0104 {
    pub const SIZE: usize = 60;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_slot_num.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.move_flag.to_le_bytes());
        write_prim(out, 12, &self.from_x.to_le_bytes());
        write_prim(out, 16, &self.from_y.to_le_bytes());
        write_prim(out, 20, &self.from_z.to_le_bytes());
        write_prim(out, 24, &self.to_x.to_le_bytes());
        write_prim(out, 28, &self.to_y.to_le_bytes());
        write_prim(out, 32, &self.to_z.to_le_bytes());
        write_prim(out, 36, &self.main_target_type.to_le_bytes());
        write_prim(out, 40, &self.main_target_id.to_le_bytes());
        write_prim(out, 44, &self.target_location_x.to_le_bytes());
        write_prim(out, 48, &self.target_location_y.to_le_bytes());
        write_prim(out, 52, &self.target_location_z.to_le_bytes());
        write_prim(out, 56, &self.target_count.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i32::from_le_bytes(read_array(bytes, 4)),
            move_flag: i32::from_le_bytes(read_array(bytes, 8)),
            from_x: i32::from_le_bytes(read_array(bytes, 12)),
            from_y: i32::from_le_bytes(read_array(bytes, 16)),
            from_z: i32::from_le_bytes(read_array(bytes, 20)),
            to_x: i32::from_le_bytes(read_array(bytes, 24)),
            to_y: i32::from_le_bytes(read_array(bytes, 28)),
            to_z: i32::from_le_bytes(read_array(bytes, 32)),
            main_target_type: i32::from_le_bytes(read_array(bytes, 36)),
            main_target_id: i32::from_le_bytes(read_array(bytes, 40)),
            target_location_x: i32::from_le_bytes(read_array(bytes, 44)),
            target_location_y: i32::from_le_bytes(read_array(bytes, 48)),
            target_location_z: i32::from_le_bytes(read_array(bytes, 52)),
            target_count: i32::from_le_bytes(read_array(bytes, 56)),
        }
    }
}

impl WirePayload for PcSkillUseRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ROPE` (`#pragma pack(4)`, 48 bytes, packet ID `0x1300009d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRopeRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `fVX` at offset 20.
    pub vx: f32,
    /// `fVY` at offset 24.
    pub vy: f32,
    /// `fVZ` at offset 28.
    pub vz: f32,
    /// `iRopeID` at offset 32.
    pub rope_id: i32,
    /// `iAngle` at offset 36.
    pub angle: i32,
    /// `cKeyValue` at offset 40.
    pub c_key_value: u8,
    /// `iSpeed` at offset 44.
    pub speed: i32,
}

impl PcRopeRequest0104 {
    pub const SIZE: usize = 48;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.vx.to_le_bytes());
        write_prim(out, 24, &self.vy.to_le_bytes());
        write_prim(out, 28, &self.vz.to_le_bytes());
        write_prim(out, 32, &self.rope_id.to_le_bytes());
        write_prim(out, 36, &self.angle.to_le_bytes());
        write_prim(out, 40, &self.c_key_value.to_le_bytes());
        write_prim(out, 44, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            vx: f32::from_le_bytes(read_array(bytes, 20)),
            vy: f32::from_le_bytes(read_array(bytes, 24)),
            vz: f32::from_le_bytes(read_array(bytes, 28)),
            rope_id: i32::from_le_bytes(read_array(bytes, 32)),
            angle: i32::from_le_bytes(read_array(bytes, 36)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 40)),
            speed: i32::from_le_bytes(read_array(bytes, 44)),
        }
    }
}

impl WirePayload for PcRopeRequest0104 {
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

/// `sP_CL2FE_REQ_PC_BELT` (`#pragma pack(4)`, 64 bytes, packet ID `0x1300009e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBeltRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iLcX` at offset 8.
    pub lc_x: i32,
    /// `iLcY` at offset 12.
    pub lc_y: i32,
    /// `iLcZ` at offset 16.
    pub lc_z: i32,
    /// `iX` at offset 20.
    pub x: i32,
    /// `iY` at offset 24.
    pub y: i32,
    /// `iZ` at offset 28.
    pub z: i32,
    /// `fVX` at offset 32.
    pub vx: f32,
    /// `fVY` at offset 36.
    pub vy: f32,
    /// `fVZ` at offset 40.
    pub vz: f32,
    /// `bDown` at offset 44.
    pub down: i32,
    /// `iBeltID` at offset 48.
    pub belt_id: i32,
    /// `iAngle` at offset 52.
    pub angle: i32,
    /// `cKeyValue` at offset 56.
    pub c_key_value: u8,
    /// `iSpeed` at offset 60.
    pub speed: i32,
}

impl PcBeltRequest0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.lc_x.to_le_bytes());
        write_prim(out, 12, &self.lc_y.to_le_bytes());
        write_prim(out, 16, &self.lc_z.to_le_bytes());
        write_prim(out, 20, &self.x.to_le_bytes());
        write_prim(out, 24, &self.y.to_le_bytes());
        write_prim(out, 28, &self.z.to_le_bytes());
        write_prim(out, 32, &self.vx.to_le_bytes());
        write_prim(out, 36, &self.vy.to_le_bytes());
        write_prim(out, 40, &self.vz.to_le_bytes());
        write_prim(out, 44, &self.down.to_le_bytes());
        write_prim(out, 48, &self.belt_id.to_le_bytes());
        write_prim(out, 52, &self.angle.to_le_bytes());
        write_prim(out, 56, &self.c_key_value.to_le_bytes());
        write_prim(out, 60, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            lc_x: i32::from_le_bytes(read_array(bytes, 8)),
            lc_y: i32::from_le_bytes(read_array(bytes, 12)),
            lc_z: i32::from_le_bytes(read_array(bytes, 16)),
            x: i32::from_le_bytes(read_array(bytes, 20)),
            y: i32::from_le_bytes(read_array(bytes, 24)),
            z: i32::from_le_bytes(read_array(bytes, 28)),
            vx: f32::from_le_bytes(read_array(bytes, 32)),
            vy: f32::from_le_bytes(read_array(bytes, 36)),
            vz: f32::from_le_bytes(read_array(bytes, 40)),
            down: i32::from_le_bytes(read_array(bytes, 44)),
            belt_id: i32::from_le_bytes(read_array(bytes, 48)),
            angle: i32::from_le_bytes(read_array(bytes, 52)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 56)),
            speed: i32::from_le_bytes(read_array(bytes, 60)),
        }
    }
}

impl WirePayload for PcBeltRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VEHICLE_ON` (`#pragma pack(8)`, 1 bytes, packet ID `0x1300009f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVehicleOnRequest0104;

impl PcVehicleOnRequest0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        out[0] = 0;
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self
    }
}

impl WirePayload for PcVehicleOnRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VEHICLE_OFF` (`#pragma pack(8)`, 1 bytes, packet ID `0x130000a0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVehicleOffRequest0104;

impl PcVehicleOffRequest0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        out[0] = 0;
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self
    }
}

impl WirePayload for PcVehicleOffRequest0104 {
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

/// `sP_CL2FE_REQ_PC_REGIST_QUICK_SLOT` (`#pragma pack(4)`, 8 bytes, packet ID `0x130000a1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegistQuickSlotRequest0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i32,
    /// `iItemType` at offset 4.
    pub item_type: i16,
    /// `iItemID` at offset 6.
    pub item_id: i16,
}

impl PcRegistQuickSlotRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
        write_prim(out, 4, &self.item_type.to_le_bytes());
        write_prim(out, 6, &self.item_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_type: i16::from_le_bytes(read_array(bytes, 4)),
            item_id: i16::from_le_bytes(read_array(bytes, 6)),
        }
    }
}

impl WirePayload for PcRegistQuickSlotRequest0104 {
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

/// `sP_CL2FE_REQ_PC_DISASSEMBLE_ITEM` (`#pragma pack(4)`, 4 bytes, packet ID `0x130000a2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDisassembleItemRequest0104 {
    /// `iItemSlot` at offset 0.
    pub item_slot: i32,
}

impl PcDisassembleItemRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.item_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            item_slot: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcDisassembleItemRequest0104 {
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

/// `sP_CL2FE_GM_REQ_REWARD_RATE` (`#pragma pack(4)`, 16 bytes, packet ID `0x130000a3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmRewardRateRequest0104 {
    /// `iGetSet` at offset 0.
    pub get_set: i32,
    /// `iRewardType` at offset 4.
    pub reward_type: i32,
    /// `iRewardRateIndex` at offset 8.
    pub reward_rate_index: i32,
    /// `iSetRateValue` at offset 12.
    pub set_rate_value: i32,
}

impl GmRewardRateRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.get_set.to_le_bytes());
        write_prim(out, 4, &self.reward_type.to_le_bytes());
        write_prim(out, 8, &self.reward_rate_index.to_le_bytes());
        write_prim(out, 12, &self.set_rate_value.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            get_set: i32::from_le_bytes(read_array(bytes, 0)),
            reward_type: i32::from_le_bytes(read_array(bytes, 4)),
            reward_rate_index: i32::from_le_bytes(read_array(bytes, 8)),
            set_rate_value: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for GmRewardRateRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ITEM_ENCHANT` (`#pragma pack(4)`, 20 bytes, packet ID `0x130000a4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemEnchantRequest0104 {
    /// `iEnchantItemSlot` at offset 0.
    pub enchant_item_slot: i32,
    /// `iWeaponMaterialItemSlot` at offset 4.
    pub weapon_material_item_slot: i32,
    /// `iDefenceMaterialItemSlot` at offset 8.
    pub defence_material_item_slot: i32,
    /// `iCashItemSlot1` at offset 12.
    pub cash_item_slot1: i32,
    /// `iCashItemSlot2` at offset 16.
    pub cash_item_slot2: i32,
}

impl PcItemEnchantRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.enchant_item_slot.to_le_bytes());
        write_prim(out, 4, &self.weapon_material_item_slot.to_le_bytes());
        write_prim(out, 8, &self.defence_material_item_slot.to_le_bytes());
        write_prim(out, 12, &self.cash_item_slot1.to_le_bytes());
        write_prim(out, 16, &self.cash_item_slot2.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            enchant_item_slot: i32::from_le_bytes(read_array(bytes, 0)),
            weapon_material_item_slot: i32::from_le_bytes(read_array(bytes, 4)),
            defence_material_item_slot: i32::from_le_bytes(read_array(bytes, 8)),
            cash_item_slot1: i32::from_le_bytes(read_array(bytes, 12)),
            cash_item_slot2: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcItemEnchantRequest0104 {
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

/// `sP_CL2FE_REQ_PC_BARBER_OPEN` (`#pragma pack(4)`, 4 bytes, packet ID `0x130000a5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBarberOpenRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
}

impl PcBarberOpenRequest0104 {
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

impl WirePayload for PcBarberOpenRequest0104 {
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

/// `sP_CL2FE_REQ_PC_BARBER_CONFIRM` (`#pragma pack(4)`, 76 bytes, packet ID `0x130000a6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBarberConfirmRequest0104 {
    /// `sPCStyle` at offset 0.
    pub s_pc_style: PcStyle0104,
}

impl PcBarberConfirmRequest0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.s_pc_style.write_into(&mut out[0..76]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            s_pc_style: PcStyle0104::read_from(&bytes[0..76]),
        }
    }
}

impl WirePayload for PcBarberConfirmRequest0104 {
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

/// `sP_CL2FE_REQ_PRESENT_NPC_TYPES` (`#pragma pack(4)`, 8 bytes, packet ID `0x130000a7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PresentNpcTypesRequest0104 {
    /// `uiLastSyncTime` at offset 0.
    pub last_sync_time: u64,
}

impl PresentNpcTypesRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.last_sync_time.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            last_sync_time: u64::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PresentNpcTypesRequest0104 {
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
