use super::*;

pub(super) fn spawn_system_popup(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    parent
        .spawn((
            OptionSystemPopupRoot,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: px(285.0),
                top: px(210.0),
                width: px(450.0),
                height: px(210.0),
                ..default()
            },
            sliced_image(
                assets.image(OptionTextureRole::PanelSmall),
                OPTION_DARK_BOX_BORDER,
            ),
            Pickable::default(),
            ZIndex(500),
        ))
        .with_children(|popup| {
            popup.spawn((
                OptionSystemPopupLabel,
                OptionUiRect::new(35.0, 35.0, 380.0, 100.0).node(),
                Text::new(""),
                option_passthrough_text(""),
                (
                    TextFont {
                        font: (assets.chalet.clone()).into(),
                        font_size: (15.0).into(),
                        ..default()
                    },
                    LineHeight::Px(OPTION_CHALET_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            spawn_small_button(
                popup,
                OptionUiRect::new(135.0, 155.0, 180.0, 25.0),
                "OK",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionSystemPopupOkButton),
            );
        });
}

pub(super) fn spawn_social_page(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    parent
        .spawn((
            OptionPage(OptionTab::Social),
            Node {
                display: Display::None,
                ..OPTION_PAGE_RECT.node()
            },
            Pickable::IGNORE,
            ZIndex(OPTION_PAGE_Z_INDEX),
        ))
        .with_children(|page| {
            for rect in [
                OPTION_SOCIAL_REQUEST_HEADER_RECT,
                OPTION_SOCIAL_BLOCKED_HEADER_RECT,
            ] {
                page.spawn((
                    OptionSkyHeader,
                    rect.node(),
                    stretched_image(assets.image(OptionTextureRole::Frame)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OPTION_SOCIAL_REQUEST_LEFT_RECT,
                OPTION_SOCIAL_REQUEST_RIGHT_RECT,
                OPTION_SOCIAL_BLOCKED_BODY_RECT,
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2` has a zero border: Stretch is the serialized behavior.
                    stretched_image(assets.image(OptionTextureRole::PanelFlat)),
                    Pickable::IGNORE,
                ));
            }
            page.spawn((
                OPTION_SOCIAL_REQUEST_CONNECTOR_RECT.node(),
                // Clean `darkbox2_inter` has a zero border and is used at its native 26x27.
                stretched_image(assets.image(OptionTextureRole::PanelNotch)),
                Pickable::IGNORE,
            ));

            spawn_option_title(
                page,
                OptionUiRect::new(20.0, 228.0, 200.0, 50.0),
                "ALLOW REQUESTS",
                assets,
            );
            spawn_option_title(
                page,
                OptionUiRect::new(500.0, 28.0, 200.0, 50.0),
                "BLOCKED PLAYERS",
                assets,
            );
            spawn_text(
                page,
                OptionUiRect::new(510.0, 40.0, 368.0, 50.0),
                "Select a name below to remove it from your Blocked Players list. Reminder: \
                 Unblocked players can send you requests and chat messages.",
                assets.chalet.clone(),
                OPTION_CHALET_WHITE_LABEL_FONT_SIZE,
                Color::WHITE,
            );
            spawn_small_button::<OptionSocialDefaultsButton, ()>(
                page,
                OPTION_SOCIAL_DEFAULTS_RECT,
                "DEFAULT REQUESTS",
                assets,
                Some(OptionSocialDefaultsButton),
                None,
            );

            for (index, (kind, label)) in [
                (SocialRequestKind::Group, "GROUP REQUESTS:"),
                (SocialRequestKind::Buddy, "BUDDY REQUESTS:"),
                (SocialRequestKind::Trade, "TRADE REQUESTS:"),
            ]
            .into_iter()
            .enumerate()
            {
                let top = 280.0 + index as f32 * 77.0;
                page.spawn((
                    OptionUiRect::new(80.0, top, 300.0, 60.0).node(),
                    sliced_image(
                        assets.image(OptionTextureRole::PanelSmall),
                        OPTION_DARK_BOX_BORDER,
                    ),
                    Pickable::IGNORE,
                ));
                spawn_option_smallfont(
                    page,
                    OptionUiRect::new(90.0, top + 20.0, 150.0, 40.0),
                    label,
                    assets,
                );
                spawn_flag_button(
                    page,
                    OptionUiRect::new(260.0, top + 20.0, 22.0, 21.0),
                    kind,
                    true,
                    "Yes",
                    assets,
                );
                spawn_flag_button(
                    page,
                    OptionUiRect::new(320.0, top + 20.0, 22.0, 21.0),
                    kind,
                    false,
                    "No",
                    assets,
                );
            }

            page.spawn((
                OPTION_SOCIAL_BLOCKED_LIST_RECT.node(),
                sliced_image(
                    assets.image(OptionTextureRole::TextField),
                    OPTION_TEXT_FIELD_BORDER,
                ),
                Pickable::IGNORE,
            ));
            page.spawn((
                OptionBlockedScrollChrome,
                OptionUiRect::new(843.0, 105.0, OPTION_SCROLLBAR_VISUAL_WIDTH, 290.0).node(),
                sliced_image(
                    assets.image(OptionTextureRole::ScrollBar),
                    OPTION_SCROLLBAR_BORDER,
                ),
                Pickable::IGNORE,
            ));
            page.spawn((
                Button,
                OptionBlockedScrollChrome,
                OptionBlockedScrollButton::Up,
                OptionUiRect::new(843.0, 105.0, 17.0, 12.0).node(),
                stretched_image(assets.image(OptionTextureRole::ScrollUp)),
                Pickable::default(),
            ));
            page.spawn((
                Button,
                OptionBlockedScrollChrome,
                OptionBlockedScrollButton::Down,
                OptionUiRect::new(843.0, 383.0, 17.0, 12.0).node(),
                stretched_image(assets.image(OptionTextureRole::ScrollDown)),
                Pickable::default(),
            ));
            page.spawn((
                OptionBlockedScrollChrome,
                OptionBlockedScrollThumb,
                OptionUiRect::new(842.0, 115.0, OPTION_SCROLL_THUMB_VISUAL_WIDTH, 19.0).node(),
                sliced_image(
                    assets.image(OptionTextureRole::ScrollThumb),
                    OPTION_SCROLL_THUMB_BORDER,
                ),
                Pickable::IGNORE,
            ));
            for projection_index in 0..OPTION_BLOCKED_VISIBLE_ROWS {
                page.spawn((
                    Button,
                    OptionBlockedRow { projection_index },
                    Node {
                        display: Display::None,
                        ..OptionUiRect::new(
                            OPTION_SOCIAL_BLOCKED_LIST_RECT.x + 5.0,
                            OPTION_SOCIAL_BLOCKED_LIST_RECT.y
                                + 5.0
                                + projection_index as f32 * 16.0,
                            290.0,
                            16.0,
                        )
                        .node()
                    },
                    stretched_image(assets.image(OptionTextureRole::RowSelected)),
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn((
                        OptionBlockedRowLabel { projection_index },
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        Text::new(""),
                        option_passthrough_text(""),
                        (
                            TextFont {
                                font: (assets.chalet.clone()).into(),
                                font_size: (12.0).into(),
                                ..default()
                            },
                            LineHeight::Px(OPTION_CHALET_SMALL_LINE_HEIGHT),
                        ),
                        TextColor(Color::srgb(1.0, 1.0, 0.0)),
                        TextLayout::new(Justify::Left, LineBreak::NoWrap),
                        Pickable::IGNORE,
                    ));
                });
            }
            spawn_small_button::<(), OptionUnignoreButton>(
                page,
                OPTION_SOCIAL_UNIGNORE_RECT,
                "UNIGNORE",
                assets,
                None,
                Some(OptionUnignoreButton),
            );
        });
}

pub(super) fn spawn_radio_button<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    marker: M,
    indicator: OptionRadioIndicator,
    label: &str,
    assets: &OptionUiAssets,
) {
    parent.spawn((
        Button,
        marker,
        rect.node(),
        sliced_image(
            assets.image(OptionTextureRole::RadioEmpty),
            OPTION_TOGGLE_BORDER,
        ),
        Pickable::default(),
    ));
    let label_rect = OptionUiRect::new(rect.x + 30.0, rect.y, rect.width, rect.height);
    parent
        .spawn((
            Node {
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..label_rect.node()
            },
            Pickable::IGNORE,
        ))
        .with_children(|wrapper| {
            let mut entity = wrapper.spawn((
                OptionRadioLabel(indicator),
                Node {
                    width: percent(100.0),
                    ..default()
                },
                Text::new(label),
                (
                    TextFont {
                        font: (assets.chalet.clone()).into(),
                        font_size: (12.0).into(),
                        ..default()
                    },
                    LineHeight::Px(OPTION_CHALET_SMALL_LINE_HEIGHT),
                ),
                TextColor(Color::srgb(0.116_935_484, 0.665_322_6, 0.9)),
                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));
            attach_option_localization(&mut entity, label);
        });
}

pub(super) fn spawn_flag_button(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    request: SocialRequestKind,
    value: bool,
    label: &str,
    assets: &OptionUiAssets,
) {
    spawn_radio_button(
        parent,
        rect,
        OptionSocialFlagButton { request, value },
        OptionRadioIndicator::Social { request, value },
        label,
        assets,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_option_anchored_text(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    font: Handle<Font>,
    font_size: f32,
    line_height: f32,
    color: Color,
    anchor: OptionTextAnchor,
    line_break: LineBreak,
) {
    let text_width = if matches!(line_break, LineBreak::NoWrap) {
        Val::Auto
    } else {
        percent(100.0)
    };
    parent
        .spawn((
            Node {
                justify_content: anchor.justify_content(),
                align_items: anchor.align_items(),
                ..rect.node()
            },
            Pickable::IGNORE,
        ))
        .with_children(|wrapper| {
            let mut entity = wrapper.spawn((
                Node {
                    width: text_width,
                    max_width: percent(100.0),
                    ..default()
                },
                Text::new(value),
                (
                    TextFont {
                        font: (font).into(),
                        font_size: (font_size).into(),
                        ..default()
                    },
                    LineHeight::Px(line_height),
                ),
                TextColor(color),
                TextLayout::new(anchor.justify(), line_break),
                Pickable::IGNORE,
            ));
            attach_option_localization(&mut entity, value);
        });
}

pub(super) fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    font: Handle<Font>,
    font_size: f32,
    color: Color,
) {
    let rect = OptionUiRect::new(
        rect.x,
        rect.y + OPTION_CHALET_WHITE_LABEL_TOP_OFFSET,
        rect.width,
        rect.height,
    );
    spawn_option_anchored_text(
        parent,
        rect,
        value,
        font,
        font_size,
        OPTION_CHALET_SMALL_LINE_HEIGHT,
        color,
        OptionTextAnchor::MiddleLeft,
        LineBreak::WordBoundary,
    );
}

pub(super) fn spawn_footer_button(
    parent: &mut ChildSpawnerCommands,
    kind: OptionChromeButton,
    rect: OptionUiRect,
    label: &str,
    assets: &OptionUiAssets,
) {
    parent
        .spawn((
            Button,
            kind,
            option_button_node(rect),
            sliced_image(
                assets.image(OptionTextureRole::Button),
                OPTION_BIG_LABEL_BORDER,
            ),
            ZIndex(OPTION_CHROME_Z_INDEX),
            Pickable::default(),
        ))
        .with_children(|button| {
            let mut entity = button.spawn((
                OptionKeyButtonLabel,
                Text::new(label),
                (
                    TextFont {
                        font: (assets.jeffe.clone()).into(),
                        font_size: (OPTION_JEFFE_BUTTON_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(OPTION_JEFFE_14_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));
            attach_option_localization(&mut entity, label);
        });
}

pub(super) fn spawn_small_button<M: Bundle, N: Bundle>(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    label: &str,
    assets: &OptionUiAssets,
    marker_a: Option<M>,
    marker_b: Option<N>,
) {
    let mut entity = parent.spawn((
        Button,
        option_button_node(rect),
        sliced_image(
            assets.image(OptionTextureRole::Button),
            OPTION_BIG_LABEL_BORDER,
        ),
        Pickable::default(),
    ));
    if let Some(marker) = marker_a {
        entity.insert(marker);
    }
    if let Some(marker) = marker_b {
        entity.insert(marker);
    }
    entity.with_children(|button| {
        let mut entity = button.spawn((
            OptionButtonLabel,
            Text::new(label),
            (
                TextFont {
                    font: (assets.jeffe.clone()).into(),
                    font_size: (OPTION_JEFFE_BUTTON_FONT_SIZE).into(),
                    ..default()
                },
                LineHeight::Px(OPTION_JEFFE_14_LINE_HEIGHT),
            ),
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Pickable::IGNORE,
        ));
        attach_option_localization(&mut entity, label);
    });
}

pub(super) fn spawn_option_title(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    assets: &OptionUiAssets,
) {
    spawn_option_anchored_text(
        parent,
        rect,
        value,
        assets.jeffe.clone(),
        OPTION_JEFFE_TITLE_FONT_SIZE,
        OPTION_JEFFE_16_LINE_HEIGHT,
        Color::srgb(0.0, 0.0, 0.149_019_61),
        OptionTextAnchor::UpperLeft,
        LineBreak::NoWrap,
    );
}

pub(super) fn spawn_option_smallfont(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    assets: &OptionUiAssets,
) {
    spawn_option_anchored_text(
        parent,
        rect,
        value,
        assets.jeffe.clone(),
        12.0,
        OPTION_JEFFE_12_LINE_HEIGHT,
        Color::srgb(0.689_516_1, 1.0, 1.0),
        OptionTextAnchor::UpperRight,
        LineBreak::NoWrap,
    );
}

pub(super) fn spawn_option_smallcyan(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    assets: &OptionUiAssets,
) {
    spawn_option_anchored_text(
        parent,
        rect,
        value,
        assets.jeffe.clone(),
        12.0,
        OPTION_JEFFE_12_LINE_HEIGHT,
        Color::srgb(0.0, 0.383_064_5, 0.641_129),
        OptionTextAnchor::UpperLeft,
        LineBreak::NoWrap,
    );
}

pub(super) fn spawn_option_bigfont14(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    assets: &OptionUiAssets,
) {
    spawn_option_anchored_text(
        parent,
        rect,
        value,
        assets.jeffe.clone(),
        OPTION_JEFFE_BUTTON_FONT_SIZE,
        OPTION_JEFFE_14_LINE_HEIGHT,
        Color::srgb(0.504_032_25, 1.0, 1.0),
        OptionTextAnchor::MiddleRight,
        LineBreak::NoWrap,
    );
}

pub(super) fn spawn_option_default_label(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    value: &str,
    assets: &OptionUiAssets,
) {
    spawn_option_anchored_text(
        parent,
        rect,
        value,
        assets.chalet.clone(),
        12.0,
        OPTION_CHALET_SMALL_LINE_HEIGHT,
        Color::srgb(0.116_935_484, 0.665_322_6, 0.9),
        OptionTextAnchor::MiddleLeft,
        LineBreak::WordBoundary,
    );
}
