use super::*;

pub fn project_combi_mode_0104(
    snapshot: &CombiAuthoritativeSnapshot0104,
    selection: CombiSelectionOverlay0104,
    catalog: &impl CombiItemCatalog0104,
    recipes: &CombiRecipeTable0104,
) -> Result<CombiModeProjection0104, CombiProjectionError0104> {
    let inventory = array::from_fn(|index| {
        project_inventory_slot(
            snapshot.inventory[index],
            selection.contains_inventory_index(index),
            catalog,
        )
    });
    let equipment =
        array::from_fn(|index| project_inventory_slot(snapshot.equipment[index], false, catalog));

    let mut look = match selection.style_inventory_index() {
        Some(index) => Some(project_look(snapshot, index, catalog)?),
        None => None,
    };
    let mut stats = match selection.stats_inventory_index() {
        Some(index) => Some(project_stats(snapshot, index, catalog)?),
        None => None,
    };

    let mut cost = 0;
    let mut combined_item_cannot_equip = false;
    let chance;
    if let (Some(look_projection), Some(stats_projection)) = (&mut look, &mut stats) {
        if look_projection.error == CombiLookError0104::None
            && stats_projection.error == CombiStatsError0104::None
        {
            let look_metadata = look_projection.appearance_metadata.as_ref().ok_or(
                CombiProjectionError0104::MissingAppearanceMetadata {
                    item_type: look_projection.item.item_type,
                    item_id: look_projection.appearance_item_id,
                },
            )?;
            let stats_metadata = stats_projection.metadata.as_ref().ok_or(
                CombiProjectionError0104::MissingBaseMetadata {
                    item_type: stats_projection.item.item_type,
                    item_id: stats_projection.item.item_id,
                },
            )?;
            let compatible = look_projection.item.item_type == stats_projection.item.item_type
                && (look_projection.item.item_type != 0
                    || look_metadata.target_mode == stats_metadata.target_mode);
            if compatible {
                combined_item_cannot_equip =
                    catalog.enable_equip_combi(stats_projection.item) != Some(true);
                let level_gap = look_metadata
                    .minimum_level
                    .abs_diff(stats_metadata.minimum_level)
                    as usize;
                let recipe = recipes
                    .row_for_level_gap(level_gap)
                    .ok_or(CombiProjectionError0104::MissingRecipeForLevelGap { level_gap })?;
                let raw_cost = i64::from(stats_metadata.item_price)
                    * i64::from(recipe.stat_constant)
                    + i64::from(look_metadata.item_price) * i64::from(recipe.look_constant);
                cost = i32::try_from(raw_cost)
                    .map_err(|_| CombiProjectionError0104::CostOverflow { raw_cost })?;
                if cost < 0 {
                    return Err(CombiProjectionError0104::NegativeCost { cost });
                }
                let rarity_gap = look_metadata.rarity.abs_diff(stats_metadata.rarity);
                let raw_percent = match rarity_gap {
                    0 => recipe.same_grade,
                    1 => recipe.one_grade,
                    2 => recipe.two_grade,
                    3 => recipe.three_grade,
                    // Clean initializes the local chance to zero and leaves it
                    // there for differences greater than three.
                    _ => 0.0,
                };
                chance = classify_clean_client_chance(raw_percent)?;
            } else {
                look_projection.error = CombiLookError0104::StatsTypeMismatch;
                stats_projection.error = CombiStatsError0104::StyleTypeMismatch;
                chance = CombiChance0104::NotPossible;
                cost = 0;
            }
        } else {
            chance = CombiChance0104::NotPossible;
        }
    } else if look
        .as_ref()
        .is_some_and(|value| value.error != CombiLookError0104::None)
        || stats
            .as_ref()
            .is_some_and(|value| value.error != CombiStatsError0104::None)
    {
        chance = CombiChance0104::NotPossible;
    } else {
        chance = CombiChance0104::NotReady;
    }

    if look
        .as_ref()
        .is_some_and(|value| value.error != CombiLookError0104::None)
        || stats
            .as_ref()
            .is_some_and(|value| value.error != CombiStatsError0104::None)
    {
        cost = 0;
    }
    let clear_enabled = look.is_some() || stats.is_some();
    let combine_enabled = look
        .as_ref()
        .is_some_and(|value| value.error == CombiLookError0104::None)
        && stats
            .as_ref()
            .is_some_and(|value| value.error == CombiStatsError0104::None);

    Ok(CombiModeProjection0104 {
        owner_pc_id: snapshot.owner_pc_id,
        taros: snapshot.taros,
        selection,
        look,
        stats,
        chance,
        cost,
        combined_item_cannot_equip,
        clear_enabled,
        combine_enabled,
        inventory,
        equipment,
    })
}

pub(super) fn project_inventory_slot(
    item: ItemBase0104,
    hidden_by_selection_overlay: bool,
    catalog: &impl CombiItemCatalog0104,
) -> CombiInventorySlotProjection0104 {
    if CombiAuthoritativeSnapshot0104::item_is_empty(item) {
        return CombiInventorySlotProjection0104::default();
    }
    let appearance_id = combined_appearance_item_id(item).unwrap_or(item.item_id);
    let icon_path = catalog
        .resolve_icon(item.item_type, appearance_id)
        .or_else(|| catalog.resolve_icon(item.item_type, item.item_id));
    CombiInventorySlotProjection0104 {
        item: Some(item),
        icon_path,
        show_combined_badge: combined_appearance_item_id(item).is_some(),
        hidden_by_selection_overlay,
    }
}

pub(super) fn project_look(
    snapshot: &CombiAuthoritativeSnapshot0104,
    index: usize,
    catalog: &impl CombiItemCatalog0104,
) -> Result<CombiLookProjection0104, CombiProjectionError0104> {
    let item = snapshot_inventory_item(snapshot, index)?;
    let valid_type = combi_equipment_item_type(item.item_type);
    if !valid_type {
        let appearance_item_id = combined_appearance_item_id(item).unwrap_or(item.item_id);
        let icon_path = catalog
            .resolve_icon(item.item_type, appearance_item_id)
            .or_else(|| catalog.resolve_icon(item.item_type, item.item_id));
        return Ok(CombiLookProjection0104 {
            inventory_index: index,
            item,
            icon_path,
            base_metadata: None,
            appearance_item_id,
            appearance_metadata: None,
            error: CombiLookError0104::UnsupportedItemType,
            cannot_equip: true,
        });
    }
    let base_metadata = catalog.resolve(item.item_type, item.item_id).ok_or(
        CombiProjectionError0104::MissingBaseMetadata {
            item_type: item.item_type,
            item_id: item.item_id,
        },
    )?;
    let appearance_item_id = combined_appearance_item_id(item).unwrap_or(item.item_id);
    let appearance_metadata = catalog.resolve(item.item_type, appearance_item_id).ok_or(
        CombiProjectionError0104::MissingAppearanceMetadata {
            item_type: item.item_type,
            item_id: appearance_item_id,
        },
    )?;
    let error = if appearance_metadata.required_gender > 0
        && appearance_metadata.required_gender != snapshot.gender
    {
        CombiLookError0104::GenderMismatch
    } else {
        CombiLookError0104::None
    };
    Ok(CombiLookProjection0104 {
        inventory_index: index,
        item,
        icon_path: appearance_metadata.icon_path.clone(),
        base_metadata: Some(base_metadata),
        appearance_item_id,
        appearance_metadata: Some(appearance_metadata),
        error,
        cannot_equip: catalog.enable_equip(item) != Some(true),
    })
}

pub(super) fn project_stats(
    snapshot: &CombiAuthoritativeSnapshot0104,
    index: usize,
    catalog: &impl CombiItemCatalog0104,
) -> Result<CombiStatsProjection0104, CombiProjectionError0104> {
    let item = snapshot_inventory_item(snapshot, index)?;
    if !combi_equipment_item_type(item.item_type) {
        let appearance_item_id = combined_appearance_item_id(item).unwrap_or(item.item_id);
        let icon_path = catalog
            .resolve_icon(item.item_type, appearance_item_id)
            .or_else(|| catalog.resolve_icon(item.item_type, item.item_id));
        return Ok(CombiStatsProjection0104 {
            inventory_index: index,
            item,
            icon_path,
            metadata: None,
            error: CombiStatsError0104::UnsupportedItemType,
            cannot_equip: true,
            point_comparison: CombiStatComparison0104::Equal,
            group_comparison: CombiStatComparison0104::Equal,
            defense_comparison: CombiStatComparison0104::Equal,
            range_label: String::new(),
        });
    }
    let metadata = catalog.resolve(item.item_type, item.item_id).ok_or(
        CombiProjectionError0104::MissingBaseMetadata {
            item_type: item.item_type,
            item_id: item.item_id,
        },
    )?;
    let combined_mentor = combined_appearance_item_id(item)
        .map(|appearance_item_id| {
            catalog
                .resolve(item.item_type, appearance_item_id)
                .ok_or(CombiProjectionError0104::MissingAppearanceMetadata {
                    item_type: item.item_type,
                    item_id: appearance_item_id,
                })
                .map(|appearance| appearance.mentor)
        })
        .transpose()?
        .unwrap_or(0);
    let error = if combined_mentor > 0 || metadata.mentor > 0 {
        CombiStatsError0104::GuideItem
    } else if metadata.cashable > 0 {
        CombiStatsError0104::ShopItem
    } else {
        CombiStatsError0104::None
    };
    let equipped_metadata = snapshot
        .equipment
        .iter()
        .copied()
        .find(|equipped| {
            !CombiAuthoritativeSnapshot0104::item_is_empty(*equipped)
                && equipped.item_type == item.item_type
        })
        .and_then(|equipped| catalog.resolve(equipped.item_type, equipped.item_id));
    let point_comparison = equipped_metadata
        .as_ref()
        .map_or(CombiStatComparison0104::Equal, |equipped| {
            compare_stat(metadata.point_rating, equipped.point_rating)
        });
    let group_comparison = equipped_metadata
        .as_ref()
        .map_or(CombiStatComparison0104::Equal, |equipped| {
            compare_stat(metadata.group_rating, equipped.group_rating)
        });
    let defense_comparison = equipped_metadata
        .as_ref()
        .map_or(CombiStatComparison0104::Equal, |equipped| {
            compare_stat(metadata.defense_rating, equipped.defense_rating)
        });
    let range_label = clean_range_label(item.item_type, metadata.equip_type, metadata.delay_time);
    Ok(CombiStatsProjection0104 {
        inventory_index: index,
        item,
        icon_path: metadata.icon_path.clone(),
        metadata: Some(metadata),
        error,
        cannot_equip: catalog.enable_equip(item) != Some(true),
        point_comparison,
        group_comparison,
        defense_comparison,
        range_label,
    })
}
