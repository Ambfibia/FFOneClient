use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_login_text(
    parent: &mut ChildSpawnerCommands,
    node: Option<Node>,
    localized: LocalizedText,
    assets: &LoginUiAssets,
    style: LoginTextStyle0104,
    color: Color,
    marker: impl Bundle,
) {
    let fallback = localized.fallback.clone();
    let mut entity = parent.spawn((
        Text::new(fallback),
        localized,
        style.font(assets),
        TextColor(color),
        style.layout(),
        UiTransform::from_translation(Val2::px(0.0, style.y_offset())),
        style,
        marker,
    ));
    if matches!(style, LoginTextStyle0104::TextField) {
        entity
            .insert((
                EditVisual::default(),
                Node {
                    width: percent(100),
                    height: px(LOGIN_CHALET_LINE_HEIGHT.ceil()),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(text_edit::spawn_decorations);
    }
    if let Some(node) = node {
        entity.insert(node);
    }
}

pub(super) fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    key: &'static str,
    value: &'static str,
    rect: UiSourceRect,
    assets: &LoginUiAssets,
) {
    let mut node = rect.node();
    LoginTextStyle0104::Label.apply_to_control(&mut node);
    spawn_login_text(
        parent,
        Some(node),
        LocalizedText::new(key, value),
        assets,
        LoginTextStyle0104::Label,
        Color::WHITE,
        account_view::CredentialLabel,
    );
}

pub(super) fn spawn_input(
    parent: &mut ChildSpawnerCommands,
    field: LoginField,
    rect: UiSourceRect,
    background: Handle<Image>,
    assets: &LoginUiAssets,
) {
    let mut node = rect.node();
    LoginTextStyle0104::TextField.apply_to_control(&mut node);
    node.overflow = Overflow::clip();
    let mut entity = parent.spawn((
        Button,
        node,
        ImageNode {
            image: background,
            visual_box: bevy::ui::VisualBox::BorderBox,
            image_mode: NodeImageMode::Sliced(TextureSlicer {
                border: LOGIN_TEXT_FIELD_BORDER,
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
            ..default()
        },
    ));
    match field {
        LoginField::Username => {
            entity.insert(LoginUsernameField);
        }
        LoginField::Password => {
            entity.insert(LoginPasswordField);
        }
    }
    entity.with_children(|input| {
        let localized = LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "");
        match field {
            LoginField::Username => spawn_login_text(
                input,
                None,
                localized,
                assets,
                LoginTextStyle0104::TextField,
                Color::srgb(0.901_960_85, 0.901_960_85, 0.901_960_85),
                LoginUsernameText,
            ),
            LoginField::Password => spawn_login_text(
                input,
                None,
                localized,
                assets,
                LoginTextStyle0104::TextField,
                Color::srgb(0.901_960_85, 0.901_960_85, 0.901_960_85),
                LoginPasswordText,
            ),
        }
    });
}

pub(super) fn spawn_login_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = LoginUiAssets {
        check_empty: asset_server.load("ui/en/option/radio-empty.png"),
        check_checked: asset_server.load("ui/en/option/radio-checked.png"),
        scroll_track: asset_server.load("ui/en/user-equip/scroll-track.png"),
        scroll_thumb: asset_server.load("ui/en/user-equip/scroll-thumb.png"),
        background: asset_server.load(LOGIN_BACKGROUND_PATH),
        fallback_background: asset_server.load(LOGIN_FALLBACK_BACKGROUND_PATH),
        panel: asset_server.load(LOGIN_PANEL_PATH),
        button: asset_server.load(LOGIN_BUTTON_PATH),
        button_over: asset_server.load(LOGIN_BUTTON_OVER_PATH),
        button_active: asset_server.load(LOGIN_BUTTON_ACTIVE_PATH),
        text_field: asset_server.load(LOGIN_TEXT_FIELD_PATH),
        font: asset_server.load(LOGIN_FONT_PATH),
        text_field_font: asset_server.load(LOGIN_TEXT_FIELD_FONT_PATH),
        music: asset_server.load(LOGIN_MUSIC_PATH),
    };
    commands.insert_resource(assets.clone());
    commands.spawn((
        AudioPlayer::new(assets.music.clone()),
        PlaybackSettings::LOOP
            .paused()
            .with_volume(Volume::Linear(0.5)),
        LoginMusic,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            ZIndex(800),
            LoginRoot,
        ))
        .with_children(|root| {
            browser_view::spawn(root, &assets);
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: px(LOGIN_FALLBACK_BACKGROUND_SIZE.x),
                    height: px(LOGIN_FALLBACK_BACKGROUND_SIZE.y),
                    ..default()
                },
                ImageNode {
                    image: assets.fallback_background.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                LoginFallbackBackground,
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: px(LOGIN_BACKGROUND_SIZE.x),
                    height: px(LOGIN_BACKGROUND_SIZE.y),
                    ..default()
                },
                ImageNode {
                    image: assets.background.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                LoginLoadedBackground,
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: px(LOGIN_PANEL_SIZE.x),
                    height: px(LOGIN_PANEL_SIZE.y),
                    ..default()
                },
                ImageNode {
                    image: assets.panel.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: LOGIN_PANEL_BORDER,
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                UiTransform::default(),
                LoginPanel,
            ))
            .with_children(|panel| {
                account_view::spawn(panel);
                spawn_label(
                    panel,
                    "ui.login.username",
                    "Username :",
                    LOGIN_USERNAME_LABEL_RECT,
                    &assets,
                );
                spawn_input(
                    panel,
                    LoginField::Username,
                    LOGIN_USERNAME_FIELD_RECT,
                    assets.text_field.clone(),
                    &assets,
                );
                spawn_label(
                    panel,
                    "ui.login.password",
                    "Password :",
                    LOGIN_PASSWORD_LABEL_RECT,
                    &assets,
                );
                spawn_input(
                    panel,
                    LoginField::Password,
                    LOGIN_PASSWORD_FIELD_RECT,
                    assets.text_field.clone(),
                    &assets,
                );

                let mut submit_node = LOGIN_SUBMIT_RECT.node();
                LoginTextStyle0104::Button.apply_to_control(&mut submit_node);
                panel
                    .spawn((
                        Button,
                        submit_node,
                        sliced_button_image(assets.button.clone()),
                        LoginSubmitButton,
                        LoginStyledButton,
                    ))
                    .with_children(|button| {
                        spawn_login_text(
                            button,
                            None,
                            LocalizedText::new("ui.login.submit", "Log In"),
                            &assets,
                            LoginTextStyle0104::Button,
                            Color::srgb(0.9, 0.9, 0.9),
                            (LoginSubmitText, LoginButtonTextRole::Submit),
                        );
                    });

                let mut language_node = LOGIN_LANGUAGE_RECT.node();
                LoginTextStyle0104::Button.apply_to_control(&mut language_node);
                panel
                    .spawn((
                        Button,
                        language_node,
                        sliced_button_image(assets.button.clone()),
                        LoginLanguageButton,
                    ))
                    .with_children(|button| {
                        spawn_login_text(
                            button,
                            None,
                            LocalizedText::new("ui.login.language_value", "LANGUAGE: {language}")
                                .with_arg("language", "Russian"),
                            &assets,
                            LoginTextStyle0104::Button,
                            Color::srgb(0.9, 0.9, 0.9),
                            LoginLanguageText,
                        );
                    });

                panel
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(LOGIN_LANGUAGE_RECT.y + LOGIN_LANGUAGE_RECT.height + 8.0),
                            width: px(LOGIN_PANEL_SIZE.x),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        LoginGLayoutArea,
                    ))
                    .with_children(|area| {
                        area.spawn((
                            Node {
                                flex_grow: LOGIN_GLAYOUT_LEADING_FLEX,
                                flex_basis: px(0),
                                ..default()
                            },
                            LoginGLayoutFlexibleSpace,
                        ));
                        area.spawn((
                            Node {
                                width: px(LOGIN_SUBMIT_RECT.width),
                                min_width: px(LOGIN_SUBMIT_RECT.width),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Stretch,
                                row_gap: px(LOGIN_GLAYOUT_BUTTON_GAP),
                                ..default()
                            },
                            LoginGLayoutColumn,
                        ))
                        .with_children(|column| {
                            let mut register_node = Node {
                                height: px(LOGIN_SUBMIT_RECT.height),
                                ..default()
                            };
                            LoginTextStyle0104::Button.apply_to_control(&mut register_node);
                            column
                                .spawn((
                                    Button,
                                    register_node,
                                    sliced_button_image(assets.button.clone()),
                                    LoginRegisterButton,
                                    LoginStyledButton,
                                ))
                                .with_children(|button| {
                                    spawn_login_text(
                                        button,
                                        None,
                                        LocalizedText::new(
                                            "ui.login.register",
                                            "How Do I Register?",
                                        ),
                                        &assets,
                                        LoginTextStyle0104::Button,
                                        Color::srgb(0.9, 0.9, 0.9),
                                        LoginButtonTextRole::Register,
                                    );
                                });
                        });
                        area.spawn((
                            Node {
                                flex_grow: LOGIN_GLAYOUT_TRAILING_FLEX,
                                flex_basis: px(0),
                                ..default()
                            },
                            LoginGLayoutFlexibleSpace,
                        ));
                    });

                let mut status_node = Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    left: px(0),
                    top: px(280),
                    width: px(LOGIN_PANEL_SIZE.x),
                    ..default()
                };
                LoginTextStyle0104::StatusAdapter.apply_to_control(&mut status_node);
                spawn_login_text(
                    panel,
                    Some(status_node),
                    LocalizedText::new(
                        "status.login.credentials",
                        "Enter your account name and password.",
                    ),
                    &assets,
                    LoginTextStyle0104::StatusAdapter,
                    Color::srgb(1.0, 0.55, 0.45),
                    LoginStatusText,
                );
            });
        });
}
