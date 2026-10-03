// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_PC_GRENADE_STYLE_HIT` (`#pragma pack(4)`, 24 bytes, packet ID `0x31000041`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcGrenadeStyleHit0104 {
    /// `iPC_ID` at offset 0.
    pub pc_id: i32,
    /// `iBulletID` at offset 4.
    pub bullet_id: i8,
    /// `Bullet` at offset 8.
    pub bullet: PcBullet0104,
    /// `iTargetCnt` at offset 20.
    pub target_cnt: i32,
}

impl PcGrenadeStyleHit0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.pc_id.to_le_bytes());
        write_prim(out, 4, &self.bullet_id.to_le_bytes());
        self.bullet.write_into(&mut out[8..20]);
        write_prim(out, 20, &self.target_cnt.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_id: i32::from_le_bytes(read_array(bytes, 0)),
            bullet_id: i8::from_le_bytes(read_array(bytes, 4)),
            bullet: PcBullet0104::read_from(&bytes[8..20]),
            target_cnt: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for PcGrenadeStyleHit0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_OFFER` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000042`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferReply0104 {
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

impl WirePayload for PcTradeOfferReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_OFFER_CANCEL` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000043`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferCancelReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferCancelReply0104 {
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

impl WirePayload for PcTradeOfferCancelReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_OFFER_SUCC` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000044`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferSuccess0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferSuccess0104 {
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

impl WirePayload for PcTradeOfferSuccess0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_OFFER_REFUSAL` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000045`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferRefusalReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeOfferRefusalReply0104 {
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

impl WirePayload for PcTradeOfferRefusalReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_OFFER_ABORT` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000046`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeOfferAbortReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i16,
}

impl PcTradeOfferAbortReply0104 {
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

impl WirePayload for PcTradeOfferAbortReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_CONFIRM` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000047`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeConfirmReply0104 {
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

impl WirePayload for PcTradeConfirmReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_CONFIRM_CANCEL` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000048`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmCancelReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeConfirmCancelReply0104 {
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

impl WirePayload for PcTradeConfirmCancelReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_CONFIRM_ABORT` (`#pragma pack(4)`, 12 bytes, packet ID `0x31000049`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmAbortReply0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
}

impl PcTradeConfirmAbortReply0104 {
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

impl WirePayload for PcTradeConfirmAbortReply0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_CONFIRM_SUCC` (`#pragma pack(4)`, 400 bytes, packet ID `0x3100004a`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmSuccess0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `Item` at offset 12.
    pub item: [ItemTrade0104; 12],
    /// `iCandy` at offset 204.
    pub candy: i32,
    /// `ItemStay` at offset 208.
    pub item_stay: [ItemTrade0104; 12],
}

impl PcTradeConfirmSuccess0104 {
    pub const SIZE: usize = 400;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        for (index, value) in self.item.iter().enumerate() {
            let start = 12 + index * 16;
            value.write_into(&mut out[start..start + 16]);
        }
        write_prim(out, 204, &self.candy.to_le_bytes());
        for (index, value) in self.item_stay.iter().enumerate() {
            let start = 208 + index * 16;
            value.write_into(&mut out[start..start + 16]);
        }
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            item: std::array::from_fn(|index| {
                let start = 12 + index * 16;
                ItemTrade0104::read_from(&bytes[start..start + 16])
            }),
            candy: i32::from_le_bytes(read_array(bytes, 204)),
            item_stay: std::array::from_fn(|index| {
                let start = 208 + index * 16;
                ItemTrade0104::read_from(&bytes[start..start + 16])
            }),
        }
    }
}

impl WirePayload for PcTradeConfirmSuccess0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_CONFIRM_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x3100004b`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeConfirmFailure0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl PcTradeConfirmFailure0104 {
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
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTradeConfirmFailure0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_ITEM_REGISTER_SUCC` (`#pragma pack(4)`, 44 bytes, packet ID `0x3100004c`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeItemRegisterSuccess0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `TradeItem` at offset 12.
    pub trade_item: ItemTrade0104,
    /// `InvenItem` at offset 28.
    pub inven_item: ItemTrade0104,
}

impl PcTradeItemRegisterSuccess0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        self.trade_item.write_into(&mut out[12..28]);
        self.inven_item.write_into(&mut out[28..44]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            trade_item: ItemTrade0104::read_from(&bytes[12..28]),
            inven_item: ItemTrade0104::read_from(&bytes[28..44]),
        }
    }
}

impl WirePayload for PcTradeItemRegisterSuccess0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_ITEM_REGISTER_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x3100004d`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeItemRegisterFailure0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl PcTradeItemRegisterFailure0104 {
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
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTradeItemRegisterFailure0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_ITEM_UNREGISTER_SUCC` (`#pragma pack(4)`, 44 bytes, packet ID `0x3100004e`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeItemUnregisterSuccess0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `TradeItem` at offset 12.
    pub trade_item: ItemTrade0104,
    /// `InvenItem` at offset 28.
    pub inven_item: ItemTrade0104,
}

impl PcTradeItemUnregisterSuccess0104 {
    pub const SIZE: usize = 44;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.id_request.to_le_bytes());
        write_prim(out, 4, &self.id_from.to_le_bytes());
        write_prim(out, 8, &self.id_to.to_le_bytes());
        self.trade_item.write_into(&mut out[12..28]);
        self.inven_item.write_into(&mut out[28..44]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            id_request: i32::from_le_bytes(read_array(bytes, 0)),
            id_from: i32::from_le_bytes(read_array(bytes, 4)),
            id_to: i32::from_le_bytes(read_array(bytes, 8)),
            trade_item: ItemTrade0104::read_from(&bytes[12..28]),
            inven_item: ItemTrade0104::read_from(&bytes[28..44]),
        }
    }
}

impl WirePayload for PcTradeItemUnregisterSuccess0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_ITEM_UNREGISTER_FAIL` (`#pragma pack(4)`, 16 bytes, packet ID `0x3100004f`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeItemUnregisterFailure0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iErrorCode` at offset 12.
    pub error_code: i32,
}

impl PcTradeItemUnregisterFailure0104 {
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
            error_code: i32::from_le_bytes(read_array(bytes, 12)),
        }
    }
}

impl WirePayload for PcTradeItemUnregisterFailure0104 {
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

/// `sP_FE2CL_REP_PC_TRADE_CASH_REGISTER_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x31000050`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PcTradeCashRegisterSuccess0104 {
    /// `iID_Request` at offset 0.
    pub id_request: i32,
    /// `iID_From` at offset 4.
    pub id_from: i32,
    /// `iID_To` at offset 8.
    pub id_to: i32,
    /// `iCandy` at offset 12.
    pub candy: i32,
}

impl PcTradeCashRegisterSuccess0104 {
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

impl WirePayload for PcTradeCashRegisterSuccess0104 {
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
