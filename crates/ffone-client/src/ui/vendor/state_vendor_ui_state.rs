use super::*;

impl VendorUiState {
    pub fn begin_open(
        &mut self,
        requested_npc_id: i32,
        table_vendor_id: i32,
        outbox: &mut VendorUiOutbox0104,
    ) {
        self.phase = VendorLifecyclePhase::Opening;
        self.opening_elapsed_seconds = 0.0;
        self.tab = VendorTab0104::Buy;
        self.vendor_scroll_y = 0.0;
        self.inventory_scroll_y = 0.0;
        self.send_pending = true;
        outbox.push(VendorUiCommand0104::StartSession {
            requested_npc_id,
            table_vendor_id,
        });
    }

    pub fn accept_start_success(&mut self) {
        // Clean immediately sends table update, so controls remain disabled.
        self.send_pending = true;
    }

    pub fn accept_authoritative_table(&mut self) {
        self.send_pending = false;
    }

    pub fn accept_request_reply(&mut self) {
        // A success only releases the gate. The caller must separately replace
        // `VendorModeProjection0104` from authoritative state.
        self.send_pending = false;
    }

    pub fn close_after_failure(&mut self) {
        *self = Self::default();
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        if self.phase != VendorLifecyclePhase::Opening {
            return;
        }
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.opening_elapsed_seconds =
                (self.opening_elapsed_seconds + delta_seconds).min(VENDOR_OPEN_SECONDS);
        }
        if self.opening_elapsed_seconds >= VENDOR_OPEN_SECONDS {
            self.phase = VendorLifecyclePhase::Visible;
        }
    }

    #[must_use]
    pub const fn input_capabilities(self, modal: VendorModalState) -> VendorInputCapabilities {
        let draw = !matches!(self.phase, VendorLifecyclePhase::Hidden);
        let vendor_controls = matches!(self.phase, VendorLifecyclePhase::Visible)
            && !self.send_pending
            && !modal.vendor_panel_blocked();
        let pc_stuff_controls = matches!(self.phase, VendorLifecyclePhase::Visible)
            && !self.send_pending
            && !modal.pc_stuff_blocked();
        VendorInputCapabilities {
            draw,
            vendor_controls,
            pc_stuff_controls,
            tabs: vendor_controls,
            scroll: vendor_controls,
            row_actions: vendor_controls,
            close: pc_stuff_controls,
            help: pc_stuff_controls,
            go_to_stuff: vendor_controls,
        }
    }

    pub fn switch_tab(
        &mut self,
        modal: VendorModalState,
        tab: VendorTab0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).tabs {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        if self.tab != tab {
            self.tab = tab;
            self.vendor_scroll_y = 0.0;
        }
        Ok(())
    }

    pub fn apply_scroll_axis(
        &mut self,
        modal: VendorModalState,
        target: VendorScrollTarget,
        axis: f32,
        vendor_row_count: usize,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).scroll {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        if !axis.is_finite() {
            return Ok(());
        }
        match target {
            VendorScrollTarget::Vendor => {
                let legacy_value = (axis.clamp(-1.0, 1.0) * VENDOR_SCROLL_VELOCITY)
                    .clamp(-VENDOR_SCROLL_INPUT_CLAMP, VENDOR_SCROLL_INPUT_CLAMP);
                self.vendor_scroll_y =
                    clamp_vendor_scroll(self.vendor_scroll_y - legacy_value, vendor_row_count);
            }
            VendorScrollTarget::Inventory => {
                let legacy_value = axis.clamp(-1.0, 1.0) * VENDOR_SCROLL_VELOCITY;
                self.inventory_scroll_y = crate::user_equip_ui::clamp_user_equip_scroll(
                    self.inventory_scroll_y - legacy_value,
                );
            }
        }
        Ok(())
    }

    pub fn dispatch_outcome(
        &mut self,
        modal: VendorModalState,
        outcome: VendorActionOutcome0104,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).row_actions {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        self.enqueue_outcome(outcome, outbox);
        Ok(())
    }

    pub fn dispatch_inventory_outcome(
        &mut self,
        modal: VendorModalState,
        outcome: VendorActionOutcome0104,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).pc_stuff_controls {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        self.enqueue_outcome(outcome, outbox);
        Ok(())
    }

    pub fn dispatch_inventory_drop(
        &mut self,
        modal: VendorModalState,
        projection: &VendorModeProjection0104,
        inventory_slot: usize,
        target: VendorInventoryDropTarget0104,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).pc_stuff_controls {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        self.enqueue_outcome(
            projection.inventory_drop_activation(inventory_slot, target),
            outbox,
        );
        Ok(())
    }

    pub(super) fn enqueue_outcome(
        &mut self,
        outcome: VendorActionOutcome0104,
        outbox: &mut VendorUiOutbox0104,
    ) {
        match outcome {
            VendorActionOutcome0104::Intent(intent) => {
                outbox.push(VendorUiCommand0104::Intent(intent));
                self.send_pending = true;
            }
            VendorActionOutcome0104::SystemMessage(message) => {
                outbox.push(VendorUiCommand0104::ShowSystemMessage(message));
            }
            VendorActionOutcome0104::Confirmation(confirmation) => {
                outbox.push(VendorUiCommand0104::OpenConfirmation(confirmation));
            }
            VendorActionOutcome0104::SilentBlocked(_) => {}
        }
    }

    pub fn request_close_button(
        &mut self,
        modal: VendorModalState,
        gate: VendorCloseGate0104,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).close {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        if !gate.target_action_idle {
            return Err(VendorActionBlocked0104::CloseGateRejected);
        }
        outbox.push(VendorUiCommand0104::ExitMode);
        *self = Self::default();
        Ok(())
    }

    pub fn request_escape(
        &mut self,
        modal: VendorModalState,
        gate: VendorCloseGate0104,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        // Exact `cnVendor.Update` gate: !bSend, mode event accepts, no system
        // popup, and target action equals zero.
        if matches!(self.phase, VendorLifecyclePhase::Hidden) || self.send_pending {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        if !gate.mode_accepts_escape || !gate.target_action_idle || modal.system_popup {
            return Err(VendorActionBlocked0104::CloseGateRejected);
        }
        outbox.push(VendorUiCommand0104::ExitMode);
        *self = Self::default();
        Ok(())
    }

    pub fn request_help(
        &self,
        modal: VendorModalState,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).help {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        outbox.push(VendorUiCommand0104::OpenHelp {
            event_id: VENDOR_HELP_EVENT_ID,
        });
        Ok(())
    }

    pub fn request_redeem_code(
        &self,
        modal: VendorModalState,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).pc_stuff_controls {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        // Clean `Panel_PCStuffScript` sets its local redeem view and disables
        // the shared panel. Native popup ownership remains outside Vendor's
        // packet boundary, so this is a semantic presentation command only.
        outbox.push(VendorUiCommand0104::OpenRedeemCode);
        Ok(())
    }

    pub fn request_go_to_stuff(
        &mut self,
        modal: VendorModalState,
        outbox: &mut VendorUiOutbox0104,
    ) -> Result<(), VendorActionBlocked0104> {
        if !self.input_capabilities(modal).go_to_stuff {
            return Err(VendorActionBlocked0104::ControlsDisabled);
        }
        // The production owner maps this single semantic command to the clean
        // five-event `Panel_Vendor.OnGoToStuff` sequence; it does not imply a
        // packet.
        outbox.push(VendorUiCommand0104::GoToMyStuff);
        *self = Self::default();
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VendorModeView0104 {
    pub layout: VendorModeLayout,
    pub tab: VendorTab0104,
    pub controls: VendorInputCapabilities,
    pub npc_name: String,
    pub npc_title: String,
    pub npc_service: String,
    pub rows: Vec<VendorRowView0104>,
    pub inventory: [VendorSlotView0104; INVENTORY_SLOT_COUNT_0104],
    pub equipment: [VendorSlotView0104; EQUIPMENT_SLOT_COUNT_0104],
    pub taros: i32,
    pub weapon_battery: i32,
    pub nano_battery: i32,
}

#[must_use]
pub fn vendor_mode_view(
    viewport_width: u32,
    viewport_height: u32,
    state: VendorUiState,
    modal: VendorModalState,
    projection: &VendorModeProjection0104,
    static_assets_ready: bool,
) -> Option<VendorModeView0104> {
    let controls = state.input_capabilities(modal);
    if viewport_width == 0 || viewport_height == 0 || !controls.draw || !static_assets_ready {
        return None;
    }
    let row_count = projection.rows_for_tab(state.tab);
    let layout = vendor_mode_layout(
        viewport_width,
        viewport_height,
        state.opening_elapsed_seconds,
        state.vendor_scroll_y,
        state.inventory_scroll_y,
        row_count,
    );
    let rows = match state.tab {
        VendorTab0104::Buy => projection
            .catalog_rows
            .iter()
            .map(|row| {
                vendor_row_view(
                    row.item,
                    row.metadata.as_ref(),
                    &row.icon,
                    row.price,
                    row.affordable,
                    row.equip_validation,
                    row.vehicle_speed_class,
                    false,
                )
            })
            .collect(),
        VendorTab0104::Buyback => projection
            .recent_buy_rows
            .iter()
            .map(|row| {
                vendor_row_view(
                    row.item,
                    row.metadata.as_ref(),
                    &row.icon,
                    row.price,
                    row.affordable,
                    VendorEquipValidation0104::NotApplicable,
                    row.vehicle_speed_class,
                    true,
                )
            })
            .collect(),
    };
    let inventory = array::from_fn(|slot| vendor_slot_view(&projection.inventory[slot].item));
    let equipment = array::from_fn(|slot| vendor_slot_view(&projection.equipment[slot].item));
    Some(VendorModeView0104 {
        layout,
        tab: state.tab,
        controls,
        npc_name: projection.npc_name.clone(),
        npc_title: format!("SHOPKEEPER - {}", projection.npc_name),
        npc_service: projection.npc_service.clone(),
        rows,
        inventory,
        equipment,
        taros: projection.taros,
        weapon_battery: projection.weapon_battery,
        nano_battery: projection.nano_battery,
    })
}
