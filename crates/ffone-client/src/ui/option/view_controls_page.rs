use super::*;

pub(super) fn spawn_game_ui_page(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    parent
        .spawn((
            OptionPage(OptionTab::GameUi),
            Node {
                display: Display::None,
                ..OPTION_PAGE_RECT.node()
            },
            Pickable::IGNORE,
            ZIndex(OPTION_PAGE_Z_INDEX),
        ))
        .with_children(|page| {
            for rect in [OPTION_DISPLAY_HEADER_RECT, OPTION_CHAT_HEADER_RECT] {
                page.spawn((
                    OptionSkyHeader,
                    rect.node(),
                    stretched_image(assets.image(OptionTextureRole::Frame)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OPTION_DISPLAY_BODY_RECT,
                OPTION_DISPLAY_SIDE_RECT,
                OPTION_CHAT_BODY_RECT,
                OPTION_CHAT_SIDE_RECT,
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2` has a zero border: Stretch is the serialized behavior.
                    stretched_image(assets.image(OptionTextureRole::PanelFlat)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OptionUiRect::new(660.0, 47.0, 26.0, 27.0),
                OptionUiRect::new(660.0, 327.0, 26.0, 27.0),
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2_inter` has a zero border and is used at its native 26x27.
                    stretched_image(assets.image(OptionTextureRole::PanelNotch)),
                    Pickable::IGNORE,
                ));
            }
            page.spawn((
                OptionUiRect::new(70.0, 60.0, 239.0, 36.0).node(),
                sliced_image(
                    assets.image(OptionTextureRole::PanelConnector),
                    OPTION_PANEL_CONNECTOR_BORDER,
                ),
                Pickable::IGNORE,
            ));
            page.spawn((
                OptionUiRect::new(70.0, 81.0, 370.0, 179.0).node(),
                sliced_image(
                    assets.image(OptionTextureRole::PanelSide),
                    OPTION_DARK_BOX_BORDER,
                ),
                Pickable::IGNORE,
            ));
            spawn_option_title(
                page,
                OptionUiRect::new(20.0, 28.0, 200.0, 50.0),
                "DISPLAY ELEMENTS",
                assets,
            );
            spawn_option_title(
                page,
                OptionUiRect::new(20.0, 308.0, 200.0, 50.0),
                "CHAT TEXT COLORS",
                assets,
            );
            spawn_small_button(
                page,
                OptionUiRect::new(690.0, 30.0, 250.0, 25.0),
                "DEFAULT DISPLAY ELEMENTS",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionDisplayDefaultsButton),
            );
            spawn_small_button(
                page,
                OptionUiRect::new(690.0, 310.0, 250.0, 25.0),
                "DEFAULT TEXT COLORS",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionTextColorDefaultsButton),
            );
            spawn_option_smallcyan(
                page,
                OptionUiRect::new(100.0, 70.0, 150.0, 50.0),
                "FLOATING DISPLAY",
                assets,
            );

            for rect in [
                OptionUiRect::new(90.0, 360.0, 194.0, 120.0),
                OptionUiRect::new(380.0, 360.0, 194.0, 120.0),
                OptionUiRect::new(670.0, 360.0, 194.0, 120.0),
            ] {
                page.spawn((
                    OptionSkyHeader,
                    rect.node(),
                    stretched_image(assets.image(OptionTextureRole::Frame)),
                    Pickable::IGNORE,
                ));
            }

            let left = &OptionDisplayElement::ALL[..6];
            let right = &OptionDisplayElement::ALL[6..];
            for (index, &element) in left.iter().enumerate() {
                spawn_display_row(
                    page,
                    element,
                    0.0,
                    100.0 + index as f32 * 25.0,
                    310.0,
                    95.0 + index as f32 * 26.0,
                    assets,
                );
            }
            for (index, &element) in right.iter().enumerate() {
                spawn_display_row(
                    page,
                    element,
                    450.0,
                    90.0 + index as f32 * 25.0,
                    760.0,
                    85.0 + index as f32 * 26.0,
                    assets,
                );
            }

            for (channel_index, channel) in OptionTextColorChannel::ALL.into_iter().enumerate() {
                let x_offset = channel_index as f32 * 290.0;
                spawn_option_smallcyan(
                    page,
                    OptionUiRect::new(100.0 + x_offset, 365.0, 150.0, 50.0),
                    channel.label(),
                    assets,
                );
                for index in 0..18_u8 {
                    let column = f32::from(index % 6);
                    let row = f32::from(index / 6);
                    let (red, green, blue) = OPTION_CHAT_PALETTE_RGB[usize::from(index)];
                    let mut swatch_image =
                        stretched_image(assets.image(OptionTextureRole::ColorBox));
                    swatch_image.color = Color::srgb(red, green, blue);
                    page.spawn((
                        Button,
                        OptionTextColorButton { channel, index },
                        OptionUiRect::new(
                            100.0 + x_offset + column * 30.0,
                            390.0 + row * 30.0,
                            20.0,
                            20.0,
                        )
                        .node(),
                        swatch_image,
                        Pickable::default(),
                    ))
                    .with_children(|swatch| {
                        swatch.spawn((
                            OptionTextColorSelection { channel, index },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(-2.0),
                                top: px(-2.0),
                                width: px(24.0),
                                height: px(24.0),
                                ..default()
                            },
                            stretched_image(assets.image(OptionTextureRole::ColorSelected)),
                            Visibility::Hidden,
                            Pickable::IGNORE,
                        ));
                    });
                    spawn_option_anchored_text(
                        page,
                        OptionUiRect::new(
                            103.0 + x_offset + column * 30.0,
                            387.0 + row * 30.0,
                            20.0,
                            20.0,
                        ),
                        "Aa",
                        assets.chalet.clone(),
                        10.0,
                        OPTION_CHALET_SMALL_LINE_HEIGHT,
                        Color::WHITE,
                        OptionTextAnchor::MiddleLeft,
                        LineBreak::NoWrap,
                    );
                }
            }
        });
}

pub(super) fn spawn_display_row(
    parent: &mut ChildSpawnerCommands,
    element: OptionDisplayElement,
    label_x: f32,
    label_y: f32,
    radio_x: f32,
    radio_y: f32,
    assets: &OptionUiAssets,
) {
    spawn_option_smallfont(
        parent,
        OptionUiRect::new(label_x, label_y, 300.0, 5.0),
        &format!("{}:", element.label()),
        assets,
    );
    for (x, value, label) in [(radio_x, true, "On"), (radio_x + 60.0, false, "Off")] {
        spawn_radio_button(
            parent,
            OptionUiRect::new(x, radio_y, 22.0, 21.0),
            OptionDisplayFlagButton { element, value },
            OptionRadioIndicator::Display { element, value },
            label,
            assets,
        );
    }
}

pub(super) fn spawn_controls_page(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    parent
        .spawn((
            OptionPage(OptionTab::Controls),
            Node {
                display: Display::None,
                ..OPTION_PAGE_RECT.node()
            },
            Pickable::IGNORE,
            ZIndex(OPTION_PAGE_Z_INDEX),
        ))
        .with_children(|page| {
            for rect in [OPTION_CONTROLS_HEADER_RECT, OPTION_KEYMAP_HEADER_RECT] {
                page.spawn((
                    OptionSkyHeader,
                    rect.node(),
                    stretched_image(assets.image(OptionTextureRole::Frame)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OPTION_CONTROLS_INPUT_BODY_RECT,
                OPTION_CONTROLS_INPUT_SIDE_RECT,
                OptionUiRect::new(10.0, 190.0, 650.0, 320.0),
                OptionUiRect::new(460.0, 204.0, 485.0, 306.0),
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2` has a zero border: Stretch is the serialized behavior.
                    stretched_image(assets.image(OptionTextureRole::PanelFlat)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OptionUiRect::new(660.0, 47.0, 26.0, 27.0),
                OptionUiRect::new(660.0, 187.0, 26.0, 27.0),
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2_inter` has a zero border and is used at its native 26x27.
                    stretched_image(assets.image(OptionTextureRole::PanelNotch)),
                    Pickable::IGNORE,
                ));
            }
            spawn_option_title(
                page,
                OptionUiRect::new(20.0, 28.0, 200.0, 50.0),
                "INPUT CONTROLS",
                assets,
            );
            spawn_option_title(
                page,
                OptionUiRect::new(20.0, 168.0, 200.0, 50.0),
                "KEY MAPPING",
                assets,
            );
            spawn_small_button(
                page,
                OptionUiRect::new(690.0, 30.0, 250.0, 25.0),
                "DEFAULT INPUT CONTROLS",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionInputDefaultsButton),
            );
            spawn_small_button(
                page,
                OptionUiRect::new(690.0, 170.0, 250.0, 25.0),
                "DEFAULT KEY MAPPING",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionKeyMappingDefaultsButton),
            );
            spawn_option_smallfont(
                page,
                OptionUiRect::new(30.0, 90.0, 200.0, 50.0),
                "INVERT CAMERA Y-AXIS:",
                assets,
            );
            for (x, value, label) in [(250.0, true, "Yes"), (310.0, false, "No")] {
                spawn_radio_button(
                    page,
                    OptionUiRect::new(x, 88.0, 22.0, 21.0),
                    OptionInvertYButton(value),
                    OptionRadioIndicator::InvertY(value),
                    label,
                    assets,
                );
            }
            spawn_option_smallfont(
                page,
                OptionUiRect::new(450.0, 90.0, 200.0, 50.0),
                "CAMERA SENSITIVITY:",
                assets,
            );
            spawn_option_default_label(
                page,
                OptionUiRect::new(655.0, 70.0, 50.0, 50.0),
                "Low",
                assets,
            );
            page.spawn((
                OptionUiRect::new(680.0, 90.0, 170.0, 16.0).node(),
                stretched_image(assets.image(OptionTextureRole::SliderTrack)),
                Pickable::IGNORE,
            ));
            page.spawn((
                OptionSensitivityThumb,
                OptionUiRect::new(
                    680.0,
                    85.0,
                    OPTION_SLIDER_THUMB_WIDTH,
                    OPTION_SLIDER_THUMB_HEIGHT,
                )
                .node(),
                stretched_image(assets.image(OptionTextureRole::SliderThumb)),
                Pickable::IGNORE,
            ));
            let sensitivity_width = 170.0 / 10.0;
            for value in 1..=10_u8 {
                page.spawn((
                    Button,
                    OptionSensitivityButton(value),
                    OptionUiRect::new(
                        680.0 + f32::from(value - 1) * sensitivity_width,
                        85.0,
                        sensitivity_width,
                        OPTION_SLIDER_THUMB_HEIGHT,
                    )
                    .node(),
                    Pickable::default(),
                ));
            }
            spawn_option_default_label(
                page,
                OptionUiRect::new(855.0, 70.0, 50.0, 50.0),
                "High",
                assets,
            );

            spawn_option_smallfont(
                page,
                OptionUiRect::new(30.0, 130.0, 200.0, 50.0),
                "PAD INVERT Y-AXIS:",
                assets,
            );
            for (x, value, label) in [(250.0, true, "Yes"), (310.0, false, "No")] {
                spawn_radio_button(
                    page,
                    OptionUiRect::new(x, 128.0, 22.0, 21.0),
                    OptionPadInvertYButton(value),
                    OptionRadioIndicator::PadInvertY(value),
                    label,
                    assets,
                );
            }
            spawn_option_smallfont(
                page,
                OptionUiRect::new(450.0, 130.0, 200.0, 50.0),
                "PAD SENSITIVITY:",
                assets,
            );
            spawn_option_default_label(
                page,
                OptionUiRect::new(655.0, 110.0, 50.0, 50.0),
                "Low",
                assets,
            );
            page.spawn((
                OptionUiRect::new(680.0, 130.0, 170.0, 16.0).node(),
                stretched_image(assets.image(OptionTextureRole::SliderTrack)),
                Pickable::IGNORE,
            ));
            page.spawn((
                OptionPadSensitivityThumb,
                OptionUiRect::new(
                    680.0,
                    125.0,
                    OPTION_SLIDER_THUMB_WIDTH,
                    OPTION_SLIDER_THUMB_HEIGHT,
                )
                .node(),
                stretched_image(assets.image(OptionTextureRole::SliderThumb)),
                Pickable::IGNORE,
            ));
            let sensitivity_width = 170.0 / 10.0;
            for value in 1..=10_u8 {
                page.spawn((
                    Button,
                    OptionPadSensitivityButton(value),
                    OptionUiRect::new(
                        680.0 + f32::from(value - 1) * sensitivity_width,
                        125.0,
                        sensitivity_width,
                        OPTION_SLIDER_THUMB_HEIGHT,
                    )
                    .node(),
                    Pickable::default(),
                ));
            }
            spawn_option_default_label(
                page,
                OptionUiRect::new(855.0, 110.0, 50.0, 50.0),
                "High",
                assets,
            );

            spawn_option_bigfont14(
                page,
                OptionUiRect::new(20.0, 188.0, 210.0, 21.0),
                "Selected Gamepad :",
                assets,
            );
            spawn_dropdown_button(
                page,
                OptionUiRect::new(250.0, 190.0, 223.0, 21.0),
                OptionDropdownKind::Pad,
                assets,
            );
            page.spawn((
                OptionKeyCapturePrompt,
                OptionUiRect::new(30.0, 210.0, 800.0, 50.0).node(),
                Text::new(""),
                option_passthrough_text(""),
                (
                    TextFont {
                        font: (assets.chalet.clone()).into(),
                        font_size: (12.0).into(),
                        ..default()
                    },
                    LineHeight::Px(OPTION_CHALET_LINE_HEIGHT),
                ),
                TextColor(Color::srgb(1.0, 0.921_568_6, 0.015_686_275)),
                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));

            page.spawn((
                OPTION_KEYMAP_VIEW_RECT.node(),
                sliced_image(
                    assets.image(OptionTextureRole::TextField),
                    OPTION_TEXT_FIELD_BORDER,
                ),
                Pickable::IGNORE,
            ));
            page.spawn((
                Node {
                    overflow: Overflow::clip(),
                    ..OPTION_KEYMAP_CONTENT_RECT.node()
                },
                Pickable::default(),
            ))
            .with_children(|viewport| {
                viewport
                    .spawn((
                        OptionControlsScrollContent,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0.0),
                            top: px(0.0),
                            width: px(900.0),
                            height: px(OPTION_CONTROL_CONTENT_HEIGHT),
                            ..default()
                        },
                        Pickable::default(),
                    ))
                    .with_children(|content| spawn_control_mapping_content(content, assets));
            });
            page.spawn((
                OptionUiRect::new(898.0, 235.0, OPTION_SCROLLBAR_VISUAL_WIDTH, 250.0).node(),
                sliced_image(
                    assets.image(OptionTextureRole::ScrollBar),
                    OPTION_SCROLLBAR_BORDER,
                ),
                Pickable::IGNORE,
            ));
            for (direction, y, role) in [
                (
                    OptionControlsScrollButton::Up,
                    235.0,
                    OptionTextureRole::ScrollUp,
                ),
                (
                    OptionControlsScrollButton::Down,
                    473.0,
                    OptionTextureRole::ScrollDown,
                ),
            ] {
                page.spawn((
                    Button,
                    direction,
                    OptionUiRect::new(898.0, y, 17.0, 12.0).node(),
                    stretched_image(assets.image(role)),
                    Pickable::default(),
                ));
            }
            page.spawn((
                OptionControlsScrollThumb,
                OptionUiRect::new(897.0, 245.0, OPTION_SCROLL_THUMB_VISUAL_WIDTH, 54.0).node(),
                sliced_image(
                    assets.image(OptionTextureRole::ScrollThumb),
                    OPTION_SCROLL_THUMB_BORDER,
                ),
                Pickable::IGNORE,
            ));

            let pad_choices = LegacyPadProfile::ALL
                .into_iter()
                .map(|profile| {
                    (
                        profile.label().to_owned(),
                        OptionDropdownChoice::Pad(profile),
                    )
                })
                .collect::<Vec<_>>();
            spawn_dropdown_panel(
                page,
                OptionDropdownKind::Pad,
                OptionUiRect::new(250.0, 190.0, 200.0, 100.0),
                &pad_choices,
                assets,
                30.0,
            );
        });
}

pub(super) fn spawn_control_mapping_content(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    const INTERFACE_VISIBLE: [LegacyOptionAction; 5] = [
        LegacyOptionAction::NanoBook,
        LegacyOptionAction::Inventory,
        LegacyOptionAction::Journal,
        LegacyOptionAction::Email,
        LegacyOptionAction::WorldMap,
    ];
    const CAMERA_VISIBLE: [LegacyOptionAction; 6] = [
        LegacyOptionAction::CameraUp,
        LegacyOptionAction::CameraDown,
        LegacyOptionAction::CameraLeft,
        LegacyOptionAction::CameraRight,
        LegacyOptionAction::ZoomIn,
        LegacyOptionAction::ZoomOut,
    ];
    const COMBAT_VISIBLE: [LegacyOptionAction; 8] = [
        LegacyOptionAction::Fire1,
        LegacyOptionAction::WeaponChange,
        LegacyOptionAction::Fire2,
        LegacyOptionAction::NanoCharge,
        LegacyOptionAction::Nano1,
        LegacyOptionAction::Nano2,
        LegacyOptionAction::Nano3,
        LegacyOptionAction::VehicleToggle,
    ];
    let sections: [(
        OptionControlGroup,
        OptionUiRect,
        OptionUiRect,
        OptionUiRect,
        f32,
        &[LegacyOptionAction],
    ); 4] = [
        (
            OptionControlGroup::Movement,
            OptionUiRect::new(10.0, 12.0, 169.0, 36.0),
            OptionUiRect::new(10.0, 35.0, 900.0, 280.0),
            OptionUiRect::new(20.0, 20.0, 200.0, 50.0),
            60.0,
            &OPTION_MOVEMENT_ACTIONS,
        ),
        (
            OptionControlGroup::Interface,
            OptionUiRect::new(10.0, 340.0, 169.0, 36.0),
            OptionUiRect::new(10.0, 363.0, 900.0, 180.0),
            OptionUiRect::new(20.0, 349.0, 200.0, 50.0),
            390.0,
            &INTERFACE_VISIBLE,
        ),
        (
            OptionControlGroup::Camera,
            OptionUiRect::new(10.0, 575.0, 139.0, 36.0),
            OptionUiRect::new(10.0, 595.0, 870.0, 220.0),
            OptionUiRect::new(20.0, 584.0, 200.0, 50.0),
            625.0,
            &CAMERA_VISIBLE,
        ),
        (
            OptionControlGroup::Combat,
            OptionUiRect::new(10.0, 835.0, 139.0, 36.0),
            OptionUiRect::new(10.0, 855.0, 870.0, 280.0),
            OptionUiRect::new(20.0, 844.0, 200.0, 50.0),
            885.0,
            &COMBAT_VISIBLE,
        ),
    ];
    for (group, header_rect, body_rect, title_rect, rows_y, actions) in sections {
        parent.spawn((
            body_rect.node(),
            sliced_image(
                assets.image(OptionTextureRole::PanelSide),
                OPTION_DARK_BOX_BORDER,
            ),
            Pickable::IGNORE,
        ));
        parent.spawn((
            header_rect.node(),
            sliced_image(
                assets.image(OptionTextureRole::PanelConnector),
                OPTION_PANEL_CONNECTOR_BORDER,
            ),
            Pickable::IGNORE,
        ));
        spawn_option_title(parent, title_rect, group.label(), assets);
        for (slot_index, slot) in OptionInputMappingSlot::ALL.into_iter().enumerate() {
            spawn_option_default_label(
                parent,
                OptionUiRect::new(
                    180.0 + slot_index as f32 * 220.0,
                    header_rect.y + 8.0,
                    180.0,
                    25.0,
                ),
                slot.label(),
                assets,
            );
        }
        for (row_index, &action) in actions.iter().enumerate() {
            let y = rows_y + row_index as f32 * 30.0;
            spawn_option_smallfont(
                parent,
                OptionUiRect::new(10.0, y, 150.0, 20.0),
                action.label(),
                assets,
            );
            for (slot_index, slot) in OptionInputMappingSlot::ALL.into_iter().enumerate() {
                parent
                    .spawn((
                        Button,
                        OptionMappingButton { action, slot },
                        OptionPageButtonVisual,
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect {
                                left: px(6.0),
                                right: px(6.0),
                                top: px(3.0),
                                bottom: px(6.0),
                            },
                            ..OptionUiRect::new(180.0 + slot_index as f32 * 220.0, y, 180.0, 25.0)
                                .node()
                        },
                        sliced_image(
                            assets.image(OptionTextureRole::Button),
                            OPTION_BIG_LABEL_BORDER,
                        ),
                        Pickable::default(),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            OptionMappingLabel { action, slot },
                            OptionKeyButtonLabel,
                            Text::new(""),
                            option_passthrough_text(""),
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
                    });
            }
        }
    }
}

pub(super) fn spawn_language_extension(
    parent: &mut ChildSpawnerCommands,
    assets: &OptionUiAssets,
    localization: Option<&Localization>,
    audio_catalog: Option<&NativeAudioCatalog>,
) {
    parent
        .spawn((
            OptionLanguageExtensionRoot,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: px(OPTION_PAGE_RECT.x),
                top: px(OPTION_PAGE_RECT.y),
                width: px(OPTION_REFERENCE_WIDTH),
                height: px(OPTION_REFERENCE_HEIGHT),
                ..default()
            },
            ZIndex(OPTION_CHROME_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|extension| {
            extension.spawn((
                OptionSkyHeader,
                OPTION_LANGUAGE_HEADER_RECT.node(),
                stretched_image(assets.image(OptionTextureRole::Frame)),
                Pickable::IGNORE,
            ));
            extension.spawn((
                OPTION_LANGUAGE_BODY_RECT.node(),
                stretched_image(assets.image(OptionTextureRole::PanelFlat)),
                Pickable::IGNORE,
            ));
            spawn_option_title(
                extension,
                OptionUiRect::new(20.0, 28.0, 400.0, 50.0),
                "GAME LANGUAGE",
                assets,
            );
            for (y, label) in [(70.0, "TEXT LANGUAGE:"), (118.0, "VOICE LANGUAGE:")] {
                extension.spawn((
                    OptionUiRect::new(45.0, y, 380.0, 43.0).node(),
                    sliced_image(
                        assets.image(OptionTextureRole::PanelSmall),
                        OPTION_DARK_BOX_BORDER,
                    ),
                    Pickable::IGNORE,
                ));
                spawn_option_anchored_text(
                    extension,
                    OptionUiRect::new(58.0, y, 170.0, 43.0),
                    label,
                    assets.jeffe.clone(),
                    12.0,
                    OPTION_JEFFE_12_LINE_HEIGHT,
                    Color::srgb(0.689_516_1, 1.0, 1.0),
                    OptionTextAnchor::MiddleRight,
                    LineBreak::NoWrap,
                );
            }
            spawn_text(
                extension,
                OptionUiRect::new(45.0, 169.0, 380.0, 25.0),
                "Text and voice can be changed independently.",
                assets.chalet.clone(),
                OPTION_CHALET_WHITE_LABEL_FONT_SIZE,
                Color::WHITE,
            );
            spawn_dropdown_button(
                extension,
                OPTION_TRANSLATION_RECT,
                OptionDropdownKind::Translation,
                assets,
            );
            spawn_dropdown_button(
                extension,
                OPTION_VOICE_LANGUAGE_RECT,
                OptionDropdownKind::Voice,
                assets,
            );
            let language_choices = localization
                .map(|localization| {
                    localization
                        .locales()
                        .map(|locale| {
                            (
                                locale_label(locale),
                                OptionDropdownChoice::Translation(locale.to_owned()),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| {
                    vec![
                        (
                            "ENGLISH".to_owned(),
                            OptionDropdownChoice::Translation("en".to_owned()),
                        ),
                        (
                            "RUSSIAN".to_owned(),
                            OptionDropdownChoice::Translation("ru".to_owned()),
                        ),
                    ]
                });
            spawn_dropdown_panel(
                extension,
                OptionDropdownKind::Translation,
                OptionUiRect::new(
                    OPTION_TRANSLATION_RECT.x,
                    OPTION_TRANSLATION_RECT.y + OPTION_TRANSLATION_RECT.height,
                    OPTION_TRANSLATION_RECT.width,
                    (40.0 + language_choices.len() as f32 * 20.0).max(80.0),
                ),
                &language_choices,
                assets,
                30.0,
            );
            let voice_choices = audio_catalog
                .map(|catalog| {
                    catalog
                        .voice_locales()
                        .map(|locale| {
                            (
                                locale_label(locale),
                                OptionDropdownChoice::Voice(locale.to_owned()),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| {
                    vec![
                        (
                            "ENGLISH".to_owned(),
                            OptionDropdownChoice::Voice("en".to_owned()),
                        ),
                        (
                            "RUSSIAN".to_owned(),
                            OptionDropdownChoice::Voice("ru".to_owned()),
                        ),
                    ]
                });
            spawn_dropdown_panel(
                extension,
                OptionDropdownKind::Voice,
                OptionUiRect::new(
                    OPTION_VOICE_LANGUAGE_RECT.x,
                    OPTION_VOICE_LANGUAGE_RECT.y + OPTION_VOICE_LANGUAGE_RECT.height,
                    OPTION_VOICE_LANGUAGE_RECT.width,
                    (40.0 + voice_choices.len() as f32 * 20.0).max(80.0),
                ),
                &voice_choices,
                assets,
                30.0,
            );
        });
}
