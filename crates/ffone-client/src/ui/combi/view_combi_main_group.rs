use super::*;

pub(super) fn spawn_combi_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    contract: Res<CombiUiAssetContract>,
) {
    let assets = CombiUiAssets::load(&asset_server, &contract);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            CombiUiRoot0104,
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
            GlobalZIndex(COMBI_UI_Z_INDEX),
            // Modal: nothing beneath the Croc Pot takes pointer focus.
            Pickable::IGNORE,
            FocusPolicy::Block,
        ))
        .with_children(|root| {
            root.spawn((
                CombiUiElement0104::Backdrop,
                CombiUiRect::default().node(),
                stretched_image(assets.image(CombiStaticAssetRole::Backdrop)),
                Pickable::IGNORE,
                FocusPolicy::Block,
            ));
            root.spawn((
                CombiUiElement0104::Panel,
                CombiUiRect::default().node(),
                stretched_image(assets.image(CombiStaticAssetRole::Panel)),
                combi_passive(),
            ));
            root.spawn((
                CombiUiElement0104::RightBackplate,
                CombiUiRect::default().node(),
                sliced_image(
                    assets.image(CombiStaticAssetRole::RightBackplate),
                    USER_EQUIP_RIGHT_PANEL_BORDER,
                ),
                combi_passive(),
            ));
            spawn_combi_main_group(root, &assets);
            spawn_combi_pc_stuff(root, &assets);
            spawn_combi_equipment(root, &assets);
            spawn_combi_overlays(root, &assets);
            root.spawn((
                CombiUiElement0104::DraggedItem,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    width: px(USER_EQUIP_INVENTORY_SLOT_SIZE),
                    height: px(USER_EQUIP_INVENTORY_SLOT_SIZE),
                    ..default()
                },
                ImageNode::default(),
                GlobalZIndex(COMBI_UI_Z_INDEX + 3),
                combi_passive(),
            ));
        });
}

pub(super) fn spawn_combi_main_group(parent: &mut ChildSpawnerCommands, assets: &CombiUiAssets) {
    parent
        .spawn((
            CombiUiElement0104::MainGroup,
            CombiUiRect::default().node(),
            combi_passive(),
        ))
        .with_children(|group| {
            // Exact external `AvatarUtil.DrawCamera` boundary. It remains
            // transparent unless a native camera owner is explicitly bound.
            group.spawn((
                CombiUiElement0104::PrimaryNpcBoundary,
                crate::service_portrait::ServicePortraitSlot::CombiPrimary,
                ImageNode::default(),
                COMBI_PRIMARY_NPC_PREVIEW_RECT.node(),
                BackgroundColor(Color::NONE),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::Title,
                COMBI_TITLE_RECT,
                LocalizedText::new("ui.combi.title", "Croc Pot Catering Company"),
                assets.jeffe_font.clone(),
                12.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::Intro,
                COMBI_INTRO_RECT,
                LocalizedText::new(
                    "ui.combi.intro",
                    "Select one item for the STYLE slot and one item for the STATS slot. We'll display your chance for a successful combination.",
                ),
                assets.chalet_font.clone(),
                12.0,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::StyleTitle,
                COMBI_STYLE_TITLE_RECT,
                LocalizedText::new("ui.combi.style", "STYLE"),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::StatsTitle,
                COMBI_STATS_TITLE_RECT,
                LocalizedText::new("ui.combi.stats", "STATS"),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::Cost,
                COMBI_COST_RECT,
                combi_cost_text(0),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::TarosLabel,
                COMBI_TAROS_LABEL_RECT,
                LocalizedText::new("ui.combi.taros", "TAROS"),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::NoWrap,
            );
            group.spawn((
                CombiUiElement0104::TarosIcon,
                COMBI_TAROS_ICON_RECT.node(),
                stretched_image(assets.image(CombiStaticAssetRole::Taros)),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::ChanceTitle,
                COMBI_CHANCE_TITLE_RECT,
                LocalizedText::new("ui.combi.chance.title", "CHANCE OF SUCCESS:"),
                assets.jeffe_font.clone(),
                6.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::Chance,
                COMBI_CHANCE_RECT,
                CombiChance0104::NotReady.localized_text(),
                assets.jeffe_font.clone(),
                8.0,
                Color::WHITE,
                Justify::Center,
                LineBreak::WordBoundary,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::ChanceLevel,
                COMBI_CHANCE_LEVEL_RECT,
                combi_chance_level_text("?"),
                assets.jeffe_font.clone(),
                24.0,
                Color::WHITE,
                Justify::Center,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::NewItemTitle,
                COMBI_NEW_ITEM_TITLE_RECT,
                LocalizedText::new("ui.combi.new_combined_item", "NEW COMBINED ITEM"),
                assets.jeffe_font.clone(),
                6.0,
                Color::srgb(0.0, 0.153, 0.239),
                Justify::Center,
                LineBreak::NoWrap,
            );

            group.spawn((
                Button,
                CombiUiElement0104::LookDrop,
                CombiInteractiveControl0104::LookDrop,
                COMBI_LOOK_DROP_RECT.node(),
                BackgroundColor(Color::NONE),
            ));
            group.spawn((
                Button,
                CombiUiElement0104::StatDrop,
                CombiInteractiveControl0104::StatsDrop,
                COMBI_STAT_DROP_RECT.node(),
                BackgroundColor(Color::NONE),
            ));
            group.spawn((
                CombiUiElement0104::LookBackground,
                Node {
                    display: Display::None,
                    ..COMBI_LOOK_DROP_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::LookItemBg)),
                combi_passive(),
            ));
            spawn_combi_selection_slot(
                group,
                CombiSelectionSlot0104::Style,
                COMBI_LOOK_SELECTED_SLOT_RECT,
                CombiUiElement0104::LookSelectionFrame,
                CombiUiElement0104::LookSelectionIcon,
                CombiUiElement0104::LookSelectionBadge,
                CombiUiElement0104::LookSelectionHover,
                CombiInteractiveControl0104::LookSelection,
                assets,
            );
            group.spawn((
                CombiUiElement0104::LookDetailRestricted,
                Node {
                    display: Display::None,
                    ..COMBI_LOOK_ICON_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::Restricted)),
                combi_passive(),
            ));
            group.spawn((
                CombiUiElement0104::LookDetailIcon,
                Node {
                    display: Display::None,
                    ..COMBI_LOOK_ICON_RECT.node()
                },
                ImageNode::default(),
                combi_passive(),
            ));
            group.spawn((
                CombiUiElement0104::LookDetailBadge,
                Node {
                    display: Display::None,
                    ..COMBI_LOOK_BADGE_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::Combined)),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::LookName,
                COMBI_LOOK_NAME_RECT,
                combi_passthrough_text(""),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::LookLevel,
                COMBI_LOOK_LEVEL_RECT,
                combi_item_level_text(0),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgba(1.0, 1.0, 0.0, 0.3),
                Justify::Left,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::LookDescription,
                COMBI_LOOK_DESCRIPTION_RECT,
                combi_passthrough_text(""),
                assets.chalet_font.clone(),
                10.0,
                Color::srgb(0.0, 1.0, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            group.spawn((
                CombiUiElement0104::LookErrorBackground,
                Node {
                    display: Display::None,
                    ..COMBI_LOOK_ERROR_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::LookError)),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::LookErrorText,
                CombiUiRect::new(215.0, 221.0, 202.0, 24.0),
                combi_passthrough_text(""),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::LookEmptyText,
                COMBI_LOOK_EMPTY_TEXT_RECT,
                LocalizedText::new(
                    "ui.combi.look.empty",
                    "Place an item here to choose what your new combined item will LOOK like.",
                ),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );

            group.spawn((
                CombiUiElement0104::StatBackground,
                Node {
                    display: Display::None,
                    ..COMBI_STAT_DROP_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::StatItemBg)),
                combi_passive(),
            ));
            spawn_combi_selection_slot(
                group,
                CombiSelectionSlot0104::Stats,
                COMBI_STAT_SELECTED_SLOT_RECT,
                CombiUiElement0104::StatSelectionFrame,
                CombiUiElement0104::StatSelectionIcon,
                CombiUiElement0104::StatSelectionBadge,
                CombiUiElement0104::StatSelectionHover,
                CombiInteractiveControl0104::StatsSelection,
                assets,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::StatLevel,
                COMBI_STAT_LEVEL_RECT,
                combi_item_level_text(0),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Center,
                LineBreak::NoWrap,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::StatSection,
                COMBI_STAT_SECTION_RECT,
                LocalizedText::new("ui.combi.stats", "STATS"),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            for (element, rect) in [
                (CombiUiElement0104::StatSingle, COMBI_STAT_SINGLE_RECT),
                (CombiUiElement0104::StatMulti, COMBI_STAT_MULTI_RECT),
                (
                    CombiUiElement0104::StatDefense,
                    COMBI_STAT_DEFENSE_RECT,
                ),
            ] {
                spawn_combi_text(
                    group,
                    element,
                    rect,
                    combi_stat_value_text(0),
                    assets.jeffe_font.clone(),
                    8.0,
                    COMBI_COLOR_DEFAULT.bevy(),
                    Justify::Center,
                    LineBreak::NoWrap,
                );
            }
            spawn_combi_text(
                group,
                CombiUiElement0104::InfoSection,
                COMBI_INFO_SECTION_RECT,
                LocalizedText::new("ui.combi.info", "INFO"),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Left,
                LineBreak::NoWrap,
            );
            for (element, rect, key, label) in [
                (
                    CombiUiElement0104::InfoTypeLabel,
                    CombiUiRect::new(274.0, 470.0, 27.0, 14.0),
                    "ui.combi.info.type",
                    "Type",
                ),
                (
                    CombiUiElement0104::InfoRangeLabel,
                    CombiUiRect::new(267.0, 491.0, 34.0, 14.0),
                    "ui.combi.info.range",
                    "Range",
                ),
                (
                    CombiUiElement0104::InfoRarityLabel,
                    CombiUiRect::new(269.0, 512.0, 32.0, 14.0),
                    "ui.combi.info.rarity",
                    "Rarity",
                ),
                (
                    CombiUiElement0104::InfoTradeLabel,
                    CombiUiRect::new(213.0, 532.0, 88.0, 14.0),
                    "ui.combi.info.trade_availability",
                    "Trade Availability",
                ),
            ] {
                spawn_combi_text(
                    group,
                    element,
                    rect,
                    LocalizedText::new(key, label),
                    assets.chalet_font.clone(),
                    10.0,
                    Color::srgb(0.0, 1.0, 1.0),
                    Justify::Center,
                    LineBreak::NoWrap,
                );
            }
            for (element, top) in [
                (CombiUiElement0104::InfoTypeValue, 470.0),
                (CombiUiElement0104::InfoRangeValue, 491.0),
                (CombiUiElement0104::InfoRarityValue, 512.0),
                (CombiUiElement0104::InfoTradeValue, 532.0),
            ] {
                spawn_combi_text(
                    group,
                    element,
                    CombiUiRect::new(335.0, top, 115.0, 14.0),
                    combi_passthrough_text(""),
                    assets.chalet_font.clone(),
                    10.0,
                    Color::srgb(0.8, 1.0, 1.0),
                    Justify::Center,
                    LineBreak::NoWrap,
                );
            }
            group.spawn((
                CombiUiElement0104::StatErrorBackground,
                Node {
                    display: Display::None,
                    ..COMBI_STAT_ERROR_RECT.node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::StatError)),
                combi_passive(),
            ));
            spawn_combi_text(
                group,
                CombiUiElement0104::StatErrorText,
                CombiUiRect::new(215.0, 423.0, 202.0, 24.0),
                combi_passthrough_text(""),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            spawn_combi_text(
                group,
                CombiUiElement0104::StatEmptyText,
                COMBI_STAT_EMPTY_TEXT_RECT,
                LocalizedText::new(
                    "ui.combi.stats.empty",
                    "Place an item here to choose your new combined item's STATS and LEVEL.",
                ),
                assets.jeffe_font.clone(),
                8.0,
                Color::srgb(0.741_176_5, 0.905_882_36, 1.0),
                Justify::Center,
                LineBreak::WordBoundary,
            );
            spawn_combi_button(
                group,
                CombiUiElement0104::ClearAll,
                CombiInteractiveControl0104::ClearAll,
                COMBI_CLEAR_ALL_RECT,
                LocalizedText::new("ui.combi.clear_all", "CLEAR ALL"),
                assets,
            );
            spawn_combi_button(
                group,
                CombiUiElement0104::Combine,
                CombiInteractiveControl0104::Combine,
                COMBI_COMBINE_RECT,
                LocalizedText::new("ui.combi.combine", "COMBINE"),
                assets,
            );
            // Last, so the carried item's target reads above the item details.
            spawn_combi_hover_frame(
                group,
                CombiUiElement0104::LookDropHighlight,
                COMBI_LOOK_DROP_RECT,
            );
            spawn_combi_hover_frame(
                group,
                CombiUiElement0104::StatDropHighlight,
                COMBI_STAT_DROP_RECT,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_combi_selection_slot(
    parent: &mut ChildSpawnerCommands,
    slot: CombiSelectionSlot0104,
    rect: CombiUiRect,
    frame_marker: CombiUiElement0104,
    icon_marker: CombiUiElement0104,
    badge_marker: CombiUiElement0104,
    hover_marker: CombiUiElement0104,
    control: CombiInteractiveControl0104,
    assets: &CombiUiAssets,
) {
    parent
        .spawn((
            Button,
            frame_marker,
            control,
            rect.node(),
            stretched_image(assets.image(CombiStaticAssetRole::SlotEmpty)),
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                icon_marker,
                Node {
                    display: Display::None,
                    ..CombiUiRect::new(0.0, 0.0, rect.width, rect.height).node()
                },
                ImageNode::default(),
                combi_passive(),
            ));
            slot_node.spawn((
                badge_marker,
                Node {
                    display: Display::None,
                    ..CombiUiRect::new(
                        if slot == CombiSelectionSlot0104::Style {
                            38.0
                        } else {
                            38.0
                        },
                        36.0,
                        26.0,
                        26.0,
                    )
                    .node()
                },
                stretched_image(assets.image(CombiStaticAssetRole::Combined)),
                combi_passive(),
            ));
            spawn_combi_hover_frame(
                slot_node,
                hover_marker,
                CombiUiRect::new(0.0, 0.0, rect.width, rect.height),
            );
        });
}

pub(super) fn spawn_combi_pc_stuff(parent: &mut ChildSpawnerCommands, assets: &CombiUiAssets) {
    parent
        .spawn((
            CombiUiElement0104::PcStuffPanel,
            CombiUiRect::from(USER_EQUIP_PC_STUFF_RECT).node(),
            sliced_image(
                assets.image(CombiStaticAssetRole::InventoryPanel),
                USER_EQUIP_INVENTORY_PANEL_BORDER,
            ),
            combi_passive(),
        ))
        .with_children(|panel| {
            spawn_combi_text(
                panel,
                CombiUiElement0104::ItemTabLabel,
                USER_EQUIP_ITEM_TAB_HIT_RECT.into(),
                LocalizedText::new("ui.combi.inventory.equipment", "EQUIPMENT"),
                assets.jeffe_font.clone(),
                12.0,
                Color::srgb(0.794_354_86, 1.0, 1.0),
                Justify::Center,
                LineBreak::NoWrap,
            );
            panel
                .spawn((
                    CombiUiElement0104::InventoryViewport,
                    Node {
                        overflow: Overflow::clip(),
                        ..CombiUiRect::from(USER_EQUIP_INVENTORY_VIEWPORT_RECT).node()
                    },
                    combi_passive(),
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            CombiUiElement0104::InventoryContent,
                            CombiUiRect::new(
                                0.0,
                                0.0,
                                USER_EQUIP_INVENTORY_CONTENT_WIDTH,
                                USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
                            )
                            .node(),
                            combi_passive(),
                        ))
                        .with_children(|content| {
                            for slot in 0..INVENTORY_SLOT_COUNT_0104 {
                                let rect = CombiUiRect::new(
                                    (slot % 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
                                    (slot / 5) as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
                                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                                );
                                content
                                    .spawn((
                                        Button,
                                        CombiUiElement0104::InventorySlotFrame(slot),
                                        CombiInteractiveControl0104::InventorySlot(slot),
                                        rect.node(),
                                        stretched_image(
                                            assets.image(CombiStaticAssetRole::SlotEmpty),
                                        ),
                                    ))
                                    .with_children(|slot_node| {
                                        spawn_shared_slot_children(
                                            slot_node,
                                            CombiUiElement0104::InventorySlotIcon(slot),
                                            CombiUiElement0104::InventorySlotBadge(slot),
                                            CombiUiElement0104::InventorySlotHover(slot),
                                            USER_EQUIP_INVENTORY_SLOT_SIZE,
                                            assets,
                                        );
                                    });
                            }
                        });
                });
            panel.spawn((
                Button,
                CombiUiElement0104::Close,
                CombiInteractiveControl0104::Close,
                CombiUiRect::from(USER_EQUIP_CLOSE_RECT).node(),
                stretched_image(assets.image(CombiStaticAssetRole::Close)),
            ));
            panel.spawn((
                CombiUiElement0104::Trash,
                CombiUiRect::from(USER_EQUIP_TRASH_RECT).node(),
                stretched_image(assets.image(CombiStaticAssetRole::Trash)),
                combi_passive(),
            ));
            panel.spawn((
                Button,
                CombiUiElement0104::Help,
                CombiInteractiveControl0104::Help,
                CombiUiRect::from(USER_EQUIP_HELP_RECT).node(),
                stretched_image(assets.image(CombiStaticAssetRole::Help)),
            ));
        });
}

pub(super) fn spawn_combi_equipment(parent: &mut ChildSpawnerCommands, assets: &CombiUiAssets) {
    parent
        .spawn((
            CombiUiElement0104::EquipmentPanel,
            CombiUiRect::from(USER_EQUIP_EQUIP_STRIP_RECT).node(),
            combi_passive(),
        ))
        .with_children(|panel| {
            panel.spawn((
                CombiUiElement0104::EquipmentTitle,
                CombiUiRect::from(USER_EQUIP_EQUIPMENT_TITLE_RECT).node(),
                stretched_image(assets.image(CombiStaticAssetRole::EquipTitle)),
                combi_passive(),
            ));
            for visual_index in 0..USER_EQUIP_EQUIPMENT_STRIP_COUNT {
                let top = USER_EQUIP_EQUIPMENT_CONTENT_RECT.top
                    + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE;
                panel
                    .spawn((
                        Button,
                        CombiUiElement0104::EquipmentSlotFrame(visual_index),
                        CombiInteractiveControl0104::EquipmentSlot(visual_index),
                        CombiUiRect::new(
                            USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
                            top,
                            USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                            USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                        )
                        .node(),
                        stretched_image(assets.image(CombiStaticAssetRole::SlotEmpty)),
                    ))
                    .with_children(|slot_node| {
                        spawn_shared_slot_children(
                            slot_node,
                            CombiUiElement0104::EquipmentSlotIcon(visual_index),
                            CombiUiElement0104::EquipmentSlotBadge(visual_index),
                            CombiUiElement0104::EquipmentSlotHover(visual_index),
                            USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                            assets,
                        );
                    });
            }
        });
}

pub(super) fn spawn_shared_slot_children(
    parent: &mut ChildSpawnerCommands,
    icon_marker: CombiUiElement0104,
    badge_marker: CombiUiElement0104,
    hover_marker: CombiUiElement0104,
    slot_size: f32,
    assets: &CombiUiAssets,
) {
    parent.spawn((
        icon_marker,
        Node {
            display: Display::None,
            ..CombiUiRect::new(0.0, 0.0, slot_size, slot_size).node()
        },
        ImageNode::default(),
        combi_passive(),
    ));
    parent.spawn((
        badge_marker,
        Node {
            display: Display::None,
            ..CombiUiRect::new(
                USER_EQUIP_COMBINED_BADGE_LEFT,
                USER_EQUIP_COMBINED_BADGE_TOP,
                USER_EQUIP_COMBINED_BADGE_SIZE,
                USER_EQUIP_COMBINED_BADGE_SIZE,
            )
            .node()
        },
        stretched_image(assets.image(CombiStaticAssetRole::Combined)),
        combi_passive(),
    ));
    spawn_combi_hover_frame(
        parent,
        hover_marker,
        CombiUiRect::new(0.0, 0.0, slot_size, slot_size),
    );
}

/// Pointer-feedback frame over a cell or drop area; `bind_combi_ui` decides
/// when it shows and in which color.
pub(super) fn spawn_combi_hover_frame(
    parent: &mut ChildSpawnerCommands,
    marker: CombiUiElement0104,
    rect: CombiUiRect,
) {
    parent.spawn((
        marker,
        Node {
            display: Display::None,
            border: UiRect::all(px(COMBI_HOVER_FRAME_WIDTH)),
            ..rect.node()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        combi_passive(),
    ));
}
