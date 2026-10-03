// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_NANO_SKILL_USE` (`#pragma pack(4)`, 20 bytes, packet ID `0x13000011`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NanoSkillUseRequest0104 {
    /// `iBulletID` at offset 0.
    pub bullet_id: i8,
    /// `iArg1` at offset 4.
    pub arg1: i32,
    /// `iArg2` at offset 8.
    pub arg2: i32,
    /// `iArg3` at offset 12.
    pub arg3: i32,
    /// `iTargetCnt` at offset 16.
    pub target_cnt: i32,
}

impl NanoSkillUseRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.bullet_id.to_le_bytes());
        write_prim(out, 4, &self.arg1.to_le_bytes());
        write_prim(out, 8, &self.arg2.to_le_bytes());
        write_prim(out, 12, &self.arg3.to_le_bytes());
        write_prim(out, 16, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            bullet_id: i8::from_le_bytes(read_array(bytes, 0)),
            arg1: i32::from_le_bytes(read_array(bytes, 4)),
            arg2: i32::from_le_bytes(read_array(bytes, 8)),
            arg3: i32::from_le_bytes(read_array(bytes, 12)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for NanoSkillUseRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TASK_STOP` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000012`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskStopRequest0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
}

impl PcTaskStopRequest0104 {
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

impl WirePayload for PcTaskStopRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TASK_CONTINUE` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000013`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTaskContinueRequest0104 {
    /// `iTaskNum` at offset 0.
    pub task_num: i32,
}

impl PcTaskContinueRequest0104 {
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

impl WirePayload for PcTaskContinueRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GOTO` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000014`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGotoRequest0104 {
    /// `iToX` at offset 0.
    pub to_x: i32,
    /// `iToY` at offset 4.
    pub to_y: i32,
    /// `iToZ` at offset 8.
    pub to_z: i32,
}

impl PcGotoRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.to_x.to_le_bytes());
        write_prim(out, 4, &self.to_y.to_le_bytes());
        write_prim(out, 8, &self.to_z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            to_x: i32::from_le_bytes(read_array(bytes, 0)),
            to_y: i32::from_le_bytes(read_array(bytes, 4)),
            to_z: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcGotoRequest0104 {
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

/// `sP_CL2FE_REQ_CHARGE_NANO_STAMINA` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000015`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ChargeNanoStaminaRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl ChargeNanoStaminaRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for ChargeNanoStaminaRequest0104 {
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

/// `sP_CL2FE_REQ_PC_KILL_QUEST_NPCs` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000016`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcKillQuestNpcsRequest0104 {
    /// `iNPCCnt` at offset 0.
    pub npc_cnt: i32,
}

impl PcKillQuestNpcsRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcKillQuestNpcsRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VENDOR_ITEM_BUY` (`#pragma pack(4)`, 28 bytes, packet ID `0x13000017`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemBuyRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iVendorID` at offset 4.
    pub vendor_id: i32,
    /// `iListID` at offset 8.
    pub list_id: i8,
    /// `Item` at offset 12.
    pub item: ItemBase0104,
    /// `iInvenSlotNum` at offset 24.
    pub inven_slot_num: i32,
}

impl PcVendorItemBuyRequest0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.vendor_id.to_le_bytes());
        write_prim(out, 8, &self.list_id.to_le_bytes());
        self.item.write_into(&mut out[12..24]);
        write_prim(out, 24, &self.inven_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            vendor_id: i32::from_le_bytes(read_array(bytes, 4)),
            list_id: i8::from_le_bytes(read_array(bytes, 8)),
            item: ItemBase0104::read_from(&bytes[12..24]),
            inven_slot_num: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for PcVendorItemBuyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VENDOR_ITEM_SELL` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000018`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemSellRequest0104 {
    /// `iInvenSlotNum` at offset 0.
    pub inven_slot_num: i32,
    /// `iItemCnt` at offset 4.
    pub item_cnt: i32,
}

impl PcVendorItemSellRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.inven_slot_num.to_le_bytes());
        write_prim(out, 4, &self.item_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            inven_slot_num: i32::from_le_bytes(read_array(bytes, 0)),
            item_cnt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcVendorItemSellRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ITEM_DELETE` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000019`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemDeleteRequest0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
}

impl PcItemDeleteRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcItemDeleteRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GIVE_ITEM` (`#pragma pack(4)`, 24 bytes, packet ID `0x1300001a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGiveItemRequest0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
    /// `Item` at offset 8.
    pub item: ItemBase0104,
    /// `iTimeLeft` at offset 20.
    pub time_left: i32,
}

impl PcGiveItemRequest0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
        self.item.write_into(&mut out[8..20]);
        write_prim(out, 20, &self.time_left.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            item: ItemBase0104::read_from(&bytes[8..20]),
            time_left: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for PcGiveItemRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ROCKET_STYLE_READY` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300001b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleReadyRequest0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i32,
}

impl PcRocketStyleReadyRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcRocketStyleReadyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ROCKET_STYLE_FIRE` (`#pragma pack(4)`, 28 bytes, packet ID `0x1300001c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleFireRequest0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iToX` at offset 16.
    pub to_x: i32,
    /// `iToY` at offset 20.
    pub to_y: i32,
    /// `iToZ` at offset 24.
    pub to_z: i32,
}

impl PcRocketStyleFireRequest0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.to_x.to_le_bytes());
        write_prim(out, 20, &self.to_y.to_le_bytes());
        write_prim(out, 24, &self.to_z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            to_x: i32::from_le_bytes(read_array(bytes, 16)),
            to_y: i32::from_le_bytes(read_array(bytes, 20)),
            to_z: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for PcRocketStyleFireRequest0104 {
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

/// `sP_CL2FE_REQ_PC_ROCKET_STYLE_HIT` (`#pragma pack(4)`, 20 bytes, packet ID `0x1300001d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRocketStyleHitRequest0104 {
    /// `iBulletID` at offset 0.
    pub bullet_id: i8,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iTargetCnt` at offset 16.
    pub target_cnt: i32,
}

impl PcRocketStyleHitRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.bullet_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            bullet_id: i8::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcRocketStyleHitRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GRENADE_STYLE_READY` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300001e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleReadyRequest0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i32,
}

impl PcGrenadeStyleReadyRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGrenadeStyleReadyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GRENADE_STYLE_FIRE` (`#pragma pack(4)`, 16 bytes, packet ID `0x1300001f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleFireRequest0104 {
    /// `iSkillID` at offset 0.
    pub skill_id: i32,
    /// `iToX` at offset 4.
    pub to_x: i32,
    /// `iToY` at offset 8.
    pub to_y: i32,
    /// `iToZ` at offset 12.
    pub to_z: i32,
}

impl PcGrenadeStyleFireRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.skill_id.to_le_bytes());
        write_prim(out, 4, &self.to_x.to_le_bytes());
        write_prim(out, 8, &self.to_y.to_le_bytes());
        write_prim(out, 12, &self.to_z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            skill_id: i32::from_le_bytes(read_array(bytes, 0)),
            to_x: i32::from_le_bytes(read_array(bytes, 4)),
            to_y: i32::from_le_bytes(read_array(bytes, 8)),
            to_z: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcGrenadeStyleFireRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GRENADE_STYLE_HIT` (`#pragma pack(4)`, 20 bytes, packet ID `0x13000020`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleHitRequest0104 {
    /// `iBulletID` at offset 0.
    pub bullet_id: i8,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iTargetCnt` at offset 16.
    pub target_cnt: i32,
}

impl PcGrenadeStyleHitRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.bullet_id.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            bullet_id: i8::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcGrenadeStyleHitRequest0104 {
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

/// `sP_CL2FE_REQ_PC_NANO_CREATE` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000021`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNanoCreateRequest0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
    /// `iNeedQuestItemSlotNum` at offset 4.
    pub need_quest_item_slot_num: i32,
}

impl PcNanoCreateRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
        write_prim(out, 4, &self.need_quest_item_slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
            need_quest_item_slot_num: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcNanoCreateRequest0104 {
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
