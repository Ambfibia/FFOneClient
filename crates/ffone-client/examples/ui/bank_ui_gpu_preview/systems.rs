use super::*;

// Match the production host's Bank system-message context synchronization.
pub(super) fn sync_preview_system_modal(
    messages: Res<ffone_client::system_message_ui::SystemMessageUiModel>,
    mut modal: ResMut<BankModalState>,
) {
    let active = messages.is_popup();
    if modal.system_popup != active {
        modal.system_popup = active;
    }
}
