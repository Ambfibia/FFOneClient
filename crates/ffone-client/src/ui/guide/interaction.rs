use super::*;

pub const GUIDE_BLUE_BUTTON_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const GUIDE_BLUE_BUTTON_OVER_PATH: &str = "ui/en/gameplay/chat/blue_button_over.png";

pub const GUIDE_CANCEL_BUTTON_PATH: &str = "ui/en/character/selection/controls/CancelNormal.png";

pub const GUIDE_CLOSE_BUTTON_PATH: &str = "ui/en/rule/close.png";

pub const GUIDE_CLOSE_BUTTON_OVER_PATH: &str = "ui/en/world-map/controls/closeover.png";

pub const GUIDE_HELP_BUTTON_PATH: &str = "ui/en/world-map/controls/NanoMachineHelpButton.png";

pub const GUIDE_HELP_BUTTON_OVER_PATH: &str =
    "ui/en/world-map/controls/NanoMachineHelpButtonOver.png";

pub const GUIDE_CHOOSE_BUTTON_LABEL: &str = "CHOOSE GUIDE";

pub const GUIDE_CHANGE_BUTTON_LABEL: &str = "CHANGE GUIDE";

pub const GUIDE_WARP_BUTTON_LABEL: &str = "WARP TO THE PAST";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GuideUiInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub mouse_controls_enabled: bool,
    pub escape_dismiss_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GuideUiButtonVisual {
    Blue,
    BigBlue,
    Cancel,
    Close,
    Help,
    Toggle(GuideMentor),
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct GuideUiCommandButton {
    pub(super) command: GuideUiCommand,
    pub(super) visual: GuideUiButtonVisual,
}

#[derive(Component, Debug)]
pub(super) struct GuideUiButtonLabel;

pub(super) fn handle_guide_ui_interactions(
    buttons: Query<(&Interaction, &GuideUiCommandButton), Changed<Interaction>>,
    mut model: ResMut<GuideUiModel>,
    mut outbox: ResMut<GuideUiOutbox>,
    mut audio: ResMut<GuideUiAudioOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed
            && apply_guide_ui_command(&mut model, &mut outbox, &mut audio, button.command)
        {
            break;
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_guide_ui_button_visuals(
    model: Res<GuideUiModel>,
    assets: Res<GuideUiAssets>,
    mut buttons: Query<(
        &Interaction,
        &GuideUiCommandButton,
        &Children,
        &mut ImageNode,
        Option<&mut Pickable>,
    )>,
    mut labels: Query<&mut TextColor, With<GuideUiButtonLabel>>,
) {
    for (interaction, button, children, mut image, pickable) in &mut buttons {
        let enabled = guide_ui_button_enabled(&model, button.command);
        let interaction = if enabled {
            *interaction
        } else {
            Interaction::None
        };
        image.image = match button.visual {
            GuideUiButtonVisual::Toggle(mentor) => {
                if model.selected == Some(mentor) {
                    assets.toggle_on.clone()
                } else {
                    assets.toggle_off.clone()
                }
            }
            _ => button_image_handle(&assets, button.visual, interaction),
        };
        image.color = Color::WHITE;
        if let Some(mut pickable) = pickable {
            *pickable = if enabled {
                Pickable::default()
            } else {
                Pickable::IGNORE
            };
        }
        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child) {
                color.0 = button_text_color(button.visual, interaction);
            }
        }
    }
}

#[must_use]
pub fn guide_ui_button_enabled(model: &GuideUiModel, command: GuideUiCommand) -> bool {
    if matches!(
        command,
        GuideUiCommand::Dismiss(GuideUiDismissalSource::EscapeKey)
    ) {
        return model.input_boundary().escape_dismiss_enabled;
    }
    if !model.controls_enabled() {
        return false;
    }
    match command {
        GuideUiCommand::AcceptWarpWarning => model.phase == GuideUiPhase::WarpWarning,
        GuideUiCommand::SelectMentor(_)
        | GuideUiCommand::OpenConfirmation
        | GuideUiCommand::Dismiss(
            GuideUiDismissalSource::CancelButton | GuideUiDismissalSource::CloseButton,
        )
        | GuideUiCommand::RequestHelp => {
            model.phase == GuideUiPhase::MentorSelection && !model.confirmation_open
        }
        GuideUiCommand::ConfirmMentor | GuideUiCommand::CancelConfirmation => {
            model.phase == GuideUiPhase::MentorSelection && model.confirmation_open
        }
        GuideUiCommand::Dismiss(GuideUiDismissalSource::WarpCancelButton) => {
            model.phase == GuideUiPhase::WarpWarning
        }
        GuideUiCommand::Dismiss(GuideUiDismissalSource::EscapeKey) => false,
    }
}

pub(super) fn button_image(
    assets: &GuideUiAssets,
    visual: GuideUiButtonVisual,
    interaction: Interaction,
) -> ImageNode {
    let image = button_image_handle(assets, visual, interaction);
    match visual {
        GuideUiButtonVisual::Blue | GuideUiButtonVisual::BigBlue => {
            sliced_image(image, GUIDE_BLUE_BUTTON_BORDER)
        }
        GuideUiButtonVisual::Cancel => sliced_image(image, GUIDE_CANCEL_BUTTON_BORDER),
        GuideUiButtonVisual::Close | GuideUiButtonVisual::Help | GuideUiButtonVisual::Toggle(_) => {
            stretch_image(image)
        }
    }
}

pub(super) fn button_image_handle(
    assets: &GuideUiAssets,
    visual: GuideUiButtonVisual,
    interaction: Interaction,
) -> Handle<Image> {
    match (visual, interaction) {
        (GuideUiButtonVisual::Blue | GuideUiButtonVisual::BigBlue, Interaction::Hovered) => {
            assets.blue_button_over.clone()
        }
        (
            GuideUiButtonVisual::Blue | GuideUiButtonVisual::BigBlue,
            Interaction::None | Interaction::Pressed,
        ) => assets.blue_button.clone(),
        (GuideUiButtonVisual::Cancel, Interaction::None) => assets.cancel_button.clone(),
        (GuideUiButtonVisual::Cancel, Interaction::Hovered | Interaction::Pressed) => {
            assets.blue_button.clone()
        }
        (GuideUiButtonVisual::Close, Interaction::Hovered) => assets.close_button_over.clone(),
        (GuideUiButtonVisual::Close, Interaction::None | Interaction::Pressed) => {
            assets.close_button.clone()
        }
        (GuideUiButtonVisual::Help, Interaction::Hovered) => assets.help_button_over.clone(),
        (GuideUiButtonVisual::Help, Interaction::None | Interaction::Pressed) => {
            assets.help_button.clone()
        }
        (GuideUiButtonVisual::Toggle(_), _) => assets.toggle_off.clone(),
    }
}

pub(super) fn button_text_color(visual: GuideUiButtonVisual, interaction: Interaction) -> Color {
    match (visual, interaction) {
        (GuideUiButtonVisual::Blue, Interaction::None) => Color::srgb(0.9, 0.9, 0.9),
        (GuideUiButtonVisual::Blue, Interaction::Hovered) => {
            Color::srgb(0.0, 0.278_431_4, 0.478_431_37)
        }
        (GuideUiButtonVisual::Blue, Interaction::Pressed) => Color::WHITE,
        (GuideUiButtonVisual::BigBlue, Interaction::None) => {
            Color::srgb(0.752_941_2, 0.949_019_6, 0.952_941_2)
        }
        (GuideUiButtonVisual::BigBlue, Interaction::Hovered) => {
            Color::srgb(0.011_764_706, 0.172_549_02, 0.301_960_8)
        }
        (GuideUiButtonVisual::BigBlue, Interaction::Pressed) => {
            Color::srgb(0.752_941_2, 0.941_176_5, 0.941_176_5)
        }
        (GuideUiButtonVisual::Cancel, Interaction::None) => {
            Color::srgb(0.971_774_2, 0.971_774_2, 0.971_774_2)
        }
        (GuideUiButtonVisual::Cancel, Interaction::Hovered | Interaction::Pressed)
        | (
            GuideUiButtonVisual::Close | GuideUiButtonVisual::Help | GuideUiButtonVisual::Toggle(_),
            _,
        ) => Color::WHITE,
    }
}

pub(super) fn button_padding(visual: GuideUiButtonVisual) -> UiRect {
    match visual {
        GuideUiButtonVisual::BigBlue => UiRect {
            left: px(10),
            right: px(6),
            top: px(4),
            bottom: px(6),
        },
        GuideUiButtonVisual::Blue | GuideUiButtonVisual::Cancel => UiRect {
            left: px(10),
            right: px(6),
            top: px(3),
            bottom: px(6),
        },
        GuideUiButtonVisual::Close | GuideUiButtonVisual::Help | GuideUiButtonVisual::Toggle(_) => {
            UiRect::ZERO
        }
    }
}
