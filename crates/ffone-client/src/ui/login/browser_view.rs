use super::*;
use super::browser::{LoginBrowser, ServerHealth};
use super::layout::login_ui_scale;
use crate::localization::UiTextAutoFit;

const WIDTH: f32 = 280.0;
const PADDING: f32 = 14.0;

#[derive(Component)]
pub(super) struct BrowserPanel;
#[derive(Component)]
pub(super) struct ServerInputText;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Screen { #[default] Servers, AddServer }
#[derive(Default, Resource)]
pub(super) struct BrowserPopover { screen: Screen }
#[derive(Component, Clone)]
pub(super) enum Action {
    Server(usize), Refresh, AddServer, ServerInput, ServersPage, OpenAddServer, Back,
}

pub(super) fn text(parent: &mut ChildSpawnerCommands, value: LocalizedText, assets: &LoginUiAssets, width: f32, color: Color) {
    let style = LoginTextStyle0104::Button;
    parent.spawn((Text::new(value.fallback.clone()), value, style.font(assets), TextColor(color),
        TextLayout::new(Justify::Left, LineBreak::NoWrap),
        UiTextAutoFit::new(width, 22.0, &style.font(assets)), bevy::ui::FocusPolicy::Pass));
}
pub(super) fn field_image(assets: &LoginUiAssets) -> ImageNode {
    ImageNode { image: assets.text_field.clone(), visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer { border: LOGIN_TEXT_FIELD_BORDER,
            center_scale_mode: SliceScaleMode::Stretch, sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 1.0 }), ..default() }
}
fn button(parent: &mut ChildSpawnerCommands, value: LocalizedText, assets: &LoginUiAssets, action: Action, width: Val) {
    let content_width = match width { Val::Px(w) => w - 16.0, _ => WIDTH - PADDING * 2.0 - 16.0 };
    let submit = matches!(action, Action::AddServer);
    parent.spawn((Button, Node { width, height: px(if submit {35.0} else {30.0}), flex_shrink: 0.0,
        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
        padding: UiRect::horizontal(px(8)), ..default() },
        if submit {sliced_button_image(assets.button.clone())} else {field_image(assets)}, action))
        .with_children(|p| text(p, value, assets, content_width,
            if submit {login_button_text_color(Interaction::None)} else {Color::srgb(0.78, 0.9, 0.95)}));
}
fn row(parent: &mut ChildSpawnerCommands, assets: &LoginUiAssets, index: usize, name: &str, health: &ServerHealth, selected: bool) {
    parent.spawn((Button, Node { height: px(36), width: percent(100), padding: UiRect::horizontal(px(10)),
        align_items: AlignItems::Center, column_gap: px(10), flex_shrink: 0.0, ..default() },
        field_image(assets), Action::Server(index)))
        .with_children(|p| {
            let color = match health { ServerHealth::Online(_) => Color::srgb(0.7, 1.0, 0.05),
                ServerHealth::Checking => Color::srgb(1.0, 0.78, 0.0), ServerHealth::Offline => Color::srgb(1.0, 0.25, 0.42) };
            p.spawn((Node { width: px(5), height: px(16), flex_shrink: 0.0, ..default() }, BackgroundColor(color)));
            p.spawn(Node { flex_grow: 1.0, min_width: px(0), overflow: Overflow::clip(), ..default() })
                .with_children(|p| text(p, LocalizedText::new("ui.login.server_name", "{server}").with_arg("server", name),
                    assets, 170.0, if selected { Color::WHITE } else { Color::srgb(0.7, 0.78, 0.82) }));
            if let ServerHealth::Online(Some(count)) = health {
                text(p, LocalizedText::new("ui.login.server_count", "{count}").with_arg("count", count.to_string()),
                    assets, 36.0, Color::srgb(0.7, 1.0, 0.05));
            }
        });
}
fn server_name(address: &str) -> &str {
    match address { "localhost:23000" => "localhost", "api.slavicfall.ru" => "SlavicFall", _ => address }
}
fn header(parent: &mut ChildSpawnerCommands, assets: &LoginUiAssets, screen: Screen) {
    parent.spawn(Node { height: px(26), width: percent(100), align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween, flex_shrink: 0.0, ..default() }).with_children(|p| {
        let title = match screen { Screen::Servers => LocalizedText::new("ui.login.servers", "SERVERS"),
            Screen::AddServer => LocalizedText::new("ui.login.add_server", "Add server") };
        text(p, title, assets, 170.0, Color::srgb(0.7, 0.78, 0.82));
        p.spawn(Node { column_gap: px(6), ..default() }).with_children(|p| {
            if screen == Screen::Servers {
                p.spawn((Button, Node { width: px(30), height: px(30), align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center, ..default() }, field_image(assets), Action::Refresh))
                    .with_children(|p| symbols::spawn(p, symbols::Icon::Refresh));
            }
            button(p, if screen == Screen::Servers { LocalizedText::new("ui.login.server_add_icon", "+") }
                else { LocalizedText::new("ui.login.browser_back", "<") }, assets,
                if screen == Screen::Servers { Action::OpenAddServer } else { Action::Back }, px(30));
        });
    });
}
pub(super) fn spawn(parent: &mut ChildSpawnerCommands, assets: &LoginUiAssets) {
    parent.spawn((Node { position_type: PositionType::Absolute, width: px(WIDTH), height: px(168),
        padding: UiRect::all(px(PADDING)), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() },
        ImageNode { image: assets.panel.clone(), visual_box: bevy::ui::VisualBox::BorderBox,
            image_mode: NodeImageMode::Sliced(TextureSlicer { border: LOGIN_PANEL_BORDER,
                center_scale_mode: SliceScaleMode::Stretch, sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 1.0 }), ..default() },
        UiTransform::default(), ZIndex(1), BrowserPanel));
}
pub(super) fn bind_browser(
    mut commands: Commands, mut browser: ResMut<LoginBrowser>, popover: Res<BrowserPopover>,
    model: Res<LoginUiModel>, assets: Res<LoginUiAssets>, windows: Query<&Window, With<PrimaryWindow>>,
    option_ui: Res<OptionUiModel>, mut panels: Query<(Entity, &mut Node, &mut UiTransform), With<BrowserPanel>>,
    mut fingerprint: Local<String>,
) {
    if model.visible { browser.poll(); }
    let Ok(window) = windows.single() else { return; };
    let Ok((entity, mut node, mut transform)) = panels.single_mut() else { return; };
    node.display = if model.visible && model.surface == LoginSurface::Manual { Display::Flex } else { Display::None };
    let scale = login_ui_scale(window.height(), option_ui.persisted_options.display.scale_ui,
        window.resolution.scale_factor_override().is_some());
    let left = window.width() / 2.0 + LOGIN_PANEL_SIZE.x * scale / 2.0 + 16.0;
    let count = match popover.screen {
        Screen::Servers => 1 + browser.servers.len().min(3) + usize::from(browser.servers.len() > 3),
        Screen::AddServer => 3,
    };
    let height = 28.0 + count as f32 * 36.0 + if browser.message.is_empty() { 0.0 } else { 44.0 };
    let fit = ((window.width() - left - 16.0) / WIDTH).min((window.height()-24.0)/height).min(scale).max(0.1);
    node.height = px(height);
    transform.scale = Vec2::splat(fit);
    node.left = px(left + WIDTH / 2.0 * (fit - 1.0));
    let panel_height = LOGIN_PANEL_SIZE.y + if model.visible {24.0} else {0.0};
    let top = (window.height() - panel_height * scale) / 2.0;
    node.top = px(top + height / 2.0 * (fit - 1.0));
    let current = format!("{:?}|{:?}|{:?}|{}|{}|{}|{}|{}|{}", browser.servers, browser.health, popover.screen,
        browser.selected, browser.server_page, browser.account_page, browser.remember,
        browser.message, browser.selected_accounts().iter().map(|a| a.username.as_str()).collect::<Vec<_>>().join("|"));
    if *fingerprint == current { return; }
    *fingerprint = current;
    commands.entity(entity).despawn_children();
    commands.entity(entity).with_children(|p| {
        header(p, &assets, popover.screen);
        match popover.screen {
            Screen::Servers => {
                for (index, server) in browser.servers.iter().enumerate().skip(browser.server_page*3).take(3) {
                    row(p, &assets, index, server_name(&server.address), &browser.health[index], index == browser.selected);
                }
                if browser.servers.len() > 3 { button(p, LocalizedText::new("ui.login.more_servers", "More servers"), &assets, Action::ServersPage, percent(100)); }
            }
            Screen::AddServer => {
                let mut node = Node { width: percent(100), height: px(25), flex_shrink: 0.0, ..default() };
                LoginTextStyle0104::TextField.apply_to_control(&mut node);
                node.overflow = Overflow::clip();
                p.spawn((Button, node, field_image(&assets), Action::ServerInput)).with_children(|p| {
                    super::view::spawn_login_text(p, None, LocalizedText::new("ui.login.server_input", "{address}")
                        .with_arg("address", &browser.server_input), &assets, LoginTextStyle0104::TextField,
                        Color::WHITE, ServerInputText);
                });
                button(p, LocalizedText::new("ui.login.add_server", "Add server"), &assets, Action::AddServer, percent(100));
            }

        }
        if !browser.message.is_empty() {
            text(p, LocalizedText::new("ui.login.profile_error", "Account settings: {error}").with_arg("error", &browser.message),
                &assets, WIDTH-2.0*PADDING, Color::srgb(1.0, 0.6, 0.5));
        }
    });
}
pub(super) fn interactions(
    buttons: Query<(&Interaction, &Action), Changed<Interaction>>, mut browser: ResMut<LoginBrowser>,
    mut popover: ResMut<BrowserPopover>, mut model: ResMut<LoginUiModel>, option_ui: Res<OptionUiModel>,
) {
    if !model.accepts_manual_input() || option_ui.visible { return; }
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed { continue; }
        match action {
            Action::Refresh => browser.refresh(),
            Action::Server(index) => { if *index != browser.selected { browser.select(*index); model.password.clear(); model.username.clear(); model.saved_account = None; model.status.clear(); } }
            Action::AddServer => { browser.add_server(); if browser.message.is_empty() { popover.screen = Screen::Servers; model.username.clear(); model.password.clear(); } }
            Action::ServerInput => browser.editing_server = true,
            Action::ServersPage => browser.server_page = (browser.server_page+1) % browser.servers.len().div_ceil(3),
            Action::OpenAddServer => { popover.screen = Screen::AddServer; browser.editing_server = true; }
            Action::Back => { popover.screen = Screen::Servers; browser.editing_server = false; browser.message.clear(); }
        }
    }
}
pub(super) fn button_states(
    mut buttons: Query<(&Interaction, &Action, &mut ImageNode, Option<&Children>)>, assets: Res<LoginUiAssets>, browser: Res<LoginBrowser>,
    mut labels: Query<&mut TextColor>,
) {
    for (interaction, action, mut image, children) in &mut buttons {
        if matches!(action, Action::ServerInput) {
            if image.image != assets.text_field { *image = field_image(&assets); }
            continue;
        }
        if matches!(action, Action::AddServer) {
            if let Some(children) = children { for child in children {
                if let Ok(mut color) = labels.get_mut(*child) { color.0 = login_button_text_color(*interaction); }
            }}
        }
        let selected = matches!(action, Action::Server(index) if *index == browser.selected);
        let handle = match interaction {
            Interaction::Pressed | Interaction::Hovered if matches!(action, Action::Refresh) => &assets.button,
            Interaction::Pressed => &assets.button_active,
            Interaction::Hovered => &assets.button_over,
            Interaction::None if selected || matches!(action, Action::AddServer) => &assets.button,
            Interaction::None => &assets.text_field,
        };
        if image.image != *handle {
            *image = if *handle == assets.text_field { field_image(&assets) }
                else { sliced_button_image(handle.clone()) };
        }
    }
}

/// Update the editable address without recreating its caret after each keystroke.
pub(super) fn bind_server_input(browser: Res<LoginBrowser>, model: Res<LoginUiModel>, options: Res<OptionUiModel>,
    mut texts: Query<(&mut LocalizedText, &mut EditVisual, &mut TextColor), With<ServerInputText>>) {
    for (mut text, mut visual, mut color) in &mut texts {
        let active = browser.editing_server && model.accepts_manual_input() && !options.visible;
        let placeholder = browser.server_input.is_empty() && !active;
        let value = LocalizedText::new("ui.login.server_input", "{address}")
            .with_arg("address", if placeholder {"host:23000 / https://host"} else {&browser.server_input});
        if *text != value { *text = value; }
        visual.active = active;
        if visual.edit != browser.server_edit { visual.edit = browser.server_edit.clone(); }
        color.0 = if placeholder {Color::srgb(0.5, 0.65, 0.7)} else {Color::WHITE};
    }
}

pub(super) fn server_input_pointer(
    windows: Query<&Window, With<PrimaryWindow>>, mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>, options: Res<OptionUiModel>, model: Res<LoginUiModel>,
    fields: Query<(&Interaction, &Action)>,
    texts: Query<(&bevy::text::ComputedTextBlock, &EditVisual, &ComputedNode, &UiGlobalTransform, &Text), With<ServerInputText>>,
    mut browser: ResMut<LoginBrowser>, mut dragging: Local<bool>) {
    let (Some(mouse), Ok(window)) = (mouse, windows.single()) else {return;};
    if !window.focused || !mouse.pressed(MouseButton::Left) || options.visible || !model.accepts_manual_input() {
        *dragging = false; return;
    }
    let start = mouse.just_pressed(MouseButton::Left);
    if start { *dragging = fields.iter().any(|(i,a)| *i == Interaction::Pressed && matches!(a, Action::ServerInput)); }
    if !*dragging {return;}
    let (Some(cursor), Ok((block, visual, computed, transform, text))) = (window.physical_cursor_position(), texts.single()) else {return;};
    if let Some(position) = text_edit::hit_position(block, computed, transform, &text.0, visual, cursor, 0.0) {
        let (_, shift) = text_edit::modifiers(keys.as_deref());
        let browser = &mut *browser;
        browser.server_edit.clamp(&browser.server_input);
        browser.server_edit.place(position, !start || shift);
    }
}
pub(super) fn keyboard(
    mut keys: MessageReader<KeyboardInput>, buttons: Option<Res<ButtonInput<KeyCode>>>, mut browser: ResMut<LoginBrowser>,
    mut popover: ResMut<BrowserPopover>, mut model: ResMut<LoginUiModel>, option_ui: Res<OptionUiModel>,
) {
    if !browser.editing_server || !model.accepts_manual_input() || option_ui.visible { return; }
    for key in keys.read().filter(|k| k.state == ButtonState::Pressed) {
        match key.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => { browser.add_server(); if browser.message.is_empty() { popover.screen = Screen::Servers; model.username.clear(); model.password.clear(); } }
            KeyCode::Escape => { browser.editing_server = false; popover.screen = Screen::Servers; }
            KeyCode::Tab => browser.editing_server = false,
            _ => {
                let (control, shift) = text_edit::modifiers(buttons.as_deref());
                let browser = &mut *browser;
                if !browser.server_edit.key(&mut browser.server_input, key.key_code, control, shift) && !control {
                    if let Some(text) = &key.text { browser.server_edit.insert(&mut browser.server_input, text, 240, false); }
                }
            }
        }
    }
}
