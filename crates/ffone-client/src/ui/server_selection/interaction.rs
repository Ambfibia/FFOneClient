use super::*;

pub const SERVER_SELECTION_ROW_HOVER_PATH: &str = "ui/en/server-selection/SSButtonOver.png";

pub const SERVER_SELECTION_BUTTON_NORMAL_PATH: &str = "ui/en/server-selection/ff-button-normal.png";

pub const SERVER_SELECTION_BUTTON_HOVER_PATH: &str = "ui/en/server-selection/ff-button-hover.png";

pub const SERVER_SELECTION_BUTTON_ACTIVE_PATH: &str = "ui/en/server-selection/ff-button-active.png";

pub const SERVER_SELECTION_RED_HOVER_PATH: &str = "ui/en/server-selection/red_button_over.png";

pub const SERVER_SELECTION_SCROLL_TRACK_PATH: &str = "ui/en/server-selection/scroll_bar.png";

pub const SERVER_SELECTION_SCROLL_THUMB_PATH: &str = "ui/en/server-selection/scroll_Thumb.png";

pub const SERVER_SELECTION_SCROLL_UP_PATH: &str = "ui/en/server-selection/scroll_up.png";

pub const SERVER_SELECTION_SCROLL_DOWN_PATH: &str = "ui/en/server-selection/scroll_dn.png";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerSelectionKeyboardContract {
    pub named_control_count: usize,
    pub accepts_text_input: bool,
    pub tab_navigation: bool,
    pub escape_handler: bool,
}

/// Neither clean owner creates a named IMGUI control or inspects keyboard
/// focus/Tab/Escape. `SetFocusOUt(true)` belongs solely to accepted quit.
pub const SERVER_SELECTION_KEYBOARD_CONTRACT: ServerSelectionKeyboardContract =
    ServerSelectionKeyboardContract {
        named_control_count: 0,
        accepts_text_input: false,
        tab_navigation: false,
        escape_handler: false,
    };

pub const SERVER_SELECTION_SCROLL_STEP: f32 = 18.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerSelectionInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub escape_enabled: bool,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) enum LegacyButtonKind {
    Blue,
    Red,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct LegacyButtonLabel;

pub(super) fn centered_button_node(rect: ServerSelectionUiRect, kind: LegacyButtonKind) -> Node {
    let mut node = rect.node();
    node.align_items = AlignItems::Center;
    node.justify_content = JustifyContent::Center;
    if kind == LegacyButtonKind::Blue {
        node.padding = UiRect::new(px(10), px(6), px(3), px(6));
    }
    node
}

pub(super) fn legacy_button_text_color(kind: LegacyButtonKind, interaction: Interaction) -> Color {
    match (kind, interaction) {
        (LegacyButtonKind::Blue, Interaction::None) => Color::srgb(0.9, 0.9, 0.9),
        (LegacyButtonKind::Red, Interaction::Pressed) => Color::BLACK,
        _ => Color::WHITE,
    }
}

pub(super) fn collect_server_selection_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut model: ResMut<ServerSelectionUiModel>,
) {
    if keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape))
    {
        let _ = model.press_escape();
    }
    if model.phase != ServerSelectionPhase::Visible {
        return;
    }
    let axis = mouse_scroll.as_ref().map_or(0.0, |scroll| scroll.delta.y);
    if !axis.is_finite() || axis == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let layout = ServerSelectionUiLayout::from_viewport(
        Vec2::new(window.width(), window.height()),
        model.scroll_y,
        model.server_expanded,
    );
    if SERVER_SELECTION_VIEWPORT_RECT
        .translated(layout.panel.left, layout.panel.top)
        .contains(cursor)
    {
        model.scroll_by(-axis * SERVER_SELECTION_SCROLL_STEP);
    }
}

pub(super) fn bind_server_selection_button_images(
    model: Res<ServerSelectionUiModel>,
    assets: Res<ServerSelectionUiAssets>,
    mut controls: Query<(
        &ServerSelectionUiControl,
        &Interaction,
        Option<&LegacyButtonKind>,
        &mut ImageNode,
    )>,
    parents: Query<&ChildOf>,
    button_states: Query<(&Interaction, &LegacyButtonKind)>,
    mut labels: Query<(Entity, &mut TextColor), With<LegacyButtonLabel>>,
) {
    for (control, interaction, kind, mut image) in &mut controls {
        match *control {
            ServerSelectionUiControl::ServerToggle => {
                image.image = assets.row_hover.clone();
                image.color = if matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                {
                    Color::WHITE
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.0)
                };
            }
            ServerSelectionUiControl::Shard(shard) => {
                if model.selected_server == 1 && model.selected_shard == i32::from(shard) {
                    // `SSButtonSelected` is drawn after the button in clean
                    // OnGUI, so it covers even the hover texture.
                    image.image = assets.row_selected.clone();
                    image.color = Color::WHITE;
                } else {
                    image.image = assets.row_hover.clone();
                    image.color =
                        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
                            Color::WHITE
                        } else {
                            Color::srgba(1.0, 1.0, 1.0, 0.0)
                        };
                }
            }
            ServerSelectionUiControl::Connect
            | ServerSelectionUiControl::MyAccount
            | ServerSelectionUiControl::Homepage
            | ServerSelectionUiControl::Quit => match kind.copied() {
                Some(LegacyButtonKind::Blue) => {
                    image.image = match interaction {
                        Interaction::Pressed => assets.button_active.clone(),
                        Interaction::Hovered => assets.button_hover.clone(),
                        Interaction::None => assets.button_normal.clone(),
                    };
                    image.color = Color::WHITE;
                }
                Some(LegacyButtonKind::Red) => {
                    image.image = if *interaction == Interaction::Hovered {
                        assets.red_hover.clone()
                    } else {
                        // Clean RedButton active texture is the normal texture.
                        assets.red_normal.clone()
                    };
                    image.color = Color::WHITE;
                }
                None => {}
            },
            ServerSelectionUiControl::ScrollUp | ServerSelectionUiControl::ScrollDown => {}
        }
    }
    for (entity, mut color) in &mut labels {
        let Some(parent) = parents.get(entity).ok().map(ChildOf::parent) else {
            continue;
        };
        let Ok((interaction, kind)) = button_states.get(parent) else {
            continue;
        };
        color.0 = legacy_button_text_color(*kind, *interaction);
    }
}
