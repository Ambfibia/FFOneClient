// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_CL2FE_REQ_PC_TRADE_OFFER` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000022`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeOfferRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_OFFER_CANCEL` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000023`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferCancelRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferCancelRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeOfferCancelRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_OFFER_ACCEPT` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000024`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferAcceptRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferAcceptRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeOfferAcceptRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_OFFER_REFUSAL` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000025`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferRefusalRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferRefusalRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeOfferRefusalRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_OFFER_ABORT` (`#pragma pack(4)`, 16 bytes, packet ID `0x13000026`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferAbortRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i16,
}

impl PcTradeOfferAbortRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        write_prim(out, 12, &self.error_code.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            error_code: i16::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTradeOfferAbortRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_CONFIRM` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000027`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeConfirmRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeConfirmRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_CONFIRM_CANCEL` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000028`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmCancelRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeConfirmCancelRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeConfirmCancelRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_CONFIRM_ABORT` (`#pragma pack(4)`, 12 bytes, packet ID `0x13000029`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmAbortRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeConfirmAbortRequest0104 {
    pub const SIZE: usize = 12;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for PcTradeConfirmAbortRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_ITEM_REGISTER` (`#pragma pack(4)`, 28 bytes, packet ID `0x1300002a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeItemRegisterRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `Item` at offset 12.
    pub item: ItemTrade0104,
}

impl PcTradeItemRegisterRequest0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        self.item.write_into(&mut out[12..28]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            item: ItemTrade0104::read_from(&bytes[12..28]),
        }
    }
}

impl WirePayload for PcTradeItemRegisterRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_ITEM_UNREGISTER` (`#pragma pack(4)`, 28 bytes, packet ID `0x1300002b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeItemUnregisterRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `Item` at offset 12.
    pub item: ItemTrade0104,
}

impl PcTradeItemUnregisterRequest0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        self.item.write_into(&mut out[12..28]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            item: ItemTrade0104::read_from(&bytes[12..28]),
        }
    }
}

impl WirePayload for PcTradeItemUnregisterRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_CASH_REGISTER` (`#pragma pack(4)`, 16 bytes, packet ID `0x1300002c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeCashRegisterRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iCandy` at offset 12.
    pub candy: i32,
}

impl PcTradeCashRegisterRequest0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        write_prim(out, 12, &self.candy.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            candy: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTradeCashRegisterRequest0104 {
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

/// `sP_CL2FE_REQ_PC_TRADE_EMOTES_CHAT` (`#pragma pack(4)`, 276 bytes, packet ID `0x1300002d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeEmotesChatRequest0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `szFreeChat` at offset 12.
    pub free_chat: FixedUtf16<128>,
    /// `iEmoteCode` at offset 268.
    pub emote_code: i32,
    /// `iFreeChatUse` at offset 272.
    pub free_chat_use: i8,
}

impl PcTradeEmotesChatRequest0104 {
    pub const SIZE: usize = 276;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        write_utf16(out, 12, &self.free_chat);
        write_prim(out, 268, &self.emote_code.to_le_bytes());
        write_prim(out, 272, &self.free_chat_use.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            free_chat: read_utf16(bytes, 12),
            emote_code: i32::from_le_bytes(read_array(bytes, 268)),
            free_chat_use: i8::from_le_bytes(read_array(bytes, 272)),
        }
    }
}

impl WirePayload for PcTradeEmotesChatRequest0104 {
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

/// `sP_CL2FE_REQ_PC_BANK_OPEN` (`#pragma pack(4)`, 8 bytes, packet ID `0x1300002e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
///
/// OpenFusion divergence: the pinned `structs/0104.hpp` declares this struct as 4 bytes. Wiring code must choose the server-compatible length explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBankOpenRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iNPC_ID` at offset 4.
    pub npc_id: i32,
}

impl PcBankOpenRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.npc_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            npc_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcBankOpenRequest0104 {
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

/// `sP_CL2FE_REQ_PC_BANK_CLOSE` (`#pragma pack(4)`, 4 bytes, packet ID `0x1300002f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcBankCloseRequest0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
}

impl PcBankCloseRequest0104 {
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

impl WirePayload for PcBankCloseRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VENDOR_START` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000030`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorStartRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iVendorID` at offset 4.
    pub vendor_id: i32,
}

impl PcVendorStartRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.vendor_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            vendor_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcVendorStartRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE` (`#pragma pack(4)`, 8 bytes, packet ID `0x13000031`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorTableUpdateRequest0104 {
    /// `iNPC_ID` at offset 0.
    pub npc_id: i32,
    /// `iVendorID` at offset 4.
    pub vendor_id: i32,
}

impl PcVendorTableUpdateRequest0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.npc_id.to_le_bytes());
        write_prim(out, 4, &self.vendor_id.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            npc_id: i32::from_le_bytes(read_array(bytes, 0)),
            vendor_id: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for PcVendorTableUpdateRequest0104 {
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

/// `sP_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY` (`#pragma pack(4)`, 28 bytes, packet ID `0x13000032`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcVendorItemRestoreBuyRequest0104 {
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

impl PcVendorItemRestoreBuyRequest0104 {
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

impl WirePayload for PcVendorItemRestoreBuyRequest0104 {
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
