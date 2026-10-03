use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantReplyPacket0104 {
    Success(EnchantSuccess0104),
    Failure(EnchantFailure0104),
    DeleteSuccess(PcItemDeleteSuccess0104),
    DisassembleSuccess(PcDisassembleItemSuccess0104),
    DisassembleFailure(PcDisassembleItemFailure0104),
}

impl EnchantReplyPacket0104 {
    #[must_use]
    pub const fn operation(&self) -> EnchantOperation0104 {
        match self {
            Self::Success(_) | Self::Failure(_) => EnchantOperation0104::Enchant,
            Self::DeleteSuccess(_) => EnchantOperation0104::Delete,
            Self::DisassembleSuccess(_) | Self::DisassembleFailure(_) => {
                EnchantOperation0104::Disassemble
            }
        }
    }
}

pub(super) fn ensure_no_pending_request(
    session: &EnchantProductionSession0104,
) -> Result<(), EnchantProductionError0104> {
    if let Some(pending) = &session.pending_request {
        Err(EnchantProductionError0104::RequestAlreadyPending {
            operation: pending.operation(),
        })
    } else {
        Ok(())
    }
}

pub(super) fn set_single_request(
    output: &mut EnchantProductionOutput0104,
    request: RegisteredGameplayRequest0104,
) -> Result<(), EnchantProductionError0104> {
    if output.request.is_some() {
        return Err(EnchantProductionError0104::MultipleOutboundRequests);
    }
    output.request = Some(request);
    Ok(())
}

pub(super) fn commit_enchant_reply(
    snapshot: &EnchantAuthoritySnapshot0104,
    reply: EnchantSuccess0104,
    receipt: &EnchantAuthoritativeReceipt0104,
) -> Result<EnchantAuthoritativeCommit0104, EnchantProductionError0104> {
    if reply.success_flag != 0 && reply.success_flag != 1 {
        return Err(EnchantProductionError0104::AuthoritativeReceiptMismatch);
    }
    if *receipt != expected_enchant_receipt(reply) {
        return Err(EnchantProductionError0104::AuthoritativeReceiptMismatch);
    }
    if reply.taros < 0 {
        return Err(EnchantProductionError0104::NegativeAuthority {
            field: "reply.taros",
            value: reply.taros,
        });
    }
    let mut staged = Vec::with_capacity(3);
    stage_enchant_write(
        &mut staged,
        "iEnchantItemSlot",
        reply.enchant_item_slot,
        reply.enchant_item,
        false,
    )?;
    stage_enchant_write(
        &mut staged,
        "iWeaponMaterialItemSlot",
        reply.weapon_material_item_slot,
        reply.weapon_material_item,
        true,
    )?;
    stage_enchant_write(
        &mut staged,
        "iDefenceMaterialItemSlot",
        reply.defence_material_item_slot,
        reply.defence_material_item,
        true,
    )?;
    let mut next = snapshot.clone();
    for write in staged.iter().copied() {
        next.inventory[write.inventory_index] = write.item;
    }
    next.taros = reply.taros;
    Ok(EnchantAuthoritativeCommit0104 {
        operation: EnchantOperation0104::Enchant,
        success_flag: Some(reply.success_flag),
        inventory_writes: staged,
        snapshot_after: next,
    })
}

pub(super) fn commit_delete_reply(
    snapshot: &EnchantAuthoritySnapshot0104,
    pending: &PendingDelete0104,
    reply: PcItemDeleteSuccess0104,
) -> Result<EnchantAuthoritativeCommit0104, EnchantProductionError0104> {
    if reply.item_location != 1 {
        return Err(EnchantProductionError0104::ReplyIdentityMismatch {
            field: "eIL",
            expected: 1,
            actual: reply.item_location,
        });
    }
    if reply.slot_num != pending.inventory_index as i32 {
        return Err(EnchantProductionError0104::ReplyIdentityMismatch {
            field: "iSlotNum",
            expected: pending.inventory_index as i32,
            actual: reply.slot_num,
        });
    }
    let actual = snapshot.inventory[pending.inventory_index];
    if actual != pending.item_before {
        return Err(EnchantProductionError0104::SelectedItemChanged {
            inventory_index: pending.inventory_index,
            expected: pending.item_before,
            actual,
        });
    }
    // Exact OpenFusion delete post-state: identity/option clear while the
    // serialized time-limit word is retained.
    let item = ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: actual.time_limit,
    };
    let write = EnchantInventoryWrite0104 {
        inventory_index: pending.inventory_index,
        item,
    };
    let mut next = snapshot.clone();
    next.inventory[pending.inventory_index] = item;
    Ok(EnchantAuthoritativeCommit0104 {
        operation: EnchantOperation0104::Delete,
        success_flag: None,
        inventory_writes: vec![write],
        snapshot_after: next,
    })
}
