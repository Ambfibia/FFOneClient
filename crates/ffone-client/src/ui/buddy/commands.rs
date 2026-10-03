use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuddyUiAction {
    RefreshStatesRequested,
    AddByNameRequested {
        first_name: String,
        last_name: String,
    },
    AddNameRejected {
        message: String,
    },
    ConfirmationRequested(BuddyConfirmation),
    RemoveRequested(BuddyTarget),
    WarpCooldownNotice {
        remaining_seconds: u32,
    },
    WarpRequested {
        target: BuddyTarget,
        leave_group: bool,
    },
    InviteResponse {
        invite_id: u64,
        requester_pc_id: i32,
        requester_pc_uid: i64,
        accepted: bool,
    },
}
