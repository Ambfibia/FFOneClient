use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) enum PendingBuddySystemAction {
    Dismiss,
    Confirmation(BuddyConfirmation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) enum PendingBuddyNanocomAction {
    Invite {
        invite_id: u64,
    },
    NameInvite {
        pc_uid: i64,
        first_name: String,
        last_name: String,
    },
}
