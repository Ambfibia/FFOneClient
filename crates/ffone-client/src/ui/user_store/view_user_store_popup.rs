use super::*;

pub(super) fn spawn_user_store_ui_0104(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    let assets = UserStoreUiAssets0104::load(&asset_server, &mut image_assets);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            UserStoreUiRoot0104,
            UserStoreUiElement0104::Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(USER_STORE_UI_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            spawn_user_store_image(
                root,
                UserStoreUiElement0104::Backdrop,
                UserStoreUiRect::default(),
                assets.image(UserStoreAssetRole0104::Backdrop),
                None,
            );
            spawn_user_store_image(
                root,
                UserStoreUiElement0104::StoreBackplate,
                UserStoreUiRect::default(),
                assets.image(UserStoreAssetRole0104::StoreBackplate),
                None,
            );
            spawn_user_store_image(
                root,
                UserStoreUiElement0104::RightBackplate,
                UserStoreUiRect::default(),
                assets.image(UserStoreAssetRole0104::RightBackplate),
                Some(USER_STORE_RIGHT_PANEL_BORDER),
            );
            spawn_user_store_panel_0104(root, &assets);
            spawn_user_store_pc_stuff_0104(root, &assets);
            spawn_user_store_equipment_0104(root, &assets);
            spawn_user_store_popup_0104(root, &assets);
            root.spawn((
                UserStoreUiElement0104::BusyLabel,
                UserStoreUiTextRole0104::Busy,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                Text::new("BUSY"),
                LocalizedText::new("ui.user_store.busy", "BUSY"),
                user_store_text_font(&assets, 28.0),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));
            root.spawn((
                UserStoreUiElement0104::ErrorLabel,
                UserStoreUiTextRole0104::Error,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    left: px(14),
                    bottom: px(12),
                    width: px(420),
                    height: px(28),
                    ..default()
                },
                Text::new(""),
                user_store_passthrough_text(""),
                user_store_text_font(&assets, 18.0),
                TextColor(Color::srgb(1.0, 0.25, 0.2)),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_user_store_panel_0104(parent: &mut ChildSpawnerCommands, assets: &UserStoreUiAssets0104) {
    parent
        .spawn((
            UserStoreUiElement0104::StorePanel,
            UserStoreUiRect::default().node(),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            spawn_user_store_image(
                panel,
                UserStoreUiElement0104::Info,
                USER_STORE_INFO_RECT,
                assets.image(UserStoreAssetRole0104::Info),
                None,
            );
            spawn_user_store_text(
                panel,
                UserStoreUiElement0104::Title,
                UserStoreUiTextRole0104::Title,
                USER_STORE_TITLE_RECT,
                LocalizedText::new("ui.user_store.title", USER_STORE_TITLE_SUFFIX_TYPO),
                assets,
                13.0,
                Color::WHITE,
                Justify::Left,
            );
            panel
                .spawn((
                    UserStoreUiElement0104::Dialog,
                    USER_STORE_DIALOG_RECT.node(),
                    Pickable::IGNORE,
                ))
                .with_children(|dialog| {
                    spawn_user_store_image(
                        dialog,
                        UserStoreUiElement0104::ListBack,
                        USER_STORE_LIST_BACK_RECT,
                        assets.image(UserStoreAssetRole0104::ListBack),
                        Some(USER_STORE_LIST_BACK_BORDER),
                    );
                    spawn_user_store_image(
                        dialog,
                        UserStoreUiElement0104::ListDivider,
                        USER_STORE_LIST_DIVIDER_RECT,
                        assets.image(UserStoreAssetRole0104::ListDivider),
                        None,
                    );
                    spawn_user_store_image(
                        dialog,
                        UserStoreUiElement0104::ItemTab,
                        USER_STORE_ITEM_TAB_RECT,
                        assets.image(UserStoreAssetRole0104::ItemTab),
                        None,
                    );
                    spawn_user_store_text(
                        dialog,
                        UserStoreUiElement0104::ItemTabLabel,
                        UserStoreUiTextRole0104::ItemTab,
                        USER_STORE_ITEM_TAB_HIT_RECT,
                        LocalizedText::new("ui.user_store.tab.item", "Item"),
                        assets,
                        12.0,
                        Color::srgb(0.8, 1.0, 1.0),
                        Justify::Center,
                    );
                    dialog
                        .spawn((
                            UserStoreUiElement0104::Table,
                            USER_STORE_TABLE_RECT.node(),
                            Pickable::IGNORE,
                        ))
                        .with_children(|table| {
                            table
                                .spawn((
                                    UserStoreUiElement0104::ListViewport,
                                    Node {
                                        overflow: Overflow::clip(),
                                        ..USER_STORE_LIST_VIEWPORT_RECT.node()
                                    },
                                    Pickable::IGNORE,
                                ))
                                .with_children(|viewport| {
                                    viewport
                                        .spawn((
                                            UserStoreUiElement0104::ListContent,
                                            UserStoreUiRect::new(0.0, 0.0, 433.0, 400.0).node(),
                                            Pickable::IGNORE,
                                        ))
                                        .with_children(|content| {
                                            for visual_row in 0..USER_STORE_LIST_CAPACITY {
                                                spawn_user_store_row_0104(
                                                    content, visual_row, assets,
                                                );
                                            }
                                        });
                                });
                            spawn_user_store_image(
                                table,
                                UserStoreUiElement0104::TableShadow,
                                USER_STORE_TABLE_SHADOW_RECT,
                                assets.image(UserStoreAssetRole0104::Shadow),
                                None,
                            );
                        });
                });
            spawn_user_store_button_0104(
                panel,
                UserStoreUiElement0104::PrimaryButton,
                UserStoreInteractiveControl0104::PrimaryButton,
                UserStoreUiTextRole0104::PrimaryButton,
                USER_STORE_PRIMARY_BUTTON_RECT,
                LocalizedText::new("ui.user_store.open_store", "OPEN STORE"),
                assets,
            );
            spawn_user_store_button_0104(
                panel,
                UserStoreUiElement0104::GoToGameButton,
                UserStoreInteractiveControl0104::GoToGame,
                UserStoreUiTextRole0104::GoToGameButton,
                USER_STORE_GO_TO_GAME_RECT,
                LocalizedText::new("ui.user_store.go_to_game", "GO TO GAME"),
                assets,
            );
        });
}

pub(super) fn spawn_user_store_row_0104(
    parent: &mut ChildSpawnerCommands,
    visual_row: usize,
    assets: &UserStoreUiAssets0104,
) {
    parent
        .spawn((
            Button,
            UserStoreUiElement0104::Row(visual_row),
            UserStoreInteractiveControl0104::Row(visual_row),
            UserStoreUiRect::new(
                0.0,
                visual_row as f32 * USER_STORE_ROW_HEIGHT,
                USER_STORE_ROW_WIDTH,
                USER_STORE_ROW_VISUAL_HEIGHT,
            )
            .node(),
            stretched_image(assets.image(UserStoreAssetRole0104::ItemRow)),
        ))
        .with_children(|row| {
            spawn_user_store_image(
                row,
                UserStoreUiElement0104::RowIconFrame(visual_row),
                USER_STORE_ROW_ITEM_RECT,
                assets.image(UserStoreAssetRole0104::SlotOccupied),
                None,
            );
            row.spawn((
                UserStoreUiElement0104::RowIcon(visual_row),
                USER_STORE_ROW_ITEM_RECT.node(),
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            spawn_user_store_text(
                row,
                UserStoreUiElement0104::RowCount(visual_row),
                UserStoreUiTextRole0104::RowCount(visual_row),
                UserStoreUiRect::new(9.0, 9.0, 55.0, 55.0),
                user_store_passthrough_text(""),
                assets,
                14.0,
                Color::WHITE,
                Justify::Left,
            );
            spawn_user_store_text(
                row,
                UserStoreUiElement0104::RowName(visual_row),
                UserStoreUiTextRole0104::RowName(visual_row),
                USER_STORE_ROW_NAME_RECT,
                user_store_passthrough_text(""),
                assets,
                13.0,
                Color::WHITE,
                Justify::Left,
            );
            spawn_user_store_text(
                row,
                UserStoreUiElement0104::RowLevel(visual_row),
                UserStoreUiTextRole0104::RowLevel(visual_row),
                USER_STORE_ROW_LEVEL_RECT,
                user_store_level_text(0),
                assets,
                12.0,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Left,
            );
            spawn_user_store_text(
                row,
                UserStoreUiElement0104::RowPrice(visual_row),
                UserStoreUiTextRole0104::RowPrice(visual_row),
                USER_STORE_ROW_PRICE_RECT,
                user_store_taros_text(""),
                assets,
                12.0,
                Color::WHITE,
                Justify::Right,
            );
            spawn_user_store_text(
                row,
                UserStoreUiElement0104::RowSellerNet(visual_row),
                UserStoreUiTextRole0104::RowSellerNet(visual_row),
                USER_STORE_ROW_SELLER_NET_RECT,
                user_store_taros_text(""),
                assets,
                12.0,
                Color::WHITE,
                Justify::Right,
            );
        });
}

pub(super) fn spawn_user_store_pc_stuff_0104(
    parent: &mut ChildSpawnerCommands,
    assets: &UserStoreUiAssets0104,
) {
    parent
        .spawn((
            UserStoreUiElement0104::PcStuffPanel,
            USER_STORE_PC_STUFF_RECT.node(),
            sliced_image(
                assets.image(UserStoreAssetRole0104::InventoryPanel),
                USER_STORE_INVENTORY_PANEL_BORDER,
            ),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            spawn_user_store_text(
                panel,
                UserStoreUiElement0104::ItemTabPcStuff,
                UserStoreUiTextRole0104::ItemTab,
                UserStoreUiRect::new(12.0, 7.0, 130.0, 22.0),
                LocalizedText::new("ui.user_store.inventory.item", "ITEM"),
                assets,
                13.0,
                Color::srgb(0.8, 1.0, 1.0),
                Justify::Center,
            );
            panel
                .spawn((
                    UserStoreUiElement0104::InventoryViewport,
                    Node {
                        overflow: Overflow::clip(),
                        ..USER_STORE_INVENTORY_VIEWPORT_RECT.node()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            UserStoreUiElement0104::InventoryContent,
                            UserStoreUiRect::new(
                                0.0,
                                0.0,
                                366.0,
                                USER_STORE_INVENTORY_CONTENT_HEIGHT,
                            )
                            .node(),
                            Pickable::IGNORE,
                        ))
                        .with_children(|content| {
                            for slot in 0..USER_STORE_INVENTORY_CAPACITY {
                                spawn_user_store_inventory_slot_0104(content, slot, assets);
                            }
                        });
                });
            spawn_user_store_control_image(
                panel,
                UserStoreUiElement0104::Close,
                UserStoreInteractiveControl0104::Close,
                USER_STORE_CLOSE_RECT,
                assets.image(UserStoreAssetRole0104::Close),
            );
            spawn_user_store_image(
                panel,
                UserStoreUiElement0104::Trash,
                USER_STORE_TRASH_RECT,
                assets.image(UserStoreAssetRole0104::Trash),
                None,
            );
            spawn_user_store_control_image(
                panel,
                UserStoreUiElement0104::Help,
                UserStoreInteractiveControl0104::Help,
                USER_STORE_HELP_RECT,
                assets.image(UserStoreAssetRole0104::Help),
            );
        });
}

pub(super) fn spawn_user_store_inventory_slot_0104(
    parent: &mut ChildSpawnerCommands,
    slot: usize,
    assets: &UserStoreUiAssets0104,
) {
    let column = slot % USER_STORE_INVENTORY_COLUMNS;
    let row = slot / USER_STORE_INVENTORY_COLUMNS;
    let rect = UserStoreUiRect::new(
        column as f32 * USER_STORE_INVENTORY_SLOT_STRIDE,
        row as f32 * USER_STORE_INVENTORY_SLOT_STRIDE,
        USER_STORE_INVENTORY_SLOT_SIZE,
        USER_STORE_INVENTORY_SLOT_SIZE,
    );
    parent
        .spawn((
            Button,
            UserStoreUiElement0104::InventorySlotFrame(slot),
            UserStoreInteractiveControl0104::InventorySlot(slot),
            rect.node(),
            stretched_image(assets.image(UserStoreAssetRole0104::SlotEmpty)),
        ))
        .with_children(|frame| {
            frame.spawn((
                UserStoreUiElement0104::InventorySlotIcon(slot),
                UserStoreUiRect::new(
                    0.0,
                    0.0,
                    USER_STORE_INVENTORY_SLOT_SIZE,
                    USER_STORE_INVENTORY_SLOT_SIZE,
                )
                .node(),
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            spawn_user_store_text(
                frame,
                UserStoreUiElement0104::InventorySlotCount(slot),
                UserStoreUiTextRole0104::InventoryCount(slot),
                UserStoreUiRect::new(5.0, 5.0, 57.0, 57.0),
                user_store_passthrough_text(""),
                assets,
                14.0,
                Color::WHITE,
                Justify::Left,
            );
        });
}

pub(super) fn spawn_user_store_equipment_0104(
    parent: &mut ChildSpawnerCommands,
    assets: &UserStoreUiAssets0104,
) {
    parent
        .spawn((
            UserStoreUiElement0104::EquipmentPanel,
            USER_STORE_EQUIP_PANEL_RECT.node(),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            spawn_user_store_image(
                panel,
                UserStoreUiElement0104::EquipmentTitle,
                UserStoreUiRect::new(0.0, 3.0, 64.0, 13.0),
                assets.image(UserStoreAssetRole0104::EquipTitle),
                None,
            );
            for slot in 0..USER_STORE_EQUIPMENT_CAPACITY {
                spawn_user_store_image(
                    panel,
                    UserStoreUiElement0104::EquipmentSlot(slot),
                    UserStoreUiRect::new(
                        0.0,
                        14.0 + slot as f32 * USER_STORE_INVENTORY_SLOT_STRIDE,
                        USER_STORE_INVENTORY_SLOT_SIZE,
                        USER_STORE_INVENTORY_SLOT_SIZE,
                    ),
                    assets.image(UserStoreAssetRole0104::SlotEmpty),
                    None,
                );
            }
        });
}

pub(super) fn spawn_user_store_popup_0104(parent: &mut ChildSpawnerCommands, assets: &UserStoreUiAssets0104) {
    parent
        .spawn((
            UserStoreUiElement0104::Popup,
            Node {
                display: Display::None,
                ..USER_STORE_POPUP_RECT.node()
            },
            stretched_image(assets.image(UserStoreAssetRole0104::PopupBackground)),
            ZIndex(USER_STORE_POPUP_LOCAL_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|popup| {
            popup.spawn((
                UserStoreUiElement0104::PopupIcon,
                USER_STORE_POPUP_ICON_RECT.node(),
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            spawn_user_store_popup_text_0104(
                popup,
                UserStoreUiElement0104::PopupName,
                UserStoreUiTextRole0104::PopupName,
                USER_STORE_POPUP_NAME_RECT,
                user_store_passthrough_text(""),
                assets,
                UserStorePopupTextStyle0104::LabelUpperLeft,
                Color::srgb(1.0, 1.0, 0.0),
            );
            spawn_user_store_popup_text_0104(
                popup,
                UserStoreUiElement0104::PopupCost,
                UserStoreUiTextRole0104::PopupCost,
                USER_STORE_POPUP_COST_RECT,
                user_store_popup_cost_text(0),
                assets,
                UserStorePopupTextStyle0104::LabelUpperLeft,
                Color::srgb(1.0, 1.0, 0.0),
            );
            spawn_user_store_popup_text_0104(
                popup,
                UserStoreUiElement0104::PopupDescription,
                UserStoreUiTextRole0104::PopupDescription,
                USER_STORE_POPUP_DESCRIPTION_RECT,
                user_store_passthrough_text(""),
                assets,
                UserStorePopupTextStyle0104::LabelUpperLeft,
                Color::WHITE,
            );
            popup.spawn((
                Button,
                UserStoreUiElement0104::PopupClose,
                UserStoreInteractiveControl0104::PopupClose,
                USER_STORE_POPUP_CLOSE_RECT.node(),
                stretched_image(assets.image(UserStoreAssetRole0104::Close)),
            ));
            spawn_user_store_popup_text_0104(
                popup,
                UserStoreUiElement0104::PopupValueType,
                UserStoreUiTextRole0104::PopupValueType,
                USER_STORE_POPUP_VALUE_TYPE_RECT,
                LocalizedText::new("ui.user_store.popup.amount", "Amount"),
                assets,
                UserStorePopupTextStyle0104::CenterLabel,
                Color::WHITE,
            );
            popup
                .spawn((
                    UserStoreUiElement0104::PopupCalculator,
                    USER_STORE_POPUP_CALCULATOR_RECT.node(),
                    stretched_image(assets.image(UserStoreAssetRole0104::PopupCalculator)),
                    Pickable::IGNORE,
                ))
                .with_children(|calculator| {
                    spawn_user_store_popup_text_0104(
                        calculator,
                        UserStoreUiElement0104::PopupValue,
                        UserStoreUiTextRole0104::PopupValue,
                        USER_STORE_POPUP_VALUE_RECT,
                        user_store_popup_value_text(0),
                        assets,
                        UserStorePopupTextStyle0104::LabelMiddleRight,
                        Color::srgb(1.0, 1.0, 0.0),
                    );
                    for (digit, rect) in USER_STORE_POPUP_BUTTON_RECTS {
                        spawn_user_store_popup_key_0104(calculator, digit, rect, assets);
                    }
                    spawn_user_store_popup_key_text_0104(
                        calculator,
                        UserStoreUiElement0104::PopupClear,
                        UserStoreInteractiveControl0104::PopupClear,
                        UserStoreUiTextRole0104::PopupClear,
                        USER_STORE_POPUP_CLEAR_RECT,
                        LocalizedText::new("ui.user_store.popup.clear", "C"),
                        assets,
                    );
                    calculator.spawn((
                        Button,
                        UserStoreUiElement0104::PopupNoOp,
                        USER_STORE_POPUP_NO_OP_RECT.node(),
                    ));
                });
            let mut action_node = USER_STORE_POPUP_REGISTER_ACTION_RECT.node();
            UserStorePopupTextStyle0104::Button.apply_to_container(&mut action_node);
            popup
                .spawn((
                    Button,
                    UserStoreUiElement0104::PopupAction,
                    UserStoreInteractiveControl0104::PopupAction,
                    action_node,
                    sliced_image(
                        assets.image(UserStoreAssetRole0104::ButtonNormal),
                        USER_STORE_POPUP_BUTTON_BORDER,
                    ),
                ))
                .with_children(|button| {
                    spawn_user_store_popup_text_entity_0104(
                        button,
                        UserStoreUiTextRole0104::PopupAction,
                        LocalizedText::new("ui.user_store.popup.add", "ADD"),
                        assets,
                        UserStorePopupTextStyle0104::Button,
                        Color::WHITE,
                        USER_STORE_POPUP_REGISTER_ACTION_RECT,
                    );
                });
        });
}

pub(super) fn spawn_user_store_popup_key_0104(
    parent: &mut ChildSpawnerCommands,
    digit: u8,
    rect: UserStoreUiRect,
    assets: &UserStoreUiAssets0104,
) {
    spawn_user_store_popup_key_text_0104(
        parent,
        UserStoreUiElement0104::PopupDigit(digit),
        UserStoreInteractiveControl0104::PopupDigit(digit),
        UserStoreUiTextRole0104::PopupDigit(digit),
        rect,
        LocalizedText::new("ui.user_store.popup.digit", "{digit}")
            .with_arg("digit", digit.to_string()),
        assets,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_user_store_popup_key_text_0104(
    parent: &mut ChildSpawnerCommands,
    element: UserStoreUiElement0104,
    control: UserStoreInteractiveControl0104,
    role: UserStoreUiTextRole0104,
    rect: UserStoreUiRect,
    localized: LocalizedText,
    assets: &UserStoreUiAssets0104,
) {
    let mut node = rect.node();
    UserStorePopupTextStyle0104::CalculatorButton.apply_to_container(&mut node);
    parent
        .spawn((Button, element, control, node))
        .with_children(|button| {
            spawn_user_store_popup_text_entity_0104(
                button,
                role,
                localized,
                assets,
                UserStorePopupTextStyle0104::CalculatorButton,
                Color::WHITE,
                rect,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_user_store_popup_text_0104(
    parent: &mut ChildSpawnerCommands,
    element: UserStoreUiElement0104,
    role: UserStoreUiTextRole0104,
    rect: UserStoreUiRect,
    localized: LocalizedText,
    assets: &UserStoreUiAssets0104,
    style: UserStorePopupTextStyle0104,
    color: Color,
) {
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent
        .spawn((element, node, Pickable::IGNORE))
        .with_children(|text| {
            spawn_user_store_popup_text_entity_0104(
                text, role, localized, assets, style, color, rect,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_user_store_popup_text_entity_0104(
    parent: &mut ChildSpawnerCommands,
    role: UserStoreUiTextRole0104,
    localized: LocalizedText,
    assets: &UserStoreUiAssets0104,
    style: UserStorePopupTextStyle0104,
    color: Color,
    source_rect: UserStoreUiRect,
) {
    let fallback = user_store_resolved_fallback(&localized);
    let font = style.font(assets);
    let bounds = style.content_bounds(source_rect);
    parent.spawn((
        role,
        style,
        Node {
            width: percent(100),
            ..default()
        },
        Text::new(fallback),
        localized,
        font.clone(),
        TextColor(color),
        style.layout(),
        UiTransform::default(),
        UiTextAutoFit::new(bounds.x, bounds.y, &font),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_user_store_button_0104(
    parent: &mut ChildSpawnerCommands,
    element: UserStoreUiElement0104,
    control: UserStoreInteractiveControl0104,
    text_role: UserStoreUiTextRole0104,
    rect: UserStoreUiRect,
    localized: LocalizedText,
    assets: &UserStoreUiAssets0104,
) {
    let fallback = user_store_resolved_fallback(&localized);
    parent
        .spawn((
            Button,
            element,
            control,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..rect.node()
            },
            sliced_image(
                assets.image(UserStoreAssetRole0104::ButtonNormal),
                USER_STORE_BUTTON_BORDER,
            ),
        ))
        .with_children(|button| {
            button.spawn((
                text_role,
                Text::new(fallback),
                localized,
                user_store_text_font(assets, 13.0),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_user_store_control_image(
    parent: &mut ChildSpawnerCommands,
    element: UserStoreUiElement0104,
    control: UserStoreInteractiveControl0104,
    rect: UserStoreUiRect,
    image: Handle<Image>,
) {
    parent.spawn((
        Button,
        element,
        control,
        rect.node(),
        stretched_image(image),
    ));
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_user_store_text(
    parent: &mut ChildSpawnerCommands,
    element: UserStoreUiElement0104,
    role: UserStoreUiTextRole0104,
    rect: UserStoreUiRect,
    localized: LocalizedText,
    assets: &UserStoreUiAssets0104,
    size: f32,
    color: Color,
    justify: Justify,
) {
    let fallback = user_store_resolved_fallback(&localized);
    parent.spawn((
        element,
        role,
        Node {
            align_items: AlignItems::Center,
            ..rect.node()
        },
        Text::new(fallback),
        localized,
        user_store_text_font(assets, size),
        TextColor(color),
        TextLayout::new(justify, LineBreak::NoWrap),
        Pickable::IGNORE,
    ));
}
