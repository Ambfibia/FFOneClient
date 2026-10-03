use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct ResurrectUiModel {
    pub visible: bool,
    pub request_sent: bool,
    pub nano_selection_was_open: bool,
    pub elapsed_seconds: f32,
    pub last_blocker: Option<ResurrectRequestBlocker>,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for ResurrectUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            request_sent: false,
            nano_selection_was_open: false,
            elapsed_seconds: 0.0,
            last_blocker: None,
            ui_scale_override: None,
        }
    }
}

impl ResurrectUiModel {
    /// Repeated dead status packets belong to the same death. Preserve its
    /// deadline and any pending regeneration request until server success.
    pub fn enter_from_authoritative_death(
        &mut self,
        entry_element: i32,
        outbox: &mut ResurrectUiOutbox,
    ) {
        if !self.visible {
            self.enter(entry_element, outbox);
        }
    }

    /// Enters the mode. Clean `InitMode` treats entry element `22` as the
    /// "Nano selection was open" marker.
    pub fn enter(&mut self, entry_element: i32, outbox: &mut ResurrectUiOutbox) {
        self.visible = true;
        self.request_sent = false;
        self.nano_selection_was_open = entry_element == 22;
        self.elapsed_seconds = 0.0;
        self.last_blocker = None;
        outbox.push(ResurrectUiAction::Entered {
            effects: CLEAN_RESURRECT_ENTER_EFFECTS,
        });
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_ui_scale(&mut self, ui_scale: f32) {
        self.ui_scale_override = Some(valid_ui_scale(ui_scale));
    }

    pub fn clear_ui_scale_override(&mut self) {
        self.ui_scale_override = None;
    }

    #[must_use]
    pub const fn ui_scale_override(&self) -> Option<f32> {
        self.ui_scale_override
    }

    #[must_use]
    pub fn effective_ui_scale(&self, viewport_height: f32) -> f32 {
        self.ui_scale_override
            .unwrap_or_else(|| clean_resurrect_ui_scale(viewport_height))
    }

    #[must_use]
    pub fn countdown_seconds(&self) -> i32 {
        // Native usability correction: keep zero visible while the server
        // response or a temporarily unavailable respawn context is pending.
        (RESURRECT_TIMEOUT_SECONDS - self.elapsed_seconds)
            .max(0.0)
            .floor() as i32
    }

    #[must_use]
    pub const fn controls_enabled(&self, context: ResurrectUiContext) -> bool {
        self.visible && !self.request_sent && !context.system_popup_active
    }

    #[must_use]
    pub const fn input_boundary(&self, context: ResurrectUiContext) -> ResurrectInputBoundary {
        ResurrectInputBoundary {
            blocks_lower_ui: self.visible,
            blocks_gameplay_input: self.visible,
            requires_pointer: self.visible,
            cursor_locked: false,
            mouse_controls_enabled: self.controls_enabled(context),
        }
    }

    pub fn activate_choice(
        &mut self,
        choice: ResurrectChoice,
        context: ResurrectUiContext,
        outbox: &mut ResurrectUiOutbox,
    ) -> Result<ResurrectRegenRequest, ResurrectRequestBlocker> {
        let result = self.activate_choice_inner(
            choice,
            context,
            ResurrectRequestOrigin::Manual,
            true,
            outbox,
        );
        self.last_blocker = result.as_ref().err().copied();
        result
    }

    /// Advances `Time.time - fStartTime`.
    ///
    /// When the game is not ready, clean `Update` writes `fStartTime =
    /// Time.time` every frame, so elapsed time becomes zero. Timeout uses a
    /// strict `> 60` comparison and is independent of `GUI.enabled`; a system
    /// popup therefore blocks mouse buttons but not automatic resurrection.
    pub fn advance(
        &mut self,
        delta_seconds: f32,
        context: ResurrectUiContext,
        outbox: &mut ResurrectUiOutbox,
    ) -> Option<ResurrectRegenRequest> {
        if !self.visible {
            return None;
        }
        if !context.ready_for_play {
            self.elapsed_seconds = 0.0;
            return None;
        }
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.elapsed_seconds += delta_seconds;
        }
        if !context.player_available
            || self.request_sent
            || self.elapsed_seconds <= RESURRECT_TIMEOUT_SECONDS
        {
            return None;
        }

        let result = self.activate_choice_inner(
            ResurrectChoice::NearestResurrectEm,
            context,
            ResurrectRequestOrigin::StrictTimeout,
            false,
            outbox,
        );
        self.last_blocker = result.as_ref().err().copied();
        result.ok()
    }

    pub fn accept_regen_success(&mut self, outbox: &mut ResurrectUiOutbox) -> bool {
        if !self.visible {
            return false;
        }
        let restore_nano_selection = self.nano_selection_was_open;
        self.visible = false;
        outbox.push(ResurrectUiAction::RegenSucceeded {
            effects: ResurrectSuccessEffects {
                grayscale_enabled: false,
                cursor_locked: true,
                return_to_normal_mode: true,
                restore_nano_selection,
            },
        });
        true
    }

    pub(super) fn activate_choice_inner(
        &mut self,
        choice: ResurrectChoice,
        context: ResurrectUiContext,
        origin: ResurrectRequestOrigin,
        enforce_gui_gate: bool,
        outbox: &mut ResurrectUiOutbox,
    ) -> Result<ResurrectRegenRequest, ResurrectRequestBlocker> {
        if !self.visible {
            return Err(ResurrectRequestBlocker::Hidden);
        }
        if self.request_sent {
            return Err(ResurrectRequestBlocker::AlreadySent);
        }
        if enforce_gui_gate {
            if context.system_popup_active {
                return Err(ResurrectRequestBlocker::SystemPopup);
            }
            if !context.clean_controls_gate_open() {
                return Err(ResurrectRequestBlocker::ControlsGateClosed);
            }
            if !context.choice_is_visible(choice) {
                return Err(ResurrectRequestBlocker::ChoiceNotVisible);
            }
        }

        let (e_il, index) = match choice {
            ResurrectChoice::UseItem => (
                1,
                context
                    .resurrection_item_slot
                    .ok_or(ResurrectRequestBlocker::ItemSlotUnresolved)?,
            ),
            ResurrectChoice::NearestResurrectEm
            | ResurrectChoice::PhoenixSelf
            | ResurrectChoice::PhoenixGroup => (
                0,
                context
                    .nearest_xcom_index
                    .ok_or(ResurrectRequestBlocker::NearestXcomUnresolved)?,
            ),
        };
        let request = ResurrectRegenRequest {
            choice,
            regen_type: choice.regen_type(),
            e_il,
            index,
            origin,
        };
        self.request_sent = true;
        outbox.push(ResurrectUiAction::RequestRegen(request));
        Ok(request)
    }
}
