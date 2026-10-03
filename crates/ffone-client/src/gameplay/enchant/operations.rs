use super::*;

/// Registration/layout proof used before the clean confirmation callback is
/// allowed to enter its four-second waiting animation.
pub fn prove_enchant_transport_registered_0104() -> Result<(), EnchantProductionError0104> {
    RegisteredGameplayRequest0104::new(
        ENCHANT_REQUEST_PACKET_ID_0104,
        vec![0; ENCHANT_REQUEST_PACKET_SIZE_0104],
    )?;
    Ok(())
}

pub(super) fn snapshot_from_authority(
    inventory: &InventoryRuntime0104,
    player: EnchantPlayerAuthority0104,
) -> Result<EnchantAuthoritySnapshot0104, EnchantProductionError0104> {
    validate_player_authority(player.owner_pc_id, inventory, player)?;
    Ok(EnchantAuthoritySnapshot0104 {
        owner_pc_id: player.owner_pc_id,
        inventory: *inventory.inventory(),
        equipment: *inventory.equipment(),
        taros: player.taros,
        weapon_battery: player.weapon_battery,
        nano_battery: player.nano_battery,
    })
}

pub(super) fn open_delete_modal(
    session: &mut EnchantProductionSession0104,
    output: &mut EnchantProductionOutput0104,
) -> Result<(), EnchantProductionError0104> {
    if !session.model.input_capabilities().base_gui_enabled {
        return Err(EnchantModelError0104::InputBlocked.into());
    }
    if session.runtime_modal.is_some() {
        return Err(EnchantProductionError0104::RuntimeModalAlreadyOpen);
    }
    let drag = session
        .drag
        .take()
        .ok_or(EnchantProductionError0104::MissingDragSource)?;
    if session.reserved_by_source[drag.inventory_index] > 0 {
        return Err(EnchantProductionError0104::DragSourceAlreadyReserved {
            inventory_index: drag.inventory_index,
        });
    }
    let actual = session.snapshot.inventory[drag.inventory_index];
    if actual != drag.authority_item {
        return Err(EnchantProductionError0104::StaleDragSource {
            inventory_index: drag.inventory_index,
            expected: drag.authority_item,
            actual,
        });
    }
    let modal = EnchantRuntimeModal0104::DeleteItem {
        message_id: ENCHANT_MESSAGE_DELETE_ITEM_0104,
        inventory_index: drag.inventory_index,
        item: drag.authority_item,
        quantity: if drag.authority_item.item_type == 7 {
            drag.authority_item.option
        } else {
            1
        },
    };
    session.runtime_modal = Some(modal.clone());
    let mut gates = session.model.external_gates();
    gates.system_popup_open = true;
    session.model.set_external_gates(gates);
    output
        .effects
        .push(EnchantShellEffect0104::RuntimeModal(modal));
    Ok(())
}

pub(super) fn expected_enchant_receipt(reply: EnchantSuccess0104) -> EnchantAuthoritativeReceipt0104 {
    let mutations = if reply.success_flag == 1 {
        let mut mutations = vec![crate::enchant_ui::EnchantInventoryMutation0104 {
            slot: reply.enchant_item_slot,
            item: reply.enchant_item,
        }];
        if reply.weapon_material_item_slot != -1 {
            mutations.push(crate::enchant_ui::EnchantInventoryMutation0104 {
                slot: reply.weapon_material_item_slot,
                item: reply.weapon_material_item,
            });
        }
        if reply.defence_material_item_slot != -1 {
            mutations.push(crate::enchant_ui::EnchantInventoryMutation0104 {
                slot: reply.defence_material_item_slot,
                item: reply.defence_material_item,
            });
        }
        mutations
    } else {
        vec![
            crate::enchant_ui::EnchantInventoryMutation0104 {
                slot: reply.enchant_item_slot,
                item: reply.enchant_item,
            },
            crate::enchant_ui::EnchantInventoryMutation0104 {
                slot: reply.weapon_material_item_slot,
                item: reply.weapon_material_item,
            },
            crate::enchant_ui::EnchantInventoryMutation0104 {
                slot: reply.defence_material_item_slot,
                item: reply.defence_material_item,
            },
        ]
    };
    EnchantAuthoritativeReceipt0104 {
        taros: reply.taros,
        mutations,
        success_flag: reply.success_flag,
    }
}

pub(super) fn one_authoritative_receipt(
    mut receipts: Vec<EnchantAuthoritativeReceipt0104>,
) -> Result<EnchantAuthoritativeReceipt0104, EnchantProductionError0104> {
    if receipts.len() != 1 {
        return Err(if receipts.is_empty() {
            EnchantProductionError0104::MissingAuthoritativeReceipt
        } else {
            EnchantProductionError0104::UnexpectedAuthoritativeReceipt
        });
    }
    Ok(receipts.pop().expect("length is one"))
}
