use super::*;

pub(super) fn spawn_enchant_ui_0104(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = EnchantUiAssets0104::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            EnchantUiRoot0104,
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
            GlobalZIndex(ENCHANT_UI_Z_INDEX_0104),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                EnchantUiElement0104::Backdrop,
                EnchantUiRect0104::default().node(),
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Backdrop)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                EnchantUiElement0104::RightBackplate,
                EnchantUiRect0104::default().node(),
                enchant_sliced_image_0104(
                    assets.image(EnchantStaticAssetRole0104::RightBackplate),
                    ENCHANT_RIGHT_PANEL_BORDER_0104,
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_enchant_main_group_0104(root, &assets);
            spawn_enchant_pc_stuff_0104(root, &assets);
            spawn_enchant_equipment_0104(root, &assets);
            spawn_enchant_overlays_0104(root, &assets);
        });
}

pub(super) fn spawn_enchant_main_group_0104(parent: &mut ChildSpawnerCommands, assets: &EnchantUiAssets0104) {
    parent
        .spawn((
            EnchantUiElement0104::MainGroup,
            EnchantUiRect0104::default().node(),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|group| {
            group.spawn((
                EnchantUiElement0104::Panel,
                ENCHANT_PANEL_RECT.node(),
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Panel)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            group.spawn((
                EnchantUiElement0104::PrimaryNpcBoundary,
                crate::service_portrait::ServicePortraitSlot::EnchantPrimary,
                ImageNode::default(),
                ENCHANT_PRIMARY_NPC_RECT.node(),
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::Title,
                ENCHANT_TITLE_RECT,
                "ITEM ENCHANT",
                LocalizedText::new("ui.enchant.title", "ITEM ENCHANT"),
                EnchantUiTextStyle0104::Jeff12LightBlueUpperLeft,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                assets,
            );
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::Intro,
                ENCHANT_INTRO_RECT,
                "Select an item. We'll display your chance for a successful enchant.",
                LocalizedText::new(
                    "ui.enchant.intro",
                    "Select an item. We'll display your chance for a successful enchant.",
                ),
                EnchantUiTextStyle0104::Cha12YellowMiddleLeft,
                Color::srgb(1.0, 1.0, 0.0),
                assets,
            );
            for (element, rect, key, label) in [
                (
                    EnchantUiElement0104::TargetTitle,
                    ENCHANT_TARGET_TITLE_RECT,
                    "ui.enchant.section.target",
                    "Item to enchant",
                ),
                (
                    EnchantUiElement0104::NeededTitle,
                    ENCHANT_NEEDED_TITLE_RECT,
                    "ui.enchant.section.required",
                    "Items needed",
                ),
                (
                    EnchantUiElement0104::HelpTitle,
                    ENCHANT_HELP_TITLE_RECT,
                    "ui.enchant.section.helpers",
                    "Helping items",
                ),
            ] {
                spawn_enchant_text_0104(
                    group,
                    element,
                    rect,
                    label,
                    LocalizedText::new(key, label),
                    EnchantUiTextStyle0104::Cha12YellowMiddleLeft,
                    Color::srgb(1.0, 1.0, 0.0),
                    assets,
                );
            }
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::ChanceTitle,
                ENCHANT_CHANCE_TITLE_RECT,
                "CHANCE",
                LocalizedText::new("ui.enchant.chance.title", "CHANCE"),
                EnchantUiTextStyle0104::Jeff12LightBlueMiddleRight,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                assets,
            );
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::Chance,
                ENCHANT_CHANCE_RECT,
                "",
                enchant_chance_text_0104(EnchantChance0104::LegacyUnassigned),
                EnchantUiTextStyle0104::Cha12YellowMiddleRight,
                Color::srgb(1.0, 1.0, 0.0),
                assets,
            );
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::TarosTitle,
                ENCHANT_TAROS_TITLE_RECT,
                "TAROS",
                LocalizedText::new("ui.enchant.taros", "TAROS"),
                EnchantUiTextStyle0104::Jeff12LightBlueMiddleRight,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                assets,
            );
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::Taros,
                ENCHANT_TAROS_RECT,
                "",
                enchant_cost_text_0104(""),
                EnchantUiTextStyle0104::Cha12YellowMiddleRight,
                Color::srgb(1.0, 1.0, 0.0),
                assets,
            );

            for (slot, rect) in [
                (EnchantAttachmentSlot0104::Target, ENCHANT_TARGET_SLOT_RECT),
                (
                    EnchantAttachmentSlot0104::WeaponMaterial,
                    ENCHANT_WEAPON_SLOT_RECT,
                ),
                (
                    EnchantAttachmentSlot0104::ArmorMaterial,
                    ENCHANT_ARMOR_SLOT_RECT,
                ),
                (EnchantAttachmentSlot0104::Helper1, ENCHANT_HELP_1_SLOT_RECT),
                (EnchantAttachmentSlot0104::Helper2, ENCHANT_HELP_2_SLOT_RECT),
            ] {
                spawn_enchant_attachment_0104(group, slot, rect, assets);
            }

            group.spawn((
                EnchantUiElement0104::TargetCombinedBadge,
                Node {
                    display: Display::None,
                    ..ENCHANT_TARGET_COMBINED_BADGE_RECT_0104.node()
                },
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Combined)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            group
                .spawn((
                    EnchantUiElement0104::TargetLevelBadge,
                    Node {
                        display: Display::None,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..ENCHANT_TARGET_LEVEL_BADGE_RECT_0104.node()
                    },
                    enchant_stretched_image_0104(
                        assets.image(EnchantStaticAssetRole0104::LevelBadge),
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|badge| {
                    spawn_enchant_text_0104(
                        badge,
                        EnchantUiElement0104::TargetLevelText,
                        EnchantUiRect0104::new(0.0, 0.0, 25.0, 12.0),
                        "",
                        enchant_level_badge_text_0104(""),
                        EnchantUiTextStyle0104::EnchantLevelMiddleCenter,
                        Color::srgb(0.0, 0.229_927, 0.332_116_78),
                        assets,
                    );
                });

            for (element, rect, style, color) in [
                (
                    EnchantUiElement0104::TargetName,
                    ENCHANT_TARGET_NAME_RECT_0104,
                    EnchantUiTextStyle0104::Jeff8LightBlueUpperLeft,
                    Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                ),
                (
                    EnchantUiElement0104::TargetDescription,
                    ENCHANT_TARGET_DESCRIPTION_RECT_0104,
                    EnchantUiTextStyle0104::Cha10SkyBlueMiddleLeft,
                    Color::srgb(0.0, 1.0, 1.0),
                ),
            ] {
                spawn_enchant_text_0104(
                    group,
                    element,
                    rect,
                    "",
                    enchant_passthrough_text_0104(""),
                    style,
                    color,
                    assets,
                );
            }
            for (slot, name_rect, description_rect) in [
                (
                    EnchantAttachmentSlot0104::WeaponMaterial,
                    ENCHANT_WEAPON_NAME_RECT_0104,
                    ENCHANT_WEAPON_DESCRIPTION_RECT_0104,
                ),
                (
                    EnchantAttachmentSlot0104::ArmorMaterial,
                    ENCHANT_ARMOR_NAME_RECT_0104,
                    ENCHANT_ARMOR_DESCRIPTION_RECT_0104,
                ),
                (
                    EnchantAttachmentSlot0104::Helper1,
                    ENCHANT_HELP_1_NAME_RECT_0104,
                    ENCHANT_HELP_1_DESCRIPTION_RECT_0104,
                ),
                (
                    EnchantAttachmentSlot0104::Helper2,
                    ENCHANT_HELP_2_NAME_RECT_0104,
                    ENCHANT_HELP_2_DESCRIPTION_RECT_0104,
                ),
            ] {
                spawn_enchant_text_0104(
                    group,
                    EnchantUiElement0104::SupportName(slot),
                    name_rect,
                    "",
                    enchant_passthrough_text_0104(""),
                    EnchantUiTextStyle0104::Jeff8LightBlueUpperLeft,
                    Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                    assets,
                );
                spawn_enchant_text_0104(
                    group,
                    EnchantUiElement0104::SupportDescription(slot),
                    description_rect,
                    "",
                    enchant_passthrough_text_0104(""),
                    EnchantUiTextStyle0104::Cha10SkyBlueMiddleLeft,
                    Color::srgb(0.0, 1.0, 1.0),
                    assets,
                );
            }
            for (slot, rect) in [
                (
                    EnchantAttachmentSlot0104::WeaponMaterial,
                    EnchantUiRect0104::new(29.0, 303.0, 62.0, 62.0),
                ),
                (
                    EnchantAttachmentSlot0104::ArmorMaterial,
                    EnchantUiRect0104::new(274.0, 303.0, 62.0, 62.0),
                ),
            ] {
                group.spawn((
                    EnchantUiElement0104::MaterialXMark(slot),
                    Node {
                        display: Display::None,
                        ..rect.node()
                    },
                    ImageNode {
                        color: Color::srgb(0.3, 0.3, 0.3),
                        ..enchant_stretched_image_0104(
                            assets.image(EnchantStaticAssetRole0104::XMark),
                        )
                    },
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
            }
            for (slot, cover_rect, warning_rect, required_rect) in [
                (
                    EnchantAttachmentSlot0104::WeaponMaterial,
                    ENCHANT_WEAPON_QUANTITY_COVER_RECT_0104,
                    ENCHANT_WEAPON_QUANTITY_WARNING_RECT_0104,
                    EnchantUiRect0104::new(28.0, 302.0, 64.0, 64.0),
                ),
                (
                    EnchantAttachmentSlot0104::ArmorMaterial,
                    ENCHANT_ARMOR_QUANTITY_COVER_RECT_0104,
                    ENCHANT_ARMOR_QUANTITY_WARNING_RECT_0104,
                    EnchantUiRect0104::new(273.0, 302.0, 64.0, 64.0),
                ),
            ] {
                group.spawn((
                    EnchantUiElement0104::MaterialQuantityCover(slot),
                    Node {
                        display: Display::None,
                        ..cover_rect.node()
                    },
                    enchant_stretched_image_0104(
                        assets.image(EnchantStaticAssetRole0104::QuantityCover),
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
                spawn_enchant_text_0104(
                    group,
                    EnchantUiElement0104::MaterialQuantityWarning(slot),
                    warning_rect,
                    "Item is not enough. Please check again.",
                    LocalizedText::new(
                        "ui.enchant.material.insufficient",
                        "Item is not enough. Please check again.",
                    ),
                    EnchantUiTextStyle0104::Cha10BlueUpperLeft,
                    Color::srgb(0.0, 0.153, 0.239),
                    assets,
                );
                spawn_enchant_text_0104(
                    group,
                    EnchantUiElement0104::MaterialRequiredCount(slot),
                    required_rect,
                    "",
                    enchant_quantity_text_0104(""),
                    EnchantUiTextStyle0104::Jeff40LightBlueMiddleCenter,
                    Color::srgba(0.741_176_5, 0.905_882_36, 1.0, 0.53),
                    assets,
                );
            }
            group.spawn((
                EnchantUiElement0104::RawCover,
                Node {
                    display: Display::None,
                    ..ENCHANT_RAW_COVER_RECT.node()
                },
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::RawCover)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_enchant_text_0104(
                group,
                EnchantUiElement0104::EmptyPrompt,
                ENCHANT_EMPTY_PROMPT_RECT,
                "Please Select Item to Enchant First.",
                LocalizedText::new(
                    "ui.enchant.empty_prompt",
                    "Please Select Item to Enchant First.",
                ),
                EnchantUiTextStyle0104::Char12BlueMiddleCenter,
                Color::srgb(0.0, 0.153, 0.239),
                assets,
            );
            spawn_enchant_button_0104(
                group,
                EnchantUiElement0104::Preview,
                EnchantInteractiveControl0104::Preview,
                ENCHANT_PREVIEW_RECT,
                "ui.enchant.preview",
                "PREVIEW",
                EnchantUiTextStyle0104::EnchantDefaultButtonMiddleCenter,
                assets,
            );
            spawn_enchant_button_0104(
                group,
                EnchantUiElement0104::Clear,
                EnchantInteractiveControl0104::Clear,
                ENCHANT_CLEAR_RECT,
                "ui.enchant.clear",
                "CLEAR",
                EnchantUiTextStyle0104::EnchantDefaultButtonMiddleCenter,
                assets,
            );
            spawn_enchant_button_0104(
                group,
                EnchantUiElement0104::Enchant,
                EnchantInteractiveControl0104::Enchant,
                ENCHANT_ACTION_RECT,
                "ui.enchant.action",
                "ENCHANT",
                EnchantUiTextStyle0104::EnchantDefaultButtonMiddleCenter,
                assets,
            );
        });
}

pub(super) fn spawn_enchant_attachment_0104(
    parent: &mut ChildSpawnerCommands,
    slot: EnchantAttachmentSlot0104,
    rect: EnchantUiRect0104,
    assets: &EnchantUiAssets0104,
) {
    parent
        .spawn((
            Button,
            EnchantUiElement0104::AttachmentFrame(slot),
            EnchantInteractiveControl0104::Attachment(slot),
            rect.node(),
            enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::SlotEmpty)),
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                EnchantUiElement0104::AttachmentIcon(slot),
                Node {
                    display: Display::None,
                    ..EnchantUiRect0104::new(0.0, 0.0, 64.0, 64.0).node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            if matches!(
                slot,
                EnchantAttachmentSlot0104::WeaponMaterial
                    | EnchantAttachmentSlot0104::ArmorMaterial
            ) {
                spawn_enchant_text_0104(
                    slot_node,
                    EnchantUiElement0104::AttachmentCount(slot),
                    EnchantUiRect0104::new(5.0, 5.0, 64.0, 64.0),
                    "",
                    enchant_quantity_text_0104(""),
                    EnchantUiTextStyle0104::EnchantDefaultLabelUpperLeft,
                    Color::WHITE,
                    assets,
                );
            }
        });
}

pub(super) fn spawn_enchant_pc_stuff_0104(parent: &mut ChildSpawnerCommands, assets: &EnchantUiAssets0104) {
    parent
        .spawn((
            EnchantUiElement0104::PcStuffPanel,
            ENCHANT_INVENTORY_PANEL_RECT_0104.node(),
            enchant_sliced_image_0104(
                assets.image(EnchantStaticAssetRole0104::InventoryPanel),
                ENCHANT_INVENTORY_PANEL_BORDER_0104,
            ),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            spawn_enchant_text_0104(
                panel,
                EnchantUiElement0104::ItemTabLabel,
                EnchantUiRect0104::new(10.0, 5.0, 105.0, 15.0),
                "EQUIPMENT",
                LocalizedText::new("ui.enchant.inventory.equipment", "EQUIPMENT"),
                EnchantUiTextStyle0104::InventoryBlankBoxUpperLeft,
                Color::srgb(0.794_354_86, 1.0, 1.0),
                assets,
            );
            panel
                .spawn((
                    EnchantUiElement0104::InventoryViewport,
                    Node {
                        overflow: Overflow::clip(),
                        ..ENCHANT_INVENTORY_VIEWPORT_RECT_0104.node()
                    },
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            EnchantUiElement0104::InventoryContent,
                            EnchantUiRect0104::new(0.0, 0.0, 345.0, 690.0).node(),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ))
                        .with_children(|content| {
                            for slot in 0..ENCHANT_INVENTORY_SLOT_COUNT_0104 {
                                let rect = EnchantUiRect0104::new(
                                    (slot % ENCHANT_INVENTORY_COLUMNS_0104) as f32
                                        * ENCHANT_INVENTORY_SLOT_STRIDE_0104,
                                    (slot / ENCHANT_INVENTORY_COLUMNS_0104) as f32
                                        * ENCHANT_INVENTORY_SLOT_STRIDE_0104,
                                    ENCHANT_INVENTORY_SLOT_SIZE_0104,
                                    ENCHANT_INVENTORY_SLOT_SIZE_0104,
                                );
                                content
                                    .spawn((
                                        Button,
                                        EnchantUiElement0104::InventorySlotFrame(slot),
                                        EnchantInteractiveControl0104::InventorySlot(slot),
                                        rect.node(),
                                        enchant_stretched_image_0104(
                                            assets.image(EnchantStaticAssetRole0104::SlotEmpty),
                                        ),
                                    ))
                                    .with_children(|slot_node| {
                                        spawn_enchant_shared_slot_children_0104(
                                            slot_node,
                                            EnchantUiElement0104::InventorySlotIcon(slot),
                                            EnchantUiElement0104::InventorySlotBadge(slot),
                                            Some(EnchantUiElement0104::InventorySlotCount(slot)),
                                            ENCHANT_INVENTORY_SLOT_SIZE_0104,
                                            assets,
                                        );
                                    });
                            }
                        });
                });
            panel.spawn((
                EnchantUiElement0104::DexlabsBanner,
                ENCHANT_DEXLABS_RECT_0104.node(),
                enchant_stretched_image_0104(
                    assets.image(EnchantStaticAssetRole0104::DexlabsBanner),
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            panel.spawn((
                EnchantUiElement0104::TarosCounter,
                ENCHANT_TAROS_COUNTER_RECT_0104.node(),
                enchant_stretched_image_0104(
                    assets.image(EnchantStaticAssetRole0104::TarosCounter),
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            for (index, rect) in ENCHANT_TAROS_DIGIT_RECTS_0104.into_iter().enumerate() {
                spawn_enchant_text_0104(
                    panel,
                    EnchantUiElement0104::TarosDigit(index),
                    rect,
                    "0",
                    enchant_counter_digit_text_0104("0"),
                    EnchantUiTextStyle0104::InventoryBlankBoxMiddleRight,
                    Color::srgb(0.794_354_86, 1.0, 1.0),
                    assets,
                );
            }
            spawn_enchant_button_0104(
                panel,
                EnchantUiElement0104::RedeemCode,
                EnchantInteractiveControl0104::RedeemCode,
                ENCHANT_REDEEM_CODE_RECT_0104,
                "ui.enchant.redeem_code",
                "REDEEM CODE",
                EnchantUiTextStyle0104::InventoryButtonMiddleCenter,
                assets,
            );
            panel.spawn((
                Button,
                EnchantUiElement0104::Close,
                EnchantInteractiveControl0104::Close,
                ENCHANT_CLOSE_RECT_0104.node(),
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Close)),
            ));
            panel.spawn((
                Button,
                EnchantUiElement0104::Trash,
                EnchantInteractiveControl0104::Trash,
                ENCHANT_TRASH_RECT_0104.node(),
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Trash)),
            ));
            panel.spawn((
                Button,
                EnchantUiElement0104::Help,
                EnchantInteractiveControl0104::Help,
                ENCHANT_HELP_RECT_0104.node(),
                enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Help)),
            ));
        });
}

pub(super) fn spawn_enchant_equipment_0104(parent: &mut ChildSpawnerCommands, assets: &EnchantUiAssets0104) {
    parent
        .spawn((
            EnchantUiElement0104::EquipmentPanel,
            ENCHANT_EQUIPMENT_LOCAL_RECT_0104.node(),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    EnchantUiElement0104::EquipmentTitle,
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..ENCHANT_EQUIPMENT_TITLE_RECT_0104.node()
                    },
                    enchant_stretched_image_0104(
                        assets.image(EnchantStaticAssetRole0104::EquipTitle),
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|title| {
                    spawn_enchant_text_0104(
                        title,
                        EnchantUiElement0104::EquipmentTitleLabel,
                        EnchantUiRect0104::new(0.0, 0.0, 64.0, 13.0),
                        " Equipped",
                        LocalizedText::new("ui.enchant.equipment.title", " Equipped"),
                        EnchantUiTextStyle0104::InventoryEquipBarMiddleCenter,
                        Color::srgb(0.0, 0.2, 0.4),
                        assets,
                    );
                });
            for visual_index in 0..ENCHANT_EQUIPMENT_SLOT_COUNT_0104 {
                let rect = EnchantUiRect0104::new(
                    0.0,
                    14.0 + visual_index as f32 * ENCHANT_EQUIPMENT_SLOT_STRIDE_0104,
                    ENCHANT_EQUIPMENT_SLOT_SIZE_0104,
                    ENCHANT_EQUIPMENT_SLOT_SIZE_0104,
                );
                panel
                    .spawn((
                        EnchantUiElement0104::EquipmentSlotFrame(visual_index),
                        rect.node(),
                        enchant_stretched_image_0104(
                            assets.image(EnchantStaticAssetRole0104::SlotEmpty),
                        ),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ))
                    .with_children(|slot_node| {
                        spawn_enchant_shared_slot_children_0104(
                            slot_node,
                            EnchantUiElement0104::EquipmentSlotIcon(visual_index),
                            EnchantUiElement0104::EquipmentSlotBadge(visual_index),
                            None,
                            ENCHANT_EQUIPMENT_SLOT_SIZE_0104,
                            assets,
                        );
                        spawn_enchant_text_0104(
                            slot_node,
                            EnchantUiElement0104::EquipmentSlotLabel(visual_index),
                            EnchantUiRect0104::new(0.0, -2.0, 60.0, 19.0),
                            ENCHANT_EQUIPMENT_LABELS_0104[visual_index],
                            enchant_equipment_slot_text_0104(visual_index),
                            EnchantUiTextStyle0104::InventoryEquipFontMiddleRight,
                            Color::WHITE,
                            assets,
                        );
                    });
            }
            for battery_index in 0..2 {
                panel.spawn((
                    EnchantUiElement0104::BatterySlotFrame(battery_index),
                    ENCHANT_BATTERY_SLOT_RECTS_0104[battery_index].node(),
                    enchant_stretched_image_0104(
                        assets.image(EnchantStaticAssetRole0104::SlotEmpty),
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
                panel.spawn((
                    EnchantUiElement0104::BatteryIcon(battery_index),
                    ENCHANT_BATTERY_ICON_RECTS_0104[battery_index].node(),
                    enchant_stretched_image_0104(assets.image(if battery_index == 0 {
                        EnchantStaticAssetRole0104::BoostIcon
                    } else {
                        EnchantStaticAssetRole0104::PotionIcon
                    })),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
                spawn_enchant_text_0104(
                    panel,
                    EnchantUiElement0104::BatteryLabel(battery_index),
                    ENCHANT_BATTERY_LABEL_RECTS_0104[battery_index],
                    if battery_index == 0 {
                        "BOOSTS"
                    } else {
                        "POTIONS"
                    },
                    if battery_index == 0 {
                        LocalizedText::new("ui.enchant.battery.boosts", "BOOSTS")
                    } else {
                        LocalizedText::new("ui.enchant.battery.potions", "POTIONS")
                    },
                    EnchantUiTextStyle0104::InventoryEquipFontMiddleRight,
                    Color::srgba(1.0, 1.0, 1.0, 0.8),
                    assets,
                );
                spawn_enchant_text_0104(
                    panel,
                    EnchantUiElement0104::BatteryCount(battery_index),
                    ENCHANT_BATTERY_COUNT_RECTS_0104[battery_index],
                    "0",
                    enchant_battery_count_text_0104("0"),
                    EnchantUiTextStyle0104::InventoryLabelUpperLeft,
                    Color::WHITE,
                    assets,
                );
            }
        });
}

pub(super) fn spawn_enchant_shared_slot_children_0104(
    parent: &mut ChildSpawnerCommands,
    icon: EnchantUiElement0104,
    badge: EnchantUiElement0104,
    count: Option<EnchantUiElement0104>,
    size: f32,
    assets: &EnchantUiAssets0104,
) {
    parent.spawn((
        icon,
        Node {
            display: Display::None,
            ..EnchantUiRect0104::new(0.0, 0.0, size, size).node()
        },
        ImageNode::default(),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    parent.spawn((
        badge,
        Node {
            display: Display::None,
            ..EnchantUiRect0104::new(36.0, 36.0, 26.0, 26.0).node()
        },
        enchant_stretched_image_0104(assets.image(EnchantStaticAssetRole0104::Combined)),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    if let Some(count) = count {
        spawn_enchant_text_0104(
            parent,
            count,
            EnchantUiRect0104::new(5.0, 5.0, size - 10.0, size - 10.0),
            "",
            enchant_quantity_text_0104(""),
            EnchantUiTextStyle0104::InventoryLabelUpperLeft,
            Color::WHITE,
            assets,
        );
    }
}
