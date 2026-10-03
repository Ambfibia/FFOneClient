use super::*;

pub(super) fn spawn_resurrect_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = ResurrectUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());

    commands
        .spawn((
            ResurrectUiRoot,
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
            GlobalZIndex(RESURRECT_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode {
                    image: assets.black_back.clone(),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1.0, 1.0, 1.0, RESURRECT_BACKDROP_ALPHA),
                    ..default()
                },
                Pickable::IGNORE,
            ));

            root.spawn((
                ResurrectDialog,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(RESURRECT_WINDOW_RECT.width),
                    height: px(RESURRECT_WINDOW_RECT.height),
                    // Grim deliberately extends 171 pixels above the panel.
                    // Only the screen root may clip this composition.
                    overflow: Overflow::visible(),
                    ..default()
                },
                UiTransform::default(),
                sliced_image(assets.dialog.clone(), RESURRECT_DIALOG_BORDER),
                Pickable::IGNORE,
            ))
            .with_children(|dialog| {
                spawn_stretched_image(dialog, RESURRECT_GRIM_RECT, assets.grim.clone());
                spawn_stretched_image(dialog, RESURRECT_ICON_RECT, assets.icon.clone());
                spawn_text_area(dialog, &assets);

                spawn_resurrect_button(dialog, ResurrectChoice::NearestResurrectEm, &assets, 0);
                spawn_resurrect_button(dialog, ResurrectChoice::PhoenixGroup, &assets, 0);
                spawn_resurrect_button(dialog, ResurrectChoice::PhoenixSelf, &assets, 0);
                // Clean draws the selected Phoenix button first, then its
                // backing/icon pair. These visuals must therefore cover the
                // left edge of REVIVE.
                spawn_phoenix_visual(dialog, ResurrectChoice::PhoenixGroup, &assets);
                spawn_phoenix_visual(dialog, ResurrectChoice::PhoenixSelf, &assets);
                // Clean OnGUI draws this last, in PhoenixButton rather than
                // the serialized UseItemButton rectangle. It covers both the
                // Phoenix button and icon when an item is available.
                spawn_resurrect_button(dialog, ResurrectChoice::UseItem, &assets, 2);
            });
        });
}

pub(super) fn spawn_stretched_image(
    parent: &mut ChildSpawnerCommands,
    rect: ResurrectUiRect,
    image: Handle<Image>,
) {
    parent.spawn((
        rect.node(),
        ImageNode {
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_text_area(parent: &mut ChildSpawnerCommands, assets: &ResurrectUiAssets) {
    let copy = ResurrectUiText::default();
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                ..RESURRECT_TEXT_LABEL_RECT.node()
            },
            Pickable::IGNORE,
        ))
        .with_children(|area| {
            area.spawn((
                ResurrectTextNode(ResurrectTextRole::Title),
                ResurrectTextStyle::Label,
                Text::new(copy.title.clone()),
                copy.title_localized(),
                (
                    TextFont {
                        font: (assets.button_font.clone()).into(),
                        font_size: (RESURRECT_BUTTON_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(RESURRECT_BUTTON_LINE_HEIGHT),
                ),
                TextColor(array_color(RESURRECT_LABEL_COLOR)),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Node {
                    align_self: AlignSelf::FlexStart,
                    margin: UiRect {
                        left: px(RESURRECT_LABEL_MARGIN[0]),
                        right: px(RESURRECT_LABEL_MARGIN[1]),
                        // `GUILayoutGroup` suppresses the first child's top
                        // margin but collapses its bottom margin against the
                        // zero-margin Imagewindow style.
                        bottom: px(RESURRECT_LABEL_MARGIN[3]),
                        ..default()
                    },
                    padding: UiRect {
                        top: px(RESURRECT_LABEL_PADDING[2]),
                        bottom: px(RESURRECT_LABEL_PADDING[3]),
                        ..default()
                    },
                    ..default()
                },
                Pickable::IGNORE,
            ));
            area.spawn((
                ResurrectTextNode(ResurrectTextRole::Question),
                ResurrectTextStyle::ImageWindow,
                Text::new(copy.question.clone()),
                copy.question_localized(),
                (
                    TextFont {
                        font: (assets.body_font.clone()).into(),
                        font_size: (RESURRECT_BODY_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(RESURRECT_BODY_LINE_HEIGHT),
                ),
                TextColor(array_color(RESURRECT_BODY_COLOR)),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
            area.spawn((Node {
                width: percent(100),
                height: px(10),
                flex_shrink: 0.0,
                ..default()
            },));
            area.spawn((
                ResurrectTextNode(ResurrectTextRole::Countdown),
                ResurrectTextStyle::ImageWindow,
                Text::new(copy.countdown(RESURRECT_TIMEOUT_SECONDS as i32)),
                copy.countdown_localized(RESURRECT_TIMEOUT_SECONDS as i32),
                (
                    TextFont {
                        font: (assets.body_font.clone()).into(),
                        font_size: (RESURRECT_BODY_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(RESURRECT_BODY_LINE_HEIGHT),
                ),
                TextColor(array_color(RESURRECT_COUNTDOWN_COLOR)),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_phoenix_visual(
    parent: &mut ChildSpawnerCommands,
    choice: ResurrectChoice,
    assets: &ResurrectUiAssets,
) {
    let image = match choice {
        ResurrectChoice::PhoenixGroup => assets.phoenix_group.clone(),
        ResurrectChoice::PhoenixSelf => assets.phoenix_self.clone(),
        _ => return,
    };
    parent.spawn((
        ResurrectPhoenixVisual { choice },
        ZIndex(1),
        RESURRECT_PHOENIX_SKILL_BACK_RECT.node(),
        ImageNode {
            image: assets.skill_icon_back.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        Pickable::IGNORE,
    ));
    parent.spawn((
        ResurrectPhoenixVisual { choice },
        ZIndex(1),
        RESURRECT_PHOENIX_SKILL_RECT.node(),
        ImageNode {
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_resurrect_button(
    parent: &mut ChildSpawnerCommands,
    choice: ResurrectChoice,
    assets: &ResurrectUiAssets,
    z_index: i32,
) {
    parent
        .spawn((
            Button,
            ResurrectUiButton { choice },
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..choice.rect().node()
            },
            ZIndex(z_index),
            sliced_image(assets.button_normal.clone(), RESURRECT_BUTTON_BORDER),
            Pickable::IGNORE,
        ))
        .with_children(|button| {
            button.spawn((
                ResurrectUiButtonLabel { choice },
                ResurrectTextStyle::Button,
                Text::new(choice.label()),
                ResurrectUiText::default().choice_localized(choice),
                (
                    TextFont {
                        font: (assets.button_font.clone()).into(),
                        font_size: (RESURRECT_BUTTON_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(RESURRECT_BUTTON_LINE_HEIGHT),
                ),
                TextColor(array_color(RESURRECT_BUTTON_NORMAL_COLOR)),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));
        });
}
