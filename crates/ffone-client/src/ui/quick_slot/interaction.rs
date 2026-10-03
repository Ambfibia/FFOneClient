use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct QuickSlotButton(pub(super) usize);

pub(super) fn handle_quick_slot_buttons(
    buttons: Query<(&Interaction, &QuickSlotButton), Changed<Interaction>>,
    asset_server: Res<AssetServer>,
    model: Res<QuickSlotUiModel>,
    mut outbox: ResMut<QuickSlotUiOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(entry) = model.slots.get(button.0) else {
            continue;
        };
        let icon_loaded = entry
            .icon_path
            .as_deref()
            .filter(|path| is_semantic_png_path(path))
            .map(|path| asset_server.load::<Image>(path.to_owned()))
            .is_some_and(|handle| {
                matches!(asset_server.load_state(handle.id()), LoadState::Loaded)
            });
        if icon_loaded {
            if let Some(action) = model.pointer_action_for_slot(button.0) {
                outbox.actions.push_back(action);
            }
        }
    }
}
