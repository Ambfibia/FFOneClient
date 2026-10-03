use super::*;

pub const NANOCOM_BLUE_BUTTON_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const NANOCOM_BLUE_BUTTON_OVER_PATH: &str = "ui/en/gameplay/chat/blue_button_over.png";

pub const NANOCOM_RED_BUTTON_PATH: &str = "ui/en/server-selection/red_button_normal.png";

pub const NANOCOM_RED_BUTTON_OVER_PATH: &str = "ui/en/server-selection/red_button_over.png";

#[derive(Clone, Copy, Debug, Component)]
pub(super) struct NanocomMessageButton {
    pub(super) choice: NanocomMessageChoice,
    pub(super) presented_request_id: Option<u64>,
}

#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
pub(super) struct NanocomMessageButtonLabel(pub(super) NanocomMessageChoice);

pub(super) fn nanocom_button_text_color(choice: NanocomMessageChoice, interaction: Interaction) -> Color {
    match (choice, interaction) {
        (NanocomMessageChoice::Accept, Interaction::None) => Color::srgb(0.9, 0.9, 0.9),
        (NanocomMessageChoice::Accept, Interaction::Hovered) => {
            Color::srgb(0.0, 0.278_431_4, 0.478_431_37)
        }
        (NanocomMessageChoice::Accept, Interaction::Pressed)
        | (NanocomMessageChoice::Decline, Interaction::Hovered | Interaction::Pressed) => {
            Color::WHITE
        }
        (NanocomMessageChoice::Decline, Interaction::None) => Color::srgb(1.0, 0.995_967_75, 1.0),
    }
}

pub(super) fn handle_nanocom_buttons(
    buttons: Query<(&Interaction, &NanocomMessageButton), Changed<Interaction>>,
    mut model: ResMut<NanocomMessageUiModel>,
    mut outbox: ResMut<NanocomMessageUiOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let active_request_id = model.active().map(|active| active.request.request_id);
        if !nanocom_button_targets_presented_head(button.presented_request_id, active_request_id) {
            continue;
        }
        if let Some(action) = model.choose(button.choice) {
            outbox.push(action);
        }
    }
}

pub(super) fn nanocom_button_targets_presented_head(
    presented_request_id: Option<u64>,
    active_request_id: Option<u64>,
) -> bool {
    presented_request_id.is_some() && presented_request_id == active_request_id
}

pub(super) fn latch_nanocom_button_request_ids(
    model: Res<NanocomMessageUiModel>,
    mut buttons: Query<&mut NanocomMessageButton>,
) {
    let active_request_id = model.active().map(|active| active.request.request_id);
    for mut button in &mut buttons {
        if button.presented_request_id != active_request_id {
            button.presented_request_id = active_request_id;
        }
    }
}

pub(super) fn update_nanocom_button_visuals(
    assets: Res<NanocomMessageUiAssets>,
    mut buttons: Query<
        (
            &Interaction,
            &NanocomMessageButton,
            &Children,
            &mut ImageNode,
        ),
        Changed<Interaction>,
    >,
    mut labels: Query<(&NanocomMessageButtonLabel, &mut TextColor)>,
) {
    for (interaction, button, children, mut image) in &mut buttons {
        image.image = assets.button_image(button.choice, *interaction);
        for child in children.iter() {
            if let Ok((label, mut color)) = labels.get_mut(child) {
                debug_assert_eq!(label.0, button.choice);
                color.0 = nanocom_button_text_color(button.choice, *interaction);
            }
        }
    }
}
