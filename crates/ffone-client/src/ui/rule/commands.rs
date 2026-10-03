use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleUiAction {
    RequestEscapeCloseGate {
        event_group: u8,
        event_function: u8,
    },
    ExitMode {
        source: RuleUiDismissalSource,
        event_group: u8,
        event_function: u8,
        cursor_locked: bool,
    },
}

#[must_use]
pub fn request_rule_ui_escape_close(model: &mut RuleUiModel, outbox: &mut RuleUiOutbox) -> bool {
    if !model.input_boundary().escape_close_gate_enabled {
        return false;
    }
    model.escape_close_pending = true;
    outbox.push(RuleUiAction::RequestEscapeCloseGate {
        event_group: 2,
        event_function: 24,
    });
    true
}
