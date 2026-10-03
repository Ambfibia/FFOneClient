use super::*;

#[test]
fn custom_name_validation_keeps_legacy_limits() {
    assert_eq!(
        validate_custom_name(""),
        Err(CharacterNameValidationError::Empty)
    );
    assert_eq!(
        validate_custom_name("Alice"),
        Err(CharacterNameValidationError::MissingLastPart)
    );
    assert_eq!(
        validate_custom_name("123456789 Lee"),
        Err(CharacterNameValidationError::FirstPartTooLong)
    );
    assert_eq!(
        validate_custom_name("Ava 12345678901234567"),
        Err(CharacterNameValidationError::LastPartTooLong)
    );
    assert_eq!(
        validate_custom_name("Ava Ter Lee").unwrap(),
        CustomCharacterName {
            first: "Ava".to_owned(),
            last: "Ter Lee".to_owned(),
        }
    );
}
