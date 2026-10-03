use super::*;

pub(super) fn read_quick_slot_hotkeys(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    config: Res<QuickSlotUiConfig>,
    model: Res<QuickSlotUiModel>,
    mut outbox: ResMut<QuickSlotUiOutbox>,
) {
    let Some(keyboard) = keyboard else {
        return;
    };
    if let Some(action) = model.hotkey_action(*config, quick_slot_input_frame_from_keys(&keyboard))
    {
        outbox.actions.push_back(action);
    }
}
