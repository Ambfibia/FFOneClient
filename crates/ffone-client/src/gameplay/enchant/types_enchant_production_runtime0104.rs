use super::*;

impl EnchantProductionRuntime0104 {
    #[must_use]
    pub const fn session(&self) -> Option<&EnchantProductionSession0104> {
        self.session.as_ref()
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    #[must_use]
    pub fn request_pending(&self) -> bool {
        self.session
            .as_ref()
            .and_then(|session| session.pending_request.as_ref())
            .is_some()
    }

    pub fn reset(&mut self) {
        self.session = None;
    }

    pub fn open(
        &mut self,
        context: EnchantOpenContext0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        if self.session.is_some() {
            return Err(EnchantProductionError0104::AlreadyActive);
        }
        validate_open_context(context, inventory)?;
        let snapshot = snapshot_from_authority(inventory, context.player)?;
        let mut model = EnchantModeModel0104::default();
        model.open(context.player.taros, context.old_cursor_lock);
        let mut session = EnchantProductionSession0104 {
            context,
            model,
            snapshot,
            drag: None,
            reserved_by_source: [0; INVENTORY_SLOT_COUNT_0104],
            reservations_by_attachment: array::from_fn(|_| None),
            runtime_modal: None,
            close_decision_pending: false,
            pending_request: None,
        };
        let mut output = EnchantProductionOutput0104 {
            effects: vec![EnchantShellEffect0104::ModeOpened(context)],
            ..EnchantProductionOutput0104::default()
        };
        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        self.session = Some(session);
        Ok(output)
    }

    /// Refreshes global authority only while the clean base panel owns input.
    /// Any selected source must stay byte-identical.  A Taros change rebuilds
    /// the pure model from the same attached objects because `EnchantModeModel`
    /// intentionally exposes no speculative currency setter.
    pub fn refresh_authority(
        &mut self,
        inventory: &InventoryRuntime0104,
        player: EnchantPlayerAuthority0104,
    ) -> Result<(), EnchantProductionError0104> {
        let session = self
            .session
            .as_ref()
            .ok_or(EnchantProductionError0104::NotActive)?;
        if !matches!(session.model.phase(), EnchantPhase0104::Ready)
            || session.pending_request.is_some()
            || session.runtime_modal.is_some()
            || session.close_decision_pending
            || session.model.external_gates().any()
        {
            return Err(EnchantProductionError0104::AuthorityRefreshBlocked {
                phase: session.model.phase().clone(),
            });
        }
        validate_player_authority(session.context.player.owner_pc_id, inventory, player)?;
        let candidate = snapshot_from_authority(inventory, player)?;
        for (inventory_index, reserved) in session.reserved_by_source.iter().copied().enumerate() {
            if reserved > 0
                && candidate.inventory[inventory_index]
                    != session.snapshot.inventory[inventory_index]
            {
                return Err(EnchantProductionError0104::SelectedItemChanged {
                    inventory_index,
                    expected: session.snapshot.inventory[inventory_index],
                    actual: candidate.inventory[inventory_index],
                });
            }
        }

        let mut next = session.clone();
        if candidate.taros != session.snapshot.taros {
            if session.model.selection().any_orphaned() {
                return Err(EnchantProductionError0104::AuthorityRefreshBlocked {
                    phase: session.model.phase().clone(),
                });
            }
            let selected: Vec<(EnchantAttachmentSlot0104, EnchantSelectableItem0104)> =
                EnchantAttachmentSlot0104::ALL
                    .into_iter()
                    .filter(|slot| session.model.selection().is_attached(*slot))
                    .filter_map(|slot| {
                        session
                            .model
                            .selection()
                            .visual_item(slot)
                            .cloned()
                            .map(|item| (slot, item))
                    })
                    .collect();
            let mut rebuilt = EnchantModeModel0104::default();
            rebuilt.open(candidate.taros, session.context.old_cursor_lock);
            rebuilt.clear_intents();
            for (slot, item) in selected {
                rebuilt.attach(slot, item)?;
            }
            rebuilt.clear_intents();
            next.model = rebuilt;
        }
        next.context.player = player;
        next.snapshot = candidate;
        if let Some(drag) = &next.drag {
            let actual = next.snapshot.inventory[drag.inventory_index];
            if actual != drag.authority_item {
                next.drag = None;
            }
        }
        self.session = Some(next);
        Ok(())
    }

    /// Projects the pure model into the passive UI.  Item icons/labels remain
    /// caller-owned table data; Taros and both battery counters come only from
    /// this controller's last accepted authority snapshot.
    pub fn write_presentation(
        &self,
        projection: &mut EnchantModeProjection0104,
        support: EnchantSupportPresentation0104,
        inventory: [EnchantInventorySlotProjection0104; INVENTORY_SLOT_COUNT_0104],
        equipment: [EnchantInventorySlotProjection0104; EQUIPMENT_SLOT_COUNT_0104],
    ) {
        let Some(session) = &self.session else {
            *projection = EnchantModeProjection0104::default();
            return;
        };
        *projection = EnchantModeProjection0104::from_model(
            &session.model,
            support,
            inventory,
            equipment,
            session.snapshot.weapon_battery,
            session.snapshot.nano_battery,
        );
        if session.pending_request.is_some()
            || session.runtime_modal.is_some()
            || session.close_decision_pending
        {
            projection.capabilities.base_gui_enabled = false;
            projection.capabilities.preview_enabled = false;
            projection.capabilities.clear_enabled = false;
            projection.capabilities.enchant_enabled = false;
            projection.capabilities.close_enabled = false;
        }
    }

    pub fn set_external_gates(
        &mut self,
        mut gates: EnchantExternalGates0104,
    ) -> Result<(), EnchantProductionError0104> {
        let session = self
            .session
            .as_mut()
            .ok_or(EnchantProductionError0104::NotActive)?;
        if session.runtime_modal.is_some() {
            gates.system_popup_open = true;
        }
        session.model.set_external_gates(gates);
        Ok(())
    }

    pub fn apply_next_ui_command(
        &mut self,
        outbox: &mut EnchantUiOutbox0104,
        catalog: &impl EnchantPresentationCatalog0104,
    ) -> Option<Result<EnchantProductionOutput0104, EnchantProductionError0104>> {
        outbox
            .pop_front()
            .map(|command| self.apply_ui_command(command, catalog))
    }

    pub fn apply_ui_command(
        &mut self,
        command: EnchantUiCommand0104,
        catalog: &impl EnchantPresentationCatalog0104,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        ensure_runtime_input_available(&session)?;
        let mut output = EnchantProductionOutput0104::default();
        let mut terminal = false;

        match command {
            EnchantUiCommand0104::BeginInventoryDrag(inventory_index) => {
                begin_inventory_drag(&mut session, inventory_index, catalog, &mut output)?;
            }
            EnchantUiCommand0104::DropOnAttachment(slot) => {
                let drag = session
                    .drag
                    .clone()
                    .ok_or(EnchantProductionError0104::MissingDragSource)?;
                let actual = session.snapshot.inventory[drag.inventory_index];
                if actual != drag.authority_item {
                    return Err(EnchantProductionError0104::StaleDragSource {
                        inventory_index: drag.inventory_index,
                        expected: drag.authority_item,
                        actual,
                    });
                }
                session.model.attach(slot, drag.selectable)?;
                session.drag = None;
            }
            EnchantUiCommand0104::DetachAttachment(slot) => {
                session.model.detach(slot)?;
            }
            EnchantUiCommand0104::Preview => {
                session.model.preview()?;
            }
            EnchantUiCommand0104::Clear => {
                session.model.clear_all()?;
                session.drag = None;
            }
            EnchantUiCommand0104::Enchant => {
                match session.model.activate_enchant() {
                    Ok(()) | Err(EnchantModelError0104::NotEnoughTaros) => {}
                    Err(error) => return Err(error.into()),
                }
                session.drag = None;
            }
            EnchantUiCommand0104::Close => {
                if !session.model.input_capabilities().close_enabled {
                    return Err(EnchantModelError0104::InputBlocked.into());
                }
                session.close_decision_pending = true;
                output.effects.push(EnchantShellEffect0104::Input(
                    EnchantInputEffect0104::InventoryCloseDecisionRequested,
                ));
            }
            EnchantUiCommand0104::DropOnTrash => {
                open_delete_modal(&mut session, &mut output)?;
            }
            EnchantUiCommand0104::Help => {
                if !session.model.input_capabilities().base_gui_enabled {
                    return Err(EnchantModelError0104::InputBlocked.into());
                }
                output.effects.push(EnchantShellEffect0104::HelpRequested);
            }
            EnchantUiCommand0104::OpenRedeemCode => {
                session.model.open_redeem_code()?;
            }
            EnchantUiCommand0104::EnchantMoreItems => {
                session.model.enchant_more_items()?;
                session.drag = None;
            }
            EnchantUiCommand0104::GoToMyStuff => {
                session.model.go_to_my_stuff()?;
                terminal = true;
            }
        }

        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        if terminal {
            output
                .effects
                .push(EnchantShellEffect0104::ModeClosed(session.context));
            self.session = None;
        } else {
            self.session = Some(session);
        }
        Ok(output)
    }

    /// Resolves either a clean system-message phase or the clean inventory
    /// delete confirmation.  Enchant confirmation first proves that the exact
    /// request is registered by the pinned shard.  At present that proof fails
    /// with `UnregisteredPacket(0x130000a4)` and the modal/model are unchanged.
    pub fn resolve_modal(
        &mut self,
        choice: EnchantModalChoice0104,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        if session.pending_request.is_some() {
            return Err(EnchantProductionError0104::RequestAlreadyPending {
                operation: session
                    .pending_request
                    .as_ref()
                    .expect("checked pending request")
                    .operation(),
            });
        }
        let mut output = EnchantProductionOutput0104::default();

        if let Some(modal) = session.runtime_modal.clone() {
            match (modal, choice) {
                (
                    EnchantRuntimeModal0104::DeleteItem {
                        inventory_index,
                        item,
                        ..
                    },
                    EnchantModalChoice0104::Accept,
                ) => {
                    let actual = session.snapshot.inventory[inventory_index];
                    if actual != item {
                        return Err(EnchantProductionError0104::StaleDragSource {
                            inventory_index,
                            expected: item,
                            actual,
                        });
                    }
                    let request = PcItemDeleteRequest0104 {
                        item_location: 1,
                        slot_num: inventory_index as i32,
                    };
                    output.request = Some(encode_enchant_delete_request_0104(request)?);
                    session.pending_request = Some(PendingRequest0104::Delete(PendingDelete0104 {
                        inventory_index,
                        item_before: item,
                    }));
                }
                (EnchantRuntimeModal0104::DeleteItem { .. }, EnchantModalChoice0104::Dismiss) => {}
            }
            session.runtime_modal = None;
            let mut gates = session.model.external_gates();
            gates.system_popup_open = false;
            session.model.set_external_gates(gates);
            self.session = Some(session);
            return Ok(output);
        }

        let EnchantPhase0104::SystemMessage { callback, .. } = session.model.phase().clone() else {
            return Err(EnchantProductionError0104::NoModalToResolve);
        };
        if callback == EnchantSystemCallback0104::EnchantConfirmed
            && choice == EnchantModalChoice0104::Accept
        {
            prove_enchant_transport_registered_0104()?;
        }
        match choice {
            EnchantModalChoice0104::Accept => session.model.accept_system_message()?,
            EnchantModalChoice0104::Dismiss => session.model.dismiss_system_message()?,
        }
        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        self.session = Some(session);
        Ok(output)
    }

    pub fn resolve_close(
        &mut self,
        inventory_accepts_close: bool,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        if !session.close_decision_pending {
            return Err(EnchantProductionError0104::NoCloseDecisionPending);
        }
        session.close_decision_pending = false;
        let mut output = EnchantProductionOutput0104::default();
        if !inventory_accepts_close {
            output.effects.push(EnchantShellEffect0104::Input(
                EnchantInputEffect0104::InventoryCloseRejected,
            ));
            self.session = Some(session);
            return Ok(output);
        }
        session.model.close(true)?;
        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        output
            .effects
            .push(EnchantShellEffect0104::ModeClosed(session.context));
        self.session = None;
        Ok(output)
    }

    pub fn cancel_redeem_code(
        &mut self,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        ensure_no_pending_request(&session)?;
        let mut output = EnchantProductionOutput0104::default();
        session.model.cancel_redeem_code()?;
        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        self.session = Some(session);
        Ok(output)
    }

    pub fn submit_redeem_code(
        &mut self,
        code: &str,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        ensure_no_pending_request(&session)?;
        let mut output = EnchantProductionOutput0104::default();
        let result = session.model.submit_redeem_code(code);
        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        match result {
            Ok(_) => {
                self.session = Some(session);
                Ok(output)
            }
            Err(error @ EnchantModelError0104::Redeem(_)) => {
                // Space-error popup/audio are observable clean side effects.
                self.session = Some(session);
                if output.effects.is_empty() {
                    Err(error.into())
                } else {
                    Ok(output)
                }
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Advances the exact four-second waiting state.  The emitted wire intent
    /// is accepted only through [`RegisteredGameplayRequest0104`].
    pub fn tick(
        &mut self,
        delta_seconds: f32,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        ensure_no_pending_request(&session)?;
        let mut output = EnchantProductionOutput0104::default();
        session.model.advance(delta_seconds)?;
        let receipts = drain_model_intents(&mut session, &mut output)?;
        if !receipts.is_empty() {
            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
        }
        self.session = Some(session);
        Ok(output)
    }

    pub fn apply_frame(
        &mut self,
        frame: DecodedFrame,
    ) -> Result<Option<EnchantProductionOutput0104>, EnchantProductionError0104> {
        match decode_enchant_gameplay_frame_0104(frame) {
            EnchantGameplayFrame0104::Passthrough(_) => Ok(None),
            EnchantGameplayFrame0104::Malformed { frame, error } => {
                Err(EnchantProductionError0104::MalformedFrame { frame, error })
            }
            EnchantGameplayFrame0104::Decoded { packet, .. } => self.apply_reply(packet).map(Some),
        }
    }

    pub fn apply_reply(
        &mut self,
        packet: EnchantReplyPacket0104,
    ) -> Result<EnchantProductionOutput0104, EnchantProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(EnchantProductionError0104::NotActive)?;
        let reply_operation = packet.operation();
        let Some(pending) = session.pending_request.clone() else {
            return Err(EnchantProductionError0104::NoPendingRequest {
                reply: reply_operation,
            });
        };
        if pending.operation() != reply_operation {
            return Err(EnchantProductionError0104::UnexpectedReply {
                pending: pending.operation(),
                reply: reply_operation,
            });
        }

        let mut output = EnchantProductionOutput0104::default();
        match (pending, packet) {
            (PendingRequest0104::Enchant(pending), EnchantReplyPacket0104::Success(reply)) => {
                validate_enchant_success_identity(pending.request, reply)?;
                let disposition = session.model.receive_success(reply)?;
                let receipts = drain_model_intents(&mut session, &mut output)?;
                match disposition {
                    EnchantReplyDisposition0104::SuccessOverlay
                    | EnchantReplyDisposition0104::FailureMessage => {
                        let receipt = one_authoritative_receipt(receipts)?;
                        let commit = commit_enchant_reply(&session.snapshot, reply, &receipt)?;
                        session.snapshot = commit.snapshot_after.clone();
                        session.context.player.taros = commit.taros_after();
                        output.commit = Some(commit);
                    }
                    EnchantReplyDisposition0104::IgnoredLegacyFlag => {
                        if !receipts.is_empty() {
                            return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
                        }
                    }
                }
                session.pending_request = None;
            }
            (PendingRequest0104::Enchant(pending), EnchantReplyPacket0104::Failure(failure)) => {
                validate_enchant_failure_identity(pending.request, failure)?;
                session.model.receive_failure(failure)?;
                let receipts = drain_model_intents(&mut session, &mut output)?;
                if !receipts.is_empty() {
                    return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
                }
                session.pending_request = None;
            }
            (PendingRequest0104::Delete(pending), EnchantReplyPacket0104::DeleteSuccess(reply)) => {
                let commit = commit_delete_reply(&session.snapshot, &pending, reply)?;
                session.snapshot = commit.snapshot_after.clone();
                session
                    .model
                    .receive_auxiliary_unlock(ENCHANT_DELETE_SUCCESS_PACKET_ID_0104);
                let receipts = drain_model_intents(&mut session, &mut output)?;
                if !receipts.is_empty() {
                    return Err(EnchantProductionError0104::UnexpectedAuthoritativeReceipt);
                }
                session.pending_request = None;
                output.commit = Some(commit);
                output
                    .effects
                    .push(EnchantShellEffect0104::RefreshInventory);
            }
            (_, EnchantReplyPacket0104::DisassembleSuccess(_)) => {
                return Err(EnchantProductionError0104::UnsupportedAuxiliaryReply {
                    packet_type: ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104,
                });
            }
            (_, EnchantReplyPacket0104::DisassembleFailure(_)) => {
                return Err(EnchantProductionError0104::UnsupportedAuxiliaryReply {
                    packet_type: ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104,
                });
            }
            _ => unreachable!("operation equality exhausts reply family"),
        }
        self.session = Some(session);
        Ok(output)
    }
}
