use super::*;

pub const RULE_UI_BACK_HOVER_PATH: &str = "ui/en/rule/back-hover.png";

pub const RULE_UI_NAV_HOVER_PATH: &str = "ui/en/rule/nav-hover.png";

pub const RULE_UI_CLOSE_HOVER_PATH: &str = "ui/en/rule/close-over.png";

pub const RULE_UI_BUTTON_NORMAL_TEXT_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];

pub const RULE_UI_BUTTON_HOVER_TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuleUiInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub cursor_locked_while_visible: bool,
    pub cursor_locked_after_exit: bool,
    pub mouse_controls_enabled: bool,
    pub escape_close_gate_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleUiButtonKind {
    Close,
    Back,
    Previous,
    Next,
}

#[must_use]
pub fn activate_rule_ui_button(
    kind: RuleUiButtonKind,
    model: &mut RuleUiModel,
    outbox: &mut RuleUiOutbox,
    audio: &mut RuleUiAudioOutbox,
) -> bool {
    if !model.controls_enabled() {
        return false;
    }

    match kind {
        RuleUiButtonKind::Close => {
            exit_rule_ui(model, outbox, RuleUiDismissalSource::CloseButton);
        }
        RuleUiButtonKind::Back => {
            exit_rule_ui(model, outbox, RuleUiDismissalSource::BackButton);
        }
        RuleUiButtonKind::Previous | RuleUiButtonKind::Next => {
            let Some(page) = model.page() else {
                return false;
            };
            let destination = match kind {
                RuleUiButtonKind::Previous => page.previous,
                RuleUiButtonKind::Next => page.next,
                RuleUiButtonKind::Close | RuleUiButtonKind::Back => unreachable!(),
            };
            let Some(destination) = destination else {
                return false;
            };
            model.current_page = Some(destination);
        }
    }

    audio.push(RuleUiAudioCue::ButtonSound);
    true
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiButton {
    pub kind: RuleUiButtonKind,
}

pub(super) fn handle_rule_ui_keyboard(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<RuleUiModel>,
    mut outbox: ResMut<RuleUiOutbox>,
) {
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state == ButtonState::Pressed
            && !key.repeat
            && key.key_code == KeyCode::Escape
            && request_rule_ui_escape_close(&mut model, &mut outbox)
        {
            break;
        }
    }
}

pub(super) fn handle_rule_ui_interactions(
    buttons: Query<(&Interaction, &RuleUiButton), Changed<Interaction>>,
    mut model: ResMut<RuleUiModel>,
    mut outbox: ResMut<RuleUiOutbox>,
    mut audio: ResMut<RuleUiAudioOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed
            && activate_rule_ui_button(button.kind, &mut model, &mut outbox, &mut audio)
        {
            break;
        }
    }
}

pub(super) fn sync_rule_ui_button_visuals(
    model: Res<RuleUiModel>,
    assets: Res<RuleUiAssets>,
    mut buttons: Query<(
        &Interaction,
        &RuleUiButton,
        &Children,
        &mut ImageNode,
        Option<&mut Pickable>,
    )>,
    mut labels: Query<&mut TextColor, With<RuleUiTextElement>>,
) {
    let enabled = model.controls_enabled();
    for (interaction, button, children, mut image, pickable) in &mut buttons {
        let interaction = if enabled {
            *interaction
        } else {
            Interaction::None
        };
        image.image = match (button.kind, interaction) {
            (RuleUiButtonKind::Close, Interaction::Hovered) => assets.close_hover.clone(),
            (RuleUiButtonKind::Close, Interaction::None | Interaction::Pressed) => {
                assets.close_normal.clone()
            }
            (RuleUiButtonKind::Back, Interaction::Hovered) => assets.back_hover.clone(),
            (RuleUiButtonKind::Back, Interaction::None | Interaction::Pressed) => {
                assets.back_normal.clone()
            }
            (RuleUiButtonKind::Previous | RuleUiButtonKind::Next, Interaction::Hovered) => {
                assets.nav_hover.clone()
            }
            (
                RuleUiButtonKind::Previous | RuleUiButtonKind::Next,
                Interaction::None | Interaction::Pressed,
            ) => assets.nav_normal.clone(),
        };
        image.color = Color::srgba(
            1.0,
            1.0,
            1.0,
            if enabled { 1.0 } else { RULE_UI_DISABLED_ALPHA },
        );
        if let Some(mut pickable) = pickable {
            *pickable = if enabled {
                Pickable::default()
            } else {
                Pickable::IGNORE
            };
        }

        let base_color = match (button.kind, interaction) {
            (RuleUiButtonKind::Back, Interaction::None | Interaction::Pressed) => {
                RULE_UI_BACK_NORMAL_TEXT_COLOR
            }
            (_, Interaction::Hovered) => RULE_UI_BUTTON_HOVER_TEXT_COLOR,
            _ => RULE_UI_BUTTON_NORMAL_TEXT_COLOR,
        };
        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child) {
                let mut value = base_color;
                if !enabled {
                    value[3] *= RULE_UI_DISABLED_ALPHA;
                }
                color.0 = rgba(value);
            }
        }
    }
}
