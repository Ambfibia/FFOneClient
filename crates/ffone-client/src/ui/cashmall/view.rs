use super::*;

pub(super) fn spawn_cashmall_ui_0104(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_assets: ResMut<Assets<Image>>,
    contract: Res<CashmallUiAssetContract0104>,
) {
    let assets = CashmallUiAssets0104::load(&asset_server, &mut image_assets, &contract);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            CashmallUiRoot0104,
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
            GlobalZIndex(CASHMALL_UI_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                CashmallUiElement0104::Backdrop,
                CashmallUiRect0104::default().node(),
                stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Backdrop)),
                Pickable::IGNORE,
            ));
            root.spawn((
                CashmallUiElement0104::CashmallBackplate,
                CashmallUiRect0104::default().node(),
                stretched_cashmall_image_0104(
                    assets.image(CashmallStaticAssetRole0104::CashmallBackplate),
                ),
                Pickable::IGNORE,
            ));
            root.spawn((
                CashmallUiElement0104::RightBackplate,
                CashmallUiRect0104::default().node(),
                sliced_cashmall_image_0104(
                    assets.image(CashmallStaticAssetRole0104::RightBackplate),
                    USER_EQUIP_RIGHT_PANEL_BORDER,
                ),
                Pickable::IGNORE,
            ));
            spawn_cashmall_left_panel_0104(root, &assets);
            spawn_cashmall_pc_stuff_panel_0104(root, &assets);
            spawn_cashmall_equipment_panel_0104(root, &assets);
        });
}

pub(super) fn spawn_cashmall_left_panel_0104(
    parent: &mut ChildSpawnerCommands,
    assets: &CashmallUiAssets0104,
) {
    parent
        .spawn((
            CashmallUiElement0104::CashmallPanel,
            CashmallUiRect0104::default().node(),
            ZIndex(CASHMALL_PANEL_NATIVE_Z_0104),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    CashmallUiElement0104::Info,
                    CASHMALL_INFO_RECT.node(),
                    stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Info)),
                    Pickable::IGNORE,
                ))
                .with_children(|info| {
                    spawn_cashmall_text_0104(
                        info,
                        CashmallUiElement0104::Title,
                        CASHMALL_TITLE_RECT,
                        LocalizedText::new("ui.cashmall.title", "SHOPPING MALL"),
                        assets.font.clone(),
                        CashmallTextStyle0104::LabelUpperLeft,
                        Color::WHITE,
                    );
                    spawn_cashmall_text_0104(
                        info,
                        CashmallUiElement0104::NpcName,
                        CASHMALL_NPC_NAME_RECT,
                        cashmall_passthrough_text_0104(""),
                        assets.font.clone(),
                        CashmallTextStyle0104::LabelUpperLeft,
                        Color::WHITE,
                    );
                    info.spawn((
                        CashmallUiElement0104::Cash,
                        CASHMALL_CASH_RECT.node(),
                        stretched_cashmall_image_0104(
                            assets.image(CashmallStaticAssetRole0104::Cash),
                        ),
                        Pickable::IGNORE,
                    ));
                    for digit in 0..CASHMALL_CASH_DIGIT_COUNT {
                        spawn_cashmall_text_0104(
                            info,
                            CashmallUiElement0104::CashDigit(digit),
                            CashmallUiRect0104::new(
                                CASHMALL_CASH_RECT.left + 1.0 + 12.0 * digit as f32,
                                CASHMALL_CASH_RECT.top + 3.0,
                                12.0,
                                20.0,
                            ),
                            cashmall_passthrough_text_0104("0"),
                            assets.font.clone(),
                            CashmallTextStyle0104::BlankBoxMiddleRight,
                            Color::srgb(0.794_354_86, 1.0, 1.0),
                        );
                    }
                });
            panel
                .spawn((
                    CashmallUiElement0104::Dialog,
                    CASHMALL_DIALOG_RECT.node(),
                    Pickable::IGNORE,
                ))
                .with_children(|dialog| {
                    dialog.spawn((
                        CashmallUiElement0104::ListBack,
                        CASHMALL_LIST_BACK_RECT.node(),
                        sliced_cashmall_image_0104(
                            assets.image(CashmallStaticAssetRole0104::ListBack),
                            CASHMALL_LIST_BACK_BORDER,
                        ),
                        Pickable::IGNORE,
                    ));
                    for tab in CashmallTab0104::ALL.into_iter().rev() {
                        dialog.spawn((
                            CashmallUiElement0104::TabVisual(tab),
                            CASHMALL_TAB_RECTS[tab.index()].node(),
                            sliced_cashmall_image_0104(
                                assets.image(cashmall_tab_asset_role_0104(
                                    tab,
                                    CashmallTabVisual0104::Normal,
                                )),
                                cashmall_tab_border_0104(tab),
                            ),
                            ZIndex(1),
                            Pickable::IGNORE,
                        ));
                    }
                    dialog.spawn((
                        CashmallUiElement0104::ListDivider,
                        CASHMALL_LIST_DIVIDER_RECT.node(),
                        stretched_cashmall_image_0104(
                            assets.image(CashmallStaticAssetRole0104::BackBar),
                        ),
                        ZIndex(2),
                        Pickable::IGNORE,
                    ));
                    for tab in CashmallTab0104::ALL {
                        dialog.spawn((
                            Button,
                            CashmallInteractiveControl0104::Tab(tab),
                            CASHMALL_TAB_HIT_RECTS[tab.index()].node(),
                            ZIndex(5),
                            BackgroundColor(Color::NONE),
                        ));
                        spawn_cashmall_text_with_z_0104(
                            dialog,
                            CashmallUiElement0104::TabLabel(tab),
                            CASHMALL_TAB_HIT_RECTS[tab.index()],
                            cashmall_tab_localized_text_0104(tab),
                            assets.font.clone(),
                            CashmallTextStyle0104::BlankBoxUpperLeft,
                            Color::srgb(0.794_354_86, 1.0, 1.0),
                            4,
                        );
                    }
                    dialog
                        .spawn((
                            CashmallUiElement0104::Table,
                            CASHMALL_TABLE_RECT.node(),
                            Pickable::IGNORE,
                        ))
                        .with_children(|table| {
                            table
                                .spawn((
                                    CashmallUiElement0104::ListViewport,
                                    Node {
                                        overflow: Overflow::clip(),
                                        ..CASHMALL_LIST_VIEWPORT_RECT.node()
                                    },
                                    Pickable::IGNORE,
                                ))
                                .with_children(|viewport| {
                                    viewport
                                        .spawn((
                                            CashmallUiElement0104::ListContent,
                                            CashmallUiRect0104::new(
                                                0.0,
                                                0.0,
                                                CASHMALL_ROW_WIDTH,
                                                CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT,
                                            )
                                            .node(),
                                            Pickable::IGNORE,
                                        ))
                                        .with_children(|content| {
                                            for row in 0..CASHMALL_SLOT_SCAN_COUNT {
                                                spawn_cashmall_row_0104(content, row, assets);
                                            }
                                        });
                                });
                            table.spawn((
                                CashmallUiElement0104::TableShadow,
                                CASHMALL_TABLE_SHADOW_RECT.node(),
                                sliced_cashmall_image_0104(
                                    assets.image(CashmallStaticAssetRole0104::ScrollShadow),
                                    CASHMALL_SCROLL_SHADOW_BORDER,
                                ),
                                Pickable::IGNORE,
                            ));
                        });
                });
            panel
                .spawn((
                    Button,
                    CashmallUiElement0104::GoToStuff,
                    CashmallInteractiveControl0104::GoToStuff,
                    CashmallTextStyle0104::ButtonMiddleCenter.node(CASHMALL_GO_TO_STUFF_RECT),
                    sliced_cashmall_image_0104(
                        assets.image(CashmallStaticAssetRole0104::ButtonNormal),
                        CASHMALL_BUTTON_BORDER,
                    ),
                ))
                .with_children(|button| {
                    button.spawn((
                        CashmallUiElement0104::GoToStuffLabel,
                        CashmallTextStyle0104::ButtonMiddleCenter,
                        Text::new("GO TO MY STUFF"),
                        LocalizedText::new("ui.cashmall.go_to_my_stuff", "GO TO MY STUFF"),
                        CashmallTextStyle0104::ButtonMiddleCenter.font(assets.font.clone()),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        CashmallTextStyle0104::ButtonMiddleCenter.layout(),
                        Pickable::IGNORE,
                    ));
                });
        });
}

pub(super) fn spawn_cashmall_row_0104(
    parent: &mut ChildSpawnerCommands,
    row: usize,
    assets: &CashmallUiAssets0104,
) {
    parent
        .spawn((
            Button,
            CashmallUiElement0104::Row(row),
            CashmallInteractiveControl0104::Row(row),
            CashmallUiRect0104::new(
                0.0,
                row as f32 * CASHMALL_ROW_HEIGHT,
                CASHMALL_ROW_WIDTH,
                CASHMALL_ROW_VISUAL_HEIGHT,
            )
            .node(),
            BackgroundColor(Color::NONE),
        ))
        .with_children(|row_node| {
            row_node.spawn((
                CashmallUiElement0104::RowFrame(row),
                CASHMALL_ROW_ITEM_BOX_RECT.node(),
                stretched_cashmall_image_0104(
                    assets.image(CashmallStaticAssetRole0104::SlotOccupied),
                ),
                ZIndex(0),
                Pickable::IGNORE,
            ));
            row_node.spawn((
                CashmallUiElement0104::RowIcon(row),
                Node {
                    display: Display::None,
                    ..CASHMALL_ROW_ITEM_BOX_RECT.node()
                },
                ImageNode::default(),
                ZIndex(1),
                Pickable::IGNORE,
            ));
            spawn_cashmall_text_0104(
                row_node,
                CashmallUiElement0104::RowName(row),
                CASHMALL_ROW_NAME_RECT,
                cashmall_passthrough_text_0104(""),
                assets.font.clone(),
                CashmallTextStyle0104::LabelUpperLeft,
                Color::WHITE,
            );
            spawn_cashmall_text_0104(
                row_node,
                CashmallUiElement0104::RowLevel(row),
                CASHMALL_ROW_LEVEL_RECT,
                cashmall_item_level_text_0104(0),
                assets.font.clone(),
                CashmallTextStyle0104::LabelUpperLeft,
                Color::srgb(1.0, 1.0, 0.0),
            );
        });
}

pub(super) fn spawn_cashmall_pc_stuff_panel_0104(
    parent: &mut ChildSpawnerCommands,
    assets: &CashmallUiAssets0104,
) {
    parent
        .spawn((
            CashmallUiElement0104::PcStuffPanel,
            CashmallUiRect0104::from(USER_EQUIP_PC_STUFF_RECT).node(),
            ZIndex(CASHMALL_SHARED_NATIVE_Z_0104),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            // Clean draws the inactive Nano tab before beginning the
            // `FFInvBack` group, so the panel's nine-slice overlays it.
            panel.spawn((
                CashmallUiElement0104::NanoTab,
                CASHMALL_PC_STUFF_NANO_TAB_RECT.node(),
                stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::NanoTab)),
                ZIndex(0),
                Pickable::IGNORE,
            ));
            panel.spawn((
                CashmallUiElement0104::InventoryPanel,
                CASHMALL_PC_STUFF_INVENTORY_PANEL_RECT.node(),
                sliced_cashmall_image_0104(
                    assets.image(CashmallStaticAssetRole0104::InventoryPanel),
                    USER_EQUIP_INVENTORY_PANEL_BORDER,
                ),
                ZIndex(1),
                Pickable::IGNORE,
            ));
            spawn_cashmall_text_with_z_0104(
                panel,
                CashmallUiElement0104::ItemTabLabel,
                CashmallUiRect0104::from(USER_EQUIP_ITEM_TAB_HIT_RECT),
                LocalizedText::new("ui.cashmall.inventory.equipment", "EQUIPMENT"),
                assets.font.clone(),
                CashmallTextStyle0104::BlankBoxUpperLeft,
                Color::srgb(0.794_354_86, 1.0, 1.0),
                3,
            );
            panel.spawn((
                Button,
                CashmallUiElement0104::NanoTabHit,
                CashmallInteractiveControl0104::NanoTab,
                CASHMALL_PC_STUFF_NANO_TAB_HIT_RECT.node(),
                BackgroundColor(Color::NONE),
                ZIndex(4),
            ));
            spawn_cashmall_text_with_z_0104(
                panel,
                CashmallUiElement0104::NanoTabLabel,
                CashmallUiRect0104::from(USER_EQUIP_NANO_TAB_HIT_RECT),
                LocalizedText::new("ui.inventory.tab.nanos", "NANOS"),
                assets.font.clone(),
                CashmallTextStyle0104::BlankBoxUpperLeft,
                Color::srgb(0.794_354_86, 1.0, 1.0),
                3,
            );
            panel
                .spawn((
                    CashmallUiElement0104::InventoryViewport,
                    Node {
                        overflow: Overflow::clip(),
                        ..CashmallUiRect0104::from(USER_EQUIP_INVENTORY_VIEWPORT_RECT).node()
                    },
                    ZIndex(2),
                    Pickable::IGNORE,
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            CashmallUiElement0104::InventoryContent,
                            CashmallUiRect0104::new(
                                0.0,
                                0.0,
                                USER_EQUIP_INVENTORY_CONTENT_WIDTH,
                                USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
                            )
                            .node(),
                            Pickable::IGNORE,
                        ))
                        .with_children(|content| {
                            for slot in 0..INVENTORY_SLOT_COUNT_0104 {
                                spawn_cashmall_inventory_slot_0104(content, slot, assets);
                            }
                        });
                });
            panel.spawn((
                CashmallUiElement0104::DexlabsBanner,
                CASHMALL_PC_STUFF_DEXLABS_RECT.node(),
                stretched_cashmall_image_0104(
                    assets.image(CashmallStaticAssetRole0104::DexlabsBanner),
                ),
                ZIndex(2),
                Pickable::IGNORE,
            ));
            panel.spawn((
                CashmallUiElement0104::TarosCounter,
                CASHMALL_PC_STUFF_TAROS_COUNTER_RECT.node(),
                stretched_cashmall_image_0104(
                    assets.image(CashmallStaticAssetRole0104::TarosCounter),
                ),
                ZIndex(2),
                Pickable::IGNORE,
            ));
            for (index, rect) in CASHMALL_PC_STUFF_TAROS_DIGIT_RECTS.into_iter().enumerate() {
                spawn_cashmall_text_with_z_0104(
                    panel,
                    CashmallUiElement0104::TarosDigit(index),
                    rect,
                    cashmall_passthrough_text_0104("0"),
                    assets.font.clone(),
                    CashmallTextStyle0104::BlankBoxMiddleRight,
                    Color::srgb(0.794_354_86, 1.0, 1.0),
                    3,
                );
            }
            panel
                .spawn((
                    Button,
                    CashmallUiElement0104::RedeemCode,
                    CashmallInteractiveControl0104::RedeemCode,
                    CashmallTextStyle0104::ButtonMiddleCenter
                        .node(CASHMALL_PC_STUFF_REDEEM_CODE_RECT),
                    sliced_cashmall_image_0104(
                        assets.image(CashmallStaticAssetRole0104::ButtonNormal),
                        CASHMALL_BUTTON_BORDER,
                    ),
                    ZIndex(3),
                ))
                .with_children(|button| {
                    button.spawn((
                        CashmallUiElement0104::RedeemCodeLabel,
                        CashmallTextStyle0104::ButtonMiddleCenter,
                        Text::new("REDEEM CODE"),
                        LocalizedText::new("ui.enchant.redeem_code", "REDEEM CODE"),
                        CashmallTextStyle0104::ButtonMiddleCenter.font(assets.font.clone()),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        CashmallTextStyle0104::ButtonMiddleCenter.layout(),
                        Pickable::IGNORE,
                    ));
                });
            panel.spawn((
                Button,
                CashmallUiElement0104::Close,
                CashmallInteractiveControl0104::Close,
                CashmallUiRect0104::from(USER_EQUIP_CLOSE_RECT).node(),
                stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Close)),
                ZIndex(3),
            ));
            panel.spawn((
                CashmallUiElement0104::Trash,
                CashmallUiRect0104::from(USER_EQUIP_TRASH_RECT).node(),
                stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Trash)),
                ZIndex(3),
                Pickable::IGNORE,
            ));
            panel.spawn((
                Button,
                CashmallUiElement0104::Help,
                CashmallInteractiveControl0104::Help,
                CashmallUiRect0104::from(USER_EQUIP_HELP_RECT).node(),
                stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Help)),
                ZIndex(3),
            ));
        });
}

pub(super) fn spawn_cashmall_inventory_slot_0104(
    parent: &mut ChildSpawnerCommands,
    slot: usize,
    assets: &CashmallUiAssets0104,
) {
    let rect = CashmallUiRect0104::new(
        (slot % 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        (slot / 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
    );
    parent
        .spawn((
            CashmallUiElement0104::InventorySlotFrame(slot),
            rect.node(),
            stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::SlotEmpty)),
            Pickable::IGNORE,
        ))
        .with_children(|slot_node| {
            spawn_cashmall_shared_slot_children_0104(
                slot_node,
                CashmallUiElement0104::InventorySlotIcon(slot),
                CashmallUiElement0104::InventorySlotBadge(slot),
                CashmallUiElement0104::InventorySlotCount(slot),
                USER_EQUIP_INVENTORY_SLOT_SIZE,
                assets,
            );
        });
}

pub(super) fn spawn_cashmall_shared_slot_children_0104(
    parent: &mut ChildSpawnerCommands,
    icon_marker: CashmallUiElement0104,
    badge_marker: CashmallUiElement0104,
    count_marker: CashmallUiElement0104,
    slot_size: f32,
    assets: &CashmallUiAssets0104,
) {
    parent.spawn((
        icon_marker,
        Node {
            display: Display::None,
            ..CashmallUiRect0104::new(0.0, 0.0, slot_size, slot_size).node()
        },
        ImageNode::default(),
        Pickable::IGNORE,
    ));
    parent.spawn((
        badge_marker,
        Node {
            display: Display::None,
            ..CashmallUiRect0104::new(
                USER_EQUIP_COMBINED_BADGE_LEFT,
                USER_EQUIP_COMBINED_BADGE_TOP,
                USER_EQUIP_COMBINED_BADGE_SIZE,
                USER_EQUIP_COMBINED_BADGE_SIZE,
            )
            .node()
        },
        stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Combined)),
        Pickable::IGNORE,
    ));
    spawn_cashmall_text_0104(
        parent,
        count_marker,
        CashmallUiRect0104::new(
            USER_EQUIP_COUNT_LABEL_LEFT,
            USER_EQUIP_COUNT_LABEL_TOP,
            slot_size,
            slot_size,
        ),
        cashmall_passthrough_text_0104(""),
        assets.font.clone(),
        CashmallTextStyle0104::LabelUpperLeft,
        Color::WHITE,
    );
}

pub(super) fn spawn_cashmall_equipment_panel_0104(
    parent: &mut ChildSpawnerCommands,
    assets: &CashmallUiAssets0104,
) {
    parent
        .spawn((
            CashmallUiElement0104::EquipmentPanel,
            CashmallUiRect0104::from(USER_EQUIP_EQUIP_STRIP_RECT).node(),
            ZIndex(CASHMALL_SHARED_NATIVE_Z_0104),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    CashmallUiElement0104::EquipmentTitle,
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..CashmallUiRect0104::from(USER_EQUIP_EQUIPMENT_TITLE_RECT).node()
                    },
                    stretched_cashmall_image_0104(
                        assets.image(CashmallStaticAssetRole0104::EquipTitle),
                    ),
                    Pickable::IGNORE,
                ))
                .with_children(|title| {
                    title.spawn((
                        CashmallUiElement0104::EquipmentTitleLabel,
                        CashmallTextStyle0104::EquipBarMiddleCenter,
                        Text::new(" Equipped"),
                        LocalizedText::new("ui.cashmall.equipment.title", " Equipped"),
                        CashmallTextStyle0104::EquipBarMiddleCenter.font(assets.font.clone()),
                        TextColor(Color::srgb(0.0, 0.2, 0.4)),
                        CashmallTextStyle0104::EquipBarMiddleCenter.layout(),
                        Pickable::IGNORE,
                    ));
                });
            for visual_index in 0..USER_EQUIP_EQUIPMENT_STRIP_COUNT {
                spawn_cashmall_equipment_slot_0104(panel, visual_index, assets);
            }
        });
}

pub(super) fn spawn_cashmall_equipment_slot_0104(
    parent: &mut ChildSpawnerCommands,
    visual_index: usize,
    assets: &CashmallUiAssets0104,
) {
    let top = USER_EQUIP_EQUIPMENT_CONTENT_RECT.top
        + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE;
    parent
        .spawn((
            CashmallUiElement0104::EquipmentSlotFrame(visual_index),
            CashmallUiRect0104::new(
                USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
                top,
                USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                USER_EQUIP_EQUIPMENT_SLOT_SIZE,
            )
            .node(),
            stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::SlotEmpty)),
            Pickable::IGNORE,
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                CashmallUiElement0104::EquipmentSlotIcon(visual_index),
                Node {
                    display: Display::None,
                    ..CashmallUiRect0104::new(
                        0.0,
                        0.0,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                    )
                    .node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            slot_node.spawn((
                CashmallUiElement0104::EquipmentSlotBadge(visual_index),
                Node {
                    display: Display::None,
                    ..CashmallUiRect0104::new(
                        USER_EQUIP_COMBINED_BADGE_LEFT,
                        USER_EQUIP_COMBINED_BADGE_TOP,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                    )
                    .node()
                },
                stretched_cashmall_image_0104(assets.image(CashmallStaticAssetRole0104::Combined)),
                Pickable::IGNORE,
            ));
            spawn_cashmall_text_0104(
                slot_node,
                CashmallUiElement0104::EquipmentSlotLabel(visual_index),
                CashmallUiRect0104::new(0.0, -2.0, 60.0, 19.0),
                cashmall_equipment_slot_localized_text_0104(visual_index),
                assets.font.clone(),
                CashmallTextStyle0104::EquipFontMiddleRight,
                Color::WHITE,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_cashmall_text_0104(
    parent: &mut ChildSpawnerCommands,
    marker: CashmallUiElement0104,
    rect: CashmallUiRect0104,
    localized: LocalizedText,
    font: Handle<Font>,
    style: CashmallTextStyle0104,
    color: Color,
) {
    spawn_cashmall_text_with_z_0104(parent, marker, rect, localized, font, style, color, 0);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_cashmall_text_with_z_0104(
    parent: &mut ChildSpawnerCommands,
    marker: CashmallUiElement0104,
    rect: CashmallUiRect0104,
    localized: LocalizedText,
    font: Handle<Font>,
    style: CashmallTextStyle0104,
    color: Color,
    z_index: i32,
) {
    let fallback = cashmall_resolved_fallback_0104(&localized);
    parent.spawn((
        marker,
        style,
        style.node(rect),
        Text::new(fallback),
        localized,
        style.font(font),
        TextColor(color),
        style.layout(),
        ZIndex(z_index),
        Pickable::IGNORE,
    ));
}
