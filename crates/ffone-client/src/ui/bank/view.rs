use super::*;

pub(super) fn spawn_bank_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_assets: ResMut<Assets<Image>>,
    contract: Res<BankUiAssetContract>,
) {
    let assets = BankUiAssets::load(&asset_server, &mut image_assets, &contract);
    commands.insert_resource(BankUiRuntimeAssets(assets.clone()));
    commands
        .spawn((
            BankUiRoot,
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
            GlobalZIndex(BANK_UI_Z_INDEX),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                BankUiElement::Backdrop,
                BankUiRect::default().node(),
                stretched_image(assets.image(BankStaticAssetRole::Backdrop)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                BankUiElement::BankBackplate,
                BankUiRect::default().node(),
                stretched_image(assets.image(BankStaticAssetRole::TradeBack)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                BankUiElement::RightBackplate,
                BankUiRect::default().node(),
                sliced_image(
                    assets.image(BankStaticAssetRole::RightBackplate),
                    USER_EQUIP_RIGHT_PANEL_BORDER,
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_bank_panel(root, &assets);
            spawn_pc_stuff_panel(root, &assets);
            spawn_equipment_panel(root, &assets);
        });
}

pub(super) fn spawn_bank_panel(parent: &mut ChildSpawnerCommands, assets: &BankUiAssets) {
    parent
        .spawn((
            BankUiElement::BankPanel,
            BankUiRect::default().node(),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            panel.spawn((
                BankUiElement::BankInfo,
                BANK_INFO_RECT.node(),
                stretched_image(assets.image(BankStaticAssetRole::BankInfo)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_text_label(
                panel,
                BankUiElement::BankTitle,
                BANK_TITLE_RECT.translated(BANK_INFO_RECT.left, BANK_INFO_RECT.top),
                "Morbucks Savings and Loan",
                assets.font.clone(),
            );
            panel
                .spawn((
                    BankUiElement::BankDialog,
                    BANK_DIALOG_RECT.node(),
                    sliced_image(
                        assets.image(BankStaticAssetRole::BankPanel),
                        BANK_PANEL_BORDER,
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|dialog| {
                    search::spawn(dialog, assets);
                    spawn_text_label(
                        dialog,
                        BankUiElement::BankTab,
                        BANK_TAB_RECT,
                        "BANK",
                        assets.font.clone(),
                    );
                    dialog
                        .spawn((
                            BankUiElement::BankViewport,
                            Node {
                                overflow: Overflow::clip(),
                                ..BANK_VIEWPORT_RECT.node()
                            },
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ))
                        .with_children(|viewport| {
                            viewport
                                .spawn((
                                    BankUiElement::BankContent,
                                    BankUiRect::new(
                                        0.0,
                                        0.0,
                                        BANK_CONTENT_WIDTH,
                                        BANK_CONTENT_HEIGHT,
                                    )
                                    .node(),
                                    Pickable::IGNORE,
                                    bevy::ui::FocusPolicy::Pass,
                                ))
                                .with_children(|content| {
                                    for slot in 0..BANK_SLOT_COUNT_0104 {
                                        spawn_bank_slot(content, slot, assets);
                                    }
                                });
                        });
                    dialog.spawn((
                        BankUiElement::BankScrollTrack,
                        BANK_SCROLL_TRACK_RECT.node(),
                        sliced_image(
                            assets.image(BankStaticAssetRole::ScrollTrack),
                            crate::vendor_ui::VENDOR_SCROLL_TRACK_BORDER,
                        ),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Bank,
                            crate::service_scroll::Part::Track,
                        ),
                    ));
                    dialog.spawn((
                        BankUiElement::BankScrollThumb,
                        BankUiRect::new(
                            BANK_SCROLL_TRACK_RECT.left + 2.0,
                            BANK_SCROLL_TRACK_RECT.top,
                            13.0,
                            15.0,
                        )
                        .node(),
                        sliced_image(
                            assets.image(BankStaticAssetRole::ScrollThumb),
                            crate::vendor_ui::VENDOR_SCROLL_TRACK_BORDER,
                        ),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Bank,
                            crate::service_scroll::Part::Thumb,
                        ),
                    ));
                    dialog.spawn((
                        BankUiElement::BankScrollUp,
                        BANK_SCROLL_UP_RECT.node(),
                        stretched_image(assets.image(BankStaticAssetRole::ScrollUp)),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Bank,
                            crate::service_scroll::Part::Up,
                        ),
                    ));
                    dialog.spawn((
                        BankUiElement::BankScrollDown,
                        BANK_SCROLL_DOWN_RECT.node(),
                        stretched_image(assets.image(BankStaticAssetRole::ScrollDown)),
                        crate::service_scroll::control(
                            crate::service_scroll::Owner::Bank,
                            crate::service_scroll::Part::Down,
                        ),
                    ));
                    dialog.spawn((
                        BankUiElement::BankShadow,
                        BANK_SHADOW_RECT.node(),
                        ImageNode {
                            color: Color::BLACK,
                            ..sliced_image(
                                assets.image(BankStaticAssetRole::ScrollShadow),
                                BANK_SHADOW_BORDER,
                            )
                        },
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
        });
}

pub(super) fn spawn_bank_slot(parent: &mut ChildSpawnerCommands, slot: usize, assets: &BankUiAssets) {
    let frame = BankUiRect::new(
        (slot % BANK_GRID_COLUMNS) as f32 * BANK_SLOT_STRIDE,
        (slot / BANK_GRID_COLUMNS) as f32 * BANK_SLOT_STRIDE,
        BANK_SLOT_SIZE,
        BANK_SLOT_SIZE,
    );
    parent
        .spawn((
            BankUiElement::BankSlotFrame(slot),
            BankSlotControl(BankSlotRef0104::new(BankSlotLocation0104::Bank, slot).unwrap()),
            Button,
            frame.node(),
            sliced_image(
                assets.image(BankStaticAssetRole::BankSlotButton),
                BANK_SLOT_BUTTON_BORDER,
            ),
            Pickable::default(),
        ))
        .with_children(|slot_node| {
            spawn_slot_children(
                slot_node,
                BankUiElement::BankSlotIcon(slot),
                BankUiElement::BankSlotBadge(slot),
                BankUiElement::BankSlotCount(slot),
                BANK_SLOT_SIZE,
                BANK_COUNT_LABEL_LEFT,
                BANK_COUNT_LABEL_TOP,
                assets,
            );
        });
}

pub(super) fn spawn_pc_stuff_panel(parent: &mut ChildSpawnerCommands, assets: &BankUiAssets) {
    parent
        .spawn((
            BankUiElement::PcStuffPanel,
            BankUiRect::from(USER_EQUIP_PC_STUFF_RECT).node(),
            sliced_image(
                assets.image(BankStaticAssetRole::InventoryPanel),
                USER_EQUIP_INVENTORY_PANEL_BORDER,
            ),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            spawn_disabled_tab(
                panel,
                BankUiElement::ItemTabLabel,
                USER_EQUIP_ITEM_TAB_HIT_RECT,
                "EQUIPMENT",
                assets.font.clone(),
                BankDisabledControl::ActiveItemTab,
            );
            panel
                .spawn((
                    BankUiElement::InventoryViewport,
                    Node {
                        overflow: Overflow::clip(),
                        ..BankUiRect::from(USER_EQUIP_INVENTORY_VIEWPORT_RECT).node()
                    },
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            BankUiElement::InventoryContent,
                            BankUiRect::new(
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
                                spawn_inventory_slot(content, slot, assets);
                            }
                        });
                });
            panel.spawn((
                BankUiElement::DexlabsBanner,
                BANK_PC_STUFF_DEXLABS_RECT.node(),
                stretched_image(assets.image(BankStaticAssetRole::DexlabsBanner)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            let button_style = BankUiTextStyle::ButtonMiddleCenter;
            let mut redeem_node = BANK_PC_STUFF_REDEEM_CODE_RECT.node();
            button_style.apply_to_container(&mut redeem_node);
            panel
                .spawn((
                    Button,
                    BankUiElement::RedeemCode,
                    BankRedeemControl,
                    redeem_node,
                    sliced_image(
                        assets.image(BankStaticAssetRole::BankSlotButton),
                        BANK_SLOT_BUTTON_BORDER,
                    ),
                    Pickable::default(),
                    bevy::ui::FocusPolicy::Block,
                ))
                .with_children(|button| {
                    button.spawn((
                        BankUiElement::RedeemCodeLabel,
                        Text::new("REDEEM CODE"),
                        button_style.font(assets.font.clone()),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
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
                BankUiElement::TarosCounter,
                BANK_PC_STUFF_TAROS_COUNTER_RECT.node(),
                stretched_image(assets.image(BankStaticAssetRole::TarosCounter)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            for (index, rect) in BANK_PC_STUFF_TAROS_DIGIT_RECTS.into_iter().enumerate() {
                let style = BankUiTextStyle::BlankBoxMiddleRight;
                let mut node = rect.node();
                style.apply_to_container(&mut node);
                node.justify_content = JustifyContent::Center;
                panel
                    .spawn((
                        BankUiElement::TarosDigit(index),
                        node,
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ))
                    .with_children(|digit| {
                        digit.spawn((
                            BankTarosDigitText0104(index),
                            Text::new("0"),
                            style.font(assets.font.clone()),
                            TextColor(Color::srgb(0.794_354_86, 1.0, 1.0)),
                            style.layout(),
                            style,
                            UiTransform::from_translation(Val2::px(
                                0.0,
                                style.replacement_y_offset(),
                            )),
                            bank_taros_digit_localized("0"),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ));
                    });
            }
            panel.spawn((
                Button,
                BankUiElement::Close,
                BankCloseControl,
                BankUiRect::from(USER_EQUIP_CLOSE_RECT).node(),
                stretched_image(assets.image(BankStaticAssetRole::Close)),
            ));
            spawn_disabled_image(
                panel,
                BankUiElement::Trash,
                USER_EQUIP_TRASH_RECT,
                assets.image(BankStaticAssetRole::Trash),
                BankDisabledControl::Trash,
            );
            spawn_disabled_image(
                panel,
                BankUiElement::Help,
                USER_EQUIP_HELP_RECT,
                assets.image(BankStaticAssetRole::Help),
                BankDisabledControl::Help,
            );
        });
}

pub(super) fn spawn_inventory_slot(parent: &mut ChildSpawnerCommands, slot: usize, assets: &BankUiAssets) {
    let frame = BankUiRect::new(
        (slot % 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        (slot / 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
        USER_EQUIP_INVENTORY_SLOT_SIZE,
    );
    parent
        .spawn((
            BankUiElement::InventorySlotFrame(slot),
            BankSlotControl(BankSlotRef0104::new(BankSlotLocation0104::Inventory, slot).unwrap()),
            Button,
            frame.node(),
            stretched_image(assets.image(BankStaticAssetRole::SlotEmpty)),
            Pickable::default(),
        ))
        .with_children(|slot_node| {
            spawn_slot_children(
                slot_node,
                BankUiElement::InventorySlotIcon(slot),
                BankUiElement::InventorySlotBadge(slot),
                BankUiElement::InventorySlotCount(slot),
                USER_EQUIP_INVENTORY_SLOT_SIZE,
                USER_EQUIP_COUNT_LABEL_LEFT,
                USER_EQUIP_COUNT_LABEL_TOP,
                assets,
            );
        });
}

pub(super) fn spawn_slot_children(
    parent: &mut ChildSpawnerCommands,
    icon_marker: BankUiElement,
    badge_marker: BankUiElement,
    count_marker: BankUiElement,
    slot_size: f32,
    count_left: f32,
    count_top: f32,
    assets: &BankUiAssets,
) {
    parent.spawn((
        icon_marker,
        Node {
            display: Display::None,
            ..BankUiRect::new(0.0, 0.0, slot_size, slot_size).node()
        },
        ImageNode::default(),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    parent.spawn((
        badge_marker,
        Node {
            display: Display::None,
            ..BankUiRect::new(
                USER_EQUIP_COMBINED_BADGE_LEFT,
                USER_EQUIP_COMBINED_BADGE_TOP,
                USER_EQUIP_COMBINED_BADGE_SIZE,
                USER_EQUIP_COMBINED_BADGE_SIZE,
            )
            .node()
        },
        stretched_image(assets.image(BankStaticAssetRole::Combined)),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    let count_style = BankUiTextStyle::LabelUpperLeft;
    let mut count_node = BankUiRect::new(count_left, count_top, slot_size, slot_size).node();
    count_style.apply_to_container(&mut count_node);
    parent.spawn((
        count_marker,
        Node {
            display: Display::None,
            ..count_node
        },
        Text::new(""),
        count_style.font(assets.font.clone()),
        TextColor(Color::WHITE),
        count_style.layout(),
        count_style,
        UiTransform::from_translation(Val2::px(0.0, count_style.replacement_y_offset())),
        bank_count_localized(""),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
}

pub(super) fn spawn_equipment_panel(parent: &mut ChildSpawnerCommands, assets: &BankUiAssets) {
    parent
        .spawn((
            BankUiElement::EquipmentPanel,
            BankUiRect::new(0.0, 0.0, 66.0, 639.0).node(),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    BankUiElement::EquipmentTitle,
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..BankUiRect::from(USER_EQUIP_EQUIPMENT_TITLE_RECT).node()
                    },
                    stretched_image(assets.image(BankStaticAssetRole::EquipTitle)),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|title| {
                    let style = BankUiTextStyle::EquipBarMiddleCenter;
                    title.spawn((
                        BankUiElement::EquipmentTitleLabel,
                        Text::new(" Equipped"),
                        style.font(assets.font.clone()),
                        TextColor(Color::srgb(0.0, 0.2, 0.4)),
                        style.layout(),
                        style,
                        UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                        LocalizedText::new("ui.inventory.equipped", " Equipped"),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
            for visual_index in 0..USER_EQUIP_EQUIPMENT_STRIP_COUNT {
                spawn_equipment_slot(panel, visual_index, assets);
            }
        });
}

pub(super) fn spawn_equipment_slot(
    parent: &mut ChildSpawnerCommands,
    visual_index: usize,
    assets: &BankUiAssets,
) {
    let top = USER_EQUIP_EQUIPMENT_CONTENT_RECT.top
        + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE;
    parent
        .spawn((
            BankUiElement::EquipmentSlotFrame(visual_index),
            BankDisabledControl::EquipmentSlot(visual_index),
            BankUiRect::new(
                USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
                top,
                USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                USER_EQUIP_EQUIPMENT_SLOT_SIZE,
            )
            .node(),
            stretched_image(assets.image(BankStaticAssetRole::SlotEmpty)),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                BankUiElement::EquipmentSlotIcon(visual_index),
                Node {
                    display: Display::None,
                    ..BankUiRect::new(
                        0.0,
                        0.0,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                    )
                    .node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            slot_node.spawn((
                BankUiElement::EquipmentSlotBadge(visual_index),
                Node {
                    display: Display::None,
                    ..BankUiRect::new(
                        USER_EQUIP_COMBINED_BADGE_LEFT,
                        USER_EQUIP_COMBINED_BADGE_TOP,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                    )
                    .node()
                },
                stretched_image(assets.image(BankStaticAssetRole::Combined)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            let style = BankUiTextStyle::EquipFontMiddleRight;
            let mut label_node = BankUiRect::new(0.0, -2.0, 60.0, 19.0).node();
            style.apply_to_container(&mut label_node);
            slot_node
                .spawn((
                    BankUiElement::EquipmentSlotLabel(visual_index),
                    label_node,
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|label| {
                    label.spawn((
                        Text::new(equipment_slot_label(visual_index)),
                        style.font(assets.font.clone()),
                        TextColor(Color::WHITE),
                        style.layout(),
                        style,
                        UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                        bank_equipment_slot_localized(visual_index),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
        });
}

pub(super) fn spawn_text_label(
    parent: &mut ChildSpawnerCommands,
    marker: BankUiElement,
    rect: BankUiRect,
    value: &'static str,
    font: Handle<Font>,
) {
    let style = BankUiTextStyle::LabelUpperLeft;
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent
        .spawn((node, Pickable::IGNORE, bevy::ui::FocusPolicy::Pass))
        .with_children(|container| {
            container.spawn((
                marker,
                Node {
                    max_width: percent(100),
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new(value),
                style.font(font),
                TextColor(Color::WHITE),
                style.layout(),
                style,
                UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                bank_static_localized(value),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

pub(super) fn spawn_disabled_tab(
    parent: &mut ChildSpawnerCommands,
    marker: BankUiElement,
    rect: UserEquipUiRect,
    value: &'static str,
    font: Handle<Font>,
    disabled: BankDisabledControl,
) {
    let style = BankUiTextStyle::BlankBoxUpperLeft;
    let mut node = BankUiRect::from(rect).node();
    style.apply_to_container(&mut node);
    parent
        .spawn((node, Pickable::IGNORE, bevy::ui::FocusPolicy::Pass))
        .with_children(|container| {
            container.spawn((
                marker,
                Node {
                    max_width: percent(100),
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new(value),
                style.font(font),
                TextColor(Color::srgb(0.794_354_86, 1.0, 1.0)),
                style.layout(),
                style,
                UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                bank_static_localized(value),
                disabled,
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

pub(super) fn spawn_disabled_image(
    parent: &mut ChildSpawnerCommands,
    marker: BankUiElement,
    rect: UserEquipUiRect,
    image: Handle<Image>,
    disabled: BankDisabledControl,
) {
    let mut entity = parent.spawn((
        marker,
        BankUiRect::from(rect).node(),
        stretched_image(image),
        disabled,
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    if matches!(disabled, BankDisabledControl::Help) {
        entity.remove::<BankDisabledControl>().insert((
            BankHelpControl,
            Button,
            Pickable::default(),
            bevy::ui::FocusPolicy::Block,
        ));
    }
}
