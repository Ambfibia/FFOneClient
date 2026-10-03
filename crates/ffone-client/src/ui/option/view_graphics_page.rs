use super::*;

pub(super) fn attach_option_localization(entity: &mut EntityCommands, source: &str) {
    entity.insert(option_localized_text(source).unwrap_or_else(|| option_passthrough_text(source)));
}

pub(super) fn spawn_option_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    localization: Option<Res<Localization>>,
    audio_catalog: Option<Res<NativeAudioCatalog>>,
) {
    let assets = OptionUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            OptionUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Visibility::Hidden,
            Pickable::default(),
            GlobalZIndex(OPTION_UI_Z_INDEX),
        ))
        .with_children(|root| {
            root.spawn((
                OptionUiWindow,
                OPTION_WINDOW_RECT.node(),
                UiTransform::default(),
                Pickable::default(),
            ))
            .with_children(|window| {
                window.spawn((
                    OptionUiBackdrop,
                    OPTION_BACKDROP_RECT.node(),
                    stretched_image(assets.image(OptionTextureRole::Backdrop)),
                    Pickable::IGNORE,
                ));
                window.spawn((
                    OPTION_FRAME_RECT.node(),
                    sliced_image(
                        assets.image(OptionTextureRole::OuterFrame),
                        OPTION_FRAME_BORDER,
                    ),
                    ZIndex(OPTION_FRAME_Z_INDEX),
                    Pickable::IGNORE,
                ));

                spawn_graphics_page(window, &assets);
                spawn_game_ui_page(window, &assets);
                spawn_social_page(window, &assets);
                spawn_controls_page(window, &assets);

                for tab in OptionTab::ALL {
                    let rect = tab.normal_rect();
                    let (justify_content, padding) = option_tab_content_style(tab, false);
                    window
                        .spawn((
                            Button,
                            OptionTabButton(tab),
                            Node {
                                justify_content,
                                align_items: AlignItems::Center,
                                padding,
                                ..rect.node()
                            },
                            option_tab_image(assets.image(tab_normal_role(tab)), tab, false),
                            ZIndex(OPTION_NORMAL_TAB_Z_INDEX),
                            Pickable::default(),
                        ))
                        .with_children(|button| {
                            let label = tab_label(tab);
                            let mut entity = button.spawn((
                                OptionTabLabel(tab),
                                Text::new(label),
                                (
                                    TextFont {
                                        font: (assets.jeffe.clone()).into(),
                                        font_size: (OPTION_JEFFE_TAB_FONT_SIZE).into(),
                                        ..default()
                                    },
                                    LineHeight::Px(OPTION_JEFFE_13_LINE_HEIGHT),
                                ),
                                TextColor(Color::srgb(0.8, 1.0, 1.0)),
                                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                                Pickable::IGNORE,
                            ));
                            attach_option_localization(&mut entity, label);
                        });
                }

                window.spawn((
                    Button,
                    OptionChromeButton::Close,
                    OPTION_CLOSE_RECT.node(),
                    sliced_image(assets.image(OptionTextureRole::Close), OPTION_CLOSE_BORDER),
                    ZIndex(OPTION_CHROME_Z_INDEX),
                    Pickable::default(),
                ));
                spawn_footer_button(
                    window,
                    OptionChromeButton::Apply,
                    OPTION_APPLY_RECT,
                    "APPLY CHANGE",
                    &assets,
                );
                spawn_footer_button(
                    window,
                    OptionChromeButton::Save,
                    OPTION_SAVE_RECT,
                    "SAVE AND EXIT",
                    &assets,
                );
                spawn_language_extension(
                    window,
                    &assets,
                    localization.as_deref(),
                    audio_catalog.as_deref(),
                );
                spawn_system_popup(window, &assets);
            });
        });
}

pub(super) fn spawn_graphics_page(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    parent
        .spawn((
            OptionPage(OptionTab::Graphics),
            Node {
                display: Display::None,
                ..OPTION_PAGE_RECT.node()
            },
            Pickable::IGNORE,
            ZIndex(OPTION_PAGE_Z_INDEX),
        ))
        .with_children(|page| {
            for rect in [
                OPTION_GRAPHICS_LEFT_HEADER_RECT,
                OPTION_GRAPHICS_RIGHT_HEADER_RECT,
            ] {
                page.spawn((
                    OptionSkyHeader,
                    rect.node(),
                    stretched_image(assets.image(OptionTextureRole::Frame)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OPTION_GRAPHICS_LEFT_BODY_RECT,
                OPTION_GRAPHICS_LEFT_SIDE_RECT,
                OPTION_GRAPHICS_RIGHT_BODY_RECT,
                OPTION_GRAPHICS_RIGHT_SIDE_RECT,
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2` has a zero border: Stretch is the serialized behavior.
                    stretched_image(assets.image(OptionTextureRole::PanelFlat)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OptionUiRect::new(248.0, 47.0, 26.0, 27.0),
                OptionUiRect::new(730.0, 47.0, 26.0, 27.0),
            ] {
                page.spawn((
                    rect.node(),
                    // Clean `darkbox2_inter` has a zero border and is used at its native 26x27.
                    stretched_image(assets.image(OptionTextureRole::PanelNotch)),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OptionUiRect::new(20.0, 160.0, 239.0, 36.0),
                OptionUiRect::new(500.0, 150.0, 209.0, 36.0),
            ] {
                page.spawn((
                    rect.node(),
                    sliced_image(
                        assets.image(OptionTextureRole::PanelConnector),
                        OPTION_PANEL_CONNECTOR_BORDER,
                    ),
                    Pickable::IGNORE,
                ));
            }
            for rect in [
                OptionUiRect::new(20.0, 181.0, 430.0, 319.0),
                OptionUiRect::new(500.0, 171.0, 435.0, 329.0),
            ] {
                page.spawn((
                    rect.node(),
                    sliced_image(
                        assets.image(OptionTextureRole::PanelSide),
                        OPTION_DARK_BOX_BORDER,
                    ),
                    Pickable::IGNORE,
                ));
            }

            spawn_option_title(
                page,
                OptionUiRect::new(20.0, 28.0, 100.0, 50.0),
                "GRAPHICS",
                assets,
            );
            spawn_option_title(
                page,
                OptionUiRect::new(500.0, 28.0, 100.0, 50.0),
                "SOUND",
                assets,
            );
            spawn_small_button(
                page,
                OptionUiRect::new(275.0, 30.0, 180.0, 25.0),
                "DEFAULT GRAPHICS",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionGraphicsDefaultsButton),
            );
            spawn_small_button(
                page,
                OptionUiRect::new(755.0, 30.0, 180.0, 25.0),
                "DEFAULT SOUND",
                assets,
                Some(OptionPageButtonVisual),
                Some(OptionSoundDefaultsButton),
            );

            spawn_option_bigfont14(
                page,
                OptionUiRect::new(9.0, 65.0, 150.0, 50.0),
                "SCREEN MODE:",
                assets,
            );
            spawn_option_bigfont14(
                page,
                OptionUiRect::new(0.0, 100.0, 150.0, 50.0),
                "DETAIL LEVEL:",
                assets,
            );
            for (rect, label) in [
                (OptionUiRect::new(30.0, 200.0, 150.0, 50.0), "VISIBILITY:"),
                (
                    OptionUiRect::new(30.0, 235.0, 150.0, 50.0),
                    "PARTICLE EFFECT DENSITY:",
                ),
                (OptionUiRect::new(30.0, 440.0, 150.0, 50.0), "SHADOWS:"),
                (
                    OptionUiRect::new(30.0, 470.0, 150.0, 50.0),
                    "TEXTURE QUALITY:",
                ),
            ] {
                spawn_option_smallfont(page, rect, label, assets);
            }
            spawn_option_smallcyan(
                page,
                OptionUiRect::new(40.0, 170.0, 200.0, 50.0),
                "CUSTOM DETAIL LEVEL",
                assets,
            );

            spawn_dropdown_button(
                page,
                OptionUiRect::new(200.0, 82.0, 223.0, 21.0),
                OptionDropdownKind::Resolution,
                assets,
            );
            spawn_dropdown_button(
                page,
                OptionUiRect::new(200.0, 115.0, 223.0, 21.0),
                OptionDropdownKind::Detail,
                assets,
            );
            spawn_segmented_graphics_slider(
                page,
                OptionUiRect::new(230.0, 202.0, 170.0, 16.0),
                true,
                assets,
            );
            spawn_segmented_graphics_slider(
                page,
                OptionUiRect::new(230.0, 242.0, 170.0, 16.0),
                false,
                assets,
            );
            for (rect, label) in [
                (OptionUiRect::new(200.0, 185.0, 40.0, 40.0), "Near"),
                (OptionUiRect::new(405.0, 186.0, 40.0, 40.0), "Far"),
                (OptionUiRect::new(200.0, 225.0, 40.0, 40.0), "Low"),
                (OptionUiRect::new(405.0, 226.0, 40.0, 40.0), "High"),
            ] {
                spawn_option_default_label(page, rect, label, assets);
            }

            for (toggle, (label_y, radio_y)) in OptionGraphicsToggle::ALL.into_iter().zip([
                (280.0, 277.0),
                (310.0, 310.0),
                (340.0, 345.0),
                (380.0, 377.0),
                (410.0, 408.0),
            ]) {
                spawn_option_smallfont(
                    page,
                    OptionUiRect::new(30.0, label_y, 150.0, 50.0),
                    &format!("{}:", toggle.label()),
                    assets,
                );
                spawn_graphics_toggle_pair(page, toggle, radio_y, assets);
            }
            spawn_dropdown_button(
                page,
                OptionUiRect::new(200.0, 437.0, 223.0, 21.0),
                OptionDropdownKind::Shadow,
                assets,
            );
            spawn_dropdown_button(
                page,
                OptionUiRect::new(200.0, 468.0, 223.0, 21.0),
                OptionDropdownKind::Texture,
                assets,
            );

            spawn_option_smallcyan(
                page,
                OptionUiRect::new(520.0, 160.0, 200.0, 50.0),
                "CUSTOM VOLUME",
                assets,
            );
            let channel_y = [90.0, 195.0, 285.0, 365.0, 435.0];
            let label_y = [80.0, 200.0, 275.0, 350.0, 425.0];
            let track_y = [122.0, 225.0, 315.0, 395.0, 465.0];
            let low_high_y = [105.0, 210.0, 300.0, 380.0, 450.0];
            for (index, channel) in OptionSoundChannel::ALL.into_iter().enumerate() {
                spawn_option_bigfont14(
                    page,
                    OptionUiRect::new(
                        if index == 0 { 510.0 } else { 520.0 },
                        label_y[index],
                        150.0,
                        50.0,
                    ),
                    &format!("{}:", channel.label()),
                    assets,
                );
                spawn_sound_toggle_pair(page, channel, channel_y[index], assets);
                spawn_option_default_label(
                    page,
                    OptionUiRect::new(695.0, low_high_y[index], 40.0, 40.0),
                    "Low",
                    assets,
                );
                spawn_segmented_sound_slider(
                    page,
                    OptionUiRect::new(723.0, track_y[index], 170.0, 16.0),
                    channel,
                    assets,
                );
                spawn_option_default_label(
                    page,
                    OptionUiRect::new(900.0, low_high_y[index], 40.0, 40.0),
                    "High",
                    assets,
                );
            }

            spawn_graphics_dropdowns(page, assets);
        });
}

pub(super) fn spawn_graphics_toggle_pair(
    parent: &mut ChildSpawnerCommands,
    toggle: OptionGraphicsToggle,
    y: f32,
    assets: &OptionUiAssets,
) {
    for (x, value, label) in [(200.0, true, "On"), (260.0, false, "Off")] {
        spawn_radio_button(
            parent,
            OptionUiRect::new(x, y, 22.0, 21.0),
            OptionGraphicsToggleButton { toggle, value },
            OptionRadioIndicator::Graphics { toggle, value },
            label,
            assets,
        );
    }
}

pub(super) fn spawn_sound_toggle_pair(
    parent: &mut ChildSpawnerCommands,
    channel: OptionSoundChannel,
    y: f32,
    assets: &OptionUiAssets,
) {
    for (x, value, label) in [(700.0, true, "On"), (760.0, false, "Off")] {
        spawn_radio_button(
            parent,
            OptionUiRect::new(x, y, 22.0, 21.0),
            OptionSoundToggleButton { channel, value },
            OptionRadioIndicator::Sound { channel, value },
            label,
            assets,
        );
    }
}

pub(super) fn spawn_segmented_graphics_slider(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    visibility: bool,
    assets: &OptionUiAssets,
) {
    let maximum = if visibility { 10 } else { 3 };
    let thumb = if visibility {
        OptionGraphicsSliderThumb::Visibility
    } else {
        OptionGraphicsSliderThumb::Particles
    };
    spawn_option_slider_track(parent, rect, thumb, assets);
    let width = rect.width / (f32::from(maximum) + 1.0);
    for step in 0..=maximum {
        parent.spawn((
            Button,
            if visibility {
                OptionGraphicsSliderButton::Visibility(step)
            } else {
                OptionGraphicsSliderButton::Particles(step)
            },
            OptionUiRect::new(
                rect.x + width * f32::from(step),
                rect.y - 5.0,
                width,
                OPTION_SLIDER_THUMB_HEIGHT,
            )
            .node(),
            Pickable::default(),
        ));
    }
}

pub(super) fn spawn_segmented_sound_slider(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    channel: OptionSoundChannel,
    assets: &OptionUiAssets,
) {
    spawn_option_slider_track(
        parent,
        rect,
        OptionGraphicsSliderThumb::Sound(channel),
        assets,
    );
    let width = rect.width / 11.0;
    for step in 0..=10 {
        parent.spawn((
            Button,
            OptionSoundSliderButton { channel, step },
            OptionUiRect::new(
                rect.x + width * f32::from(step),
                rect.y - 5.0,
                width,
                OPTION_SLIDER_THUMB_HEIGHT,
            )
            .node(),
            Pickable::default(),
        ));
    }
}

pub(super) fn spawn_option_slider_track(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    thumb: OptionGraphicsSliderThumb,
    assets: &OptionUiAssets,
) {
    parent.spawn((
        rect.node(),
        stretched_image(assets.image(OptionTextureRole::SliderTrack)),
        Pickable::IGNORE,
    ));
    parent.spawn((
        thumb,
        OptionUiRect::new(
            rect.x,
            rect.y - 5.0,
            OPTION_SLIDER_THUMB_WIDTH,
            OPTION_SLIDER_THUMB_HEIGHT,
        )
        .node(),
        stretched_image(assets.image(OptionTextureRole::SliderThumb)),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_dropdown_button(
    parent: &mut ChildSpawnerCommands,
    rect: OptionUiRect,
    kind: OptionDropdownKind,
    assets: &OptionUiAssets,
) {
    let language_field = matches!(
        kind,
        OptionDropdownKind::Translation | OptionDropdownKind::Voice
    );
    parent
        .spawn((
            Button,
            OptionDropdownButton(kind),
            Node {
                justify_content: if language_field {
                    JustifyContent::Center
                } else {
                    JustifyContent::Start
                },
                align_items: AlignItems::Center,
                padding: if language_field {
                    // Center the value in the text well, excluding the arrow button.
                    UiRect {
                        left: px(4.0),
                        right: px(24.0),
                        ..default()
                    }
                } else {
                    UiRect::left(px(10.0))
                },
                ..rect.node()
            },
            stretched_image(assets.image(OptionTextureRole::Pulldown)),
            ZIndex(OPTION_DROPDOWN_BUTTON_Z_INDEX),
            Pickable::default(),
        ))
        .with_children(|button| {
            button.spawn((
                OptionDropdownValueLabel(kind),
                Text::new(""),
                option_passthrough_text(""),
                (
                    TextFont {
                        font: (assets.comic.clone()).into(),
                        font_size: (10.0).into(),
                        ..default()
                    },
                    LineHeight::Px(OPTION_COMIC_LINE_HEIGHT),
                ),
                TextColor(Color::srgb(0.0, 0.383_064_5, 0.641_129)),
                TextLayout::new(
                    if language_field {
                        Justify::Center
                    } else {
                        Justify::Left
                    },
                    LineBreak::NoWrap,
                ),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_graphics_dropdowns(parent: &mut ChildSpawnerCommands, assets: &OptionUiAssets) {
    spawn_dropdown_panel(
        parent,
        OptionDropdownKind::Resolution,
        OptionUiRect::new(200.0, 82.0, 200.0, 170.0),
        &OPTION_RESOLUTION_CHOICES
            .into_iter()
            .map(|(width, height)| {
                (
                    format!("{width}X{height}"),
                    OptionDropdownChoice::Resolution {
                        width,
                        height,
                        windowed: false,
                    },
                )
            })
            .chain(std::iter::once((
                "Browser".to_owned(),
                OptionDropdownChoice::Resolution {
                    width: 1_024,
                    height: 768,
                    windowed: true,
                },
            )))
            .collect::<Vec<_>>(),
        assets,
        20.0,
    );
    spawn_dropdown_panel(
        parent,
        OptionDropdownKind::Detail,
        OptionUiRect::new(200.0, 115.0, 200.0, 150.0),
        &[
            (
                "BEST QUALITY".into(),
                OptionDropdownChoice::Detail(GraphicsDetail::BestQuality),
            ),
            (
                "GOOD QUALITY".into(),
                OptionDropdownChoice::Detail(GraphicsDetail::GoodQuality),
            ),
            (
                "BALANCED".into(),
                OptionDropdownChoice::Detail(GraphicsDetail::Balanced),
            ),
            (
                "GOOD PERFORMANCE".into(),
                OptionDropdownChoice::Detail(GraphicsDetail::GoodPerformance),
            ),
            (
                "BEST PERFORMANCE".into(),
                OptionDropdownChoice::Detail(GraphicsDetail::BestPerformance),
            ),
        ],
        assets,
        30.0,
    );
    spawn_dropdown_panel(
        parent,
        OptionDropdownKind::Shadow,
        OptionUiRect::new(200.0, 437.0, 200.0, 80.0),
        &[
            (
                "All Characters".into(),
                OptionDropdownChoice::Shadow(ShadowQuality::AllCharacters),
            ),
            (
                "Own Player Only".into(),
                OptionDropdownChoice::Shadow(ShadowQuality::PlayerOnly),
            ),
        ],
        assets,
        30.0,
    );
    spawn_dropdown_panel(
        parent,
        OptionDropdownKind::Texture,
        OptionUiRect::new(200.0, 390.0, 200.0, 100.0),
        &[
            (
                "High".into(),
                OptionDropdownChoice::Texture(TextureQuality::High),
            ),
            (
                "Medium".into(),
                OptionDropdownChoice::Texture(TextureQuality::Medium),
            ),
            (
                "Low".into(),
                OptionDropdownChoice::Texture(TextureQuality::Low),
            ),
        ],
        assets,
        8.0,
    );
}

pub(super) fn spawn_dropdown_panel(
    parent: &mut ChildSpawnerCommands,
    kind: OptionDropdownKind,
    rect: OptionUiRect,
    choices: &[(String, OptionDropdownChoice)],
    assets: &OptionUiAssets,
    item_top: f32,
) {
    parent
        .spawn((
            OptionDropdownPanel(kind),
            Node {
                display: Display::None,
                ..rect.node()
            },
            sliced_image(
                assets.image(OptionTextureRole::DropdownPanel),
                OPTION_DARK_BOX_BORDER,
            ),
            Pickable::default(),
            ZIndex(OPTION_DROPDOWN_PANEL_Z_INDEX),
        ))
        .with_children(|panel| {
            for (index, (label, choice)) in choices.iter().enumerate() {
                panel
                    .spawn((
                        Button,
                        choice.clone(),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0.0),
                            top: px(item_top + index as f32 * 20.0),
                            width: px(rect.width),
                            height: px(20.0),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            padding: UiRect::left(px(10.0)),
                            ..default()
                        },
                        Pickable::default(),
                    ))
                    .with_children(|button| {
                        let mut item_image =
                            stretched_image(assets.image(OptionTextureRole::DropdownItemHover));
                        item_image.color = Color::srgba(1.0, 1.0, 1.0, 0.0);
                        button.spawn((
                            OptionDropdownChoiceBackground,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(-OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT),
                                top: px(0.0),
                                width: px(rect.width
                                    + OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT
                                    + OPTION_DROPDOWN_ITEM_OVERFLOW_RIGHT),
                                height: px(20.0),
                                ..default()
                            },
                            item_image,
                            Pickable::IGNORE,
                        ));
                        let mut entity = button.spawn((
                            OptionDropdownChoiceLabel,
                            Text::new(label.clone()),
                            (
                                TextFont {
                                    font: (assets.comic.clone()).into(),
                                    font_size: (10.0).into(),
                                    ..default()
                                },
                                LineHeight::Px(OPTION_COMIC_LINE_HEIGHT),
                            ),
                            TextColor(Color::srgb(0.584_677_4, 1.0, 0.995_967_75)),
                            TextLayout::new(Justify::Left, LineBreak::NoWrap),
                            Pickable::IGNORE,
                        ));
                        attach_option_localization(&mut entity, label);
                    });
            }
        });
}
