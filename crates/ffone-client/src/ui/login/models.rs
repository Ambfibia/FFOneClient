use super::*;

#[derive(Debug, Resource)]
pub struct LoginUiModel {
    pub visible: bool,
    pub username: String,
    pub password: String,
    /// Explicit selection permits using the stored credential without a password field.
    pub saved_account: Option<String>,
    pub focused: LoginField,
    pub username_edit: TextEdit,
    pub password_edit: TextEdit,
    pub status: String,
    pub busy: bool,
    pub surface: LoginSurface,
    /// Mirrors the global `cnSystemMessageManager.IsSysPopUp()` input gate.
    pub system_popup_active: bool,
}

impl Default for LoginUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            username: String::new(),
            password: String::new(),
            saved_account: None,
            focused: LoginField::Username,
            username_edit: TextEdit::default(),
            password_edit: TextEdit::default(),
            status: String::new(),
            busy: false,
            surface: LoginSurface::Manual,
            system_popup_active: false,
        }
    }
}

impl LoginUiModel {
    pub(super) fn accepts_manual_input(&self) -> bool {
        self.visible
            && self.surface == LoginSurface::Manual
            && !self.busy
            && !self.system_popup_active
    }
}
