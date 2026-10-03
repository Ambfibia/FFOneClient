use super::*;

pub(super) fn snapshot_from_authority(
    inventory: &InventoryRuntime0104,
    player: CombiPlayerAuthority0104,
) -> Result<CombiAuthoritativeSnapshot0104, CombiProductionError0104> {
    validate_player_authority(player.owner_pc_id, inventory, player)?;
    Ok(CombiAuthoritativeSnapshot0104::from_inventory_runtime(
        inventory,
        player.gender,
        player.level,
        player.guide,
        player.taros,
    ))
}

pub(super) fn modal_message(
    session: &CombiProductionSession0104,
    modal: CombiSystemModal0104,
) -> CombiSystemMessage0104 {
    let (style_item, stats_item) = if modal == CombiSystemModal0104::CombinationFailed {
        (
            session
                .machine
                .selection
                .style_inventory_index()
                .map(|index| session.snapshot.inventory[index]),
            session
                .machine
                .selection
                .stats_inventory_index()
                .map(|index| session.snapshot.inventory[index]),
        )
    } else {
        (None, None)
    };
    CombiSystemMessage0104::Modal {
        modal,
        style_item,
        stats_item,
    }
}

pub(super) fn authoritative_commit(
    snapshot: &CombiAuthoritativeSnapshot0104,
    receipt: CombiAuthorityReceipt0104,
) -> Result<CombiAuthoritativeCommit0104, CombiReceiptCommitError0104> {
    let inventory_writes = match &receipt {
        CombiAuthorityReceipt0104::Success {
            style_slot,
            stats_slot,
            style_after,
            stats_after,
            ..
        } => [
            Some(CombiInventoryWrite0104 {
                inventory_index: *style_slot,
                item: *style_after,
            }),
            Some(CombiInventoryWrite0104 {
                inventory_index: *stats_slot,
                item: *stats_after,
            }),
        ],
        CombiAuthorityReceipt0104::Failure { .. } => [None, None],
    };
    let taros_after = match &receipt {
        CombiAuthorityReceipt0104::Success { taros_after, .. }
        | CombiAuthorityReceipt0104::Failure { taros_after, .. } => *taros_after,
    };
    let mut snapshot_after = snapshot.clone();
    snapshot_after.apply_authoritative_receipt(&receipt)?;
    Ok(CombiAuthoritativeCommit0104 {
        receipt,
        inventory_writes,
        taros_after,
        snapshot_after,
    })
}
