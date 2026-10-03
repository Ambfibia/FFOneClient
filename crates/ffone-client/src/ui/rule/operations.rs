use super::*;

pub(super) fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn exit_rule_ui(model: &mut RuleUiModel, outbox: &mut RuleUiOutbox, source: RuleUiDismissalSource) {
    model.close();
    outbox.push(RuleUiAction::ExitMode {
        source,
        event_group: 2,
        event_function: 1,
        cursor_locked: false,
    });
}
