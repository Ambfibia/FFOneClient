use super::*;

pub const BUDDY_ADD_NAME_ERROR: &str = "You must enter a first name and a last name.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuddyUiError {
    SlotOutOfRange(usize),
    EmptySlot(usize),
    TargetMismatch {
        slot: usize,
        expected_pc_uid: i64,
        actual_pc_uid: i64,
    },
    NoSelection,
    SelectedBuddyOffline,
    PlayerMoving,
    StaleConfirmation(BuddyConfirmation),
    InviteNotCurrent(u64),
}

impl fmt::Display for BuddyUiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SlotOutOfRange(slot) => write!(formatter, "buddy slot {slot} is outside 0..50"),
            Self::EmptySlot(slot) => write!(formatter, "buddy slot {slot} is empty"),
            Self::TargetMismatch {
                slot,
                expected_pc_uid,
                actual_pc_uid,
            } => write!(
                formatter,
                "buddy slot {slot} changed from PCUID {expected_pc_uid} to {actual_pc_uid}"
            ),
            Self::NoSelection => write!(formatter, "no buddy is selected"),
            Self::SelectedBuddyOffline => write!(formatter, "selected buddy is offline"),
            Self::PlayerMoving => write!(formatter, "buddy warp is unavailable while moving"),
            Self::StaleConfirmation(confirmation) => {
                write!(formatter, "stale buddy confirmation: {confirmation:?}")
            }
            Self::InviteNotCurrent(invite_id) => {
                write!(
                    formatter,
                    "buddy invite {invite_id} is not the current invite"
                )
            }
        }
    }
}

impl Error for BuddyUiError {}
