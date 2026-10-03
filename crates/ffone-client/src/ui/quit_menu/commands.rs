use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuitMenuUiAction {
    ChangeCharacter,
    QuitGame,
    QuitAndLogout,
    Cancel { source: QuitMenuDismissalSource },
}
