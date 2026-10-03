use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pc2pcPair0104 {
    pub from_pc_id: i32,
    pub to_pc_id: i32,
}

impl Pc2pcPair0104 {
    pub fn new(from_pc_id: i32, to_pc_id: i32) -> Result<Self, Pc2pcIdentityError0104> {
        if from_pc_id <= 0 || to_pc_id <= 0 {
            return Err(Pc2pcIdentityError0104::NonPositiveParticipant {
                from_pc_id,
                to_pc_id,
            });
        }
        if from_pc_id == to_pc_id {
            return Err(Pc2pcIdentityError0104::SameParticipant { pc_id: from_pc_id });
        }
        Ok(Self {
            from_pc_id,
            to_pc_id,
        })
    }

    #[must_use]
    pub const fn contains(self, pc_id: i32) -> bool {
        self.from_pc_id == pc_id || self.to_pc_id == pc_id
    }

    #[must_use]
    pub const fn other(self, pc_id: i32) -> Option<i32> {
        if pc_id == self.from_pc_id {
            Some(self.to_pc_id)
        } else if pc_id == self.to_pc_id {
            Some(self.from_pc_id)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcIdentityError0104 {
    NonPositiveParticipant {
        from_pc_id: i32,
        to_pc_id: i32,
    },
    SameParticipant {
        pc_id: i32,
    },
    LocalOutsidePair {
        pair: Pc2pcPair0104,
        local_pc_id: i32,
    },
    DirectionMismatch {
        local_pc_id: i32,
        expected: Pc2pcOfferDirection0104,
        actual: Pc2pcOfferDirection0104,
    },
}

impl fmt::Display for Pc2pcIdentityError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Pc2pc identity: {self:?}")
    }
}

impl Error for Pc2pcIdentityError0104 {}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pc2pcEnvelope0104 {
    pub pair: Pc2pcPair0104,
    pub requester_pc_id: i32,
}

impl Pc2pcEnvelope0104 {
    #[must_use]
    pub const fn local(identity: Pc2pcSessionIdentity0104) -> Self {
        Self {
            pair: identity.pair,
            requester_pc_id: identity.local_pc_id,
        }
    }
}

/// Exact `#pragma pack(4)` twelve-byte offer-family request.
///
/// The clean server keeps the original ordered `from/to` pair for the whole
/// invitation. `requester_pc_id` identifies the participant issuing this
/// particular transition and must therefore remain inside that pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pc2pcOfferRequest0104 {
    pub kind: Pc2pcOfferRequestKind0104,
    pub envelope: Pc2pcEnvelope0104,
}

impl Pc2pcOfferRequest0104 {
    pub fn outgoing_offer(
        local_pc_id: i32,
        remote_pc_id: i32,
    ) -> Result<(Pc2pcSessionIdentity0104, Self), Pc2pcIdentityError0104> {
        let pair = Pc2pcPair0104::new(local_pc_id, remote_pc_id)?;
        let identity =
            Pc2pcSessionIdentity0104::new(pair, local_pc_id, Pc2pcOfferDirection0104::Outgoing)?;
        Ok((
            identity,
            Self {
                kind: Pc2pcOfferRequestKind0104::Offer,
                envelope: Pc2pcEnvelope0104::local(identity),
            },
        ))
    }

    pub fn for_session(
        identity: Pc2pcSessionIdentity0104,
        kind: Pc2pcOfferRequestKind0104,
    ) -> Self {
        Self {
            kind,
            envelope: Pc2pcEnvelope0104::local(identity),
        }
    }

    pub fn encode_registered(
        self,
    ) -> Result<RegisteredGameplayRequest0104, Pc2pcOfferRequestCodecError0104> {
        if !self.envelope.pair.contains(self.envelope.requester_pc_id) {
            return Err(Pc2pcOfferRequestCodecError0104::RequesterOutsidePair {
                pair: self.envelope.pair,
                requester_pc_id: self.envelope.requester_pc_id,
            });
        }
        let mut payload = vec![0; 12];
        payload[0..4].copy_from_slice(&self.envelope.requester_pc_id.to_le_bytes());
        payload[4..8].copy_from_slice(&self.envelope.pair.from_pc_id.to_le_bytes());
        payload[8..12].copy_from_slice(&self.envelope.pair.to_pc_id.to_le_bytes());
        Ok(RegisteredGameplayRequest0104::new(
            self.kind.packet_id(),
            payload,
        )?)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pc2pcOfferReply0104 {
    pub envelope: Pc2pcEnvelope0104,
    pub kind: Pc2pcOfferReplyKind0104,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcPendingOffer0104 {
    Outgoing(Pc2pcSessionIdentity0104),
    Incoming(Pc2pcSessionIdentity0104),
}

impl Pc2pcPendingOffer0104 {
    #[must_use]
    pub const fn identity(self) -> Pc2pcSessionIdentity0104 {
        match self {
            Self::Outgoing(identity) | Self::Incoming(identity) => identity,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct Pc2pcOfferRuntime0104 {
    pub(super) pending: Option<Pc2pcPendingOffer0104>,
    pub(super) accepted: Option<Pc2pcSessionIdentity0104>,
    pub(super) queued_transition: Option<Pc2pcOfferRequestKind0104>,
    pub(super) outbox: VecDeque<Pc2pcOfferRequest0104>,
    pub(super) last_abort_code: Option<i16>,
}

impl Pc2pcOfferRuntime0104 {
    #[must_use]
    pub const fn pending(&self) -> Option<Pc2pcPendingOffer0104> {
        self.pending
    }

    #[must_use]
    pub const fn accepted(&self) -> Option<Pc2pcSessionIdentity0104> {
        self.accepted
    }

    pub fn take_accepted(&mut self) -> Option<Pc2pcSessionIdentity0104> {
        self.accepted.take()
    }

    #[must_use]
    pub const fn last_abort_code(&self) -> Option<i16> {
        self.last_abort_code
    }

    pub fn pop_request(&mut self) -> Option<Pc2pcOfferRequest0104> {
        self.outbox.pop_front()
    }

    pub fn request_outgoing(
        &mut self,
        local_pc_id: i32,
        remote_pc_id: i32,
    ) -> Result<Pc2pcOfferRequest0104, Pc2pcOfferFlowError0104> {
        self.ensure_idle()?;
        let (identity, request) = Pc2pcOfferRequest0104::outgoing_offer(local_pc_id, remote_pc_id)?;
        self.pending = Some(Pc2pcPendingOffer0104::Outgoing(identity));
        self.accepted = None;
        self.last_abort_code = None;
        self.outbox.push_back(request);
        Ok(request)
    }

    pub fn accept_incoming(&mut self) -> Result<Pc2pcOfferRequest0104, Pc2pcOfferFlowError0104> {
        let Some(Pc2pcPendingOffer0104::Incoming(identity)) = self.pending else {
            return Err(Pc2pcOfferFlowError0104::NoIncomingOffer);
        };
        self.ensure_transition_not_queued()?;
        let request =
            Pc2pcOfferRequest0104::for_session(identity, Pc2pcOfferRequestKind0104::Accept);
        self.queued_transition = Some(request.kind);
        self.outbox.push_back(request);
        Ok(request)
    }

    pub fn refuse_incoming(&mut self) -> Result<Pc2pcOfferRequest0104, Pc2pcOfferFlowError0104> {
        let Some(Pc2pcPendingOffer0104::Incoming(identity)) = self.pending else {
            return Err(Pc2pcOfferFlowError0104::NoIncomingOffer);
        };
        self.ensure_transition_not_queued()?;
        let request =
            Pc2pcOfferRequest0104::for_session(identity, Pc2pcOfferRequestKind0104::Refusal);
        self.pending = None;
        self.queued_transition = None;
        self.outbox.push_back(request);
        Ok(request)
    }

    pub fn cancel_outgoing(&mut self) -> Result<Pc2pcOfferRequest0104, Pc2pcOfferFlowError0104> {
        let Some(Pc2pcPendingOffer0104::Outgoing(identity)) = self.pending else {
            return Err(Pc2pcOfferFlowError0104::NoOutgoingOffer);
        };
        self.ensure_transition_not_queued()?;
        let request =
            Pc2pcOfferRequest0104::for_session(identity, Pc2pcOfferRequestKind0104::Cancel);
        // OpenFusion returns `P_FE2CL_REP_PC_TRADE_OFFER_CANCEL` to the
        // original `from` participant. Retain the pair until that response so
        // it remains strictly correlated instead of accepting a stray ACK.
        self.queued_transition = Some(request.kind);
        self.outbox.push_back(request);
        Ok(request)
    }

    pub fn apply_reply(
        &mut self,
        local_pc_id: i32,
        reply: Pc2pcOfferReply0104,
    ) -> Result<(), Pc2pcOfferFlowError0104> {
        let direction = if reply.envelope.pair.from_pc_id == local_pc_id {
            Pc2pcOfferDirection0104::Outgoing
        } else {
            Pc2pcOfferDirection0104::Incoming
        };
        let reply_identity =
            Pc2pcSessionIdentity0104::new(reply.envelope.pair, local_pc_id, direction)?;
        match reply.kind {
            Pc2pcOfferReplyKind0104::Offered => {
                if direction != Pc2pcOfferDirection0104::Incoming {
                    return Err(Pc2pcOfferFlowError0104::UnexpectedReply {
                        pending: self.pending,
                        kind: reply.kind,
                    });
                }
                self.ensure_idle()?;
                self.pending = Some(Pc2pcPendingOffer0104::Incoming(reply_identity));
                self.accepted = None;
                self.queued_transition = None;
                self.last_abort_code = None;
            }
            Pc2pcOfferReplyKind0104::Accepted => {
                self.require_matching_pending(reply_identity, reply.kind)?;
                self.pending = None;
                self.accepted = Some(reply_identity);
                self.queued_transition = None;
                self.last_abort_code = None;
            }
            Pc2pcOfferReplyKind0104::Cancelled | Pc2pcOfferReplyKind0104::Refused => {
                self.require_matching_pending(reply_identity, reply.kind)?;
                self.pending = None;
                self.accepted = None;
                self.queued_transition = None;
                self.last_abort_code = None;
            }
            Pc2pcOfferReplyKind0104::Aborted { error_code } => {
                self.require_matching_pending(reply_identity, reply.kind)?;
                self.pending = None;
                self.accepted = None;
                self.queued_transition = None;
                self.last_abort_code = Some(error_code);
            }
        }
        Ok(())
    }

    /// Applies one retained gameplay frame atomically. Known malformed or
    /// mismatched offer frames return an error without consuming the current
    /// pending invitation; unrelated frames return `Ok(false)`.
    pub fn apply_frame(
        &mut self,
        local_pc_id: i32,
        frame: &DecodedFrame,
    ) -> Result<bool, Pc2pcOfferIngressError0104> {
        let Some(reply) = decode_pc2pc_offer_frame_0104(frame)? else {
            return Ok(false);
        };
        self.apply_reply(local_pc_id, reply)?;
        Ok(true)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn ensure_idle(&self) -> Result<(), Pc2pcOfferFlowError0104> {
        if self.pending.is_some() || self.accepted.is_some() {
            return Err(Pc2pcOfferFlowError0104::Busy {
                pending: self.pending,
                accepted: self.accepted,
            });
        }
        Ok(())
    }

    pub(super) fn ensure_transition_not_queued(&self) -> Result<(), Pc2pcOfferFlowError0104> {
        if let Some(kind) = self.queued_transition {
            return Err(Pc2pcOfferFlowError0104::TransitionAlreadyQueued { kind });
        }
        Ok(())
    }

    pub(super) fn require_matching_pending(
        &self,
        reply: Pc2pcSessionIdentity0104,
        kind: Pc2pcOfferReplyKind0104,
    ) -> Result<Pc2pcSessionIdentity0104, Pc2pcOfferFlowError0104> {
        let Some(pending) = self.pending else {
            return Err(Pc2pcOfferFlowError0104::UnexpectedReply {
                pending: None,
                kind,
            });
        };
        let expected = pending.identity();
        if expected.pair != reply.pair || expected.local_pc_id != reply.local_pc_id {
            return Err(Pc2pcOfferFlowError0104::PairMismatch {
                expected: expected.pair,
                actual: reply.pair,
            });
        }
        Ok(expected)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcOfferFlowError0104 {
    Identity(Pc2pcIdentityError0104),
    Busy {
        pending: Option<Pc2pcPendingOffer0104>,
        accepted: Option<Pc2pcSessionIdentity0104>,
    },
    NoIncomingOffer,
    NoOutgoingOffer,
    TransitionAlreadyQueued {
        kind: Pc2pcOfferRequestKind0104,
    },
    PairMismatch {
        expected: Pc2pcPair0104,
        actual: Pc2pcPair0104,
    },
    UnexpectedReply {
        pending: Option<Pc2pcPendingOffer0104>,
        kind: Pc2pcOfferReplyKind0104,
    },
}

impl fmt::Display for Pc2pcOfferFlowError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "trade-offer flow rejected: {self:?}")
    }
}

impl Error for Pc2pcOfferFlowError0104 {}

impl From<Pc2pcIdentityError0104> for Pc2pcOfferFlowError0104 {
    fn from(value: Pc2pcIdentityError0104) -> Self {
        Self::Identity(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcOfferIngressError0104 {
    Frame(Pc2pcOfferFrameError0104),
    Flow(Pc2pcOfferFlowError0104),
}

impl fmt::Display for Pc2pcOfferIngressError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "trade-offer ingress rejected: {self:?}")
    }
}

impl Error for Pc2pcOfferIngressError0104 {}

impl From<Pc2pcOfferFrameError0104> for Pc2pcOfferIngressError0104 {
    fn from(value: Pc2pcOfferFrameError0104) -> Self {
        Self::Frame(value)
    }
}

impl From<Pc2pcOfferFlowError0104> for Pc2pcOfferIngressError0104 {
    fn from(value: Pc2pcOfferFlowError0104) -> Self {
        Self::Flow(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcParticipant0104 {
    Local,
    Remote,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcParticipantNames0104 {
    pub local: String,
    pub remote: String,
}

impl Pc2pcParticipantNames0104 {
    pub fn new(
        local: impl Into<String>,
        remote: impl Into<String>,
    ) -> Result<Self, Pc2pcSnapshotError0104> {
        let local = local.into();
        let remote = remote.into();
        if local.trim().is_empty() {
            return Err(Pc2pcSnapshotError0104::EmptyParticipantName {
                participant: Pc2pcParticipant0104::Local,
            });
        }
        if remote.trim().is_empty() {
            return Err(Pc2pcSnapshotError0104::EmptyParticipantName {
                participant: Pc2pcParticipant0104::Remote,
            });
        }
        Ok(Self { local, remote })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Pc2pcTradeItem0104 {
    pub item_type: i16,
    pub item_id: i16,
    pub option: i32,
    pub inventory_slot: i32,
    pub offer_slot: i32,
}

impl Pc2pcTradeItem0104 {
    pub const SIZE: usize = 16;

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            item_type: 0,
            item_id: 0,
            option: 0,
            inventory_slot: 0,
            offer_slot: 0,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.item_id <= 0
    }

    #[must_use]
    pub const fn as_item_base(self) -> ItemBase0104 {
        ItemBase0104 {
            item_type: self.item_type,
            item_id: self.item_id,
            option: self.option,
            time_limit: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcTradeItemError0104 {
    MalformedIdentity { item_type: i16, item_id: i16 },
    InventorySlotOutOfBounds { inventory_slot: i32 },
    OfferSlotOutOfBounds { offer_slot: i32 },
    GeneralCountNotPositive { option: i32 },
}

impl fmt::Display for Pc2pcTradeItemError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Pc2pc trade item: {self:?}")
    }
}

impl Error for Pc2pcTradeItemError0104 {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcAuthoritativeSnapshot0104 {
    pub identity: Pc2pcSessionIdentity0104,
    pub names: Pc2pcParticipantNames0104,
    /// Inventory authority before any trade offer.
    pub(super) base_inventory: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
    /// Server-returned `InvenItem` availability while the trade is open.
    pub(super) available_inventory: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
    pub(super) equipment: [ItemBase0104; EQUIPMENT_SLOT_COUNT_0104],
    pub(super) local_wallet_taros: i32,
    pub(super) local_offer_taros: i32,
    pub(super) remote_offer_taros: i32,
    pub(super) local_offer: [Option<Pc2pcTradeItem0104>; PC2PC_OFFER_SLOT_COUNT],
    pub(super) remote_offer: [Option<Pc2pcTradeItem0104>; PC2PC_OFFER_SLOT_COUNT],
    pub(super) revision: u64,
}
