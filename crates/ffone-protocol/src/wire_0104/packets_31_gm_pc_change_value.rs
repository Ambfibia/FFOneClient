// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_GM_PC_CHANGE_VALUE` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000c6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcChangeValue0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSetValueType` at offset 4.
    pub set_value_type: i32,
    /// `iSetValue` at offset 8.
    pub set_value: i32,
}

impl GmPcChangeValue0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.set_value_type.to_le_bytes());
        write_prim(out, 8, &self.set_value.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            set_value_type: i32::from_le_bytes(read_array(bytes, 4)),
            set_value: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for GmPcChangeValue0104 {
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

/// `sP_FE2CL_GM_REP_PC_LOCATION` (`#pragma pack(4)`, 96 bytes, packet ID `0x310000c7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcLocationReply0104 {
    /// `iTargetPC_UID` at offset 0.
    pub target_pc_uid: i64,
    /// `iTargetPC_ID` at offset 8.
    pub target_pc_id: i32,
    /// `iShardID` at offset 12.
    pub shard_id: i32,
    /// `iMapType` at offset 16.
    pub map_type: i32,
    /// `iMapID` at offset 20.
    pub map_id: i32,
    /// `iMapNum` at offset 24.
    pub map_num: i32,
    /// `iX` at offset 28.
    pub x: i32,
    /// `iY` at offset 32.
    pub y: i32,
    /// `iZ` at offset 36.
    pub z: i32,
    /// `szTargetPC_FirstName` at offset 40.
    pub target_pc_first_name: FixedUtf16<10>,
    /// `szTargetPC_LastName` at offset 60.
    pub target_pc_last_name: FixedUtf16<18>,
}

impl GmPcLocationReply0104 {
    pub const SIZE: usize = 96;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.target_pc_uid.to_le_bytes());
        write_prim(out, 8, &self.target_pc_id.to_le_bytes());
        write_prim(out, 12, &self.shard_id.to_le_bytes());
        write_prim(out, 16, &self.map_type.to_le_bytes());
        write_prim(out, 20, &self.map_id.to_le_bytes());
        write_prim(out, 24, &self.map_num.to_le_bytes());
        write_prim(out, 28, &self.x.to_le_bytes());
        write_prim(out, 32, &self.y.to_le_bytes());
        write_prim(out, 36, &self.z.to_le_bytes());
        write_utf16(out, 40, &self.target_pc_first_name);
        write_utf16(out, 60, &self.target_pc_last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            target_pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
            target_pc_id: i32::from_le_bytes(read_array(bytes, 8)),
            shard_id: i32::from_le_bytes(read_array(bytes, 12)),
            map_type: i32::from_le_bytes(read_array(bytes, 16)),
            map_id: i32::from_le_bytes(read_array(bytes, 20)),
            map_num: i32::from_le_bytes(read_array(bytes, 24)),
            x: i32::from_le_bytes(read_array(bytes, 28)),
            y: i32::from_le_bytes(read_array(bytes, 32)),
            z: i32::from_le_bytes(read_array(bytes, 36)),
            target_pc_first_name: read_utf16(bytes, 40),
            target_pc_last_name: read_utf16(bytes, 60),
        }
    }
}

impl WirePayload for GmPcLocationReply0104 {
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

/// `sP_FE2CL_GM_REP_PC_ANNOUNCE` (`#pragma pack(4)`, 1032 bytes, packet ID `0x310000c8`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcAnnounceReply0104 {
    /// `iAnnounceType` at offset 0.
    pub announce_type: i8,
    /// `iDuringTime` at offset 4.
    pub during_time: i32,
    /// `szAnnounceMsg` at offset 8.
    pub announce_msg: FixedUtf16<512>,
}

impl GmPcAnnounceReply0104 {
    pub const SIZE: usize = 1032;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.announce_type.to_le_bytes());
        write_prim(out, 4, &self.during_time.to_le_bytes());
        write_utf16(out, 8, &self.announce_msg);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            announce_type: i8::from_le_bytes(read_array(bytes, 0)),
            during_time: i32::from_le_bytes(read_array(bytes, 4)),
            announce_msg: read_utf16(bytes, 8),
        }
    }
}

impl WirePayload for GmPcAnnounceReply0104 {
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

/// `sP_FE2CL_REP_PC_BUDDY_WARP_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000c9`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuddyWarpFailure0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl PcBuddyWarpFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcBuddyWarpFailure0104 {
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

/// `sP_FE2CL_REP_PC_CHANGE_LEVEL` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000ca`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcChangeLevelReply0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iPC_Level` at offset 4.
    pub pc_level: i16,
}

impl PcChangeLevelReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.pc_level.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            pc_level: i16::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcChangeLevelReply0104 {
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

/// `sP_FE2CL_REP_SET_PC_BLOCK_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000cb`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SetPcBlockSuccess0104 {
    /// `iBlock_ID` at offset 0.
    pub block_id: i32,
    /// `iBlock_PCUID` at offset 4.
    pub block_pcuid: i64,
    /// `iBuddySlot` at offset 12.
    pub buddy_slot: i8,
}

impl SetPcBlockSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.block_id.to_le_bytes());
        write_prim(out, 4, &self.block_pcuid.to_le_bytes());
        write_prim(out, 12, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            block_id: i32::from_le_bytes(read_array(bytes, 0)),
            block_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for SetPcBlockSuccess0104 {
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

/// `sP_FE2CL_REP_SET_PC_BLOCK_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000cc`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SetPcBlockFailure0104 {
    /// `iBlock_ID` at offset 0.
    pub block_id: i32,
    /// `iBlock_PCUID` at offset 4.
    pub block_pcuid: i64,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl SetPcBlockFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.block_id.to_le_bytes());
        write_prim(out, 4, &self.block_pcuid.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            block_id: i32::from_le_bytes(read_array(bytes, 0)),
            block_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for SetPcBlockFailure0104 {
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

/// `sP_FE2CL_REP_REGIST_RXCOM` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000cd`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistRxcomReply0104 {
    /// `iMapNum` at offset 0.
    pub map_num: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
}

impl RegistRxcomReply0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.map_num.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            map_num: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for RegistRxcomReply0104 {
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

/// `sP_FE2CL_REP_REGIST_RXCOM_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000ce`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistRxcomFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl RegistRxcomFailure0104 {
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

impl WirePayload for RegistRxcomFailure0104 {
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

/// `sP_FE2CL_PC_INVEN_FULL_MSG` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000cf`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcInvenFullMsg0104 {
    /// `iType` at offset 0.
    pub type_: i8,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcInvenFullMsg0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.type_.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            type_: i8::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcInvenFullMsg0104 {
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

/// `sP_FE2CL_REQ_LIVE_CHECK` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000d0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveCheck0104 {
    /// `iTempValue` at offset 0.
    pub temp_value: i32,
}

impl LiveCheck0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.temp_value.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            temp_value: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LiveCheck0104 {
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

/// `sP_FE2CL_PC_MOTD_LOGIN` (`#pragma pack(2)`, 1026 bytes, packet ID `0x310000d1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMotdLogin0104 {
    /// `iType` at offset 0.
    pub type_: i8,
    /// `szSystemMsg` at offset 2.
    pub system_msg: FixedUtf16<512>,
}

impl PcMotdLogin0104 {
    pub const SIZE: usize = 1026;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.type_.to_le_bytes());
        write_utf16(out, 2, &self.system_msg);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            type_: i8::from_le_bytes(read_array(bytes, 0)),
            system_msg: read_utf16(bytes, 2),
        }
    }
}

impl WirePayload for PcMotdLogin0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_USE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000d2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemUseFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcItemUseFailure0104 {
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

impl WirePayload for PcItemUseFailure0104 {
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

/// `sP_FE2CL_REP_PC_ITEM_USE_SUCC` (`#pragma pack(4)`, 36 bytes, packet ID `0x310000d3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemUseSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `eIL` at offset 4.
    pub e_il: i32,
    /// `iSlotNum` at offset 8.
    pub slot_num: i32,
    /// `RemainItem` at offset 12.
    pub remain_item: ItemBase0104,
    /// `iSkillID` at offset 24.
    pub skill_id: i16,
    /// `eST` at offset 28.
    pub e_st: i32,
    /// `iTargetCnt` at offset 32.
    pub target_cnt: i32,
}

impl PcItemUseSuccess0104 {
    pub const SIZE: usize = 36;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.e_il.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        self.remain_item.write_into(&mut out[12..24]);
        write_prim(out, 24, &self.skill_id.to_le_bytes());
        write_prim(out, 28, &self.e_st.to_le_bytes());
        write_prim(out, 32, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            e_il: i32::from_le_bytes(read_array(bytes, 4)),
            slot_num: i32::from_le_bytes(read_array(bytes, 8)),
            remain_item: ItemBase0104::read_from(&bytes[12..24]),
            skill_id: i16::from_le_bytes(read_array(bytes, 24)),
            e_st: i32::from_le_bytes(read_array(bytes, 28)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 32)),
        }
    }
}

impl WirePayload for PcItemUseSuccess0104 {
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

/// `sP_FE2CL_PC_ITEM_USE` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000d4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcItemUse0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSkillID` at offset 4.
    pub skill_id: i16,
    /// `eST` at offset 8.
    pub e_st: i32,
    /// `iTargetCnt` at offset 12.
    pub target_cnt: i32,
}

impl PcItemUse0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.skill_id.to_le_bytes());
        write_prim(out, 8, &self.e_st.to_le_bytes());
        write_prim(out, 12, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            skill_id: i16::from_le_bytes(read_array(bytes, 4)),
            e_st: i32::from_le_bytes(read_array(bytes, 8)),
            target_cnt: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcItemUse0104 {
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

/// `sP_FE2CL_REP_GET_BUDDY_LOCATION_SUCC` (`#pragma pack(4)`, 28 bytes, packet ID `0x310000d5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyLocationSuccess0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
    /// `iX` at offset 12.
    pub x: i32,
    /// `iY` at offset 16.
    pub y: i32,
    /// `iZ` at offset 20.
    pub z: i32,
    /// `iShardNum` at offset 24.
    pub shard_num: i8,
}

impl GetBuddyLocationSuccess0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 12, &self.x.to_le_bytes());
        write_prim(out, 16, &self.y.to_le_bytes());
        write_prim(out, 20, &self.z.to_le_bytes());
        write_prim(out, 24, &self.shard_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
            x: i32::from_le_bytes(read_array(bytes, 12)),
            y: i32::from_le_bytes(read_array(bytes, 16)),
            z: i32::from_le_bytes(read_array(bytes, 20)),
            shard_num: i8::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for GetBuddyLocationSuccess0104 {
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

/// `sP_FE2CL_REP_GET_BUDDY_LOCATION_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000d6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyLocationFailure0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl GetBuddyLocationFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for GetBuddyLocationFailure0104 {
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
