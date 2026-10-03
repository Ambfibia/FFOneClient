use super::*;

#[cfg(test)]
pub(super) fn apply_login_key(
    model: &mut LoginUiModel,
    outbox: &mut LoginUiOutbox,
    key_code: KeyCode,
    produced: Option<&str>,
) {
    apply_login_edit_key(model, outbox, key_code, produced, false, false);
}

pub(super) fn apply_login_edit_key(
    model: &mut LoginUiModel,
    outbox: &mut LoginUiOutbox,
    key_code: KeyCode,
    produced: Option<&str>,
    control: bool,
    shift: bool,
) {
    match key_code {
        KeyCode::Tab => {
            model.focused = match model.focused {
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Username,
            };
            return;
        }
        KeyCode::Enter | KeyCode::NumpadEnter => {
            queue_login(model, outbox);
            return;
        }
        _ => {}
    }
    let (field, edit) = match model.focused {
        LoginField::Username => (&mut model.username, &mut model.username_edit),
        LoginField::Password => (&mut model.password, &mut model.password_edit),
    };
    if edit.key(field, key_code, control, shift) || control {
        return;
    }
    if let Some(produced) = produced {
        for ch in produced.chars().filter(|ch| !ch.is_control()) {
            edit.insert(field, ch.encode_utf8(&mut [0; 4]), 32, false);
            if model.focused == LoginField::Password {
                let leading = field.chars().take_while(|ch| ch.is_whitespace()).count();
                *field = field.trim().to_owned();
                edit.place(edit.cursor.saturating_sub(leading), false);
                edit.clamp(field);
            }
        }
    }
}
