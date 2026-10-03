use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct GuideUiModel {
    pub visible: bool,
    pub purpose: GuideUiPurpose,
    pub phase: GuideUiPhase,
    pub current: Option<GuideMentor>,
    pub selected: Option<GuideMentor>,
    pub confirmation_open: bool,
    pub awaiting_server: bool,
    pub system_popup_open: bool,
    pub help_open: bool,
    pub(super) pending_mentor: Option<GuideMentor>,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for GuideUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            purpose: GuideUiPurpose::InitialSelection,
            phase: GuideUiPhase::WarpWarning,
            current: None,
            selected: None,
            confirmation_open: false,
            awaiting_server: false,
            system_popup_open: false,
            help_open: false,
            pending_mentor: None,
            ui_scale_override: None,
        }
    }
}

impl GuideUiModel {
    pub fn open_initial_selection(&mut self) {
        *self = Self {
            visible: true,
            ..Self::default()
        };
    }

    /// Opens the future/Computress initial-selection path after an NPC service
    /// already supplied a nonzero Guide context. Unlike accepting the Past
    /// Warp warning, this emits no click audio or synthetic UI action.
    pub fn open_initial_mentor_selection(&mut self) {
        *self = Self {
            visible: true,
            phase: GuideUiPhase::MentorSelection,
            ..Self::default()
        };
    }

    pub fn open_change(&mut self, current: GuideMentor) {
        *self = Self {
            visible: true,
            purpose: GuideUiPurpose::ChangeMentor,
            phase: GuideUiPhase::MentorSelection,
            current: Some(current),
            selected: Some(current),
            ..Self::default()
        };
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.confirmation_open = false;
        self.awaiting_server = false;
        self.pending_mentor = None;
    }

    /// Restore the selectable surface when the local transport could not
    /// enqueue a request. This is not a fabricated server rejection.
    pub fn cancel_pending_request(&mut self) {
        self.awaiting_server = false;
        self.pending_mentor = None;
    }

    pub fn set_external_modes(&mut self, system_popup_open: bool, help_open: bool) {
        self.system_popup_open = system_popup_open;
        self.help_open = help_open;
    }

    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale_override = Some(valid_scale(scale));
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
            .unwrap_or_else(|| clean_guide_ui_scale(viewport_height))
    }

    #[must_use]
    pub const fn controls_enabled(&self) -> bool {
        self.visible && !self.system_popup_open && !self.help_open && !self.awaiting_server
    }

    #[must_use]
    pub const fn displayed_price(&self) -> Option<u32> {
        match (self.purpose, self.selected) {
            (GuideUiPurpose::ChangeMentor, Some(mentor)) => {
                Some(GUIDE_CHANGE_PRICES[mentor.slot()])
            }
            _ => None,
        }
    }

    #[must_use]
    pub const fn input_boundary(&self) -> GuideUiInputBoundary {
        GuideUiInputBoundary {
            blocks_lower_ui: self.visible,
            blocks_gameplay_input: self.visible,
            requires_pointer: self.visible,
            mouse_controls_enabled: self.controls_enabled(),
            // `cnGuideMode.Update` blocks Escape for an in-flight request or
            // system popup, but not merely because `GameFrame.bHelp` disabled
            // IMGUI controls. The shell must still supply event 2/24's gate.
            escape_dismiss_enabled: self.visible
                && !self.awaiting_server
                && !self.system_popup_open,
        }
    }

    #[must_use]
    pub const fn primary_blocker(&self) -> Option<GuideUiBlocker> {
        if !self.visible {
            return Some(GuideUiBlocker::Hidden);
        }
        if self.awaiting_server {
            return Some(GuideUiBlocker::AwaitingServer);
        }
        if self.system_popup_open || self.help_open {
            return Some(GuideUiBlocker::ControlsDisabled);
        }
        if self.confirmation_open {
            return Some(GuideUiBlocker::ConfirmationOpen);
        }
        if self.selected.is_none() {
            return Some(GuideUiBlocker::NoMentorSelected);
        }
        None
    }

    #[must_use]
    pub const fn pending_mentor(&self) -> Option<GuideMentor> {
        self.pending_mentor
    }
}
