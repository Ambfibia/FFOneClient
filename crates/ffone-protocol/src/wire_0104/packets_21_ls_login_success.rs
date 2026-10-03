// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_LS2CL_REP_LOGIN_SUCC` (`#pragma pack(4)`, 84 bytes, packet ID `0x21000001`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsLoginSuccess0104 {
    /// `iCharCount` at offset 0.
    pub char_count: i8,
    /// `iSlotNum` at offset 1.
    pub slot_num: i8,
    /// `iPaymentFlag` at offset 2.
    pub payment_flag: i8,
    /// `iTempForPacking4` at offset 3.
    pub temp_for_packing4: i8,
    /// `uiSvrTime` at offset 4.
    pub svr_time: u64,
    /// `szID` at offset 12.
    pub id: FixedUtf16<33>,
    /// `iOpenBetaFlag` at offset 80.
    pub open_beta_flag: i32,
}

impl LsLoginSuccess0104 {
    pub const SIZE: usize = 84;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.char_count.to_le_bytes());
        write_prim(out, 1, &self.slot_num.to_le_bytes());
        write_prim(out, 2, &self.payment_flag.to_le_bytes());
        write_prim(out, 3, &self.temp_for_packing4.to_le_bytes());
        write_prim(out, 4, &self.svr_time.to_le_bytes());
        write_utf16(out, 12, &self.id);
        write_prim(out, 80, &self.open_beta_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            char_count: i8::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 1)),
            payment_flag: i8::from_le_bytes(read_array(bytes, 2)),
            temp_for_packing4: i8::from_le_bytes(read_array(bytes, 3)),
            svr_time: u64::from_le_bytes(read_array(bytes, 4)),
            id: read_utf16(bytes, 12),
            open_beta_flag: i32::from_le_bytes(read_array(bytes, 80)),
        }
    }
}

impl WirePayload for LsLoginSuccess0104 {
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

/// `sP_LS2CL_REP_LOGIN_FAIL` (`#pragma pack(4)`, 72 bytes, packet ID `0x21000002`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsLoginFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `szID` at offset 4.
    pub id: FixedUtf16<33>,
}

impl LsLoginFailure0104 {
    pub const SIZE: usize = 72;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_utf16(out, 4, &self.id);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            id: read_utf16(bytes, 4),
        }
    }
}

impl WirePayload for LsLoginFailure0104 {
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

/// `sP_LS2CL_REP_CHAR_INFO` (`#pragma pack(4)`, 204 bytes, packet ID `0x21000003`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharInfoReply0104 {
    /// `iSlot` at offset 0.
    pub slot: i8,
    /// `iLevel` at offset 2.
    pub level: i16,
    /// `sPC_Style` at offset 4.
    pub s_pc_style: PcStyle0104,
    /// `sPC_Style2` at offset 80.
    pub s_pc_style2: PcStyle20104,
    /// `iX` at offset 84.
    pub x: i32,
    /// `iY` at offset 88.
    pub y: i32,
    /// `iZ` at offset 92.
    pub z: i32,
    /// `aEquip` at offset 96.
    pub equip: [ItemBase0104; 9],
}

impl LsCharInfoReply0104 {
    pub const SIZE: usize = 204;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot.to_le_bytes());
        write_prim(out, 2, &self.level.to_le_bytes());
        self.s_pc_style.write_into(&mut out[4..80]);
        self.s_pc_style2.write_into(&mut out[80..83]);
        write_prim(out, 84, &self.x.to_le_bytes());
        write_prim(out, 88, &self.y.to_le_bytes());
        write_prim(out, 92, &self.z.to_le_bytes());
        for (index, value) in self.equip.iter().enumerate() {
            let start = 96 + index * 12;
            value.write_into(&mut out[start..start + 12]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot: i8::from_le_bytes(read_array(bytes, 0)),
            level: i16::from_le_bytes(read_array(bytes, 2)),
            s_pc_style: PcStyle0104::read_from(&bytes[4..80]),
            s_pc_style2: PcStyle20104::read_from(&bytes[80..83]),
            x: i32::from_le_bytes(read_array(bytes, 84)),
            y: i32::from_le_bytes(read_array(bytes, 88)),
            z: i32::from_le_bytes(read_array(bytes, 92)),
            equip: std::array::from_fn(|index| {
                let start = 96 + index * 12;
                ItemBase0104::read_from(&bytes[start..start + 12])
            }),
        }
    }
}

impl WirePayload for LsCharInfoReply0104 {
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

/// `sP_LS2CL_REP_CHECK_CHAR_NAME_SUCC` (`#pragma pack(2)`, 52 bytes, packet ID `0x21000005`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCheckCharNameSuccess0104 {
    /// `szFirstName` at offset 0.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 18.
    pub last_name: FixedUtf16<17>,
}

impl LsCheckCharNameSuccess0104 {
    pub const SIZE: usize = 52;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.first_name);
        write_utf16(out, 18, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
        }
    }
}

impl WirePayload for LsCheckCharNameSuccess0104 {
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

/// `sP_LS2CL_REP_CHECK_CHAR_NAME_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x21000006`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCheckCharNameFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsCheckCharNameFailure0104 {
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

impl WirePayload for LsCheckCharNameFailure0104 {
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

/// `sP_LS2CL_REP_SAVE_CHAR_NAME_SUCC` (`#pragma pack(4)`, 64 bytes, packet ID `0x21000007`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsSaveCharNameSuccess0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
    /// `iGender` at offset 9.
    pub gender: i8,
    /// `szFirstName` at offset 10.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 28.
    pub last_name: FixedUtf16<17>,
}

impl LsSaveCharNameSuccess0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 9, &self.gender.to_le_bytes());
        write_utf16(out, 10, &self.first_name);
        write_utf16(out, 28, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
            gender: i8::from_le_bytes(read_array(bytes, 9)),
            first_name: read_utf16(bytes, 10),
            last_name: read_utf16(bytes, 28),
        }
    }
}

impl WirePayload for LsSaveCharNameSuccess0104 {
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

/// `sP_LS2CL_REP_SAVE_CHAR_NAME_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x21000008`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsSaveCharNameFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsSaveCharNameFailure0104 {
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

impl WirePayload for LsSaveCharNameFailure0104 {
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

/// `sP_LS2CL_REP_CHAR_CREATE_SUCC` (`#pragma pack(4)`, 100 bytes, packet ID `0x21000009`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharCreateSuccess0104 {
    /// `iLevel` at offset 0.
    pub level: i16,
    /// `sPC_Style` at offset 4.
    pub s_pc_style: PcStyle0104,
    /// `sPC_Style2` at offset 80.
    pub s_pc_style2: PcStyle20104,
    /// `sOn_Item` at offset 84.
    pub s_on_item: OnItem0104,
}

impl LsCharCreateSuccess0104 {
    pub const SIZE: usize = 100;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.level.to_le_bytes());
        self.s_pc_style.write_into(&mut out[4..80]);
        self.s_pc_style2.write_into(&mut out[80..83]);
        self.s_on_item.write_into(&mut out[84..98]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            level: i16::from_le_bytes(read_array(bytes, 0)),
            s_pc_style: PcStyle0104::read_from(&bytes[4..80]),
            s_pc_style2: PcStyle20104::read_from(&bytes[80..83]),
            s_on_item: OnItem0104::read_from(&bytes[84..98]),
        }
    }
}

impl WirePayload for LsCharCreateSuccess0104 {
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

/// `sP_LS2CL_REP_CHAR_CREATE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x2100000a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharCreateFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsCharCreateFailure0104 {
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

impl WirePayload for LsCharCreateFailure0104 {
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

/// `sP_LS2CL_REP_CHAR_SELECT_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x2100000b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharSelectSuccess0104;

impl LsCharSelectSuccess0104 {
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

impl WirePayload for LsCharSelectSuccess0104 {
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

/// `sP_LS2CL_REP_CHAR_SELECT_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x2100000c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharSelectFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsCharSelectFailure0104 {
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

impl WirePayload for LsCharSelectFailure0104 {
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

/// `sP_LS2CL_REP_CHAR_DELETE_SUCC` (`#pragma pack(1)`, 1 bytes, packet ID `0x2100000d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharDeleteSuccess0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i8,
}

impl LsCharDeleteSuccess0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i8::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LsCharDeleteSuccess0104 {
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

/// `sP_LS2CL_REP_CHAR_DELETE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x2100000e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharDeleteFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsCharDeleteFailure0104 {
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

impl WirePayload for LsCharDeleteFailure0104 {
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

/// `sP_LS2CL_REP_SHARD_SELECT_SUCC` (`#pragma pack(4)`, 28 bytes, packet ID `0x2100000f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsShardSelectSuccess0104 {
    /// `g_FE_ServerIP` at offset 0.
    pub g_fe_server_ip: [u8; 16],
    /// `g_FE_ServerPort` at offset 16.
    pub g_fe_server_port: i32,
    /// `iEnterSerialKey` at offset 20.
    pub enter_serial_key: i64,
}

impl LsShardSelectSuccess0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.g_fe_server_ip.iter().enumerate() {
            write_prim(out, 0 + index * 1, &value.to_le_bytes());
        }
        write_prim(out, 16, &self.g_fe_server_port.to_le_bytes());
        write_prim(out, 20, &self.enter_serial_key.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            g_fe_server_ip: std::array::from_fn(|index| {
                u8::from_le_bytes(read_array(bytes, 0 + index * 1))
            }),
            g_fe_server_port: i32::from_le_bytes(read_array(bytes, 16)),
            enter_serial_key: i64::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for LsShardSelectSuccess0104 {
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

/// `sP_LS2CL_REP_SHARD_SELECT_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x21000010`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsShardSelectFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsShardSelectFailure0104 {
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

impl WirePayload for LsShardSelectFailure0104 {
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

/// `sP_LS2CL_REP_VERSION_CHECK_SUCC` (`#pragma pack(8)`, 1 bytes, packet ID `0x21000011`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct LsVersionCheckSuccess0104;

impl LsVersionCheckSuccess0104 {
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

impl WirePayload for LsVersionCheckSuccess0104 {
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

/// `sP_LS2CL_REP_VERSION_CHECK_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x21000012`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsVersionCheckFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl LsVersionCheckFailure0104 {
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

impl WirePayload for LsVersionCheckFailure0104 {
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

/// `sP_LS2CL_REP_CHECK_NAME_LIST_SUCC` (`#pragma pack(4)`, 76 bytes, packet ID `0x21000013`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCheckNameListSuccess0104 {
    /// `iFNCode` at offset 0.
    pub fn_code: i32,
    /// `iMNCode` at offset 4.
    pub mn_code: i32,
    /// `iLNCode` at offset 8.
    pub ln_code: i32,
    /// `aNameCodeFlag` at offset 12.
    pub name_code_flag: [i64; 8],
}

impl LsCheckNameListSuccess0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.fn_code.to_le_bytes());
        write_prim(out, 4, &self.mn_code.to_le_bytes());
        write_prim(out, 8, &self.ln_code.to_le_bytes());
        for (index, value) in self.name_code_flag.iter().enumerate() {
            write_prim(out, 12 + index * 8, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            fn_code: i32::from_le_bytes(read_array(bytes, 0)),
            mn_code: i32::from_le_bytes(read_array(bytes, 4)),
            ln_code: i32::from_le_bytes(read_array(bytes, 8)),
            name_code_flag: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 12 + index * 8))
            }),
        }
    }
}

impl WirePayload for LsCheckNameListSuccess0104 {
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
