// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 268 bytes, packet ID `0x310000b6`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAllGroupFreechatMessageFailure0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
    /// `iErrorCode` at offset 264.
    pub error_code: i32,
}

impl SendAllGroupFreechatMessageFailure0104 {
    pub const SIZE: usize = 268;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
        write_prim(out, 264, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
            error_code: i32::from_le_bytes(read_array(bytes, 264)),
        }
    }
}

impl WirePayload for SendAllGroupFreechatMessageFailure0104 {
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

/// `sP_FE2CL_REP_SEND_ANY_GROUP_FREECHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 268 bytes, packet ID `0x310000b7`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAnyGroupFreechatMessageSuccess0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `iGroupPC_ID` at offset 4.
    pub group_pc_id: i32,
    /// `szFreeChat` at offset 8.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 264.
    pub emote_code: i32,
}

impl SendAnyGroupFreechatMessageSuccess0104 {
    pub const SIZE: usize = 268;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_prim(out, 4, &self.group_pc_id.to_le_bytes());
        write_utf16(out, 8, &self.free_chat);
        write_prim(out, 264, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            group_pc_id: i32::from_le_bytes(read_array(bytes, 4)),
            free_chat: read_utf16(bytes, 8),
            emote_code: i32::from_le_bytes(read_array(bytes, 264)),
        }
    }
}

impl WirePayload for SendAnyGroupFreechatMessageSuccess0104 {
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

/// `sP_FE2CL_REP_SEND_ANY_GROUP_FREECHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 272 bytes, packet ID `0x310000b8`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAnyGroupFreechatMessageFailure0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `iGroupPC_ID` at offset 4.
    pub group_pc_id: i32,
    /// `szFreeChat` at offset 8.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 264.
    pub emote_code: i32,
    /// `iErrorCode` at offset 268.
    pub error_code: i32,
}

impl SendAnyGroupFreechatMessageFailure0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_prim(out, 4, &self.group_pc_id.to_le_bytes());
        write_utf16(out, 8, &self.free_chat);
        write_prim(out, 264, &self.emote_code.to_le_bytes());
        write_prim(out, 268, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            group_pc_id: i32::from_le_bytes(read_array(bytes, 4)),
            free_chat: read_utf16(bytes, 8),
            emote_code: i32::from_le_bytes(read_array(bytes, 264)),
            error_code: i32::from_le_bytes(read_array(bytes, 268)),
        }
    }
}

impl WirePayload for SendAnyGroupFreechatMessageFailure0104 {
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

/// `sP_FE2CL_REP_BARKER` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000b9`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BarkerReply0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iMissionStringID` at offset 4.
    pub mission_string_id: i32,
}

impl BarkerReply0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.mission_string_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            mission_string_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for BarkerReply0104 {
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

/// `sP_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 264 bytes, packet ID `0x310000ba`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAllGroupMenuchatMessageSuccess0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
}

impl SendAllGroupMenuchatMessageSuccess0104 {
    pub const SIZE: usize = 264;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
        }
    }
}

impl WirePayload for SendAllGroupMenuchatMessageSuccess0104 {
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

/// `sP_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 268 bytes, packet ID `0x310000bb`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAllGroupMenuchatMessageFailure0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `szFreeChat` at offset 4.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 260.
    pub emote_code: i32,
    /// `iErrorCode` at offset 264.
    pub error_code: i32,
}

impl SendAllGroupMenuchatMessageFailure0104 {
    pub const SIZE: usize = 268;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_utf16(out, 4, &self.free_chat);
        write_prim(out, 260, &self.emote_code.to_le_bytes());
        write_prim(out, 264, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            free_chat: read_utf16(bytes, 4),
            emote_code: i32::from_le_bytes(read_array(bytes, 260)),
            error_code: i32::from_le_bytes(read_array(bytes, 264)),
        }
    }
}

impl WirePayload for SendAllGroupMenuchatMessageFailure0104 {
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

/// `sP_FE2CL_REP_SEND_ANY_GROUP_MENUCHAT_MESSAGE_SUCC` (`#pragma pack(4)`, 268 bytes, packet ID `0x310000bc`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAnyGroupMenuchatMessageSuccess0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `iGroupPC_ID` at offset 4.
    pub group_pc_id: i32,
    /// `szFreeChat` at offset 8.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 264.
    pub emote_code: i32,
}

impl SendAnyGroupMenuchatMessageSuccess0104 {
    pub const SIZE: usize = 268;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_prim(out, 4, &self.group_pc_id.to_le_bytes());
        write_utf16(out, 8, &self.free_chat);
        write_prim(out, 264, &self.emote_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            group_pc_id: i32::from_le_bytes(read_array(bytes, 4)),
            free_chat: read_utf16(bytes, 8),
            emote_code: i32::from_le_bytes(read_array(bytes, 264)),
        }
    }
}

impl WirePayload for SendAnyGroupMenuchatMessageSuccess0104 {
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

/// `sP_FE2CL_REP_SEND_ANY_GROUP_MENUCHAT_MESSAGE_FAIL` (`#pragma pack(4)`, 272 bytes, packet ID `0x310000bd`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SendAnyGroupMenuchatMessageFailure0104 {
    /// `iSendPCID` at offset 0.
    pub send_pcid: i32,
    /// `iGroupPC_ID` at offset 4.
    pub group_pc_id: i32,
    /// `szFreeChat` at offset 8.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 264.
    pub emote_code: i32,
    /// `iErrorCode` at offset 268.
    pub error_code: i32,
}

impl SendAnyGroupMenuchatMessageFailure0104 {
    pub const SIZE: usize = 272;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.send_pcid.to_le_bytes());
        write_prim(out, 4, &self.group_pc_id.to_le_bytes());
        write_utf16(out, 8, &self.free_chat);
        write_prim(out, 264, &self.emote_code.to_le_bytes());
        write_prim(out, 268, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            send_pcid: i32::from_le_bytes(read_array(bytes, 0)),
            group_pc_id: i32::from_le_bytes(read_array(bytes, 4)),
            free_chat: read_utf16(bytes, 8),
            emote_code: i32::from_le_bytes(read_array(bytes, 264)),
            error_code: i32::from_le_bytes(read_array(bytes, 268)),
        }
    }
}

impl WirePayload for SendAnyGroupMenuchatMessageFailure0104 {
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

/// `sP_FE2CL_REP_PC_REGIST_TRANSPORTATION_LOCATION_FAIL` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000be`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegistTransportationLocationFailure0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iLocationID` at offset 4.
    pub location_id: i32,
    /// `iErrorCode` at offset 8.
    pub error_code: i32,
}

impl PcRegistTransportationLocationFailure0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.location_id.to_le_bytes());
        write_prim(out, 8, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            location_id: i32::from_le_bytes(read_array(bytes, 4)),
            error_code: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcRegistTransportationLocationFailure0104 {
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

/// `sP_FE2CL_REP_PC_REGIST_TRANSPORTATION_LOCATION_SUCC` (`#pragma pack(4)`, 28 bytes, packet ID `0x310000bf`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcRegistTransportationLocationSuccess0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iLocationID` at offset 4.
    pub location_id: i32,
    /// `iWarpLocationFlag` at offset 8.
    pub warp_location_flag: i32,
    /// `aWyvernLocationFlag` at offset 12.
    pub wyvern_location_flag: [i64; 2],
}

impl PcRegistTransportationLocationSuccess0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.location_id.to_le_bytes());
        write_prim(out, 8, &self.warp_location_flag.to_le_bytes());
        for (index, value) in self.wyvern_location_flag.iter().enumerate() {
            write_prim(out, 12 + index * 8, &value.to_le_bytes());
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            location_id: i32::from_le_bytes(read_array(bytes, 4)),
            warp_location_flag: i32::from_le_bytes(read_array(bytes, 8)),
            wyvern_location_flag: std::array::from_fn(|index| {
                i64::from_le_bytes(read_array(bytes, 12 + index * 8))
            }),
        }
    }
}

impl WirePayload for PcRegistTransportationLocationSuccess0104 {
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

/// `sP_FE2CL_REP_PC_WARP_USE_TRANSPORTATION_FAIL` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000c0`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpUseTransportationFailure0104 {
    /// `iTransportationID` at offset 0.
    pub transportation_id: i32,
    /// `iErrorCode` at offset 4.
    pub error_code: i32,
}

impl PcWarpUseTransportationFailure0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.transportation_id.to_le_bytes());
        write_prim(out, 4, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            transportation_id: i32::from_le_bytes(read_array(bytes, 0)),
            error_code: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcWarpUseTransportationFailure0104 {
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

/// `sP_FE2CL_REP_PC_WARP_USE_TRANSPORTATION_SUCC` (`#pragma pack(4)`, 20 bytes, packet ID `0x310000c1`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcWarpUseTransportationSuccess0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iX` at offset 4.
    pub x: i32,
    /// `iY` at offset 8.
    pub y: i32,
    /// `iZ` at offset 12.
    pub z: i32,
    /// `iCandy` at offset 16.
    pub candy: i32,
}

impl PcWarpUseTransportationSuccess0104 {
    pub const SIZE: usize = 20;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.x.to_le_bytes());
        write_prim(out, 8, &self.y.to_le_bytes());
        write_prim(out, 12, &self.z.to_le_bytes());
        write_prim(out, 16, &self.candy.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            x: i32::from_le_bytes(read_array(bytes, 4)),
            y: i32::from_le_bytes(read_array(bytes, 8)),
            z: i32::from_le_bytes(read_array(bytes, 12)),
            candy: i32::from_le_bytes(read_array(bytes, 16)),
        }
    }
}

impl WirePayload for PcWarpUseTransportationSuccess0104 {
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

/// `sP_FE2CL_ANNOUNCE_MSG` (`#pragma pack(4)`, 1032 bytes, packet ID `0x310000c2`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnounceMsg0104 {
    /// `iAnnounceType` at offset 0.
    pub announce_type: i8,
    /// `iDuringTime` at offset 4.
    pub during_time: i32,
    /// `szAnnounceMsg` at offset 8.
    pub announce_msg: FixedUtf16<512>,
}

impl AnnounceMsg0104 {
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

impl WirePayload for AnnounceMsg0104 {
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

/// `sP_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000c3`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSpecialStateSwitchSuccess0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iReqSpecialStateFlag` at offset 4.
    pub req_special_state_flag: i8,
    /// `iSpecialState` at offset 5.
    pub special_state: i8,
}

impl PcSpecialStateSwitchSuccess0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.req_special_state_flag.to_le_bytes());
        write_prim(out, 5, &self.special_state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            req_special_state_flag: i8::from_le_bytes(read_array(bytes, 4)),
            special_state: i8::from_le_bytes(read_array(bytes, 5)),
        }
    }
}

impl WirePayload for PcSpecialStateSwitchSuccess0104 {
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

/// `sP_FE2CL_PC_SPECIAL_STATE_CHANGE` (`#pragma pack(4)`, 8 bytes, packet ID `0x310000c4`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSpecialStateChange0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iReqSpecialStateFlag` at offset 4.
    pub req_special_state_flag: i8,
    /// `iSpecialState` at offset 5.
    pub special_state: i8,
}

impl PcSpecialStateChange0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.req_special_state_flag.to_le_bytes());
        write_prim(out, 5, &self.special_state.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            req_special_state_flag: i8::from_le_bytes(read_array(bytes, 4)),
            special_state: i8::from_le_bytes(read_array(bytes, 5)),
        }
    }
}

impl WirePayload for PcSpecialStateChange0104 {
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

/// `sP_FE2CL_GM_REP_PC_SET_VALUE` (`#pragma pack(4)`, 12 bytes, packet ID `0x310000c5`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GmPcSetValueReply0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iSetValueType` at offset 4.
    pub set_value_type: i32,
    /// `iSetValue` at offset 8.
    pub set_value: i32,
}

impl GmPcSetValueReply0104 {
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

impl WirePayload for GmPcSetValueReply0104 {
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
