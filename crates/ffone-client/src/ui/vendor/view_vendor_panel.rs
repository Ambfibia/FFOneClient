use super::*;

pub(super) fn spawn_vendor_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_assets: ResMut<Assets<Image>>,
    contract: Res<VendorUiAssetContract>,
) {
    let assets = VendorUiAssets::load(&asset_server, &mut image_assets, &contract);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            VendorUiRoot,
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
            GlobalZIndex(VENDOR_UI_Z_INDEX),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                VendorUiElement::Backdrop,
                VendorUiRect::default().node(),
                stretched_image(assets.image(VendorStaticAssetRole::Backdrop)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                VendorUiElement::VendorBackplate,
                VendorUiRect::default().node(),
                stretched_image(assets.image(VendorStaticAssetRole::VendorBackplate)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                VendorUiElement::RightBackplate,
                VendorUiRect::default().node(),
                sliced_image(
                    assets.image(VendorStaticAssetRole::RightBackplate),
                    USER_EQUIP_RIGHT_PANEL_BORDER,
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_vendor_panel(root, &assets);
            spawn_vendor_pc_stuff_panel(root, &assets);
            spawn_vendor_equipment_panel(root, &assets);
        });
}

pub(super) fn spawn_vendor_panel(parent: &mut ChildSpawnerCommands, assets: &VendorUiAssets) {
    parent
        .spawn((
            VendorUiElement::VendorPanel,
            VendorUiRect::default().node(),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            panel.spawn((
                VendorUiElement::VendorInfo,
                VENDOR_INFO_RECT.node(),
                stretched_image(assets.image(VendorStaticAssetRole::Info)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_vendor_text(
                panel,
                VendorUiElement::VendorTitle,
                VENDOR_TITLE_RECT,
                "",
                vendor_title_localized(""),
                assets.font.clone(),
                VendorUiTextStyle::LabelUpperLeft,
                Color::WHITE,
            );
            spawn_vendor_text(
                panel,
                VendorUiElement::VendorService,
                VENDOR_SERVICE_RECT,
                "",
                vendor_service_localized(""),
                assets.service_font.clone(),
                VendorUiTextStyle::LabelUpperLeftService,
                Color::srgb(1.0, 1.0, 0.0),
            );
            // `AvatarUtil.DrawCamera(ownposition, cNPC)` is an external render
            // surface, not a semantic bitmap. Preserve its exact 200x150
            // boundary but keep it transparent until a real native NPC camera
            // owner is bound; never invent a portrait or diagnostic copy.
            panel.spawn((
                VendorUiElement::NpcPreviewBoundary,
                VENDOR_NPC_PREVIEW_RECT.node(),
                crate::service_portrait::ServicePortraitSlot::Vendor,
                ImageNode {
                    color: Color::NONE,
                    ..default()
                },
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            panel
                .spawn((
                    VendorUiElement::VendorDialog,
                    VENDOR_DIALOG_RECT.node(),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|dialog| {
                    dialog.spawn((
                        VendorUiElement::ListBack,
                        VENDOR_LIST_BACK_RECT.node(),
                        sliced_image(
                            assets.image(VendorStaticAssetRole::ListBack),
                            VENDOR_LIST_BACK_BORDER,
                        ),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                    dialog.spawn((
                        VendorUiElement::BuyTab,
                        VENDOR_BUY_TAB_RECT.node(),
                        stretched_image(assets.image(VendorStaticAssetRole::BuyNormal)),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                    dialog.spawn((
                        VendorUiElement::BuybackTab,
                        VENDOR_BUYBACK_TAB_RECT.node(),
                        stretched_image(assets.image(VendorStaticAssetRole::BuybackNormal)),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                    dialog.spawn((
                        VendorUiElement::ListDivider,
                        VENDOR_LIST_DIVIDER_RECT.node(),
                        stretched_image(assets.image(VendorStaticAssetRole::ListDivider)),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                    dialog.spawn((
                        VendorUiElement::BuyTabSelected,
                        VENDOR_BUY_TAB_RECT.node(),
                        stretched_image(assets.image(VendorStaticAssetRole::BuySelected)),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                    dialog.spawn((
                        VendorUiElement::BuybackTabSelected,
                        Node {
                            display: Display::None,
                            ..VENDOR_BUYBACK_TAB_RECT.node()
                        },
                        stretched_image(assets.image(VendorStaticAssetRole::BuybackSelected)),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                    spawn_vendor_tab_button(
                        dialog,
                        VendorUiElement::BuyTabLabel,
                        VendorInteractiveControl::BuyTab,
                        VENDOR_BUY_TAB_HIT_RECT,
                        "BUY",
                        assets.font.clone(),
                    );
                    spawn_vendor_tab_button(
                        dialog,
                        VendorUiElement::BuybackTabLabel,
                        VendorInteractiveControl::BuybackTab,
                        VENDOR_BUYBACK_TAB_HIT_RECT,
                        "BUY BACK",
                        assets.font.clone(),
                    );
                    dialog
                        .spawn((
                            VendorUiElement::Table,
                            VENDOR_TABLE_RECT.node(),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ))
                        .with_children(|table| {
                            table
                                .spawn((
                                    VendorUiElement::ListViewport,
                                    Node {
                                        overflow: Overflow::clip(),
                                        ..VENDOR_LIST_VIEWPORT_RECT.node()
                                    },
                                    Pickable::IGNORE,
                                    bevy::ui::FocusPolicy::Pass,
                                ))
                                .with_children(|viewport| {
                                    viewport
                                        .spawn((
                                            VendorUiElement::ListContent,
                                            VendorUiRect::new(
                                                0.0,
                                                0.0,
                                                VENDOR_ROW_WIDTH,
                                                VENDOR_CATALOG_CAPACITY as f32 * VENDOR_ROW_HEIGHT,
                                            )
                                            .node(),
                                            Pickable::IGNORE,
                                            bevy::ui::FocusPolicy::Pass,
                                        ))
                                        .with_children(|content| {
                                            for row in 0..VENDOR_CATALOG_CAPACITY {
                                                spawn_vendor_row(content, row, assets);
                                            }
                                        });
                                });
                            table.spawn((
                                VendorUiElement::TableShadow,
                                VENDOR_TABLE_SHADOW_RECT.node(),
                                ImageNode {
                                    color: Color::BLACK,
                                    ..sliced_image(
                                        assets.image(VendorStaticAssetRole::ScrollShadow),
                                        VENDOR_SCROLL_SHADOW_BORDER,
                                    )
                                },
                                Pickable::IGNORE,
                                bevy::ui::FocusPolicy::Pass,
                            ));
                        });
                    dialog.spawn((
                        VendorUiElement::ScrollTrack,
                        VENDOR_SCROLL_TRACK_RECT.node(),
                        sliced_image(
                            assets.image(VendorStaticAssetRole::ScrollTrack),
                            VENDOR_SCROLL_TRACK_BORDER,
                        ),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Vendor,
                            crate::service_scroll::Part::Track,
                        ),
                    ));
                    dialog.spawn((
                        VendorUiElement::ScrollThumb,
                        VendorUiRect::new(
                            VENDOR_SCROLL_TRACK_RECT.left + 2.0,
                            VENDOR_SCROLL_TRACK_RECT.top,
                            13.0,
                            15.0,
                        )
                        .node(),
                        sliced_image(
                            assets.image(VendorStaticAssetRole::ScrollThumb),
                            VENDOR_SCROLL_TRACK_BORDER,
                        ),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Vendor,
                            crate::service_scroll::Part::Thumb,
                        ),
                    ));
                    dialog.spawn((
                        VendorUiElement::ScrollUp,
                        VENDOR_SCROLL_UP_RECT.node(),
                        stretched_image(assets.image(VendorStaticAssetRole::ScrollUp)),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Vendor,
                            crate::service_scroll::Part::Up,
                        ),
                    ));
                    dialog.spawn((
                        VendorUiElement::ScrollDown,
                        VENDOR_SCROLL_DOWN_RECT.node(),
                        stretched_image(assets.image(VendorStaticAssetRole::ScrollDown)),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Vendor,
                            crate::service_scroll::Part::Down,
                        ),
                    ));
                });
            let button_style = VendorUiTextStyle::ButtonMiddleCenter;
            let mut button_node = VENDOR_GO_TO_STUFF_RECT.node();
            button_style.apply_to_container(&mut button_node);
            panel
                .spawn((
                    Button,
                    VendorUiElement::GoToStuff,
                    VendorInteractiveControl::GoToStuff,
                    button_node,
                    sliced_image(
                        assets.image(VendorStaticAssetRole::ButtonNormal),
                        VENDOR_BUTTON_BORDER,
                    ),
                ))
                .with_children(|button| {
                    button.spawn((
                        VendorUiElement::GoToStuffLabel,
                        Text::new("GO TO MY STUFF"),
                        button_style.font(assets.font.clone()),
                        TextColor(vendor_button_text_color(false)),
                        button_style.layout(),
                        button_style,
                        UiTransform::from_translation(Val2::px(
                            0.0,
                            button_style.replacement_y_offset(),
                        )),
                        LocalizedText::new("ui.inventory.go_to_my_stuff", "GO TO MY STUFF"),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
        });
}

pub(super) fn spawn_vendor_row(parent: &mut ChildSpawnerCommands, row: usize, assets: &VendorUiAssets) {
    parent
        .spawn((
            Button,
            VendorUiElement::Row(row),
            crate::ui::shared::controller::ControllerUiDefault,
            VendorInteractiveControl::Row(row),
            Node {
                display: Display::None,
                ..VendorUiRect::new(
                    0.0,
                    row as f32 * VENDOR_ROW_HEIGHT,
                    VENDOR_ROW_WIDTH,
                    VENDOR_ROW_VISUAL_HEIGHT,
                )
                .node()
            },
            stretched_image(assets.image(VendorStaticAssetRole::ItemRow)),
        ))
        .with_children(|row_node| {
            row_node.spawn((
                VendorUiElement::RowFrameUnder(row),
                BackgroundColor(Color::NONE),
                Node {
                    display: Display::None,
                    ..VENDOR_ROW_ITEM_BOX_RECT.node()
                },
                stretched_image(assets.image(VendorStaticAssetRole::SlotOccupied)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            row_node.spawn((
                VendorUiElement::RowIcon(row),
                Node {
                    display: Display::None,
                    ..VENDOR_ROW_ITEM_BOX_RECT.node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            row_node.spawn((
                VendorUiElement::RowFrameOver(row),
                Node {
                    display: Display::None,
                    ..VENDOR_ROW_ITEM_BOX_RECT.node()
                },
                stretched_image(assets.image(VendorStaticAssetRole::SlotOccupied)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_vendor_text(
                row_node,
                VendorUiElement::RowCount(row),
                VendorUiRect::new(9.0, 10.0, 62.0, 62.0),
                "",
                inventory_count_localized(""),
                assets.font.clone(),
                VendorUiTextStyle::LabelUpperLeft,
                Color::WHITE,
            );
            spawn_vendor_text(
                row_node,
                VendorUiElement::RowName(row),
                VENDOR_ROW_NAME_RECT,
                "",
                vendor_item_name_localized(""),
                assets.font.clone(),
                VendorUiTextStyle::LabelUpperLeft,
                Color::WHITE,
            );
            spawn_vendor_text(
                row_node,
                VendorUiElement::RowLevel(row),
                VENDOR_ROW_LEVEL_RECT,
                "",
                vendor_level_localized(None),
                assets.font.clone(),
                VendorUiTextStyle::LabelUpperLeft,
                Color::srgb(1.0, 1.0, 0.0),
            );
            spawn_vendor_text(
                row_node,
                VendorUiElement::RowVehicleSpeed(row),
                VENDOR_ROW_VEHICLE_SPEED_RECT,
                "",
                vendor_vehicle_speed_localized(0),
                assets.font.clone(),
                VendorUiTextStyle::LabelUpperLeft,
                Color::srgb(1.0, 1.0, 0.0),
            );
            spawn_vendor_text(
                row_node,
                VendorUiElement::RowPrice(row),
                VENDOR_ROW_PRICE_RECT,
                "",
                vendor_price_localized(None),
                assets.font.clone(),
                VendorUiTextStyle::BlankBoxMiddleRight,
                Color::srgb(0.794_354_86, 1.0, 1.0),
            );
            row_node.spawn((
                VendorUiElement::RowTaros(row),
                VENDOR_ROW_PRICE_ICON_RECT.node(),
                stretched_image(assets.image(VendorStaticAssetRole::Taros)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

pub(super) fn spawn_vendor_tab_button(
    parent: &mut ChildSpawnerCommands,
    marker: VendorUiElement,
    control: VendorInteractiveControl,
    rect: VendorUiRect,
    value: &'static str,
    font: Handle<Font>,
) {
    let style = VendorUiTextStyle::BlankBoxUpperLeft;
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent.spawn((
        Button,
        marker,
        control,
        crate::ui::shared::controller::ControllerUiTab(if control == VendorInteractiveControl::BuyTab { 0 } else { 1 }),
        node,
        BackgroundColor(Color::NONE),
        Text::new(value),
        style.font(font),
        TextColor(Color::srgb(0.794_354_86, 1.0, 1.0)),
        style.layout(),
        style,
        UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
        vendor_tab_localized(value),
    ));
}

pub(super) fn spawn_vendor_pc_stuff_panel(parent: &mut ChildSpawnerCommands, assets: &VendorUiAssets) {
    parent
        .spawn((
            VendorUiElement::PcStuffPanel,
            VendorUiRect::from(USER_EQUIP_PC_STUFF_RECT).node(),
            sliced_image(
                assets.image(VendorStaticAssetRole::InventoryPanel),
                USER_EQUIP_INVENTORY_PANEL_BORDER,
            ),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            spawn_vendor_text(
                panel,
                VendorUiElement::ItemTabLabel,
                VendorUiRect::from(USER_EQUIP_ITEM_TAB_HIT_RECT),
                "EQUIPMENT",
                LocalizedText::new("ui.inventory.tab.equipment", "EQUIPMENT"),
                assets.font.clone(),
                VendorUiTextStyle::BlankBoxUpperLeft,
                Color::srgb(0.794_354_86, 1.0, 1.0),
            );
            panel
                .spawn((
                    VendorUiElement::InventoryViewport,
                    Node {
                        overflow: Overflow::clip(),
                        ..VendorUiRect::from(USER_EQUIP_INVENTORY_VIEWPORT_RECT).node()
                    },
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            VendorUiElement::InventoryContent,
                            VendorUiRect::new(
                                0.0,
                                0.0,
                                USER_EQUIP_INVENTORY_CONTENT_WIDTH,
                                USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
                            )
                            .node(),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ))
                        .with_children(|content| {
                            for slot in 0..INVENTORY_SLOT_COUNT_0104 {
                                spawn_vendor_inventory_slot(content, slot, assets);
                            }
                        });
                });
            panel.spawn((
                VendorUiElement::InventoryShadow,
                VENDOR_PC_STUFF_INVENTORY_SHADOW_RECT.node(),
                ImageNode {
                    color: Color::BLACK,
                    ..sliced_image(
                        assets.image(VendorStaticAssetRole::ScrollShadow),
                        VENDOR_SCROLL_SHADOW_BORDER,
                    )
                },
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            panel.spawn((
                VendorUiElement::DexlabsBanner,
                VENDOR_PC_STUFF_DEXLABS_RECT.node(),
                stretched_image(assets.image(VendorStaticAssetRole::DexlabsBanner)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            let button_style = VendorUiTextStyle::ButtonMiddleCenter;
            let mut redeem_node = VENDOR_PC_STUFF_REDEEM_CODE_RECT.node();
            button_style.apply_to_container(&mut redeem_node);
            panel
                .spawn((
                    Button,
                    VendorUiElement::RedeemCode,
                    VendorInteractiveControl::RedeemCode,
                    redeem_node,
                    sliced_image(
                        assets.image(VendorStaticAssetRole::ButtonNormal),
                        VENDOR_BUTTON_BORDER,
                    ),
                ))
                .with_children(|button| {
                    button.spawn((
                        VendorUiElement::RedeemCodeLabel,
                        Text::new("REDEEM CODE"),
                        button_style.font(assets.font.clone()),
                        TextColor(vendor_button_text_color(false)),
                        button_style.layout(),
                        button_style,
                        UiTransform::from_translation(Val2::px(
                            0.0,
                            button_style.replacement_y_offset(),
                        )),
                        LocalizedText::new("ui.enchant.redeem_code", "REDEEM CODE"),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
            panel.spawn((
                VendorUiElement::TarosCounter,
                VENDOR_PC_STUFF_TAROS_COUNTER_RECT.node(),
                stretched_image(assets.image(VendorStaticAssetRole::TarosCounter)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            for (index, rect) in VENDOR_PC_STUFF_TAROS_DIGIT_RECTS.into_iter().enumerate() {
                spawn_vendor_text(
                    panel,
                    VendorUiElement::TarosDigit(index),
                    rect,
                    "0",
                    vendor_taros_digit_localized("0"),
                    assets.font.clone(),
                    VendorUiTextStyle::BlankBoxMiddleRight,
                    Color::srgb(0.794_354_86, 1.0, 1.0),
                );
            }
            panel.spawn((
                Button,
                VendorUiElement::Close,
                crate::ui::shared::controller::ControllerUiClose,
                VendorInteractiveControl::Close,
                VendorUiRect::from(USER_EQUIP_CLOSE_RECT).node(),
                stretched_image(assets.image(VendorStaticAssetRole::Close)),
            ));
            panel.spawn((
                VendorUiElement::Trash,
                VendorUiRect::from(USER_EQUIP_TRASH_RECT).node(),
                stretched_image(assets.image(VendorStaticAssetRole::Trash)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            panel.spawn((
                Button,
                VendorUiElement::Help,
                VendorInteractiveControl::Help,
                VendorUiRect::from(USER_EQUIP_HELP_RECT).node(),
                stretched_image(assets.image(VendorStaticAssetRole::Help)),
            ));
        });
}

pub(super) fn spawn_vendor_inventory_slot(
    parent: &mut ChildSpawnerCommands,
    slot: usize,
    assets: &VendorUiAssets,
) {
    let rect = VendorUiRect::new(
        (slot % 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        (slot / 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
    );
    parent
        .spawn((
            Button,
            VendorUiElement::InventorySlotFrame(slot),
            BackgroundColor(Color::NONE),
            VendorInteractiveControl::InventorySlot(slot),
            rect.node(),
            stretched_image(assets.image(VendorStaticAssetRole::SlotEmpty)),
        ))
        .with_children(|slot_node| {
            spawn_vendor_slot_children(
                slot_node,
                VendorUiElement::InventorySlotIcon(slot),
                VendorUiElement::InventorySlotBadge(slot),
                VendorUiElement::InventorySlotCount(slot),
                USER_EQUIP_INVENTORY_SLOT_SIZE,
                assets,
            );
        });
}

pub(super) fn spawn_vendor_slot_children(
    parent: &mut ChildSpawnerCommands,
    icon_marker: VendorUiElement,
    badge_marker: VendorUiElement,
    count_marker: VendorUiElement,
    slot_size: f32,
    assets: &VendorUiAssets,
) {
    parent.spawn((
        icon_marker,
        Node {
            display: Display::None,
            ..VendorUiRect::new(0.0, 0.0, slot_size, slot_size).node()
        },
        ImageNode::default(),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    parent.spawn((
        badge_marker,
        Node {
            display: Display::None,
            ..VendorUiRect::new(
                USER_EQUIP_COMBINED_BADGE_LEFT,
                USER_EQUIP_COMBINED_BADGE_TOP,
                USER_EQUIP_COMBINED_BADGE_SIZE,
                USER_EQUIP_COMBINED_BADGE_SIZE,
            )
            .node()
        },
        stretched_image(assets.image(VendorStaticAssetRole::Combined)),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    spawn_vendor_text(
        parent,
        count_marker,
        VendorUiRect::new(
            USER_EQUIP_COUNT_LABEL_LEFT,
            USER_EQUIP_COUNT_LABEL_TOP,
            slot_size,
            slot_size,
        ),
        "",
        inventory_count_localized(""),
        assets.font.clone(),
        VendorUiTextStyle::LabelUpperLeft,
        Color::WHITE,
    );
}

pub(super) fn spawn_vendor_equipment_panel(parent: &mut ChildSpawnerCommands, assets: &VendorUiAssets) {
    parent
        .spawn((
            VendorUiElement::EquipmentPanel,
            VendorUiRect::from(USER_EQUIP_EQUIP_STRIP_RECT).node(),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            let title_style = VendorUiTextStyle::EquipBarMiddleCenter;
            let mut title_node = VendorUiRect::from(USER_EQUIP_EQUIPMENT_TITLE_RECT).node();
            title_style.apply_to_container(&mut title_node);
            panel
                .spawn((
                    VendorUiElement::EquipmentTitle,
                    title_node,
                    stretched_image(assets.image(VendorStaticAssetRole::EquipTitle)),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|title| {
                    title.spawn((
                        VendorUiElement::EquipmentTitleLabel,
                        Text::new(" Equipped"),
                        title_style.font(assets.font.clone()),
                        TextColor(Color::srgb(0.0, 0.2, 0.4)),
                        title_style.layout(),
                        title_style,
                        UiTransform::from_translation(Val2::px(
                            0.0,
                            title_style.replacement_y_offset(),
                        )),
                        LocalizedText::new("ui.inventory.equipped", " Equipped"),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
            for visual_index in 0..USER_EQUIP_EQUIPMENT_STRIP_COUNT {
                spawn_vendor_equipment_slot(panel, visual_index, assets);
            }
            for battery_index in 0..2 {
                spawn_vendor_battery_slot(panel, battery_index, assets);
            }
        });
}

pub(super) fn spawn_vendor_battery_slot(
    parent: &mut ChildSpawnerCommands,
    battery_index: usize,
    assets: &VendorUiAssets,
) {
    parent.spawn((
        VendorUiElement::BatterySlotFrame(battery_index),
        VENDOR_BATTERY_FRAME_RECTS[battery_index].node(),
        stretched_image(assets.image(VendorStaticAssetRole::SlotEmpty)),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    parent.spawn((
        VendorUiElement::BatteryIcon(battery_index),
        VENDOR_BATTERY_ICON_RECTS[battery_index].node(),
        stretched_image(assets.image(if battery_index == 0 {
            VendorStaticAssetRole::BoostIcon
        } else {
            VendorStaticAssetRole::PotionIcon
        })),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    let (label, localized) = if battery_index == 0 {
        (
            "BOOSTS",
            LocalizedText::new("ui.enchant.battery.boosts", "BOOSTS"),
        )
    } else {
        (
            "POTIONS",
            LocalizedText::new("ui.enchant.battery.potions", "POTIONS"),
        )
    };
    spawn_vendor_text(
        parent,
        VendorUiElement::BatteryLabel(battery_index),
        VENDOR_BATTERY_LABEL_RECTS[battery_index],
        label,
        localized,
        assets.font.clone(),
        VendorUiTextStyle::EquipFontMiddleRight,
        Color::WHITE,
    );
    spawn_vendor_text(
        parent,
        VendorUiElement::BatteryCount(battery_index),
        VENDOR_BATTERY_COUNT_RECTS[battery_index],
        "0",
        vendor_battery_count_localized(0),
        assets.font.clone(),
        VendorUiTextStyle::LabelUpperLeft,
        Color::WHITE,
    );
}
