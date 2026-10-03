use super::*;

pub(super) fn validate_open_context(
    context: CombiOpenContext0104,
    inventory: &InventoryRuntime0104,
) -> Result<(), CombiProductionError0104> {
    if context.source.npc_type != COMBI_NPC_TYPE_0104 {
        return Err(CombiProductionError0104::WrongNpcType {
            expected: COMBI_NPC_TYPE_0104,
            actual: context.source.npc_type,
        });
    }
    if context.npc_button_type != COMBI_NPC_BUTTON_TYPE_0104 {
        return Err(CombiProductionError0104::WrongNpcButtonType {
            expected: COMBI_NPC_BUTTON_TYPE_0104,
            actual: context.npc_button_type,
        });
    }
    if context.game_mode != COMBI_GAME_MODE_0104 {
        return Err(CombiProductionError0104::WrongGameMode {
            expected: COMBI_GAME_MODE_0104,
            actual: context.game_mode,
        });
    }
    validate_player_authority(context.player.owner_pc_id, inventory, context.player)
}

pub(super) fn validate_player_authority(
    expected_pc_id: i32,
    inventory: &InventoryRuntime0104,
    player: CombiPlayerAuthority0104,
) -> Result<(), CombiProductionError0104> {
    if player.owner_pc_id != expected_pc_id {
        return Err(CombiProductionError0104::PlayerOwnerMismatch {
            expected_pc_id,
            player_pc_id: player.owner_pc_id,
        });
    }
    if inventory.owner_pc_id() != expected_pc_id {
        return Err(CombiProductionError0104::InventoryOwnerMismatch {
            expected_pc_id,
            inventory_pc_id: inventory.owner_pc_id(),
        });
    }
    if player.taros < 0 {
        return Err(CombiProductionError0104::NegativeTaros {
            taros: player.taros,
        });
    }
    Ok(())
}

pub(super) fn validate_selected_items_unchanged(
    session: &CombiProductionSession0104,
    candidate: &CombiAuthoritativeSnapshot0104,
) -> Result<(), CombiProductionError0104> {
    for slot in [CombiSelectionSlot0104::Style, CombiSelectionSlot0104::Stats] {
        let Some(inventory_index) = session.machine.selection.selected(slot) else {
            continue;
        };
        let expected = session.snapshot.inventory[inventory_index];
        let actual = candidate.inventory[inventory_index];
        if expected != actual {
            return Err(CombiProductionError0104::SelectedItemChanged {
                slot,
                inventory_index,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

pub(super) fn require_ready(machine: &CombiMachine0104) -> Result<(), CombiProductionError0104> {
    if machine.phase == CombiPhase0104::Ready {
        Ok(())
    } else {
        Err(CombiProductionError0104::State(
            CombiStateError0104::InputBlocked {
                phase: machine.phase.clone(),
            },
        ))
    }
}
