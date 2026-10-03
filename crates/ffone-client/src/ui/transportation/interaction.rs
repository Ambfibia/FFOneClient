use super::*;

pub const TRANSPORTATION_SCROLL_VELOCITY_SOURCE: &str = "InventoryManagerScript.scrollVel";

pub const TRANSPORTATION_BLUE_BUTTON_PATH: &str =
    "ui/en/transportation/controls/blue_button_normal.png";

pub const TRANSPORTATION_BLUE_BUTTON_HOVER_PATH: &str =
    "ui/en/transportation/controls/blue_button_over.png";

pub const TRANSPORTATION_SCROLL_UP_PATH: &str = "ui/en/transportation/controls/scroll_up.png";

pub const TRANSPORTATION_SCROLL_BAR_PATH: &str = "ui/en/transportation/controls/scroll_bar.png";

pub const TRANSPORTATION_SCROLL_THUMB_PATH: &str = "ui/en/transportation/controls/scroll_thumb.png";

pub const TRANSPORTATION_SCROLL_DOWN_PATH: &str = "ui/en/transportation/controls/scroll_down.png";

pub const TRANSPORTATION_BUTTON_REPLACEMENT_Y_OFFSET: f32 = 0.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TransportationInputGates {
    pub system_popup_open: bool,
    pub escape_close_allowed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationInputResult {
    Ignored,
    Changed,
    BeganWarp,
    Closed,
}

#[derive(Clone, Copy, Debug, Default, Resource, PartialEq)]
pub struct TransportationPresentationInput {
    pub gates: TransportationInputGates,
    /// The owning inventory runtime supplies its exact
    /// `InventoryManagerScript.scrollVel` equivalent.
    pub legacy_scroll_velocity: f32,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationScrollThumb;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationScrollbar;

pub(super) fn queue_transportation_scroll(
    model: Res<TransportationModel>,
    input: Res<TransportationPresentationInput>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut outbox: ResMut<TransportationUiCommandOutbox>,
) {
    if model.phase() == TransportationPhase::Hidden {
        for _ in wheel_events.read() {}
        return;
    }
    let axis = wheel_events.read().map(|event| event.y).sum::<f32>();
    if axis != 0.0 {
        outbox.push(TransportationUiCommand::ScrollAxis {
            axis,
            legacy_scroll_velocity: input.legacy_scroll_velocity,
        });
    }
}
