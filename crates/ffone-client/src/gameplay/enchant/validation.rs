use super::*;

pub(super) fn validate_open_context(
    context: EnchantOpenContext0104,
    inventory: &InventoryRuntime0104,
) -> Result<(), EnchantProductionError0104> {
    if context.source.npc_type != ENCHANT_NPC_TYPE_0104 {
        return Err(EnchantProductionError0104::WrongNpcType {
            expected: ENCHANT_NPC_TYPE_0104,
            actual: context.source.npc_type,
        });
    }
    if context.npc_button_type != ENCHANT_NPC_BUTTON_TYPE_0104 {
        return Err(EnchantProductionError0104::WrongNpcButtonType {
            expected: ENCHANT_NPC_BUTTON_TYPE_0104,
            actual: context.npc_button_type,
        });
    }
    if context.game_mode != ENCHANT_GAME_MODE_0104 {
        return Err(EnchantProductionError0104::WrongGameMode {
            expected: ENCHANT_GAME_MODE_0104,
            actual: context.game_mode,
        });
    }
    validate_player_authority(context.player.owner_pc_id, inventory, context.player)
}

pub(super) fn validate_player_authority(
    expected_pc_id: i32,
    inventory: &InventoryRuntime0104,
    player: EnchantPlayerAuthority0104,
) -> Result<(), EnchantProductionError0104> {
    if inventory.owner_pc_id() != expected_pc_id {
        return Err(EnchantProductionError0104::InventoryOwnerMismatch {
            expected_pc_id,
            inventory_pc_id: inventory.owner_pc_id(),
        });
    }
    if player.owner_pc_id != expected_pc_id {
        return Err(EnchantProductionError0104::PlayerOwnerMismatch {
            expected_pc_id,
            player_pc_id: player.owner_pc_id,
        });
    }
    for (field, value) in [
        ("taros", player.taros),
        ("weapon_battery", player.weapon_battery),
        ("nano_battery", player.nano_battery),
    ] {
        if value < 0 {
            return Err(EnchantProductionError0104::NegativeAuthority { field, value });
        }
    }
    Ok(())
}

pub(super) fn validate_request_against_reservations(
    session: &EnchantProductionSession0104,
    request: EnchantRequest0104,
) -> Result<(), EnchantProductionError0104> {
    if request.enchant_item_slot < 0 {
        return Err(EnchantProductionError0104::InvalidRequestSlots {
            detail: "target slot is negative",
        });
    }
    if request.weapon_material_item_slot == -1 && request.defence_material_item_slot == -1 {
        return Err(EnchantProductionError0104::InvalidRequestSlots {
            detail: "both material slots are absent",
        });
    }
    let pairs = [
        (EnchantAttachmentSlot0104::Target, request.enchant_item_slot),
        (
            EnchantAttachmentSlot0104::WeaponMaterial,
            request.weapon_material_item_slot,
        ),
        (
            EnchantAttachmentSlot0104::ArmorMaterial,
            request.defence_material_item_slot,
        ),
        (EnchantAttachmentSlot0104::Helper1, request.cash_item_slot_1),
        (EnchantAttachmentSlot0104::Helper2, request.cash_item_slot_2),
    ];
    let mut seen = Vec::new();
    for (slot, source_slot) in pairs {
        if source_slot == -1 {
            if session.reservations_by_attachment[slot.index()].is_some() {
                return Err(EnchantProductionError0104::InvalidRequestSlots {
                    detail: "absent request slot still has a reservation",
                });
            }
            continue;
        }
        let source = usize::try_from(source_slot).map_err(|_| {
            EnchantProductionError0104::ReservationSourceOutOfBounds { source_slot }
        })?;
        if source >= INVENTORY_SLOT_COUNT_0104 {
            return Err(EnchantProductionError0104::ReservationSourceOutOfBounds { source_slot });
        }
        if seen.contains(&source_slot) {
            return Err(EnchantProductionError0104::InvalidRequestSlots {
                detail: "one inventory slot is attached more than once",
            });
        }
        seen.push(source_slot);
        let reservation = session.reservations_by_attachment[slot.index()]
            .ok_or(EnchantProductionError0104::ReservationMissing { slot })?;
        if reservation.source_slot != source_slot {
            return Err(EnchantProductionError0104::ReservationSourceMismatch {
                slot,
                expected: reservation.source_slot,
                actual: source_slot,
            });
        }
    }
    Ok(())
}

pub(super) fn validate_enchant_success_identity(
    request: EnchantRequest0104,
    reply: EnchantSuccess0104,
) -> Result<(), EnchantProductionError0104> {
    for (field, expected, actual) in [
        (
            "iEnchantItemSlot",
            request.enchant_item_slot,
            reply.enchant_item_slot,
        ),
        (
            "iWeaponMaterialItemSlot",
            request.weapon_material_item_slot,
            reply.weapon_material_item_slot,
        ),
        (
            "iDefenceMaterialItemSlot",
            request.defence_material_item_slot,
            reply.defence_material_item_slot,
        ),
        (
            "iCashItemSlot1",
            request.cash_item_slot_1,
            reply.cash_item_slot_1,
        ),
        (
            "iCashItemSlot2",
            request.cash_item_slot_2,
            reply.cash_item_slot_2,
        ),
    ] {
        if actual != expected {
            return Err(EnchantProductionError0104::ReplyIdentityMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

pub(super) fn validate_enchant_failure_identity(
    request: EnchantRequest0104,
    reply: EnchantFailure0104,
) -> Result<(), EnchantProductionError0104> {
    for (field, expected, actual) in [
        (
            "iEnchantItemSlot",
            request.enchant_item_slot,
            reply.enchant_item_slot,
        ),
        (
            "iWeaponMaterialItemSlot",
            request.weapon_material_item_slot,
            reply.weapon_material_item_slot,
        ),
        (
            "iDefenceMaterialItemSlot",
            request.defence_material_item_slot,
            reply.defence_material_item_slot,
        ),
        (
            "iCashItemSlot1",
            request.cash_item_slot_1,
            reply.cash_item_slot_1,
        ),
        (
            "iCashItemSlot2",
            request.cash_item_slot_2,
            reply.cash_item_slot_2,
        ),
    ] {
        if actual != expected {
            return Err(EnchantProductionError0104::ReplyIdentityMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}
