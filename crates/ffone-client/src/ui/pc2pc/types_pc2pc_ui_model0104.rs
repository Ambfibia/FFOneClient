use super::*;

impl Pc2pcUiModel0104 {
    pub fn begin_session(
        &mut self,
        snapshot: Pc2pcAuthoritativeSnapshot0104,
        catalog: &impl UserEquipItemCatalog,
        eligibility: &impl Pc2pcEquipEligibility,
    ) {
        self.projection =
            Pc2pcModeProjection0104::from_authoritative(&snapshot, catalog, eligibility);
        self.snapshot = Some(snapshot);
        self.chat.clear();
        self.outbox.0.clear();
        self.state = Pc2pcUiState {
            phase: Pc2pcLifecyclePhase::Opening,
            ..default()
        };
    }

    /// Available only after a fully correlated final server commit.
    pub fn completed_inventory(&self) -> Option<InventoryRuntime0104> {
        (self.state.phase == Pc2pcLifecyclePhase::Completed).then(|| self.snapshot.as_ref().map(|s| s.inventory_runtime_from_available())).flatten()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub fn identity(&self) -> Option<Pc2pcSessionIdentity0104> {
        self.snapshot.as_ref().map(|snapshot| snapshot.identity)
    }

    pub(super) fn ensure_action_gate(
        &self,
        modal: Pc2pcModalState,
    ) -> Result<&Pc2pcAuthoritativeSnapshot0104, Pc2pcActionError0104> {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or(Pc2pcActionError0104::NoSession)?;
        if !self.state.phase.accepts_actions() {
            return Err(Pc2pcActionError0104::PhaseBlocked {
                phase: self.state.phase,
            });
        }
        if !modal.controls_enabled() {
            return Err(Pc2pcActionError0104::ModalBlocked);
        }
        if self.state.pending.is_some() {
            return Err(Pc2pcActionError0104::RequestPending);
        }
        Ok(snapshot)
    }

    pub(super) fn queue_intent(&mut self, intent: Pc2pcIntent0104) {
        self.state.pending = Some(intent.clone().into());
        self.outbox.0.push_back(intent);
    }

    pub fn request_confirm(
        &mut self,
        modal: Pc2pcModalState,
    ) -> Result<Pc2pcConfirmIntent0104, Pc2pcActionError0104> {
        let snapshot = self.ensure_action_gate(modal)?;
        if self.state.local_visual_ready() && !self.state.remote_ready {
            return Err(Pc2pcActionError0104::AlreadySubmitted);
        }
        let intent = Pc2pcConfirmIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
        };
        self.queue_intent(Pc2pcIntent0104::Confirm(intent.clone()));
        if self.state.remote_ready {
            self.state.phase = Pc2pcLifecyclePhase::Completing;
        }
        Ok(intent)
    }

    pub fn request_register_item(
        &mut self,
        modal: Pc2pcModalState,
        inventory_slot: usize,
        offer_slot: usize,
        general_count: Option<i32>,
    ) -> Result<Pc2pcRegisterItemIntent0104, Pc2pcActionError0104> {
        let snapshot = self.ensure_action_gate(modal)?;
        if inventory_slot >= INVENTORY_SLOT_COUNT_0104 {
            return Err(Pc2pcActionError0104::InventorySlotOutOfBounds {
                slot: inventory_slot,
            });
        }
        if offer_slot >= PC2PC_OFFER_SLOT_COUNT {
            return Err(Pc2pcActionError0104::OfferSlotOutOfBounds { slot: offer_slot });
        }
        if snapshot.local_offer[offer_slot].is_some() {
            return Err(Pc2pcActionError0104::OccupiedOfferSlot { slot: offer_slot });
        }
        let item = snapshot.available_inventory[inventory_slot];
        if InventoryRuntime0104::item_is_empty(item) {
            return Err(Pc2pcActionError0104::EmptyInventorySlot {
                slot: inventory_slot,
            });
        }
        let option = if item.item_type == 7 {
            let requested = general_count.unwrap_or(item.option);
            if requested <= 0 || requested > item.option {
                return Err(Pc2pcActionError0104::GeneralCountOutOfRange {
                    requested,
                    available: item.option,
                });
            }
            requested
        } else {
            item.option
        };
        let intent = Pc2pcRegisterItemIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
            item: Pc2pcTradeItem0104 {
                item_type: item.item_type,
                item_id: item.item_id,
                option,
                inventory_slot: inventory_slot as i32,
                offer_slot: offer_slot as i32,
            },
        };
        self.queue_intent(Pc2pcIntent0104::RegisterItem(intent.clone()));
        Ok(intent)
    }

    pub fn request_unregister_item(
        &mut self,
        modal: Pc2pcModalState,
        offer_slot: usize,
    ) -> Result<Pc2pcUnregisterItemIntent0104, Pc2pcActionError0104> {
        let snapshot = self.ensure_action_gate(modal)?;
        if offer_slot >= PC2PC_OFFER_SLOT_COUNT {
            return Err(Pc2pcActionError0104::OfferSlotOutOfBounds { slot: offer_slot });
        }
        let Some(item) = snapshot.local_offer[offer_slot] else {
            return Err(Pc2pcActionError0104::EmptyOfferSlot { slot: offer_slot });
        };
        let intent = Pc2pcUnregisterItemIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
            item,
        };
        self.queue_intent(Pc2pcIntent0104::UnregisterItem(intent.clone()));
        Ok(intent)
    }

    pub fn request_register_taros(
        &mut self,
        modal: Pc2pcModalState,
        capabilities: Pc2pcBackendCapabilities,
        taros: i32,
    ) -> Result<Pc2pcRegisterCashIntent0104, Pc2pcActionError0104> {
        let snapshot = self.ensure_action_gate(modal)?;
        if !capabilities.numeric_popup_backend {
            return Err(Pc2pcActionError0104::NumericPopupBackendUnavailable);
        }
        if taros < 0 || taros > snapshot.local_wallet_taros {
            return Err(Pc2pcActionError0104::TarosOutOfRange {
                requested: taros,
                wallet: snapshot.local_wallet_taros,
            });
        }
        let intent = Pc2pcRegisterCashIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
            taros,
        };
        self.queue_intent(Pc2pcIntent0104::RegisterCash(intent.clone()));
        Ok(intent)
    }

    pub fn request_cancel(
        &mut self,
        modal: Pc2pcModalState,
    ) -> Result<Pc2pcCancelIntent0104, Pc2pcActionError0104> {
        let snapshot = self.ensure_action_gate(modal)?;
        if !modal.escape_allowed() {
            return Err(Pc2pcActionError0104::ModalBlocked);
        }
        let intent = Pc2pcCancelIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
        };
        self.queue_intent(Pc2pcIntent0104::Cancel(intent.clone()));
        // Clean `cnTrade.Exit` closes immediately after sending. The
        // immutable base inventory means there is no rollback mutation.
        self.state.phase = Pc2pcLifecyclePhase::Cancelling;
        Ok(intent)
    }

    pub fn request_chat(
        &mut self,
        modal: Pc2pcModalState,
        capabilities: Pc2pcBackendCapabilities,
        local_free_chat_allowed: Option<bool>,
        text: impl Into<String>,
    ) -> Result<Pc2pcChatIntent0104, Pc2pcActionError0104> {
        let snapshot = self.ensure_action_gate(modal)?;
        if !capabilities.free_chat_backend {
            return Err(Pc2pcActionError0104::FreeChatBackendUnavailable);
        }
        if local_free_chat_allowed != Some(true) {
            return Err(Pc2pcActionError0104::FreeChatSpecialState);
        }
        let text = text.into().replace('\n', "");
        if text.is_empty() {
            return Err(Pc2pcActionError0104::EmptyChat);
        }
        let characters = text.chars().count();
        if characters > PC2PC_CHAT_MAX_INPUT_CHARS {
            return Err(Pc2pcActionError0104::ChatTooLong {
                characters,
                maximum: PC2PC_CHAT_MAX_INPUT_CHARS,
            });
        }
        let command = text.split(' ').next().unwrap_or_default();
        if PC2PC_BLOCKED_CHAT_COMMANDS.contains(&command) {
            return Err(Pc2pcActionError0104::CheatCommandBlocked {
                command: command.to_owned(),
            });
        }
        let intent = Pc2pcChatIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
            text,
            emote_code: 1,
            free_chat_use: true,
        };
        self.queue_intent(Pc2pcIntent0104::Chat(intent.clone()));
        Ok(intent)
    }

    pub fn request_combat_cancel(&mut self) -> Result<Pc2pcCancelIntent0104, Pc2pcActionError0104> {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or(Pc2pcActionError0104::NoSession)?;
        if !matches!(
            self.state.phase,
            Pc2pcLifecyclePhase::Opening | Pc2pcLifecyclePhase::Trading
        ) {
            return Err(Pc2pcActionError0104::PhaseBlocked {
                phase: self.state.phase,
            });
        }
        if self.state.pending.is_some() {
            return Err(Pc2pcActionError0104::RequestPending);
        }
        let intent = Pc2pcCancelIntent0104 {
            envelope: Pc2pcEnvelope0104::local(snapshot.identity),
        };
        self.queue_intent(Pc2pcIntent0104::Cancel(intent.clone()));
        self.state.phase = Pc2pcLifecyclePhase::Cancelling;
        Ok(intent)
    }

    pub fn acknowledge_local_cancel_sent(
        &mut self,
        envelope: Pc2pcEnvelope0104,
    ) -> Result<(), Pc2pcCorrelationError0104> {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or(Pc2pcCorrelationError0104::FinalWithoutLocalAcceptance)?;
        snapshot.identity.validate_envelope(envelope)?;
        match self.state.pending.as_ref().cloned() {
            Some(Pc2pcPendingRequest0104::Cancel(expected)) if expected.envelope == envelope => {
                self.state.pending = None;
                self.state.phase = Pc2pcLifecyclePhase::Cancelled;
                Ok(())
            }
            Some(pending) => Err(Pc2pcCorrelationError0104::PendingKindMismatch {
                pending,
                outcome: Pc2pcRequestFailureKind0104::Cancel,
            }),
            None => Err(Pc2pcCorrelationError0104::MissingPending {
                expected: Pc2pcRequestFailureKind0104::Cancel,
            }),
        }
    }

    pub fn apply_server_outcome(
        &mut self,
        outcome: Pc2pcServerOutcome0104,
        capabilities: Pc2pcBackendCapabilities,
        catalog: &impl UserEquipItemCatalog,
        eligibility: &impl Pc2pcEquipEligibility,
    ) -> Result<Option<Pc2pcFinalCommitReceipt0104>, Pc2pcCorrelationError0104> {
        let snapshot = self
            .snapshot
            .as_mut()
            .ok_or(Pc2pcCorrelationError0104::FinalWithoutLocalAcceptance)?;
        let receipt = match outcome {
            Pc2pcServerOutcome0104::Confirmed { envelope } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                match participant {
                    Pc2pcParticipant0104::Local => {
                        take_local_pending_confirm(&mut self.state, envelope)?;
                        self.state.local_ready = true;
                        if self.state.remote_ready {
                            self.state.phase = Pc2pcLifecyclePhase::Completing;
                        } else {
                            self.state.phase = Pc2pcLifecyclePhase::Trading;
                        }
                    }
                    Pc2pcParticipant0104::Remote => {
                        self.state.remote_ready = true;
                    }
                }
                None
            }
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope,
                trade_item,
                inventory_item,
            } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                if participant == Pc2pcParticipant0104::Local {
                    let pending = pending_request(&self.state)?;
                    match pending {
                        Pc2pcPendingRequest0104::RegisterItem(expected)
                            if expected.envelope == envelope && expected.item == trade_item => {}
                        Pc2pcPendingRequest0104::RegisterItem(expected) => {
                            return Err(Pc2pcCorrelationError0104::RegisterItemMismatch {
                                expected: expected.item,
                                actual: trade_item,
                            });
                        }
                        pending => {
                            return Err(Pc2pcCorrelationError0104::PendingKindMismatch {
                                pending,
                                outcome: Pc2pcRequestFailureKind0104::RegisterItem,
                            });
                        }
                    }
                }
                snapshot
                    .apply_register_success(participant, trade_item, inventory_item)
                    .map_err(Pc2pcCorrelationError0104::Authority)?;
                if participant == Pc2pcParticipant0104::Local {
                    self.state.pending = None;
                }
                self.state.reset_readiness();
                self.state.phase = Pc2pcLifecyclePhase::Trading;
                None
            }
            Pc2pcServerOutcome0104::UnregisterItemSuccess {
                envelope,
                trade_item,
                inventory_item,
            } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                if participant == Pc2pcParticipant0104::Local {
                    let pending = pending_request(&self.state)?;
                    match pending {
                        Pc2pcPendingRequest0104::UnregisterItem(expected)
                            if expected.envelope == envelope && expected.item == trade_item => {}
                        Pc2pcPendingRequest0104::UnregisterItem(expected) => {
                            return Err(Pc2pcCorrelationError0104::UnregisterItemMismatch {
                                expected: expected.item,
                                actual: trade_item,
                            });
                        }
                        pending => {
                            return Err(Pc2pcCorrelationError0104::PendingKindMismatch {
                                pending,
                                outcome: Pc2pcRequestFailureKind0104::UnregisterItem,
                            });
                        }
                    }
                }
                snapshot
                    .apply_unregister_success(participant, trade_item, inventory_item)
                    .map_err(Pc2pcCorrelationError0104::Authority)?;
                if participant == Pc2pcParticipant0104::Local {
                    self.state.pending = None;
                }
                self.state.reset_readiness();
                self.state.phase = Pc2pcLifecyclePhase::Trading;
                None
            }
            Pc2pcServerOutcome0104::RegisterCashSuccess { envelope, taros } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                if participant == Pc2pcParticipant0104::Local {
                    let pending = pending_request(&self.state)?;
                    match pending {
                        Pc2pcPendingRequest0104::RegisterCash(expected)
                            if expected.envelope == envelope && expected.taros == taros => {}
                        Pc2pcPendingRequest0104::RegisterCash(expected) => {
                            return Err(Pc2pcCorrelationError0104::CashMismatch {
                                expected: expected.taros,
                                actual: taros,
                            });
                        }
                        pending => {
                            return Err(Pc2pcCorrelationError0104::PendingKindMismatch {
                                pending,
                                outcome: Pc2pcRequestFailureKind0104::RegisterCash,
                            });
                        }
                    }
                }
                snapshot
                    .apply_cash_success(participant, taros)
                    .map_err(Pc2pcCorrelationError0104::Authority)?;
                if participant == Pc2pcParticipant0104::Local {
                    self.state.pending = None;
                }
                self.state.reset_readiness();
                self.state.phase = Pc2pcLifecyclePhase::Trading;
                None
            }
            Pc2pcServerOutcome0104::RequestFailure {
                envelope,
                kind,
                error_code,
            } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                if participant != Pc2pcParticipant0104::Local {
                    return Err(Pc2pcCorrelationError0104::RemoteFailureNotCorrelatable);
                }
                take_matching_failure_pending(&mut self.state, envelope, kind)?;
                self.state.last_failure = Some(Pc2pcLastFailure0104::Request { kind, error_code });
                self.state.phase = Pc2pcLifecyclePhase::Trading;
                None
            }
            Pc2pcServerOutcome0104::ConfirmSuccess {
                envelope,
                received,
                item_stay,
                wallet_taros,
            } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                validate_final_pending(&self.state, participant, envelope)?;
                let receipt = snapshot
                    .apply_final_success(received, item_stay, wallet_taros)
                    .map_err(Pc2pcCorrelationError0104::Authority)?;
                self.state.pending = None;
                self.state.phase = Pc2pcLifecyclePhase::Completed;
                self.state.local_ready = false;
                self.state.remote_ready = false;
                Some(receipt)
            }
            Pc2pcServerOutcome0104::ConfirmFailure {
                envelope,
                error_code,
            } => {
                let participant = snapshot.identity.validate_envelope(envelope)?;
                validate_final_pending(&self.state, participant, envelope)?;
                self.state.pending = None;
                self.state.last_failure = Some(Pc2pcLastFailure0104::Confirm { error_code });
                self.state.phase = Pc2pcLifecyclePhase::Failed;
                None
            }
            Pc2pcServerOutcome0104::SessionEnded {
                envelope,
                reason,
                error_code: _,
            } => {
                snapshot.identity.validate_envelope(envelope)?;
                self.state.pending = None;
                self.state.reset_readiness();
                self.state.phase = match reason {
                    Pc2pcSessionEndReason0104::OfferCancelled
                    | Pc2pcSessionEndReason0104::OfferRefused
                    | Pc2pcSessionEndReason0104::OfferAborted
                    | Pc2pcSessionEndReason0104::ConfirmCancelled
                    | Pc2pcSessionEndReason0104::ConfirmAborted
                    | Pc2pcSessionEndReason0104::CombatStarted => Pc2pcLifecyclePhase::Cancelled,
                };
                None
            }
            Pc2pcServerOutcome0104::ChatMessage {
                envelope,
                text,
                emote_code,
            } => {
                if !capabilities.free_chat_backend && !capabilities.menu_chat_backend {
                    return Err(Pc2pcCorrelationError0104::ChatBackendUnavailable);
                }
                let participant = snapshot.identity.validate_envelope(envelope)?;
                if participant == Pc2pcParticipant0104::Local {
                    match pending_request(&self.state)? {
                        Pc2pcPendingRequest0104::Chat(expected)
                            if expected.envelope == envelope
                                && expected.text == text
                                && expected.emote_code == emote_code =>
                        {
                            self.state.pending = None;
                        }
                        Pc2pcPendingRequest0104::Chat(expected) => {
                            return Err(Pc2pcCorrelationError0104::ChatMismatch {
                                expected,
                                actual_text: text,
                                actual_emote_code: emote_code,
                            });
                        }
                        pending => {
                            return Err(Pc2pcCorrelationError0104::PendingKindMismatch {
                                pending,
                                outcome: Pc2pcRequestFailureKind0104::Chat,
                            });
                        }
                    }
                }
                let display_name = match participant {
                    Pc2pcParticipant0104::Local => snapshot.names.local.clone(),
                    Pc2pcParticipant0104::Remote => snapshot.names.remote.clone(),
                };
                if self.chat.len() > 40 {
                    self.chat.pop_front();
                }
                self.chat.push_back(Pc2pcChatLine0104 {
                    speaker_pc_id: envelope.requester_pc_id,
                    display_name,
                    text,
                    emote_code,
                });
                if self.chat.len() > 3 {
                    self.state.chat_scroll_y = self.chat.len() as f32 * PC2PC_CHAT_LINE_HEIGHT;
                }
                None
            }
            Pc2pcServerOutcome0104::ChatFailure {
                envelope,
                error_code,
            } => {
                if !capabilities.free_chat_backend && !capabilities.menu_chat_backend {
                    return Err(Pc2pcCorrelationError0104::ChatBackendUnavailable);
                }
                if snapshot.identity.validate_envelope(envelope)? != Pc2pcParticipant0104::Local {
                    return Err(Pc2pcCorrelationError0104::RemoteFailureNotCorrelatable);
                }
                take_matching_failure_pending(
                    &mut self.state,
                    envelope,
                    Pc2pcRequestFailureKind0104::Chat,
                )?;
                self.state.last_failure = Some(Pc2pcLastFailure0104::Chat { error_code });
                None
            }
        };
        self.projection =
            Pc2pcModeProjection0104::from_authoritative(snapshot, catalog, eligibility);
        Ok(receipt)
    }

    pub fn dismiss_terminal(&mut self) {
        if matches!(
            self.state.phase,
            Pc2pcLifecyclePhase::Completed
                | Pc2pcLifecyclePhase::Cancelled
                | Pc2pcLifecyclePhase::Failed
        ) {
            self.reset();
        }
    }
}

#[derive(Resource)]
pub(super) struct Pc2pcUiAssets {
    pub(super) images: [Handle<Image>; Pc2pcStaticAssetRole::COUNT],
    pub(super) jeffe_font: Handle<Font>,
    pub(super) chalet_font: Handle<Font>,
    pub(super) missing_checker: Handle<Image>,
    pub(super) contract_valid: bool,
}

impl Pc2pcUiAssets {
    pub(super) fn load(
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        contract: &Pc2pcUiAssetContract,
    ) -> Self {
        let contract_valid = contract.validate().is_ok();
        let image_paths = if contract_valid {
            contract.image_paths.clone()
        } else {
            PC2PC_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned)
        };
        let font_path = if contract_valid {
            contract.font_path.clone()
        } else {
            PC2PC_JEFFE_FONT_PATH.to_owned()
        };
        let chalet_font_path = if contract_valid {
            contract.chalet_font_path.clone()
        } else {
            PC2PC_CHALET_FONT_PATH.to_owned()
        };
        Self {
            images: image_paths.map(|path| asset_server.load(path)),
            jeffe_font: asset_server.load(font_path),
            chalet_font: asset_server.load(chalet_font_path),
            missing_checker: images.add(Image::new(
                Extent3d {
                    width: USER_EQUIP_MISSING_CHECKER_SIZE,
                    height: USER_EQUIP_MISSING_CHECKER_SIZE,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                user_equip_missing_checker_rgba(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )),
            contract_valid,
        }
    }

    #[must_use]
    pub(super) fn image(&self, role: Pc2pcStaticAssetRole) -> Handle<Image> {
        self.images[role as usize].clone()
    }

    #[must_use]
    pub(super) fn readiness(&self, asset_server: &AssetServer) -> Pc2pcStaticAssetReadiness {
        if !self.contract_valid {
            return Pc2pcStaticAssetReadiness::Rejected;
        }
        let images_loaded = self
            .images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
        let fonts_loaded = [self.jeffe_font.id(), self.chalet_font.id()]
            .into_iter()
            .all(|id| matches!(asset_server.load_state(id), LoadState::Loaded));
        if images_loaded && fonts_loaded {
            Pc2pcStaticAssetReadiness::Ready
        } else {
            Pc2pcStaticAssetReadiness::Loading
        }
    }
}

#[derive(Component)]
pub struct Pc2pcUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum Pc2pcUiElement {
    Backdrop,
    TradeBackplate,
    RightBackplate,
    TradePanel,
    TradeArea,
    LocalOfferBox,
    RemoteOfferBox,
    LocalOfferFrame(usize),
    LocalOfferIcon(usize),
    LocalOfferCount(usize),
    RemoteOfferFrame(usize),
    RemoteOfferIcon(usize),
    RemoteOfferCount(usize),
    LocalMoney,
    RemoteMoney,
    LocalMoneyText,
    RemoteMoneyText,
    LocalTaros,
    RemoteTaros,
    AddTaros,
    LocalPortrait,
    RemotePortrait,
    LocalFreeChat,
    RemoteFreeChat,
    LocalTitle,
    RemoteTitle,
    MainButton,
    MainButtonText,
    ReadyName,
    ReadyWaitingPrefix,
    ReadyPlayerName,
    ReadySubject,
    ChatBox,
    ChatList,
    ChatInput,
    ChatSend,
    PcStuffPanel,
    InventoryViewport,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotCount(usize),
    ItemTabLabel,
    NanoTab,
    NanoTabLabel,
    Close,
    Trash,
    Help,
    EquipmentPanel,
    EquipmentContent,
    EquipmentTitle,
    EquipmentTitleLabel,
    EquipmentSlotFrame(usize),
    EquipmentSlotIcon(usize),
    EquipmentSlotBadge(usize),
    EquipmentSlotLabel(usize),
}

/// Exact clean-0104 GUIStyle role attached to every native PC2PC `Text`.
///
/// This is deliberately separate from localization identity: translated text
/// keeps the source style, Rect and replacement-font calibration.
#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum Pc2pcTextStyle0104 {
    LabelUpperLeft,
    LabelMiddleRight,
    RightLabel,
    Button,
    ChatLine,
    ChatInput,
    ChatSendButton,
    ReadyLeftLabel,
    InventoryTab,
    InventoryCount,
    EquipmentTitle,
    EquipmentSlot,
}
