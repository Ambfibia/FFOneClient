use super::*;

pub(super) fn spawn_nano_free_tuning_presentation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    roots: Query<Entity, With<NanoFreeTuningPresentationRoot>>,
) {
    // The startup phase is re-entered after returning through login or
    // character selection. Keep this persistent presentation singleton:
    // duplicate roots make the old `single_mut()` binding silently skip the
    // visible-state update.
    if !roots.is_empty() {
        return;
    }
    let assets = NanoFreeTuningPresentationAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            NanoFreeTuningPresentationRoot,
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
            GlobalZIndex(NANO_FREE_TUNING_UI_Z_INDEX),
            FocusPolicy::Block,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                NanoFreeTuningPresentationBar { bottom: false },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    ..default()
                },
                stretched_image(assets.black.clone()),
                Pickable::IGNORE,
                FocusPolicy::Pass,
            ))
            .with_children(|top_bar| {
                top_bar
                    .spawn((
                        NanoFreeTuningAnnouncement,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            width: percent(100),
                            height: percent(100),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                        FocusPolicy::Pass,
                    ))
                    .with_children(|announcement| {
                        spawn_nano_free_tuning_text(
                            announcement,
                            NanoFreeTuningTextRole::AcquiredPrefix,
                            NANO_FREE_TUNING_COPY_ACQUIRED,
                            &assets.font,
                            NanoFreeTuningUiTextStyle::BigBlueMiddleLeft,
                            None,
                        );
                        spawn_nano_free_tuning_text(
                            announcement,
                            NanoFreeTuningTextRole::NanoName,
                            "",
                            &assets.font,
                            NanoFreeTuningUiTextStyle::BigYellowMiddleLeft,
                            None,
                        );
                        spawn_nano_free_tuning_text(
                            announcement,
                            NanoFreeTuningTextRole::Bang,
                            NANO_FREE_TUNING_COPY_BANG,
                            &assets.font,
                            NanoFreeTuningUiTextStyle::BigBlueMiddleLeft,
                            None,
                        );
                    });
            });
            root.spawn((
                NanoFreeTuningPresentationBar { bottom: true },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    width: percent(100),
                    ..default()
                },
                stretched_image(assets.black.clone()),
                Pickable::IGNORE,
                FocusPolicy::Pass,
            ));
            root.spawn((
                NanoFreeTuningPresentationPanel,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(NANO_FREE_TUNING_PANEL_WIDTH),
                    height: px(NANO_FREE_TUNING_PANEL_HEIGHT),
                    overflow: Overflow::visible(),
                    ..default()
                },
                stretched_image(assets.panel.clone()),
                Pickable::IGNORE,
                FocusPolicy::Pass,
            ))
            .with_children(|panel| {
                spawn_static_nano_free_tuning_text(
                    panel,
                    NANO_FREE_TUNING_TITLE_RECT,
                    NANO_FREE_TUNING_COPY_SELECT_POWER,
                    LocalizedText::new(
                        NANO_FREE_TUNING_TITLE_LOCALIZATION_KEY,
                        NANO_FREE_TUNING_COPY_SELECT_POWER,
                    ),
                    &assets.font,
                    NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
                );
                for (power_index, layout) in
                    NANO_FREE_TUNING_POWER_LAYOUTS.iter().copied().enumerate()
                {
                    let mut icon_slot = layout.icon.node();
                    let [left, right, top, bottom] = NanoFreeTuningIconLabelStyle::PADDING;
                    icon_slot.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
                    icon_slot.overflow = Overflow::clip();
                    panel
                        .spawn((
                            NanoFreeTuningIconLabelStyle,
                            icon_slot,
                            Pickable::IGNORE,
                            FocusPolicy::Pass,
                        ))
                        .with_children(|slot| {
                            slot.spawn((
                                NanoFreeTuningPowerIcon(power_index),
                                Node {
                                    width: percent(100),
                                    height: percent(100),
                                    ..default()
                                },
                                ImageNode::default(),
                                Pickable::IGNORE,
                                FocusPolicy::Pass,
                            ));
                        });
                    spawn_nano_free_tuning_text(
                        panel,
                        NanoFreeTuningTextRole::PowerName(power_index),
                        "",
                        &assets.font,
                        NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
                        Some(layout.name),
                    );
                    spawn_nano_free_tuning_text(
                        panel,
                        NanoFreeTuningTextRole::PowerType(power_index),
                        "",
                        &assets.font,
                        NanoFreeTuningUiTextStyle::TransparentYellowSmallUpperLeft,
                        Some(layout.power_type),
                    );
                    spawn_nano_free_tuning_text(
                        panel,
                        NanoFreeTuningTextRole::PowerDescription(power_index),
                        "",
                        &assets.font,
                        NanoFreeTuningUiTextStyle::TransparentBlueUpperLeft,
                        Some(layout.description),
                    );
                    let button_style = NanoFreeTuningUiTextStyle::ButtonMiddleCenter;
                    let mut button_node = layout.select_button.node();
                    button_style.apply_to_container(&mut button_node);
                    panel
                        .spawn((
                            Button,
                            Pickable::default(),
                            RelativeCursorPosition::default(),
                            NanoFreeTuningPowerButton { power_index },
                            button_node,
                            sliced_image(
                                assets.select_normal.clone(),
                                BorderRect {
                                    min_inset: Vec2::new(6.0, 6.0),
                                    max_inset: Vec2::new(6.0, 4.0),
                                },
                            ),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                NanoFreeTuningButtonLabel,
                                Text::new(NANO_FREE_TUNING_COPY_SELECT),
                                LocalizedText::new(
                                    NANO_FREE_TUNING_SELECT_LOCALIZATION_KEY,
                                    NANO_FREE_TUNING_COPY_SELECT,
                                ),
                                button_style,
                                button_style.font(&assets.font),
                                button_style.color(),
                                button_style.layout(),
                                UiTransform::from_translation(Val2::px(
                                    0.0,
                                    button_style.replacement_y_offset(),
                                )),
                                Pickable::IGNORE,
                                FocusPolicy::Pass,
                            ));
                        });
                }
            });
        });
}

pub(super) fn spawn_nano_free_tuning_text(
    parent: &mut ChildSpawnerCommands,
    role: NanoFreeTuningTextRole,
    copy: &str,
    font: &Handle<Font>,
    style: NanoFreeTuningUiTextStyle,
    rect: Option<NanoFreeTuningRect>,
) {
    let mut entity = parent.spawn((
        NanoFreeTuningText(role),
        Text::new(copy),
        nano_free_tuning_localized_text(role, copy),
        style,
        style.font(font),
        style.color(),
        style.layout(),
        UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
        Pickable::IGNORE,
        FocusPolicy::Pass,
    ));
    if let Some(rect) = rect {
        let mut node = rect.node();
        style.apply_to_container(&mut node);
        entity.insert(node);
    }
}

pub(super) fn spawn_static_nano_free_tuning_text(
    parent: &mut ChildSpawnerCommands,
    rect: NanoFreeTuningRect,
    copy: &str,
    localized: LocalizedText,
    font: &Handle<Font>,
    style: NanoFreeTuningUiTextStyle,
) {
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    parent.spawn((
        Text::new(copy),
        localized,
        style,
        style.font(font),
        style.color(),
        style.layout(),
        UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
        node,
        Pickable::IGNORE,
        FocusPolicy::Pass,
    ));
}
