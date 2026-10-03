use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct RuleUiModel {
    pub visible: bool,
    pub current_page: Option<RulePageId>,
    pub system_popup_active: bool,
    pub help_active: bool,
    pub escape_close_pending: bool,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for RuleUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            current_page: None,
            system_popup_active: false,
            help_active: false,
            escape_close_pending: false,
            ui_scale_override: None,
        }
    }
}

impl RuleUiModel {
    pub fn open(&mut self, page: RulePageId) {
        self.visible = true;
        self.current_page = Some(page);
        self.escape_close_pending = false;
    }

    pub fn open_table_index(&mut self, table_index: i32) -> Result<(), RulePageIndexError> {
        let page = RulePageId::try_from(table_index)?;
        self.open(page);
        Ok(())
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.escape_close_pending = false;
    }

    pub fn set_system_popup_active(&mut self, active: bool) {
        self.system_popup_active = active;
    }

    pub fn set_help_active(&mut self, active: bool) {
        self.help_active = active;
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
            .unwrap_or_else(|| clean_rule_ui_scale(viewport_height))
    }

    #[must_use]
    pub const fn controls_enabled(&self) -> bool {
        self.visible && !self.system_popup_active && !self.help_active
    }

    #[must_use]
    pub const fn input_boundary(&self) -> RuleUiInputBoundary {
        RuleUiInputBoundary {
            blocks_lower_ui: self.visible,
            blocks_gameplay_input: self.visible,
            requires_pointer: self.visible,
            // `cnRule.Update` writes `Screen.lockCursor = false` every frame.
            cursor_locked_while_visible: false,
            // `bOldLockCursor` is never assigned in the clean component and
            // therefore remains its serialized/default `false` value.
            cursor_locked_after_exit: false,
            mouse_controls_enabled: self.controls_enabled(),
            // Help only affects `GUI.enabled`; Update checks only SysPopUp.
            escape_close_gate_enabled: self.visible
                && !self.system_popup_active
                && !self.escape_close_pending,
        }
    }

    #[must_use]
    pub fn page(&self) -> Option<&'static RulePageSpec> {
        self.current_page.map(RulePageId::spec)
    }
}
