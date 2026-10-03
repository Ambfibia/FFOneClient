use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct QuitMenuUiModel {
    pub visible: bool,
    pub enabled: bool,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for QuitMenuUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            enabled: true,
            ui_scale_override: None,
        }
    }
}

impl QuitMenuUiModel {
    pub fn open(&mut self) {
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
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
            .unwrap_or_else(|| clean_quit_menu_ui_scale(viewport_height))
    }

    #[must_use]
    pub const fn input_boundary(&self) -> QuitMenuInputBoundary {
        QuitMenuInputBoundary {
            blocks_lower_ui: self.visible,
            blocks_gameplay_input: self.visible,
            requires_pointer: self.visible,
            mouse_controls_enabled: self.visible && self.enabled,
            // `cnQuit.Update` does not condition Escape on `GUI.enabled`.
            escape_dismiss_enabled: self.visible,
        }
    }
}
