// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_PC_SLOPE` (`#pragma pack(4)`, 48 bytes, packet ID `0x13000041`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSlopeRequest0104 {
    /// `iCliTime` at offset 0.
    pub cli_time: u64,
    /// `iX` at offset 8.
    pub x: i32,
    /// `iY` at offset 12.
    pub y: i32,
    /// `iZ` at offset 16.
    pub z: i32,
    /// `iAngle` at offset 20.
    pub angle: i32,
    /// `iSpeed` at offset 24.
    pub speed: i32,
    /// `cKeyValue` at offset 28.
    pub c_key_value: u8,
    /// `fVX` at offset 32.
    pub vx: f32,
    /// `fVY` at offset 36.
    pub vy: f32,
    /// `fVZ` at offset 40.
    pub vz: f32,
    /// `iSlopeID` at offset 44.
    pub slope_id: i32,
}

impl PcSlopeRequest0104 {
    pub const SIZE: usize = 48;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cli_time.to_le_bytes());
        write_prim(out, 8, &self.x.to_le_bytes());
        write_prim(out, 12, &self.y.to_le_bytes());
        write_prim(out, 16, &self.z.to_le_bytes());
        write_prim(out, 20, &self.angle.to_le_bytes());
        write_prim(out, 24, &self.speed.to_le_bytes());
        write_prim(out, 28, &self.c_key_value.to_le_bytes());
        write_prim(out, 32, &self.vx.to_le_bytes());
        write_prim(out, 36, &self.vy.to_le_bytes());
        write_prim(out, 40, &self.vz.to_le_bytes());
        write_prim(out, 44, &self.slope_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cli_time: u64::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 8)),
            y: i32::from_le_bytes(read_array(bytes, 12)),
            z: i32::from_le_bytes(read_array(bytes, 16)),
            angle: i32::from_le_bytes(read_array(bytes, 20)),
            speed: i32::from_le_bytes(read_array(bytes, 24)),
            c_key_value: u8::from_le_bytes(read_array(bytes, 28)),
            vx: f32::from_le_bytes(read_array(bytes, 32)),
            vy: f32::from_le_bytes(read_array(bytes, 36)),
            vz: f32::from_le_bytes(read_array(bytes, 40)),
            slope_id: i32::from_le_bytes(read_array(bytes, 44)),
        }
    }
}

impl WirePayload for PcSlopeRequest0104 {
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

/// `sP_CL2FE_REQ_PC_STATE_CHANGE` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000042`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcStateChangeRequest0104 {
    /// `iState` at offset 0.
    pub state: i32,
}

impl PcStateChangeRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            state: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcStateChangeRequest0104 {
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

/// `sP_CL2FE_REQ_PC_MAP_WARP` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000043`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMapWarpRequest0104 {
    /// `iMapNum` at offset 0.
    pub map_num: i32,
}

impl PcMapWarpRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.map_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            map_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcMapWarpRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GIVE_NANO` (`#pragma pack(2)`, 2 bytes, packet ID `0x13000044`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGiveNanoRequest0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
}

impl PcGiveNanoRequest0104 {
    pub const SIZE: usize = 2;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGiveNanoRequest0104 {
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

/// `sP_CL2FE_REQ_NPC_SUMMON` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000045`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcSummonRequest0104 {
    /// `iNPCType` at offset 0.
    pub npc_type: i32,
    /// `iNPCCnt` at offset 4.
    pub npc_cnt: i16,
}

impl NpcSummonRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_type.to_le_bytes());
        write_prim(out, 4, &self.npc_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_type: i32::from_le_bytes(read_array(bytes, 0)),
            npc_cnt: i16::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for NpcSummonRequest0104 {
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

/// `sP_CL2FE_REQ_NPC_UNSUMMON` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000046`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcUnsummonRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
}

impl NpcUnsummonRequest0104 {
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

impl WirePayload for NpcUnsummonRequest0104 {
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

/// `sP_CL2FE_REQ_ITEM_CHEST_OPEN` (`#pragma pack(4)`, 20 bytes, packet ID `0x13000047`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemChestOpenRequest0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
    /// `ChestItem` at offset 8.
    pub chest_item: ItemBase0104,
}

impl ItemChestOpenRequest0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
        self.chest_item.write_into(&mut out[8..20]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
            chest_item: ItemBase0104::read_from(&bytes[8..20]),
        }
    }
}

impl WirePayload for ItemChestOpenRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GIVE_NANO_SKILL` (`#pragma pack(2)`, 4 bytes, packet ID `0x13000048`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGiveNanoSkillRequest0104 {
    /// `iNanoID` at offset 0.
    pub nano_id: i16,
    /// `iNanoSkillID` at offset 2.
    pub nano_skill_id: i16,
}

impl PcGiveNanoSkillRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.nano_id.to_le_bytes());
        write_prim(out, 2, &self.nano_skill_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            nano_id: i16::from_le_bytes(read_array(bytes, 0)),
            nano_skill_id: i16::from_le_bytes(read_array(bytes, 2)),
        }
    }
}

impl WirePayload for PcGiveNanoSkillRequest0104 {
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

/// `sP_CL2FE_DOT_DAMAGE_ONOFF` (`#pragma pack(4)`, 4 bytes, packet ID `0x13000049`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct DotDamageOnoff0104 {
    /// `iFlag` at offset 0.
    pub flag: i32,
}

impl DotDamageOnoff0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            flag: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for DotDamageOnoff0104 {
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

/// `sP_CL2FE_REQ_PC_VENDOR_BATTERY_BUY` (`#pragma pack(4)`, 24 bytes, packet ID `0x1300004a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorBatteryBuyRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iVendorID` at offset 4.
    pub vendor_id: i32,
    /// `iListID` at offset 8.
    pub list_id: i8,
    /// `Item` at offset 12.
    pub item: ItemBase0104,
}

impl PcVendorBatteryBuyRequest0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.vendor_id.to_le_bytes());
        write_prim(out, 8, &self.list_id.to_le_bytes());
        self.item.write_into(&mut out[12..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            vendor_id: i32::from_le_bytes(read_array(bytes, 4)),
            list_id: i8::from_le_bytes(read_array(bytes, 8)),
            item: ItemBase0104::read_from(&bytes[12..24]),
        }
    }
}

impl WirePayload for PcVendorBatteryBuyRequest0104 {
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

/// `sP_CL2FE_REQ_PC_WARP_USE_NPC` (`#pragma pack(4)`, 24 bytes, packet ID `0x1300004b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpUseNpcRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iWarpID` at offset 4.
    pub warp_id: i32,
    /// `eIL1` at offset 8.
    pub e_il1: i32,
    /// `iItemSlot1` at offset 12.
    pub item_slot1: i32,
    /// `eIL2` at offset 16.
    pub e_il2: i32,
    /// `iItemSlot2` at offset 20.
    pub item_slot2: i32,
}

impl PcWarpUseNpcRequest0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.warp_id.to_le_bytes());
        write_prim(out, 8, &self.e_il1.to_le_bytes());
        write_prim(out, 12, &self.item_slot1.to_le_bytes());
        write_prim(out, 16, &self.e_il2.to_le_bytes());
        write_prim(out, 20, &self.item_slot2.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            warp_id: i32::from_le_bytes(read_array(bytes, 4)),
            e_il1: i32::from_le_bytes(read_array(bytes, 8)),
            item_slot1: i32::from_le_bytes(read_array(bytes, 12)),
            e_il2: i32::from_le_bytes(read_array(bytes, 16)),
            item_slot2: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for PcWarpUseNpcRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GROUP_INVITE` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300004c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupInviteRequest0104 {
    /// `iID_To` at offset 0.
    pub id_to: i32,
}

impl PcGroupInviteRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_to: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGroupInviteRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GROUP_INVITE_REFUSE` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300004d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupInviteRefuseRequest0104 {
    /// `iID_From` at offset 0.
    pub id_from: i32,
}

impl PcGroupInviteRefuseRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_from.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_from: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGroupInviteRefuseRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GROUP_JOIN` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300004e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupJoinRequest0104 {
    /// `iID_From` at offset 0.
    pub id_from: i32,
}

impl PcGroupJoinRequest0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_from.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_from: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcGroupJoinRequest0104 {
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

/// `sP_CL2FE_REQ_PC_GROUP_LEAVE` (`#pragma pack(8)`, 1 bytes, packet ID `0x1300004f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGroupLeaveRequest0104;

impl PcGroupLeaveRequest0104 {
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

impl WirePayload for PcGroupLeaveRequest0104 {
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

/// `sP_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000050`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcAvatarEmotesChatRequest0104 {
    /// `iID_From` at offset 0.
    pub id_from: i32,
    /// `iEmoteCode` at offset 4.
    pub emote_code: i32,
}

impl PcAvatarEmotesChatRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_from.to_le_bytes());
        write_prim(out, 4, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_from: i32::from_le_bytes(read_array(bytes, 0)),
            emote_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcAvatarEmotesChatRequest0104 {
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

/// `sP_CL2FE_REQ_PC_BUDDY_WARP` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000051`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuddyWarpRequest0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
}

impl PcBuddyWarpRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcBuddyWarpRequest0104 {
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

/// `sP_CL2FE_REQ_GET_MEMBER_STYLE` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000052`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetMemberStyleRequest0104 {
    /// `iMemberID` at offset 0.
    pub member_id: i32,
    /// `iMemberUID` at offset 4.
    pub member_uid: i64,
}

impl GetMemberStyleRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.member_id.to_le_bytes());
        write_prim(out, 4, &self.member_uid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            member_id: i32::from_le_bytes(read_array(bytes, 0)),
            member_uid: i64::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for GetMemberStyleRequest0104 {
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
