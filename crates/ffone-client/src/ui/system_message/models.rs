use super::*;

#[derive(Debug, Resource)]
pub struct SystemMessageUiModel {
    pub(super) stack: Vec<SystemMessageRequest>,
    pub(super) focus_out: bool,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for SystemMessageUiModel {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            focus_out: false,
            ui_scale_override: None,
        }
    }
}

impl SystemMessageUiModel {
    pub fn push(&mut self, request: SystemMessageRequest) {
        self.stack.push(request);
    }

    pub fn try_push_legacy(
        &mut self,
        request_id: u64,
        text: impl Into<String>,
        raw_button_type: i32,
    ) -> Result<(), SystemMessageButtonTypeError> {
        self.push(SystemMessageRequest::try_from_legacy(
            request_id,
            text,
            raw_button_type,
        )?);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Removes one correlated owner without disturbing newer or unrelated
    /// modal requests in the shared LIFO stack.
    pub fn remove(&mut self, request_id: u64) -> bool {
        let Some(index) = self
            .stack
            .iter()
            .rposition(|request| request.request_id == request_id)
        else {
            return false;
        };
        self.stack.remove(index);
        true
    }

    #[must_use]
    pub fn current(&self) -> Option<&SystemMessageRequest> {
        self.stack.last()
    }

    #[must_use]
    pub fn stack(&self) -> &[SystemMessageRequest] {
        &self.stack
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Mirrors `IsSysPopUp`: focus-out also blocks gameplay even without a
    /// visible message.
    #[must_use]
    pub fn is_popup(&self) -> bool {
        !self.stack.is_empty() || self.focus_out
    }

    #[must_use]
    pub const fn focus_out(&self) -> bool {
        self.focus_out
    }

    pub fn set_focus_out(&mut self, focus_out: bool) {
        self.focus_out = focus_out;
    }

    #[must_use]
    pub const fn ui_scale_override(&self) -> Option<f32> {
        self.ui_scale_override
    }

    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale_override = Some(valid_ui_scale(scale));
    }

    pub fn clear_ui_scale_override(&mut self) {
        self.ui_scale_override = None;
    }

    #[must_use]
    pub fn effective_ui_scale(&self, viewport_height: f32) -> f32 {
        self.ui_scale_override
            .unwrap_or_else(|| clean_system_message_ui_scale(viewport_height))
    }

    /// Selects a button and removes only the newest message. A secondary
    /// choice on a one-button message fails closed and leaves the stack intact.
    pub fn choose(&mut self, choice: SystemMessageChoice) -> Option<SystemMessageUiAction> {
        if self.focus_out {
            return None;
        }
        let current = self.current()?;
        if !system_message_button_layout(current.button_type).contains(choice) {
            return None;
        }
        let request_id = current.request_id;
        let button_type = current.button_type;
        self.stack.pop();
        Some(SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        })
    }
}
