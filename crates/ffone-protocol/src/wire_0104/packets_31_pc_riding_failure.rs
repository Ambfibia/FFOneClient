// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_PC_RIDING_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000d7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRidingFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcRidingFailure0104 {
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

impl WirePayload for PcRidingFailure0104 {
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

/// `sP_FE2CL_REP_PC_RIDING_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000d8`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRidingSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `eRT` at offset 4.
    pub e_rt: i32,
}

impl PcRidingSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.e_rt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            e_rt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcRidingSuccess0104 {
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

/// `sP_FE2CL_PC_RIDING` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000d9`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRiding0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `eRT` at offset 4.
    pub e_rt: i32,
}

impl PcRiding0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.e_rt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            e_rt: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcRiding0104 {
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

/// `sP_FE2CL_PC_BROOMSTICK_MOVE` (`#pragma pack(4)`, 20 bytes, packet ID `0x310000da`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBroomstickMove0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iToX` at offset 4.
    pub to_x: i32,
    /// `iToY` at offset 8.
    pub to_y: i32,
    /// `iToZ` at offset 12.
    pub to_z: i32,
    /// `iSpeed` at offset 16.
    pub speed: i32,
}

impl PcBroomstickMove0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.to_x.to_le_bytes());
        write_prim(out, 8, &self.to_y.to_le_bytes());
        write_prim(out, 12, &self.to_z.to_le_bytes());
        write_prim(out, 16, &self.speed.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            to_x: i32::from_le_bytes(read_array(bytes, 4)),
            to_y: i32::from_le_bytes(read_array(bytes, 8)),
            to_z: i32::from_le_bytes(read_array(bytes, 12)),
            speed: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcBroomstickMove0104 {
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

/// `sP_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x310000db`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuddyWarpOtherShardSuccess0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iShardNum` at offset 8.
    pub shard_num: i8,
    /// `iChannelNum` at offset 12.
    pub channel_num: i32,
}

impl PcBuddyWarpOtherShardSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.shard_num.to_le_bytes());
        write_prim(out, 12, &self.channel_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            shard_num: i8::from_le_bytes(read_array(bytes, 8)),
            channel_num: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcBuddyWarpOtherShardSuccess0104 {
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

/// `sP_FE2CL_REP_WARP_USE_RECALL_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000dc`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct WarpUseRecallFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl WarpUseRecallFailure0104 {
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

impl WirePayload for WarpUseRecallFailure0104 {
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

/// `sP_FE2CL_REP_PC_EXIT_DUPLICATE` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000dd`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcExitDuplicateReply0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcExitDuplicateReply0104 {
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

impl WirePayload for PcExitDuplicateReply0104 {
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

/// `sP_FE2CL_REP_PC_MISSION_COMPLETE_SUCC` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000de`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcMissionCompleteSuccess0104 {
    /// `iMissionNum` at offset 0.
    pub mission_num: i32,
}

impl PcMissionCompleteSuccess0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.mission_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            mission_num: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcMissionCompleteSuccess0104 {
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

/// `sP_FE2CL_PC_BUFF_UPDATE` (`#pragma pack(4)`, 44 bytes, packet ID `0x310000df`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuffUpdate0104 {
    /// `eCSTB` at offset 0.
    pub cstb: i32,
    /// `eTBU` at offset 4.
    pub e_tbu: i32,
    /// `eTBT` at offset 8.
    pub e_tbt: i32,
    /// `TimeBuff` at offset 12.
    pub time_buff: TimeBuff0104,
    /// `iConditionBitFlag` at offset 40.
    pub condition_bit_flag: i32,
}

impl PcBuffUpdate0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.cstb.to_le_bytes());
        write_prim(out, 4, &self.e_tbu.to_le_bytes());
        write_prim(out, 8, &self.e_tbt.to_le_bytes());
        self.time_buff.write_into(&mut out[12..40]);
        write_prim(out, 40, &self.condition_bit_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            cstb: i32::from_le_bytes(read_array(bytes, 0)),
            e_tbu: i32::from_le_bytes(read_array(bytes, 4)),
            e_tbt: i32::from_le_bytes(read_array(bytes, 8)),
            time_buff: TimeBuff0104::read_from(&bytes[12..40]),
            condition_bit_flag: i32::from_le_bytes(read_array(bytes, 40)),
        }
    }
}

impl WirePayload for PcBuffUpdate0104 {
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

/// `sP_FE2CL_REP_PC_NEW_EMAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x310000e0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcNewEmailReply0104 {
    /// `iNewEmailCnt` at offset 0.
    pub new_email_cnt: i32,
}

impl PcNewEmailReply0104 {
    pub const SIZE: usize = 4;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.new_email_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            new_email_cnt: i32::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for PcNewEmailReply0104 {
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

/// `sP_FE2CL_REP_PC_READ_EMAIL_SUCC` (`#pragma pack(4)`, 1084 bytes, packet ID `0x310000e1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcReadEmailSuccess0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `szContent` at offset 8.
    pub content: FixedUtf16<512>,
    /// `aItem` at offset 1032.
    pub item: [ItemBase0104; 4],
    /// `iCash` at offset 1080.
    pub cash: i32,
}

impl PcReadEmailSuccess0104 {
    pub const SIZE: usize = 1084;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_utf16(out, 8, &self.content);
        for (index, value) in self.item.iter().enumerate() {
            let start = 1032 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
        write_prim(out, 1080, &self.cash.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            content: read_utf16(bytes, 8),
            item: std::array::from_fn(|index| {
                let start = 1032 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
            cash: i32::from_le_bytes(read_array(bytes, 1080)),
        }
    }
}

impl WirePayload for PcReadEmailSuccess0104 {
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

/// `sP_FE2CL_REP_PC_READ_EMAIL_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000e2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcReadEmailFailure0104 {
    /// `iEmailIndex` at offset 0.
    pub email_index: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl PcReadEmailFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.email_index.to_le_bytes());
        write_prim(out, 8, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index: i64::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcReadEmailFailure0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_PAGE_LIST_SUCC` (`#pragma pack(4)`, 1024 bytes, packet ID `0x310000e3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailPageListSuccess0104 {
    /// `iPageNum` at offset 0.
    pub page_num: i8,
    /// `aEmailInfo` at offset 4.
    pub email_info: [EmailInfo0104; 5],
}

impl PcRecvEmailPageListSuccess0104 {
    pub const SIZE: usize = 1024;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.page_num.to_le_bytes());
        for (index, value) in self.email_info.iter().enumerate() {
            let start = 4 + index * 204;
            value.write_into(&mut out[start..start + 204]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            page_num: i8::from_le_bytes(read_array(bytes, 0)),
            email_info: std::array::from_fn(|index| {
                let start = 4 + index * 204;
                EmailInfo0104::read_from(&bytes[start..start + 204])
            }),
        }
    }
}

impl WirePayload for PcRecvEmailPageListSuccess0104 {
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

/// `sP_FE2CL_REP_PC_RECV_EMAIL_PAGE_LIST_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000e4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRecvEmailPageListFailure0104 {
    /// `iPageNum` at offset 0.
    pub page_num: i8,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcRecvEmailPageListFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.page_num.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            page_num: i8::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcRecvEmailPageListFailure0104 {
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

/// `sP_FE2CL_REP_PC_DELETE_EMAIL_SUCC` (`#pragma pack(4)`, 40 bytes, packet ID `0x310000e5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDeleteEmailSuccess0104 {
    /// `iEmailIndexArray` at offset 0.
    pub email_index_array: [i64; 5],
}

impl PcDeleteEmailSuccess0104 {
    pub const SIZE: usize = 40;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.email_index_array.iter().enumerate() {
            write_prim(out, 0 + index * 8, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index_array: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 0 + index * 8))
            }),
        }
    }
}

impl WirePayload for PcDeleteEmailSuccess0104 {
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

/// `sP_FE2CL_REP_PC_DELETE_EMAIL_FAIL` (`#pragma pack(4)`, 44 bytes, packet ID `0x310000e6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcDeleteEmailFailure0104 {
    /// `iEmailIndexArray` at offset 0.
    pub email_index_array: [i64; 5],
    /// `iErrorCode` at offset 40.
    pub error_code: i32,
}

impl PcDeleteEmailFailure0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.email_index_array.iter().enumerate() {
            write_prim(out, 0 + index * 8, &value.to_le_bytes());
        }
        write_prim(out, 40, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            email_index_array: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 0 + index * 8))
            }),
            error_code: i32::from_le_bytes(read_array(bytes, 40)),
        }
    }
}

impl WirePayload for PcDeleteEmailFailure0104 {
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

/// `sP_FE2CL_REP_PC_SEND_EMAIL_SUCC` (`#pragma pack(4)`, 76 bytes, packet ID `0x310000e7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSendEmailSuccess0104 {
    /// `iTo_PCUID` at offset 0.
    pub to_pcuid: i64,
    /// `iCandy` at offset 8.
    pub candy: i32,
    /// `aItem` at offset 12.
    pub item: [EmailItemInfoFromClient0104; 4],
}

impl PcSendEmailSuccess0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.to_pcuid.to_le_bytes());
        write_prim(out, 8, &self.candy.to_le_bytes());
        for (index, value) in self.item.iter().enumerate() {
            let start = 12 + index * 16;
            value.write_into(&mut out[start..start + 16]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            to_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            candy: i32::from_le_bytes(read_array(bytes, 8)),
            item: std::array::from_fn(|index| {
                let start = 12 + index * 16;
                EmailItemInfoFromClient0104::read_from(&bytes[start..start + 16])
            }),
        }
    }
}

impl WirePayload for PcSendEmailSuccess0104 {
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

/// `sP_FE2CL_REP_PC_SEND_EMAIL_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000e8`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSendEmailFailure0104 {
    /// `iTo_PCUID` at offset 0.
    pub to_pcuid: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl PcSendEmailFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.to_pcuid.to_le_bytes());
        write_prim(out, 8, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            to_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcSendEmailFailure0104 {
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
