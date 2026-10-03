use super::*;

#[must_use]
pub const fn empty_enchant_item_0104() -> ItemBase0104 {
    ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    }
}

pub fn enchant_requirements_0104(
    item: ItemBase0104,
) -> Result<EnchantRequirements0104, EnchantProjectionError0104> {
    let target_kind = EnchantTargetKind0104::from_item_type(item.item_type).ok_or(
        EnchantProjectionError0104::UnsupportedTargetItemType(item.item_type),
    )?;
    let low = item.option & 0xFFFF;
    let current_encoded_level = if low > 1 { low } else { 1 };
    // Exact cnEnchantMode call: GetAt(..., iEnchantLevel + 1).
    let recipe_index = (current_encoded_level + 1) as usize;
    let recipe = CLEAN_ENCHANT_RECIPES_0104
        .get(recipe_index)
        .copied()
        .ok_or(EnchantProjectionError0104::MissingRecipe { recipe_index })?;
    let (weapon_material_id, weapon_material_count, armor_material_id, armor_material_count) =
        if recipe.class == 0 {
            if target_kind == EnchantTargetKind0104::Hand {
                (ENCHANT_WEAPON_MATERIAL_ID_0104, recipe.weapon_matter, 0, 0)
            } else {
                (0, 0, ENCHANT_ARMOR_MATERIAL_ID_0104, recipe.costume_matter)
            }
        } else {
            (
                ENCHANT_WEAPON_MATERIAL_ID_0104,
                recipe.weapon_matter,
                ENCHANT_ARMOR_MATERIAL_ID_0104,
                recipe.costume_matter,
            )
        };
    let chance = (recipe.probability <= 100)
        .then(|| EnchantChance0104::from_probability(recipe.probability));
    Ok(EnchantRequirements0104 {
        recipe_index,
        current_encoded_level,
        displayed_level: low.saturating_sub(1).max(0),
        weapon_material_id,
        weapon_material_count,
        armor_material_id,
        armor_material_count,
        cost: recipe.cost,
        probability: recipe.probability,
        chance,
    })
}

#[must_use]
pub fn enchant_counter_digit_0104(mut value: i32, index: usize) -> Option<i32> {
    if index >= ENCHANT_TAROS_DIGIT_RECTS_0104.len() {
        return None;
    }
    let mut divisor = 100_000_000;
    for current in 0..=index {
        let digit = value / divisor;
        if current == index {
            return Some(digit);
        }
        value -= digit * divisor;
        divisor /= 10;
    }
    None
}

pub(super) fn enchant_template_text_0104(
    key: &'static str,
    fallback: &'static str,
    argument: &'static str,
    value: impl ToString,
) -> LocalizedText {
    LocalizedText::new(key, fallback).with_arg(argument, value.to_string())
}

pub(super) fn enchant_passthrough_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.content.passthrough", "{text}", "text", value)
}

pub(super) fn enchant_cost_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.cost", "{cost}", "cost", value)
}

pub(super) fn enchant_quantity_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.item.quantity", "{quantity}", "quantity", value)
}

pub(super) fn enchant_level_badge_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.item.enchant_level", "+{level}", "level", value)
}

pub(super) fn enchant_counter_digit_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.counter_digit", "{digit}", "digit", value)
}

pub(super) fn enchant_battery_count_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.battery.count", "{count}", "count", value)
}

pub(super) fn enchant_item_level_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.item.level", "LEVEL {level}", "level", value)
}

pub(super) fn enchant_stat_value_text_0104(value: impl ToString) -> LocalizedText {
    enchant_template_text_0104("ui.enchant.item.stat_value", "{value}", "value", value)
}

pub(super) fn enchant_chance_text_0104(chance: EnchantChance0104) -> LocalizedText {
    let Some((key, fallback)) = (match chance {
        EnchantChance0104::VeryLow => Some(("ui.enchant.chance.very_low", "1(Very Low)")),
        EnchantChance0104::Low => Some(("ui.enchant.chance.low", "2(Low)")),
        EnchantChance0104::Medium => Some(("ui.enchant.chance.medium", "3(Medium)")),
        EnchantChance0104::High => Some(("ui.enchant.chance.high", "4(High)")),
        EnchantChance0104::VeryHigh => Some(("ui.enchant.chance.very_high", "5(Very High)")),
        EnchantChance0104::LegacyUnassigned => None,
    }) else {
        return enchant_passthrough_text_0104("");
    };
    LocalizedText::new(key, fallback)
}

pub(super) fn enchant_equipment_slot_text_0104(visual_index: usize) -> LocalizedText {
    match visual_index {
        0 => LocalizedText::new("ui.enchant.equipment.slot.head", "HEAD"),
        1 => LocalizedText::new("ui.enchant.equipment.slot.face", "FACE"),
        2 => LocalizedText::new("ui.enchant.equipment.slot.back", "BACK"),
        3 => LocalizedText::new("ui.enchant.equipment.slot.chest", "CHEST"),
        4 => LocalizedText::new("ui.enchant.equipment.slot.legs", "LEGS"),
        5 => LocalizedText::new("ui.enchant.equipment.slot.feet", "FEET"),
        6 | 7 => LocalizedText::new("ui.enchant.equipment.slot.weapon", "WEAPON {ordinal}")
            .with_arg("ordinal", (visual_index - 5).to_string()),
        8 => LocalizedText::new("ui.enchant.equipment.slot.vehicle", "VEHICLE"),
        _ => enchant_passthrough_text_0104(""),
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn bind_enchant_ui_0104(
    asset_server: Res<AssetServer>,
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
    assets: Res<EnchantUiAssets0104>,
    projection: Res<EnchantModeProjection0104>,
    inventory_ui: Res<EnchantInventoryUiState0104>,
    mut asset_status: ResMut<EnchantUiAssetStatus0104>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(&mut Node, &mut Visibility), With<EnchantUiRoot0104>>,
    mut elements: Query<
        (
            &EnchantUiElement0104,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut LocalizedText>,
            Option<&mut TextColor>,
            Option<&Interaction>,
        ),
        Without<EnchantUiRoot0104>,
    >,
) {
    asset_status.0 = assets.readiness(&asset_server);
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0) as u32;
    let layout = enchant_mode_animated_layout_0104(
        width,
        height,
        inventory_ui.opening_elapsed_seconds(),
        inventory_ui.scroll_y(),
    );
    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if projection.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !projection.visible {
        return;
    }

    let target_attached = projection
        .selection
        .is_attached(EnchantAttachmentSlot0104::Target);
    let valid_target =
        target_attached && !projection.target_error && projection.requirements.is_some();
    let waiting = matches!(projection.phase, EnchantPhase0104::Waiting { .. });
    let success_phase = matches!(projection.phase, EnchantPhase0104::Success { .. });
    let panel_controls_enabled = inventory_ui.panel_controls_enabled();
    let base_controls_enabled = projection.capabilities.base_gui_enabled && panel_controls_enabled;

    for (element, mut node, image, localized, text_color, interaction) in &mut elements {
        if let Some(value) = content
            .as_deref()
            .and_then(|content| content::localized_field(content, &projection, *element))
        {
            let visible = match *element {
                EnchantUiElement0104::TargetName | EnchantUiElement0104::TargetDescription => {
                    valid_target
                }
                EnchantUiElement0104::SupportName(slot) => {
                    enchant_support_text_0104(&projection, slot, valid_target, false).is_some()
                }
                EnchantUiElement0104::SupportDescription(slot) => {
                    enchant_support_text_0104(&projection, slot, valid_target, true).is_some()
                }
                _ => true,
            };
            enchant_bind_optional_localized_text_0104(
                &mut node,
                localized,
                visible.then_some(value),
            );
            continue;
        }
        match *element {
            EnchantUiElement0104::Backdrop => {
                enchant_bind_rect_0104(&mut node, layout.full_backdrop)
            }
            EnchantUiElement0104::RightBackplate => {
                enchant_bind_rect_0104(&mut node, layout.right_backplate)
            }
            EnchantUiElement0104::MainGroup => enchant_bind_rect_0104(&mut node, layout.main_group),
            EnchantUiElement0104::PcStuffPanel => {
                enchant_bind_rect_0104(&mut node, layout.pc_stuff_panel)
            }
            EnchantUiElement0104::EquipmentPanel => {
                enchant_bind_rect_0104(&mut node, layout.equipment_panel)
            }
            EnchantUiElement0104::InventoryContent => {
                node.left = px(0.0);
                node.top = px(-layout.inventory_scroll_y);
                node.width = px(345.0);
                node.height = px(690.0);
            }
            EnchantUiElement0104::Chance => {
                enchant_bind_optional_localized_text_0104(
                    &mut node,
                    localized,
                    valid_target
                        .then(|| projection.requirements.and_then(|value| value.chance))
                        .flatten()
                        .map(enchant_chance_text_0104),
                );
            }
            EnchantUiElement0104::Taros => {
                let value = valid_target
                    .then(|| {
                        projection
                            .requirements
                            .map(|requirements| requirements.cost)
                    })
                    .flatten()
                    .map(|cost| enchant_cost_text_0104(cost.to_string()));
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::AttachmentFrame(slot) => {
                if let Some(mut image) = image {
                    image.image = assets.image(match projection.selection.visual_item(slot) {
                        Some(item)
                            if slot == EnchantAttachmentSlot0104::Target
                                && !item.presentation.can_equip =>
                        {
                            EnchantStaticAssetRole0104::RestrictedItemFrame
                        }
                        Some(_) => EnchantStaticAssetRole0104::SlotOccupied,
                        None => EnchantStaticAssetRole0104::SlotEmpty,
                    });
                    image.color = if base_controls_enabled {
                        Color::WHITE
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.65)
                    };
                }
            }
            EnchantUiElement0104::AttachmentIcon(slot) => {
                let (path, alpha) = enchant_attachment_icon_0104(&projection, slot, valid_target);
                enchant_bind_dynamic_icon_0104(&mut node, image, path, alpha, &asset_server);
            }
            EnchantUiElement0104::AttachmentCount(slot) => {
                let value = enchant_attachment_count_0104(&projection, slot)
                    .map(enchant_quantity_text_0104);
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::MaterialRequiredCount(slot) => {
                let value = enchant_material_required_count_0104(&projection, slot, valid_target)
                    .map(enchant_quantity_text_0104);
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::TargetCombinedBadge => {
                node.display = enchant_display_if_0104(
                    projection
                        .selection
                        .visual_item(EnchantAttachmentSlot0104::Target)
                        .is_some_and(|item| ((item.item.option >> 16) as i16) > 0),
                );
            }
            EnchantUiElement0104::TargetLevelBadge => {
                node.display = enchant_display_if_0104(
                    projection
                        .selection
                        .visual_item(EnchantAttachmentSlot0104::Target)
                        .is_some_and(|item| (item.item.option & 0xFFFF) > 1),
                );
            }
            EnchantUiElement0104::TargetLevelText => {
                let value = projection
                    .selection
                    .visual_item(EnchantAttachmentSlot0104::Target)
                    .map(|item| enchant_level_badge_text_0104((item.item.option & 0xFFFF) - 1));
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::TargetName => enchant_bind_optional_localized_text_0104(
                &mut node,
                localized,
                valid_target
                    .then(|| {
                        projection
                            .selection
                            .visual_item(EnchantAttachmentSlot0104::Target)
                    })
                    .flatten()
                    .map(|item| enchant_passthrough_text_0104(item.presentation.name.clone())),
            ),
            EnchantUiElement0104::TargetDescription => enchant_bind_optional_localized_text_0104(
                &mut node,
                localized,
                valid_target
                    .then(|| {
                        projection
                            .selection
                            .visual_item(EnchantAttachmentSlot0104::Target)
                    })
                    .flatten()
                    .map(|item| {
                        enchant_passthrough_text_0104(item.presentation.description.clone())
                    }),
            ),
            EnchantUiElement0104::MaterialXMark(slot) => {
                node.display = enchant_display_if_0104(
                    valid_target && enchant_material_id_0104(&projection, slot) == 0,
                );
            }
            EnchantUiElement0104::MaterialQuantityCover(slot)
            | EnchantUiElement0104::MaterialQuantityWarning(slot) => {
                node.display = enchant_display_if_0104(
                    valid_target && enchant_material_error_0104(&projection, slot),
                );
            }
            EnchantUiElement0104::SupportName(slot) => {
                let value = enchant_support_text_0104(&projection, slot, valid_target, false)
                    .map(enchant_passthrough_text_0104);
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::SupportDescription(slot) => {
                let value = enchant_support_text_0104(&projection, slot, valid_target, true)
                    .map(enchant_passthrough_text_0104);
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::RawCover | EnchantUiElement0104::EmptyPrompt => {
                node.display = enchant_display_if_0104(!target_attached);
            }
            EnchantUiElement0104::Preview => enchant_bind_button_0104(
                image,
                interaction,
                projection.capabilities.preview_enabled && panel_controls_enabled,
                &assets,
            ),
            EnchantUiElement0104::Clear => enchant_bind_button_0104(
                image,
                interaction,
                projection.capabilities.clear_enabled && panel_controls_enabled,
                &assets,
            ),
            EnchantUiElement0104::Enchant => enchant_bind_button_0104(
                image,
                interaction,
                projection.capabilities.enchant_enabled && panel_controls_enabled,
                &assets,
            ),
            EnchantUiElement0104::RedeemCode => {
                enchant_bind_button_0104(image, interaction, base_controls_enabled, &assets)
            }
            EnchantUiElement0104::TarosDigit(index) => {
                let value = enchant_counter_digit_0104(projection.taros, index)
                    .unwrap_or_default()
                    .to_string();
                enchant_bind_optional_localized_text_0104(
                    &mut node,
                    localized,
                    Some(enchant_counter_digit_text_0104(value)),
                );
            }
            EnchantUiElement0104::Close
            | EnchantUiElement0104::Trash
            | EnchantUiElement0104::Help => {
                if let Some(mut image) = image {
                    let enabled = match *element {
                        EnchantUiElement0104::Close => {
                            projection.capabilities.close_enabled && panel_controls_enabled
                        }
                        _ => base_controls_enabled,
                    };
                    image.color = if enabled {
                        Color::WHITE
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.45)
                    };
                }
            }
            EnchantUiElement0104::InventorySlotFrame(slot) => {
                let view = &projection.inventory[slot];
                if let Some(mut image) = image {
                    image.image = assets.image(if view.hidden_by_selection_overlay {
                        EnchantStaticAssetRole0104::SlotEmpty
                    } else if view.item.is_some() && view.restricted {
                        EnchantStaticAssetRole0104::RestrictedItemFrame
                    } else if view.item.is_some() {
                        EnchantStaticAssetRole0104::SlotOccupied
                    } else {
                        EnchantStaticAssetRole0104::SlotEmpty
                    });
                    image.color = if base_controls_enabled {
                        Color::WHITE
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.65)
                    };
                }
            }
            EnchantUiElement0104::InventorySlotIcon(slot) => {
                let view = &projection.inventory[slot];
                let path = (!view.hidden_by_selection_overlay)
                    .then_some(view.icon_path.as_deref())
                    .flatten();
                enchant_bind_dynamic_icon_0104(&mut node, image, path, 1.0, &asset_server);
            }
            EnchantUiElement0104::InventorySlotBadge(slot) => {
                let view = &projection.inventory[slot];
                node.display = enchant_display_if_0104(
                    view.show_combined_badge && !view.hidden_by_selection_overlay,
                );
            }
            EnchantUiElement0104::InventorySlotCount(slot) => {
                let view = &projection.inventory[slot];
                enchant_bind_optional_localized_text_0104(
                    &mut node,
                    localized,
                    (!view.hidden_by_selection_overlay)
                        .then_some(view.quantity_label.as_deref())
                        .flatten()
                        .map(enchant_quantity_text_0104),
                );
            }
            EnchantUiElement0104::EquipmentSlotFrame(visual_index) => {
                let wire = ENCHANT_EQUIPMENT_WIRE_ORDER_0104[visual_index];
                if let Some(mut image) = image {
                    image.image = assets.image(if projection.equipment[wire].item.is_some() {
                        EnchantStaticAssetRole0104::SlotOccupied
                    } else {
                        EnchantStaticAssetRole0104::SlotEmpty
                    });
                    image.color = if base_controls_enabled {
                        Color::WHITE
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.65)
                    };
                }
            }
            EnchantUiElement0104::EquipmentSlotIcon(visual_index) => {
                let wire = ENCHANT_EQUIPMENT_WIRE_ORDER_0104[visual_index];
                enchant_bind_dynamic_icon_0104(
                    &mut node,
                    image,
                    projection.equipment[wire].icon_path.as_deref(),
                    1.0,
                    &asset_server,
                );
            }
            EnchantUiElement0104::EquipmentSlotBadge(visual_index) => {
                let wire = ENCHANT_EQUIPMENT_WIRE_ORDER_0104[visual_index];
                node.display =
                    enchant_display_if_0104(projection.equipment[wire].show_combined_badge);
            }
            EnchantUiElement0104::EquipmentSlotLabel(visual_index) => {
                let wire = ENCHANT_EQUIPMENT_WIRE_ORDER_0104[visual_index];
                if let Some(mut text_color) = text_color {
                    text_color.0 = if projection.equipment[wire].item.is_some() {
                        Color::srgba(0.6, 1.0, 0.0, 0.8)
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.8)
                    };
                }
            }
            EnchantUiElement0104::BatterySlotFrame(_) => {
                if let Some(mut image) = image {
                    image.color = if base_controls_enabled {
                        Color::WHITE
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.65)
                    };
                }
            }
            EnchantUiElement0104::BatteryCount(index) => {
                let value = if index == 0 {
                    projection.weapon_battery
                } else {
                    projection.nano_battery
                }
                .to_string();
                enchant_bind_optional_localized_text_0104(
                    &mut node,
                    localized,
                    Some(enchant_battery_count_text_0104(value)),
                );
            }
            EnchantUiElement0104::Shade => {
                enchant_bind_rect_0104(&mut node, layout.shade);
                node.display = enchant_display_if_0104(waiting || success_phase);
            }
            EnchantUiElement0104::WaitingGroup => {
                enchant_bind_rect_0104(&mut node, layout.waiting_group);
                node.display = enchant_display_if_0104(waiting);
            }
            EnchantUiElement0104::WaitingProgress => {
                node.width = px(projection.waiting_progress_width.clamp(0.0, 414.0));
            }
            EnchantUiElement0104::SuccessGroup => {
                enchant_bind_rect_0104(&mut node, layout.success_group);
                node.display = enchant_display_if_0104(success_phase);
            }
            EnchantUiElement0104::SuccessIcon => enchant_bind_dynamic_icon_0104(
                &mut node,
                image,
                projection
                    .success
                    .as_ref()
                    .and_then(|success| success.presentation.icon_path.as_deref()),
                1.0,
                &asset_server,
            ),
            EnchantUiElement0104::SuccessLevelText => {
                let value = projection.success.as_ref().map(|success| {
                    enchant_level_badge_text_0104(success.displayed_enchant_level.to_string())
                });
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::SuccessName => enchant_bind_success_passthrough_text_0104(
                &mut node,
                localized,
                projection.success.as_ref(),
                |success| success.presentation.name.as_str(),
            ),
            EnchantUiElement0104::SuccessLevel => {
                let value = projection.success.as_ref().map(|success| {
                    enchant_item_level_text_0104(success.presentation.minimum_level.to_string())
                });
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::SuccessDescription => enchant_bind_success_passthrough_text_0104(
                &mut node,
                localized,
                projection.success.as_ref(),
                |success| success.presentation.description.as_str(),
            ),
            EnchantUiElement0104::SuccessPoint => {
                let value = projection.success.as_ref().map(|success| {
                    enchant_stat_value_text_0104(success.presentation.point_rating.to_string())
                });
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::SuccessGroupRating => {
                let value = projection.success.as_ref().map(|success| {
                    enchant_stat_value_text_0104(success.presentation.group_rating.to_string())
                });
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::SuccessDefense => {
                let value = projection.success.as_ref().map(|success| {
                    enchant_stat_value_text_0104(success.presentation.defense_rating.to_string())
                });
                enchant_bind_optional_localized_text_0104(&mut node, localized, value);
            }
            EnchantUiElement0104::SuccessTypeValue => enchant_bind_success_passthrough_text_0104(
                &mut node,
                localized,
                projection.success.as_ref(),
                |success| success.presentation.type_label.as_str(),
            ),
            EnchantUiElement0104::SuccessRangeValue => enchant_bind_success_passthrough_text_0104(
                &mut node,
                localized,
                projection.success.as_ref(),
                |success| success.presentation.range_label.as_str(),
            ),
            EnchantUiElement0104::SuccessRarityValue => enchant_bind_success_passthrough_text_0104(
                &mut node,
                localized,
                projection.success.as_ref(),
                |success| success.presentation.rarity_label.as_str(),
            ),
            EnchantUiElement0104::SuccessTradeValue => enchant_bind_success_passthrough_text_0104(
                &mut node,
                localized,
                projection.success.as_ref(),
                |success| success.presentation.trade_label.as_str(),
            ),
            EnchantUiElement0104::EnchantMoreItems | EnchantUiElement0104::GoToMyStuff => {
                enchant_bind_button_0104(
                    image,
                    interaction,
                    projection.capabilities.success_buttons_enabled,
                    &assets,
                )
            }
            EnchantUiElement0104::SuccessLevelBadge => {
                node.display = enchant_display_if_0104(projection.success.is_some());
            }
            EnchantUiElement0104::Panel
            | EnchantUiElement0104::PrimaryNpcBoundary
            | EnchantUiElement0104::Title
            | EnchantUiElement0104::Intro
            | EnchantUiElement0104::TargetTitle
            | EnchantUiElement0104::NeededTitle
            | EnchantUiElement0104::HelpTitle
            | EnchantUiElement0104::ChanceTitle
            | EnchantUiElement0104::TarosTitle
            | EnchantUiElement0104::ItemTabLabel
            | EnchantUiElement0104::InventoryViewport
            | EnchantUiElement0104::EquipmentTitle
            | EnchantUiElement0104::EquipmentTitleLabel
            | EnchantUiElement0104::DexlabsBanner
            | EnchantUiElement0104::TarosCounter
            | EnchantUiElement0104::BatteryIcon(_)
            | EnchantUiElement0104::BatteryLabel(_)
            | EnchantUiElement0104::WaitingLabel
            | EnchantUiElement0104::WaitingNpcBoundary
            | EnchantUiElement0104::SuccessNpc
            | EnchantUiElement0104::SuccessHooray
            | EnchantUiElement0104::SuccessMessage
            | EnchantUiElement0104::SuccessTypeLabel
            | EnchantUiElement0104::SuccessRangeLabel
            | EnchantUiElement0104::SuccessRarityLabel
            | EnchantUiElement0104::SuccessTradeLabel => {}
        }
    }
}

pub(super) const fn enchant_display_if_0104(value: bool) -> Display {
    if value { Display::Flex } else { Display::None }
}

pub(super) fn enchant_bind_success_passthrough_text_0104<'a>(
    node: &mut Node,
    localized: Option<Mut<LocalizedText>>,
    success: Option<&'a EnchantSuccessPresentation0104>,
    value: impl FnOnce(&'a EnchantSuccessPresentation0104) -> &'a str,
) {
    enchant_bind_optional_localized_text_0104(
        node,
        localized,
        success.map(value).map(enchant_passthrough_text_0104),
    );
}

pub(super) fn enchant_bind_dynamic_icon_0104(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    path: Option<&str>,
    alpha: f32,
    asset_server: &AssetServer,
) {
    let Some(mut image) = image else {
        return;
    };
    let Some(path) = path.filter(|path| enchant_safe_relative_asset_path_0104(path)) else {
        node.display = Display::None;
        image.image = Handle::default();
        return;
    };
    let handle = asset_server.load::<Image>(path.to_owned());
    let display = if matches!(asset_server.load_state(handle.id()), LoadState::Loaded) {
        Display::Flex
    } else {
        Display::None
    };
    // Keep the requested image alive while loading. Dropping the only strong
    // handle here cancels it before it can become visible on a later frame.
    if image.image != handle {
        image.image = handle;
    }
    if node.display != display {
        node.display = display;
    }
    let color = Color::srgba(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0));
    if image.color != color {
        image.color = color;
    }
}

pub(super) fn enchant_attachment_icon_0104<'a>(
    projection: &'a EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
    valid_target: bool,
) -> (Option<&'a str>, f32) {
    if let Some(item) = projection.selection.visual_item(slot) {
        return (item.presentation.icon_path.as_deref(), 1.0);
    }
    if !valid_target {
        return (None, 1.0);
    }
    match slot {
        EnchantAttachmentSlot0104::Target => (None, 1.0),
        EnchantAttachmentSlot0104::WeaponMaterial => {
            if enchant_material_id_0104(projection, slot) == 0 {
                (None, 1.0)
            } else {
                (projection.support.weapon_material.icon_path.as_deref(), 0.3)
            }
        }
        EnchantAttachmentSlot0104::ArmorMaterial => {
            if enchant_material_id_0104(projection, slot) == 0
                || projection.support.armor_material.icon_path.is_none()
            {
                (None, 1.0)
            } else {
                // Exact clean typo: test texArmorRawItem, draw texWpnRawItem.
                (projection.support.weapon_material.icon_path.as_deref(), 0.3)
            }
        }
        EnchantAttachmentSlot0104::Helper1 => {
            (projection.support.helper_1.icon_path.as_deref(), 0.3)
        }
        EnchantAttachmentSlot0104::Helper2 => {
            if projection.support.helper_1.icon_path.is_some() {
                // Exact clean typo: helper-two preview is gated by helper one.
                (projection.support.helper_2.icon_path.as_deref(), 0.3)
            } else {
                (None, 1.0)
            }
        }
    }
}

pub(super) fn enchant_attachment_count_0104(
    projection: &EnchantModeProjection0104,
    slot: EnchantAttachmentSlot0104,
) -> Option<String> {
    if !matches!(
        slot,
        EnchantAttachmentSlot0104::WeaponMaterial | EnchantAttachmentSlot0104::ArmorMaterial
    ) {
        return None;
    }
    if let Some(item) = projection.selection.visual_item(slot) {
        return Some(item.item.option.to_string());
    }
    None
}
