use super::*;

pub const QUIT_MENU_BUTTON_FONT_PATH_ID: i64 = 903;

pub const QUIT_MENU_BUTTON_NORMAL_PATH: &str = "ui/en/gameplay/quit-menu/quitbutnor.png";

pub const QUIT_MENU_BUTTON_HOVER_PATH: &str = "ui/en/gameplay/quit-menu/quitbutup.png";

pub const QUIT_MENU_CANCEL_HOVER_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const QUIT_MENU_BUTTON_VERTICAL_GAP: f32 = 8.0;

pub const QUIT_MENU_BUTTON_NORMAL_TEXT_COLOR: [f32; 4] = [0.770_161_3, 1.0, 1.0, 1.0];

pub const QUIT_MENU_BUTTON_HOVER_TEXT_COLOR: [f32; 4] = [0.762_096_76, 1.0, 1.0, 1.0];

pub const QUIT_MENU_CANCEL_HOVER_TEXT_COLOR: [f32; 4] =
    [0.978_271_1, 0.978_271_1, 0.978_271_1, 1.0];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum QuitMenuButtonKind {
    ChangeCharacter = 0,
    QuitGame = 1,
    QuitAndLogout = 2,
    Cancel = 3,
}

impl QuitMenuButtonKind {
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuitMenuButtonVisual {
    Standard,
    Cancel,
}

impl QuitMenuButtonVisual {
    #[must_use]
    pub const fn text_style(self) -> QuitMenuTextStyleSpec {
        match self {
            Self::Standard => QuitMenuTextStyleSpec {
                source_style: "Button",
                source_font_path_id: QUIT_MENU_BUTTON_FONT_PATH_ID,
                font_size: QUIT_MENU_FONT_SIZE,
                line_height: QUIT_MENU_FONT_LINE_HEIGHT,
                padding: [6.0, 6.0, 3.0, 3.0],
                word_wrap: true,
                clips_text: true,
                y_offset: QUIT_MENU_TEXT_Y_OFFSET,
            },
            Self::Cancel => QuitMenuTextStyleSpec {
                source_style: "CancelButton",
                source_font_path_id: QUIT_MENU_BUTTON_FONT_PATH_ID,
                font_size: QUIT_MENU_FONT_SIZE,
                line_height: QUIT_MENU_FONT_LINE_HEIGHT,
                padding: [0.0; 4],
                word_wrap: false,
                clips_text: true,
                y_offset: QUIT_MENU_TEXT_Y_OFFSET,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuitMenuButtonSpec {
    pub kind: QuitMenuButtonKind,
    pub localization_key: &'static str,
    pub label: &'static str,
    pub visual: QuitMenuButtonVisual,
    pub rect: QuitMenuUiRect,
}

pub const QUIT_MENU_BUTTONS: [QuitMenuButtonSpec; 3] = [
    QuitMenuButtonSpec {
        kind: QuitMenuButtonKind::ChangeCharacter,
        localization_key: "ui.quit_menu.change_character",
        label: "CHANGE CHARACTER",
        visual: QuitMenuButtonVisual::Standard,
        rect: QUIT_MENU_BUTTON_RECT,
    },
    QuitMenuButtonSpec {
        kind: QuitMenuButtonKind::QuitGame,
        localization_key: "ui.quit_menu.quit_game",
        label: "QUIT GAME",
        visual: QuitMenuButtonVisual::Standard,
        rect: QuitMenuUiRect::new(13.0, 69.0, 175.0, 45.0),
    },
    QuitMenuButtonSpec {
        kind: QuitMenuButtonKind::Cancel,
        localization_key: "ui.common.cancel",
        label: "CANCEL",
        visual: QuitMenuButtonVisual::Cancel,
        rect: QuitMenuUiRect::new(13.0, 122.0, 175.0, 45.0),
    },
];

/// Passive boundary information consumed by the owning gameplay shell.
///
/// The plugin blocks lower pointer targets through its root `Pickable`, but it
/// deliberately does not mutate `CursorOptions`, movement resources, or global
/// keyboard state. The shell uses this snapshot to unlock/show the cursor and
/// gate gameplay in the same frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QuitMenuInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub mouse_controls_enabled: bool,
    pub escape_dismiss_enabled: bool,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct QuitMenuButton {
    pub kind: QuitMenuButtonKind,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct QuitMenuButtonLabel {
    pub(super) kind: QuitMenuButtonKind,
}

pub(super) fn handle_quit_menu_keyboard(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<QuitMenuUiModel>,
    mut outbox: ResMut<QuitMenuUiOutbox>,
) {
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state == ButtonState::Pressed
            && !key.repeat
            && key.key_code == KeyCode::Escape
            && dismiss_quit_menu_with_escape(&mut model, &mut outbox)
        {
            break;
        }
    }
}

pub(super) fn handle_quit_menu_interactions(
    buttons: Query<(&Interaction, &QuitMenuButton), Changed<Interaction>>,
    mut model: ResMut<QuitMenuUiModel>,
    mut outbox: ResMut<QuitMenuUiOutbox>,
    mut audio: ResMut<QuitMenuAudioOutbox>,
    mut sequence: ResMut<QuitMenuClickSoundSequence>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed
            && activate_quit_menu_button(
                button.kind,
                &mut model,
                &mut outbox,
                &mut audio,
                &mut sequence,
            )
        {
            break;
        }
    }
}

pub(super) fn activate_quit_menu_button(
    kind: QuitMenuButtonKind,
    model: &mut QuitMenuUiModel,
    outbox: &mut QuitMenuUiOutbox,
    audio: &mut QuitMenuAudioOutbox,
    sequence: &mut QuitMenuClickSoundSequence,
) -> bool {
    if !model.visible || !model.enabled {
        return false;
    }
    let action = match kind {
        QuitMenuButtonKind::ChangeCharacter => QuitMenuUiAction::ChangeCharacter,
        QuitMenuButtonKind::QuitGame => QuitMenuUiAction::QuitGame,
        QuitMenuButtonKind::QuitAndLogout => QuitMenuUiAction::QuitAndLogout,
        QuitMenuButtonKind::Cancel => QuitMenuUiAction::Cancel {
            source: QuitMenuDismissalSource::CancelButton,
        },
    };
    audio.push(QuitMenuAudioCue::ButtonClick {
        clip_index: sequence.next_clip_index(),
        gain: QUIT_MENU_BUTTON_SOUND_GAIN,
    });
    outbox.push(action);
    // `cnQuit` calls `Exit()` before dispatching every destination.
    model.close();
    true
}

pub(super) fn update_quit_menu_button_visuals(
    model: Res<QuitMenuUiModel>,
    assets: Res<QuitMenuUiAssets>,
    mut buttons: Query<(
        &Interaction,
        &QuitMenuButton,
        &Children,
        &mut ImageNode,
        Option<&mut Pickable>,
    )>,
    mut labels: Query<(&QuitMenuButtonLabel, &mut TextColor)>,
) {
    for (interaction, marker, children, mut image, pickable) in &mut buttons {
        let interaction = if model.visible && model.enabled {
            *interaction
        } else {
            Interaction::None
        };
        let Some(spec) = QUIT_MENU_BUTTONS
            .iter()
            .find(|spec| spec.kind == marker.kind)
        else {
            continue;
        };
        image.image = assets.button_image(spec.visual, interaction);
        image.color = Color::WHITE;
        if let Some(mut pickable) = pickable {
            *pickable = if model.visible && model.enabled {
                Pickable::default()
            } else {
                Pickable::IGNORE
            };
        }
        for child in children.iter() {
            if let Ok((label, mut color)) = labels.get_mut(child)
                && label.kind == marker.kind
            {
                color.0 = button_text_color(spec.visual, interaction);
            }
        }
    }
}

pub(super) fn button_text_color(visual: QuitMenuButtonVisual, interaction: Interaction) -> Color {
    let [red, green, blue, alpha] = match (visual, interaction) {
        (QuitMenuButtonVisual::Standard, Interaction::Hovered) => QUIT_MENU_BUTTON_HOVER_TEXT_COLOR,
        (QuitMenuButtonVisual::Standard, Interaction::None | Interaction::Pressed) => {
            QUIT_MENU_BUTTON_NORMAL_TEXT_COLOR
        }
        (QuitMenuButtonVisual::Cancel, Interaction::None) => QUIT_MENU_CANCEL_NORMAL_TEXT_COLOR,
        (QuitMenuButtonVisual::Cancel, Interaction::Hovered) => QUIT_MENU_CANCEL_HOVER_TEXT_COLOR,
        (QuitMenuButtonVisual::Cancel, Interaction::Pressed) => QUIT_MENU_CANCEL_ACTIVE_TEXT_COLOR,
    };
    Color::srgba(red, green, blue, alpha)
}
