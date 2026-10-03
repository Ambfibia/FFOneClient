use super::*;

pub(super) fn spawn_rule_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = RuleUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());

    commands
        .spawn((
            RuleUiRoot,
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
            GlobalZIndex(RULE_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                RuleUiBackground,
                RULE_UI_BACKGROUND_RECT.node(),
                UiTransform::default(),
                stretch_image(assets.panel_back.clone()),
                Pickable::IGNORE,
            ));

            root.spawn((
                RuleUiWindow,
                RULE_UI_WINDOW_RECT.node(),
                UiTransform::default(),
                Pickable::IGNORE,
            ))
            .with_children(|window| {
                window.spawn((
                    RULE_UI_RULE_BACK_RECT.node(),
                    stretch_image(assets.rule_back.clone()),
                    Pickable::IGNORE,
                ));

                spawn_rule_text(
                    window,
                    RULE_UI_TITLE_RECT,
                    RuleUiTextRole::Title,
                    RULE_UI_VEHICLE_STRINGS[0],
                    &assets.jeffe,
                    RULE_UI_JEFFE_16_FONT_SIZE,
                    RULE_UI_JEFFE_16_LINE_HEIGHT,
                    rgba(RULE_UI_CYAN_TEXT_COLOR),
                    Justify::Left,
                    true,
                    LineBreak::NoWrap,
                );

                window
                    .spawn((
                        RuleUiFrame,
                        Node {
                            overflow: Overflow::clip(),
                            ..RULE_UI_FRAME_RECT.node()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|frame| {
                        spawn_rule_text(
                            frame,
                            RULE_UI_SUBTITLE_1_RECT,
                            RuleUiTextRole::Subtitle1,
                            RULE_UI_VEHICLE_STRINGS[1],
                            &assets.jeffe,
                            RULE_UI_JEFFE_16_FONT_SIZE,
                            RULE_UI_JEFFE_16_LINE_HEIGHT,
                            rgba(RULE_UI_BLUE_TEXT_COLOR),
                            Justify::Center,
                            true,
                            LineBreak::NoWrap,
                        );
                        spawn_rule_text(
                            frame,
                            RULE_UI_SUBTITLE_2_RECT,
                            RuleUiTextRole::Subtitle2,
                            RULE_UI_VEHICLE_STRINGS[2],
                            &assets.jeffe,
                            RULE_UI_JEFFE_16_FONT_SIZE,
                            RULE_UI_JEFFE_16_LINE_HEIGHT,
                            rgba(RULE_UI_BLUE_TEXT_COLOR),
                            Justify::Left,
                            true,
                            LineBreak::NoWrap,
                        );
                        spawn_rule_text(
                            frame,
                            RULE_UI_CONTENT_1_RECT,
                            RuleUiTextRole::Content1,
                            RULE_UI_VEHICLE_STRINGS[3],
                            &assets.chalet,
                            RULE_UI_CHALET_SMALL_FONT_SIZE,
                            RULE_UI_CHALET_SMALL_LINE_HEIGHT,
                            rgba(RULE_UI_BLUE_TEXT_COLOR),
                            Justify::Left,
                            false,
                            LineBreak::WordBoundary,
                        );
                        spawn_rule_text(
                            frame,
                            RULE_UI_SUBTITLE_3_RECT,
                            RuleUiTextRole::Subtitle3,
                            RULE_UI_VEHICLE_STRINGS[4],
                            &assets.jeffe,
                            RULE_UI_JEFFE_16_FONT_SIZE,
                            RULE_UI_JEFFE_16_LINE_HEIGHT,
                            rgba(RULE_UI_BLUE_TEXT_COLOR),
                            Justify::Left,
                            true,
                            LineBreak::NoWrap,
                        );
                        spawn_rule_text(
                            frame,
                            RULE_UI_CONTENT_2_RECT,
                            RuleUiTextRole::Content2,
                            RULE_UI_VEHICLE_STRINGS[5],
                            &assets.chalet,
                            RULE_UI_CHALET_SMALL_FONT_SIZE,
                            RULE_UI_CHALET_SMALL_LINE_HEIGHT,
                            rgba(RULE_UI_BLUE_TEXT_COLOR),
                            Justify::Left,
                            false,
                            LineBreak::WordBoundary,
                        );
                        spawn_rule_text(
                            frame,
                            RULE_UI_LAST_COMMENT_RECT,
                            RuleUiTextRole::LastComment,
                            RULE_UI_VEHICLE_STRINGS[6],
                            &assets.jeffe,
                            RULE_UI_JEFFE_16_FONT_SIZE,
                            RULE_UI_JEFFE_16_LINE_HEIGHT,
                            rgba(RULE_UI_BLUE_TEXT_COLOR),
                            Justify::Center,
                            true,
                            LineBreak::NoWrap,
                        );

                        // `cnRule` paints all text before the four page images.
                        for (slot, rect) in RULE_UI_IMAGE_RECTS.into_iter().enumerate() {
                            frame.spawn((
                                RuleUiIllustration { slot },
                                rect.node(),
                                stretch_image(assets.page_images[0][slot].clone()),
                                Pickable::IGNORE,
                            ));
                        }
                    });

                spawn_rule_button(
                    window,
                    RuleUiButtonKind::Close,
                    RULE_UI_CLOSE_RECT,
                    None,
                    stretch_image(assets.close_normal.clone()),
                    &assets.jeffe,
                );
                spawn_rule_button(
                    window,
                    RuleUiButtonKind::Back,
                    RULE_UI_BACK_RECT,
                    Some((RuleUiTextRole::BackButton, RULE_UI_BACK_LABEL_KEY)),
                    sliced_image(assets.back_normal.clone(), RULE_UI_BUTTON_BORDER),
                    &assets.jeffe,
                );
                spawn_rule_button(
                    window,
                    RuleUiButtonKind::Previous,
                    RULE_UI_PREVIOUS_RECT,
                    Some((RuleUiTextRole::PreviousButton, RULE_UI_PREVIOUS_LABEL_KEY)),
                    sliced_image(assets.nav_normal.clone(), RULE_UI_BUTTON_BORDER),
                    &assets.jeffe,
                );
                spawn_rule_button(
                    window,
                    RuleUiButtonKind::Next,
                    RULE_UI_NEXT_RECT,
                    Some((RuleUiTextRole::NextButton, RULE_UI_NEXT_LABEL_KEY)),
                    sliced_image(assets.nav_normal.clone(), RULE_UI_BUTTON_BORDER),
                    &assets.jeffe,
                );
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_rule_text(
    parent: &mut ChildSpawnerCommands,
    rect: RuleUiRect,
    role: RuleUiTextRole,
    value: &'static str,
    font: &Handle<Font>,
    font_size: f32,
    line_height: f32,
    color: Color,
    justify: Justify,
    vertically_centered: bool,
    line_break: LineBreak,
) {
    let rect = if vertically_centered {
        RuleUiRect::new(
            rect.x,
            rect.y + (rect.height - line_height) * 0.5,
            rect.width,
            line_height,
        )
    } else {
        rect
    };
    parent.spawn((
        RuleUiTextElement { role },
        rect.node(),
        Text::new(value),
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(line_height),
        ),
        TextColor(color),
        TextLayout::new(justify, line_break),
        rule_page_text_localized(RulePageId::Vehicle, role, value),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_rule_button(
    parent: &mut ChildSpawnerCommands,
    kind: RuleUiButtonKind,
    rect: RuleUiRect,
    label: Option<(RuleUiTextRole, &'static str)>,
    image: ImageNode,
    font: &Handle<Font>,
) {
    let mut node = rect.node();
    node.justify_content = JustifyContent::Center;
    node.align_items = AlignItems::Center;
    node.padding = match kind {
        RuleUiButtonKind::Back => UiRect {
            bottom: px(2),
            ..UiRect::ZERO
        },
        RuleUiButtonKind::Previous | RuleUiButtonKind::Next => UiRect {
            left: px(6),
            right: px(6),
            top: px(3),
            bottom: px(3),
        },
        RuleUiButtonKind::Close => UiRect::ZERO,
    };
    if matches!(kind, RuleUiButtonKind::Previous | RuleUiButtonKind::Next) {
        node.display = Display::None;
    }

    parent
        .spawn((
            Button,
            RuleUiButton { kind },
            node,
            image,
            Pickable::default(),
        ))
        .with_children(|button| {
            if let Some((role, label)) = label {
                button.spawn((
                    RuleUiTextElement { role },
                    Text::new(label),
                    (
                        TextFont {
                            font: (font.clone()).into(),
                            font_size: (RULE_UI_JEFFE_12_FONT_SIZE).into(),
                            ..default()
                        },
                        LineHeight::Px(RULE_UI_JEFFE_12_LINE_HEIGHT),
                    ),
                    TextColor(rgba(match kind {
                        RuleUiButtonKind::Back => RULE_UI_BACK_NORMAL_TEXT_COLOR,
                        RuleUiButtonKind::Previous | RuleUiButtonKind::Next => {
                            RULE_UI_BUTTON_NORMAL_TEXT_COLOR
                        }
                        RuleUiButtonKind::Close => RULE_UI_BUTTON_NORMAL_TEXT_COLOR,
                    })),
                    TextLayout::new(Justify::Center, LineBreak::NoWrap),
                    rule_button_localized(role, label),
                    Pickable::IGNORE,
                ));
            }
        });
}
