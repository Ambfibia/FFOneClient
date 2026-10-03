use super::*;

pub(super) fn spawn_quit_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = QuitMenuUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            QuitMenuUiRoot,
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
            GlobalZIndex(QUIT_MENU_UI_Z_INDEX),
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
        ))
        .with_children(|root| {
            root.spawn((
                QuitMenuBackdrop,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                UiTransform::default(),
                sliced_image(assets.backdrop.clone(), QUIT_MENU_BACKDROP_BORDER),
                Pickable::IGNORE,
            ));
            root.spawn((
                QuitMenuDialog,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: px(QUIT_MENU_DIALOG_RECT.width),
                    height: px(QUIT_MENU_DIALOG_RECT.height),
                    ..default()
                },
                UiTransform::default(),
                ImageNode {
                    image: assets.dialog.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|dialog| {
                for spec in QUIT_MENU_BUTTONS {
                    spawn_quit_menu_button(dialog, spec, &assets);
                }
            });
        });
}

pub(super) fn spawn_quit_menu_button(
    parent: &mut ChildSpawnerCommands,
    spec: QuitMenuButtonSpec,
    assets: &QuitMenuUiAssets,
) {
    let text_style = spec.visual.text_style();
    let border = match spec.visual {
        QuitMenuButtonVisual::Standard => QUIT_MENU_BUTTON_BORDER,
        QuitMenuButtonVisual::Cancel => QUIT_MENU_CANCEL_BORDER,
    };
    let [left, right, top, bottom] = text_style.padding;
    let padding = UiRect {
        left: px(left),
        right: px(right),
        top: px(top),
        bottom: px(bottom),
    };
    let mut background = sliced_image(assets.button_image(spec.visual, Interaction::None), border);
    // Unity GUIStyle padding constrains text, not the button's background rectangle.
    background.visual_box = bevy::ui::VisualBox::BorderBox;
    parent
        .spawn((
            Button,
            QuitMenuButton { kind: spec.kind },
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding,
                overflow: Overflow::clip(),
                ..spec.rect.node()
            },
            background,
            Pickable::default(),
        ))
        .with_children(|button| {
            button.spawn((
                QuitMenuButtonLabel { kind: spec.kind },
                Text::new(spec.label),
                LocalizedText::new(spec.localization_key, spec.label),
                (
                    TextFont {
                        font: (assets.font.clone()).into(),
                        font_size: (text_style.font_size).into(),
                        ..default()
                    },
                    LineHeight::Px(text_style.line_height),
                ),
                TextColor(button_text_color(spec.visual, Interaction::None)),
                TextLayout::new(
                    Justify::Center,
                    if text_style.word_wrap {
                        LineBreak::WordBoundary
                    } else {
                        LineBreak::NoWrap
                    },
                ),
                UiTransform::from_translation(Val2::px(0.0, text_style.y_offset)),
                QuitMenuTextStyle(spec.visual),
                Pickable::IGNORE,
            ));
        });
}
