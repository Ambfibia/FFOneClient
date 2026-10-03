use super::*;

pub(super) fn data_gender(gender: UiGender) -> DataGender {
    match gender {
        UiGender::Boy => DataGender::Male,
        UiGender::Girl => DataGender::Female,
    }
}

pub(super) fn protocol_gender(code: i8) -> CharacterCreationDataResult<DataGender> {
    match code {
        1 => Ok(DataGender::Male),
        2 => Ok(DataGender::Female),
        _ => invalid(format!("unsupported protocol gender {code}")),
    }
}

pub(super) fn positive_style(label: &str, value: i8) -> CharacterCreationDataResult<u32> {
    if value <= 0 {
        return invalid(format!("{label} style must be positive, got {value}"));
    }
    Ok(value as u32)
}

pub(super) fn palette_color(
    label: &str,
    code: u8,
    colors: &[CharacterPaletteColor],
) -> CharacterCreationDataResult<LinearRgba> {
    let color = colors
        .iter()
        .find(|color| color.code == code)
        .ok_or_else(|| {
            CharacterCreationDataError::Invalid(format!(
                "{label} palette has no protocol code {code}"
            ))
        })?;
    Ok(LinearRgba::new(
        color.rgba[0],
        color.rgba[1],
        color.rgba[2],
        color.rgba[3],
    ))
}

pub(super) fn compose_legacy_last_name(middle: &str, last: &str) -> String {
    let middle = if middle == " " { "" } else { middle };
    let last = if last == " " { "" } else { last };
    let mut suffix = last.to_owned();
    if let Some(first) = suffix.get_mut(0..1) {
        if middle.contains(' ') || middle.is_empty() {
            first.make_ascii_uppercase();
        } else {
            first.make_ascii_lowercase();
        }
    }
    format!("{middle}{suffix}")
}

pub(super) fn to_u8(label: &str, value: u16) -> CharacterCreationDataResult<u8> {
    u8::try_from(value)
        .map_err(|_| CharacterCreationDataError::Invalid(format!("{label} {value} exceeds u8")))
}

pub(super) fn to_i8(label: &str, value: u32) -> CharacterCreationDataResult<i8> {
    i8::try_from(value)
        .map_err(|_| CharacterCreationDataError::Invalid(format!("{label} {value} exceeds i8")))
}

pub(super) fn to_i16(label: &str, value: u32) -> CharacterCreationDataResult<i16> {
    i16::try_from(value)
        .map_err(|_| CharacterCreationDataError::Invalid(format!("{label} {value} exceeds i16")))
}

pub(super) fn invalid<T>(message: impl Into<String>) -> CharacterCreationDataResult<T> {
    Err(CharacterCreationDataError::Invalid(message.into()))
}
