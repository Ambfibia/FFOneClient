use super::*;

/// Projects the production buddy-slot authority in exact slot order. Blocked
/// and empty slots are not reachable from clean `Panel_NewEmail`.
pub fn email_buddies_from_buddy_ui_0104(
    model: &BuddyUiModel,
) -> Result<Vec<EmailBuddy>, EmailProductionError0104> {
    let buddies = (0..BUDDY_MAX_SLOTS)
        .filter_map(|slot| model.slot(slot))
        .filter(|entry| entry.pc_uid != 0 && !entry.blocked)
        .map(|entry| EmailBuddy {
            pc_uid: entry.pc_uid,
            first_name: entry.first_name.clone(),
            last_name: entry.last_name.clone(),
            name_check_flag: i32::from(entry.name_check_flag),
        })
        .collect::<Vec<_>>();
    project_email_buddies_0104(&buddies)
}

#[must_use]
pub const fn email_item_from_base(item: ItemBase0104) -> EmailWireItem {
    EmailWireItem {
        item_type: item.item_type,
        item_id: item.item_id,
        option: item.option,
        time_limit: item.time_limit,
    }
}

#[must_use]
pub const fn item_base_from_email(item: EmailWireItem) -> ItemBase0104 {
    ItemBase0104 {
        item_type: item.item_type,
        item_id: item.item_id,
        option: item.option,
        time_limit: item.time_limit,
    }
}
