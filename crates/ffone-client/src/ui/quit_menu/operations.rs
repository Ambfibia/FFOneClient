use super::*;

pub fn dismiss_quit_menu_with_escape(
    model: &mut QuitMenuUiModel,
    outbox: &mut QuitMenuUiOutbox,
) -> bool {
    if !model.visible {
        return false;
    }
    // Source parity: `GUI.enabled=false` disables mouse buttons, but
    // `cnQuit.Update` evaluates Escape independently.
    outbox.push(QuitMenuUiAction::Cancel {
        source: QuitMenuDismissalSource::EscapeKey,
    });
    model.close();
    true
}
