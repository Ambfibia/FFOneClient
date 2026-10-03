use super::*;

pub const RACE_RESULT_BUTTON_NORMAL_PATH: &str = "ui/en/race/result/button_normal.png";

pub const RACE_RESULT_BUTTON_HOVER_PATH: &str = "ui/en/race/result/button_hover.png";

pub const RACE_RESULT_BUTTON_ACTIVE_PATH: &str = "ui/en/race/result/button_active.png";

#[derive(Clone, Copy, Debug, Default, Resource, PartialEq, Eq)]
pub struct RaceModePresentationInput {
    pub system_popup_active: bool,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceModeAcceptButton;

pub(super) fn sync_race_mode_button_visual(
    model: Res<RaceModeModel>,
    input: Res<RaceModePresentationInput>,
    assets: Res<RaceModePresentationAssets>,
    mut buttons: Query<(&Interaction, &mut ImageNode, &mut Pickable), With<RaceModeAcceptButton>>,
) {
    let enabled = model.controls_enabled(input.system_popup_active);
    for (interaction, mut image, mut pickable) in &mut buttons {
        image.image = match (*interaction, enabled) {
            (Interaction::Pressed, true) => assets.button_active.clone(),
            (Interaction::Hovered, true) => assets.button_hover.clone(),
            _ => assets.button_normal.clone(),
        };
        *pickable = if enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}
