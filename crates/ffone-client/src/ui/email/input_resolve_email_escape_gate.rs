use super::*;

pub const EMAIL_REQ_READ_ID: u32 = 0x1300_007C;

pub const EMAIL_REP_READ_SUCCESS_ID: u32 = 0x3100_00E1;

pub const EMAIL_REP_READ_FAILURE_ID: u32 = 0x3100_00E2;

pub const EMAIL_REQ_READ_SIZE: usize = 8;

pub const EMAIL_REP_READ_SUCCESS_SIZE: usize = 1_084;

pub const EMAIL_REP_READ_FAILURE_SIZE: usize = 12;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailReadMessage {
    pub email_index: i64,
    pub content: String,
    pub items: [EmailWireItem; EMAIL_ATTACHMENT_COUNT],
    pub cash: i32,
}

pub fn resolve_email_escape_gate(
    accepted: bool,
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
) -> bool {
    if !model.visible || !model.escape_gate_pending {
        return false;
    }
    model.escape_gate_pending = false;
    if accepted {
        model.computress_query_pending = true;
        actions.push(EmailUiAction::QueryComputressExitGate {
            event_group: 11,
            event_function: 13,
        });
    }
    true
}

pub fn resolve_email_computress_gate(
    computress_active: bool,
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
) -> bool {
    if !model.visible || !model.computress_query_pending {
        return false;
    }
    model.computress_query_pending = false;
    if !computress_active {
        exit_email_ui(model, actions);
    }
    true
}
