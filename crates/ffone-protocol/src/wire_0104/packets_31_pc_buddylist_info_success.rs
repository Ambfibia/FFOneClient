// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000063`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuddylistInfoSuccess0104 {
    /// `iID` at offset 0.
    pub id: i32,
    /// `iPCUID` at offset 4.
    pub pcuid: i64,
    /// `iListNum` at offset 12.
    pub list_num: i8,
    /// `iBuddyCnt` at offset 13.
    pub buddy_cnt: i8,
}

impl PcBuddylistInfoSuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id.to_le_bytes());
        write_prim(out, 4, &self.pcuid.to_le_bytes());
        write_prim(out, 12, &self.list_num.to_le_bytes());
        write_prim(out, 13, &self.buddy_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id: i32::from_le_bytes(read_array(bytes, 0)),
            pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            list_num: i8::from_le_bytes(read_array(bytes, 12)),
            buddy_cnt: i8::from_le_bytes(read_array(bytes, 13)),
        }
    }
}

impl WirePayload for PcBuddylistInfoSuccess0104 {
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

/// `sP_FE2CL_REP_PC_BUDDYLIST_INFO_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000064`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBuddylistInfoFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl PcBuddylistInfoFailure0104 {
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

impl WirePayload for PcBuddylistInfoFailure0104 {
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

/// `sP_FE2CL_REP_REQUEST_MAKE_BUDDY_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000066`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestMakeBuddyFailure0104 {
    /// `iBuddyID` at offset 0.
    pub buddy_id: i32,
    /// `iBuddyPCUID` at offset 4.
    pub buddy_pcuid: i64,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl RequestMakeBuddyFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_id.to_le_bytes());
        write_prim(out, 4, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_id: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for RequestMakeBuddyFailure0104 {
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

/// `sP_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC` (`#pragma pack(4)`, 76 bytes, packet ID `0x31000067`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptMakeBuddySuccess0104 {
    /// `iBuddySlot` at offset 0.
    pub buddy_slot: i8,
    /// `BuddyInfo` at offset 4.
    pub buddy_info: BuddyBaseInfo0104,
}

impl AcceptMakeBuddySuccess0104 {
    pub const SIZE: usize = 76;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_slot.to_le_bytes());
        self.buddy_info.write_into(&mut out[4..76]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_slot: i8::from_le_bytes(read_array(bytes, 0)),
            buddy_info: BuddyBaseInfo0104::read_from(&bytes[4..76]),
        }
    }
}

impl WirePayload for AcceptMakeBuddySuccess0104 {
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

/// `sP_FE2CL_REP_ACCEPT_MAKE_BUDDY_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000068`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptMakeBuddyFailure0104 {
    /// `iBuddyID` at offset 0.
    pub buddy_id: i32,
    /// `iBuddyPCUID` at offset 4.
    pub buddy_pcuid: i64,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl AcceptMakeBuddyFailure0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_id.to_le_bytes());
        write_prim(out, 4, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_id: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for AcceptMakeBuddyFailure0104 {
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

/// `sP_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 276 bytes, packet ID `0x31000069`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendBuddyFreechatMessageSuccess0104 {
    /// `iFromPCUID` at offset 0.
    pub from_pcuid: i64,
    /// `iToPCUID` at offset 8.
    pub to_pcuid: i64,
    /// `szFreeChat` at offset 16.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 272.
    pub emote_code: i32,
}

impl SendBuddyFreechatMessageSuccess0104 {
    pub const SIZE: usize = 276;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.from_pcuid.to_le_bytes());
        write_prim(out, 8, &self.to_pcuid.to_le_bytes());
        write_utf16(out, 16, &self.free_chat);
        write_prim(out, 272, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            from_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            to_pcuid: i64::from_le_bytes(read_array(bytes, 8)),
            free_chat: read_utf16(bytes, 16),
            emote_code: i32::from_le_bytes(read_array(bytes, 272)),
        }
    }
}

impl WirePayload for SendBuddyFreechatMessageSuccess0104 {
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

/// `sP_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 272 bytes, packet ID `0x3100006a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendBuddyFreechatMessageFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iToPCUID` at offset 4.
    pub to_pcuid: i64,
    /// `szFreeChat` at offset 12.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 268.
    pub emote_code: i32,
}

impl SendBuddyFreechatMessageFailure0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.to_pcuid.to_le_bytes());
        write_utf16(out, 12, &self.free_chat);
        write_prim(out, 268, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            to_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            free_chat: read_utf16(bytes, 12),
            emote_code: i32::from_le_bytes(read_array(bytes, 268)),
        }
    }
}

impl WirePayload for SendBuddyFreechatMessageFailure0104 {
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

/// `sP_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 276 bytes, packet ID `0x3100006b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendBuddyMenuchatMessageSuccess0104 {
    /// `iFromPCUID` at offset 0.
    pub from_pcuid: i64,
    /// `iToPCUID` at offset 8.
    pub to_pcuid: i64,
    /// `szFreeChat` at offset 16.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 272.
    pub emote_code: i32,
}

impl SendBuddyMenuchatMessageSuccess0104 {
    pub const SIZE: usize = 276;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.from_pcuid.to_le_bytes());
        write_prim(out, 8, &self.to_pcuid.to_le_bytes());
        write_utf16(out, 16, &self.free_chat);
        write_prim(out, 272, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            from_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            to_pcuid: i64::from_le_bytes(read_array(bytes, 8)),
            free_chat: read_utf16(bytes, 16),
            emote_code: i32::from_le_bytes(read_array(bytes, 272)),
        }
    }
}

impl WirePayload for SendBuddyMenuchatMessageSuccess0104 {
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

/// `sP_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 272 bytes, packet ID `0x3100006c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendBuddyMenuchatMessageFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iToPCUID` at offset 4.
    pub to_pcuid: i64,
    /// `szFreeChat` at offset 12.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 268.
    pub emote_code: i32,
}

impl SendBuddyMenuchatMessageFailure0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.to_pcuid.to_le_bytes());
        write_utf16(out, 12, &self.free_chat);
        write_prim(out, 268, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            to_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
            free_chat: read_utf16(bytes, 12),
            emote_code: i32::from_le_bytes(read_array(bytes, 268)),
        }
    }
}

impl WirePayload for SendBuddyMenuchatMessageFailure0104 {
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

/// `sP_FE2CL_REP_GET_BUDDY_STYLE_SUCC` (`#pragma pack(4)`, 196 bytes, packet ID `0x3100006d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyStyleSuccess0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 8.
    pub buddy_slot: i8,
    /// `sBuddyStyle` at offset 12.
    pub s_buddy_style: BuddyStyleInfo0104,
}

impl GetBuddyStyleSuccess0104 {
    pub const SIZE: usize = 196;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.buddy_slot.to_le_bytes());
        self.s_buddy_style.write_into(&mut out[12..196]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 8)),
            s_buddy_style: BuddyStyleInfo0104::read_from(&bytes[12..196]),
        }
    }
}

impl WirePayload for GetBuddyStyleSuccess0104 {
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

/// `sP_FE2CL_REP_GET_BUDDY_STYLE_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x3100006e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyStyleFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
    /// `iBuddyPCUID` at offset 4.
    pub buddy_pcuid: i64,
}

impl GetBuddyStyleFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.error_code.to_le_bytes());
        write_prim(out, 4, &self.buddy_pcuid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            error_code: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for GetBuddyStyleFailure0104 {
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

/// `sP_FE2CL_REP_GET_BUDDY_STATE_SUCC` (`#pragma pack(4)`, 252 bytes, packet ID `0x3100006f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyStateSuccess0104 {
    /// `aBuddyID` at offset 0.
    pub buddy_id: [i32; 50],
    /// `aBuddyState` at offset 200.
    pub buddy_state: [u8; 50],
}

impl GetBuddyStateSuccess0104 {
    pub const SIZE: usize = 252;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        for (index, value) in self.buddy_id.iter().enumerate() {
            write_prim(out, 0 + index * 4, &value.to_le_bytes());
        }
        for (index, value) in self.buddy_state.iter().enumerate() {
            write_prim(out, 200 + index * 1, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_id: std::array::from_fn(|index| {
                i32::from_le_bytes(read_array(bytes, 0 + index * 4))
            }),
            buddy_state: std::array::from_fn(|index| {
                u8::from_le_bytes(read_array(bytes, 200 + index * 1))
            }),
        }
    }
}

impl WirePayload for GetBuddyStateSuccess0104 {
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

/// `sP_FE2CL_REP_GET_BUDDY_STATE_FAIL` (`#pragma pack(4)`, 4 bytes, packet ID `0x31000070`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBuddyStateFailure0104 {
    /// `iErrorCode` at offset 0.
    pub error_code: i32,
}

impl GetBuddyStateFailure0104 {
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

impl WirePayload for GetBuddyStateFailure0104 {
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

/// `sP_FE2CL_REP_SET_BUDDY_BLOCK_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000071`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SetBuddyBlockSuccess0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 8.
    pub buddy_slot: i8,
}

impl SetBuddyBlockSuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for SetBuddyBlockSuccess0104 {
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

/// `sP_FE2CL_REP_SET_BUDDY_BLOCK_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000072`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SetBuddyBlockFailure0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl SetBuddyBlockFailure0104 {
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

impl WirePayload for SetBuddyBlockFailure0104 {
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

/// `sP_FE2CL_REP_REMOVE_BUDDY_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000073`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveBuddySuccess0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iBuddySlot` at offset 8.
    pub buddy_slot: i8,
}

impl RemoveBuddySuccess0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.buddy_pcuid.to_le_bytes());
        write_prim(out, 8, &self.buddy_slot.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 0)),
            buddy_slot: i8::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for RemoveBuddySuccess0104 {
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

/// `sP_FE2CL_REP_REMOVE_BUDDY_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000074`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveBuddyFailure0104 {
    /// `iBuddyPCUID` at offset 0.
    pub buddy_pcuid: i64,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl RemoveBuddyFailure0104 {
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

impl WirePayload for RemoveBuddyFailure0104 {
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
