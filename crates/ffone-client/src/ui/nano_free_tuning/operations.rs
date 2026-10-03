use super::*;

#[must_use]
pub const fn nano_free_tuning_entry_first_use_condition(nano_id: i16) -> Option<i32> {
    match nano_id {
        3 => Some(52),
        4 => Some(39),
        5 => Some(53),
        8 => Some(51),
        9 => Some(49),
        11 => Some(50),
        _ => None,
    }
}

pub(super) fn queue_nano_free_tuning_controls(
    model: Res<NanoFreeTuningModel>,
    mouse: Res<ButtonInput<MouseButton>>,
    buttons: Query<(
        Entity,
        &NanoFreeTuningPowerButton,
        &Interaction,
        &RelativeCursorPosition,
    )>,
    mut armed: Local<Option<Entity>>,
    mut outbox: ResMut<NanoFreeTuningUiCommandOutbox>,
) {
    if !model.controls_enabled() {
        *armed = None;
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        *armed = buttons.iter().find_map(|(entity, _, interaction, cursor)| {
            (*interaction == Interaction::Pressed && cursor.cursor_over()).then_some(entity)
        });
    }
    if mouse.just_released(MouseButton::Left) {
        if let Some(entity) = armed.take()
            && let Ok((_, button, interaction, cursor)) = buttons.get(entity)
            && cursor.cursor_over()
            && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
        {
            outbox.push(NanoFreeTuningUiCommand::SelectAndConfirm {
                power_index: button.power_index,
            });
        }
    } else if !mouse.pressed(MouseButton::Left) {
        *armed = None;
    }
}
