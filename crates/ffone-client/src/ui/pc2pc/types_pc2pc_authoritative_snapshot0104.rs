use super::*;

impl Pc2pcAuthoritativeSnapshot0104 {
    pub fn from_accepted_trade(
        identity: Pc2pcSessionIdentity0104,
        names: Pc2pcParticipantNames0104,
        local_wallet_taros: i32,
        inventory: &InventoryRuntime0104,
    ) -> Result<Self, Pc2pcSnapshotError0104> {
        if inventory.owner_pc_id() != identity.local_pc_id {
            return Err(Pc2pcSnapshotError0104::OwnerMismatch {
                expected_pc_id: identity.local_pc_id,
                inventory_pc_id: inventory.owner_pc_id(),
            });
        }
        if local_wallet_taros < 0 {
            return Err(Pc2pcSnapshotError0104::NegativeTaros {
                value: local_wallet_taros,
            });
        }
        Ok(Self {
            identity,
            names,
            base_inventory: *inventory.inventory(),
            available_inventory: *inventory.inventory(),
            equipment: *inventory.equipment(),
            local_wallet_taros,
            local_offer_taros: 0,
            remote_offer_taros: 0,
            local_offer: [None; PC2PC_OFFER_SLOT_COUNT],
            remote_offer: [None; PC2PC_OFFER_SLOT_COUNT],
            revision: 0,
        })
    }

    #[must_use]
    pub const fn base_inventory(&self) -> &[ItemBase0104; INVENTORY_SLOT_COUNT_0104] {
        &self.base_inventory
    }

    #[must_use]
    pub const fn available_inventory(&self) -> &[ItemBase0104; INVENTORY_SLOT_COUNT_0104] {
        &self.available_inventory
    }

    #[must_use]
    pub const fn equipment(&self) -> &[ItemBase0104; EQUIPMENT_SLOT_COUNT_0104] {
        &self.equipment
    }

    #[must_use]
    pub const fn local_wallet_taros(&self) -> i32 {
        self.local_wallet_taros
    }

    #[must_use]
    pub const fn local_offer_taros(&self) -> i32 {
        self.local_offer_taros
    }

    #[must_use]
    pub const fn remote_offer_taros(&self) -> i32 {
        self.remote_offer_taros
    }

    #[must_use]
    pub const fn local_offer(&self) -> &[Option<Pc2pcTradeItem0104>; PC2PC_OFFER_SLOT_COUNT] {
        &self.local_offer
    }

    #[must_use]
    pub const fn remote_offer(&self) -> &[Option<Pc2pcTradeItem0104>; PC2PC_OFFER_SLOT_COUNT] {
        &self.remote_offer
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn inventory_runtime_from_available(&self) -> InventoryRuntime0104 {
        let mut load = PcLoadData0104::zeroed();
        for (slot, item) in self.equipment.iter().copied().enumerate() {
            write_item_base(
                load.as_bytes_mut(),
                PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE,
                item,
            );
        }
        for (slot, item) in self.available_inventory.iter().copied().enumerate() {
            write_item_base(
                load.as_bytes_mut(),
                PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
                item,
            );
        }
        InventoryRuntime0104::from_pc_load(self.identity.local_pc_id, &load)
    }

    pub(super) fn reset_offer_readiness_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub(super) fn apply_register_success(
        &mut self,
        participant: Pc2pcParticipant0104,
        trade_item: Pc2pcTradeItem0104,
        inventory_item: Pc2pcTradeItem0104,
    ) -> Result<(), Pc2pcAuthorityError0104> {
        validate_trade_item(trade_item, false).map_err(Pc2pcAuthorityError0104::TradeItem)?;
        let slot = trade_item.offer_slot as usize;
        match participant {
            Pc2pcParticipant0104::Local => {
                if inventory_item.inventory_slot != trade_item.inventory_slot {
                    return Err(Pc2pcAuthorityError0104::InventorySlotMismatch {
                        expected: trade_item.inventory_slot,
                        actual: inventory_item.inventory_slot,
                    });
                }
                let inventory_slot = trade_item.inventory_slot as usize;
                let mut post = inventory_item.as_item_base();
                if trade_item.item_type == 7 {
                    if post.option == 0 {
                        post.item_id = 0;
                    }
                } else {
                    // Exact clean `cnTrade`: non-stack offers hide the source
                    // item but retain the response type/option sentinel.
                    post.item_id = 0;
                }
                self.available_inventory[inventory_slot] = post;
                self.local_offer[slot] = Some(trade_item);
            }
            Pc2pcParticipant0104::Remote => {
                self.remote_offer[slot] = Some(trade_item);
            }
        }
        self.reset_offer_readiness_revision();
        Ok(())
    }

    pub(super) fn apply_unregister_success(
        &mut self,
        participant: Pc2pcParticipant0104,
        trade_item: Pc2pcTradeItem0104,
        inventory_item: Pc2pcTradeItem0104,
    ) -> Result<(), Pc2pcAuthorityError0104> {
        validate_trade_item(trade_item, false).map_err(Pc2pcAuthorityError0104::TradeItem)?;
        let slot = trade_item.offer_slot as usize;
        let offered = match participant {
            Pc2pcParticipant0104::Local => self.local_offer[slot],
            Pc2pcParticipant0104::Remote => self.remote_offer[slot],
        };
        if offered != Some(trade_item) {
            return Err(Pc2pcAuthorityError0104::OfferSlotMismatch {
                participant,
                slot,
                expected: offered,
                actual: trade_item,
            });
        }
        match participant {
            Pc2pcParticipant0104::Local => {
                if inventory_item.inventory_slot != trade_item.inventory_slot {
                    return Err(Pc2pcAuthorityError0104::InventorySlotMismatch {
                        expected: trade_item.inventory_slot,
                        actual: inventory_item.inventory_slot,
                    });
                }
                self.available_inventory[trade_item.inventory_slot as usize] =
                    inventory_item.as_item_base();
                self.local_offer[slot] = None;
            }
            Pc2pcParticipant0104::Remote => self.remote_offer[slot] = None,
        }
        self.reset_offer_readiness_revision();
        Ok(())
    }

    pub(super) fn apply_cash_success(
        &mut self,
        participant: Pc2pcParticipant0104,
        taros: i32,
    ) -> Result<(), Pc2pcAuthorityError0104> {
        if taros < 0 {
            return Err(Pc2pcAuthorityError0104::NegativeTaros { value: taros });
        }
        if participant == Pc2pcParticipant0104::Local && taros > self.local_wallet_taros {
            return Err(Pc2pcAuthorityError0104::TarosExceedsWallet {
                offered: taros,
                wallet: self.local_wallet_taros,
            });
        }
        match participant {
            Pc2pcParticipant0104::Local => self.local_offer_taros = taros,
            Pc2pcParticipant0104::Remote => self.remote_offer_taros = taros,
        }
        self.reset_offer_readiness_revision();
        Ok(())
    }

    pub(super) fn apply_final_success(
        &mut self,
        received: [Pc2pcTradeItem0104; PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
        item_stay: [Pc2pcTradeItem0104; PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
        wallet_taros: i32,
    ) -> Result<Pc2pcFinalCommitReceipt0104, Pc2pcAuthorityError0104> {
        if wallet_taros < 0 {
            return Err(Pc2pcAuthorityError0104::NegativeTaros {
                value: wallet_taros,
            });
        }
        correlate_local_offer_stay(&self.local_offer, &item_stay)?;

        let mut committed = self.available_inventory;
        let mut targets = [false; INVENTORY_SLOT_COUNT_0104];
        let mut received_count = 0usize;
        for item in received {
            if item.is_empty() {
                continue;
            }
            validate_trade_item(item, false).map_err(Pc2pcAuthorityError0104::TradeItem)?;
            let target = item.inventory_slot as usize;
            if targets[target] {
                return Err(Pc2pcAuthorityError0104::DuplicateFinalTarget {
                    inventory_slot: target,
                });
            }
            if !InventoryRuntime0104::item_is_empty(committed[target]) {
                return Err(Pc2pcAuthorityError0104::OccupiedFinalTarget {
                    inventory_slot: target,
                    existing: committed[target],
                    received: item,
                });
            }
            targets[target] = true;
            committed[target] = item.as_item_base();
            received_count += 1;
        }

        self.base_inventory = committed;
        self.available_inventory = committed;
        self.local_wallet_taros = wallet_taros;
        self.local_offer_taros = 0;
        self.remote_offer_taros = 0;
        self.local_offer = [None; PC2PC_OFFER_SLOT_COUNT];
        self.remote_offer = [None; PC2PC_OFFER_SLOT_COUNT];
        self.revision = self.revision.saturating_add(1);
        Ok(Pc2pcFinalCommitReceipt0104 {
            received_item_count: received_count,
            wallet_taros,
            revision: self.revision,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcSnapshotError0104 {
    OwnerMismatch {
        expected_pc_id: i32,
        inventory_pc_id: i32,
    },
    NegativeTaros {
        value: i32,
    },
    EmptyParticipantName {
        participant: Pc2pcParticipant0104,
    },
}

impl fmt::Display for Pc2pcSnapshotError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Pc2pc authoritative snapshot: {self:?}")
    }
}

impl Error for Pc2pcSnapshotError0104 {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcAuthorityError0104 {
    TradeItem(Pc2pcTradeItemError0104),
    InventorySlotMismatch {
        expected: i32,
        actual: i32,
    },
    OfferSlotMismatch {
        participant: Pc2pcParticipant0104,
        slot: usize,
        expected: Option<Pc2pcTradeItem0104>,
        actual: Pc2pcTradeItem0104,
    },
    NegativeTaros {
        value: i32,
    },
    TarosExceedsWallet {
        offered: i32,
        wallet: i32,
    },
    FinalStayMissingLocalOffer {
        slot: usize,
        expected: Pc2pcTradeItem0104,
    },
    UnexpectedFinalStay {
        actual: Pc2pcTradeItem0104,
    },
    DuplicateFinalTarget {
        inventory_slot: usize,
    },
    OccupiedFinalTarget {
        inventory_slot: usize,
        existing: ItemBase0104,
        received: Pc2pcTradeItem0104,
    },
}

impl fmt::Display for Pc2pcAuthorityError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rejected Pc2pc authority mutation: {self:?}")
    }
}

impl Error for Pc2pcAuthorityError0104 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pc2pcFinalCommitReceipt0104 {
    pub received_item_count: usize,
    pub wallet_taros: i32,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Pc2pcLifecyclePhase {
    #[default]
    Hidden,
    Opening,
    Trading,
    Cancelling,
    Completing,
    Completed,
    Cancelled,
    Failed,
}

impl Pc2pcLifecyclePhase {
    #[must_use]
    pub const fn renders_shell(self) -> bool {
        matches!(
            self,
            Self::Opening | Self::Trading | Self::Completing | Self::Failed
        )
    }

    #[must_use]
    pub const fn accepts_actions(self) -> bool {
        matches!(self, Self::Trading)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct Pc2pcBackendCapabilities {
    /// Exact live avatar cameras are not yet wired into this UI slice.
    pub portrait_backend: bool,
    /// Free-chat transport + special-state authority.
    pub free_chat_backend: bool,
    /// Menu-chat/emote hierarchy and transport.
    pub menu_chat_backend: bool,
    /// Exact legacy numeric `PopupControll` boundary for ADD TAROS.
    pub numeric_popup_backend: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pc2pcPortraitRef {
    pub(super) runtime_path: String,
}

impl Pc2pcPortraitRef {
    pub fn new(runtime_path: impl Into<String>) -> Result<Self, Pc2pcPortraitRefError> {
        let runtime_path = runtime_path.into();
        if runtime_path.trim().is_empty() {
            return Err(Pc2pcPortraitRefError::EmptyPath);
        }
        if runtime_path.contains('\\')
            || runtime_path.starts_with('/')
            || runtime_path.split('/').any(|part| part == "..")
        {
            return Err(Pc2pcPortraitRefError::UnsafePath { runtime_path });
        }
        Ok(Self { runtime_path })
    }

    #[must_use]
    pub fn runtime_path(&self) -> &str {
        &self.runtime_path
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct Pc2pcPortraitBindings {
    pub local: Option<Pc2pcPortraitRef>,
    pub remote: Option<Pc2pcPortraitRef>,
    /// `None` means special-state authority is unavailable and the badge is
    /// hidden, matching the fail-closed backend boundary.
    pub local_free_chat_allowed: Option<bool>,
    pub remote_free_chat_allowed: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcChatLine0104 {
    pub speaker_pc_id: i32,
    pub display_name: String,
    pub text: String,
    pub emote_code: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcConfirmIntent0104 {
    pub envelope: Pc2pcEnvelope0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcCancelIntent0104 {
    pub envelope: Pc2pcEnvelope0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcRegisterItemIntent0104 {
    pub envelope: Pc2pcEnvelope0104,
    pub item: Pc2pcTradeItem0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcUnregisterItemIntent0104 {
    pub envelope: Pc2pcEnvelope0104,
    pub item: Pc2pcTradeItem0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcRegisterCashIntent0104 {
    pub envelope: Pc2pcEnvelope0104,
    pub taros: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcChatIntent0104 {
    pub envelope: Pc2pcEnvelope0104,
    pub text: String,
    pub emote_code: i32,
    pub free_chat_use: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcIntent0104 {
    Confirm(Pc2pcConfirmIntent0104),
    Cancel(Pc2pcCancelIntent0104),
    RegisterItem(Pc2pcRegisterItemIntent0104),
    UnregisterItem(Pc2pcUnregisterItemIntent0104),
    RegisterCash(Pc2pcRegisterCashIntent0104),
    Chat(Pc2pcChatIntent0104),
}

impl Pc2pcIntent0104 {
    #[must_use]
    pub const fn packet_id(&self) -> u32 {
        match self {
            Self::Confirm(_) => 0x1300_0027,
            Self::Cancel(_) => 0x1300_0028,
            Self::RegisterItem(_) => 0x1300_002a,
            Self::UnregisterItem(_) => 0x1300_002b,
            Self::RegisterCash(_) => 0x1300_002c,
            Self::Chat(_) => 0x1300_002d,
        }
    }

    /// Encode the exact `#pragma pack(4)` 0104 trade request and pass it
    /// through the complete registered-handler ABI gate.
    pub fn encode_registered(
        &self,
    ) -> Result<RegisteredGameplayRequest0104, Pc2pcRequestCodecError0104> {
        let envelope = match self {
            Self::Confirm(intent) => intent.envelope,
            Self::Cancel(intent) => intent.envelope,
            Self::RegisterItem(intent) => intent.envelope,
            Self::UnregisterItem(intent) => intent.envelope,
            Self::RegisterCash(intent) => intent.envelope,
            Self::Chat(intent) => intent.envelope,
        };
        let mut payload = vec![
            0;
            match self {
                Self::Confirm(_) | Self::Cancel(_) => 12,
                Self::RegisterItem(_) | Self::UnregisterItem(_) => 28,
                Self::RegisterCash(_) => 16,
                Self::Chat(_) => 276,
            }
        ];
        payload[0..4].copy_from_slice(&envelope.requester_pc_id.to_le_bytes());
        payload[4..8].copy_from_slice(&envelope.pair.from_pc_id.to_le_bytes());
        payload[8..12].copy_from_slice(&envelope.pair.to_pc_id.to_le_bytes());

        match self {
            Self::RegisterItem(intent) => encode_trade_item(intent.item, &mut payload[12..28]),
            Self::UnregisterItem(intent) => encode_trade_item(intent.item, &mut payload[12..28]),
            Self::RegisterCash(intent) => {
                payload[12..16].copy_from_slice(&intent.taros.to_le_bytes());
            }
            Self::Chat(intent) => {
                let text = FixedUtf16::<128>::from_str(&intent.text)?;
                for (index, unit) in text.as_units().iter().enumerate() {
                    let offset = 12 + index * 2;
                    payload[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
                }
                payload[268..272].copy_from_slice(&intent.emote_code.to_le_bytes());
                payload[272] = u8::from(intent.free_chat_use);
            }
            Self::Confirm(_) | Self::Cancel(_) => {}
        }

        Ok(RegisteredGameplayRequest0104::new(
            self.packet_id(),
            payload,
        )?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcPendingRequest0104 {
    Confirm(Pc2pcConfirmIntent0104),
    Cancel(Pc2pcCancelIntent0104),
    RegisterItem(Pc2pcRegisterItemIntent0104),
    UnregisterItem(Pc2pcUnregisterItemIntent0104),
    RegisterCash(Pc2pcRegisterCashIntent0104),
    Chat(Pc2pcChatIntent0104),
}

impl From<Pc2pcIntent0104> for Pc2pcPendingRequest0104 {
    fn from(value: Pc2pcIntent0104) -> Self {
        match value {
            Pc2pcIntent0104::Confirm(value) => Self::Confirm(value),
            Pc2pcIntent0104::Cancel(value) => Self::Cancel(value),
            Pc2pcIntent0104::RegisterItem(value) => Self::RegisterItem(value),
            Pc2pcIntent0104::UnregisterItem(value) => Self::UnregisterItem(value),
            Pc2pcIntent0104::RegisterCash(value) => Self::RegisterCash(value),
            Pc2pcIntent0104::Chat(value) => Self::Chat(value),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct Pc2pcUiOutbox0104(pub VecDeque<Pc2pcIntent0104>);

impl Pc2pcUiOutbox0104 {
    pub fn pop_front(&mut self) -> Option<Pc2pcIntent0104> {
        self.0.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcServerOutcome0104 {
    Confirmed {
        envelope: Pc2pcEnvelope0104,
    },
    RegisterItemSuccess {
        envelope: Pc2pcEnvelope0104,
        trade_item: Pc2pcTradeItem0104,
        inventory_item: Pc2pcTradeItem0104,
    },
    UnregisterItemSuccess {
        envelope: Pc2pcEnvelope0104,
        trade_item: Pc2pcTradeItem0104,
        inventory_item: Pc2pcTradeItem0104,
    },
    RegisterCashSuccess {
        envelope: Pc2pcEnvelope0104,
        taros: i32,
    },
    RequestFailure {
        envelope: Pc2pcEnvelope0104,
        kind: Pc2pcRequestFailureKind0104,
        error_code: i32,
    },
    ConfirmSuccess {
        envelope: Pc2pcEnvelope0104,
        received: [Pc2pcTradeItem0104; PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
        item_stay: [Pc2pcTradeItem0104; PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
        wallet_taros: i32,
    },
    ConfirmFailure {
        envelope: Pc2pcEnvelope0104,
        error_code: i32,
    },
    SessionEnded {
        envelope: Pc2pcEnvelope0104,
        reason: Pc2pcSessionEndReason0104,
        error_code: Option<i32>,
    },
    ChatMessage {
        envelope: Pc2pcEnvelope0104,
        text: String,
        emote_code: i32,
    },
    ChatFailure {
        envelope: Pc2pcEnvelope0104,
        error_code: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcLastFailure0104 {
    Request {
        kind: Pc2pcRequestFailureKind0104,
        error_code: i32,
    },
    Confirm {
        error_code: i32,
    },
    Chat {
        error_code: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcCorrelationError0104 {
    PairMismatch {
        expected: Pc2pcPair0104,
        actual: Pc2pcPair0104,
    },
    RequesterOutsidePair {
        pair: Pc2pcPair0104,
        requester_pc_id: i32,
    },
    MissingPending {
        expected: Pc2pcRequestFailureKind0104,
    },
    PendingKindMismatch {
        pending: Pc2pcPendingRequest0104,
        outcome: Pc2pcRequestFailureKind0104,
    },
    RegisterItemMismatch {
        expected: Pc2pcTradeItem0104,
        actual: Pc2pcTradeItem0104,
    },
    UnregisterItemMismatch {
        expected: Pc2pcTradeItem0104,
        actual: Pc2pcTradeItem0104,
    },
    CashMismatch {
        expected: i32,
        actual: i32,
    },
    ChatMismatch {
        expected: Pc2pcChatIntent0104,
        actual_text: String,
        actual_emote_code: i32,
    },
    RemoteFailureNotCorrelatable,
    FinalWithoutLocalAcceptance,
    FinalWhileUnrelatedRequestPending {
        pending: Pc2pcPendingRequest0104,
    },
    ChatBackendUnavailable,
    Authority(Pc2pcAuthorityError0104),
}

impl fmt::Display for Pc2pcCorrelationError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pc2pc outcome correlation failed: {self:?}")
    }
}

impl Error for Pc2pcCorrelationError0104 {}

pub trait Pc2pcEquipEligibility {
    fn enable_equip(&self, item: ItemBase0104) -> Option<bool>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Pc2pcFailClosedEquipEligibility;

impl Pc2pcEquipEligibility for Pc2pcFailClosedEquipEligibility {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pc2pcOfferSlotProjection0104 {
    pub slot_index: usize,
    pub item: Option<Pc2pcTradeItem0104>,
    pub icon: UserEquipProjectedIcon,
    pub equip_validation: Pc2pcEquipValidation,
    pub count_label: Option<String>,
}

impl Pc2pcOfferSlotProjection0104 {
    pub(super) fn project(
        slot_index: usize,
        item: Option<Pc2pcTradeItem0104>,
        catalog: &impl UserEquipItemCatalog,
        eligibility: &impl Pc2pcEquipEligibility,
    ) -> Self {
        let Some(item) = item else {
            return Self {
                slot_index,
                item: None,
                icon: UserEquipProjectedIcon::Empty,
                equip_validation: Pc2pcEquipValidation::NotApplicable,
                count_label: None,
            };
        };
        let base = item.as_item_base();
        let icon = project_pc2pc_icon(base, catalog);
        let equip_validation = if (0..=6).contains(&item.item_type) || item.item_type == 10 {
            match eligibility.enable_equip(base) {
                Some(true) => Pc2pcEquipValidation::Allowed,
                Some(false) => Pc2pcEquipValidation::Rejected,
                None => Pc2pcEquipValidation::Unverified,
            }
        } else {
            Pc2pcEquipValidation::NotApplicable
        };
        Self {
            slot_index,
            item: Some(item),
            icon,
            equip_validation,
            count_label: (item.item_type == 7).then(|| item.option.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct Pc2pcUiModel0104 {
    pub state: Pc2pcUiState,
    pub snapshot: Option<Pc2pcAuthoritativeSnapshot0104>,
    pub projection: Pc2pcModeProjection0104,
    pub chat: VecDeque<Pc2pcChatLine0104>,
    pub outbox: Pc2pcUiOutbox0104,
}

impl Default for Pc2pcUiModel0104 {
    fn default() -> Self {
        Self {
            state: Pc2pcUiState::default(),
            snapshot: None,
            projection: Pc2pcModeProjection0104::default(),
            chat: VecDeque::new(),
            outbox: Pc2pcUiOutbox0104::default(),
        }
    }
}
