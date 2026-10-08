use super::*;

pub(super) fn spawn_transportation_presentation(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = TransportationPresentationAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            TransportationPresentationRoot,
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
            GlobalZIndex(TRANSPORTATION_UI_Z_INDEX),
        ))
        .with_children(|root| {
            root.spawn((
                TransportationPresentationBackdrop,
                transportation_node(TRANSPORTATION_BACKDROP_RECT),
                transportation_stretch_image(assets.image(WORLD_MAP_BACKDROP_PATH)),
                Pickable::IGNORE,
                ZIndex(0),
            ));
            root.spawn((
                TransportationPresentationWindow,
                transportation_node(TRANSPORTATION_WINDOW_RECT),
                Pickable::IGNORE,
                ZIndex(1),
            ))
            .with_children(|window| {
                window.spawn((
                    transportation_node(TRANSPORTATION_RIGHT_BACK_RECT),
                    transportation_sliced_image(
                        assets.image(TRANSPORTATION_RIGHT_BACK_PATH),
                        BorderRect {
                            min_inset: Vec2::new(5.0, 0.0),
                            max_inset: Vec2::new(10.0, 0.0),
                        },
                    ),
                    Pickable::IGNORE,
                    ZIndex(0),
                ));
                window.spawn((
                    transportation_node(TRANSPORTATION_LEFT_BACK_RECT),
                    transportation_sliced_image(
                        assets.image(TRANSPORTATION_LEFT_BACK_PATH),
                        BorderRect {
                            min_inset: Vec2::new(5.0, 0.0),
                            max_inset: Vec2::new(5.0, 0.0),
                        },
                    ),
                    Pickable::IGNORE,
                    ZIndex(1),
                ));
                window
                    .spawn((
                        Node {
                            overflow: Overflow::clip(),
                            ..transportation_node(TRANSPORTATION_MAP_RECT)
                        },
                        Pickable::IGNORE,
                        ZIndex(1),
                    ))
                    .with_children(|map| {
                        map.spawn((
                            TransportationPresentationMap,
                            transportation_node(TransportationUiRect::new(
                                0.0,
                                0.0,
                                TRANSPORTATION_MAP_RECT.width,
                                TRANSPORTATION_MAP_RECT.height,
                            )),
                            transportation_stretch_image(assets.image(WORLD_MAP_PAYZONE_PATHS[3])),
                            Pickable::IGNORE,
                            ZIndex(0),
                        ));
                        map.spawn((
                            transportation_node(TransportationUiRect::new(
                                0.0,
                                0.0,
                                TRANSPORTATION_MAP_RECT.width,
                                TRANSPORTATION_MAP_RECT.height,
                            )),
                            transportation_stretch_image(assets.image(WORLD_MAP_LINE_PATH)),
                            Pickable::IGNORE,
                            ZIndex(2),
                        ));
                        map.spawn((
                            TransportationPresentationLineEffect(0),
                            transportation_node(TransportationUiRect::new(
                                0.0,
                                200.0,
                                TRANSPORTATION_MAP_RECT.width,
                                64.0,
                            )),
                            transportation_stretch_image(
                                assets.image(WORLD_MAP_LINE_EFFECT_LARGE_PATH),
                            ),
                            Pickable::IGNORE,
                            ZIndex(3),
                        ));
                        map.spawn((
                            TransportationPresentationLineEffect(1),
                            transportation_node(TransportationUiRect::new(
                                0.0,
                                420.0,
                                TRANSPORTATION_MAP_RECT.width,
                                4.0,
                            )),
                            transportation_stretch_image(
                                assets.image(WORLD_MAP_LINE_EFFECT_SMALL_PATH),
                            ),
                            Pickable::IGNORE,
                            ZIndex(3),
                        ));
                        map.spawn((
                            TransportationPresentationMarkerLayer,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0),
                                top: px(0),
                                width: percent(100),
                                height: percent(100),
                                ..default()
                            },
                            Pickable::IGNORE,
                            ZIndex(1),
                        ));
                    });
                window.spawn((
                    transportation_node(TRANSPORTATION_LEFT_BOX_RECT),
                    transportation_sliced_image(
                        assets.image(TRANSPORTATION_LEFT_BOX_PATH),
                        BorderRect {
                            min_inset: Vec2::new(0.0, 85.0),
                            max_inset: Vec2::new(0.0, 55.0),
                        },
                    ),
                    Pickable::IGNORE,
                    ZIndex(2),
                ));
                window.spawn((
                    TransportationPresentationNpcCameraSlot,
                    transportation_node(TRANSPORTATION_CAMERA_RECT),
                    ImageNode::default(),
                    Pickable::IGNORE,
                    ZIndex(3),
                ));
                window.spawn((
                    TransportationPresentationMonkey,
                    transportation_node(TransportationUiRect::new(20.0, 124.0 - 86.0, 99.0, 86.0)),
                    transportation_stretch_image(assets.image(TRANSPORTATION_MONKEY_PATH)),
                    Pickable::IGNORE,
                    ZIndex(3),
                ));
                window.spawn((
                    transportation_node(TRANSPORTATION_BUBBLE_RECT),
                    transportation_stretch_image(assets.image(TRANSPORTATION_BUBBLE_PATH)),
                    Pickable::IGNORE,
                    ZIndex(3),
                ));
                let title_style = TransportationUiTextStyle::BigFont16UpperLeft;
                window.spawn((
                    TransportationPresentationTitle,
                    transportation_text_node(TRANSPORTATION_TITLE_RECT, title_style),
                    Text::new("TRANSPORTATION"),
                    LocalizedText::new("ui.transportation.title", "TRANSPORTATION"),
                    title_style.font(&assets.jeffe),
                    TextColor(Color::WHITE),
                    title_style.layout(),
                    title_style,
                    UiTransform::from_translation(Val2::px(
                        0.0,
                        title_style.replacement_y_offset(),
                    )),
                    Pickable::IGNORE,
                    ZIndex(4),
                ));
                let subtitle_style = TransportationUiTextStyle::BigFont14UpperLeft;
                window.spawn((
                    TransportationPresentationSubtitle,
                    transportation_text_node(TRANSPORTATION_SUBTITLE_RECT, subtitle_style),
                    Text::new(""),
                    transportation_empty_subtitle_text(),
                    subtitle_style.font(&assets.jeffe),
                    TextColor(Color::WHITE),
                    subtitle_style.layout(),
                    subtitle_style,
                    UiTransform::from_translation(Val2::px(
                        0.0,
                        subtitle_style.replacement_y_offset(),
                    )),
                    Pickable::IGNORE,
                    ZIndex(4),
                ));
                let where_to_style = TransportationUiTextStyle::BigFont16UpperLeft;
                window.spawn((
                    TransportationPresentationWhereTo,
                    transportation_text_node(TRANSPORTATION_WHERE_TO_RECT, where_to_style),
                    Text::new("WHERE TO?"),
                    LocalizedText::new("ui.transportation.where_to", "WHERE TO?"),
                    where_to_style.font(&assets.jeffe),
                    TextColor(Color::BLACK),
                    where_to_style.layout(),
                    where_to_style,
                    UiTransform::from_translation(Val2::px(
                        0.0,
                        where_to_style.replacement_y_offset(),
                    )),
                    Pickable::IGNORE,
                    ZIndex(4),
                ));
                window
                    .spawn((
                        Node {
                            overflow: Overflow::clip(),
                            ..transportation_node(TRANSPORTATION_SELECT_RECT)
                        },
                        Pickable::IGNORE,
                        ZIndex(5),
                    ))
                    .with_children(|viewport| {
                        viewport.spawn((
                            TransportationPresentationRouteLayer,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0),
                                top: px(0),
                                width: px(TRANSPORTATION_SCROLL_CONTENT_WIDTH),
                                height: px(TRANSPORTATION_SELECT_RECT.height),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        viewport.spawn((
                            TransportationPresentationScrollbar,
                            transportation_node(TRANSPORTATION_SCROLLBAR_RECT),
                            transportation_sliced_image(
                                assets.image(TRANSPORTATION_SCROLL_BAR_PATH),
                                BorderRect {
                                    min_inset: Vec2::new(2.0, 4.0),
                                    max_inset: Vec2::new(2.0, 4.0),
                                },
                            ),
                            Pickable::IGNORE,
                            ZIndex(2),
                        ));
                        viewport.spawn((
                            TransportationPresentationScrollbar,
                            transportation_node(TRANSPORTATION_SCROLL_UP_RECT),
                            transportation_stretch_image(
                                assets.image(TRANSPORTATION_SCROLL_UP_PATH),
                            ),
                            Pickable::IGNORE,
                            ZIndex(3),
                        ));
                        viewport.spawn((
                            TransportationPresentationScrollbar,
                            transportation_node(TRANSPORTATION_SCROLL_DOWN_RECT),
                            transportation_stretch_image(
                                assets.image(TRANSPORTATION_SCROLL_DOWN_PATH),
                            ),
                            Pickable::IGNORE,
                            ZIndex(3),
                        ));
                        viewport.spawn((
                            TransportationPresentationScrollThumb,
                            TransportationPresentationScrollbar,
                            transportation_node(TRANSPORTATION_SCROLL_THUMB_RECT),
                            transportation_sliced_image(
                                assets.image(TRANSPORTATION_SCROLL_THUMB_PATH),
                                BorderRect {
                                    min_inset: Vec2::new(2.0, 4.0),
                                    max_inset: Vec2::new(2.0, 4.0),
                                },
                            ),
                            Pickable::IGNORE,
                            ZIndex(4),
                        ));
                    });
                spawn_transportation_control(
                    window,
                    TransportationPresentationControl::Close,
                    TRANSPORTATION_CLOSE_RECT,
                    WORLD_MAP_CLOSE_PATH,
                    &assets,
                );
                spawn_transportation_text_control(
                    window,
                    TransportationPresentationControl::GoNow,
                    TRANSPORTATION_GO_RECT,
                    "GO NOW!",
                    &assets,
                );
                window
                    .spawn((
                        TransportationPresentationTurboLayer,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        Pickable::IGNORE,
                        ZIndex(6),
                    ))
                    .with_children(|turbo| {
                        turbo.spawn((
                            transportation_node(TRANSPORTATION_TURBO_BACKGROUND_RECT),
                            transportation_sliced_image(
                                assets.image(TRANSPORTATION_BLUE_BUTTON_PATH),
                                BorderRect::all(5.0),
                            ),
                            Pickable::IGNORE,
                            ZIndex(1),
                        ));
                        turbo.spawn((
                            Button,
                            TransportationPresentationControlNode(
                                TransportationPresentationControl::Turbo,
                            ),
                            transportation_node(TRANSPORTATION_TURBO_INTERACT_RECT),
                            BackgroundColor(Color::NONE),
                            ZIndex(4),
                        ));
                        turbo.spawn((
                            TransportationPresentationTurboCheck,
                            transportation_node(TRANSPORTATION_TURBO_TOGGLE_RECT),
                            transportation_stretch_image(
                                assets.image(TRANSPORTATION_BLUE_BUTTON_HOVER_PATH),
                            ),
                            Pickable::IGNORE,
                            ZIndex(2),
                        ));
                        let turbo_style = TransportationUiTextStyle::BigFont14UpperLeft;
                        turbo.spawn((
                            TransportationPresentationTurboLabel,
                            transportation_text_node(TRANSPORTATION_TURBO_LABEL_RECT, turbo_style),
                            Text::new("TURBO TRAVEL"),
                            LocalizedText::new("ui.transportation.turbo_travel", "TURBO TRAVEL"),
                            turbo_style.font(&assets.jeffe),
                            TextColor(Color::WHITE),
                            turbo_style.layout(),
                            turbo_style,
                            UiTransform::from_translation(Val2::px(
                                0.0,
                                turbo_style.replacement_y_offset(),
                            )),
                            Pickable::IGNORE,
                            ZIndex(3),
                        ));
                    });
            });
        });
}

pub(super) fn spawn_transportation_control(
    parent: &mut ChildSpawnerCommands,
    control: TransportationPresentationControl,
    rect: TransportationUiRect,
    path: &'static str,
    assets: &TransportationPresentationAssets,
) {
    let mut button = parent.spawn((
        Button,
        TransportationPresentationControlNode(control),
        transportation_node(rect),
        transportation_stretch_image(assets.image(path)),
        ZIndex(7),
    ));
    if control == TransportationPresentationControl::Close {
        button.insert(crate::ui::shared::controller::ControllerUiClose);
    }
}

pub(super) fn spawn_transportation_text_control(
    parent: &mut ChildSpawnerCommands,
    control: TransportationPresentationControl,
    rect: TransportationUiRect,
    label: &'static str,
    assets: &TransportationPresentationAssets,
) {
    let style = TransportationUiTextStyle::ButtonMiddleCenter;
    let mut text_node = Node {
        width: percent(100),
        height: percent(100),
        ..default()
    };
    style.apply_to_container(&mut text_node);
    parent
        .spawn((
            Button,
            TransportationPresentationControlNode(control),
            transportation_node(rect),
            transportation_sliced_image(
                assets.image(TRANSPORTATION_BLUE_BUTTON_PATH),
                BorderRect {
                    min_inset: Vec2::new(5.0, 5.0),
                    max_inset: Vec2::new(5.0, 5.0),
                },
            ),
            ZIndex(7),
        ))
        .with_child((
            TransportationPresentationGoLabel,
            text_node,
            Text::new(label),
            LocalizedText::new("ui.transportation.go_now", label),
            style.font(&assets.jeffe),
            TextColor(Color::WHITE),
            style.layout(),
            style,
            UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
            Pickable::IGNORE,
        ));
}

pub(super) fn spawn_transportation_route(
    commands: &mut Commands,
    layer: Entity,
    route_index: usize,
    route: &TransportationRoute,
    model: &TransportationModel,
    copy: &TransportationUiCopy,
    assets: &TransportationPresentationAssets,
) {
    let enabled = route.registered;
    let selected = model.selected_route() == Some(route_index);
    let alpha = if enabled { 1.0 } else { 0.5 };
    let row_y = route_index as f32 * TRANSPORTATION_ROUTE_STRIDE + 5.0;
    let mut row_image = transportation_stretch_image(assets.image(TRANSPORTATION_ROUTE_ROW_PATH));
    row_image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    let mut row = commands.spawn((
        TransportationPresentationDynamicRoute,
        TransportationPresentationRouteRow(route_index),
        TransportationPresentationControlNode(TransportationPresentationControl::Route(
            route_index,
        )),
        transportation_node(TransportationUiRect::new(10.0, row_y, 295.0, 75.0)),
        row_image,
        ZIndex(0),
    ));
    if enabled {
        row.insert((Button, crate::ui::shared::controller::ControllerUiDefault));
    } else {
        row.insert(Pickable::IGNORE);
    }
    let row_entity = row.id();
    commands.entity(layer).add_child(row_entity);
    commands.entity(row_entity).with_children(|row| {
        if selected {
            row.spawn((
                transportation_node(TransportationUiRect::new(-1.0, -1.0, 297.0, 77.0)),
                transportation_sliced_image(
                    assets.image(TRANSPORTATION_ROUTE_SELECTED_PATH),
                    BorderRect::all(4.0),
                ),
                Pickable::IGNORE,
                ZIndex(1),
            ));
        }
        let name_color = if !enabled {
            Color::srgba(1.0, 1.0, 1.0, 0.5)
        } else if selected {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            Color::WHITE
        };
        let bigfont14_style = TransportationUiTextStyle::BigFont14UpperLeft;
        row.spawn((
            transportation_text_node(
                TransportationUiRect::new(83.0, 14.0, 200.0, 20.0),
                bigfont14_style,
            ),
            Text::new(""),
            transportation_route_name_text(route),
            bigfont14_style.font(&assets.jeffe),
            TextColor(name_color),
            bigfont14_style.layout(),
            bigfont14_style,
            UiTransform::from_translation(Val2::px(0.0, bigfont14_style.replacement_y_offset())),
            Pickable::IGNORE,
            ZIndex(2),
        ));
        row.spawn((
            transportation_text_node(
                TransportationUiRect::new(83.0, 34.0, 200.0, 20.0),
                bigfont14_style,
            ),
            Text::new(""),
            transportation_route_region_text(&route.region),
            bigfont14_style.font(&assets.jeffe),
            TextColor(name_color),
            bigfont14_style.layout(),
            bigfont14_style,
            UiTransform::from_translation(Val2::px(0.0, bigfont14_style.replacement_y_offset())),
            Pickable::IGNORE,
            ZIndex(2),
        ));

        let mut icon_box = transportation_sliced_image(
            assets.image(TRANSPORTATION_ICON_BOX_PATH),
            BorderRect::all(5.0),
        );
        icon_box.color = Color::srgba(1.0, 1.0, 1.0, alpha);
        row.spawn((
            transportation_node(TransportationUiRect::new(8.0, 5.0, 64.0, 64.0)),
            icon_box,
            Pickable::IGNORE,
            ZIndex(2),
        ))
        .with_child((
            Node {
                position_type: PositionType::Absolute,
                left: px(TRANSPORTATION_ICON_CONTENT_PADDING),
                top: px(TRANSPORTATION_ICON_CONTENT_PADDING),
                width: px(64.0 - TRANSPORTATION_ICON_CONTENT_PADDING * 2.0),
                height: px(64.0 - TRANSPORTATION_ICON_CONTENT_PADDING * 2.0),
                ..default()
            },
            {
                let mut image = transportation_stretch_image(assets.image(route.icon_path));
                image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
                image
            },
            Pickable::IGNORE,
        ));

        if !enabled {
            let right_text_style = TransportationUiTextStyle::RightTextUpperRight;
            row.spawn((
                transportation_text_node(
                    TransportationUiRect::new(133.0, 50.0, 130.0, 25.0),
                    right_text_style,
                ),
                Text::new(copy.unregistered.clone()),
                LocalizedText::new("ui.transportation.unregistered", copy.unregistered.clone()),
                right_text_style.font(&assets.jeffe),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                right_text_style.layout(),
                right_text_style,
                UiTransform::from_translation(Val2::px(
                    0.0,
                    right_text_style.replacement_y_offset(),
                )),
                Pickable::IGNORE,
                ZIndex(2),
            ));
        } else if model.service() != TransportationService::ItemUse {
            let affordable = model.route_affordable(route_index).unwrap_or(false);
            let cost_color = if !affordable {
                Color::srgb(1.0, 0.0, 0.0)
            } else if model.turbo() {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                Color::WHITE
            };
            let cost = model.route_effective_cost(route_index).unwrap_or_default();
            let right_text_style = TransportationUiTextStyle::RightTextUpperRight;
            row.spawn((
                transportation_text_node(
                    TransportationUiRect::new(133.0, 50.0, 130.0, 25.0),
                    right_text_style,
                ),
                Text::new(""),
                transportation_cost_text(cost),
                right_text_style.font(&assets.jeffe),
                TextColor(cost_color),
                right_text_style.layout(),
                right_text_style,
                UiTransform::from_translation(Val2::px(
                    0.0,
                    right_text_style.replacement_y_offset(),
                )),
                Pickable::IGNORE,
                ZIndex(2),
            ));
            row.spawn((
                transportation_node(TransportationUiRect::new(263.0, 45.0, 31.0, 31.0)),
                transportation_stretch_image(assets.image(TRANSPORTATION_TAROS_PATH)),
                Pickable::IGNORE,
                ZIndex(2),
            ));
        }
    });
}
