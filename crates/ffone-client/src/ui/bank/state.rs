use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct BankModeProjection0104 {
    pub owner_pc_id: i32,
    pub npc_id: i32,
    pub extra_bank: i32,
    pub bank: [BankSlotProjection0104; BANK_SLOT_COUNT_0104],
    pub item_mode: UserEquipItemModeProjection,
}

impl Default for BankModeProjection0104 {
    fn default() -> Self {
        let empty = ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        };
        let catalog = EmptyBankCatalog;
        let eligibility = BankFailClosedEquipEligibility;
        Self {
            owner_pc_id: 0,
            npc_id: 0,
            extra_bank: 0,
            bank: array::from_fn(|slot_index| {
                BankSlotProjection0104::from_authoritative(
                    slot_index,
                    empty,
                    slot_index >= BANK_HALF_ACCESS_SLOT_COUNT,
                    &catalog,
                    &eligibility,
                )
            }),
            item_mode: UserEquipItemModeProjection::default(),
        }
    }
}

impl BankModeProjection0104 {
    pub fn from_authoritative_open(
        owner_pc_id: i32,
        npc_id: i32,
        open: &PcBankOpenSuccess0104,
        inventory: &InventoryRuntime0104,
        catalog: &impl UserEquipItemCatalog,
        equip_eligibility: &impl BankEquipEligibility,
    ) -> Result<Self, BankProjectionError0104> {
        let snapshot =
            BankAuthoritativeSnapshot0104::from_open_success(owner_pc_id, npc_id, open, inventory)?;
        Ok(Self::from_authoritative_snapshot(
            &snapshot,
            catalog,
            equip_eligibility,
        ))
    }

    #[must_use]
    pub fn from_authoritative_snapshot(
        snapshot: &BankAuthoritativeSnapshot0104,
        catalog: &impl UserEquipItemCatalog,
        equip_eligibility: &impl BankEquipEligibility,
    ) -> Self {
        let half = snapshot.extra_bank != BANK_FULL_ACCESS_VALUE;
        let inventory = snapshot.inventory_runtime();
        Self {
            owner_pc_id: snapshot.owner_pc_id,
            npc_id: snapshot.npc_id,
            extra_bank: snapshot.extra_bank,
            bank: array::from_fn(|slot_index| {
                BankSlotProjection0104::from_authoritative(
                    slot_index,
                    snapshot.bank[slot_index],
                    half && slot_index >= BANK_HALF_ACCESS_SLOT_COUNT,
                    catalog,
                    equip_eligibility,
                )
            }),
            item_mode: UserEquipItemModeProjection::from_authoritative(&inventory, catalog),
        }
    }

    /// Replaces the complete visual projection from current server authority.
    /// This is intentionally the only post-open update path.
    pub fn rebuild_from_authoritative_snapshot(
        &mut self,
        snapshot: &BankAuthoritativeSnapshot0104,
        catalog: &impl UserEquipItemCatalog,
        equip_eligibility: &impl BankEquipEligibility,
    ) {
        *self = Self::from_authoritative_snapshot(snapshot, catalog, equip_eligibility);
    }

    #[must_use]
    pub const fn has_full_access(&self) -> bool {
        self.extra_bank == BANK_FULL_ACCESS_VALUE
    }

    #[must_use]
    pub const fn accessible_bank_slots(&self) -> usize {
        if self.has_full_access() {
            BANK_SLOT_COUNT_0104
        } else {
            BANK_HALF_ACCESS_SLOT_COUNT
        }
    }

    #[must_use]
    pub fn item_at(&self, slot: BankSlotRef0104) -> ItemBase0104 {
        match slot.location {
            BankSlotLocation0104::Inventory => self.item_mode.inventory[slot.index].item.item,
            BankSlotLocation0104::Bank => self.bank[slot.index].item,
        }
    }

    #[must_use]
    pub fn slot_locked(&self, slot: BankSlotRef0104) -> bool {
        slot.location == BankSlotLocation0104::Bank && self.bank[slot.index].locked
    }

    pub fn transfer_intent(
        &self,
        from: BankSlotRef0104,
        to: BankSlotRef0104,
    ) -> Result<BankTransferIntent0104, BankTransferError0104> {
        if from == to {
            return Err(BankTransferError0104::SameSlotNoOp { slot: from });
        }
        if self.slot_locked(from) {
            return Err(BankTransferError0104::LockedBankSlot { slot: from });
        }
        if self.slot_locked(to) {
            return Err(BankTransferError0104::LockedBankSlot { slot: to });
        }
        if InventoryRuntime0104::item_is_empty(self.item_at(from)) {
            return Err(BankTransferError0104::EmptySource { slot: from });
        }
        Ok(BankTransferIntent0104 { from, to })
    }

    pub fn one_click_intent(
        &self,
        from: BankSlotRef0104,
    ) -> Result<BankTransferIntent0104, BankTransferError0104> {
        if self.slot_locked(from) {
            return Err(BankTransferError0104::LockedBankSlot { slot: from });
        }
        if InventoryRuntime0104::item_is_empty(self.item_at(from)) {
            return Err(BankTransferError0104::EmptySource { slot: from });
        }
        let (destination, limit, full_error) = match from.location {
            BankSlotLocation0104::Inventory => (
                BankSlotLocation0104::Bank,
                self.accessible_bank_slots(),
                BankTransferError0104::BankFull,
            ),
            BankSlotLocation0104::Bank => (
                BankSlotLocation0104::Inventory,
                INVENTORY_SLOT_COUNT_0104,
                BankTransferError0104::InventoryFull,
            ),
        };
        for index in 0..limit {
            let to = BankSlotRef0104::new(destination, index)
                .expect("clean destination loop is inside fixed capacity");
            if InventoryRuntime0104::item_is_empty(self.item_at(to)) {
                return self.transfer_intent(from, to);
            }
        }
        Err(full_error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct BankModalState {
    pub help: bool,
    pub inventory_popup: bool,
    pub system_popup: bool,
    pub generic_popup: bool,
}

impl BankModalState {
    #[must_use]
    pub const fn any(self) -> bool {
        self.help || self.inventory_popup || self.system_popup || self.generic_popup
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct BankUiState {
    pub phase: BankLifecyclePhase,
    pub opening_elapsed_seconds: f32,
    pub bank_scroll_y: f32,
    pub inventory_scroll_y: f32,
    pub send_pending: bool,
}

impl Default for BankUiState {
    fn default() -> Self {
        Self {
            phase: BankLifecyclePhase::Hidden,
            opening_elapsed_seconds: 0.0,
            bank_scroll_y: 0.0,
            inventory_scroll_y: 0.0,
            send_pending: false,
        }
    }
}

impl BankUiState {
    pub fn begin_open(&mut self, pc_id: i32, npc_id: i32, outbox: &mut BankUiOutbox0104) {
        self.phase = BankLifecyclePhase::Opening;
        self.opening_elapsed_seconds = 0.0;
        self.bank_scroll_y = 0.0;
        self.inventory_scroll_y = 0.0;
        self.send_pending = true;
        outbox.push(BankUiCommand0104::Open(PcBankOpenRequest0104 {
            pc_id,
            npc_id,
        }));
    }

    pub fn accept_open_success(&mut self) {
        self.send_pending = false;
    }

    pub fn accept_item_move_success(&mut self) {
        self.send_pending = false;
    }

    pub fn reject_server_request(&mut self) {
        self.send_pending = false;
    }

    pub fn close_after_failure(&mut self) {
        *self = Self::default();
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        if self.phase != BankLifecyclePhase::Opening {
            return;
        }
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.opening_elapsed_seconds =
                (self.opening_elapsed_seconds + delta_seconds).min(BANK_OPEN_SECONDS);
        }
        if self.opening_elapsed_seconds >= BANK_OPEN_SECONDS {
            self.phase = BankLifecyclePhase::Visible;
        }
    }

    #[must_use]
    pub const fn input_capabilities(self, modal: BankModalState) -> BankInputCapabilities {
        let draw = !matches!(self.phase, BankLifecyclePhase::Hidden);
        let controls =
            matches!(self.phase, BankLifecyclePhase::Visible) && !self.send_pending && !modal.any();
        BankInputCapabilities {
            draw,
            close: controls,
            scroll: controls,
            item_move: controls,
        }
    }

    pub fn apply_scroll_axis(&mut self, target: BankScrollTarget, axis: f32) {
        if !axis.is_finite() {
            return;
        }
        let legacy_value = axis.clamp(-1.0, 1.0) * BANK_SCROLL_VELOCITY;
        match target {
            BankScrollTarget::Bank => {
                self.bank_scroll_y = clamp_bank_scroll(self.bank_scroll_y - legacy_value);
            }
            BankScrollTarget::Inventory => {
                self.inventory_scroll_y = crate::user_equip_ui::clamp_user_equip_scroll(
                    self.inventory_scroll_y - legacy_value,
                );
            }
        }
    }

    pub fn request_transfer(
        &mut self,
        modal: BankModalState,
        projection: &BankModeProjection0104,
        from: BankSlotRef0104,
        to: BankSlotRef0104,
        outbox: &mut BankUiOutbox0104,
    ) -> Result<(), BankActionBlocked> {
        if !self.input_capabilities(modal).item_move {
            return Err(BankActionBlocked::ControlsDisabled);
        }
        let intent = projection
            .transfer_intent(from, to)
            .map_err(BankActionBlocked::Transfer)?;
        outbox.push(BankUiCommand0104::ItemMove(intent.wire_request()));
        self.send_pending = true;
        Ok(())
    }

    pub fn request_one_click(
        &mut self,
        modal: BankModalState,
        projection: &BankModeProjection0104,
        from: BankSlotRef0104,
        outbox: &mut BankUiOutbox0104,
    ) -> Result<(), BankActionBlocked> {
        if !self.input_capabilities(modal).item_move {
            return Err(BankActionBlocked::ControlsDisabled);
        }
        let intent = projection
            .one_click_intent(from)
            .map_err(BankActionBlocked::Transfer)?;
        outbox.push(BankUiCommand0104::ItemMove(intent.wire_request()));
        self.send_pending = true;
        Ok(())
    }

    pub fn request_close(
        &mut self,
        modal: BankModalState,
        outbox: &mut BankUiOutbox0104,
    ) -> Result<(), BankActionBlocked> {
        if !self.input_capabilities(modal).close {
            return Err(BankActionBlocked::ControlsDisabled);
        }
        // Clean `cnBank.BankOut` exits locally and does not send the available
        // bank-close request packet.
        outbox.push(BankUiCommand0104::ExitMode);
        *self = Self::default();
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BankModeView {
    pub layout: BankModeLayout,
    pub controls_enabled: bool,
    pub bank: [BankItemView; BANK_SLOT_COUNT_0104],
    pub inventory: [BankItemView; INVENTORY_SLOT_COUNT_0104],
    pub equipment: [BankItemView; USER_EQUIP_EQUIPMENT_STRIP_COUNT],
    pub taros_digits: [String; BANK_PC_STUFF_TAROS_DIGIT_RECTS.len()],
}

#[must_use]
pub fn bank_mode_view(
    viewport_width: u32,
    viewport_height: u32,
    state: BankUiState,
    modal: BankModalState,
    projection: &BankModeProjection0104,
    static_assets_ready: bool,
) -> Option<BankModeView> {
    bank_mode_view_with_pc_stuff(
        viewport_width,
        viewport_height,
        state,
        modal,
        projection,
        BankPcStuffAuthority0104::default(),
        static_assets_ready,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn bank_mode_view_with_pc_stuff(
    viewport_width: u32,
    viewport_height: u32,
    state: BankUiState,
    modal: BankModalState,
    projection: &BankModeProjection0104,
    pc_stuff: BankPcStuffAuthority0104,
    static_assets_ready: bool,
) -> Option<BankModeView> {
    let capabilities = state.input_capabilities(modal);
    if viewport_width == 0 || viewport_height == 0 || !capabilities.draw || !static_assets_ready {
        return None;
    }
    let layout = bank_mode_layout(
        viewport_width,
        viewport_height,
        state.opening_elapsed_seconds,
        state.bank_scroll_y,
        state.inventory_scroll_y,
    );
    let bank = array::from_fn(|slot| {
        let projected = &projection.bank[slot];
        BankItemView {
            frame_visual: if projected.locked {
                BankSlotFrameVisual::Locked
            } else if projected.equip_validation == BankEquipValidation::Rejected {
                BankSlotFrameVisual::Rejected
            } else if projected.empty {
                BankSlotFrameVisual::Empty
            } else {
                BankSlotFrameVisual::Occupied
            },
            icon: UserEquipPresentationIcon::from_projection(&projected.icon),
            show_combined_badge: projected.show_combined_badge,
            count_label: projected.count_label.clone(),
        }
    });
    let inventory = array::from_fn(|slot| {
        let projected = &projection.item_mode.inventory[slot].item;
        BankItemView {
            frame_visual: if projected.empty {
                BankSlotFrameVisual::Empty
            } else {
                BankSlotFrameVisual::Occupied
            },
            icon: UserEquipPresentationIcon::from_projection(&projected.icon),
            show_combined_badge: projected.show_combined_badge,
            count_label: if projected.empty || projected.item.item_type != 7 {
                None
            } else {
                Some(projected.item.option.to_string())
            },
        }
    });
    let equipment = array::from_fn(|slot| {
        let projected = &projection.item_mode.equipment[slot].item;
        BankItemView {
            frame_visual: if projected.empty {
                BankSlotFrameVisual::Empty
            } else {
                BankSlotFrameVisual::Occupied
            },
            icon: UserEquipPresentationIcon::from_projection(&projected.icon),
            show_combined_badge: projected.show_combined_badge,
            count_label: None,
        }
    });
    Some(BankModeView {
        layout,
        controls_enabled: capabilities.item_move,
        bank,
        inventory,
        equipment,
        taros_digits: bank_taros_counter_digits_0104(pc_stuff.taros()),
    })
}

#[derive(Clone, Resource)]
pub(super) struct BankUiRuntimeAssets(pub(super) BankUiAssets);
