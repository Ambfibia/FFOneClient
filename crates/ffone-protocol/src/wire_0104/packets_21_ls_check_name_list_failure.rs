// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_LS2CL_REP_CHECK_NAME_LIST_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x21000014`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCheckNameListFailure0104 {
    /// `iFNCode` at offset 0.
    pub fn_code: i32,
    /// `iMNCode` at offset 4.
    pub mn_code: i32,
    /// `iLNCode` at offset 8.
    pub ln_code: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl LsCheckNameListFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.fn_code.to_le_bytes());
        write_prim(out, 4, &self.mn_code.to_le_bytes());
        write_prim(out, 8, &self.ln_code.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            fn_code: i32::from_le_bytes(read_array(bytes, 0)),
            mn_code: i32::from_le_bytes(read_array(bytes, 4)),
            ln_code: i32::from_le_bytes(read_array(bytes, 8)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for LsCheckNameListFailure0104 {
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

/// `sP_LS2CL_REP_PC_EXIT_DUPLICATE` (`#pragma pack(4)`, 4 bytes, packet ID `0x21000015`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsPcExitDuplicateReply0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsPcExitDuplicateReply0104 {
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

impl WirePayload for LsPcExitDuplicateReply0104 {
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

/// `sP_LS2CL_REQ_LIVE_CHECK` (`#pragma pack(4)`, 4 bytes, packet ID `0x21000016`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsLiveCheck0104 {
    /// `iTempValue` at offset 0.
    pub temp_value: i32,
}

impl LsLiveCheck0104 {
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

impl WirePayload for LsLiveCheck0104 {
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

/// `sP_LS2CL_REP_CHANGE_CHAR_NAME_SUCC` (`#pragma pack(4)`, 64 bytes, packet ID `0x21000017`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsChangeCharNameSuccess0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
    /// `szFirstName` at offset 10.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 28.
    pub last_name: FixedUtf16<17>,
}

impl LsChangeCharNameSuccess0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_utf16(out, 10, &self.first_name);
        write_utf16(out, 28, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
            first_name: read_utf16(bytes, 10),
            last_name: read_utf16(bytes, 28),
        }
    }
}

impl WirePayload for LsChangeCharNameSuccess0104 {
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

/// `sP_LS2CL_REP_CHANGE_CHAR_NAME_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x21000018`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsChangeCharNameFailure0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl LsChangeCharNameFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for LsChangeCharNameFailure0104 {
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

/// `sP_LS2CL_REP_SHARD_LIST_INFO_SUCC` (`#pragma pack(1)`, 26 bytes, packet ID `0x21000019`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsShardListInfoSuccess0104 {
    /// `aShardConnectFlag` at offset 0.
    pub shard_connect_flag: [u8; 26],
}

impl LsShardListInfoSuccess0104 {
    pub const SIZE: usize = 26;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.shard_connect_flag.iter().enumerate() {
            write_prim(out, 0 + index * 1, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shard_connect_flag: std::array::from_fn(|index| {
                u8::from_le_bytes(read_array(bytes, 0 + index * 1))
            }),
        }
    }
}

impl WirePayload for LsShardListInfoSuccess0104 {
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
