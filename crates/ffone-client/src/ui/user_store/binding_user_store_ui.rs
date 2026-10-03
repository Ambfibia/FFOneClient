use super::*;

#[must_use]
pub const fn user_store_item_is_empty(item: ItemBase0104) -> bool {
    item.item_type == 0 && item.item_id == 0
}

pub(super) fn push_entry_commands(outbox: &mut UserStoreUiOutbox0104, packet: UserStorePacket0104) {
    outbox.push(UserStoreUiCommand0104::UnlockCursor);
    outbox.push(UserStoreUiCommand0104::ActivateStoreInventoryMode);
    outbox.push(UserStoreUiCommand0104::ActivateSharedPanels);
    outbox.push(UserStoreUiCommand0104::StartUiModeSound);
    outbox.push(UserStoreUiCommand0104::SendPacket(packet));
}

#[must_use]
pub fn user_store_opening_eased_fraction(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds.is_nan() || elapsed_seconds <= 0.0 {
        0.0
    } else if !elapsed_seconds.is_finite() || elapsed_seconds >= USER_STORE_OPEN_SECONDS {
        1.0
    } else {
        (elapsed_seconds / USER_STORE_OPEN_SECONDS * std::f32::consts::FRAC_PI_2).sin()
    }
}

#[must_use]
pub fn user_store_seller_display_taros(price: i32) -> i32 {
    price - ((f64::from(price) * 0.05) as i32)
}

pub fn seed_user_store_preview_0104(
    mode: UserStorePreviewMode0104,
    state: &mut UserStoreUiState0104,
    authority: &mut UserStoreAuthority0104,
    catalog: &mut UserStoreItemCatalog0104,
) {
    let items = [
        ItemBase0104 {
            item_type: 0,
            item_id: 311,
            option: 1,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 7,
            item_id: 75,
            option: 12,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 10,
            item_id: 42,
            option: 1,
            time_limit: 0,
        },
    ];
    for (item, name, level, description) in [
        (
            items[0],
            "RETRO BLASTER",
            12,
            "A compact blaster from the future of the past.",
        ),
        (
            items[1],
            "HEALTH SNACK",
            1,
            "A quick snack that restores a little health.",
        ),
        (
            items[2],
            "TURBO BOARD",
            18,
            "A fast board built for getting around in style.",
        ),
    ] {
        catalog.insert(
            item.item_type,
            item.item_id,
            UserStoreItemMetadata0104 {
                name: name.into(),
                level,
                description: description.into(),
                icon_path: None,
            },
        );
    }
    *state = UserStoreUiState0104 {
        active: true,
        mode: if matches!(
            mode,
            UserStorePreviewMode0104::UserList
                | UserStorePreviewMode0104::UserSold
                | UserStorePreviewMode0104::PopupBuy
        ) {
            UserStoreMode0104::UserStore
        } else {
            UserStoreMode0104::MyStore
        },
        opening_elapsed_seconds: USER_STORE_OPEN_SECONDS,
        ..default()
    };
    *authority = UserStoreAuthority0104 {
        owner_pc_id: 42,
        target_pc_id: (state.mode == UserStoreMode0104::UserStore).then_some(77),
        taros: 50_000,
        maximum_list_slots: USER_STORE_LIST_CAPACITY,
        open_item_inventory_slot: 0,
        open_item: items[0],
        store_open: matches!(
            mode,
            UserStorePreviewMode0104::MyOpen
                | UserStorePreviewMode0104::UserList
                | UserStorePreviewMode0104::UserSold
                | UserStorePreviewMode0104::PopupBuy
        ),
        ..default()
    };
    authority.inventory[0] = items[0];
    authority.inventory[1] = items[1];
    authority.inventory[8] = items[2];
    if !matches!(mode, UserStorePreviewMode0104::MyReady) {
        authority.listings[0] = Some(UserStoreListing0104 {
            list_slot: 0,
            inventory_slot: (state.mode == UserStoreMode0104::MyStore).then_some(2),
            item: items[0],
            price: 975,
        });
        authority.listings[2] = Some(UserStoreListing0104 {
            list_slot: 2,
            inventory_slot: (state.mode == UserStoreMode0104::MyStore).then_some(3),
            item: items[1],
            price: 12_500,
        });
        authority.listings[4] = Some(UserStoreListing0104 {
            list_slot: 4,
            inventory_slot: (state.mode == UserStoreMode0104::MyStore).then_some(4),
            item: items[2],
            price: 125_000,
        });
    }
    if mode == UserStorePreviewMode0104::UserSold {
        authority.listings[2].as_mut().expect("seed listing").price = -1;
    }
    if mode == UserStorePreviewMode0104::Busy {
        state.pending = Some(UserStorePendingRequest0104::Register {
            list_slot: 1,
            inventory_slot: 1,
            item: items[1],
            price: 500,
        });
    }
    if mode == UserStorePreviewMode0104::Error {
        state.last_error = Some(13);
    }
}

pub fn seed_user_store_popup_preview_0104(
    mode: UserStorePreviewMode0104,
    state: &mut UserStoreUiState0104,
    authority: &UserStoreAuthority0104,
    presentation: &mut UserStorePopupPresentation0104,
) {
    let popup = match mode {
        UserStorePreviewMode0104::PopupQuantity => Some(UserStoreItemPopup0104 {
            kind: UserStorePopupKind0104::RegisterQuantity,
            slot_type: 1,
            slot: 1,
            item: authority.inventory[1],
            maximum_quantity: authority.inventory[1].option.max(0),
            price: 0,
        }),
        UserStorePreviewMode0104::PopupPrice => Some(UserStoreItemPopup0104 {
            kind: UserStorePopupKind0104::RegisterPrice,
            slot_type: 1,
            slot: 0,
            item: authority.inventory[0],
            maximum_quantity: 1,
            price: 0,
        }),
        UserStorePreviewMode0104::PopupUnregister => {
            authority.listings[0].map(|listing| UserStoreItemPopup0104 {
                kind: UserStorePopupKind0104::Unregister,
                slot_type: USER_STORE_MY_SLOT_TYPE,
                slot: listing.list_slot,
                item: listing.item,
                maximum_quantity: listing.item.option.max(0),
                price: listing.price,
            })
        }
        UserStorePreviewMode0104::PopupBuy => {
            authority.listings[0].map(|listing| UserStoreItemPopup0104 {
                kind: UserStorePopupKind0104::Buy,
                slot_type: USER_STORE_OTHER_SLOT_TYPE,
                slot: listing.list_slot,
                item: listing.item,
                maximum_quantity: listing.item.option.max(0),
                price: listing.price,
            })
        }
        UserStorePreviewMode0104::MyReady
        | UserStorePreviewMode0104::MyItems
        | UserStorePreviewMode0104::MyOpen
        | UserStorePreviewMode0104::UserList
        | UserStorePreviewMode0104::UserSold
        | UserStorePreviewMode0104::Busy
        | UserStorePreviewMode0104::Error => None,
    };
    if let Some(popup) = popup {
        presentation.open(popup);
        state.modal.item_popup = true;
    } else {
        presentation.close();
        state.modal.item_popup = false;
    }
}

pub(super) fn user_store_text_font(assets: &UserStoreUiAssets0104, font_size: f32) -> (TextFont, LineHeight) {
    (
        TextFont {
            font: (assets.font.clone()).into(),
            font_size: (font_size).into(),
            ..default()
        },
        LineHeight::Px(font_size + 2.0),
    )
}

pub(super) fn user_store_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn user_store_level_text(level: i32) -> LocalizedText {
    LocalizedText::new("ui.user_store.item.level", "LEVEL {level}")
        .with_arg("level", level.to_string())
}

pub(super) fn user_store_popup_cost_text(level: i32) -> LocalizedText {
    LocalizedText::new("ui.user_store.popup.cost_level", "COST {level}")
        .with_arg("level", level.to_string())
}

pub(super) fn user_store_popup_value_text(value: i32) -> LocalizedText {
    LocalizedText::new("ui.user_store.popup.value", "{value}").with_arg("value", value.to_string())
}

pub(super) fn user_store_popup_value_type_text(kind: UserStorePopupKind0104) -> LocalizedText {
    match kind {
        UserStorePopupKind0104::RegisterPrice => {
            LocalizedText::new("ui.user_store.popup.amount_taros", "Amount Taros")
        }
        UserStorePopupKind0104::RegisterQuantity
        | UserStorePopupKind0104::Unregister
        | UserStorePopupKind0104::Buy => LocalizedText::new("ui.user_store.popup.amount", "Amount"),
    }
}

pub(super) fn user_store_taros_text(amount: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.user_store.price_taros", "{amount} TAROS").with_arg("amount", amount)
}

pub(super) fn user_store_resolved_fallback(localized: &LocalizedText) -> String {
    let mut resolved = localized.fallback.clone();
    for (name, value) in &localized.args {
        resolved = resolved.replace(&format!("{{{name}}}"), value);
    }
    resolved
}

pub(super) fn adopt_user_store_popup_outcome_0104(
    presentation: &mut UserStorePopupPresentation0104,
    outcome: &UserStoreActionOutcome0104,
) {
    if let UserStoreActionOutcome0104::Popup(popup) = outcome {
        presentation.open(*popup);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_user_store_ui_0104(
    asset_server: Res<AssetServer>,
    assets: Res<UserStoreUiAssets0104>,
    mut asset_status: ResMut<UserStoreUiAssetStatus0104>,
    state: Res<UserStoreUiState0104>,
    authority: Res<UserStoreAuthority0104>,
    catalog: Res<UserStoreItemCatalog0104>,
    popup_presentation: Res<UserStorePopupPresentation0104>,
    mut projection: ResMut<UserStoreUiProjection0104>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(&mut Node, &mut Visibility), With<UserStoreUiRoot0104>>,
    mut ui_queries: ParamSet<(
        Query<
            (
                &UserStoreUiElement0104,
                &mut Node,
                Option<&mut ImageNode>,
                Option<&Interaction>,
            ),
            Without<UserStoreUiRoot0104>,
        >,
        Query<
            (
                &UserStoreUiTextRole0104,
                &mut LocalizedText,
                &mut Node,
                &mut TextColor,
            ),
            Without<UserStoreUiRoot0104>,
        >,
    )>,
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
    let layout = user_store_layout_0104(
        width,
        height,
        state.opening_elapsed_seconds,
        state.inventory_scroll_y,
    );
    *projection = project_user_store_ui_0104(&state, &authority, &catalog);
    let popup_item = popup_presentation
        .popup
        .map(|popup| project_user_store_item(popup.item, &catalog));
    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if state.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !state.active {
        return;
    }

    for (element, mut node, image, interaction) in &mut ui_queries.p0() {
        match *element {
            UserStoreUiElement0104::Backdrop => {
                bind_user_store_rect(&mut node, layout.full_backdrop)
            }
            UserStoreUiElement0104::StoreBackplate => {
                bind_user_store_rect(&mut node, layout.store_backplate)
            }
            UserStoreUiElement0104::RightBackplate => {
                bind_user_store_rect(&mut node, layout.right_backplate)
            }
            UserStoreUiElement0104::StorePanel => {
                bind_user_store_rect(&mut node, layout.store_panel)
            }
            UserStoreUiElement0104::PcStuffPanel => {
                bind_user_store_rect(&mut node, layout.pc_stuff_panel)
            }
            UserStoreUiElement0104::EquipmentPanel => {
                bind_user_store_rect(&mut node, layout.equipment_panel)
            }
            UserStoreUiElement0104::ListContent => {
                // `vecScrollPos` is deliberately not consumed: clean updates
                // it in `Scroll`, but `DoSlot` never reads it.
                node.left = px(0);
                node.top = px(0);
            }
            UserStoreUiElement0104::InventoryContent => {
                node.left = px(0);
                node.top = px(-layout.inventory_content.top + layout.inventory_viewport.top);
                node.width = px(layout.inventory_content.width);
                node.height = px(layout.inventory_content.height);
            }
            UserStoreUiElement0104::Row(visual_row) => {
                node.display = if projection.listing_rows.get(visual_row).is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            UserStoreUiElement0104::RowIconFrame(visual_row) => {
                let row = projection.listing_rows.get(visual_row);
                node.display = if row.and_then(|row| row.item.as_ref()).is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                if let (Some(mut image), Some(row)) = (image, row) {
                    image.image = assets.image(if row.affordable {
                        UserStoreAssetRole0104::SlotOccupied
                    } else {
                        UserStoreAssetRole0104::RestrictedFrame
                    });
                }
            }
            UserStoreUiElement0104::RowIcon(visual_row) => {
                if let Some(mut image) = image {
                    let icon = projection
                        .listing_rows
                        .get(visual_row)
                        .and_then(|row| row.item.as_ref())
                        .map(|item| &item.icon)
                        .unwrap_or(&UserStorePresentationIcon0104::Empty);
                    let (handle, visible) = user_store_icon_handle(icon, &asset_server, &assets);
                    node.display = if visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    image.image = handle;
                }
            }
            UserStoreUiElement0104::InventorySlotFrame(slot) => {
                if let Some(mut image) = image {
                    let occupied = projection
                        .inventory
                        .get(slot)
                        .is_some_and(|slot| !user_store_item_is_empty(slot.item.item));
                    image.image = assets.image(if occupied {
                        UserStoreAssetRole0104::SlotOccupied
                    } else {
                        UserStoreAssetRole0104::SlotEmpty
                    });
                }
            }
            UserStoreUiElement0104::InventorySlotIcon(slot) => {
                if let Some(mut image) = image {
                    let icon = projection
                        .inventory
                        .get(slot)
                        .map(|slot| &slot.item.icon)
                        .unwrap_or(&UserStorePresentationIcon0104::Empty);
                    let (handle, visible) = user_store_icon_handle(icon, &asset_server, &assets);
                    node.display = if visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    image.image = handle;
                }
            }
            UserStoreUiElement0104::EquipmentSlot(slot) => {
                if let Some(mut image) = image {
                    image.image = assets.image(
                        if authority
                            .equipment
                            .get(slot)
                            .copied()
                            .is_some_and(|item| !user_store_item_is_empty(item))
                        {
                            UserStoreAssetRole0104::SlotOccupied
                        } else {
                            UserStoreAssetRole0104::SlotEmpty
                        },
                    );
                }
            }
            UserStoreUiElement0104::Popup => {
                if let Some(popup) = popup_presentation.popup {
                    bind_user_store_rect(
                        &mut node,
                        user_store_popup_rect_0104(width, height, popup),
                    );
                } else {
                    node.display = Display::None;
                }
            }
            UserStoreUiElement0104::PopupIcon => {
                if let Some(mut image) = image {
                    let icon = popup_item
                        .as_ref()
                        .map(|item| &item.icon)
                        .unwrap_or(&UserStorePresentationIcon0104::Empty);
                    let (handle, visible) = user_store_icon_handle(icon, &asset_server, &assets);
                    node.display = if visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    image.image = handle;
                }
            }
            UserStoreUiElement0104::PopupClose => {
                if let Some(mut image) = image {
                    image.image = assets.image(if interaction == Some(&Interaction::Hovered) {
                        UserStoreAssetRole0104::PopupCloseHover
                    } else {
                        UserStoreAssetRole0104::Close
                    });
                }
            }
            UserStoreUiElement0104::PopupAction => {
                if let Some(popup) = popup_presentation.popup {
                    bind_user_store_rect(
                        &mut node,
                        if matches!(
                            popup.kind,
                            UserStorePopupKind0104::RegisterQuantity
                                | UserStorePopupKind0104::RegisterPrice
                        ) {
                            USER_STORE_POPUP_REGISTER_ACTION_RECT
                        } else {
                            USER_STORE_POPUP_LISTING_ACTION_RECT
                        },
                    );
                }
                if let Some(mut image) = image {
                    image.image = assets.image(if interaction == Some(&Interaction::Hovered) {
                        UserStoreAssetRole0104::ButtonHover
                    } else {
                        UserStoreAssetRole0104::ButtonNormal
                    });
                }
            }
            UserStoreUiElement0104::PrimaryButton => {
                if let Some(mut image) = image {
                    image.image = assets.image(if interaction == Some(&Interaction::Hovered) {
                        UserStoreAssetRole0104::ButtonHover
                    } else {
                        UserStoreAssetRole0104::ButtonNormal
                    });
                }
                node.display = if state.mode == UserStoreMode0104::ReturnStore {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            UserStoreUiElement0104::GoToGameButton => {
                node.display = if projection.show_go_to_game {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(mut image) = image {
                    image.image = assets.image(if interaction == Some(&Interaction::Hovered) {
                        UserStoreAssetRole0104::ButtonHover
                    } else {
                        UserStoreAssetRole0104::ButtonNormal
                    });
                }
            }
            UserStoreUiElement0104::Info
            | UserStoreUiElement0104::Title
            | UserStoreUiElement0104::Dialog
            | UserStoreUiElement0104::ListBack
            | UserStoreUiElement0104::ListDivider
            | UserStoreUiElement0104::ItemTab
            | UserStoreUiElement0104::ItemTabLabel
            | UserStoreUiElement0104::Table
            | UserStoreUiElement0104::ListViewport
            | UserStoreUiElement0104::RowCount(_)
            | UserStoreUiElement0104::RowName(_)
            | UserStoreUiElement0104::RowLevel(_)
            | UserStoreUiElement0104::RowPrice(_)
            | UserStoreUiElement0104::RowSellerNet(_)
            | UserStoreUiElement0104::TableShadow
            | UserStoreUiElement0104::ItemTabPcStuff
            | UserStoreUiElement0104::InventoryViewport
            | UserStoreUiElement0104::InventorySlotCount(_)
            | UserStoreUiElement0104::Close
            | UserStoreUiElement0104::Trash
            | UserStoreUiElement0104::Help
            | UserStoreUiElement0104::EquipmentTitle
            | UserStoreUiElement0104::PopupName
            | UserStoreUiElement0104::PopupCost
            | UserStoreUiElement0104::PopupDescription
            | UserStoreUiElement0104::PopupValueType
            | UserStoreUiElement0104::PopupCalculator
            | UserStoreUiElement0104::PopupValue
            | UserStoreUiElement0104::PopupDigit(_)
            | UserStoreUiElement0104::PopupClear
            | UserStoreUiElement0104::PopupNoOp
            | UserStoreUiElement0104::BusyLabel
            | UserStoreUiElement0104::ErrorLabel
            | UserStoreUiElement0104::Root => {}
        }
    }

    for (role, mut localized, mut node, mut color) in &mut ui_queries.p1() {
        match *role {
            UserStoreUiTextRole0104::Title => {
                *localized =
                    LocalizedText::new("ui.user_store.title", USER_STORE_TITLE_SUFFIX_TYPO);
            }
            UserStoreUiTextRole0104::ItemTab => {}
            UserStoreUiTextRole0104::RowCount(row) => {
                let value = projection
                    .listing_rows
                    .get(row)
                    .and_then(|row| row.item.as_ref())
                    .and_then(|item| item.count_label.as_deref())
                    .map(user_store_passthrough_text);
                bind_optional_user_store_text(&mut node, &mut localized, value);
            }
            UserStoreUiTextRole0104::RowName(row) => {
                let value = projection
                    .listing_rows
                    .get(row)
                    .and_then(|row| row.item.as_ref())
                    .map(|item| user_store_passthrough_text(&item.name));
                bind_optional_user_store_text(&mut node, &mut localized, value);
            }
            UserStoreUiTextRole0104::RowLevel(row) => {
                let value = projection
                    .listing_rows
                    .get(row)
                    .and_then(|row| row.item.as_ref())
                    .map(|item| user_store_level_text(item.level));
                bind_optional_user_store_text(&mut node, &mut localized, value);
            }
            UserStoreUiTextRole0104::RowPrice(row) => {
                let projected = projection.listing_rows.get(row);
                let value = projected
                    .and_then(|row| row.price)
                    .map(|price| user_store_taros_text(user_store_format_number(price)));
                bind_optional_user_store_text(&mut node, &mut localized, value);
                color.0 = projected.map_or(Color::WHITE, user_store_price_color);
            }
            UserStoreUiTextRole0104::RowSellerNet(row) => {
                let value = projection
                    .listing_rows
                    .get(row)
                    .and_then(|row| row.seller_display_taros)
                    .map(|price| user_store_taros_text(user_store_format_number(price)));
                bind_optional_user_store_text(&mut node, &mut localized, value);
            }
            UserStoreUiTextRole0104::PrimaryButton => {
                *localized = user_store_primary_button_text(&projection.primary_button);
            }
            UserStoreUiTextRole0104::GoToGameButton => {
                *localized = LocalizedText::new("ui.user_store.go_to_game", "GO TO GAME");
            }
            UserStoreUiTextRole0104::InventoryCount(slot) => {
                let value = projection
                    .inventory
                    .get(slot)
                    .and_then(|slot| slot.item.count_label.as_deref())
                    .map(user_store_passthrough_text);
                bind_optional_user_store_text(&mut node, &mut localized, value);
            }
            UserStoreUiTextRole0104::PopupName => {
                *localized = user_store_passthrough_text(
                    popup_item.as_ref().map_or("", |item| item.name.as_str()),
                );
            }
            UserStoreUiTextRole0104::PopupCost => {
                *localized =
                    user_store_popup_cost_text(popup_item.as_ref().map_or(0, |item| item.level));
            }
            UserStoreUiTextRole0104::PopupDescription => {
                *localized = user_store_passthrough_text(
                    popup_item
                        .as_ref()
                        .map_or("", |item| item.description.as_str()),
                );
            }
            UserStoreUiTextRole0104::PopupValueType => {
                if let Some(popup) = popup_presentation.popup {
                    *localized = user_store_popup_value_type_text(popup.kind);
                }
            }
            UserStoreUiTextRole0104::PopupValue => {
                *localized = user_store_popup_value_text(popup_presentation.value);
            }
            UserStoreUiTextRole0104::PopupAction => {
                if let Some(popup) = popup_presentation.popup {
                    *localized = user_store_popup_action_text(popup.kind);
                }
            }
            UserStoreUiTextRole0104::PopupDigit(_) | UserStoreUiTextRole0104::PopupClear => {}
            UserStoreUiTextRole0104::Busy => {
                node.display = if projection.busy {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            UserStoreUiTextRole0104::Error => {
                if let Some(code) = projection.error {
                    node.display = Display::Flex;
                    *localized = user_store_error_text(code);
                } else {
                    node.display = Display::None;
                    *localized = user_store_passthrough_text("");
                }
            }
        }
    }
}

pub(super) fn bind_optional_user_store_text(
    node: &mut Node,
    text: &mut LocalizedText,
    value: Option<LocalizedText>,
) {
    if let Some(value) = value {
        node.display = Display::Flex;
        *text = value;
    } else {
        node.display = Display::None;
        *text = user_store_passthrough_text("");
    }
}

pub(super) fn user_store_price_color(row: &UserStoreListingRowProjection0104) -> Color {
    let Some(price) = row.price else {
        return Color::WHITE;
    };
    if row.sold {
        Color::srgb(0.4, 0.4, 0.4)
    } else if !row.affordable {
        Color::srgb(1.0, 0.0, 0.0)
    } else if price < 1_000 {
        Color::srgb(0.4, 0.4, 0.4)
    } else if price < 10_000 {
        Color::WHITE
    } else if price < 100_000 {
        Color::srgb(0.2, 1.0, 0.0)
    } else if price >= 1_000_000 {
        Color::srgb(1.0, 0.0, 1.0)
    } else {
        Color::srgb(1.0, 1.0, 0.0)
    }
}

#[must_use]
pub fn user_store_format_number(value: i32) -> String {
    let negative = value < 0;
    let digits = i64::from(value).unsigned_abs().to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3 + usize::from(negative));
    if negative {
        output.push('-');
    }
    let first = digits.len() % 3;
    if first != 0 {
        output.push_str(&digits[..first]);
    }
    for chunk_start in (first..digits.len()).step_by(3) {
        if !output.is_empty() && !(negative && output.len() == 1) {
            output.push(',');
        }
        output.push_str(&digits[chunk_start..chunk_start + 3]);
    }
    output
}

pub(super) fn user_store_icon_handle(
    icon: &UserStorePresentationIcon0104,
    asset_server: &AssetServer,
    assets: &UserStoreUiAssets0104,
) -> (Handle<Image>, bool) {
    match icon {
        UserStorePresentationIcon0104::Empty => (Handle::default(), false),
        UserStorePresentationIcon0104::MissingChecker => (assets.missing_checker.clone(), true),
        UserStorePresentationIcon0104::Resolved(path) => {
            let handle = asset_server.load::<Image>(path.clone());
            if matches!(asset_server.load_state(handle.id()), LoadState::Loaded) {
                (handle, true)
            } else {
                (assets.missing_checker.clone(), true)
            }
        }
    }
}
