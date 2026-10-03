use super::*;

pub(super) fn spawn_server_selection_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = ServerSelectionUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());

    commands
        .spawn((
            ServerSelectionUiElement::Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(SERVER_SELECTION_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                ServerSelectionUiElement::Background,
                absolute_node(),
                stretch_image(assets.background.clone()),
                Pickable::IGNORE,
            ));

            root.spawn((
                ServerSelectionUiElement::Panel,
                SERVER_SELECTION_PANEL_LOCAL_RECT.node(),
                stretch_image(assets.panel.clone()),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                spawn_static_text(
                    panel,
                    ServerSelectionUiRect::new(45.0, 15.0, 120.0, 26.0),
                    LocalizedText::new(
                        SERVER_SELECTION_HEADER_SERVER_CHANNEL_KEY,
                        SERVER_SELECTION_COPY_HEADER_SERVER_CHANNEL,
                    ),
                    &assets.font,
                    LegacyTextSize::Large,
                    Justify::Left,
                );
                spawn_static_text(
                    panel,
                    ServerSelectionUiRect::new(275.0, 15.0, 50.0, 26.0),
                    LocalizedText::new(
                        SERVER_SELECTION_HEADER_STATUS_KEY,
                        SERVER_SELECTION_COPY_HEADER_STATUS,
                    ),
                    &assets.font,
                    LegacyTextSize::Large,
                    Justify::Left,
                );
                panel.spawn((
                    SERVER_SELECTION_INNER_RECT.node(),
                    stretch_image(assets.inner.clone()),
                    Pickable::IGNORE,
                ));

                panel
                    .spawn((
                        Node {
                            overflow: Overflow::clip(),
                            ..SERVER_SELECTION_VIEWPORT_RECT.node()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|viewport| {
                        viewport
                            .spawn((
                                ServerSelectionUiElement::ScrollContent,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    top: px(0),
                                    width: px(272),
                                    height: px(SERVER_SELECTION_INITIAL_SCROLL_HEIGHT),
                                    ..default()
                                },
                            ))
                            .with_children(|content| {
                                content
                                    .spawn((
                                        Button,
                                        SERVER_SELECTION_SERVER_ROW_RECT.node(),
                                        transparent_row_image(assets.row_hover.clone()),
                                        ServerSelectionUiControl::ServerToggle,
                                    ))
                                    .with_children(|row| {
                                        spawn_dynamic_text(
                                            row,
                                            ServerSelectionUiRect::new(26.0, 0.0, 175.0, 18.0),
                                            &assets.font,
                                            LegacyTextSize::Large,
                                            ServerSelectionUiTextRole::ServerHeading,
                                        );
                                        spawn_dynamic_text(
                                            row,
                                            ServerSelectionUiRect::new(195.0, 0.0, 50.0, 18.0),
                                            &assets.font,
                                            LegacyTextSize::Large,
                                            ServerSelectionUiTextRole::ServerStatus,
                                        );
                                    });

                                for shard in
                                    SERVER_SELECTION_FIRST_SHARD..=SERVER_SELECTION_LAST_SHARD
                                {
                                    let top = SERVER_SELECTION_FIRST_CHANNEL_TOP
                                        + f32::from(shard - 1) * SERVER_SELECTION_CHANNEL_STRIDE;
                                    content
                                        .spawn((
                                            ServerSelectionUiElement::ChannelRow(shard),
                                            Button,
                                            ServerSelectionUiRect::new(0.0, top, 272.0, 16.0)
                                                .node(),
                                            transparent_row_image(assets.row_hover.clone()),
                                            ServerSelectionUiControl::Shard(shard),
                                        ))
                                        .with_children(|row| {
                                            spawn_static_text(
                                                row,
                                                ServerSelectionUiRect::new(54.0, 0.0, 112.0, 18.0),
                                                server_selection_channel_localized(shard),
                                                &assets.font,
                                                LegacyTextSize::Small,
                                                Justify::Left,
                                            );
                                            spawn_dynamic_text(
                                                row,
                                                SERVER_SELECTION_CHANNEL_STATUS_RECT,
                                                &assets.font,
                                                LegacyTextSize::Small,
                                                ServerSelectionUiTextRole::ShardStatus(shard),
                                            );
                                        });
                                }
                            });
                    });

                panel.spawn((
                    ServerSelectionUiElement::ScrollTrack,
                    ServerSelectionUiRect::new(314.0, 62.0, 18.0, 203.0).node(),
                    sliced_image(
                        assets.scroll_track.clone(),
                        BorderRect {
                            min_inset: Vec2::new(2.0, 4.0),
                            max_inset: Vec2::new(2.0, 4.0),
                        },
                    ),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    ServerSelectionUiElement::ScrollThumb,
                    ServerSelectionUiRect::new(316.0, 62.0, 13.0, 94.0).node(),
                    sliced_image(assets.scroll_thumb.clone(), BorderRect::all(6.0)),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    ServerSelectionUiElement::ScrollUp,
                    Button,
                    ServerSelectionUiRect::new(314.0, 50.0, 17.0, 12.0).node(),
                    stretch_image(assets.scroll_up.clone()),
                    ServerSelectionUiControl::ScrollUp,
                ));
                panel.spawn((
                    ServerSelectionUiElement::ScrollDown,
                    Button,
                    ServerSelectionUiRect::new(314.0, 265.0, 17.0, 12.0).node(),
                    stretch_image(assets.scroll_down.clone()),
                    ServerSelectionUiControl::ScrollDown,
                ));
                spawn_legacy_button(
                    panel,
                    SERVER_SELECTION_CONNECT_RECT,
                    LocalizedText::new(SERVER_SELECTION_CONNECT_KEY, SERVER_SELECTION_COPY_CONNECT),
                    ServerSelectionUiControl::Connect,
                    LegacyButtonKind::Blue,
                    &assets,
                );
            });

            spawn_root_button(
                root,
                ServerSelectionUiElement::AccountButton,
                ServerSelectionUiRect::new(0.0, 0.0, 200.0, 27.0),
                LocalizedText::new(
                    SERVER_SELECTION_MY_ACCOUNT_KEY,
                    SERVER_SELECTION_COPY_MY_ACCOUNT,
                ),
                ServerSelectionUiControl::MyAccount,
                LegacyButtonKind::Blue,
                &assets,
            );
            spawn_root_button(
                root,
                ServerSelectionUiElement::HomepageButton,
                ServerSelectionUiRect::new(0.0, 0.0, 200.0, 27.0),
                LocalizedText::new(
                    SERVER_SELECTION_HOMEPAGE_KEY,
                    SERVER_SELECTION_COPY_HOMEPAGE,
                ),
                ServerSelectionUiControl::Homepage,
                LegacyButtonKind::Blue,
                &assets,
            );
            spawn_root_button(
                root,
                ServerSelectionUiElement::QuitButton,
                ServerSelectionUiRect::new(0.0, 0.0, 125.0, 27.0),
                LocalizedText::new(SERVER_SELECTION_QUIT_KEY, SERVER_SELECTION_COPY_QUIT),
                ServerSelectionUiControl::Quit,
                LegacyButtonKind::Red,
                &assets,
            );
        });
}

pub(super) fn spawn_static_text(
    parent: &mut ChildSpawnerCommands,
    rect: ServerSelectionUiRect,
    localized: LocalizedText,
    font: &Handle<Font>,
    size: LegacyTextSize,
    justify: Justify,
) {
    let mut node = rect.node();
    node.align_items = match size {
        LegacyTextSize::Large => AlignItems::FlexStart,
        LegacyTextSize::Small => AlignItems::Center,
    };
    if size == LegacyTextSize::Large {
        node.padding = UiRect::top(px(SERVER_SELECTION_LABEL_TOP_PADDING));
    }
    let fallback = localized_fallback(&localized);
    parent.spawn((node, Pickable::IGNORE)).with_child((
        Text::new(fallback),
        localized,
        legacy_text_font(font, size),
        TextColor(Color::WHITE),
        TextLayout::new(justify, LineBreak::NoWrap),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_dynamic_text(
    parent: &mut ChildSpawnerCommands,
    rect: ServerSelectionUiRect,
    font: &Handle<Font>,
    size: LegacyTextSize,
    role: ServerSelectionUiTextRole,
) {
    let mut node = rect.node();
    node.align_items = AlignItems::Center;
    let localized = initial_server_selection_localized(role);
    let fallback = localized_fallback(&localized);
    parent.spawn((node, Pickable::IGNORE)).with_child((
        role,
        Text::new(fallback),
        localized,
        legacy_text_font(font, size),
        TextColor(Color::WHITE),
        TextLayout::new(Justify::Left, LineBreak::NoWrap),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_legacy_button(
    parent: &mut ChildSpawnerCommands,
    rect: ServerSelectionUiRect,
    localized: LocalizedText,
    control: ServerSelectionUiControl,
    kind: LegacyButtonKind,
    assets: &ServerSelectionUiAssets,
) {
    let normal = match kind {
        LegacyButtonKind::Blue => assets.button_normal.clone(),
        LegacyButtonKind::Red => assets.red_normal.clone(),
    };
    let fallback = localized_fallback(&localized);
    parent
        .spawn((
            Button,
            centered_button_node(rect, kind),
            sliced_image(normal, BorderRect::all(5.0)),
            control,
            kind,
        ))
        .with_child((
            LegacyButtonLabel,
            Text::new(fallback),
            localized,
            legacy_text_font(&assets.font, LegacyTextSize::Large),
            TextColor(legacy_button_text_color(kind, Interaction::None)),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Pickable::IGNORE,
        ));
}

pub(super) fn spawn_root_button(
    parent: &mut ChildSpawnerCommands,
    element: ServerSelectionUiElement,
    rect: ServerSelectionUiRect,
    localized: LocalizedText,
    control: ServerSelectionUiControl,
    kind: LegacyButtonKind,
    assets: &ServerSelectionUiAssets,
) {
    let normal = match kind {
        LegacyButtonKind::Blue => assets.button_normal.clone(),
        LegacyButtonKind::Red => assets.red_normal.clone(),
    };
    let fallback = localized_fallback(&localized);
    parent
        .spawn((
            element,
            Button,
            centered_button_node(rect, kind),
            sliced_image(normal, BorderRect::all(5.0)),
            control,
            kind,
        ))
        .with_child((
            LegacyButtonLabel,
            Text::new(fallback),
            localized,
            legacy_text_font(&assets.font, LegacyTextSize::Large),
            TextColor(legacy_button_text_color(kind, Interaction::None)),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Pickable::IGNORE,
        ));
}
