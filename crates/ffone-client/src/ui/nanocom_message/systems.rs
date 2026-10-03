use super::*;

pub(super) fn tick_nanocom_messages(
    time: Res<Time>,
    mut model: ResMut<NanocomMessageUiModel>,
    mut outbox: ResMut<NanocomMessageUiOutbox>,
) {
    if model.is_empty() {
        return;
    }
    if let Some(action) = model.tick(time.delta_secs()) {
        outbox.push(action);
    }
}
