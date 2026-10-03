use super::*;

pub(in super::super) struct RuntimeCombiItemCatalog0104<'a> {
    pub(in super::super) source: &'a CombiProductionCatalog0104,
    pub(in super::super) player: CombiPlayerAuthority0104,
}

impl RuntimeCombiItemCatalog0104<'_> {
    pub(in super::super) fn base_and_appearance(
        &self,
        item: ItemBase0104,
    ) -> Option<(CombiItemMetadata0104, CombiItemMetadata0104)> {
        let base = self.source.resolve(item.item_type, item.item_id)?;
        let appearance = combined_appearance_item_id(item).map_or_else(
            || Some(base.clone()),
            |item_id| self.source.resolve(item.item_type, item_id),
        )?;
        Some((base, appearance))
    }
}

impl CombiItemCatalog0104 for RuntimeCombiItemCatalog0104<'_> {
    fn resolve_icon(&self, item_type: i16, item_id: i16) -> Option<String> {
        self.source.resolve_icon(item_type, item_id)
    }
    fn resolve(&self, item_type: i16, item_id: i16) -> Option<CombiItemMetadata0104> {
        self.source.resolve(item_type, item_id)
    }

    fn enable_equip(&self, item: ItemBase0104) -> Option<bool> {
        let (base, appearance) = self.base_and_appearance(item)?;
        Some(
            self.player.level >= base.minimum_level
                && (appearance.required_gender <= 0
                    || appearance.required_gender == self.player.gender)
                && (appearance.mentor <= 0 || appearance.mentor == self.player.guide),
        )
    }

    fn enable_equip_combi(&self, item: ItemBase0104) -> Option<bool> {
        let (base, appearance) = self.base_and_appearance(item)?;
        // Clean EnableEquipCombi deliberately ignores gender but still uses
        // the base item's level and the combined appearance's Guide.
        Some(
            self.player.level >= base.minimum_level
                && (appearance.mentor <= 0 || appearance.mentor == self.player.guide),
        )
    }
}

pub(in super::super) fn combi_inventory_after_commit_0104(
    current: &InventoryRuntime0104,
    commit: &CombiAuthoritativeCommit0104,
) -> Result<InventoryRuntime0104, String> {
    if current.owner_pc_id() != commit.owner_pc_id() {
        return Err(format!(
            "CombiMode commit belongs to PC {}, global inventory belongs to {}",
            commit.owner_pc_id(),
            current.owner_pc_id()
        ));
    }
    let after = commit.snapshot_after();
    let mut receipt_validation = CombiAuthoritativeSnapshot0104::from_inventory_runtime(
        current,
        after.gender,
        after.level,
        after.guide,
        0,
    );
    receipt_validation
        .apply_authoritative_receipt(commit.receipt())
        .map_err(|error| error.to_string())?;
    if receipt_validation.inventory != after.inventory
        || receipt_validation.equipment != after.equipment
        || receipt_validation.taros != commit.taros_after()
    {
        return Err(
            "CombiMode receipt validation differs from its authoritative post-state".to_owned(),
        );
    }

    let mut next = current.clone();
    for write in commit.inventory_writes() {
        next.apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: InventoryLocation0104::Inventory.wire_value(),
            from_slot_num: write.inventory_index as i32,
            from_slot_item: write.item,
            to_location: InventoryLocation0104::Inventory.wire_value(),
            to_slot_num: write.inventory_index as i32,
            to_slot_item: write.item,
        })
        .map_err(|error| error.to_string())?;
    }
    if next.inventory() != &after.inventory || next.equipment() != &after.equipment {
        return Err(
            "CombiMode staged global inventory differs from its authoritative post-state"
                .to_owned(),
        );
    }
    Ok(next)
}
