use super::*;

pub(super) fn spawn_guide_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<(), With<GuideUiRoot>>,
) {
    if !existing.is_empty() {
        return;
    }
    let assets = GuideUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            GuideUiRoot,
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
            GlobalZIndex(GUIDE_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                GuideUiBackground,
                absolute_node(GUIDE_BACKGROUND_RECT),
                UiTransform::default(),
                stretch_image(assets.select_background.clone()),
                Pickable::IGNORE,
            ));
            spawn_guide_selection_window(root, &assets);
            spawn_guide_warp_window(root, &assets);
            root.spawn((
                GuideUiConfirmationOverlay,
                absolute_node(GuideUiRect::default()),
                UiTransform::default(),
                ImageNode {
                    image: assets.black.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable {
                    should_block_lower: true,
                    is_hoverable: false,
                },
            ));
            spawn_guide_confirmation_window(root, &assets);
        });
}

pub(super) fn spawn_guide_selection_window(parent: &mut ChildSpawnerCommands, assets: &GuideUiAssets) {
    parent
        .spawn((
            GuideUiSelectionWindow,
            absolute_node(GUIDE_WINDOW_RECT),
            UiTransform::default(),
            stretch_image(assets.panel.clone()),
            Pickable::IGNORE,
        ))
        .with_children(|window| {
            spawn_image(
                window,
                GUIDE_COMPUTRESS_FRAME_RECT,
                sliced_image(
                    assets.computress_frame.clone(),
                    GUIDE_COMPUTRESS_FRAME_BORDER,
                ),
            );
            spawn_text(
                window,
                GUIDE_COMPUTRESS_FRAME_RECT,
                GuideUiTextRole::Computress,
                GUIDE_COMPUTRESS_LABEL,
                &assets.jeffe,
                GUIDE_JEFFE_14_FONT_SIZE,
                GUIDE_JEFFE_14_LINE_HEIGHT,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Left,
                LineBreak::WordBoundary,
                GUIDE_COMPUTRESS_TEXT_PADDING,
            );
            spawn_image(
                window,
                GUIDE_COMPUTRESS_ICON_RECT,
                sliced_image(assets.icon_frame.clone(), GUIDE_ICON_FRAME_BORDER),
            );
            spawn_image(
                window,
                GUIDE_COMPUTRESS_ICON_RECT,
                stretch_image(assets.computress_icon.clone()),
            );
            window
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        ..absolute_node(GUIDE_HEADING_RECT)
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|heading| {
                    heading.spawn((
                        GuideUiSelectionHeading,
                        GuideUiTextElement {
                            role: GuideUiTextRole::SelectionHeading,
                        },
                        Node {
                            width: percent(100),
                            padding: GUIDE_BIG_FONT_TEXT_PADDING,
                            ..default()
                        },
                        Text::new(GUIDE_CHOOSE_HEADING),
                        guide_selection_heading_localized_text(GuideUiPurpose::InitialSelection),
                        (
                            TextFont {
                                font: (assets.jeffe.clone()).into(),
                                font_size: (GUIDE_JEFFE_16_FONT_SIZE).into(),
                                ..default()
                            },
                            LineHeight::Px(GUIDE_JEFFE_16_LINE_HEIGHT),
                        ),
                        TextColor(Color::srgb(1.0, 0.995_967_75, 0.0)),
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        Pickable::IGNORE,
                    ));
                });
            window.spawn((
                GuideUiSelectionIntro,
                GuideUiTextElement {
                    role: GuideUiTextRole::SelectionIntro,
                },
                absolute_node(GUIDE_INTRO_RECT),
                Text::new(GUIDE_CHOOSE_INTRO),
                guide_selection_intro_localized_text(GuideUiPurpose::InitialSelection),
                (
                    TextFont {
                        font: (assets.chalet.clone()).into(),
                        font_size: (GUIDE_CHALET_SMALL_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(GUIDE_CHALET_SMALL_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            for mentor in GuideMentor::CLEAN_ORDER {
                spawn_guide_mentor(window, mentor, assets);
            }
            spawn_guide_cost(window, assets);
            spawn_guide_button(
                window,
                GUIDE_PRIMARY_BUTTON_RECT,
                GuideUiCommand::OpenConfirmation,
                GuideUiButtonVisual::BigBlue,
                GUIDE_CHOOSE_BUTTON_LABEL,
                assets,
            );
            spawn_guide_button(
                window,
                GUIDE_CANCEL_BUTTON_RECT,
                GuideUiCommand::Dismiss(GuideUiDismissalSource::CancelButton),
                GuideUiButtonVisual::Cancel,
                GUIDE_CANCEL_LABEL,
                assets,
            );
            spawn_guide_button(
                window,
                GUIDE_CLOSE_BUTTON_RECT,
                GuideUiCommand::Dismiss(GuideUiDismissalSource::CloseButton),
                GuideUiButtonVisual::Close,
                "",
                assets,
            );
            spawn_guide_button(
                window,
                GUIDE_HELP_BUTTON_RECT,
                GuideUiCommand::RequestHelp,
                GuideUiButtonVisual::Help,
                "",
                assets,
            );
        });
}

pub(super) fn spawn_guide_mentor(
    parent: &mut ChildSpawnerCommands,
    mentor: GuideMentor,
    assets: &GuideUiAssets,
) {
    let card_rect = mentor.card_rect();
    // Original draw order: Box(window), Box(FrameWindow), BeginGroup(card).
    spawn_image(parent, card_rect, sliced_image(assets.card_frame.clone(), GUIDE_CARD_BORDER));
    parent
        .spawn((
            GuideUiMentorCurrentFrame { mentor },
            absolute_node(mentor.current_frame_rect()),
            sliced_image(assets.current_frame.clone(), GUIDE_CURRENT_FRAME_BORDER),
            Pickable::IGNORE,
        ))
        .with_children(|frame| {
            spawn_text(
                frame,
                GuideUiRect::new(
                    0.0,
                    0.0,
                    mentor.current_frame_rect().width,
                    mentor.current_frame_rect().height,
                ),
                GuideUiTextRole::CurrentGuide,
                GUIDE_CURRENT_LABEL,
                &assets.jeffe,
                GUIDE_JEFFE_14_FONT_SIZE,
                GUIDE_JEFFE_14_LINE_HEIGHT,
                Color::srgb(0.0, 0.2, 0.4),
                Justify::Center,
                LineBreak::WordBoundary,
                GUIDE_CURRENT_TEXT_PADDING,
            );
        });

    parent
        .spawn((
            GuideUiMentorCard { mentor },
            Node {
                overflow: Overflow::clip(),
                ..absolute_node(card_rect)
            },
            Pickable::IGNORE,
        ))
        .with_children(|card| {
            spawn_text(
                card,
                GuideUiRect::new(0.0, 0.0, card_rect.width, card_rect.height),
                GuideUiTextRole::MentorName(mentor),
                mentor.name(),
                &assets.jeffe,
                GUIDE_JEFFE_14_FONT_SIZE,
                GUIDE_JEFFE_14_LINE_HEIGHT,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Left,
                LineBreak::NoWrap,
                GUIDE_CARD_TEXT_PADDING,
            );
            spawn_guide_button(
                card,
                GUIDE_TOGGLE_RECT,
                GuideUiCommand::SelectMentor(mentor),
                GuideUiButtonVisual::Toggle(mentor),
                "",
                assets,
            );
            spawn_text(
                card,
                GUIDE_COMMENT_RECT,
                GuideUiTextRole::MentorDescription(mentor),
                mentor.description(),
                &assets.chalet,
                GUIDE_CHALET_SMALL_FONT_SIZE,
                GUIDE_CHALET_SMALL_LINE_HEIGHT,
                Color::WHITE,
                Justify::Left,
                LineBreak::WordBoundary,
                UiRect::ZERO,
            );
        });
    // The header badge straddles the card edge. Keep it outside the text
    // group's clip so the full badge remains visible over the frame.
    let icon_size = mentor.icon_size();
    spawn_image(
        parent,
        GuideUiRect::new(
            card_rect.x - icon_size.x * 0.5,
            card_rect.y,
            icon_size.x,
            icon_size.y,
        ),
        stretch_image(assets.icons[mentor.slot()].clone()),
    );
    // cnGuideMode.OnChange IL_04c2..05a7: selected light first, portrait last.
    parent.spawn((
        GuideUiMentorSelectedEffect { mentor },
        absolute_node(mentor.selected_effect_rect()),
        stretch_image(assets.selected_effect.clone()),
        Pickable::IGNORE,
    ));
    parent.spawn((
        absolute_node(mentor.portrait_rect()),
        stretch_image(assets.portraits[mentor.slot()].clone()),
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_guide_cost(parent: &mut ChildSpawnerCommands, assets: &GuideUiAssets) {
    parent
        .spawn((
            GuideUiCostGroup,
            absolute_node(GuideUiRect::new(0.0, 0.0, GUIDE_WINDOW_RECT.width, 654.0)),
            Pickable::IGNORE,
        ))
        .with_children(|cost| {
            spawn_image(
                cost,
                GUIDE_COST_BAR_RECT,
                stretch_image(assets.cost_bar.clone()),
            );
            cost.spawn((
                GuideUiCostText,
                GuideUiTextElement {
                    role: GuideUiTextRole::Cost,
                },
                Node {
                    width: Val::Auto,
                    padding: GUIDE_LABEL_TEXT_PADDING,
                    ..absolute_node(GuideUiRect::new(503.0, 572.0, 0.0, 20.0))
                },
                Text::new(""),
                guide_cost_localized_text(0),
                (
                    TextFont {
                        font: (assets.jeffe.clone()).into(),
                        font_size: (GUIDE_JEFFE_14_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(GUIDE_JEFFE_14_LINE_HEIGHT),
                ),
                TextColor(Color::BLACK),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            cost.spawn((
                GuideUiCostIcon,
                absolute_node(GuideUiRect::new(503.0, 567.0, 31.0, 31.0)),
                stretch_image(assets.fusion_matter_icon.clone()),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_guide_warp_window(parent: &mut ChildSpawnerCommands, assets: &GuideUiAssets) {
    parent
        .spawn((
            GuideUiWarpWindow,
            absolute_node(GUIDE_WINDOW_RECT),
            UiTransform::default(),
            Pickable::IGNORE,
        ))
        .with_children(|window| {
            spawn_image(
                window,
                GuideUiRect::new(287.5, 29.0, 461.0, 216.0),
                stretch_image(assets.member_back.clone()),
            );
            spawn_image(
                window,
                GUIDE_MODAL_RECT,
                sliced_image(assets.dialog.clone(), GUIDE_DIALOG_BORDER),
            );
            spawn_image(
                window,
                GUIDE_MODAL_ICON_RECT.translated(GUIDE_MODAL_RECT.x, GUIDE_MODAL_RECT.y),
                stretch_image(assets.alert_icon.clone()),
            );
            spawn_text(
                window,
                GUIDE_MODAL_TITLE_RECT,
                GuideUiTextRole::WarpTitle,
                GUIDE_WARP_TITLE,
                &assets.jeffe,
                GUIDE_JEFFE_14_FONT_SIZE,
                GUIDE_JEFFE_14_LINE_HEIGHT,
                Color::srgb(0.9, 0.9, 0.9),
                Justify::Left,
                LineBreak::WordBoundary,
                GUIDE_LABEL_TEXT_PADDING,
            );
            spawn_text(
                window,
                GUIDE_MODAL_BODY_RECT,
                GuideUiTextRole::WarpBody,
                GUIDE_WARP_BODY,
                &assets.chalet,
                GUIDE_CHALET_SMALL_FONT_SIZE,
                GUIDE_CHALET_SMALL_LINE_HEIGHT,
                Color::WHITE,
                Justify::Left,
                LineBreak::WordBoundary,
                UiRect::ZERO,
            );
            spawn_guide_button(
                window,
                GUIDE_WARP_BUTTON_RECT.translated(GUIDE_MODAL_RECT.x, GUIDE_MODAL_RECT.y),
                GuideUiCommand::AcceptWarpWarning,
                GuideUiButtonVisual::Blue,
                GUIDE_WARP_BUTTON_LABEL,
                assets,
            );
            spawn_guide_button(
                window,
                GUIDE_MODAL_CANCEL_RECT.translated(GUIDE_MODAL_RECT.x, GUIDE_MODAL_RECT.y),
                GuideUiCommand::Dismiss(GuideUiDismissalSource::WarpCancelButton),
                GuideUiButtonVisual::Cancel,
                GUIDE_CANCEL_LABEL,
                assets,
            );
        });
}

pub(super) fn spawn_guide_confirmation_window(parent: &mut ChildSpawnerCommands, assets: &GuideUiAssets) {
    parent
        .spawn((
            GuideUiConfirmationWindow,
            crate::ui::shared::controller::ControllerUiBoundary,
            absolute_node(GUIDE_WINDOW_RECT),
            UiTransform::default(),
            Pickable::IGNORE,
        ))
        .with_children(|window| {
            window.spawn((
                GuideUiConfirmationArt,
                absolute_node(GuideMentor::BenTennyson.confirm_art_rect()),
                stretch_image(assets.confirmation_art[0].clone()),
                Pickable::IGNORE,
            ));
            spawn_image(
                window,
                GUIDE_MODAL_RECT,
                sliced_image(assets.dialog.clone(), GUIDE_DIALOG_BORDER),
            );
            spawn_image(
                window,
                GUIDE_MODAL_ICON_RECT.translated(GUIDE_MODAL_RECT.x, GUIDE_MODAL_RECT.y),
                stretch_image(assets.confirm_icon.clone()),
            );
            window.spawn((
                GuideUiConfirmationTitle,
                GuideUiTextElement {
                    role: GuideUiTextRole::ConfirmationTitle,
                },
                Node {
                    padding: GUIDE_LABEL_TEXT_PADDING,
                    ..absolute_node(GUIDE_MODAL_TITLE_RECT)
                },
                Text::new(""),
                guide_confirmation_title_localized_text(GuideUiPurpose::InitialSelection),
                (
                    TextFont {
                        font: (assets.jeffe.clone()).into(),
                        font_size: (GUIDE_JEFFE_14_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(GUIDE_JEFFE_14_LINE_HEIGHT),
                ),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            window.spawn((
                GuideUiConfirmationBody,
                GuideUiTextElement {
                    role: GuideUiTextRole::ConfirmationBody,
                },
                absolute_node(GUIDE_MODAL_BODY_RECT),
                Text::new(""),
                guide_confirmation_body_localized_text(
                    GuideUiPurpose::InitialSelection,
                    GuideMentor::BenTennyson,
                ),
                (
                    TextFont {
                        font: (assets.chalet.clone()).into(),
                        font_size: (GUIDE_CHALET_SMALL_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(GUIDE_CHALET_SMALL_LINE_HEIGHT),
                ),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            spawn_guide_button(
                window,
                GUIDE_MODAL_CONFIRM_RECT.translated(GUIDE_MODAL_RECT.x, GUIDE_MODAL_RECT.y),
                GuideUiCommand::ConfirmMentor,
                GuideUiButtonVisual::Blue,
                GUIDE_CONFIRM_LABEL,
                assets,
            );
            spawn_guide_button(
                window,
                GUIDE_MODAL_CANCEL_RECT.translated(GUIDE_MODAL_RECT.x, GUIDE_MODAL_RECT.y),
                GuideUiCommand::CancelConfirmation,
                GuideUiButtonVisual::Cancel,
                GUIDE_CANCEL_LABEL,
                assets,
            );
        });
}

pub(super) fn spawn_guide_button(
    parent: &mut ChildSpawnerCommands,
    rect: GuideUiRect,
    command: GuideUiCommand,
    visual: GuideUiButtonVisual,
    label: &'static str,
    assets: &GuideUiAssets,
) {
    let image = button_image(assets, visual, Interaction::None);
    let font_size = if visual == GuideUiButtonVisual::BigBlue {
        GUIDE_JEFFE_16_FONT_SIZE
    } else {
        GUIDE_JEFFE_14_FONT_SIZE
    };
    let line_height = if visual == GuideUiButtonVisual::BigBlue {
        GUIDE_JEFFE_16_LINE_HEIGHT
    } else {
        GUIDE_JEFFE_14_LINE_HEIGHT
    };
    let mut button = parent
        .spawn((
            Button,
            GuideUiCommandButton { command, visual },
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: button_padding(visual),
                ..absolute_node(rect)
            },
            image,
            Pickable::default(),
        ));
    button.with_children(|button| {
            if !label.is_empty() {
                let role = match command {
                    GuideUiCommand::AcceptWarpWarning => GuideUiTextRole::WarpButton,
                    GuideUiCommand::OpenConfirmation => GuideUiTextRole::PrimaryButton,
                    GuideUiCommand::ConfirmMentor => GuideUiTextRole::CommonConfirm,
                    GuideUiCommand::CancelConfirmation
                    | GuideUiCommand::Dismiss(GuideUiDismissalSource::CancelButton)
                    | GuideUiCommand::Dismiss(GuideUiDismissalSource::WarpCancelButton) => {
                        GuideUiTextRole::CommonCancel
                    }
                    GuideUiCommand::SelectMentor(_)
                    | GuideUiCommand::RequestHelp
                    | GuideUiCommand::Dismiss(
                        GuideUiDismissalSource::CloseButton | GuideUiDismissalSource::EscapeKey,
                    ) => unreachable!("textless Guide controls must not spawn a label"),
                };
                button.spawn((
                    GuideUiButtonLabel,
                    GuideUiTextElement { role },
                    Text::new(label),
                    guide_static_localized_text(role, label),
                    (
                        TextFont {
                            font: (assets.jeffe.clone()).into(),
                            font_size: (font_size).into(),
                            ..default()
                        },
                        LineHeight::Px(line_height),
                    ),
                    TextColor(button_text_color(visual, Interaction::None)),
                    TextLayout::new(
                        Justify::Center,
                        if visual == GuideUiButtonVisual::BigBlue {
                            LineBreak::WordBoundary
                        } else {
                            LineBreak::NoWrap
                        },
                    ),
                    Pickable::IGNORE,
                ));
            }
        });
    match command {
        GuideUiCommand::SelectMentor(_) | GuideUiCommand::ConfirmMentor
        | GuideUiCommand::AcceptWarpWarning => {
            button.insert(crate::ui::shared::controller::ControllerUiDefault);
        }
        GuideUiCommand::Dismiss(GuideUiDismissalSource::CloseButton) => {
            button.insert(crate::ui::shared::controller::ControllerUiClose);
        }
        _ => {}
    }
}

pub(super) fn spawn_image(parent: &mut ChildSpawnerCommands, rect: GuideUiRect, image: ImageNode) -> Entity {
    parent
        .spawn((absolute_node(rect), image, Pickable::IGNORE))
        .id()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    rect: GuideUiRect,
    role: GuideUiTextRole,
    value: &'static str,
    font: &Handle<Font>,
    font_size: f32,
    line_height: f32,
    color: Color,
    justify: Justify,
    line_break: LineBreak,
    padding: UiRect,
) -> Entity {
    parent
        .spawn((
            GuideUiTextElement { role },
            Node {
                padding,
                ..absolute_node(rect)
            },
            Text::new(value),
            guide_static_localized_text(role, value),
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
            Pickable::IGNORE,
        ))
        .id()
}
