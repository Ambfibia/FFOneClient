use super::*;

pub(super) fn login_can_assign_loaded_background(surface: LoginSurface) -> bool {
    matches!(
        surface,
        LoginSurface::Manual | LoginSurface::WaitingForWebAuthentication
    )
}

pub(super) fn queue_login(model: &mut LoginUiModel, outbox: &mut LoginUiOutbox) {
    if model.busy {
        return;
    }
    model.username = model
        .username
        .chars()
        .filter(|character| *character != '\n' && *character != '\r')
        .collect::<String>()
        .trim()
        .to_owned();
    model.password = model.password.trim().to_owned();
    if model.username.is_empty() || (model.password.is_empty()
        && model.saved_account.as_deref() != Some(model.username.as_str())) {
        // `CnLoginMode.SendLogin` simply releases `bButtonLogin` in this branch.
        model.busy = false;
        model.status.clear();
        return;
    }
    outbox.requests.push_back(LoginRequest {
        username: model.username.clone(),
        password: model.password.clone(),
    });
    model.busy = true;
    model.status = "Connecting to OpenFusion...".to_owned();
}

pub(super) fn bind_login_edit_visuals(
    model: Res<LoginUiModel>,
    option_ui: Res<OptionUiModel>,
    mut texts: Query<
        (&mut EditVisual, Has<LoginUsernameText>),
        Or<(With<LoginUsernameText>, With<LoginPasswordText>)>,
    >,
) {
    for (mut visual, username) in &mut texts {
        let field = if username {
            LoginField::Username
        } else {
            LoginField::Password
        };
        let active = model.accepts_manual_input() && !option_ui.visible && model.focused == field;
        let edit = if username {
            &model.username_edit
        } else {
            &model.password_edit
        };
        if visual.active != active {
            visual.active = active;
        }
        if visual.edit != *edit {
            visual.edit = edit.clone();
        }
    }
}

pub(super) fn bind_login_language(
    model: Res<LoginUiModel>,
    option_ui: Res<OptionUiModel>,
    assets: Res<LoginUiAssets>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    mut button: Single<(&Interaction, &mut ImageNode), With<LoginLanguageButton>>,
    mut text: Single<(&mut LocalizedText, &mut TextColor), With<LoginLanguageText>>,
) {
    let enabled = model.accepts_manual_input() && !option_ui.visible;
    let interaction = if enabled {
        *button.0
    } else {
        Interaction::None
    };
    button.1.image = match interaction {
        Interaction::Pressed => assets.button_active.clone(),
        Interaction::Hovered => assets.button_over.clone(),
        Interaction::None => assets.button.clone(),
    };
    text.1.0 = login_button_text_color(interaction);
    if let (Some(localization), Some(language)) = (localization, language) {
        let locale = language.effective.as_str();
        let name = match locale {
            "en" => localization.text(&language, &LocalizedText::new("language.en", "English")),
            "ru" => localization.text(&language, &LocalizedText::new("language.ru", "Russian")),
            other => other.to_ascii_uppercase(),
        };
        *text.0 = LocalizedText::new("ui.login.language_value", "LANGUAGE: {language}")
            .with_arg("language", name);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_login_ui(
    mut model: ResMut<LoginUiModel>,
    option_ui: Res<OptionUiModel>,
    assets: Res<LoginUiAssets>,
    asset_status: Res<LoginUiAssetStatus>,
    asset_server: Res<AssetServer>,
    mut loaded_background_state: ResMut<LoginLoadedBackgroundState>,
    mut root: Single<
        &mut Node,
        (
            With<LoginRoot>,
            Without<LoginFallbackBackground>,
            Without<LoginLoadedBackground>,
            Without<LoginPanel>,
            Without<LoginStatusText>,
        ),
    >,
    mut fallback_background: Single<
        &mut Node,
        (
            With<LoginFallbackBackground>,
            Without<LoginRoot>,
            Without<LoginLoadedBackground>,
            Without<LoginPanel>,
            Without<LoginStatusText>,
        ),
    >,
    mut loaded_background: Single<
        &mut Node,
        (
            With<LoginLoadedBackground>,
            Without<LoginRoot>,
            Without<LoginFallbackBackground>,
            Without<LoginPanel>,
            Without<LoginStatusText>,
        ),
    >,
    mut panel: Single<
        &mut Node,
        (
            With<LoginPanel>,
            Without<LoginRoot>,
            Without<LoginFallbackBackground>,
            Without<LoginLoadedBackground>,
            Without<LoginStatusText>,
        ),
    >,
    mut status_node: Single<
        &mut Node,
        (
            With<LoginStatusText>,
            Without<LoginRoot>,
            Without<LoginFallbackBackground>,
            Without<LoginLoadedBackground>,
            Without<LoginPanel>,
        ),
    >,
    mut buttons: Query<
        (
            &Interaction,
            &mut ImageNode,
            Option<&LoginSubmitButton>,
            Option<&LoginCommunityButton>,
            Option<&LoginRegisterButton>,
        ),
        With<LoginStyledButton>,
    >,
    fields: Query<
        (
            &Interaction,
            Option<&LoginUsernameField>,
            Option<&LoginPasswordField>,
        ),
        Or<(With<LoginUsernameField>, With<LoginPasswordField>)>,
    >,
    mut texts: Query<
        (
            &mut LocalizedText,
            &mut TextColor,
            Option<&LoginUsernameText>,
            Option<&LoginPasswordText>,
            Option<&LoginStatusText>,
            Option<&LoginButtonTextRole>,
        ),
        With<LoginTextStyle0104>,
    >,
) {
    if model.surface == LoginSurface::Manual {
        // Clean `PasswordField(...).Trim()` assigns on every form draw.
        model.password = model.password.trim().to_owned();
    }
    root.display = if model.visible && matches!(*asset_status, LoginUiAssetStatus::Ready) {
        Display::Flex
    } else {
        Display::None
    };
    let (background_mode, assigned_after_draw) = login_background_frame(
        model.surface,
        model.visible,
        loaded_background_state.assigned,
        matches!(
            asset_server.load_state(assets.background.id()),
            LoadState::Loaded
        ),
    );
    loaded_background_state.assigned = assigned_after_draw;
    fallback_background.display = if background_mode == LoginBackgroundMode0104::FallbackScaleToFit
    {
        Display::Flex
    } else {
        Display::None
    };
    loaded_background.display = if background_mode == LoginBackgroundMode0104::LoadedScaleAndCrop {
        Display::Flex
    } else {
        Display::None
    };
    panel.display = if model.surface == LoginSurface::Manual {
        Display::Flex
    } else {
        Display::None
    };

    let mut username_interaction = Interaction::None;
    let mut password_interaction = Interaction::None;
    for (interaction, username, password) in &fields {
        if username.is_some() {
            username_interaction = *interaction;
        }
        if password.is_some() {
            password_interaction = *interaction;
        }
    }
    let enabled = model.accepts_manual_input() && !option_ui.visible;
    let mut submit_interaction = Interaction::None;
    let mut community_interaction = Interaction::None;
    let mut register_interaction = Interaction::None;
    for (interaction, mut image, submit, community, register) in &mut buttons {
        let effective = if enabled {
            *interaction
        } else {
            Interaction::None
        };
        image.image = match effective {
            Interaction::Pressed => assets.button_active.clone(),
            Interaction::Hovered => assets.button_over.clone(),
            Interaction::None => assets.button.clone(),
        };
        if submit.is_some() {
            submit_interaction = effective;
        } else if community.is_some() {
            community_interaction = effective;
        } else if register.is_some() {
            register_interaction = effective;
        }
    }

    let username_localized =
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &model.username);
    let password_value = "*".repeat(model.password.chars().count());
    let password_localized =
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &password_value);
    let show_error = model.status.starts_with("Network error")
        || model.status.starts_with("Login failed")
        || model.status.starts_with("Disconnected")
        || model.status.starts_with("Offline");
    let status_localized = localized_status(&model.status);
    status_node.display = if show_error {
        Display::Flex
    } else {
        Display::None
    };
    for (mut localized, mut color, username, password, status, button_role) in &mut texts {
        if username.is_some() {
            *localized = username_localized.clone();
            color.0 = login_text_field_color(
                username_interaction,
                enabled && model.focused == LoginField::Username,
            );
        } else if password.is_some() {
            *localized = password_localized.clone();
            color.0 = login_text_field_color(
                password_interaction,
                enabled && model.focused == LoginField::Password,
            );
        } else if status.is_some() {
            *localized = status_localized.clone();
            color.0 = if show_error {
                Color::srgb(1.0, 0.55, 0.45)
            } else {
                Color::srgb(0.72, 0.88, 0.97)
            };
        } else if let Some(role) = button_role {
            let interaction = match role {
                LoginButtonTextRole::Submit => submit_interaction,
                LoginButtonTextRole::Community => community_interaction,
                LoginButtonTextRole::Register => register_interaction,
            };
            color.0 = login_button_text_color(interaction);
        }
    }
}

pub(super) fn login_text_field_color(interaction: Interaction, focused: bool) -> Color {
    if focused || interaction == Interaction::Pressed {
        Color::WHITE
    } else if interaction == Interaction::Hovered {
        Color::srgb(0.9, 0.9, 0.9)
    } else {
        Color::srgb(0.901_960_85, 0.901_960_85, 0.901_960_85)
    }
}

pub(super) fn control_login_music(
    model: Res<LoginUiModel>,
    loading: Option<Res<crate::world_audio::RetrobutionLoadingAudioState>>,
    mut sources: Query<(&mut PlaybackSettings, Option<&AudioSink>), With<LoginMusic>>,
) {
    let active = model.visible && !loading.is_some_and(|loading| loading.active);
    for (mut settings, sink) in &mut sources {
        settings.paused = !active;
        if let Some(sink) = sink {
            if active {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }
}
