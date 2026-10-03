use super::*;

pub const PC2PC_TRADE_OFFER_REQUEST_PACKET_ID_0104: u32 = 0x1300_0022;

pub const PC2PC_TRADE_OFFER_CANCEL_REQUEST_PACKET_ID_0104: u32 = 0x1300_0023;

pub const PC2PC_TRADE_OFFER_ACCEPT_REQUEST_PACKET_ID_0104: u32 = 0x1300_0024;

pub const PC2PC_TRADE_OFFER_REFUSAL_REQUEST_PACKET_ID_0104: u32 = 0x1300_0025;

pub const PC2PC_TRADE_OFFER_RESPONSE_PACKET_ID_0104: u32 = 0x3100_0042;

pub const PC2PC_TRADE_OFFER_CANCEL_RESPONSE_PACKET_ID_0104: u32 = 0x3100_0043;

pub const PC2PC_TRADE_OFFER_SUCCESS_RESPONSE_PACKET_ID_0104: u32 = 0x3100_0044;

pub const PC2PC_TRADE_OFFER_REFUSAL_RESPONSE_PACKET_ID_0104: u32 = 0x3100_0045;

pub const PC2PC_TRADE_OFFER_ABORT_RESPONSE_PACKET_ID_0104: u32 = 0x3100_0046;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcOfferRequestCodecError0104 {
    RequesterOutsidePair {
        pair: Pc2pcPair0104,
        requester_pc_id: i32,
    },
    Registry(RegisteredGameplayRequestError0104),
}

impl fmt::Display for Pc2pcOfferRequestCodecError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "trade-offer request encoding failed: {self:?}")
    }
}

impl Error for Pc2pcOfferRequestCodecError0104 {}

impl From<RegisteredGameplayRequestError0104> for Pc2pcOfferRequestCodecError0104 {
    fn from(value: RegisteredGameplayRequestError0104) -> Self {
        Self::Registry(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcOfferFrameError0104 {
    Payload(PayloadError),
    Identity(Pc2pcIdentityError0104),
    RequesterOutsidePair {
        pair: Pc2pcPair0104,
        requester_pc_id: i32,
    },
}

impl fmt::Display for Pc2pcOfferFrameError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "trade-offer reply decoding failed: {self:?}")
    }
}

impl Error for Pc2pcOfferFrameError0104 {}

impl From<Pc2pcIdentityError0104> for Pc2pcOfferFrameError0104 {
    fn from(value: Pc2pcIdentityError0104) -> Self {
        Self::Identity(value)
    }
}

/// Strictly decodes the complete server offer family while leaving unrelated
/// gameplay frames untouched.
pub fn decode_pc2pc_offer_frame_0104(
    frame: &DecodedFrame,
) -> Result<Option<Pc2pcOfferReply0104>, Pc2pcOfferFrameError0104> {
    let (expected_size, kind) = match frame.packet_type {
        PC2PC_TRADE_OFFER_RESPONSE_PACKET_ID_0104 => (12, Pc2pcOfferReplyKind0104::Offered),
        PC2PC_TRADE_OFFER_CANCEL_RESPONSE_PACKET_ID_0104 => {
            (12, Pc2pcOfferReplyKind0104::Cancelled)
        }
        PC2PC_TRADE_OFFER_SUCCESS_RESPONSE_PACKET_ID_0104 => {
            (12, Pc2pcOfferReplyKind0104::Accepted)
        }
        PC2PC_TRADE_OFFER_REFUSAL_RESPONSE_PACKET_ID_0104 => (12, Pc2pcOfferReplyKind0104::Refused),
        PC2PC_TRADE_OFFER_ABORT_RESPONSE_PACKET_ID_0104 => {
            if frame.payload.len() != 16 {
                return Err(Pc2pcOfferFrameError0104::Payload(PayloadError::WrongSize {
                    expected: 16,
                    actual: frame.payload.len(),
                }));
            }
            let error_code = i16::from_le_bytes([frame.payload[12], frame.payload[13]]);
            (16, Pc2pcOfferReplyKind0104::Aborted { error_code })
        }
        _ => return Ok(None),
    };
    if frame.payload.len() != expected_size {
        return Err(Pc2pcOfferFrameError0104::Payload(PayloadError::WrongSize {
            expected: expected_size,
            actual: frame.payload.len(),
        }));
    }
    let requester_pc_id = i32::from_le_bytes(frame.payload[0..4].try_into().unwrap());
    let pair = Pc2pcPair0104::new(
        i32::from_le_bytes(frame.payload[4..8].try_into().unwrap()),
        i32::from_le_bytes(frame.payload[8..12].try_into().unwrap()),
    )?;
    if !pair.contains(requester_pc_id) {
        return Err(Pc2pcOfferFrameError0104::RequesterOutsidePair {
            pair,
            requester_pc_id,
        });
    }
    Ok(Some(Pc2pcOfferReply0104 {
        envelope: Pc2pcEnvelope0104 {
            pair,
            requester_pc_id,
        },
        kind,
    }))
}

pub(super) fn encode_trade_item(item: Pc2pcTradeItem0104, payload: &mut [u8]) {
    debug_assert_eq!(payload.len(), Pc2pcTradeItem0104::SIZE);
    payload[0..2].copy_from_slice(&item.item_type.to_le_bytes());
    payload[2..4].copy_from_slice(&item.item_id.to_le_bytes());
    payload[4..8].copy_from_slice(&item.option.to_le_bytes());
    payload[8..12].copy_from_slice(&item.inventory_slot.to_le_bytes());
    payload[12..16].copy_from_slice(&item.offer_slot.to_le_bytes());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pc2pcRequestCodecError0104 {
    Payload(PayloadError),
    Registry(RegisteredGameplayRequestError0104),
}

impl fmt::Display for Pc2pcRequestCodecError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Payload(error) => write!(formatter, "trade request payload failed: {error}"),
            Self::Registry(error) => write!(formatter, "trade request ABI failed: {error}"),
        }
    }
}

impl Error for Pc2pcRequestCodecError0104 {}

impl From<PayloadError> for Pc2pcRequestCodecError0104 {
    fn from(value: PayloadError) -> Self {
        Self::Payload(value)
    }
}

impl From<RegisteredGameplayRequestError0104> for Pc2pcRequestCodecError0104 {
    fn from(value: RegisteredGameplayRequestError0104) -> Self {
        Self::Registry(value)
    }
}

pub(super) fn bind_offer_frame(
    image: Option<Mut<ImageNode>>,
    slot: &Pc2pcOfferSlotProjection0104,
    assets: &Pc2pcUiAssets,
) {
    let Some(mut image) = image else {
        return;
    };
    image.image = assets.image(match (&slot.item, slot.equip_validation) {
        (None, _) => Pc2pcStaticAssetRole::SlotEmpty,
        (Some(_), Pc2pcEquipValidation::Rejected | Pc2pcEquipValidation::Unverified) => {
            Pc2pcStaticAssetRole::OfferRejected
        }
        (Some(_), _) => Pc2pcStaticAssetRole::SlotOccupied,
    });
}
