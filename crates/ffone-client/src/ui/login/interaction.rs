use super::*;

pub const LOGIN_BUTTON_PATH: &str = "ui/en/server-selection/ff-button-normal.png";

pub const LOGIN_BUTTON_OVER_PATH: &str = "ui/en/server-selection/ff-button-hover.png";

pub const LOGIN_BUTTON_ACTIVE_PATH: &str = "ui/en/server-selection/ff-button-active.png";

pub const LOGIN_BUTTON_Y_OFFSET: f32 = 0.0;

pub const LOGIN_BUTTON_PADDING: UiRect = UiRect {
    left: Val::Px(8.0),
    right: Val::Px(8.0),
    top: Val::Px(4.0),
    bottom: Val::Px(4.0),
};

#[derive(Component)]
pub(crate) struct LoginSubmitButton;

#[derive(Component)]
pub(crate) struct LoginCommunityButton;

#[derive(Component)]
pub(super) struct LoginStyledButton;

#[derive(Component)]
pub(crate) struct LoginRegisterButton;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) enum LoginButtonTextRole {
    Submit,
    Community,
    Register,
}

pub(super) fn sliced_button_image(handle: Handle<Image>) -> ImageNode {
    ImageNode {
        image: handle,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: LOGIN_BUTTON_BORDER,
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

pub(super) fn handle_login_keyboard(
    browser: Res<LoginBrowser>,
    keys: Option<MessageReader<KeyboardInput>>,
    buttons: Option<Res<ButtonInput<KeyCode>>>,
    option_ui: Res<OptionUiModel>,
    mut model: ResMut<LoginUiModel>,
    mut outbox: ResMut<LoginUiOutbox>,
) {
    if browser.editing_server || option_ui.visible || !model.accepts_manual_input() {
        return;
    }
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if model.saved_account.is_some() {
            if key.state == ButtonState::Pressed && matches!(key.key_code, KeyCode::Enter | KeyCode::NumpadEnter) { queue_login(&mut model, &mut outbox); }
            continue;
        }
        if key.state != ButtonState::Pressed {
            continue;
        }
        let (control, shift) = text_edit::modifiers(buttons.as_deref());
        apply_login_edit_key(
            &mut model,
            &mut outbox,
            key.key_code,
            key.text.as_deref(),
            control,
            shift,
        );
    }
}

pub(super) fn handle_login_edit_pointer(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    fields: Query<
        (&Interaction, Has<LoginUsernameField>),
        Or<(With<LoginUsernameField>, With<LoginPasswordField>)>,
    >,
    texts: Query<
        (
            &bevy::text::ComputedTextBlock,
            &EditVisual,
            &ComputedNode,
            &UiGlobalTransform,
            &Text,
            Has<LoginUsernameText>,
        ),
        Or<(With<LoginUsernameText>, With<LoginPasswordText>)>,
    >,
    option_ui: Res<OptionUiModel>,
    mut model: ResMut<LoginUiModel>,
    mut dragging: Local<Option<LoginField>>,
) {
    let Some(mouse) = mouse else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused
        || !mouse.pressed(MouseButton::Left)
        || option_ui.visible
        || !model.accepts_manual_input()
    {
        *dragging = None;
        return;
    }
    let start = mouse.just_pressed(MouseButton::Left);
    if start {
        *dragging = fields
            .iter()
            .find(|(interaction, _)| **interaction == Interaction::Pressed)
            .map(|(_, username)| {
                if username {
                    LoginField::Username
                } else {
                    LoginField::Password
                }
            });
    }
    let Some(field) = *dragging else {
        return;
    };
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };
    let Some((block, visual, computed, transform, text, _)) = texts
        .iter()
        .find(|(_, _, _, _, _, username)| *username == (field == LoginField::Username))
    else {
        return;
    };
    let Some(position) =
        text_edit::hit_position(block, computed, transform, &text.0, visual, cursor, 0.0)
    else {
        return;
    };
    let (_, shift) = text_edit::modifiers(keys.as_deref());
    let model = &mut *model;
    let (edit, value) = match field {
        LoginField::Username => (&mut model.username_edit, &model.username),
        LoginField::Password => (&mut model.password_edit, &model.password),
    };
    edit.clamp(value);
    edit.place(position, !start || shift);
}

pub(super) fn handle_login_interactions(
    mut model: ResMut<LoginUiModel>,
    mut outbox: ResMut<LoginUiOutbox>,
    mut effects: ResMut<LoginUiEffectOutbox>,
    mut browser: ResMut<LoginBrowser>,
    option_ui: Res<OptionUiModel>,
    username: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<LoginUsernameField>,
            Without<LoginPasswordField>,
        ),
    >,
    password: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<LoginPasswordField>,
            Without<LoginUsernameField>,
        ),
    >,
    submit: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<LoginSubmitButton>,
            Without<LoginUsernameField>,
            Without<LoginPasswordField>,
        ),
    >,
    community: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<LoginCommunityButton>,
            Without<LoginRegisterButton>,
        ),
    >,
    register: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<LoginRegisterButton>,
            Without<LoginCommunityButton>,
        ),
    >,
) {
    if option_ui.visible
        || !model.visible
        || model.surface != LoginSurface::Manual
        || model.busy
        || model.system_popup_active
    {
        return;
    }
    if community
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        effects.effects.push_back(LoginUiEffect::OpenCommunity {
            url: LOGIN_COMMUNITY_URL,
        });
        return;
    }
    if username
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        browser.editing_server = false;
        model.focused = LoginField::Username;
    }
    if password
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        browser.editing_server = false;
        model.focused = LoginField::Password;
    }
    if submit
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        queue_login(&mut model, &mut outbox);
    }
    let register_pressed = register
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if register_pressed {
        effects
            .effects
            .push_back(LoginUiEffect::ShowRegistrationInstructions {
                message: LOGIN_REGISTRATION_INSTRUCTIONS,
            });
    }
}

pub(super) fn handle_login_language_interaction(
    buttons: Query<&Interaction, (Changed<Interaction>, With<LoginLanguageButton>)>,
    model: Res<LoginUiModel>,
    option_ui: Res<OptionUiModel>,
    localization: Option<Res<Localization>>,
    mut language: Option<ResMut<Language>>,
) {
    if !model.accepts_manual_input()
        || option_ui.visible
        || !buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let (Some(localization), Some(language)) = (localization.as_deref(), language.as_deref_mut())
    else {
        return;
    };
    let locales: Vec<_> = localization.locales().collect();
    if locales.is_empty() {
        return;
    }
    let current = locales
        .iter()
        .position(|locale| *locale == language.effective.as_str())
        .unwrap_or(locales.len() - 1);
    localization.select(language, locales[(current + 1) % locales.len()]);
}

pub(super) fn login_button_text_color(interaction: Interaction) -> Color {
    if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
        Color::WHITE
    } else {
        Color::srgb(0.9, 0.9, 0.9)
    }
}
