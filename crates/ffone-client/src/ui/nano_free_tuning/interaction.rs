use super::*;

pub const NANO_FREE_TUNING_SELECT_HOVER_PATH: &str = "ui/en/nano-free-tuning/select_hover.png";

pub const NANO_FREE_TUNING_BUTTON_HOVER_TEXT: Color =
    Color::srgba(0.0, 0.278_431_4, 0.478_431_37, 1.0);

/// Native disabled feedback; never changes the authored button geometry.
pub const NANO_FREE_TUNING_BUTTON_DISABLED_ALPHA: f32 = 0.5;

pub const NANO_FREE_TUNING_BUTTON_REPLACEMENT_Y_OFFSET: f32 = 0.0;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningPowerButton {
    pub power_index: usize,
}

#[derive(Component)]
pub(super) struct NanoFreeTuningButtonLabel;

pub(super) fn sync_nano_free_tuning_button_visuals(
    model: Res<NanoFreeTuningModel>,
    assets: Res<NanoFreeTuningPresentationAssets>,
    mut buttons: Query<
        (&Interaction, &Children, &mut ImageNode, &mut Pickable),
        With<NanoFreeTuningPowerButton>,
    >,
    mut labels: Query<&mut TextColor, With<NanoFreeTuningButtonLabel>>,
) {
    let enabled = model.controls_enabled();
    let alpha = if enabled {
        1.0
    } else {
        NANO_FREE_TUNING_BUTTON_DISABLED_ALPHA
    };
    for (interaction, children, mut image, mut pickable) in &mut buttons {
        let hovered = enabled && *interaction == Interaction::Hovered;
        image.image = if hovered {
            assets.select_hover.clone()
        } else {
            // The source button has identical normal and active backgrounds.
            assets.select_normal.clone()
        };
        image.color = Color::WHITE.with_alpha(alpha);
        for child in children.iter() {
            if let Ok(mut label) = labels.get_mut(child) {
                label.0 = if hovered {
                    NANO_FREE_TUNING_BUTTON_HOVER_TEXT
                } else {
                    NanoFreeTuningUiTextStyle::ButtonMiddleCenter
                        .color()
                        .0
                        .with_alpha(alpha)
                };
            }
        }
        *pickable = if enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}
