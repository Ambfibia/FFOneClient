use super::*;

pub fn project_email_buddies_0104(
    buddies: &[EmailBuddy],
) -> Result<Vec<EmailBuddy>, EmailProductionError0104> {
    if buddies.len() > EMAIL_BUDDY_MAX_COUNT_0104 {
        return Err(EmailProductionError0104::TooManyBuddies {
            actual: buddies.len(),
            maximum: EMAIL_BUDDY_MAX_COUNT_0104,
        });
    }
    for (index, buddy) in buddies.iter().enumerate() {
        if buddy.pc_uid <= 0 {
            return Err(EmailProductionError0104::InvalidBuddy {
                index,
                pc_uid: buddy.pc_uid,
            });
        }
        if let Some((first_index, _)) = buddies[..index]
            .iter()
            .enumerate()
            .find(|(_, known)| known.pc_uid == buddy.pc_uid)
        {
            return Err(EmailProductionError0104::DuplicateBuddy {
                first_index,
                second_index: index,
                pc_uid: buddy.pc_uid,
            });
        }
    }
    Ok(buddies.to_vec())
}

pub fn project_email_inventory_0104(
    inventory: &EmailInventoryAuthority0104,
    catalog: &impl EmailItemCatalog0104,
) -> Result<Vec<Option<EmailInventorySlotView>>, EmailProductionError0104> {
    inventory
        .slots
        .iter()
        .copied()
        .enumerate()
        .map(|(inventory_slot, item)| {
            if InventoryRuntime0104::item_is_empty(item) {
                return Ok(None);
            }
            if item.item_type < 0 {
                return Err(EmailProductionError0104::MalformedInventoryItem {
                    inventory_slot,
                    item,
                });
            }
            // Rendering a slot must not gate access to the mailbox. Keep the
            // authoritative item even when its artwork/catalog entry is absent.
            // Attachment validation below still requires valid trade metadata.
            let icon_path = catalog.resolve(item)
                .and_then(|metadata| metadata.icon_path)
                .filter(|path| is_safe_email_icon_path(path));
            Ok(Some(EmailInventorySlotView {
                item: email_item_from_base(item),
                icon_path,
                count_label: (item.item_type == 7).then(|| item.option.to_string()),
            }))
        })
        .collect()
}
