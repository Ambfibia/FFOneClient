use super::*;

pub(super) fn spawn_buddy_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = BuddyUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            BuddyUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(BUDDY_UI_Z_INDEX),
        ))
        .with_children(|root| {
            spawn_buddy_panel(root, &assets);
        });

    commands
        .spawn((
            BuddyModalRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: Display::None,
                ..default()
            },
            ImageNode {
                image: assets.add_overlay.clone(),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                ..default()
            },
            GlobalZIndex(BUDDY_MODAL_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|overlay| {
            spawn_add_dialog(overlay, &assets);
        });
}

pub(super) fn spawn_buddy_panel(parent: &mut ChildSpawnerCommands, assets: &BuddyUiAssets) {
    parent
        .spawn((
            BuddyPanel,
            Node {
                position_type: PositionType::Absolute,
                width: px(BUDDY_GROUP_RECT.width),
                height: px(BUDDY_GROUP_RECT.height),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            UiTransform::default(),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel.spawn((
                BUDDY_WINDOW_RECT.node(),
                sliced_image(assets.window.clone(), BUDDY_WINDOW_BORDER),
                Pickable::IGNORE,
            ));
            panel.spawn((
                buddy_text_node(BUDDY_WINDOW_RECT, BuddyTextStyle::BuddyWindow),
                Text::new(BUDDY_LIST_TITLE),
                buddy_text_font(assets, BuddyTextStyle::BuddyWindow),
                buddy_text_color(BuddyTextStyle::BuddyWindow),
                buddy_text_layout(BuddyTextStyle::BuddyWindow),
                BuddyTextStyle::BuddyWindow,
                buddy_static_localized(BUDDY_LIST_TITLE),
                buddy_text_transform(BuddyTextStyle::BuddyWindow),
                Pickable::IGNORE,
            ));
            panel.spawn((
                BuddyListBackground,
                BUDDY_CONTENT_RECT.node(),
                sliced_image(assets.list_background.clone(), BUDDY_LIST_BACKGROUND_BORDER),
                Pickable::IGNORE,
            ));
            spawn_control(
                panel,
                BuddyControl::Delete,
                BUDDY_DELETE_RECT,
                BUDDY_DELETE_LABEL,
                assets,
            );
            spawn_control(
                panel,
                BuddyControl::Warp,
                BUDDY_WARP_RECT,
                BUDDY_WARP_LABEL,
                assets,
            );
            spawn_control(
                panel,
                BuddyControl::Add,
                BUDDY_ADD_RECT,
                BUDDY_ADD_LABEL,
                assets,
            );
            let mut list_node = BUDDY_INNER_LIST_RECT.node();
            list_node.overflow = Overflow::clip();
            panel
                .spawn((list_node, Pickable::IGNORE))
                .with_children(|list| {
                    for slot in 0..BUDDY_MAX_SLOTS {
                        spawn_buddy_row(list, slot, assets);
                    }
                    list.spawn((
                        BuddyScrollbarPart::Track,
                        BUDDY_SCROLL_TRACK_RECT.node(),
                        sliced_image(assets.scroll_track.clone(), BUDDY_SCROLL_TRACK_BORDER),
                        Pickable::IGNORE,
                        ZIndex(2),
                    ));
                    list.spawn((
                        BuddyScrollbarPart::Up,
                        BUDDY_SCROLL_UP_RECT.node(),
                        stretched_image(assets.scroll_up.clone()),
                        Pickable::IGNORE,
                        ZIndex(3),
                    ));
                    list.spawn((
                        BuddyScrollbarPart::Down,
                        BUDDY_SCROLL_DOWN_RECT.node(),
                        stretched_image(assets.scroll_down.clone()),
                        Pickable::IGNORE,
                        ZIndex(3),
                    ));
                    list.spawn((
                        BuddyScrollbarPart::Thumb,
                        BuddyUiRect::new(
                            BUDDY_SCROLL_TRACK_RECT.x,
                            BUDDY_SCROLL_TRACK_RECT.y,
                            BUDDY_SCROLL_THUMB_WIDTH,
                            BUDDY_SCROLL_THUMB_MIN_HEIGHT,
                        )
                        .node(),
                        sliced_image(assets.scroll_thumb.clone(), BUDDY_SCROLL_TRACK_BORDER),
                        Pickable::IGNORE,
                        ZIndex(4),
                    ));
                });
        });
}

pub(super) fn spawn_buddy_row(parent: &mut ChildSpawnerCommands, slot: usize, assets: &BuddyUiAssets) {
    parent
        .spawn((
            BuddyRow { slot },
            Button,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(BUDDY_INNER_LIST_RECT.width - 16.0),
                height: px(BUDDY_LIST_ROW_HEIGHT),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                BuddyRowPart {
                    slot,
                    kind: BuddyRowPartKind::Selection,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: px(BUDDY_INNER_LIST_RECT.width),
                    height: px(BUDDY_LIST_ROW_HEIGHT),
                    display: Display::None,
                    ..default()
                },
                sliced_image(assets.selection.clone(), BUDDY_SELECT_BORDER),
                Pickable::IGNORE,
            ));
            row.spawn((
                BuddyRowPart {
                    slot,
                    kind: BuddyRowPartKind::SelectionSpike,
                },
                BuddyUiRect::new(-20.0, 0.0, 38.0, BUDDY_LIST_ROW_HEIGHT).node(),
                ImageNode {
                    image: assets.selection.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Visibility::Hidden,
                Pickable::IGNORE,
            ));
            row.spawn((
                BuddyRowPart {
                    slot,
                    kind: BuddyRowPartKind::FreeChat,
                },
                BUDDY_FREECHAT_RECT.node(),
                ImageNode {
                    image: assets.freechat.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Visibility::Hidden,
                Pickable::IGNORE,
            ));
            row.spawn((
                BuddyRowPart {
                    slot,
                    kind: BuddyRowPartKind::Label,
                },
                buddy_text_node(
                    BuddyUiRect::new(
                        0.0,
                        0.0,
                        BUDDY_INNER_LIST_RECT.width - 16.0,
                        BUDDY_LIST_ROW_HEIGHT,
                    ),
                    BuddyTextStyle::BuddyItem,
                ),
                Text::new(""),
                buddy_text_font(assets, BuddyTextStyle::BuddyItem),
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                buddy_text_layout(BuddyTextStyle::BuddyItem),
                BuddyTextStyle::BuddyItem,
                LocalizedText::new("ui.buddy.row.empty", ""),
                buddy_text_transform(BuddyTextStyle::BuddyItem),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_add_dialog(parent: &mut ChildSpawnerCommands, assets: &BuddyUiAssets) {
    parent
        .spawn((
            BuddyAddDialog,
            Node {
                position_type: PositionType::Absolute,
                width: px(BUDDY_ADD_WINDOW_RECT.width),
                height: px(BUDDY_ADD_WINDOW_RECT.height),
                ..default()
            },
            UiTransform::default(),
            ImageNode {
                image: assets.add_dialog.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|dialog| {
            spawn_dialog_text(
                dialog,
                BUDDY_ADD_TITLE_RECT,
                BUDDY_ADD_TITLE,
                BuddyTextStyle::Transparent3,
                assets,
            );
            spawn_dialog_text(
                dialog,
                BUDDY_ADD_INSTRUCTION_RECT,
                BUDDY_ADD_INSTRUCTION,
                BuddyTextStyle::DeleteText,
                assets,
            );
            // Clean uses `GUI.TextField(..., PopDeleteText)`. `DeleteText` has
            // no background, so the text sits directly on `CSDeleteWindow`.
            dialog.spawn((
                BuddyAddInputText,
                buddy_text_node(BUDDY_ADD_TEXT_FIELD_RECT, BuddyTextStyle::DeleteText),
                Text::new(""),
                buddy_text_font(assets, BuddyTextStyle::DeleteText),
                buddy_text_color(BuddyTextStyle::DeleteText),
                buddy_text_layout(BuddyTextStyle::DeleteText),
                BuddyTextStyle::DeleteText,
                LocalizedText::new("ui.buddy.add_name_input", "{name}").with_arg("name", ""),
                buddy_text_transform(BuddyTextStyle::DeleteText),
                Pickable::IGNORE,
            ));
            spawn_control(
                dialog,
                BuddyControl::ModalCancel,
                BUDDY_ADD_CANCEL_RECT,
                BUDDY_CANCEL_LABEL,
                assets,
            );
            spawn_control(
                dialog,
                BuddyControl::ModalAdd,
                BUDDY_ADD_SUBMIT_RECT,
                BUDDY_ADD_LABEL,
                assets,
            );
        });
}

pub(super) fn spawn_dialog_text(
    parent: &mut ChildSpawnerCommands,
    rect: BuddyUiRect,
    value: &'static str,
    style: BuddyTextStyle,
    assets: &BuddyUiAssets,
) {
    parent.spawn((
        buddy_text_node(rect, style),
        Text::new(value),
        buddy_text_font(assets, style),
        buddy_text_color(style),
        buddy_text_layout(style),
        style,
        buddy_static_localized(value),
        buddy_text_transform(style),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_control(
    parent: &mut ChildSpawnerCommands,
    control: BuddyControl,
    rect: BuddyUiRect,
    label: &'static str,
    assets: &BuddyUiAssets,
) {
    let (style, border) = match control {
        BuddyControl::Delete => (BuddyTextStyle::RedButton2, BorderRect::all(6.0)),
        BuddyControl::ModalCancel => (BuddyTextStyle::Cancel, BorderRect::all(5.0)),
        BuddyControl::ModalAdd => (BuddyTextStyle::QuitButton, BorderRect::all(5.0)),
        BuddyControl::Warp | BuddyControl::Add => (BuddyTextStyle::BlueButton, BUDDY_BUTTON_BORDER),
    };
    parent
        .spawn((
            Button,
            BuddyControlMarker(control),
            buddy_text_node(rect, style),
            sliced_image(assets.control_image(control, Interaction::None), border),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                buddy_text_font(assets, style),
                buddy_text_color(style),
                buddy_text_layout(style),
                style,
                buddy_static_localized(label),
                buddy_text_transform(style),
                Pickable::IGNORE,
            ));
        });
}
