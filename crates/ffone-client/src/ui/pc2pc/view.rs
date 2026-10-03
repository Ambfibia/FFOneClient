use super::*;

pub(super) fn spawn_pc2pc_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    contract: Res<Pc2pcUiAssetContract>,
) {
    let assets = Pc2pcUiAssets::load(&asset_server, &mut images, &contract);
    commands
        .spawn((
            Pc2pcUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(PC2PC_UI_Z_INDEX),
            Visibility::Hidden,
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                Pc2pcUiElement::Backdrop,
                Pc2pcUiRect::default().node(),
                stretched_image(assets.image(Pc2pcStaticAssetRole::Backdrop)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                Pc2pcUiElement::TradeBackplate,
                Pc2pcUiRect::default().node(),
                stretched_image(assets.image(Pc2pcStaticAssetRole::TradeBack)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                Pc2pcUiElement::RightBackplate,
                Pc2pcUiRect::default().node(),
                sliced_image(
                    assets.image(Pc2pcStaticAssetRole::RightPanel),
                    USER_EQUIP_RIGHT_PANEL_BORDER,
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_trade_panel(root, &assets);
            spawn_item_mode(root, &assets);
        });
    commands.insert_resource(assets);
}

pub(super) fn spawn_trade_panel(root: &mut ChildSpawnerCommands, assets: &Pc2pcUiAssets) {
    root.spawn((
        Pc2pcUiElement::TradePanel,
        Pc2pcUiRect::new(0.0, 0.0, PC2PC_PANEL_WIDTH, PC2PC_PANEL_HEIGHT).node(),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ))
    .with_children(|panel| {
        panel.spawn((
            Pc2pcUiElement::TradeArea,
            PC2PC_TRADE_AREA_RECT.node(),
            sliced_image(
                assets.image(Pc2pcStaticAssetRole::TradeArea),
                PC2PC_TRADE_AREA_BORDER,
            ),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
        spawn_offer_group(
            panel,
            Pc2pcParticipant0104::Local,
            PC2PC_LOCAL_OFFER_RECT,
            assets,
        );
        spawn_offer_group(
            panel,
            Pc2pcParticipant0104::Remote,
            PC2PC_REMOTE_OFFER_RECT,
            assets,
        );
        panel.spawn((
            Pc2pcUiElement::MainButton,
            PC2PC_MAIN_BUTTON_RECT.node(),
            sliced_image(
                assets.image(Pc2pcStaticAssetRole::Button),
                PC2PC_BUTTON_BORDER,
            ),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
        spawn_pc2pc_text(
            panel,
            Some(Pc2pcUiElement::MainButtonText),
            PC2PC_MAIN_BUTTON_RECT,
            Pc2pcButtonLabel::Submit.localized_text(),
            assets,
            Pc2pcTextStyle0104::Button,
            Color::srgb(0.9, 0.9, 0.9),
            None,
        );
        panel
            .spawn((
                Pc2pcUiElement::ReadyName,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Row,
                    column_gap: px(PC2PC_READY_NAME_GAP),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..PC2PC_READY_NAME_RECT.node()
                },
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ))
            .with_children(|ready_name| {
                let mut inline_node = Node::default();
                Pc2pcTextStyle0104::ReadyLeftLabel.apply_to_node(&mut inline_node);
                spawn_pc2pc_text_node(
                    ready_name,
                    Some(Pc2pcUiElement::ReadyWaitingPrefix),
                    inline_node.clone(),
                    LocalizedText::new("ui.pc2pc.ready.waiting_for", "Waiting For"),
                    assets,
                    Pc2pcTextStyle0104::ReadyLeftLabel,
                    Color::srgba(0.0, 1.0, 0.0, 0.720_000_03),
                    None,
                );
                spawn_pc2pc_text_node(
                    ready_name,
                    Some(Pc2pcUiElement::ReadyPlayerName),
                    inline_node,
                    pc2pc_passthrough_text(""),
                    assets,
                    Pc2pcTextStyle0104::ReadyLeftLabel,
                    Color::srgba(1.0, 1.0, 0.0, 0.720_000_03),
                    None,
                );
            });
        spawn_pc2pc_text(
            panel,
            Some(Pc2pcUiElement::ReadySubject),
            PC2PC_READY_SUBJECT_RECT,
            LocalizedText::new("ui.pc2pc.ready.local_subject", " to accept your offer."),
            assets,
            Pc2pcTextStyle0104::ReadyLeftLabel,
            Color::srgba(0.0, 1.0, 0.0, 0.720_000_03),
            None,
        );
        panel
            .spawn((
                Pc2pcUiElement::ChatBox,
                PC2PC_CHAT_BOX_RECT.node(),
                stretched_image(assets.image(Pc2pcStaticAssetRole::ChatBox)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ))
            .with_children(|chat| {
                spawn_pc2pc_text(
                    chat,
                    Some(Pc2pcUiElement::ChatList),
                    PC2PC_CHAT_LIST_VIEW_RECT,
                    pc2pc_chat_log_text(&VecDeque::new()),
                    assets,
                    Pc2pcTextStyle0104::ChatLine,
                    Color::srgba(1.0, 1.0, 1.0, 0.720_000_03),
                    None,
                );
                spawn_pc2pc_text(
                    chat,
                    Some(Pc2pcUiElement::ChatInput),
                    PC2PC_CHAT_INPUT_RECT,
                    pc2pc_passthrough_text(""),
                    assets,
                    Pc2pcTextStyle0104::ChatInput,
                    Color::srgb(0.866_935_5, 1.0, 1.0),
                    Some(Pc2pcDisabledControl::ChatNeedsBackend),
                );
                chat.spawn((
                    Pc2pcUiElement::ChatSend,
                    PC2PC_CHAT_SEND_RECT.node(),
                    stretched_image(assets.image(Pc2pcStaticAssetRole::ChatSend)),
                    Pc2pcDisabledControl::ChatNeedsBackend,
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|send| {
                    spawn_pc2pc_text(
                        send,
                        None,
                        Pc2pcUiRect::new(
                            0.0,
                            0.0,
                            PC2PC_CHAT_SEND_RECT.width,
                            PC2PC_CHAT_SEND_RECT.height,
                        ),
                        LocalizedText::new("ui.pc2pc.chat.send", "SEND"),
                        assets,
                        Pc2pcTextStyle0104::ChatSendButton,
                        Color::srgb(0.778_225_8, 1.0, 1.0),
                        None,
                    );
                });
            });
    });
}

pub(super) fn spawn_offer_group(
    panel: &mut ChildSpawnerCommands,
    participant: Pc2pcParticipant0104,
    rect: Pc2pcUiRect,
    assets: &Pc2pcUiAssets,
) {
    let group_marker = match participant {
        Pc2pcParticipant0104::Local => Pc2pcUiElement::LocalOfferBox,
        Pc2pcParticipant0104::Remote => Pc2pcUiElement::RemoteOfferBox,
    };
    let group_role = match participant {
        Pc2pcParticipant0104::Local => Pc2pcStaticAssetRole::LocalOffer,
        Pc2pcParticipant0104::Remote => Pc2pcStaticAssetRole::RemoteOffer,
    };
    panel
        .spawn((
            group_marker,
            rect.node(),
            stretched_image(assets.image(group_role)),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|group| {
            for slot in 0..PC2PC_OFFER_SLOT_COUNT {
                let origin = match participant {
                    Pc2pcParticipant0104::Local => PC2PC_LOCAL_SLOT_ORIGIN,
                    Pc2pcParticipant0104::Remote => PC2PC_REMOTE_SLOT_ORIGIN,
                };
                let slot_rect = Pc2pcUiRect::new(
                    origin.x + slot as f32 * PC2PC_OFFER_SLOT_STRIDE,
                    origin.y,
                    PC2PC_OFFER_SLOT_SIZE,
                    PC2PC_OFFER_SLOT_SIZE,
                );
                group
                    .spawn((slot_rect.node(), Pickable::IGNORE))
                    .with_children(|slot_node| {
                        slot_node.spawn((
                            match participant {
                                Pc2pcParticipant0104::Local => {
                                    Pc2pcUiElement::LocalOfferFrame(slot)
                                }
                                Pc2pcParticipant0104::Remote => {
                                    Pc2pcUiElement::RemoteOfferFrame(slot)
                                }
                            },
                            Pc2pcUiRect::new(
                                0.0,
                                0.0,
                                PC2PC_OFFER_SLOT_SIZE,
                                PC2PC_OFFER_SLOT_SIZE,
                            )
                            .node(),
                            stretched_image(assets.image(Pc2pcStaticAssetRole::SlotEmpty)),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ));
                        slot_node.spawn((
                            match participant {
                                Pc2pcParticipant0104::Local => Pc2pcUiElement::LocalOfferIcon(slot),
                                Pc2pcParticipant0104::Remote => {
                                    Pc2pcUiElement::RemoteOfferIcon(slot)
                                }
                            },
                            Pc2pcUiRect::new(
                                0.0,
                                0.0,
                                PC2PC_OFFER_SLOT_SIZE,
                                PC2PC_OFFER_SLOT_SIZE,
                            )
                            .node(),
                            ImageNode::default(),
                            Pickable::IGNORE,
                            bevy::ui::FocusPolicy::Pass,
                        ));
                        let count_marker = match participant {
                            Pc2pcParticipant0104::Local => Pc2pcUiElement::LocalOfferCount(slot),
                            Pc2pcParticipant0104::Remote => Pc2pcUiElement::RemoteOfferCount(slot),
                        };
                        spawn_pc2pc_text(
                            slot_node,
                            Some(count_marker),
                            // Clean `GUI.Label` keeps the source 62x62 slot
                            // Rect and translates it by (+5,+5).
                            Pc2pcUiRect::new(5.0, 5.0, 62.0, 62.0),
                            pc2pc_count_text(""),
                            assets,
                            Pc2pcTextStyle0104::LabelUpperLeft,
                            Color::WHITE,
                            None,
                        );
                    });
            }

            let (money_marker, money_rect, money_width, taros_marker) = match participant {
                Pc2pcParticipant0104::Local => (
                    Pc2pcUiElement::LocalMoney,
                    PC2PC_LOCAL_MONEY_RECT,
                    150.0,
                    Pc2pcUiElement::LocalTaros,
                ),
                Pc2pcParticipant0104::Remote => (
                    Pc2pcUiElement::RemoteMoney,
                    PC2PC_REMOTE_MONEY_RECT,
                    290.0,
                    Pc2pcUiElement::RemoteTaros,
                ),
            };
            group
                .spawn((
                    money_marker,
                    money_rect.node(),
                    sliced_image(
                        assets.image(Pc2pcStaticAssetRole::MoneyBack),
                        PC2PC_MONEY_BACK_BORDER,
                    ),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|money| {
                    let money_text_marker = match participant {
                        Pc2pcParticipant0104::Local => Pc2pcUiElement::LocalMoneyText,
                        Pc2pcParticipant0104::Remote => Pc2pcUiElement::RemoteMoneyText,
                    };
                    spawn_pc2pc_text(
                        money,
                        Some(money_text_marker),
                        Pc2pcUiRect::new(1.0, 1.0, money_width, 28.0),
                        pc2pc_taros_amount_text(0),
                        assets,
                        Pc2pcTextStyle0104::RightLabel,
                        Color::WHITE,
                        None,
                    );
                    money.spawn((
                        taros_marker,
                        Pc2pcUiRect::new(money_width + 2.0, 0.0, 32.0, 32.0).node(),
                        stretched_image(assets.image(Pc2pcStaticAssetRole::Taros)),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });

            let (portrait_marker, portrait_rect, free_marker, title_marker, title_rect) =
                match participant {
                    Pc2pcParticipant0104::Local => (
                        Pc2pcUiElement::LocalPortrait,
                        PC2PC_LOCAL_PORTRAIT_RECT,
                        Pc2pcUiElement::LocalFreeChat,
                        Pc2pcUiElement::LocalTitle,
                        PC2PC_LOCAL_TITLE_RECT,
                    ),
                    Pc2pcParticipant0104::Remote => (
                        Pc2pcUiElement::RemotePortrait,
                        PC2PC_REMOTE_PORTRAIT_RECT,
                        Pc2pcUiElement::RemoteFreeChat,
                        Pc2pcUiElement::RemoteTitle,
                        PC2PC_REMOTE_TITLE_RECT,
                    ),
                };
            group
                .spawn((
                    portrait_marker,
                    portrait_rect.node(),
                    // The clean client draws a live `cnCharRenderCamera`
                    // directly into this rectangle. Until a native render
                    // target exists, leave the boundary transparent instead
                    // of inventing portrait pixels or a diagnostic fill.
                    BackgroundColor(Color::NONE),
                    ImageNode::default(),
                    Pc2pcDisabledControl::PortraitNeedsBackend,
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|portrait| {
                    portrait.spawn((
                        free_marker,
                        Pc2pcUiRect::new(0.0, 0.0, 18.0, 14.0).node(),
                        stretched_image(assets.image(Pc2pcStaticAssetRole::FreeChat)),
                        Visibility::Hidden,
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                });
            let (title_text, title_style) = match participant {
                Pc2pcParticipant0104::Local => (
                    LocalizedText::new("ui.pc2pc.offer.local", "MY OFFER"),
                    Pc2pcTextStyle0104::LabelMiddleRight,
                ),
                Pc2pcParticipant0104::Remote => (
                    pc2pc_remote_offer_text(""),
                    Pc2pcTextStyle0104::LabelUpperLeft,
                ),
            };
            spawn_pc2pc_text(
                group,
                Some(title_marker),
                title_rect,
                title_text,
                assets,
                title_style,
                Color::WHITE,
                None,
            );

            if participant == Pc2pcParticipant0104::Local {
                group
                    .spawn((
                        Pc2pcUiElement::AddTaros,
                        PC2PC_ADD_TAROS_RECT.node(),
                        sliced_image(
                            assets.image(Pc2pcStaticAssetRole::Button),
                            PC2PC_BUTTON_BORDER,
                        ),
                        Pc2pcDisabledControl::AddTarosNeedsNumericPopup,
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ))
                    .with_children(|button| {
                        spawn_pc2pc_text(
                            button,
                            None,
                            Pc2pcUiRect::new(
                                0.0,
                                0.0,
                                PC2PC_ADD_TAROS_RECT.width,
                                PC2PC_ADD_TAROS_RECT.height,
                            ),
                            LocalizedText::new("ui.pc2pc.button.add_taros", "ADD TAROS"),
                            assets,
                            Pc2pcTextStyle0104::Button,
                            Color::srgb(0.9, 0.9, 0.9),
                            None,
                        );
                    });
            }
        });
}

pub(super) fn spawn_item_mode(root: &mut ChildSpawnerCommands, assets: &Pc2pcUiAssets) {
    root.spawn((
        Pc2pcUiElement::PcStuffPanel,
        Pc2pcUiRect::default().node(),
        sliced_image(
            assets.image(Pc2pcStaticAssetRole::InventoryPanel),
            USER_EQUIP_INVENTORY_PANEL_BORDER,
        ),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ))
    .with_children(|panel| {
        panel.spawn((
            Pc2pcUiElement::NanoTab,
            Pc2pcUiRect::from(USER_EQUIP_NANO_TAB_TEXTURE_RECT).node(),
            stretched_image(assets.image(Pc2pcStaticAssetRole::NanoTab)),
            Pc2pcDisabledControl::NanoTabUnavailableInTrade,
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
        spawn_pc2pc_text(
            panel,
            Some(Pc2pcUiElement::ItemTabLabel),
            USER_EQUIP_ITEM_TAB_HIT_RECT.into(),
            LocalizedText::new("ui.inventory.tab.equipment", "EQUIPMENT"),
            assets,
            Pc2pcTextStyle0104::InventoryTab,
            Color::srgb(0.794_354_86, 1.0, 1.0),
            None,
        );
        spawn_pc2pc_text(
            panel,
            Some(Pc2pcUiElement::NanoTabLabel),
            USER_EQUIP_NANO_TAB_HIT_RECT.into(),
            LocalizedText::new("ui.inventory.tab.nanos", "NANOS"),
            assets,
            Pc2pcTextStyle0104::InventoryTab,
            Color::srgb(0.794_354_86, 1.0, 1.0),
            None,
        );
        spawn_static_control(
            panel,
            Pc2pcUiElement::Close,
            USER_EQUIP_CLOSE_RECT.into(),
            assets.image(Pc2pcStaticAssetRole::Close),
            Pc2pcDisabledControl::CloseOwnedBySessionController,
        );
        spawn_static_control(
            panel,
            Pc2pcUiElement::Trash,
            USER_EQUIP_TRASH_RECT.into(),
            assets.image(Pc2pcStaticAssetRole::Trash),
            Pc2pcDisabledControl::TrashUnavailableInTrade,
        );
        spawn_static_control(
            panel,
            Pc2pcUiElement::Help,
            USER_EQUIP_HELP_RECT.into(),
            assets.image(Pc2pcStaticAssetRole::Help),
            Pc2pcDisabledControl::HelpOwnedByShell,
        );
        panel
            .spawn((
                Pc2pcUiElement::InventoryViewport,
                Node {
                    overflow: Overflow::clip(),
                    ..Pc2pcUiRect::from(USER_EQUIP_INVENTORY_VIEWPORT_RECT).node()
                },
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ))
            .with_children(|viewport| {
                viewport
                    .spawn((
                        Pc2pcUiElement::InventoryContent,
                        Pc2pcUiRect::new(
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
                            let column = slot % 5;
                            let row = slot / 5;
                            let rect = Pc2pcUiRect::new(
                                column as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
                                row as f32 * USER_EQUIP_INVENTORY_SLOT_STRIDE,
                                USER_EQUIP_INVENTORY_SLOT_SIZE,
                                USER_EQUIP_INVENTORY_SLOT_SIZE,
                            );
                            spawn_item_slot(
                                content,
                                rect,
                                Pc2pcUiElement::InventorySlotFrame(slot),
                                Pc2pcUiElement::InventorySlotIcon(slot),
                                Pc2pcUiElement::InventorySlotBadge(slot),
                                Pc2pcUiElement::InventorySlotCount(slot),
                                assets,
                            );
                        }
                    });
            });
    });

    root.spawn((
        Pc2pcUiElement::EquipmentPanel,
        Pc2pcUiRect::default().node(),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ))
    .with_children(|panel| {
        panel.spawn((
            Pc2pcUiElement::EquipmentTitle,
            Pc2pcUiRect::from(USER_EQUIP_EQUIPMENT_TITLE_RECT).node(),
            stretched_image(assets.image(Pc2pcStaticAssetRole::EquipTitle)),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
        spawn_pc2pc_text(
            panel,
            Some(Pc2pcUiElement::EquipmentTitleLabel),
            USER_EQUIP_EQUIPMENT_TITLE_RECT.into(),
            LocalizedText::new("ui.inventory.equipped", " Equipped"),
            assets,
            Pc2pcTextStyle0104::EquipmentTitle,
            Color::srgb(0.0, 0.2, 0.4),
            None,
        );
        panel
            .spawn((
                Pc2pcUiElement::EquipmentContent,
                Pc2pcUiRect::from(USER_EQUIP_EQUIPMENT_CONTENT_RECT).node(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ))
            .with_children(|content| {
                for visual_index in 0..USER_EQUIP_EQUIPMENT_STRIP_COUNT {
                    let rect = Pc2pcUiRect::new(
                        0.0,
                        visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                    );
                    content
                        .spawn((rect.node(), Pickable::IGNORE))
                        .with_children(|slot_node| {
                            slot_node.spawn((
                                Pc2pcUiElement::EquipmentSlotFrame(visual_index),
                                Pc2pcUiRect::new(
                                    0.0,
                                    0.0,
                                    USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                                    USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                                )
                                .node(),
                                stretched_image(assets.image(Pc2pcStaticAssetRole::SlotEmpty)),
                                Pc2pcDisabledControl::ItemMutationOwnedByIntentBoundary,
                                Pickable::IGNORE,
                                bevy::ui::FocusPolicy::Pass,
                            ));
                            slot_node.spawn((
                                Pc2pcUiElement::EquipmentSlotIcon(visual_index),
                                Pc2pcUiRect::new(
                                    0.0,
                                    0.0,
                                    USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                                    USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                                )
                                .node(),
                                ImageNode::default(),
                                Pickable::IGNORE,
                                bevy::ui::FocusPolicy::Pass,
                            ));
                            slot_node.spawn((
                                Pc2pcUiElement::EquipmentSlotBadge(visual_index),
                                Pc2pcUiRect::new(
                                    USER_EQUIP_COMBINED_BADGE_LEFT,
                                    USER_EQUIP_COMBINED_BADGE_TOP,
                                    USER_EQUIP_COMBINED_BADGE_SIZE,
                                    USER_EQUIP_COMBINED_BADGE_SIZE,
                                )
                                .node(),
                                stretched_image(assets.image(Pc2pcStaticAssetRole::Combined)),
                                Pickable::IGNORE,
                                bevy::ui::FocusPolicy::Pass,
                            ));
                            spawn_pc2pc_text(
                                slot_node,
                                Some(Pc2pcUiElement::EquipmentSlotLabel(visual_index)),
                                Pc2pcUiRect::new(0.0, -2.0, 60.0, 19.0),
                                equipment_slot_localized_text(visual_index),
                                assets,
                                Pc2pcTextStyle0104::EquipmentSlot,
                                Color::WHITE,
                                None,
                            );
                        });
                }
            });
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_item_slot(
    parent: &mut ChildSpawnerCommands,
    rect: Pc2pcUiRect,
    frame_marker: Pc2pcUiElement,
    icon_marker: Pc2pcUiElement,
    badge_marker: Pc2pcUiElement,
    count_marker: Pc2pcUiElement,
    assets: &Pc2pcUiAssets,
) {
    parent
        .spawn((rect.node(), Pickable::IGNORE))
        .with_children(|slot_node| {
            slot_node.spawn((
                frame_marker,
                Pc2pcUiRect::new(
                    0.0,
                    0.0,
                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                )
                .node(),
                stretched_image(assets.image(Pc2pcStaticAssetRole::SlotEmpty)),
                Pc2pcDisabledControl::ItemMutationOwnedByIntentBoundary,
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            slot_node.spawn((
                icon_marker,
                Pc2pcUiRect::new(
                    0.0,
                    0.0,
                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                )
                .node(),
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            slot_node.spawn((
                badge_marker,
                Pc2pcUiRect::new(
                    USER_EQUIP_COMBINED_BADGE_LEFT,
                    USER_EQUIP_COMBINED_BADGE_TOP,
                    USER_EQUIP_COMBINED_BADGE_SIZE,
                    USER_EQUIP_COMBINED_BADGE_SIZE,
                )
                .node(),
                stretched_image(assets.image(Pc2pcStaticAssetRole::Combined)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_pc2pc_text(
                slot_node,
                Some(count_marker),
                Pc2pcUiRect::new(
                    USER_EQUIP_COUNT_LABEL_LEFT,
                    USER_EQUIP_COUNT_LABEL_TOP,
                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                    USER_EQUIP_INVENTORY_SLOT_SIZE,
                ),
                pc2pc_count_text(""),
                assets,
                Pc2pcTextStyle0104::InventoryCount,
                Color::WHITE,
                None,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_pc2pc_text(
    parent: &mut ChildSpawnerCommands,
    marker: Option<Pc2pcUiElement>,
    rect: Pc2pcUiRect,
    localized: LocalizedText,
    assets: &Pc2pcUiAssets,
    style: Pc2pcTextStyle0104,
    color: Color,
    disabled: Option<Pc2pcDisabledControl>,
) {
    spawn_pc2pc_text_node(
        parent,
        marker,
        style.node(rect),
        localized,
        assets,
        style,
        color,
        disabled,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_pc2pc_text_node(
    parent: &mut ChildSpawnerCommands,
    marker: Option<Pc2pcUiElement>,
    node: Node,
    localized: LocalizedText,
    assets: &Pc2pcUiAssets,
    style: Pc2pcTextStyle0104,
    color: Color,
    disabled: Option<Pc2pcDisabledControl>,
) {
    let fallback = pc2pc_fallback_text(&localized);
    let mut entity = parent.spawn((
        node,
        Text::new(fallback),
        localized,
        style.font(assets),
        TextColor(color),
        style.layout(),
        style,
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    if let Some(marker) = marker {
        entity.insert(marker);
    }
    if let Some(disabled) = disabled {
        entity.insert(disabled);
    }
}

pub(super) fn spawn_static_control(
    parent: &mut ChildSpawnerCommands,
    marker: Pc2pcUiElement,
    rect: Pc2pcUiRect,
    image: Handle<Image>,
    disabled: Pc2pcDisabledControl,
) {
    parent.spawn((
        marker,
        rect.node(),
        stretched_image(image),
        disabled,
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
}
