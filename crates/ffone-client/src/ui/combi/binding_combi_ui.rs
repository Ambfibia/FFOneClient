use super::*;

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn bind_combi_ui(
    asset_server: Res<AssetServer>,
    assets: Res<CombiUiAssets>,
    state: Res<CombiUiState0104>,
    projection: Res<CombiModeProjection0104>,
    pointer: Res<CombiPointerState0104>,
    windows: Query<&Window, With<PrimaryWindow>>,
    controls: Query<(&CombiInteractiveControl0104, &Interaction)>,
    mut asset_status: ResMut<CombiUiAssetStatus0104>,
    mut roots: Query<(&mut Node, &mut Visibility), With<CombiUiRoot0104>>,
    mut elements: Query<
        (
            &CombiUiElement0104,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut LocalizedText>,
            Option<&mut TextColor>,
            Option<&Interaction>,
            Option<&mut BackgroundColor>,
            Option<&mut BorderColor>,
        ),
        Without<CombiUiRoot0104>,
    >,
) {
    let readiness = assets.readiness(&asset_server);
    asset_status.0 = readiness;
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0) as u32;
    let capabilities = state.input_capabilities(&projection);
    let visible = capabilities.draw
        && readiness == CombiStaticAssetReadiness0104::Ready
        && width > 0
        && height > 0;
    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !visible {
        return;
    }
    let layout = combi_mode_layout_0104(width, height);
    let success = match &state.phase {
        CombiPhase0104::Success(success) => Some(success),
        _ => None,
    };
    let waiting = matches!(state.phase, CombiPhase0104::Waiting { .. });
    let show_overlay = waiting || success.is_some();
    let carried = pointer
        .carried()
        .filter(|_| capabilities.selection_controls)
        .map(|carried| carried.inventory_index);
    let hovered = |target: CombiInteractiveControl0104| {
        controls.iter().any(|(control, interaction)| {
            *control == target && *interaction == Interaction::Hovered
        })
    };
    let style_target = carried.is_some()
        && (hovered(CombiInteractiveControl0104::LookDrop)
            || hovered(CombiInteractiveControl0104::LookSelection));
    let stats_target = carried.is_some()
        && (hovered(CombiInteractiveControl0104::StatsDrop)
            || hovered(CombiInteractiveControl0104::StatsSelection));
    let cursor = window.cursor_position();

    for (element, mut node, image, localized, text_color, interaction, background, border) in
        &mut elements
    {
        match *element {
            CombiUiElement0104::Backdrop => bind_combi_rect(&mut node, layout.full_backdrop),
            CombiUiElement0104::Panel => bind_combi_rect(&mut node, layout.panel),
            CombiUiElement0104::RightBackplate => {
                bind_combi_rect(&mut node, layout.right_backplate)
            }
            CombiUiElement0104::MainGroup => bind_combi_rect(&mut node, layout.main_group),
            CombiUiElement0104::PcStuffPanel => bind_combi_rect(&mut node, layout.pc_stuff_panel),
            CombiUiElement0104::EquipmentPanel => {
                bind_combi_rect(&mut node, layout.equipment_panel)
            }
            CombiUiElement0104::Shade => {
                bind_combi_rect(&mut node, layout.shade);
                node.display = display_if(show_overlay);
            }
            CombiUiElement0104::SuccessGroup => {
                bind_combi_rect(&mut node, layout.success_group);
                node.display = display_if(success.is_some());
            }
            CombiUiElement0104::WaitingGroup => {
                bind_combi_rect(&mut node, layout.waiting_group);
                node.display = display_if(waiting);
            }
            CombiUiElement0104::PrimaryNpcBoundary => {
                node.display = if state.primary_npc_camera_bound {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            CombiUiElement0104::WaitingNpcBoundary => {
                node.display = if waiting && state.waiting_npc_camera_bound {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            CombiUiElement0104::Cost => {
                bind_localized_text(localized, combi_cost_text(projection.cost));
            }
            CombiUiElement0104::Chance => {
                bind_localized_text(localized, projection.chance.localized_text());
                if let Some(mut text_color) = text_color {
                    text_color.0 = projection.chance.color().bevy();
                }
            }
            CombiUiElement0104::ChanceLevel => {
                let label = if projection.chance.level() == -1 {
                    "?".to_owned()
                } else {
                    projection.chance.level().to_string()
                };
                bind_localized_text(localized, combi_chance_level_text(label));
            }
            CombiUiElement0104::LookBackground => {
                node.display = display_if(projection.look.is_some());
            }
            CombiUiElement0104::LookSelectionFrame => {
                bind_selection_frame(
                    image,
                    projection.look.as_ref().map(|look| look.cannot_equip),
                    &assets,
                );
            }
            CombiUiElement0104::LookSelectionIcon => {
                let icon = projection.look.as_ref().and_then(look_icon_path);
                bind_dynamic_icon(&mut node, image, icon, &asset_server);
            }
            CombiUiElement0104::LookSelectionBadge => {
                node.display = display_if(
                    projection
                        .look
                        .as_ref()
                        .is_some_and(|look| combined_appearance_item_id(look.item).is_some()),
                );
            }
            CombiUiElement0104::LookDetailIcon => {
                let icon = projection.look.as_ref().and_then(look_icon_path);
                bind_dynamic_icon(&mut node, image, icon, &asset_server);
            }
            CombiUiElement0104::LookDetailRestricted => {
                node.display =
                    display_if(projection.look.is_some() && projection.combined_item_cannot_equip);
            }
            CombiUiElement0104::LookDetailBadge => {
                node.display = display_if(projection.look.is_some());
            }
            CombiUiElement0104::LookName => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .look
                        .as_ref()
                        .and_then(|look| look.appearance_metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.name.as_str())),
                );
            }
            CombiUiElement0104::LookLevel => {
                let localized_value = projection
                    .look
                    .as_ref()
                    .and_then(|look| look.base_metadata.as_ref())
                    .map(|metadata| combi_item_level_text(metadata.minimum_level));
                bind_optional_localized_text(&mut node, localized, localized_value);
            }
            CombiUiElement0104::LookDescription => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .look
                        .as_ref()
                        .and_then(|look| look.appearance_metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.description.as_str())),
                );
            }
            CombiUiElement0104::LookErrorBackground => {
                node.display = display_if(
                    projection
                        .look
                        .as_ref()
                        .is_some_and(|look| look.error != CombiLookError0104::None),
                );
            }
            CombiUiElement0104::LookErrorText => {
                let rect = if projection
                    .look
                    .as_ref()
                    .is_some_and(|look| look.error == CombiLookError0104::UnsupportedItemType)
                {
                    CombiUiRect::new(215.0, 221.0, 202.0, 24.0)
                } else {
                    CombiUiRect::new(245.0, 221.0, 152.0, 24.0)
                };
                bind_combi_rect(&mut node, rect);
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .look
                        .as_ref()
                        .and_then(|look| look.error.localized_text()),
                );
            }
            CombiUiElement0104::LookEmptyText => {
                node.display = display_if(projection.look.is_none());
            }
            CombiUiElement0104::StatBackground => {
                node.display = display_if(projection.stats.is_some());
            }
            CombiUiElement0104::StatSelectionFrame => {
                bind_selection_frame(
                    image,
                    projection.stats.as_ref().map(|stats| stats.cannot_equip),
                    &assets,
                );
            }
            CombiUiElement0104::StatSelectionIcon => {
                let icon = projection.stats.as_ref().and_then(stats_icon_path);
                bind_dynamic_icon(&mut node, image, icon, &asset_server);
            }
            CombiUiElement0104::StatSelectionBadge => {
                node.display = display_if(
                    projection
                        .stats
                        .as_ref()
                        .is_some_and(|stats| combined_appearance_item_id(stats.item).is_some()),
                );
            }
            CombiUiElement0104::StatLevel => {
                let localized_value = projection
                    .stats
                    .as_ref()
                    .and_then(|stats| stats.metadata.as_ref())
                    .map(|metadata| combi_item_level_text(metadata.minimum_level));
                bind_optional_localized_text(&mut node, localized, localized_value);
            }
            CombiUiElement0104::StatSection | CombiUiElement0104::InfoSection => {
                node.display = display_if(projection.stats.is_some());
            }
            CombiUiElement0104::StatSingle => {
                bind_stat_value(
                    &mut node,
                    localized,
                    text_color,
                    projection.stats.as_ref(),
                    |metadata| metadata.point_rating,
                    |stats| stats.point_comparison,
                );
            }
            CombiUiElement0104::StatMulti => {
                bind_stat_value(
                    &mut node,
                    localized,
                    text_color,
                    projection.stats.as_ref(),
                    |metadata| metadata.group_rating,
                    |stats| stats.group_comparison,
                );
            }
            CombiUiElement0104::StatDefense => {
                bind_stat_value(
                    &mut node,
                    localized,
                    text_color,
                    projection.stats.as_ref(),
                    |metadata| metadata.defense_rating,
                    |stats| stats.defense_comparison,
                );
            }
            CombiUiElement0104::InfoTypeLabel
            | CombiUiElement0104::InfoRangeLabel
            | CombiUiElement0104::InfoRarityLabel
            | CombiUiElement0104::InfoTradeLabel => {
                node.display = display_if(projection.stats.is_some());
            }
            CombiUiElement0104::InfoTypeValue => {
                let localized_value = projection
                    .stats
                    .as_ref()
                    .filter(|stats| stats.metadata.is_some())
                    .and_then(|stats| combi_type_text(stats.item.item_type));
                bind_optional_localized_text(&mut node, localized, localized_value);
            }
            CombiUiElement0104::InfoRangeValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .stats
                        .as_ref()
                        .map(|stats| combi_range_text(stats.range_label.as_str())),
                );
            }
            CombiUiElement0104::InfoRarityValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .stats
                        .as_ref()
                        .and_then(|stats| stats.metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.rarity_label.as_str())),
                );
            }
            CombiUiElement0104::InfoTradeValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .stats
                        .as_ref()
                        .and_then(|stats| stats.metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.trade_label.as_str())),
                );
            }
            CombiUiElement0104::StatErrorBackground => {
                node.display = display_if(
                    projection
                        .stats
                        .as_ref()
                        .is_some_and(|stats| stats.error != CombiStatsError0104::None),
                );
            }
            CombiUiElement0104::StatErrorText => {
                let rect =
                    if projection.stats.as_ref().is_some_and(|stats| {
                        stats.error == CombiStatsError0104::UnsupportedItemType
                    }) {
                        CombiUiRect::new(215.0, 423.0, 202.0, 24.0)
                    } else {
                        CombiUiRect::new(245.0, 423.0, 152.0, 24.0)
                    };
                bind_combi_rect(&mut node, rect);
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    projection
                        .stats
                        .as_ref()
                        .and_then(|stats| stats.error.localized_text()),
                );
            }
            CombiUiElement0104::StatEmptyText => {
                node.display = display_if(projection.stats.is_none());
            }
            CombiUiElement0104::ClearAll => {
                bind_button_visual(image, interaction, capabilities.clear, &assets);
            }
            CombiUiElement0104::Combine => {
                bind_button_visual(image, interaction, capabilities.combine, &assets);
            }
            CombiUiElement0104::InventorySlotFrame(slot) => {
                let slot_view = &projection.inventory[slot];
                if let Some(mut image) = image {
                    image.image = assets.image(
                        if slot_view.item.is_some() && !slot_view.hidden_by_selection_overlay {
                            CombiStaticAssetRole::SlotOccupied
                        } else {
                            CombiStaticAssetRole::SlotEmpty
                        },
                    );
                }
            }
            CombiUiElement0104::InventorySlotIcon(slot) => {
                let slot_view = &projection.inventory[slot];
                let path = (!slot_view.hidden_by_selection_overlay)
                    .then_some(slot_view.icon_path.as_deref())
                    .flatten();
                // The carried item stays in its cell at half strength, as in My Stuff.
                let tint = if carried == Some(slot) {
                    Color::srgba(1.0, 1.0, 1.0, 0.5)
                } else {
                    Color::WHITE
                };
                bind_dynamic_icon_tinted(&mut node, image, path, &asset_server, tint);
            }
            CombiUiElement0104::InventorySlotBadge(slot) => {
                let slot_view = &projection.inventory[slot];
                node.display = display_if(
                    slot_view.show_combined_badge && !slot_view.hidden_by_selection_overlay,
                );
            }
            CombiUiElement0104::EquipmentSlotFrame(visual_index) => {
                let wire_slot = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index;
                if let Some(mut image) = image {
                    image.image = assets.image(if projection.equipment[wire_slot].item.is_some() {
                        CombiStaticAssetRole::SlotOccupied
                    } else {
                        CombiStaticAssetRole::SlotEmpty
                    });
                }
            }
            CombiUiElement0104::EquipmentSlotIcon(visual_index) => {
                let wire_slot = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index;
                bind_dynamic_icon(
                    &mut node,
                    image,
                    projection.equipment[wire_slot].icon_path.as_deref(),
                    &asset_server,
                );
            }
            CombiUiElement0104::EquipmentSlotBadge(visual_index) => {
                let wire_slot = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index;
                node.display = display_if(projection.equipment[wire_slot].show_combined_badge);
            }
            CombiUiElement0104::SuccessRestricted => {
                node.display =
                    display_if(success.is_some_and(|success| success.combined_item_cannot_equip));
            }
            CombiUiElement0104::SuccessIcon => {
                bind_dynamic_icon(
                    &mut node,
                    image,
                    success.and_then(|success| look_icon_path(&success.look)),
                    &asset_server,
                );
            }
            CombiUiElement0104::SuccessName => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    success
                        .and_then(|success| success.look.appearance_metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.name.as_str())),
                );
            }
            CombiUiElement0104::SuccessLevel => {
                let localized_value = success
                    .and_then(|success| success.stats.metadata.as_ref())
                    .map(|metadata| combi_item_level_text(metadata.minimum_level));
                bind_optional_localized_text(&mut node, localized, localized_value);
            }
            CombiUiElement0104::SuccessDescription => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    success
                        .and_then(|success| success.look.appearance_metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.description.as_str())),
                );
            }
            CombiUiElement0104::SuccessSingle => {
                bind_success_stat_value(
                    &mut node,
                    localized,
                    text_color,
                    success,
                    |metadata| metadata.point_rating,
                    |stats| stats.point_comparison,
                );
            }
            CombiUiElement0104::SuccessMulti => {
                bind_success_stat_value(
                    &mut node,
                    localized,
                    text_color,
                    success,
                    |metadata| metadata.group_rating,
                    |stats| stats.group_comparison,
                );
            }
            CombiUiElement0104::SuccessDefense => {
                bind_success_stat_value(
                    &mut node,
                    localized,
                    text_color,
                    success,
                    |metadata| metadata.defense_rating,
                    |stats| stats.defense_comparison,
                );
            }
            CombiUiElement0104::SuccessTypeValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    success.and_then(|success| combi_type_text(success.stats.item.item_type)),
                );
            }
            CombiUiElement0104::SuccessRangeValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    success.map(|success| combi_range_text(success.stats.range_label.as_str())),
                );
            }
            CombiUiElement0104::SuccessRarityValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    success
                        .and_then(|success| success.stats.metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.rarity_label.as_str())),
                );
            }
            CombiUiElement0104::SuccessTradeValue => {
                bind_optional_localized_text(
                    &mut node,
                    localized,
                    success
                        .and_then(|success| success.stats.metadata.as_ref())
                        .map(|metadata| combi_passthrough_text(metadata.trade_label.as_str())),
                );
            }
            CombiUiElement0104::SuccessTypeLabel
            | CombiUiElement0104::SuccessRangeLabel
            | CombiUiElement0104::SuccessRarityLabel
            | CombiUiElement0104::SuccessTradeLabel
            | CombiUiElement0104::SuccessNpcIcon
            | CombiUiElement0104::SuccessHooray
            | CombiUiElement0104::SuccessMessage
            | CombiUiElement0104::SuccessBadge => {
                node.display = display_if(success.is_some());
            }
            CombiUiElement0104::CombineMore | CombiUiElement0104::GoToStuff => {
                node.display = display_if(success.is_some());
                bind_button_visual(image, interaction, capabilities.success_controls, &assets);
            }
            CombiUiElement0104::InventorySlotHover(slot) => {
                let frame = if carried.is_none()
                    && capabilities.selection_controls
                    && combi_inventory_slot_draggable(&projection, slot)
                    && hovered(CombiInteractiveControl0104::InventorySlot(slot))
                {
                    CombiHoverFrame::Hover
                } else {
                    CombiHoverFrame::Hidden
                };
                bind_hover_frame(&mut node, background, border, frame);
            }
            CombiUiElement0104::EquipmentSlotHover(visual_index) => {
                let wire_slot = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index;
                // Equipped items are refused with message 260, so they read as blocked.
                let frame = if carried.is_none()
                    && capabilities.selection_controls
                    && projection.equipment[wire_slot].item.is_some()
                    && hovered(CombiInteractiveControl0104::EquipmentSlot(visual_index))
                {
                    CombiHoverFrame::Blocked
                } else {
                    CombiHoverFrame::Hidden
                };
                bind_hover_frame(&mut node, background, border, frame);
            }
            CombiUiElement0104::LookSelectionHover => {
                let frame = if style_target {
                    CombiHoverFrame::Target
                } else if carried.is_none()
                    && capabilities.selection_controls
                    && projection.look.is_some()
                    && hovered(CombiInteractiveControl0104::LookSelection)
                {
                    CombiHoverFrame::Hover
                } else {
                    CombiHoverFrame::Hidden
                };
                bind_hover_frame(&mut node, background, border, frame);
            }
            CombiUiElement0104::StatSelectionHover => {
                let frame = if stats_target {
                    CombiHoverFrame::Target
                } else if carried.is_none()
                    && capabilities.selection_controls
                    && projection.stats.is_some()
                    && hovered(CombiInteractiveControl0104::StatsSelection)
                {
                    CombiHoverFrame::Hover
                } else {
                    CombiHoverFrame::Hidden
                };
                bind_hover_frame(&mut node, background, border, frame);
            }
            CombiUiElement0104::LookDropHighlight => {
                bind_hover_frame(
                    &mut node,
                    background,
                    border,
                    combi_drop_frame(carried, style_target),
                );
            }
            CombiUiElement0104::StatDropHighlight => {
                bind_hover_frame(
                    &mut node,
                    background,
                    border,
                    combi_drop_frame(carried, stats_target),
                );
            }
            CombiUiElement0104::DraggedItem => {
                let icon = carried
                    .and_then(|index| projection.inventory.get(index))
                    .and_then(|slot| slot.icon_path.as_deref());
                if let (Some(icon), Some(cursor)) = (icon, cursor) {
                    bind_dynamic_icon(&mut node, image, Some(icon), &asset_server);
                    node.left = px(cursor.x - USER_EQUIP_INVENTORY_SLOT_SIZE * 0.5);
                    node.top = px(cursor.y - USER_EQUIP_INVENTORY_SLOT_SIZE * 0.5);
                } else {
                    node.display = Display::None;
                }
            }
            CombiUiElement0104::InventoryContent => {
                node.top = px(-pointer.scroll_y());
            }
            CombiUiElement0104::Close => {
                bind_hover_image(
                    image,
                    interaction,
                    capabilities.close,
                    [
                        CombiStaticAssetRole::Close,
                        CombiStaticAssetRole::CloseHover,
                    ],
                    &assets,
                );
            }
            CombiUiElement0104::Help => {
                bind_hover_image(
                    image,
                    interaction,
                    capabilities.main_controls,
                    [CombiStaticAssetRole::Help, CombiStaticAssetRole::HelpHover],
                    &assets,
                );
            }
            CombiUiElement0104::Title
            | CombiUiElement0104::Intro
            | CombiUiElement0104::StyleTitle
            | CombiUiElement0104::StatsTitle
            | CombiUiElement0104::TarosLabel
            | CombiUiElement0104::TarosIcon
            | CombiUiElement0104::ChanceTitle
            | CombiUiElement0104::NewItemTitle
            | CombiUiElement0104::LookDrop
            | CombiUiElement0104::StatDrop
            | CombiUiElement0104::ItemTabLabel
            | CombiUiElement0104::InventoryViewport
            | CombiUiElement0104::EquipmentTitle
            | CombiUiElement0104::Trash => {}
        }
    }
}

pub(super) fn bind_dynamic_icon(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    path: Option<&str>,
    asset_server: &AssetServer,
) {
    bind_dynamic_icon_tinted(node, image, path, asset_server, Color::WHITE);
}

pub(super) fn bind_dynamic_icon_tinted(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    path: Option<&str>,
    asset_server: &AssetServer,
    tint: Color,
) {
    let Some(mut image) = image else {
        return;
    };
    let Some(path) = path.filter(|path| safe_relative_asset_path(path)) else {
        node.display = Display::None;
        image.image = Handle::default();
        return;
    };
    let handle = asset_server.load::<Image>(path.to_owned());
    if matches!(asset_server.load_state(handle.id()), LoadState::Loaded) {
        node.display = Display::Flex;
        image.image = handle;
        image.color = tint;
    } else {
        // Fail closed: no guessed item image or diagnostic artwork.
        node.display = Display::None;
        image.image = Handle::default();
    }
}

pub(super) fn bind_stat_value(
    node: &mut Node,
    localized: Option<Mut<LocalizedText>>,
    text_color: Option<Mut<TextColor>>,
    stats: Option<&CombiStatsProjection0104>,
    value: impl FnOnce(&CombiItemMetadata0104) -> i32,
    comparison: impl FnOnce(&CombiStatsProjection0104) -> CombiStatComparison0104,
) {
    let Some(stats) = stats else {
        node.display = Display::None;
        bind_localized_text(localized, combi_passthrough_text(""));
        return;
    };
    let Some(metadata) = stats.metadata.as_ref() else {
        node.display = Display::None;
        bind_localized_text(localized, combi_passthrough_text(""));
        return;
    };
    node.display = Display::Flex;
    bind_localized_text(localized, combi_stat_value_text(value(metadata)));
    if let Some(mut text_color) = text_color {
        text_color.0 = comparison(stats).color().bevy();
    }
}

pub(super) fn bind_success_stat_value(
    node: &mut Node,
    localized: Option<Mut<LocalizedText>>,
    text_color: Option<Mut<TextColor>>,
    success: Option<&CombiSuccessPresentation0104>,
    value: impl FnOnce(&CombiItemMetadata0104) -> i32,
    comparison: impl FnOnce(&CombiStatsProjection0104) -> CombiStatComparison0104,
) {
    let Some(success) = success else {
        node.display = Display::None;
        bind_localized_text(localized, combi_passthrough_text(""));
        return;
    };
    let Some(metadata) = success.stats.metadata.as_ref() else {
        node.display = Display::None;
        bind_localized_text(localized, combi_passthrough_text(""));
        return;
    };
    node.display = Display::Flex;
    bind_localized_text(localized, combi_stat_value_text(value(metadata)));
    if let Some(mut text_color) = text_color {
        text_color.0 = comparison(&success.stats).color().bevy();
    }
}

#[must_use]
pub const fn clean_type_label(item_type: i16) -> &'static str {
    match item_type {
        0 => "Weapon",
        1 => "Body",
        2 => "Legs",
        3 => "Shoes",
        _ => "",
    }
}
