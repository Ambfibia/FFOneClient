use super::*;

#[must_use]
pub fn project_blocked_players(slots: &[OptionBuddySlot]) -> Vec<BlockedPlayerRow> {
    slots
        .iter()
        .take(OPTION_BLOCKED_SLOT_CAPACITY)
        .enumerate()
        .filter(|(_, buddy)| buddy.pc_uid != 0 && buddy.blocked)
        .map(|(slot, buddy)| BlockedPlayerRow {
            slot,
            pc_uid: buddy.pc_uid,
            display_name: if buddy.name_check_flag == 1 {
                format!("{} {}", buddy.first_name, buddy.last_name)
                    .trim()
                    .to_owned()
            } else {
                format!("Player {}", buddy.pc_uid)
            },
        })
        .collect()
}
