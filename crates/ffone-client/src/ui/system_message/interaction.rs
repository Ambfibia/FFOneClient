use super::*;

pub const SYSTEM_MESSAGE_BLUE_BUTTON_PATH: &str =
    "ui/en/character/selection/controls/blue_button_normal.png";

pub const SYSTEM_MESSAGE_BLUE_BUTTON_OVER_PATH: &str =
    "ui/en/character/selection/controls/blue_button_over.png";

pub const SYSTEM_MESSAGE_RED_BUTTON_PATH: &str =
    "ui/en/character/selection/controls/red_button_normal.png";

pub const SYSTEM_MESSAGE_RED_BUTTON_OVER_PATH: &str =
    "ui/en/character/selection/controls/red_button_over.png";

pub const SYSTEM_MESSAGE_CANCEL_BUTTON_PATH: &str =
    "ui/en/character/selection/controls/CancelNormal.png";

/// Renderable `cnSystemMessageElement.Type` values after the clean client's
/// failure-type normalization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum SystemMessageButtonType {
    None = 0,
    Ok = 1,
    OkCancel = 2,
    YesNo = 3,
    ExitCharacterCreation = 6,
    DeleteMission = 7,
    TutorialSkip = 8,
    LeaveGroup = 9,
    DeleteItem = 12,
    CancelWarp = 13,
    CombinationFailure = 14,
    CombinationConfirm = 15,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemMessageButtonTypeError {
    /// Declared by the legacy enum but not rendered by clean `DrawAll`.
    Unrendered(i32),
    /// Outside the declared legacy range.
    Invalid(i32),
}

impl fmt::Display for SystemMessageButtonTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrendered(raw) => {
                write!(
                    formatter,
                    "legacy system-message button type {raw} has no clean render path"
                )
            }
            Self::Invalid(raw) => {
                write!(formatter, "invalid legacy system-message button type {raw}")
            }
        }
    }
}

impl Error for SystemMessageButtonTypeError {}

impl TryFrom<i32> for SystemMessageButtonType {
    type Error = SystemMessageButtonTypeError;

    fn try_from(raw: i32) -> Result<Self, Self::Error> {
        match raw {
            0 => Ok(Self::None),
            1 | 10 => Ok(Self::Ok),
            2 | 11 => Ok(Self::OkCancel),
            3 => Ok(Self::YesNo),
            4 | 5 => Err(SystemMessageButtonTypeError::Unrendered(raw)),
            6 => Ok(Self::ExitCharacterCreation),
            7 => Ok(Self::DeleteMission),
            8 => Ok(Self::TutorialSkip),
            9 => Ok(Self::LeaveGroup),
            12 => Ok(Self::DeleteItem),
            13 => Ok(Self::CancelWarp),
            14 => Ok(Self::CombinationFailure),
            15 => Ok(Self::CombinationConfirm),
            _ => Err(SystemMessageButtonTypeError::Invalid(raw)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemMessageButtonVisual {
    Standard,
    Cancel,
    Destructive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemMessageButtonSpec {
    pub choice: SystemMessageChoice,
    pub label: &'static str,
    pub visual: SystemMessageButtonVisual,
    pub rect: SystemMessageUiRect,
}

pub(super) const fn button(
    choice: SystemMessageChoice,
    label: &'static str,
    visual: SystemMessageButtonVisual,
    rect: SystemMessageUiRect,
) -> SystemMessageButtonSpec {
    SystemMessageButtonSpec {
        choice,
        label,
        visual,
        rect,
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessageButton {
    pub(super) choice: SystemMessageChoice,
    pub(super) visual: SystemMessageButtonVisual,
}

#[derive(Default, Resource)]
pub(super) struct SystemMessageCursorLease {
    pub(super) saved: Option<(bool, CursorGrabMode)>,
}

pub(super) const fn system_message_button_text_style(
    visual: SystemMessageButtonVisual,
) -> SystemMessageTextStyle {
    match visual {
        SystemMessageButtonVisual::Standard => SystemMessageTextStyle::Button,
        SystemMessageButtonVisual::Cancel => SystemMessageTextStyle::CancelButton,
        SystemMessageButtonVisual::Destructive => SystemMessageTextStyle::RedButton,
    }
}

pub(super) fn system_message_button_padding(visual: SystemMessageButtonVisual) -> UiRect {
    style_padding(system_message_button_text_style(visual).spec().padding)
}

pub const fn system_message_button_font_size(visual: SystemMessageButtonVisual) -> f32 {
    system_message_button_text_style(visual).spec().font_size
}

pub(super) fn handle_system_message_buttons(
    buttons: Query<(&Interaction, &SystemMessageButton), Changed<Interaction>>,
    mut model: ResMut<SystemMessageUiModel>,
    mut outbox: ResMut<SystemMessageUiOutbox>,
    mut audio: ResMut<SystemMessageUiAudioOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(action) = model.choose(button.choice) {
            let &SystemMessageUiAction::Chosen {
                button_type,
                choice,
                ..
            } = &action;
            audio.push(system_message_audio_cue(button_type, choice));
            outbox.actions.push_back(action);
        }
    }
}

pub(super) fn update_system_message_button_visuals(
    model: Res<SystemMessageUiModel>,
    assets: Res<SystemMessageUiAssets>,
    mut buttons: Query<
        (
            &Interaction,
            &SystemMessageButton,
            &Children,
            &mut ImageNode,
        ),
        Changed<Interaction>,
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    for (interaction, marker, children, mut image) in &mut buttons {
        let interaction = if model.focus_out() {
            Interaction::None
        } else {
            *interaction
        };
        image.image = assets.button_image(marker.visual, interaction);
        image.color = if model.focus_out() {
            Color::srgba(0.52, 0.52, 0.52, 0.78)
        } else {
            Color::WHITE
        };
        let state = match interaction {
            Interaction::None => "normal",
            Interaction::Hovered => "hover",
            Interaction::Pressed => "active",
        };
        let color = system_message_button_text_color(marker.visual, state);
        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}

pub(super) fn sync_system_message_cursor(
    model: Res<SystemMessageUiModel>,
    mut lease: ResMut<SystemMessageCursorLease>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    if !model.is_empty() {
        if lease.saved.is_none() {
            lease.saved = Some((cursor.visible, cursor.grab_mode));
        }
        if !cursor.visible {
            cursor.visible = true;
        }
        if cursor.grab_mode != CursorGrabMode::None {
            cursor.grab_mode = CursorGrabMode::None;
        }
    } else if let Some((visible, grab_mode)) = lease.saved.take() {
        cursor.visible = visible;
        cursor.grab_mode = grab_mode;
    }
}

pub(super) fn system_message_button_text_color(visual: SystemMessageButtonVisual, state: &str) -> Color {
    let style = match visual {
        SystemMessageButtonVisual::Standard => "button",
        SystemMessageButtonVisual::Cancel => "CancelButton",
        SystemMessageButtonVisual::Destructive => "RedButton",
    };
    gui_style("FusionFallSysMessageSkin", style)
        .and_then(|style| {
            style
                .states
                .get(state)
                .or_else(|| style.states.get("normal"))
        })
        .map_or(Color::WHITE, |state| bevy_color(&state.text_color))
}
