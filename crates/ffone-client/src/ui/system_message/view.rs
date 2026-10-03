use super::*;

#[derive(Component)]
pub(super) struct SystemMessageBodyViewport;

pub(super) fn spawn_system_message_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = SystemMessageUiAssets::load(&asset_server);
    commands.insert_resource(assets);
    commands.spawn((
        SystemMessageUiRoot,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, SYSTEM_MESSAGE_OVERLAY_ALPHA)),
        GlobalZIndex(SYSTEM_MESSAGE_UI_Z_INDEX),
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
    ));
}

pub(super) fn spawn_system_message_layer(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    message: &SystemMessageRequest,
    resolved_text: &str,
    current: bool,
    input_enabled: bool,
    assets: &SystemMessageUiAssets,
    asset_server: &AssetServer,
) {
    parent
        .spawn((
            SystemMessageLayer(index),
            Node {
                position_type: PositionType::Absolute,
                width: px(SYSTEM_MESSAGE_WINDOW_RECT.width),
                height: px(SYSTEM_MESSAGE_WINDOW_RECT.height),
                ..default()
            },
            UiTransform::default(),
            ImageNode {
                image: assets.dialog.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: SYSTEM_MESSAGE_DIALOG_BORDER,
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|dialog| {
            if !current {
                return;
            }
            if let Some(icon_path) = message.icon_path.as_deref() {
                if message.button_type != SystemMessageButtonType::CombinationConfirm {
                    dialog.spawn((
                        SystemMessageIconFrame,
                        SYSTEM_MESSAGE_ICON_RECT.node(),
                        ImageNode {
                            image: assets.item_box.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                }
                dialog.spawn((
                    SystemMessagePrimaryIcon,
                    SYSTEM_MESSAGE_ICON_RECT.node(),
                    ImageNode {
                        image: asset_server.load(icon_path.to_owned()),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                if message.button_type == SystemMessageButtonType::DeleteItem
                    && message.icon_quantity > 0
                {
                    dialog.spawn((
                        SystemMessageIconQuantity,
                        Node {
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::FlexStart,
                            padding: UiRect {
                                top: px(3),
                                bottom: px(3),
                                ..default()
                            },
                            ..SYSTEM_MESSAGE_ICON_QUANTITY_RECT.node()
                        },
                        Text::new(""),
                        LocalizedText::new("ui.content.passthrough", "{text}")
                            .with_arg("text", message.icon_quantity.to_string()),
                        (
                            TextFont {
                                font: (assets.jeffe_font.clone()).into(),
                                font_size: (SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE).into(),
                                ..default()
                            },
                            LineHeight::Px(SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT),
                        ),
                        TextColor(system_message_text_color()),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                        SystemMessageTextStyle::Label,
                        Pickable::IGNORE,
                    ));
                }
            }
            if let [Some(left_path), Some(right_path)] = &message.comparison_icon_paths {
                for (comparison_index, path) in [left_path, right_path].into_iter().enumerate() {
                    let rect = SYSTEM_MESSAGE_COMPARISON_ICON_RECTS[comparison_index];
                    dialog.spawn((
                        SystemMessageIconFrame,
                        rect.node(),
                        ImageNode {
                            image: assets.item_box.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    dialog.spawn((
                        SystemMessageComparisonIcon(comparison_index),
                        rect.node(),
                        ImageNode {
                            image: asset_server.load(path.to_owned()),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    if message.comparison_icon_combined[comparison_index] {
                        dialog.spawn((
                            SystemMessageCombinedBadge(comparison_index),
                            SYSTEM_MESSAGE_COMPARISON_BADGE_RECTS[comparison_index].node(),
                            ImageNode {
                                image: assets.combined_badge.clone(),
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    }
                }
            }
            spawn_system_message_body(dialog, message.button_type, resolved_text, assets);

            let layout = system_message_button_layout(message.button_type);
            spawn_system_message_button(dialog, layout.primary, input_enabled, assets);
            if let Some(secondary) = layout.secondary {
                spawn_system_message_button(dialog, secondary, input_enabled, assets);
            }
        });
}

pub(super) fn spawn_system_message_body(
    parent: &mut ChildSpawnerCommands,
    button_type: SystemMessageButtonType,
    resolved_text: &str,
    assets: &SystemMessageUiAssets,
) {
    let lines = resolved_text.split('\n').collect::<Vec<_>>();
    let content_rect = match button_type {
        SystemMessageButtonType::DeleteMission => SYSTEM_MESSAGE_DELETE_MISSION_CONTENT_RECT,
        SystemMessageButtonType::CombinationFailure => {
            SYSTEM_MESSAGE_COMBINATION_FAILURE_CONTENT_RECT
        }
        _ => SYSTEM_MESSAGE_CONTENT_RECT,
    };
    parent
        .spawn((
            SystemMessageBodyViewport,
            Interaction::None,
            ScrollPosition::default(),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                overflow: Overflow::scroll_y(),
                ..content_rect.node()
            },
        ))
        .with_children(|area| {
            if lines.len() < 2 {
                spawn_system_message_body_text(
                    area,
                    0,
                    lines.first().copied().unwrap_or_default(),
                    SystemMessageTextStyle::Label,
                    layout_text_node(SystemMessageTextStyle::Label, None, None, false),
                    SYSTEM_MESSAGE_LABEL_COLOR,
                    assets,
                );
                return;
            }
            match button_type {
                SystemMessageButtonType::DeleteMission => {
                    spawn_delete_mission_body(area, &lines, assets)
                }
                SystemMessageButtonType::CombinationFailure => {
                    spawn_multiline_message_body(area, &lines, true, assets)
                }
                _ => spawn_multiline_message_body(area, &lines, false, assets),
            }
        });
}

pub(super) fn scroll_system_message_body(
    model: Res<SystemMessageUiModel>,
    wheel: Option<Res<bevy::input::mouse::AccumulatedMouseScroll>>,
    mut viewports: Query<
        (&ComputedNode, &Interaction, &mut ScrollPosition),
        With<SystemMessageBodyViewport>,
    >,
) {
    if model.is_empty() {
        return;
    }
    for (computed, hover, mut position) in &mut viewports {
        let maximum = ((computed.content_size().y - computed.size().y)
            * computed.inverse_scale_factor())
        .max(0.0);
        if *hover == Interaction::None {
            continue;
        }
        let delta = wheel.as_ref().map_or(0.0, |wheel| {
            wheel.delta.y
                * match wheel.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => 30.0,
                    bevy::input::mouse::MouseScrollUnit::Pixel => 1.0,
                }
        });
        position.y = (position.y - delta).clamp(0.0, maximum);
    }
}

pub(super) fn spawn_multiline_message_body(
    parent: &mut ChildSpawnerCommands,
    lines: &[&str],
    combination_failure: bool,
    assets: &SystemMessageUiAssets,
) {
    spawn_system_message_body_text(
        parent,
        0,
        lines.first().copied().unwrap_or_default(),
        SystemMessageTextStyle::Label,
        layout_text_node(SystemMessageTextStyle::Label, None, None, true),
        SYSTEM_MESSAGE_HEADER_COLOR,
        assets,
    );
    spawn_system_message_gap(parent, 10.0);
    for (index, line) in lines.iter().enumerate().skip(1) {
        spawn_system_message_body_text(
            parent,
            index,
            line,
            SystemMessageTextStyle::ImageWindow,
            layout_text_node(SystemMessageTextStyle::ImageWindow, None, None, false),
            SYSTEM_MESSAGE_IMAGE_WINDOW_COLOR,
            assets,
        );
        // Clean type 14 and the generic multiline branch both append five
        // pixels after every Imagewindow label, including the final one.
        if combination_failure || index < lines.len() {
            spawn_system_message_gap(parent, 5.0);
        }
    }
}

pub(super) fn spawn_delete_mission_body(
    parent: &mut ChildSpawnerCommands,
    lines: &[&str],
    assets: &SystemMessageUiAssets,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            spawn_clipped_delete_mission_text(
                row,
                0,
                lines.first().copied().unwrap_or_default(),
                55.0,
                10.0,
                SYSTEM_MESSAGE_LABEL_COLOR,
                assets,
            );
            spawn_clipped_delete_mission_text(
                row,
                1,
                lines.get(1).copied().unwrap_or_default(),
                350.0,
                20.0,
                SYSTEM_MESSAGE_DELETE_SUBJECT_COLOR,
                assets,
            );
        });
    spawn_system_message_gap(parent, 20.0);
    // Clean indexes exactly array[2] and array[3]; later lines are not drawn.
    for index in 2..=3 {
        spawn_system_message_body_text(
            parent,
            index,
            lines.get(index).copied().unwrap_or_default(),
            SystemMessageTextStyle::ImageWindow,
            layout_text_node(SystemMessageTextStyle::ImageWindow, None, None, false),
            SYSTEM_MESSAGE_IMAGE_WINDOW_COLOR,
            assets,
        );
    }
}

pub(super) fn spawn_clipped_delete_mission_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    text: &str,
    width: f32,
    height: f32,
    color: [f32; 4],
    assets: &SystemMessageUiAssets,
) {
    let style = SystemMessageTextStyle::Label;
    let spec = style.spec();
    parent
        .spawn((
            Node {
                width: px(width),
                height: px(height),
                flex_shrink: 0.0,
                margin: UiRect {
                    left: px(spec.margin[0]),
                    right: px(spec.margin[1]),
                    bottom: px(spec.margin[3]),
                    ..default()
                },
                overflow: Overflow::clip(),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|clip| {
            spawn_system_message_body_text(
                clip,
                index,
                text,
                style,
                Node {
                    width: percent(100),
                    // Unity lays the label out at its intrinsic font height and
                    // clips the finished draw to GUILayout.Height(10/20).  An
                    // explicit inner height preserves that order in Bevy;
                    // constraining the Text itself to the clip height would
                    // collapse the first 10 px label after its 3+3 padding.
                    height: px(spec.line_height + spec.padding[2] + spec.padding[3]),
                    flex_shrink: 0.0,
                    padding: style_padding(spec.padding),
                    ..default()
                },
                color,
                assets,
            );
        });
}

pub(super) fn spawn_system_message_gap(parent: &mut ChildSpawnerCommands, height: f32) {
    parent.spawn((
        Node {
            width: percent(100),
            height: px(height),
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_system_message_body_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    text: &str,
    style: SystemMessageTextStyle,
    node: Node,
    color: [f32; 4],
    assets: &SystemMessageUiAssets,
) {
    let spec = style.spec();
    parent.spawn((
        SystemMessageBody,
        SystemMessageBodyLine { index },
        style,
        node,
        Text::new(""),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", text),
        (
            TextFont {
                font: (assets.text_font(style)).into(),
                font_size: (spec.font_size).into(),
                ..default()
            },
            LineHeight::Px(spec.line_height),
        ),
        TextColor(array_color(color)),
        TextLayout::new(spec.justify, spec.linebreak),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_system_message_button(
    parent: &mut ChildSpawnerCommands,
    spec: SystemMessageButtonSpec,
    enabled: bool,
    assets: &SystemMessageUiAssets,
) {
    let text_style = system_message_button_text_style(spec.visual);
    let text_spec = text_style.spec();
    let mut image = sliced_button_image(
        assets.button_image(spec.visual, Interaction::None),
        SYSTEM_MESSAGE_BUTTON_BORDER,
    );
    if !enabled {
        image.color = Color::srgba(0.52, 0.52, 0.52, 0.78);
    }
    let mut button_entity = parent.spawn((
        Button,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: system_message_button_padding(spec.visual),
            ..spec.rect.node()
        },
        image,
        SystemMessageButton {
            choice: spec.choice,
            visual: spec.visual,
        },
    ));
    if !enabled {
        button_entity.insert(Pickable::IGNORE);
    }
    button_entity.with_children(|button| {
        let font = (
            TextFont {
                font: (assets.text_font(text_style)).into(),
                font_size: (text_spec.font_size).into(),
                ..default()
            },
            LineHeight::Px(text_spec.line_height),
        );
        let horizontal_padding = text_spec.padding[0] + text_spec.padding[1];
        let vertical_padding = text_spec.padding[2] + text_spec.padding[3];
        button.spawn((
            Text::new(""),
            system_message_button_localized(spec),
            font.clone(),
            TextColor(system_message_button_text_color(spec.visual, "normal")),
            TextLayout::new(text_spec.justify, text_spec.linebreak),
            UiTextAutoFit::new(
                (spec.rect.width - horizontal_padding).max(1.0),
                (spec.rect.height - vertical_padding).max(1.0),
                &font,
            ),
            text_style,
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
    });
}
