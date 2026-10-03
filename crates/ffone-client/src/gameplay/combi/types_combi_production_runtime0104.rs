use super::*;

impl CombiProductionRuntime0104 {
    #[must_use]
    pub const fn session(&self) -> Option<&CombiProductionSession0104> {
        self.session.as_ref()
    }

    #[must_use]
    pub const fn mode_lease(&self) -> Option<CombiModeLease0104> {
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
    pub const fn request_pending(&self) -> bool {
        match &self.session {
            Some(session) => session.pending_request.is_some(),
            None => false,
        }
    }

    pub fn reset(&mut self) {
        self.session = None;
    }

    pub fn open(
        &mut self,
        context: CombiOpenContext0104,
        inventory: &InventoryRuntime0104,
        catalog: &impl CombiItemCatalog0104,
        recipes: &CombiRecipeTable0104,
    ) -> Result<CombiProductionOutput0104, CombiProductionError0104> {
        if self.session.is_some() {
            return Err(CombiProductionError0104::AlreadyActive);
        }
        validate_open_context(context, inventory)?;
        let snapshot = snapshot_from_authority(inventory, context.player)?;
        let mut machine = CombiMachine0104::default();
        machine.open();
        let projection = project_combi_mode_0104(&snapshot, machine.selection, catalog, recipes)?;
        let lease = CombiModeLease0104::from(context);
        self.session = Some(CombiProductionSession0104 {
            context,
            lease,
            machine,
            snapshot,
            projection,
            drag_source_inventory_index: None,
            pending_request: None,
        });
        Ok(CombiProductionOutput0104::from_effect(
            CombiShellEffect0104::ModeOpened { context, lease },
        ))
    }

    /// Refreshes local player authority only while the clean machine accepts
    /// main controls. Selected source items must remain byte-identical.
    pub fn refresh_authority(
        &mut self,
        inventory: &InventoryRuntime0104,
        player: CombiPlayerAuthority0104,
        catalog: &impl CombiItemCatalog0104,
        recipes: &CombiRecipeTable0104,
    ) -> Result<(), CombiProductionError0104> {
        let session = self
            .session
            .as_ref()
            .ok_or(CombiProductionError0104::NotActive)?;
        if session.machine.phase != CombiPhase0104::Ready {
            return Err(CombiProductionError0104::AuthorityRefreshBlocked {
                phase: session.machine.phase.clone(),
            });
        }
        validate_player_authority(session.context.player.owner_pc_id, inventory, player)?;
        let candidate = snapshot_from_authority(inventory, player)?;
        validate_selected_items_unchanged(session, &candidate)?;
        let projection =
            project_combi_mode_0104(&candidate, session.machine.selection, catalog, recipes)?;

        let session = self.session.as_mut().expect("active session was validated");
        session.context.player = player;
        session.snapshot = candidate;
        session.projection = projection;
        Ok(())
    }

    /// Copies the controller's authoritative phase/projection into the passive
    /// Bevy UI resources. Camera booleans remain shell-owned while active.
    pub fn write_presentation(
        &self,
        state: &mut CombiUiState0104,
        projection: &mut CombiModeProjection0104,
    ) {
        if let Some(session) = &self.session {
            state.phase = session.machine.phase.clone();
            *projection = session.projection.clone();
        } else {
            state.phase = CombiPhase0104::Hidden;
            state.primary_npc_camera_bound = false;
            state.waiting_npc_camera_bound = false;
            *projection = CombiModeProjection0104::default();
        }
    }

    /// Pops and handles at most one passive UI command. The command is
    /// consumed even when validation rejects it, matching an input event.
    pub fn apply_next_ui_command(
        &mut self,
        outbox: &mut CombiUiOutbox0104,
        catalog: &impl CombiItemCatalog0104,
        recipes: &CombiRecipeTable0104,
    ) -> Option<Result<CombiProductionOutput0104, CombiProductionError0104>> {
        outbox
            .pop_front()
            .map(|command| self.apply_ui_command(command, catalog, recipes))
    }

    pub fn apply_ui_command(
        &mut self,
        command: CombiUiCommand0104,
        catalog: &impl CombiItemCatalog0104,
        recipes: &CombiRecipeTable0104,
    ) -> Result<CombiProductionOutput0104, CombiProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(CombiProductionError0104::NotActive)?;
        let mut output = CombiProductionOutput0104::default();
        let mut terminal = None;

        match command {
            CombiUiCommand0104::BeginInventoryDrag { inventory_index } => {
                require_ready(&session.machine)?;
                let Some(item) = session.snapshot.inventory.get(inventory_index).copied() else {
                    return Err(CombiProductionError0104::DragSourceOutOfBounds {
                        inventory_index,
                    });
                };
                if CombiAuthoritativeSnapshot0104::item_is_empty(item) {
                    return Err(CombiProductionError0104::EmptyDragSource { inventory_index });
                }
                if session
                    .machine
                    .selection
                    .contains_inventory_index(inventory_index)
                {
                    return Err(CombiProductionError0104::DragSourceAlreadySelected {
                        inventory_index,
                    });
                }
                session.drag_source_inventory_index = Some(inventory_index);
                output
                    .effects
                    .push(CombiShellEffect0104::DragCaptured { inventory_index });
            }
            CombiUiCommand0104::CancelInventoryDrag => {
                // A release away from both drop targets puts the item back;
                // a later drop must not reuse the abandoned source.
                session.drag_source_inventory_index = None;
            }
            CombiUiCommand0104::RejectEquippedItem {
                equipment_index,
                message_id,
            } => {
                require_ready(&session.machine)?;
                if message_id != 260 {
                    return Err(CombiProductionError0104::WrongEquippedMessageId {
                        expected: 260,
                        actual: message_id,
                    });
                }
                let Some(item) = session.snapshot.equipment.get(equipment_index).copied() else {
                    return Err(CombiProductionError0104::EquipmentSlotOutOfBounds {
                        equipment_index,
                    });
                };
                if CombiAuthoritativeSnapshot0104::item_is_empty(item) {
                    return Err(CombiProductionError0104::EmptyEquipmentSlot { equipment_index });
                }
                output.effects.push(CombiShellEffect0104::SystemMessage(
                    CombiSystemMessage0104::EquippedItem {
                        message_id,
                        equipment_index,
                        item,
                    },
                ));
            }
            CombiUiCommand0104::DropOnSelection { slot } => {
                let inventory_index = session
                    .drag_source_inventory_index
                    .ok_or(CombiProductionError0104::MissingDragSource)?;
                let changes = session.machine.attach(
                    &session.snapshot,
                    crate::combi_ui::CombiSourceLocation0104::Inventory,
                    inventory_index,
                    slot,
                )?;
                session.projection = project_combi_mode_0104(
                    &session.snapshot,
                    session.machine.selection,
                    catalog,
                    recipes,
                )?;
                session.drag_source_inventory_index = None;
                output.add_selection_changes(changes);
            }
            CombiUiCommand0104::DetachSelection { slot } => {
                let changes = session.machine.detach(slot)?;
                session.projection = project_combi_mode_0104(
                    &session.snapshot,
                    session.machine.selection,
                    catalog,
                    recipes,
                )?;
                output.add_selection_changes(changes);
            }
            CombiUiCommand0104::ClearAll => {
                let changes = session.machine.clear_all()?;
                session.projection = project_combi_mode_0104(
                    &session.snapshot,
                    session.machine.selection,
                    catalog,
                    recipes,
                )?;
                session.drag_source_inventory_index = None;
                output.add_selection_changes(changes);
            }
            CombiUiCommand0104::Combine => {
                session
                    .machine
                    .begin_combine(&session.snapshot, &session.projection)?;
                session.drag_source_inventory_index = None;
                let CombiPhase0104::Modal(modal) = session.machine.phase.clone() else {
                    return Err(CombiProductionError0104::State(
                        CombiStateError0104::MissingPendingAttempt,
                    ));
                };
                output
                    .effects
                    .push(CombiShellEffect0104::SystemMessage(modal_message(
                        &session, modal,
                    )));
            }
            CombiUiCommand0104::Close => {
                session.machine.request_close()?;
                terminal = Some(CombiCloseReason0104::Close);
                output.effects.push(CombiShellEffect0104::ModeClosed {
                    context: session.context,
                    reason: CombiCloseReason0104::Close,
                });
            }
            CombiUiCommand0104::Help => {
                require_ready(&session.machine)?;
                output.effects.push(CombiShellEffect0104::HelpRequested);
            }
            CombiUiCommand0104::CombineMoreItems => {
                session.machine.combine_more_items()?;
                session.projection = project_combi_mode_0104(
                    &session.snapshot,
                    session.machine.selection,
                    catalog,
                    recipes,
                )?;
                session.drag_source_inventory_index = None;
            }
            CombiUiCommand0104::GoToMyStuff => {
                session.machine.go_to_my_stuff()?;
                terminal = Some(CombiCloseReason0104::GoToMyStuff);
                output.effects.extend([
                    CombiShellEffect0104::ModeClosed {
                        context: session.context,
                        reason: CombiCloseReason0104::GoToMyStuff,
                    },
                    CombiShellEffect0104::GoToMyStuffRequested {
                        game_mode: COMBI_MY_STUFF_GAME_MODE_0104,
                        first_use_condition: COMBI_GO_TO_STUFF_FIRST_USE_CONDITION_0104,
                    },
                ]);
            }
        }

        if terminal.is_some() {
            self.session = None;
        } else {
            self.session = Some(session);
        }
        Ok(output)
    }

    pub fn resolve_modal(
        &mut self,
        choice: CombiModalChoice0104,
        catalog: &impl CombiItemCatalog0104,
        recipes: &CombiRecipeTable0104,
    ) -> Result<CombiProductionOutput0104, CombiProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(CombiProductionError0104::NotActive)?;
        let phase_before = session.machine.phase.clone();
        let changes = session.machine.resolve_modal(choice)?;
        let mut output = CombiProductionOutput0104::default();
        output.add_selection_changes(changes);

        if matches!(
            (phase_before, choice),
            (
                CombiPhase0104::Modal(CombiSystemModal0104::AttemptConfirmation),
                CombiModalChoice0104::Continue
            )
        ) {
            output.effects.push(CombiShellEffect0104::NpcAnimation {
                npc_id: COMBI_NPC_ID_0104,
                animation: CombiNpcAnimation0104::MakingInOut,
            });
        }
        if session.machine.phase == CombiPhase0104::Ready {
            session.projection = project_combi_mode_0104(
                &session.snapshot,
                session.machine.selection,
                catalog,
                recipes,
            )?;
        }
        self.session = Some(session);
        Ok(output)
    }

    /// Advances clean `CombiWaiting`. Construction through the registered
    /// request envelope proves both ID registration and the exact 16-byte ABI.
    pub fn tick(
        &mut self,
        delta_seconds: f32,
    ) -> Result<CombiProductionOutput0104, CombiProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(CombiProductionError0104::NotActive)?;
        if session.pending_request.is_some() {
            return Err(CombiProductionError0104::RequestAlreadyPending);
        }
        let request = session.machine.tick_waiting(delta_seconds)?;
        let Some(request) = request else {
            self.session = Some(session);
            return Ok(CombiProductionOutput0104::default());
        };
        let registered = encode_combi_request_0104(request)?;
        session.pending_request = Some(request);
        self.session = Some(session);
        Ok(CombiProductionOutput0104 {
            request: Some(registered),
            commit: None,
            effects: vec![CombiShellEffect0104::NpcAnimation {
                npc_id: COMBI_NPC_ID_0104,
                animation: CombiNpcAnimation0104::Stand,
            }],
        })
    }

    pub fn apply_frame(
        &mut self,
        frame: DecodedFrame,
    ) -> Result<Option<CombiProductionOutput0104>, CombiProductionError0104> {
        match decode_combi_gameplay_frame_0104(frame) {
            CombiGameplayFrame0104::Passthrough(_) => Ok(None),
            CombiGameplayFrame0104::Malformed { frame, error } => {
                Err(CombiProductionError0104::MalformedFrame { frame, error })
            }
            CombiGameplayFrame0104::Decoded { packet, .. } => self.apply_reply(packet).map(Some),
        }
    }

    pub fn apply_reply(
        &mut self,
        packet: CombiReplyPacket0104,
    ) -> Result<CombiProductionOutput0104, CombiProductionError0104> {
        let mut session = self
            .session
            .as_ref()
            .cloned()
            .ok_or(CombiProductionError0104::NotActive)?;
        if session.pending_request.is_none() {
            return Err(CombiProductionError0104::NoPendingRequest);
        }

        let mut output = CombiProductionOutput0104::default();
        match packet {
            CombiReplyPacket0104::Success(reply) => {
                let receipt = session.machine.receive_authoritative_reply(
                    &session.snapshot,
                    &session.projection,
                    reply,
                )?;
                let commit = authoritative_commit(&session.snapshot, receipt)?;
                session.snapshot = commit.snapshot_after.clone();
                session.context.player.taros = commit.taros_after;
                session.pending_request = None;
                if reply.success_flag == 1 {
                    output.effects.push(CombiShellEffect0104::PlaySuccessSound {
                        true_name: COMBI_SUCCESS_SOUND_TRUE_NAME_0104,
                    });
                } else {
                    output
                        .effects
                        .push(CombiShellEffect0104::SystemMessage(modal_message(
                            &session,
                            CombiSystemModal0104::CombinationFailed,
                        )));
                }
                output.commit = Some(commit);
            }
            CombiReplyPacket0104::Failure(reply) => {
                session.machine.receive_wire_failure(reply)?;
                session.pending_request = None;
                output.effects.push(CombiShellEffect0104::SystemMessage(
                    CombiSystemMessage0104::WireFailure {
                        error_code: reply.error_code,
                    },
                ));
            }
        }
        self.session = Some(session);
        Ok(output)
    }
}
