use super::*;
use super::browser_view::{field_image, text};

const SELECTOR_WIDTH: f32 = 249.0;
#[derive(Component)]
pub(super) struct CredentialLabel;
#[derive(Component)]
pub(super) struct AccountControls;
#[derive(Component)]
pub(super) struct AccountMenu;
#[derive(Resource, Default)]
pub(super) struct AccountForm {
    server: Option<String>,
    return_account: Option<String>,
}
#[derive(Component)]
pub(super) enum Action { New, Back, Select(String), Edit(String), Remember }

pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    parent.spawn((Node { position_type: PositionType::Absolute, left: px(45), top: px(15),
        width: px(280), height: px(138), ..default() }, AccountControls, bevy::ui::FocusPolicy::Pass, ZIndex(5)));
}

fn button(parent: &mut ChildSpawnerCommands, assets: &LoginUiAssets, value: LocalizedText,
    action: Action, width: f32, height: f32) {
    parent.spawn((Button, Node { width: px(width), height: px(height), flex_shrink: 0.0,
        align_items: AlignItems::Center, justify_content: JustifyContent::Center,
        padding: UiRect::horizontal(px(5)), ..default() }, field_image(assets), action))
        .with_children(|p| text(p, value, assets, width-10.0, Color::WHITE));
}
fn icon_button(parent: &mut ChildSpawnerCommands, assets: &LoginUiAssets, action: Action,
    icon: symbols::Icon, height: f32) {
    parent.spawn((Button, Node { width: px(25), height: px(height), flex_shrink: 0.0,
        align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
        field_image(assets), action)).with_children(|p| symbols::spawn(p, icon));
}
pub(super) fn bind(
    mut commands: Commands, browser: Res<LoginBrowser>, mut model: ResMut<LoginUiModel>,
    mut form: ResMut<AccountForm>, assets: Res<LoginUiAssets>,
    roots: Query<Entity, With<AccountControls>>,
    mut credentials: Query<(&mut Node, Has<LoginUsernameField>),
        Or<(With<CredentialLabel>, With<LoginUsernameField>, With<LoginPasswordField>)>>,
    mut fingerprint: Local<String>,
) {
    let server = browser.servers[browser.selected].key().to_owned();
    let accounts = browser.selected_accounts();
    if form.server.as_ref() != Some(&server) {
        form.server = Some(server);
        form.return_account = None;
        model.saved_account = accounts.first().map(|a| a.username.clone());
        if let Some(username) = model.saved_account.clone() { model.username = username; model.password.clear(); }
    }
    let saved = model.saved_account.is_some();
    for (mut node, username_field) in &mut credentials {
        node.display = if saved { Display::None } else { Display::Flex };
        if username_field { node.width = px(if accounts.is_empty() {280.0} else {SELECTOR_WIDTH}); }
    }
    let current = format!("{}|{:?}|{}|{:?}|{}", saved, form.server, browser.remember, accounts.iter().map(|a| &a.username).collect::<Vec<_>>(), model.visible);
    if *fingerprint == current { return; }
    *fingerprint = current;
    let Ok(root) = roots.single() else { return; };
    commands.entity(root).despawn_children();
    commands.entity(root).with_children(|p| {
        if saved {
            text(p, LocalizedText::new("ui.login.account_caption", "ACCOUNT"), &assets, 200.0, Color::WHITE);
            p.spawn(Node { position_type: PositionType::Absolute, left: px(255), top: px(25), ..default() })
                .with_children(|p| button(p, &assets, LocalizedText::new("ui.login.server_add_icon", "+"), Action::New, 25.0, 25.0));
            p.spawn((Node { position_type: PositionType::Absolute, top: px(25), width: px(SELECTOR_WIDTH),
                height: px(85), padding: UiRect::all(px(3)), ..default() }, field_image(&assets)))
                .with_children(|p| account_scroll::spawn(p, &assets, |p| {
                    for account in &accounts {
                        p.spawn(Node { width: percent(100), height: px(25), min_height: px(25), flex_shrink: 0.0,
                            column_gap: px(4), ..default() }).with_children(|p| {
                            button(p, &assets, LocalizedText::new("ui.login.saved_account", "{username}")
                                .with_arg("username", &account.username), Action::Select(account.username.clone()), 193.0, 25.0);
                            icon_button(p, &assets, Action::Edit(account.username.clone()), symbols::Icon::Pencil, 25.0);
                        });
                    }
                }));
        } else {
            if !accounts.is_empty() {
                p.spawn((Node { position_type: PositionType::Absolute, left: px(255), top: px(25), ..default() },
                    bevy::ui::FocusPolicy::Pass)).with_children(|p| icon_button(p, &assets, Action::Back, symbols::Icon::Back, 25.0));
            }
            p.spawn((Button, Node { position_type: PositionType::Absolute, right: px(0), top: px(118),
                width: px(128), height: px(20), align_items: AlignItems::Center, column_gap: px(5), ..default() }, Action::Remember))
                .with_children(|p| {
                    p.spawn((Node { width: px(17), height: px(17), flex_shrink: 0.0, ..default() },
                        ImageNode::new(if browser.remember {assets.check_checked.clone()} else {assets.check_empty.clone()}),
                        bevy::ui::FocusPolicy::Pass));
                    text(p, LocalizedText::new("ui.login.remember_caption", "Remember"), &assets, 106.0, Color::WHITE);
                });
        }

    });
}

pub(super) fn interactions(
    buttons: Query<(&Interaction, &Action), Changed<Interaction>>, mut model: ResMut<LoginUiModel>,
    mut browser: ResMut<LoginBrowser>, mut form: ResMut<AccountForm>, options: Res<OptionUiModel>,
) {
    if !model.accepts_manual_input() || options.visible { return; }
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed { continue; }
        browser.editing_server = false;
        match action {
            Action::Select(username) => {
                model.username.clone_from(username); model.password.clear();
                model.saved_account = Some(username.clone()); model.status.clear();
            }
            Action::New | Action::Edit(_) => {
                form.return_account = model.saved_account.clone();
                model.username = if let Action::Edit(username) = action { username.clone() } else { String::new() };
                model.password.clear(); model.saved_account = None; model.status.clear();
                model.focused = if model.username.is_empty() { LoginField::Username } else { LoginField::Password };
                browser.remember = true;
            }
            Action::Back => {
                model.saved_account = form.return_account.take().or_else(|| browser.selected_accounts().first().map(|a|a.username.clone()));
                model.username = model.saved_account.clone().unwrap_or_default(); model.password.clear();
                model.status.clear();
            }
            Action::Remember => browser.remember = !browser.remember,
        }
    }
}

pub(super) fn button_states(mut buttons: Query<(&Interaction, &Action, &mut ImageNode)>, assets: Res<LoginUiAssets>,
    model: Res<LoginUiModel>) {
    for (interaction, action, mut image) in &mut buttons {
        let selected = matches!(action, Action::Select(name) if model.saved_account.as_ref() == Some(name));
        let handle = match interaction {
            _ if selected => &assets.button_active,
            Interaction::Pressed | Interaction::Hovered if matches!(action, Action::Edit(_) | Action::Back) => &assets.button,
            Interaction::Pressed => &assets.button_active,
            Interaction::Hovered => &assets.button, Interaction::None => &assets.text_field };
        if image.image != *handle { *image = if *handle == assets.text_field { field_image(&assets) }
            else { sliced_button_image(handle.clone()) }; }
    }
}
