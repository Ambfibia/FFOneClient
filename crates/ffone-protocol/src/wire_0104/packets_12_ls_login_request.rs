// Generated wire layouts; do not edit by hand.
use super::*;
// ---- packets ----------------------------------------------------------------

/// `sP_CL2LS_REQ_LOGIN` (`#pragma pack(4)`, 468 bytes, packet ID `0x12000001`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsLoginRequest0104 {
    /// `szID` at offset 0.
    pub id: FixedUtf16<33>,
    /// `szPassword` at offset 66.
    pub password: FixedUtf16<33>,
    /// `iClientVerA` at offset 132.
    pub client_ver_a: i32,
    /// `iClientVerB` at offset 136.
    pub client_ver_b: i32,
    /// `iClientVerC` at offset 140.
    pub client_ver_c: i32,
    /// `iLoginType` at offset 144.
    pub login_type: i32,
    /// `szCookie_TEGid` at offset 148.
    pub cookie_te_gid: [u8; 64],
    /// `szCookie_authid` at offset 212.
    pub cookie_authid: [u8; 255],
}

impl LsLoginRequest0104 {
    pub const SIZE: usize = 468;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.id);
        write_utf16(out, 66, &self.password);
        write_prim(out, 132, &self.client_ver_a.to_le_bytes());
        write_prim(out, 136, &self.client_ver_b.to_le_bytes());
        write_prim(out, 140, &self.client_ver_c.to_le_bytes());
        write_prim(out, 144, &self.login_type.to_le_bytes());
        for (index, value) in self.cookie_te_gid.iter().enumerate() {
            write_prim(out, 148 + index * 1, &value.to_le_bytes());
        }
        for (index, value) in self.cookie_authid.iter().enumerate() {
            write_prim(out, 212 + index * 1, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: read_utf16(bytes, 0),
            password: read_utf16(bytes, 66),
            client_ver_a: i32::from_le_bytes(read_array(bytes, 132)),
            client_ver_b: i32::from_le_bytes(read_array(bytes, 136)),
            client_ver_c: i32::from_le_bytes(read_array(bytes, 140)),
            login_type: i32::from_le_bytes(read_array(bytes, 144)),
            cookie_te_gid: std::array::from_fn(|index| {
                u8::from_le_bytes(read_array(bytes, 148 + index * 1))
            }),
            cookie_authid: std::array::from_fn(|index| {
                u8::from_le_bytes(read_array(bytes, 212 + index * 1))
            }),
        }
    }
}

impl WirePayload for LsLoginRequest0104 {
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

/// `sP_CL2LS_REQ_CHECK_CHAR_NAME` (`#pragma pack(4)`, 64 bytes, packet ID `0x12000002`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCheckCharNameRequest0104 {
    /// `iFNCode` at offset 0.
    pub fn_code: i32,
    /// `iLNCode` at offset 4.
    pub ln_code: i32,
    /// `iMNCode` at offset 8.
    pub mn_code: i32,
    /// `szFirstName` at offset 12.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 30.
    pub last_name: FixedUtf16<17>,
}

impl LsCheckCharNameRequest0104 {
    pub const SIZE: usize = 64;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.fn_code.to_le_bytes());
        write_prim(out, 4, &self.ln_code.to_le_bytes());
        write_prim(out, 8, &self.mn_code.to_le_bytes());
        write_utf16(out, 12, &self.first_name);
        write_utf16(out, 30, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            fn_code: i32::from_le_bytes(read_array(bytes, 0)),
            ln_code: i32::from_le_bytes(read_array(bytes, 4)),
            mn_code: i32::from_le_bytes(read_array(bytes, 8)),
            first_name: read_utf16(bytes, 12),
            last_name: read_utf16(bytes, 30),
        }
    }
}

impl WirePayload for LsCheckCharNameRequest0104 {
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

/// `sP_CL2LS_REQ_SAVE_CHAR_NAME` (`#pragma pack(4)`, 68 bytes, packet ID `0x12000003`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsSaveCharNameRequest0104 {
    /// `iSlotNum` at offset 0.
    pub slot_num: i8,
    /// `iGender` at offset 1.
    pub gender: i8,
    /// `iFNCode` at offset 4.
    pub fn_code: i32,
    /// `iLNCode` at offset 8.
    pub ln_code: i32,
    /// `iMNCode` at offset 12.
    pub mn_code: i32,
    /// `szFirstName` at offset 16.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 34.
    pub last_name: FixedUtf16<17>,
}

impl LsSaveCharNameRequest0104 {
    pub const SIZE: usize = 68;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.slot_num.to_le_bytes());
        write_prim(out, 1, &self.gender.to_le_bytes());
        write_prim(out, 4, &self.fn_code.to_le_bytes());
        write_prim(out, 8, &self.ln_code.to_le_bytes());
        write_prim(out, 12, &self.mn_code.to_le_bytes());
        write_utf16(out, 16, &self.first_name);
        write_utf16(out, 34, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            slot_num: i8::from_le_bytes(read_array(bytes, 0)),
            gender: i8::from_le_bytes(read_array(bytes, 1)),
            fn_code: i32::from_le_bytes(read_array(bytes, 4)),
            ln_code: i32::from_le_bytes(read_array(bytes, 8)),
            mn_code: i32::from_le_bytes(read_array(bytes, 12)),
            first_name: read_utf16(bytes, 16),
            last_name: read_utf16(bytes, 34),
        }
    }
}

impl WirePayload for LsSaveCharNameRequest0104 {
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

/// `sP_CL2LS_REQ_CHAR_CREATE` (`#pragma pack(4)`, 100 bytes, packet ID `0x12000004`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharCreateRequest0104 {
    /// `PCStyle` at offset 0.
    pub pc_style: PcStyle0104,
    /// `sOn_Item` at offset 76.
    pub s_on_item: OnItem0104,
    /// `sOn_Item_Index` at offset 90.
    pub s_on_item_index: OnItemIndex0104,
}

impl LsCharCreateRequest0104 {
    pub const SIZE: usize = 100;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        self.pc_style.write_into(&mut out[0..76]);
        self.s_on_item.write_into(&mut out[76..90]);
        self.s_on_item_index.write_into(&mut out[90..100]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_style: PcStyle0104::read_from(&bytes[0..76]),
            s_on_item: OnItem0104::read_from(&bytes[76..90]),
            s_on_item_index: OnItemIndex0104::read_from(&bytes[90..100]),
        }
    }
}

impl WirePayload for LsCharCreateRequest0104 {
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

/// `sP_CL2LS_REQ_CHAR_SELECT` (`#pragma pack(4)`, 8 bytes, packet ID `0x12000005`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharSelectRequest0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
}

impl LsCharSelectRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LsCharSelectRequest0104 {
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

/// `sP_CL2LS_REQ_CHAR_DELETE` (`#pragma pack(4)`, 8 bytes, packet ID `0x12000006`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCharDeleteRequest0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
}

impl LsCharDeleteRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LsCharDeleteRequest0104 {
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

/// `sP_CL2LS_REQ_SHARD_SELECT` (`#pragma pack(1)`, 1 bytes, packet ID `0x12000007`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsShardSelectRequest0104 {
    /// `ShardNum` at offset 0.
    pub shard_num: i8,
}

impl LsShardSelectRequest0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.shard_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            shard_num: i8::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LsShardSelectRequest0104 {
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

/// `sP_CL2LS_REQ_SHARD_LIST_INFO` (`#pragma pack(8)`, 1 bytes, packet ID `0x12000008`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// Layout note: empty struct: one placeholder byte.
#[derive(Debug, Clone, PartialEq)]
pub struct LsShardListInfoRequest0104;

impl LsShardListInfoRequest0104 {
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

impl WirePayload for LsShardListInfoRequest0104 {
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

/// `sP_CL2LS_CHECK_NAME_LIST` (`#pragma pack(4)`, 12 bytes, packet ID `0x12000009`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsCheckNameList0104 {
    /// `iFNCode` at offset 0.
    pub fn_code: i32,
    /// `iMNCode` at offset 4.
    pub mn_code: i32,
    /// `iLNCode` at offset 8.
    pub ln_code: i32,
}

impl LsCheckNameList0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.fn_code.to_le_bytes());
        write_prim(out, 4, &self.mn_code.to_le_bytes());
        write_prim(out, 8, &self.ln_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            fn_code: i32::from_le_bytes(read_array(bytes, 0)),
            mn_code: i32::from_le_bytes(read_array(bytes, 4)),
            ln_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for LsCheckNameList0104 {
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

/// `sP_CL2LS_REQ_SAVE_CHAR_TUTOR` (`#pragma pack(4)`, 12 bytes, packet ID `0x1200000a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsSaveCharTutorRequest0104 {
    /// `iPC_UID` at offset 0.
    pub pc_uid: i64,
    /// `iTutorialFlag` at offset 8.
    pub tutorial_flag: i8,
}

impl LsSaveCharTutorRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_uid.to_le_bytes());
        write_prim(out, 8, &self.tutorial_flag.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: i64::from_le_bytes(read_array(bytes, 0)),
            tutorial_flag: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for LsSaveCharTutorRequest0104 {
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

/// `sP_CL2LS_REQ_PC_EXIT_DUPLICATE` (`#pragma pack(2)`, 132 bytes, packet ID `0x1200000b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsPcExitDuplicateRequest0104 {
    /// `szID` at offset 0.
    pub id: FixedUtf16<33>,
    /// `szPassword` at offset 66.
    pub password: FixedUtf16<33>,
}

impl LsPcExitDuplicateRequest0104 {
    pub const SIZE: usize = 132;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_utf16(out, 0, &self.id);
        write_utf16(out, 66, &self.password);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: read_utf16(bytes, 0),
            password: read_utf16(bytes, 66),
        }
    }
}

impl WirePayload for LsPcExitDuplicateRequest0104 {
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

/// `sP_CL2LS_REP_LIVE_CHECK` (`#pragma pack(4)`, 4 bytes, packet ID `0x1200000c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsLiveCheckReply0104 {
    /// `iTempValue` at offset 0.
    pub temp_value: i32,
}

impl LsLiveCheckReply0104 {
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

impl WirePayload for LsLiveCheckReply0104 {
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

/// `sP_CL2LS_REQ_CHANGE_CHAR_NAME` (`#pragma pack(4)`, 76 bytes, packet ID `0x1200000d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsChangeCharNameRequest0104 {
    /// `iPCUID` at offset 0.
    pub pcuid: i64,
    /// `iSlotNum` at offset 8.
    pub slot_num: i8,
    /// `iGender` at offset 9.
    pub gender: i8,
    /// `iFNCode` at offset 12.
    pub fn_code: i32,
    /// `iLNCode` at offset 16.
    pub ln_code: i32,
    /// `iMNCode` at offset 20.
    pub mn_code: i32,
    /// `szFirstName` at offset 24.
    pub first_name: FixedUtf16<9>,
    /// `szLastName` at offset 42.
    pub last_name: FixedUtf16<17>,
}

impl LsChangeCharNameRequest0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pcuid.to_le_bytes());
        write_prim(out, 8, &self.slot_num.to_le_bytes());
        write_prim(out, 9, &self.gender.to_le_bytes());
        write_prim(out, 12, &self.fn_code.to_le_bytes());
        write_prim(out, 16, &self.ln_code.to_le_bytes());
        write_prim(out, 20, &self.mn_code.to_le_bytes());
        write_utf16(out, 24, &self.first_name);
        write_utf16(out, 42, &self.last_name);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            slot_num: i8::from_le_bytes(read_array(bytes, 8)),
            gender: i8::from_le_bytes(read_array(bytes, 9)),
            fn_code: i32::from_le_bytes(read_array(bytes, 12)),
            ln_code: i32::from_le_bytes(read_array(bytes, 16)),
            mn_code: i32::from_le_bytes(read_array(bytes, 20)),
            first_name: read_utf16(bytes, 24),
            last_name: read_utf16(bytes, 42),
        }
    }
}

impl WirePayload for LsChangeCharNameRequest0104 {
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

/// `sP_CL2LS_REQ_SERVER_SELECT` (`#pragma pack(1)`, 1 bytes, packet ID `0x1200000e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LsServerSelectRequest0104 {
    /// `ServerNum` at offset 0.
    pub server_num: i8,
}

impl LsServerSelectRequest0104 {
    pub const SIZE: usize = 1;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.server_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            server_num: i8::from_le_bytes(read_array(bytes, 0)),
        }
    }
}

impl WirePayload for LsServerSelectRequest0104 {
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
