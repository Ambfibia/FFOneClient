use super::*;

pub(super) fn stage_enchant_write(
    staged: &mut Vec<EnchantInventoryWrite0104>,
    field: &'static str,
    raw_slot: i32,
    item: ItemBase0104,
    allow_absent: bool,
) -> Result<(), EnchantProductionError0104> {
    if raw_slot == -1 && allow_absent {
        return Ok(());
    }
    let inventory_index = usize::try_from(raw_slot).map_err(|_| {
        EnchantProductionError0104::ReplySlotOutOfBounds {
            field,
            value: raw_slot,
        }
    })?;
    if inventory_index >= INVENTORY_SLOT_COUNT_0104 {
        return Err(EnchantProductionError0104::ReplySlotOutOfBounds {
            field,
            value: raw_slot,
        });
    }
    if item.item_id > 0 && item.item_type < 0 {
        return Err(EnchantProductionError0104::MalformedReplyItem {
            inventory_index,
            item_type: item.item_type,
            item_id: item.item_id,
        });
    }
    if let Some(first) = staged
        .iter()
        .find(|write| write.inventory_index == inventory_index)
    {
        if first.item != item {
            return Err(EnchantProductionError0104::ConflictingReplyWrites {
                inventory_index,
                first: first.item,
                second: item,
            });
        }
        return Ok(());
    }
    staged.push(EnchantInventoryWrite0104 {
        inventory_index,
        item,
    });
    Ok(())
}
