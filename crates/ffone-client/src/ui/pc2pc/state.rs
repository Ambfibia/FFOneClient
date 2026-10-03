use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct Pc2pcModalState {
    pub help_active: bool,
    pub menu_chat_level_one_visible: bool,
    pub system_popup_active: bool,
    pub generic_popup_active: bool,
    pub inventory_popup_modal: bool,
    pub external_exit_blocked: bool,
    pub combat_active: bool,
}

impl Pc2pcModalState {
    #[must_use]
    pub const fn controls_enabled(self) -> bool {
        !self.help_active
            && !self.menu_chat_level_one_visible
            && !self.system_popup_active
            && !self.generic_popup_active
            && !self.inventory_popup_modal
            && !self.combat_active
    }

    #[must_use]
    pub const fn escape_allowed(self) -> bool {
        self.controls_enabled() && !self.external_exit_blocked
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct Pc2pcUiState {
    pub phase: Pc2pcLifecyclePhase,
    pub opening_elapsed: f32,
    pub local_ready: bool,
    pub remote_ready: bool,
    pub accept_pulse: f32,
    pub(super) pulse_forward: bool,
    pub chat_scroll_y: f32,
    pub pending: Option<Pc2pcPendingRequest0104>,
    pub last_failure: Option<Pc2pcLastFailure0104>,
}

impl Default for Pc2pcUiState {
    fn default() -> Self {
        Self {
            phase: Pc2pcLifecyclePhase::Hidden,
            opening_elapsed: 0.0,
            local_ready: false,
            remote_ready: false,
            accept_pulse: 0.0,
            pulse_forward: true,
            chat_scroll_y: 0.0,
            pending: None,
            last_failure: None,
        }
    }
}

impl Pc2pcUiState {
    #[must_use]
    pub fn local_visual_ready(&self) -> bool {
        self.local_ready || matches!(self.pending, Some(Pc2pcPendingRequest0104::Confirm(_)))
    }

    #[must_use]
    pub const fn button_label(&self) -> Pc2pcButtonLabel {
        if self.remote_ready {
            Pc2pcButtonLabel::Accept
        } else {
            Pc2pcButtonLabel::Submit
        }
    }

    #[must_use]
    pub fn button_enabled(&self, modal: Pc2pcModalState) -> bool {
        self.phase.accepts_actions()
            && modal.controls_enabled()
            && self.pending.is_none()
            && (self.remote_ready || !self.local_visual_ready())
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        let delta_seconds = delta_seconds.max(0.0);
        if self.phase == Pc2pcLifecyclePhase::Opening {
            self.opening_elapsed = (self.opening_elapsed + delta_seconds).min(PC2PC_OPEN_SECONDS);
            if self.opening_elapsed >= PC2PC_OPEN_SECONDS {
                self.phase = Pc2pcLifecyclePhase::Trading;
            }
        }
        if self.remote_ready {
            if self.pulse_forward {
                self.accept_pulse += delta_seconds;
                if self.accept_pulse >= 1.0 {
                    self.accept_pulse = 1.0;
                    self.pulse_forward = false;
                }
            } else {
                self.accept_pulse -= delta_seconds;
                if self.accept_pulse <= 0.0 {
                    self.accept_pulse = 0.0;
                    self.pulse_forward = true;
                }
            }
        } else {
            self.accept_pulse = 0.0;
            self.pulse_forward = true;
        }
    }

    pub fn scroll_chat(&mut self, raw_delta: f32, line_count: usize) {
        let delta = raw_delta.clamp(-30.0, 30.0);
        let content_height = line_count as f32 * PC2PC_CHAT_LINE_HEIGHT + 16.0;
        let max = (content_height - PC2PC_CHAT_LIST_VIEW_RECT.height).max(0.0);
        self.chat_scroll_y = (self.chat_scroll_y - delta).clamp(0.0, max);
    }

    pub(super) fn reset_readiness(&mut self) {
        self.local_ready = false;
        self.remote_ready = false;
        self.accept_pulse = 0.0;
        self.pulse_forward = true;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct Pc2pcModeProjection0104 {
    pub local_offer: [Pc2pcOfferSlotProjection0104; PC2PC_OFFER_SLOT_COUNT],
    pub remote_offer: [Pc2pcOfferSlotProjection0104; PC2PC_OFFER_SLOT_COUNT],
    pub item_mode: UserEquipItemModeProjection,
    pub local_wallet_taros: i32,
    pub local_offer_taros: i32,
    pub remote_offer_taros: i32,
    pub local_name: String,
    pub remote_name: String,
    pub authority_revision: u64,
}

impl Default for Pc2pcModeProjection0104 {
    fn default() -> Self {
        Self {
            local_offer: array::from_fn(|slot| Pc2pcOfferSlotProjection0104 {
                slot_index: slot,
                item: None,
                icon: UserEquipProjectedIcon::Empty,
                equip_validation: Pc2pcEquipValidation::NotApplicable,
                count_label: None,
            }),
            remote_offer: array::from_fn(|slot| Pc2pcOfferSlotProjection0104 {
                slot_index: slot,
                item: None,
                icon: UserEquipProjectedIcon::Empty,
                equip_validation: Pc2pcEquipValidation::NotApplicable,
                count_label: None,
            }),
            item_mode: UserEquipItemModeProjection::default(),
            local_wallet_taros: 0,
            local_offer_taros: 0,
            remote_offer_taros: 0,
            local_name: String::new(),
            remote_name: String::new(),
            authority_revision: 0,
        }
    }
}

impl Pc2pcModeProjection0104 {
    #[must_use]
    pub fn from_authoritative(
        snapshot: &Pc2pcAuthoritativeSnapshot0104,
        catalog: &impl UserEquipItemCatalog,
        eligibility: &impl Pc2pcEquipEligibility,
    ) -> Self {
        let runtime = snapshot.inventory_runtime_from_available();
        Self {
            local_offer: array::from_fn(|slot| {
                Pc2pcOfferSlotProjection0104::project(
                    slot,
                    snapshot.local_offer[slot],
                    catalog,
                    eligibility,
                )
            }),
            remote_offer: array::from_fn(|slot| {
                Pc2pcOfferSlotProjection0104::project(
                    slot,
                    snapshot.remote_offer[slot],
                    catalog,
                    eligibility,
                )
            }),
            item_mode: UserEquipItemModeProjection::from_authoritative(&runtime, catalog),
            local_wallet_taros: snapshot.local_wallet_taros,
            local_offer_taros: snapshot.local_offer_taros,
            remote_offer_taros: snapshot.remote_offer_taros,
            local_name: snapshot.names.local.clone(),
            remote_name: snapshot.names.remote.clone(),
            authority_revision: snapshot.revision,
        }
    }
}

pub(super) fn tick_pc2pc_state(time: Res<Time>, mut model: ResMut<Pc2pcUiModel0104>) {
    model.state.tick(time.delta_secs());
}
