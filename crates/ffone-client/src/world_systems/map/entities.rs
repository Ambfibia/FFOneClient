use super::*;

pub(super) fn spawn_world_map_presentation(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = WorldMapPresentationAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            WorldMapPresentationRoot,
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
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
            GlobalZIndex(WORLD_MAP_UI_Z_INDEX),
        ))
        .with_children(|root| {
            root.spawn((
                WorldMapPresentationBackdrop,
                presentation_node(WORLD_MAP_BACKDROP_RECT),
                UiTransform::default(),
                stretch_image(assets.image(WORLD_MAP_BACKDROP_PATH)),
                Pickable::IGNORE,
                ZIndex(0),
            ));
            root.spawn((
                WorldMapPresentationWindow,
                presentation_node(WORLD_MAP_WINDOW_RECT),
                UiTransform::default(),
                sliced_image(
                    assets.image(WORLD_MAP_MAP_BACK_PATH),
                    BorderRect {
                        min_inset: Vec2::new(10.0, 0.0),
                        max_inset: Vec2::new(10.0, 0.0),
                    },
                ),
                Pickable::IGNORE,
                ZIndex(1),
            ))
            .with_children(|window| {
                window.spawn((
                    WorldMapPresentationNoMap,
                    presentation_node(WORLD_MAP_FRAME_RECT),
                    stretch_image(assets.image(WORLD_MAP_NO_MAP_PATH)),
                    Pickable::IGNORE,
                    ZIndex(2),
                ));
                window
                    .spawn((
                        WorldMapPresentationNormalLayer,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|normal| {
                        normal.spawn((
                            WorldMapPresentationBlack,
                            presentation_node(WORLD_MAP_PICTURE_RECT),
                            stretch_image(assets.image(WORLD_MAP_BLACK_PATH)),
                            Pickable::IGNORE,
                            ZIndex(0),
                        ));
                        normal.spawn((
                            WorldMapPresentationMap,
                            presentation_node(WORLD_MAP_PICTURE_RECT),
                            stretch_image(assets.image(WORLD_MAP_PAYZONE_PATHS[0])),
                            Pickable::IGNORE,
                            ZIndex(1),
                        ));
                        normal.spawn((
                            WorldMapPresentationMarkerLayer,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0),
                                top: px(0),
                                width: percent(100),
                                height: percent(100),
                                ..default()
                            },
                            Pickable::IGNORE,
                            ZIndex(2),
                        ));
                        normal.spawn((
                            WorldMapPresentationLine,
                            presentation_node(WORLD_MAP_PICTURE_RECT),
                            stretch_image(assets.image(WORLD_MAP_LINE_PATH)),
                            Pickable::IGNORE,
                            ZIndex(3),
                        ));
                        normal.spawn((
                            WorldMapPresentationLineEffect(0),
                            presentation_node(WorldMapUiRect::new(
                                WORLD_MAP_PICTURE_RECT.x,
                                WORLD_MAP_PICTURE_RECT.y + 200.0,
                                WORLD_MAP_PICTURE_RECT.width,
                                64.0,
                            )),
                            stretch_image(assets.image(WORLD_MAP_LINE_EFFECT_LARGE_PATH)),
                            Pickable::IGNORE,
                            ZIndex(4),
                        ));
                        normal.spawn((
                            WorldMapPresentationLineEffect(1),
                            presentation_node(WorldMapUiRect::new(
                                WORLD_MAP_PICTURE_RECT.x,
                                WORLD_MAP_PICTURE_RECT.y + 420.0,
                                WORLD_MAP_PICTURE_RECT.width,
                                4.0,
                            )),
                            stretch_image(assets.image(WORLD_MAP_LINE_EFFECT_SMALL_PATH)),
                            Pickable::IGNORE,
                            ZIndex(4),
                        ));
                        normal.spawn((
                            WorldMapPresentationFrame,
                            presentation_node(WORLD_MAP_FRAME_RECT),
                            stretch_image(assets.image(WORLD_MAP_FRAME_PATH)),
                            Pickable::IGNORE,
                            ZIndex(5),
                        ));

                        for (control, rect, path) in [
                            (
                                WorldMapPresentationControl::Close,
                                WORLD_MAP_CLOSE_RECT,
                                WORLD_MAP_CLOSE_PATH,
                            ),
                            (
                                WorldMapPresentationControl::Help,
                                WORLD_MAP_HELP_RECT,
                                WORLD_MAP_HELP_PATH,
                            ),
                            (
                                WorldMapPresentationControl::Up,
                                WORLD_MAP_UP_RECT,
                                WORLD_MAP_UP_PATH,
                            ),
                            (
                                WorldMapPresentationControl::Down,
                                WORLD_MAP_DOWN_RECT,
                                WORLD_MAP_DOWN_PATH,
                            ),
                            (
                                WorldMapPresentationControl::Left,
                                WORLD_MAP_LEFT_RECT,
                                WORLD_MAP_LEFT_PATH,
                            ),
                            (
                                WorldMapPresentationControl::Right,
                                WORLD_MAP_RIGHT_RECT,
                                WORLD_MAP_RIGHT_PATH,
                            ),
                            (
                                WorldMapPresentationControl::ZoomIn,
                                WORLD_MAP_ZOOM_IN_RECT,
                                WORLD_MAP_ZOOM_IN_PATH,
                            ),
                            (
                                WorldMapPresentationControl::ZoomOut,
                                WORLD_MAP_ZOOM_OUT_RECT,
                                WORLD_MAP_ZOOM_OUT_PATH,
                            ),
                        ] {
                            spawn_world_map_image_control(normal, control, rect, path, &assets);
                        }
                        for (index, rect) in WORLD_MAP_ZOOM_TICK_RECTS.into_iter().enumerate() {
                            spawn_world_map_image_control(
                                normal,
                                WorldMapPresentationControl::ZoomTick(index as u8),
                                rect,
                                WORLD_MAP_ZOOM_TICK_PATH,
                                &assets,
                            );
                        }
                        normal.spawn((
                            WorldMapPresentationZoomBar,
                            presentation_node(WORLD_MAP_ZOOM_BAR_RECT),
                            sliced_image(
                                assets.image(WORLD_MAP_ZOOM_BAR_PATH),
                                BorderRect {
                                    min_inset: Vec2::new(3.0, 4.0),
                                    max_inset: Vec2::new(3.0, 4.0),
                                },
                            ),
                            Pickable::IGNORE,
                            ZIndex(7),
                        ));

                        spawn_world_map_text_control(
                            normal,
                            WorldMapPresentationControl::LocalView,
                            WORLD_MAP_LOCAL_VIEW_RECT,
                            &assets,
                        );
                        spawn_world_map_text_control(
                            normal,
                            WorldMapPresentationControl::WorldView,
                            WORLD_MAP_WORLD_VIEW_RECT,
                            &assets,
                        );
                        spawn_world_map_text_control(
                            normal,
                            WorldMapPresentationControl::ShowFilters,
                            WORLD_MAP_MISSION_FINDER_RECT,
                            &assets,
                        );

                        for (index, (icon, _)) in WORLD_MAP_FILTERS.iter().enumerate() {
                            normal
                                .spawn((
                                    WorldMapPresentationControlNode(
                                        WorldMapPresentationControl::Filter(index as u8),
                                    ),
                                    presentation_node(world_map_filter_rect(index)),
                                    stretch_image(
                                        assets.image(
                                            "ui/en/character/creation/layout/CCLeftIn3BG.png",
                                        ),
                                    ),
                                    ZIndex(7),
                                    Pickable::IGNORE,
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        presentation_node(WorldMapUiRect::new(
                                            10.0, -7.0, 8.0, 11.0,
                                        )),
                                        stretch_image(
                                            assets
                                                .image("ui/en/world-map/filters/filter-arrow.png"),
                                        ),
                                        UiTransform::from_rotation(Rot2::degrees(90.0)),
                                        Pickable::IGNORE,
                                    ));
                                    for part in 0..3 {
                                        let path = if part == 0 {
                                            "ui/en/world-map/markers/neon-shine-background.png"
                                        } else {
                                            WORLD_MAP_MARKER_PATHS[usize::from(*icon)]
                                        };
                                        button.spawn((
                                            WorldMapFilterVisual { index, part },
                                            presentation_node(WorldMapUiRect::new(
                                                0.0, 0.0, 30.0, 30.0,
                                            )),
                                            stretch_image(assets.image(path)),
                                            Pickable::IGNORE,
                                        ));
                                    }
                                });
                        }
                        normal.spawn((
                            WorldMapPresentationCurrentLabel,
                            WorldMapTextStyle::CurrentLocationLabel,
                            presentation_text_node(
                                WORLD_MAP_CURRENT_LOCATION_LABEL_RECT,
                                WORLD_MAP_LOCATION_LABEL_PADDING,
                                JustifyContent::Center,
                                AlignItems::Center,
                            ),
                            Text::new("current location:"),
                            LocalizedText::new(
                                "ui.world_map.current_location",
                                "current location:",
                            ),
                            // Clean JEFFE___06 contains only uppercase display
                            // glyphs even though WorldMapMode requests the
                            // lowercase localization key.
                            LocalizedTextCase::Uppercase,
                            UiTransform::from_translation(Val2::px(
                                WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_X,
                                WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_Y,
                            )),
                            (
                                TextFont {
                                    font: (assets.jeffe.clone()).into(),
                                    font_size: (WORLD_MAP_LOCATION_LABEL_FONT_SIZE).into(),
                                    ..default()
                                },
                                LineHeight::Px(WORLD_MAP_LOCATION_LABEL_FONT_LINE_HEIGHT),
                            ),
                            TextColor(Color::WHITE),
                            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                            Pickable::IGNORE,
                            ZIndex(7),
                        ));
                        normal
                            .spawn((
                                presentation_node(WORLD_MAP_CURRENT_LOCATION_VALUE_RECT),
                                sliced_image(
                                    assets.image(WORLD_MAP_LOCATION_BACK_PATH),
                                    BorderRect::all(4.0),
                                ),
                                Pickable::IGNORE,
                                ZIndex(7),
                            ))
                            .with_child((
                                WorldMapPresentationCurrentValue,
                                WorldMapTextStyle::CurrentLocationValue,
                                Node {
                                    width: percent(100),
                                    height: percent(100),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::FlexStart,
                                    padding: UiRect::all(px(WORLD_MAP_LOCATION_VALUE_PADDING)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                Text::new(""),
                                world_map_passthrough_text(""),
                                UiTransform::from_translation(Val2::px(
                                    WORLD_MAP_LOCATION_VALUE_TEXT_OFFSET_X,
                                    WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
                                )),
                                (
                                    TextFont {
                                        font: (assets.chalet.clone()).into(),
                                        font_size: (WORLD_MAP_SMALL_CHALET_FONT_SIZE).into(),
                                        ..default()
                                    },
                                    LineHeight::Px(WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT),
                                ),
                                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                                Pickable::IGNORE,
                            ));
                        normal
                            .spawn((
                                WorldMapPresentationTooltip,
                                presentation_node(WorldMapUiRect::new(0.0, 0.0, 200.0, 20.0)),
                                sliced_image(
                                    assets.image(WORLD_MAP_TOOLTIP_BACK_PATH),
                                    // FusionFallMapSkin `window`: path ID 391,
                                    // m_Border { left: 20, right: 3, top: 0,
                                    // bottom: 0 }.
                                    BorderRect {
                                        min_inset: Vec2::new(20.0, 0.0),
                                        max_inset: Vec2::new(3.0, 0.0),
                                    },
                                ),
                                Pickable::IGNORE,
                                ZIndex(9),
                            ))
                            .with_child((
                                WorldMapPresentationTooltipText,
                                WorldMapTextStyle::Tooltip,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    top: px(0),
                                    width: px(200),
                                    height: px(20),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::FlexStart,
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                Text::new(""),
                                world_map_passthrough_text(""),
                                UiTransform::from_translation(Val2::px(
                                    WORLD_MAP_TOOLTIP_TEXT_OFFSET_X,
                                    WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
                                )),
                                (
                                    TextFont {
                                        font: (assets.chalet.clone()).into(),
                                        font_size: (WORLD_MAP_SMALL_CHALET_FONT_SIZE).into(),
                                        ..default()
                                    },
                                    LineHeight::Px(WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT),
                                ),
                                TextColor(Color::BLACK),
                                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                                Pickable::IGNORE,
                            ));
                    });
            });
        });
}

pub(super) fn spawn_world_map_image_control(
    parent: &mut ChildSpawnerCommands,
    control: WorldMapPresentationControl,
    rect: WorldMapUiRect,
    path: &'static str,
    assets: &WorldMapPresentationAssets,
) {
    parent.spawn((
        WorldMapPresentationControlNode(control),
        presentation_node(rect),
        stretch_image(assets.image(path)),
        Pickable::IGNORE,
        ZIndex(7),
    ));
}

pub(super) fn spawn_world_map_text_control(
    parent: &mut ChildSpawnerCommands,
    control: WorldMapPresentationControl,
    rect: WorldMapUiRect,
    assets: &WorldMapPresentationAssets,
) {
    let localized = world_map_control_text(control);
    let label = localized.fallback.clone();
    parent
        .spawn((
            WorldMapPresentationControlNode(control),
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..presentation_node(rect)
            },
            sliced_image(
                assets.image(WORLD_MAP_VIEW_NORMAL_PATH),
                BorderRect {
                    min_inset: Vec2::new(4.0, 0.0),
                    max_inset: Vec2::new(4.0, 0.0),
                },
            ),
            Pickable::IGNORE,
            ZIndex(7),
        ))
        .with_child((
            WorldMapPresentationControlLabel(control),
            WorldMapTextStyle::ViewButton,
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(px(WORLD_MAP_VIEW_TEXT_PADDING)),
                overflow: Overflow::clip(),
                ..default()
            },
            Text::new(label),
            localized,
            UiTransform::from_translation(Val2::px(
                WORLD_MAP_VIEW_TEXT_OFFSET_X,
                WORLD_MAP_VIEW_TEXT_OFFSET_Y,
            )),
            (
                TextFont {
                    font: (assets.chalet.clone()).into(),
                    font_size: (WORLD_MAP_VIEW_FONT_SIZE).into(),
                    ..default()
                },
                LineHeight::Px(WORLD_MAP_VIEW_FONT_LINE_HEIGHT),
            ),
            TextColor(Color::srgb(0.0, 0.580_392_2, 0.580_392_2)),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            Pickable::IGNORE,
        ));
}
