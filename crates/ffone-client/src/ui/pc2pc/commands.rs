use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcOfferRequestKind0104 {
    Offer,
    Cancel,
    Accept,
    Refusal,
}

impl Pc2pcOfferRequestKind0104 {
    #[must_use]
    pub const fn packet_id(self) -> u32 {
        match self {
            Self::Offer => PC2PC_TRADE_OFFER_REQUEST_PACKET_ID_0104,
            Self::Cancel => PC2PC_TRADE_OFFER_CANCEL_REQUEST_PACKET_ID_0104,
            Self::Accept => PC2PC_TRADE_OFFER_ACCEPT_REQUEST_PACKET_ID_0104,
            Self::Refusal => PC2PC_TRADE_OFFER_REFUSAL_REQUEST_PACKET_ID_0104,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcOfferReplyKind0104 {
    Offered,
    Cancelled,
    Accepted,
    Refused,
    Aborted { error_code: i16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcRequestFailureKind0104 {
    Cancel,
    RegisterItem,
    UnregisterItem,
    RegisterCash,
    Confirm,
    Chat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcActionError0104 {
    NoSession,
    PhaseBlocked { phase: Pc2pcLifecyclePhase },
    ModalBlocked,
    RequestPending,
    InventorySlotOutOfBounds { slot: usize },
    OfferSlotOutOfBounds { slot: usize },
    EmptyInventorySlot { slot: usize },
    OccupiedOfferSlot { slot: usize },
    EmptyOfferSlot { slot: usize },
    GeneralCountOutOfRange { requested: i32, available: i32 },
    TarosOutOfRange { requested: i32, wallet: i32 },
    AlreadySubmitted,
    PortraitBackendUnavailable,
    FreeChatBackendUnavailable,
    MenuChatBackendUnavailable,
    NumericPopupBackendUnavailable,
    FreeChatSpecialState,
    EmptyChat,
    ChatTooLong { characters: usize, maximum: usize },
    CheatCommandBlocked { command: String },
}

impl fmt::Display for Pc2pcActionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pc2pc action rejected: {self:?}")
    }
}

impl Error for Pc2pcActionError0104 {}

pub(super) fn pending_request(
    state: &Pc2pcUiState,
) -> Result<Pc2pcPendingRequest0104, Pc2pcCorrelationError0104> {
    state
        .pending
        .as_ref()
        .cloned()
        .ok_or(Pc2pcCorrelationError0104::MissingPending {
            expected: Pc2pcRequestFailureKind0104::Confirm,
        })
}
