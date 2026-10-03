use super::*;

#[must_use]
pub fn resolve_rule_ui_escape_close_gate(
    model: &mut RuleUiModel,
    outbox: &mut RuleUiOutbox,
    accepted: bool,
) -> bool {
    if !model.visible || !model.escape_close_pending {
        return false;
    }
    model.escape_close_pending = false;
    if accepted {
        exit_rule_ui(model, outbox, RuleUiDismissalSource::EscapeCloseGate);
    }
    true
}
