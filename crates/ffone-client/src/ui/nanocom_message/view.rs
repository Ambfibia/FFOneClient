use super::*;

pub(super) fn spawn_nanocom_message_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = NanocomMessageUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());

    commands
        .spawn((
            NanocomMessageUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(NANOCOM_UI_Z_INDEX),
        ))
        .with_children(|root| {
            root.spawn((
                NanocomMessageElement::CompactPanel,
                NanocomMessageCompactPanel,
                NanocomRect::new(0.0, 0.0, NANOCOM_MESSAGE_WIDTH, NANOCOM_MESSAGE_HEIGHT).node(),
                UiTransform::default(),
                Visibility::Hidden,
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    NanocomMessageElement::CompactFrame,
                    NanocomDrawRole::CompactFrame,
                    NANOCOM_COMPACT_FRAME_RECT.node(),
                    ImageNode {
                        image: assets.buddy_frame.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                panel
                    .spawn((
                        NanocomDrawRole::CompactTitle,
                        Node {
                            align_items: AlignItems::Center,
                            ..NANOCOM_COMPACT_TITLE_RECT.node()
                        },
                    ))
                    .with_child((
                        NanocomMessageElement::CompactTitle,
                        NanocomGuiStyleRole::BigFont14,
                        Node {
                            width: percent(100),
                            ..default()
                        },
                        Text::new(""),
                        nanocom_passthrough(""),
                        LocalizedTextCase::Uppercase,
                        (
                            TextFont {
                                font: (assets.jeffe.clone()).into(),
                                font_size: (NANOCOM_JEFFE_14_FONT_SIZE).into(),
                                ..default()
                            },
                            LineHeight::Px(NANOCOM_JEFFE_14_LINE_HEIGHT),
                        ),
                        TextColor(Color::srgb(0.0, 1.0, 1.0)),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                        Pickable::IGNORE,
                    ));
                panel.spawn((
                    NanocomMessageElement::CompactBody,
                    NanocomDrawRole::CompactBody,
                    NanocomGuiStyleRole::MessageText,
                    // A Text node does not offset its glyph origin by its own
                    // Bevy padding. Place it at the recovered IMGUI content
                    // rectangle so MenuMessageText's 10/4/6/6 padding is
                    // represented geometrically as in the clean client.
                    NANOCOM_COMPACT_BODY_CONTENT_RECT.node(),
                    Text::new(""),
                    nanocom_passthrough(""),
                    (
                        TextFont {
                            font: (assets.chalet.clone()).into(),
                            font_size: (NANOCOM_CHALET_SMALL_FONT_SIZE).into(),
                            ..default()
                        },
                        LineHeight::Px(NANOCOM_CHALET_SMALL_LINE_HEIGHT),
                    ),
                    TextColor(Color::srgb(1.0, 0.995_967_75, 1.0)),
                    TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    NanocomMessageElement::CompactIcon,
                    NanocomDrawRole::CompactIcon,
                    NANOCOM_COMPACT_ICON_RECT.node(),
                    ImageNode {
                        image: assets.buddy_icon.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            });
        });

    commands
        .spawn((
            NanocomMessageElement::ExpandedRoot,
            NanocomMessageExpandedRoot,
            NanocomDrawRole::ModalOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, NANOCOM_OVERLAY_ALPHA)),
            Visibility::Hidden,
            GlobalZIndex(NANOCOM_EXPANDED_Z_INDEX),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    NanocomMessageElement::ExpandedDialog,
                    NanocomMessageExpandedDialog,
                    NanocomDrawRole::ModalDialogBox,
                    Node {
                        overflow: Overflow::clip(),
                        ..NANOCOM_EXPANDED_DIALOG_RECT.node()
                    },
                    UiTransform::default(),
                    ImageNode {
                        image: assets.dialog.clone(),
                        visual_box: bevy::ui::VisualBox::BorderBox,
                        image_mode: NodeImageMode::Sliced(TextureSlicer {
                            border: NANOCOM_DIALOG_BORDER,
                            center_scale_mode: SliceScaleMode::Stretch,
                            sides_scale_mode: SliceScaleMode::Stretch,
                            max_corner_scale: 1.0,
                        }),
                        ..default()
                    },
                ))
                .with_children(|dialog| {
                    dialog.spawn((
                        NanocomDrawRole::ModalMessageArea,
                        NANOCOM_EXPANDED_MESSAGE_AREA_RECT.node(),
                        ImageNode {
                            image: assets.message_area.clone(),
                            visual_box: bevy::ui::VisualBox::BorderBox,
                            image_mode: NodeImageMode::Sliced(TextureSlicer {
                                border: NANOCOM_MESSAGE_AREA_BORDER,
                                center_scale_mode: SliceScaleMode::Stretch,
                                sides_scale_mode: SliceScaleMode::Stretch,
                                max_corner_scale: 1.0,
                            }),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    dialog
                        .spawn((
                            NanocomDrawRole::ModalIcon,
                            Node {
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..NANOCOM_EXPANDED_ICON_RECT.node()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_child((
                            NanocomMessageElement::ExpandedIcon,
                            Node {
                                width: px(62),
                                height: px(62.0 * 63.0 / 64.0),
                                ..default()
                            },
                            ImageNode {
                                image: assets.buddy_icon.clone(),
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    dialog
                        .spawn((
                            NanocomDrawRole::ModalTitle,
                            Node {
                                align_items: AlignItems::Center,
                                ..NANOCOM_EXPANDED_TITLE_RECT.node()
                            },
                        ))
                        .with_child((
                            NanocomMessageElement::ExpandedTitle,
                            NanocomGuiStyleRole::MessageTitle,
                            Node {
                                width: percent(100),
                                ..default()
                            },
                            Text::new(NANOCOM_EXPANDED_TITLE),
                            LocalizedText::new(
                                NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY,
                                NANOCOM_EXPANDED_TITLE,
                            ),
                            (
                                TextFont {
                                    font: (assets.jeffe.clone()).into(),
                                    font_size: (NANOCOM_MESSAGE_TITLE_FONT_SIZE).into(),
                                    ..default()
                                },
                                LineHeight::Px(NANOCOM_MESSAGE_TITLE_LINE_HEIGHT),
                            ),
                            TextColor(Color::srgb(0.0, 1.0, 1.0)),
                            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                            Pickable::IGNORE,
                        ));
                    dialog
                        .spawn((
                            NanocomDrawRole::ModalTextArea,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Stretch,
                                row_gap: px(4),
                                overflow: Overflow::clip(),
                                ..NANOCOM_EXPANDED_TEXT_RECT.node()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|text_area| {
                            text_area.spawn((
                                NanocomMessageElement::ExpandedBody,
                                NanocomGuiStyleRole::CenterBox2,
                                Node {
                                    width: percent(100),
                                    flex_shrink: 0.0,
                                    padding: nanocom_gui_style(NanocomGuiStyleRole::CenterBox2)
                                        .padding
                                        .ui_rect(),
                                    ..default()
                                },
                                Text::new(""),
                                nanocom_passthrough(""),
                                (
                                    TextFont {
                                        font: (assets.chalet.clone()).into(),
                                        font_size: (NANOCOM_CHALET_SMALL_FONT_SIZE).into(),
                                        ..default()
                                    },
                                    LineHeight::Px(NANOCOM_CHALET_SMALL_LINE_HEIGHT),
                                ),
                                TextColor(Color::WHITE),
                                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                                Pickable::IGNORE,
                            ));
                            text_area.spawn((
                                NanocomMessageElement::ExpandedExpiration,
                                NanocomGuiStyleRole::CenterBox2,
                                Node {
                                    width: percent(100),
                                    flex_shrink: 0.0,
                                    padding: nanocom_gui_style(NanocomGuiStyleRole::CenterBox2)
                                        .padding
                                        .ui_rect(),
                                    ..default()
                                },
                                Text::new(""),
                                nanocom_passthrough(""),
                                (
                                    TextFont {
                                        font: (assets.chalet.clone()).into(),
                                        font_size: (NANOCOM_CHALET_SMALL_FONT_SIZE).into(),
                                        ..default()
                                    },
                                    LineHeight::Px(NANOCOM_CHALET_SMALL_LINE_HEIGHT),
                                ),
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                                Pickable::IGNORE,
                            ));
                        });
                    spawn_nanocom_button(
                        dialog,
                        NANOCOM_ACCEPT_RECT,
                        NanocomMessageChoice::Accept,
                        NANOCOM_ACCEPT_LABEL,
                        &assets,
                    );
                    spawn_nanocom_button(
                        dialog,
                        NANOCOM_DECLINE_RECT,
                        NanocomMessageChoice::Decline,
                        NANOCOM_DECLINE_LABEL,
                        &assets,
                    );
                });
        });
}

pub(super) fn spawn_nanocom_button(
    parent: &mut ChildSpawnerCommands,
    rect: NanocomRect,
    choice: NanocomMessageChoice,
    label: &'static str,
    assets: &NanocomMessageUiAssets,
) {
    let style_role = match choice {
        NanocomMessageChoice::Accept => NanocomGuiStyleRole::Button,
        NanocomMessageChoice::Decline => NanocomGuiStyleRole::RedButton,
    };
    let style = nanocom_gui_style(style_role);
    parent
        .spawn((
            Button,
            match choice {
                NanocomMessageChoice::Accept => NanocomDrawRole::ModalAccept,
                NanocomMessageChoice::Decline => NanocomDrawRole::ModalDecline,
            },
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: style.padding.ui_rect(),
                ..rect.node()
            },
            ImageNode {
                image: assets.button_image(choice, Interaction::None),
                visual_box: bevy::ui::VisualBox::BorderBox,
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: NANOCOM_BUTTON_BORDER,
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
            NanocomMessageButton {
                choice,
                presented_request_id: None,
            },
        ))
        .with_child((
            NanocomMessageButtonLabel(choice),
            style_role,
            Text::new(label),
            LocalizedText::new(
                match choice {
                    NanocomMessageChoice::Accept => NANOCOM_ACCEPT_LOCALIZATION_KEY,
                    NanocomMessageChoice::Decline => NANOCOM_DECLINE_LOCALIZATION_KEY,
                },
                label,
            ),
            (
                TextFont {
                    font: (assets.jeffe.clone()).into(),
                    font_size: (NANOCOM_JEFFE_14_FONT_SIZE).into(),
                    ..default()
                },
                LineHeight::Px(NANOCOM_JEFFE_14_LINE_HEIGHT),
            ),
            TextColor(nanocom_button_text_color(choice, Interaction::None)),
            TextLayout::new(
                Justify::Center,
                match choice {
                    NanocomMessageChoice::Accept => LineBreak::NoWrap,
                    NanocomMessageChoice::Decline => LineBreak::WordBoundary,
                },
            ),
            Pickable::IGNORE,
        ));
}
