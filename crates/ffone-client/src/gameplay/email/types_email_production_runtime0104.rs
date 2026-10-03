use super::*;

impl EmailProductionRuntime0104 {
    #[must_use]
    pub const fn session(&self) -> Option<&EmailProductionSession0104> {
        self.session.as_ref()
    }

    #[must_use]
    pub const fn mode_lease(&self) -> Option<EmailModeLease0104> {
        match &self.session {
            Some(session) => Some(session.lease),
            None => None,
        }
    }

    #[must_use]
    pub const fn modal_active(&self) -> bool {
        self.session.is_some()
    }

    #[must_use]
    pub fn pending_request(&self) -> Option<&EmailRequest> {
        self.pending_request.as_ref()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &mut self,
        context: EmailOpenContext0104,
        guide_messages: Vec<EmailGuideMessage>,
        buddies: Vec<EmailBuddy>,
        inventory: &EmailInventoryAuthority0104,
        catalog: &impl EmailItemCatalog0104,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
        network: &EmailNetworkRuntime0104,
        inbox: &EmailNetworkInbox0104,
        transport: &EmailTransportOutbox,
    ) -> Result<EmailProductionOutput0104, EmailProductionError0104> {
        if self.session.is_some() {
            return Err(EmailProductionError0104::AlreadyActive);
        }
        if model.visible {
            return Err(EmailProductionError0104::ModelAlreadyVisible);
        }
        if self.pending_request.is_some()
            || network.0.pending().is_some()
            || !inbox.0.is_empty()
            || !transport.0.is_empty()
        {
            return Err(EmailProductionError0104::DirtyTransportBoundary);
        }
        if !actions.0.is_empty() || !audio.0.is_empty() {
            return Err(EmailProductionError0104::DirtyEffectBoundary);
        }
        validate_player_authority(context.player, inventory)?;
        let buddies = project_email_buddies_0104(&buddies)?;
        let projected_inventory = project_email_inventory_0104(inventory, catalog)?;
        let lease = EmailModeLease0104::from(context);

        model.buddies = buddies;
        model.inventory = projected_inventory;
        model.current_local_time = context.player.current_local_time;
        open_email_ui(model, actions, guide_messages, context.player.taros);
        self.session = Some(EmailProductionSession0104 {
            context,
            lease,
            inventory: inventory.clone(),
            inventory_projection_stale: false,
            close_source: None,
        });
        self.collect_effects(actions, audio)
    }

    pub fn refresh_guide_projection(
        &mut self,
        guide_messages: Vec<EmailGuideMessage>,
        model: &mut EmailUiModel,
    ) -> Result<(), EmailProductionError0104> {
        self.require_active()?;
        if model.guide_messages == guide_messages {
            return Ok(());
        }
        model.guide_messages = guide_messages;
        if model.screen == EmailScreen::List && model.folder == EmailFolder::Guide {
            let max_page = model.guide_messages.len().div_ceil(5).saturating_sub(1);
            model.guide_page = model.guide_page.min(max_page);
            model.selected_row = (!model.visible_guide_messages().is_empty()).then_some(0);
            model.read_message = None;
        }
        Ok(())
    }

    pub fn refresh_buddy_projection(
        &mut self,
        buddies: Vec<EmailBuddy>,
        model: &mut EmailUiModel,
    ) -> Result<(), EmailProductionError0104> {
        self.require_active()?;
        model.buddies = project_email_buddies_0104(&buddies)?;
        Ok(())
    }

    pub fn refresh_inventory_projection(
        &mut self,
        player: EmailPlayerAuthority0104,
        inventory: &EmailInventoryAuthority0104,
        catalog: &impl EmailItemCatalog0104,
        model: &mut EmailUiModel,
    ) -> Result<(), EmailProductionError0104> {
        let session = self
            .session
            .as_ref()
            .ok_or(EmailProductionError0104::NotActive)?;
        if player.owner_pc_id != session.context.player.owner_pc_id {
            return Err(EmailProductionError0104::InventoryOwnerMismatch {
                expected_pc_id: session.context.player.owner_pc_id,
                inventory_pc_id: player.owner_pc_id,
            });
        }
        validate_player_authority(player, inventory)?;
        if self.pending_request.is_none() {
            validate_staged_attachments_unchanged(model, inventory)?;
        }
        let projected = project_email_inventory_0104(inventory, catalog)?;

        let session = self
            .session
            .as_mut()
            .expect("active Email session was validated");
        session.context.player = player;
        session.inventory = inventory.clone();
        session.inventory_projection_stale = false;
        model.inventory = projected;
        model.available_cash = player.taros;
        model.current_local_time = player.current_local_time;
        Ok(())
    }

    pub fn switch_folder(
        &mut self,
        folder: EmailFolder,
        model: &mut EmailUiModel,
        transport: &mut EmailTransportOutbox,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
    ) -> Result<Option<EmailProductionOutput0104>, EmailProductionError0104> {
        self.require_active()?;
        if !switch_email_folder(folder, model, transport, audio) {
            return Ok(None);
        }
        self.collect_effects(actions, audio).map(Some)
    }

    pub fn request_close(
        &mut self,
        source: EmailCloseSource,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
    ) -> Result<Option<EmailProductionOutput0104>, EmailProductionError0104> {
        self.require_active()?;
        if self.pending_request.is_some() {
            return Err(EmailProductionError0104::RequestAlreadyPending {
                request: self
                    .pending_request
                    .as_ref()
                    .expect("pending request was checked")
                    .clone(),
            });
        }
        if !request_email_close(source, model, actions) {
            return Ok(None);
        }
        self.session
            .as_mut()
            .expect("active Email session was validated")
            .close_source = Some(source);
        self.collect_effects(actions, audio).map(Some)
    }

    pub fn resolve_escape_gate(
        &mut self,
        accepted: bool,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
    ) -> Result<Option<EmailProductionOutput0104>, EmailProductionError0104> {
        let session = self
            .session
            .as_mut()
            .ok_or(EmailProductionError0104::NotActive)?;
        if session.close_source != Some(EmailCloseSource::Escape) {
            return Err(EmailProductionError0104::CloseGateSourceMismatch);
        }
        if !resolve_email_escape_gate(accepted, model, actions) {
            return Ok(None);
        }
        if !accepted {
            session.close_source = None;
        }
        self.collect_effects(actions, audio).map(Some)
    }

    pub fn resolve_computress_gate(
        &mut self,
        computress_active: bool,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
    ) -> Result<Option<EmailProductionOutput0104>, EmailProductionError0104> {
        let session = self
            .session
            .as_ref()
            .ok_or(EmailProductionError0104::NotActive)?;
        if session.close_source.is_none() {
            return Err(EmailProductionError0104::CloseGateSourceMismatch);
        }
        if !resolve_email_computress_gate(computress_active, model, actions) {
            return Ok(None);
        }
        if computress_active {
            self.session
                .as_mut()
                .expect("active Email session was validated")
                .close_source = None;
        }
        self.collect_effects(actions, audio).map(Some)
    }

    pub fn dispatch_next_request(
        &mut self,
        model: &EmailUiModel,
        transport: &mut EmailTransportOutbox,
        network: &mut EmailNetworkRuntime0104,
        catalog: &impl EmailItemCatalog0104,
    ) -> Result<EmailProductionOutput0104, EmailProductionError0104> {
        self.require_transport_alignment(network)?;
        if self.pending_request.is_some() {
            return Ok(EmailProductionOutput0104::default());
        }
        let Some(request) = transport.0.front().cloned() else {
            return Ok(EmailProductionOutput0104::default());
        };
        let session = self
            .session
            .as_ref()
            .ok_or(EmailProductionError0104::NotActive)?;
        validate_request_reachable(session, model, &request, catalog)?;

        let wire = encode_email_request_0104(&request).map_err(EmailRuntimeError0104::from)?;
        let registered = RegisteredGameplayRequest0104::new(wire.packet_id, wire.body.clone())?;
        let begun = network.0.begin(&request)?;
        debug_assert_eq!(begun, wire);
        self.pending_request = Some(request);
        transport.0.pop_front();
        Ok(EmailProductionOutput0104 {
            request: Some(registered),
            ..EmailProductionOutput0104::default()
        })
    }

    /// Clears only a request whose socket write failed. No item/Taros state is
    /// rolled back because none was predicted.
    pub fn cancel_pending_transport(
        &mut self,
        model: &mut EmailUiModel,
        network: &mut EmailNetworkRuntime0104,
    ) -> Result<EmailRequest, EmailProductionError0104> {
        self.require_transport_alignment(network)?;
        let request = self
            .pending_request
            .take()
            .ok_or(EmailProductionError0104::NoPendingRequest)?;
        network.0.cancel_pending();
        model.send_in_flight = false;
        if matches!(request, EmailRequest::Send { .. }) {
            model.mail_send_in_flight = false;
        }
        Ok(request)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn route_frame(
        &mut self,
        frame: DecodedFrame,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        transport: &mut EmailTransportOutbox,
        audio: &mut EmailUiAudioOutbox,
        network: &mut EmailNetworkRuntime0104,
    ) -> EmailFrameDisposition0104 {
        if !EmailNetworkInbox0104::owns_packet(frame.packet_type) {
            return EmailFrameDisposition0104::Passthrough(frame);
        }
        match self.apply_owned_frame(&frame, model, actions, transport, audio, network) {
            Ok(output) => EmailFrameDisposition0104::Applied { frame, output },
            Err(error) => EmailFrameDisposition0104::Rejected { frame, error },
        }
    }

    /// Queues only owned Email packets. A caller that receives `Err(frame)`
    /// must pass that exact value to the next gameplay classifier.
    pub fn enqueue_frame(
        inbox: &mut EmailNetworkInbox0104,
        frame: DecodedFrame,
    ) -> Result<(), DecodedFrame> {
        if !EmailNetworkInbox0104::owns_packet(frame.packet_type) {
            return Err(frame);
        }
        inbox.0.push_back(frame);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn consume_next_frame(
        &mut self,
        inbox: &mut EmailNetworkInbox0104,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        transport: &mut EmailTransportOutbox,
        audio: &mut EmailUiAudioOutbox,
        network: &mut EmailNetworkRuntime0104,
    ) -> Option<EmailFrameDisposition0104> {
        inbox
            .0
            .pop_front()
            .map(|frame| self.route_frame(frame, model, actions, transport, audio, network))
    }

    /// Drains the two passive UI effect channels. `ExitMode` also releases the
    /// production lease and restores the cursor bit captured on open.
    pub fn collect_effects(
        &mut self,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
    ) -> Result<EmailProductionOutput0104, EmailProductionError0104> {
        if actions
            .0
            .iter()
            .any(|action| matches!(action, EmailUiAction::ApplySendSuccessItems(_)))
        {
            return Err(EmailProductionError0104::UnexpectedSendCommitAction);
        }
        if actions
            .0
            .iter()
            .any(|action| matches!(action, EmailUiAction::ExitMode { .. }))
            && let Some(request) = self.pending_request.as_ref()
        {
            return Err(EmailProductionError0104::RequestAlreadyPending {
                request: request.clone(),
            });
        }
        let raw_actions = actions.0.drain(..).collect::<Vec<_>>();
        let audio = audio.0.drain(..).collect();
        let actions = self.normalize_lifecycle_actions(raw_actions)?;
        Ok(EmailProductionOutput0104 {
            actions,
            audio,
            ..EmailProductionOutput0104::default()
        })
    }

    /// Disconnect/session-replacement reset. This is intentionally not a
    /// clean user close and therefore emits no shell effects.
    #[allow(clippy::too_many_arguments)]
    pub fn reset_boundary(
        &mut self,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        audio: &mut EmailUiAudioOutbox,
        transport: &mut EmailTransportOutbox,
        network: &mut EmailNetworkRuntime0104,
        inbox: &mut EmailNetworkInbox0104,
    ) {
        *self = Self::default();
        *model = EmailUiModel::default();
        actions.0.clear();
        audio.0.clear();
        transport.0.clear();
        network.0.cancel_pending();
        inbox.clear();
    }

    pub(super) fn require_active(&self) -> Result<(), EmailProductionError0104> {
        if self.session.is_some() {
            Ok(())
        } else {
            Err(EmailProductionError0104::NotActive)
        }
    }

    pub(super) fn require_transport_alignment(
        &self,
        network: &EmailNetworkRuntime0104,
    ) -> Result<(), EmailProductionError0104> {
        let controller = self
            .pending_request
            .as_ref()
            .map(EmailPending0104::from_request);
        let transport = network.0.pending().cloned();
        if controller == transport {
            Ok(())
        } else {
            Err(EmailProductionError0104::TransportStateMismatch {
                controller,
                transport,
            })
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_owned_frame(
        &mut self,
        frame: &DecodedFrame,
        model: &mut EmailUiModel,
        actions: &mut EmailUiOutbox,
        transport: &mut EmailTransportOutbox,
        audio: &mut EmailUiAudioOutbox,
        network: &mut EmailNetworkRuntime0104,
    ) -> Result<EmailProductionOutput0104, EmailProductionError0104> {
        self.require_transport_alignment(network)?;
        let reply = decode_email_reply_0104(frame.packet_type, &frame.payload)
            .map_err(EmailRuntimeError0104::from)?
            .expect("owned Email packet IDs are exhaustive");
        let pending = self.pending_request.as_ref();
        validate_reply_against_request(pending, &reply)?;
        let commit = self.authoritative_commit(pending, &reply)?;

        let delivery = network
            .0
            .accept(frame.packet_type, &frame.payload)?
            .expect("owned valid Email packet always produces a delivery");
        let correlated = matches!(&delivery, EmailRuntimeDelivery0104::Correlated(_));
        let preserve_in_flight =
            matches!(&delivery, EmailRuntimeDelivery0104::UnsolicitedNewEmail(_))
                && self.pending_request.is_some();
        if correlated {
            self.pending_request = None;
        }

        let in_flight_before_push = (model.send_in_flight, model.mail_send_in_flight);
        let send_success_items = match &reply {
            EmailReply::SendSuccess { items, .. } => Some(*items),
            _ => None,
        };
        apply_email_reply(reply, model, actions, transport, audio);
        if preserve_in_flight {
            (model.send_in_flight, model.mail_send_in_flight) = in_flight_before_push;
        }
        if let Some(expected) = send_success_items {
            remove_send_commit_action(actions, expected)?;
        }
        if let Some(commit) = &commit {
            self.apply_commit_to_session(commit);
        }
        let mut output = self.collect_effects(actions, audio)?;
        output.commit = commit;
        Ok(output)
    }

    pub(super) fn authoritative_commit(
        &self,
        pending: Option<&EmailRequest>,
        reply: &EmailReply,
    ) -> Result<Option<EmailAuthoritativeCommit0104>, EmailProductionError0104> {
        let Some(session) = self.session.as_ref() else {
            return match reply {
                EmailReply::NewEmail { .. } => Ok(None),
                _ => Err(EmailProductionError0104::NotActive),
            };
        };
        let owner_pc_id = session.context.player.owner_pc_id;
        let empty_writes = [None; EMAIL_ATTACHMENT_COUNT];
        let commit = match reply {
            EmailReply::SendSuccess {
                authoritative_cash,
                items,
                ..
            } => {
                require_non_negative_authoritative_taros(*authoritative_cash)?;
                let writes = array::from_fn(|index| {
                    let outgoing = items[index];
                    (!outgoing.item.is_empty() && outgoing.item.item_type != 7).then(|| {
                        let mut item = item_base_from_email(outgoing.item);
                        // Clean `funcP_FE2CL_REP_PC_SEND_EMAIL_SUCC` changes
                        // only iID before forwarding the post-state slot.
                        item.item_id = 0;
                        EmailInventoryWrite0104 {
                            inventory_slot: outgoing.inventory_slot as usize,
                            item,
                        }
                    })
                });
                Some(EmailAuthoritativeCommit0104 {
                    owner_pc_id,
                    source: EmailAuthoritySource0104::SendSuccess,
                    inventory_writes: writes,
                    taros_after: Some(*authoritative_cash),
                    inventory_refresh_required: pending.is_some_and(|request| match request {
                        EmailRequest::Send { items, .. } => {
                            items.iter().any(|item| !item.item.is_empty())
                        }
                        _ => false,
                    }),
                })
            }
            EmailReply::ReceiveItemSuccess {
                email_index,
                inventory_slot,
                email_item_slot,
            } => Some(EmailAuthoritativeCommit0104 {
                owner_pc_id,
                source: EmailAuthoritySource0104::ReceiveItemSuccess {
                    email_index: *email_index,
                    inventory_slot: *inventory_slot,
                    email_item_slot: *email_item_slot,
                },
                inventory_writes: empty_writes,
                taros_after: None,
                inventory_refresh_required: true,
            }),
            EmailReply::ReceiveCashSuccess {
                email_index,
                authoritative_cash,
            } => {
                require_non_negative_authoritative_taros(*authoritative_cash)?;
                Some(EmailAuthoritativeCommit0104 {
                    owner_pc_id,
                    source: EmailAuthoritySource0104::ReceiveCashSuccess {
                        email_index: *email_index,
                    },
                    inventory_writes: empty_writes,
                    taros_after: Some(*authoritative_cash),
                    inventory_refresh_required: false,
                })
            }
            EmailReply::ReceiveAllItemsSuccess { email_index } => {
                Some(EmailAuthoritativeCommit0104 {
                    owner_pc_id,
                    source: EmailAuthoritySource0104::ReceiveAllItemsSuccess {
                        email_index: *email_index,
                    },
                    inventory_writes: empty_writes,
                    taros_after: None,
                    inventory_refresh_required: true,
                })
            }
            _ => None,
        };
        Ok(commit)
    }

    pub(super) fn apply_commit_to_session(&mut self, commit: &EmailAuthoritativeCommit0104) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        for write in commit.inventory_writes() {
            session.inventory.slots[write.inventory_slot] = write.item;
        }
        if let Some(taros_after) = commit.taros_after {
            session.context.player.taros = taros_after;
        }
        session.inventory_projection_stale |= commit.inventory_refresh_required;
    }

    pub(super) fn normalize_lifecycle_actions(
        &mut self,
        raw: Vec<EmailUiAction>,
    ) -> Result<Vec<EmailUiAction>, EmailProductionError0104> {
        if !raw
            .iter()
            .any(|action| matches!(action, EmailUiAction::ExitMode { .. }))
        {
            return Ok(raw);
        }
        let session = self
            .session
            .take()
            .ok_or(EmailProductionError0104::NotActive)?;
        self.pending_request = None;

        let mut other = Vec::new();
        let mut inventory_close = None;
        let mut exit = None;
        let mut stop_sound = None;
        for action in raw {
            match action {
                action @ EmailUiAction::SetInventoryMailMode { value, .. }
                    if value == EMAIL_INVENTORY_MAIL_MODE_CLOSED_0104 =>
                {
                    inventory_close = Some(action);
                }
                action @ EmailUiAction::ExitMode { .. } => exit = Some(action),
                action @ EmailUiAction::StopUiModeSound => stop_sound = Some(action),
                EmailUiAction::SetCursorLocked(_) => {}
                action => other.push(action),
            }
        }
        if let Some(action) = inventory_close {
            other.push(action);
        }
        other.push(EmailUiAction::SetCursorLocked(
            session.context.cursor_was_locked,
        ));
        if let Some(action) = exit {
            other.push(action);
        }
        if let Some(action) = stop_sound {
            other.push(action);
        }
        Ok(other)
    }
}
