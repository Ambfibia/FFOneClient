use super::*;

pub(super) fn projected_icon(
    item: ItemBase0104,
    metadata: Option<&VendorItemMetadata0104>,
) -> VendorPresentationIcon0104 {
    if InventoryRuntime0104::item_is_empty(item) {
        return VendorPresentationIcon0104::Empty;
    }
    if item.item_type < 0 || item.item_id <= 0 {
        return VendorPresentationIcon0104::MissingChecker(
            VendorMissingIconReason0104::InvalidIdentity {
                item_type: item.item_type,
                item_id: item.item_id,
            },
        );
    }
    match metadata {
        None => {
            VendorPresentationIcon0104::MissingChecker(VendorMissingIconReason0104::CatalogMiss {
                item_type: item.item_type,
                item_id: item.item_id,
            })
        }
        Some(metadata) => match metadata.icon.clone() {
            Some(icon) => VendorPresentationIcon0104::Resolved(icon),
            None => VendorPresentationIcon0104::MissingChecker(
                VendorMissingIconReason0104::CatalogWithoutIcon {
                    item_type: item.item_type,
                    item_id: item.item_id,
                },
            ),
        },
    }
}

pub(super) fn recent_price(item: ItemBase0104, metadata: &VendorItemMetadata0104) -> i32 {
    if item.item_type == 7 {
        metadata.sell_price.saturating_mul(item.option)
    } else {
        metadata.sell_price
    }
}

#[must_use]
pub const fn vendor_server_failure_outcome(
    failure: VendorServerFailure0104,
) -> VendorActionOutcome0104 {
    let message = match failure {
        VendorServerFailure0104::Start => {
            VendorSystemMessage0104::exit(VendorSystemMessageId0104::StartFailed)
        }
        VendorServerFailure0104::TableUpdate => {
            VendorSystemMessage0104::exit(VendorSystemMessageId0104::TableUpdateFailed)
        }
        VendorServerFailure0104::Buy { error_code: 1 } => {
            VendorSystemMessage0104::plain(VendorSystemMessageId0104::PurchaseRestricted)
        }
        VendorServerFailure0104::Buy { .. } => {
            VendorSystemMessage0104::plain(VendorSystemMessageId0104::BuyFailed)
        }
        VendorServerFailure0104::Battery => {
            VendorSystemMessage0104::plain(VendorSystemMessageId0104::BatteryFailed)
        }
        VendorServerFailure0104::Sell => {
            VendorSystemMessage0104::plain(VendorSystemMessageId0104::SellFailed)
        }
        VendorServerFailure0104::Restore { .. } => {
            VendorSystemMessage0104::plain(VendorSystemMessageId0104::BuyFailed)
        }
    };
    VendorActionOutcome0104::SystemMessage(message)
}

#[must_use]
pub fn vendor_opening_eased_fraction(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds.is_nan() || elapsed_seconds <= 0.0 {
        return 0.0;
    }
    if !elapsed_seconds.is_finite() || elapsed_seconds >= VENDOR_OPEN_SECONDS {
        return 1.0;
    }
    (elapsed_seconds / VENDOR_OPEN_SECONDS * std::f32::consts::FRAC_PI_2).sin()
}

pub(super) fn vendor_row_view(
    item: ItemBase0104,
    metadata: Option<&VendorItemMetadata0104>,
    icon: &VendorPresentationIcon0104,
    price: Option<i32>,
    affordable: Option<bool>,
    equip_validation: VendorEquipValidation0104,
    vehicle_speed_class: Option<i32>,
    buyback: bool,
) -> VendorRowView0104 {
    let name = metadata
        .map(|metadata| metadata.name.clone())
        .unwrap_or_else(|| format!("UNSUPPORTED ITEM {}:{}", item.item_type, item.item_id));
    let level_label = metadata
        .map(|metadata| format!("LEVEL {}", metadata.level))
        .unwrap_or_else(|| "LEVEL --".to_owned());
    let price_label = price
        .map(|price| format!("{price} TAROS"))
        .unwrap_or_else(|| "-- TAROS".to_owned());
    VendorRowView0104 {
        name,
        level: metadata.map(|metadata| metadata.level),
        level_label,
        vehicle_speed_class,
        price,
        price_label,
        affordable,
        equip_validation,
        icon: icon.clone(),
        count_label: (buyback && item.item_type == 7).then(|| item.option.to_string()),
    }
}

pub(super) fn vendor_slot_view(item: &VendorProjectedItem0104) -> VendorSlotView0104 {
    VendorSlotView0104 {
        frame_visual: if item.empty {
            VendorSlotFrameVisual0104::Empty
        } else {
            VendorSlotFrameVisual0104::Occupied
        },
        icon: item.icon.clone(),
        show_combined_badge: item.show_combined_badge,
        count_label: item.count_label.clone(),
        quest_item_id: item.quest_item_id,
    }
}

/// Exact clean quotient/subtraction loop used by the active PCStuff Taros
/// counter. No speculative clamp is added for malformed authoritative values.
#[must_use]
pub fn vendor_taros_counter_digit_0104(mut value: i32, index: usize) -> Option<i32> {
    if index >= VENDOR_PC_STUFF_TAROS_DIGIT_RECTS.len() {
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

#[must_use]
pub fn vendor_taros_counter_digits_0104(value: i32) -> [String; 9] {
    array::from_fn(|index| {
        vendor_taros_counter_digit_0104(value, index)
            .unwrap_or_default()
            .to_string()
    })
}

pub(super) fn dispatch_vendor_activation(
    state: &mut VendorUiState,
    modal: VendorModalState,
    activation: VendorActivationOutcome0104,
    inventory_source: bool,
    outbox: &mut VendorUiOutbox0104,
) {
    match activation {
        VendorActivationOutcome0104::Popup(popup) => {
            outbox.push(VendorUiCommand0104::OpenItemActionPopup(popup));
        }
        VendorActivationOutcome0104::Action(outcome) => {
            if inventory_source {
                let _ = state.dispatch_inventory_outcome(modal, outcome, outbox);
            } else {
                let _ = state.dispatch_outcome(modal, outcome, outbox);
            }
        }
    }
}

pub(super) fn equipment_slot_label(visual_index: usize) -> String {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    match spec.label_ordinal {
        Some(ordinal) => format!("{} {ordinal}", spec.label_key),
        None => spec.label_key.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_vendor_ui(
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
    presentation: Option<Res<crate::user_equip_ui::UserEquipPresentationContext>>,
    asset_server: Res<AssetServer>,
    assets: Res<VendorUiAssets>,
    state: Res<VendorUiState>,
    modal: Res<VendorModalState>,
    projection: Res<VendorModeProjection0104>,
    hover: Res<VendorHoverState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut asset_status: ResMut<VendorUiAssetStatus>,
    mut roots: Query<(&mut Node, &mut Visibility), With<VendorUiRoot>>,
    mut elements: Query<
        (
            &VendorUiElement,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut TextColor>,
            Option<&mut LocalizedText>,
            Option<&mut BackgroundColor>,
        ),
        Without<VendorUiRoot>,
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
    let view = vendor_mode_view(
        width,
        height,
        *state,
        *modal,
        &projection,
        readiness == VendorStaticAssetReadiness::Ready,
    );
    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if view.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Some(view) = view else {
        return;
    };

    let available = |item| content.as_deref().zip(presentation.as_deref()).is_none_or(|(content, pc)| {
        content.gameplay_inventory_item_available(item, pc.level, pc.gender, pc.guide)
    });
    for (element, mut node, image, text_color, localized, background) in &mut elements {
        match *element {
            VendorUiElement::Backdrop => bind_vendor_rect(&mut node, view.layout.full_backdrop),
            VendorUiElement::VendorBackplate => {
                bind_vendor_rect(&mut node, view.layout.vendor_backplate)
            }
            VendorUiElement::RightBackplate => {
                bind_vendor_rect(&mut node, view.layout.right_backplate)
            }
            VendorUiElement::VendorPanel => bind_vendor_rect(&mut node, view.layout.vendor_panel),
            VendorUiElement::PcStuffPanel => {
                bind_vendor_rect(&mut node, view.layout.item_mode.pc_stuff_panel.into())
            }
            VendorUiElement::EquipmentPanel => {
                bind_vendor_rect(&mut node, view.layout.item_mode.equipment_panel.into())
            }
            VendorUiElement::ListContent => {
                node.left = px(0);
                node.top = px(-view.layout.vendor_scroll_y);
                node.width = px(VENDOR_ROW_WIDTH);
                node.height = px(view.rows.len() as f32 * VENDOR_ROW_HEIGHT);
            }
            VendorUiElement::InventoryContent => {
                node.left = px(0);
                node.top = px(-view.layout.inventory_scroll_y);
                node.width = px(USER_EQUIP_INVENTORY_CONTENT_WIDTH);
                node.height = px(USER_EQUIP_INVENTORY_CONTENT_HEIGHT);
            }
            VendorUiElement::ScrollThumb => {
                bind_vendor_rect(&mut node, view.layout.scroll_thumb_in_dialog());
                node.display =
                    display_if(vendor_scroll_max(projection.rows_for_tab(state.tab)) > 0.0);
            }
            VendorUiElement::ScrollTrack
            | VendorUiElement::ScrollUp
            | VendorUiElement::ScrollDown => {
                node.display =
                    display_if(vendor_scroll_max(projection.rows_for_tab(state.tab)) > 0.0);
            }
            VendorUiElement::BuyTab => {
                node.display = display_if(view.tab != VendorTab0104::Buy);
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.buy {
                        VendorStaticAssetRole::BuyHover
                    } else {
                        VendorStaticAssetRole::BuyNormal
                    });
                }
            }
            VendorUiElement::BuybackTab => {
                node.display = display_if(view.tab != VendorTab0104::Buyback);
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.buyback {
                        VendorStaticAssetRole::BuybackHover
                    } else {
                        VendorStaticAssetRole::BuybackNormal
                    });
                }
            }
            VendorUiElement::BuyTabSelected => {
                node.display = display_if(view.tab == VendorTab0104::Buy);
            }
            VendorUiElement::BuybackTabSelected => {
                node.display = display_if(view.tab == VendorTab0104::Buyback);
            }
            VendorUiElement::GoToStuff => {
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.go_to_stuff {
                        VendorStaticAssetRole::ButtonHover
                    } else {
                        VendorStaticAssetRole::ButtonNormal
                    });
                }
            }
            VendorUiElement::GoToStuffLabel => {
                if let Some(mut text_color) = text_color {
                    text_color.0 = vendor_button_text_color(hover.go_to_stuff);
                }
            }
            VendorUiElement::RedeemCode => {
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.redeem_code {
                        VendorStaticAssetRole::ButtonHover
                    } else {
                        VendorStaticAssetRole::ButtonNormal
                    });
                }
            }
            VendorUiElement::RedeemCodeLabel => {
                if let Some(mut text_color) = text_color {
                    text_color.0 = vendor_button_text_color(hover.redeem_code);
                }
            }
            VendorUiElement::VendorTitle => {
                bind_vendor_text_value(localized, vendor_title_localized(&view.npc_name));
            }
            VendorUiElement::VendorService => {
                bind_vendor_text_value(
                    localized,
                    content
                        .as_ref()
                        .and_then(|content| {
                            content
                                .gameplay_npc_service_localized(projection.session.table_vendor_id)
                        })
                        .unwrap_or_else(|| vendor_service_localized(&view.npc_service)),
                );
            }
            VendorUiElement::NpcPreviewBoundary => {
                node.display = Display::Flex;
            }
            VendorUiElement::Row(row) => {
                node.display = if row < view.rows.len() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            VendorUiElement::RowIcon(row) => {
                if let Some(row_view) = view.rows.get(row) {
                    bind_vendor_icon(
                        &mut node,
                        image,
                        &row_view.icon,
                        &asset_server,
                        &assets,
                        Color::WHITE,
                    );
                } else {
                    node.display = Display::None;
                }
            }
            VendorUiElement::RowFrameUnder(row) | VendorUiElement::RowFrameOver(row) => {
                let row_view = view.rows.get(row);
                let over = matches!(*element, VendorUiElement::RowFrameOver(_));
                let unaffordable = row_view.is_some_and(|row| row.affordable == Some(false));
                node.display = display_if(row_view.is_some() && (!over || unaffordable));
                if let (Some(_row_view), Some(mut image)) = (row_view, image) {
                    let item = match state.tab {
                        VendorTab0104::Buy => projection.catalog_rows.get(row).map(|row| row.item),
                        VendorTab0104::Buyback => projection.recent_buy_rows.get(row).map(|row| row.item),
                    };
                    let restricted = item.is_some_and(|item| !available(item));
                    image.image = assets.image(VendorStaticAssetRole::SlotOccupied);
                    if let Some(mut background) = background {
                        background.0 = if restricted { Color::srgba(0.85, 0.02, 0.02, 0.8) } else { Color::NONE };
                    }
                    image.color = if restricted && !over {
                        Color::srgba(1.0, 0.08, 0.08, 0.4)
                    } else if over {
                        Color::srgba(1.0, 1.0, 1.0, 0.4)
                    } else {
                        Color::WHITE
                    };
                }
            }
            VendorUiElement::RowCount(row) => {
                bind_optional_vendor_text(
                    &mut node,
                    localized,
                    view.rows
                        .get(row)
                        .and_then(|row| row.count_label.as_deref())
                        .map(inventory_count_localized),
                );
            }
            VendorUiElement::RowName(row) => {
                bind_optional_vendor_text(
                    &mut node,
                    localized,
                    view.rows.get(row).map(|view_row| {
                        let item = match state.tab {
                            VendorTab0104::Buy => projection.catalog_rows.get(row).map(|r| r.item),
                            VendorTab0104::Buyback => {
                                projection.recent_buy_rows.get(row).map(|r| r.item)
                            }
                        };
                        content
                            .as_ref()
                            .and_then(|content| {
                                item.and_then(|item| {
                                    content
                                        .gameplay_user_equip_item_text(item.item_type, item.item_id)
                                        .map(|text| text.0)
                                })
                            })
                            .unwrap_or_else(|| vendor_item_name_localized(&view_row.name))
                    }),
                );
            }
            VendorUiElement::RowLevel(row) => {
                bind_optional_vendor_text(
                    &mut node,
                    localized,
                    view.rows
                        .get(row)
                        .map(|row| vendor_level_localized(row.level)),
                );
            }
            VendorUiElement::RowVehicleSpeed(row) => {
                bind_optional_vendor_text(
                    &mut node,
                    localized,
                    view.rows
                        .get(row)
                        .and_then(|row| row.vehicle_speed_class)
                        .map(vendor_vehicle_speed_localized),
                );
            }
            VendorUiElement::RowPrice(row) => {
                let row_view = view.rows.get(row);
                bind_optional_vendor_text(
                    &mut node,
                    localized,
                    row_view.map(|row| vendor_price_localized(row.price)),
                );
                if let (Some(row_view), Some(mut text_color)) = (row_view, text_color) {
                    text_color.0 = if row_view.affordable == Some(false) {
                        Color::srgb(0.794_354_86, 0.0, 0.0)
                    } else {
                        Color::srgb(0.794_354_86, 1.0, 1.0)
                    };
                }
            }
            VendorUiElement::RowTaros(row) => {
                let row_view = view.rows.get(row);
                node.display = display_if(row_view.is_some());
                if let (Some(row_view), Some(mut image)) = (row_view, image) {
                    image.color = if row_view.affordable == Some(false) {
                        Color::srgb(1.0, 0.0, 0.0)
                    } else {
                        Color::WHITE
                    };
                }
            }
            VendorUiElement::InventorySlotFrame(slot) => {
                let item = &projection.inventory[slot].item;
                let restricted = !item.empty && !available(item.item);
                if let Some(mut background) = background {
                    background.0 = if restricted { Color::srgba(0.85, 0.02, 0.02, 0.8) } else { Color::NONE };
                }
                if let Some(mut image) = image {
                    image.image =
                        vendor_slot_frame_image(view.inventory[slot].frame_visual, &assets);
                    image.color = if restricted { Color::srgba(1.0, 0.08, 0.08, 0.4) } else { Color::WHITE };
                }
            }
            VendorUiElement::InventorySlotIcon(slot) => {
                bind_vendor_icon(
                    &mut node,
                    image,
                    &view.inventory[slot].icon,
                    &asset_server,
                    &assets,
                    Color::WHITE,
                );
            }
            VendorUiElement::InventorySlotBadge(slot) => {
                node.display = display_if(view.inventory[slot].show_combined_badge);
            }
            VendorUiElement::InventorySlotCount(slot) => {
                let slot = &view.inventory[slot];
                bind_optional_vendor_text(
                    &mut node,
                    localized,
                    slot.count_label
                        .as_deref()
                        .map(inventory_count_localized)
                        .or_else(|| slot.quest_item_id.map(inventory_quest_localized)),
                );
            }
            VendorUiElement::EquipmentSlotFrame(slot) => {
                if let Some(mut image) = image {
                    image.image =
                        vendor_slot_frame_image(view.equipment[slot].frame_visual, &assets);
                }
            }
            VendorUiElement::EquipmentSlotIcon(slot) => {
                bind_vendor_icon(
                    &mut node,
                    image,
                    &view.equipment[slot].icon,
                    &asset_server,
                    &assets,
                    Color::WHITE,
                );
            }
            VendorUiElement::EquipmentSlotBadge(slot) => {
                node.display = display_if(view.equipment[slot].show_combined_badge);
            }
            VendorUiElement::TarosDigit(index) => {
                bind_vendor_text_value(
                    localized,
                    vendor_taros_digit_localized(
                        vendor_taros_counter_digit_0104(view.taros, index)
                            .unwrap_or_default()
                            .to_string(),
                    ),
                );
            }
            VendorUiElement::BatteryCount(index) => {
                bind_vendor_text_value(
                    localized,
                    vendor_battery_count_localized(if index == 0 {
                        view.weapon_battery
                    } else {
                        view.nano_battery
                    }),
                );
            }
            VendorUiElement::VendorInfo
            | VendorUiElement::VendorDialog
            | VendorUiElement::ListBack
            | VendorUiElement::ListDivider
            | VendorUiElement::BuyTabLabel
            | VendorUiElement::BuybackTabLabel
            | VendorUiElement::Table
            | VendorUiElement::ListViewport
            | VendorUiElement::TableShadow
            | VendorUiElement::ItemTabLabel
            | VendorUiElement::InventoryViewport
            | VendorUiElement::InventoryShadow
            | VendorUiElement::DexlabsBanner
            | VendorUiElement::TarosCounter
            | VendorUiElement::Close
            | VendorUiElement::Trash
            | VendorUiElement::Help
            | VendorUiElement::EquipmentTitle
            | VendorUiElement::EquipmentTitleLabel
            | VendorUiElement::EquipmentSlotLabel(_)
            | VendorUiElement::BatterySlotFrame(_)
            | VendorUiElement::BatteryIcon(_)
            | VendorUiElement::BatteryLabel(_) => {}
        }
    }
}

pub(super) fn bind_vendor_text_value(localized: Option<Mut<LocalizedText>>, spec: LocalizedText) {
    let Some(mut localized) = localized else {
        return;
    };
    if *localized != spec {
        *localized = spec;
    }
}

pub(super) fn bind_optional_vendor_text(
    node: &mut Node,
    localized: Option<Mut<LocalizedText>>,
    spec: Option<LocalizedText>,
) {
    let Some(mut localized) = localized else {
        return;
    };
    if let Some(spec) = spec {
        node.display = Display::Flex;
        *localized = spec;
    } else {
        node.display = Display::None;
    }
}

pub(super) fn bind_vendor_icon(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    icon: &VendorPresentationIcon0104,
    asset_server: &AssetServer,
    assets: &VendorUiAssets,
    tint: Color,
) {
    let Some(mut image) = image else {
        return;
    };
    image.color = tint;
    match icon {
        VendorPresentationIcon0104::Empty => {
            node.display = Display::None;
            image.image = Handle::default();
        }
        VendorPresentationIcon0104::MissingChecker(_) => {
            node.display = Display::Flex;
            image.image = assets.missing_checker.clone();
        }
        VendorPresentationIcon0104::Resolved(path) => {
            let handle = asset_server.load::<Image>(path.runtime_path().to_owned());
            node.display = Display::Flex;
            // Retain the strong image handle during asynchronous loading. Replacing
            // it with the checker here cancels cold loads that have no other owner.
            image.image = if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
                assets.missing_checker.clone()
            } else {
                handle
            };
        }
    }
}
