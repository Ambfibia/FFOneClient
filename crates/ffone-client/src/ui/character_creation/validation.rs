use super::*;

/// The clean default is `localized.local = US`; its Korean-only Check button
/// and 10-character input branch are unreachable in primary.
pub const CHARACTER_CREATION_PRIMARY_KOREAN_CHECK_REACHABLE: bool = false;

pub(super) const CC_CHECK_NORMAL: &str = "ui/en/character/creation/body/checkbox/CCCheckboxNormal.png";

pub(super) const CC_CHECK_OVER: &str = "ui/en/character/creation/body/checkbox/CCCheckboxOver.png";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterNameValidationError {
    NameTableUnavailable,
    Empty,
    MissingLastPart,
    FirstPartTooLong,
    LastPartTooLong,
}

pub fn validate_custom_name(
    complete_name: &str,
) -> Result<CustomCharacterName, CharacterNameValidationError> {
    if complete_name.is_empty() {
        return Err(CharacterNameValidationError::Empty);
    }
    let mut parts = complete_name.split(' ');
    let first = parts.next().unwrap_or_default();
    let remainder = parts.collect::<Vec<_>>();
    if remainder.is_empty() {
        return Err(CharacterNameValidationError::MissingLastPart);
    }
    let last = remainder.join(" ");
    if first.chars().count() > 8 {
        return Err(CharacterNameValidationError::FirstPartTooLong);
    }
    if last.chars().count() > 16 {
        return Err(CharacterNameValidationError::LastPartTooLong);
    }
    Ok(CustomCharacterName {
        first: first.to_owned(),
        last,
    })
}
