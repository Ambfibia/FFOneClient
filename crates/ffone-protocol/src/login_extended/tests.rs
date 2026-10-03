use super::*;

#[test]
fn login_extension_abi_sizes_and_round_trips_are_exact() {
    let duplicate = DuplicateExitRequest0104 {
        id: FixedUtf16::from_str("account").unwrap(),
        password: FixedUtf16::from_str("secret").unwrap(),
    };
    let duplicate_wire = duplicate.encode();
    assert_eq!(duplicate_wire.len(), 132);
    assert_eq!(
        DuplicateExitRequest0104::decode(&duplicate_wire),
        Ok(duplicate)
    );

    let request = CharacterNameChangeRequest0104 {
        pc_uid: 0x0102_0304_0506_0708,
        slot: 2,
        gender: 1,
        first_name_code: 11,
        last_name_code: 12,
        middle_name_code: 13,
        first_name: FixedUtf16::from_str("Dexter").unwrap(),
        last_name: FixedUtf16::from_str("Morgan").unwrap(),
    };
    let wire = request.encode();
    assert_eq!(wire.len(), 76);
    assert_eq!(&wire[10..12], &[0, 0]);
    assert_eq!(CharacterNameChangeRequest0104::decode(&wire), Ok(request));

    let success = CharacterNameChangeSuccess0104 {
        pc_uid: 42,
        slot: 1,
        first_name: FixedUtf16::from_str("Proto").unwrap(),
        last_name: FixedUtf16::from_str("Hero").unwrap(),
    };
    let success_wire = success.encode();
    assert_eq!(success_wire.len(), 64);
    assert_eq!(
        CharacterNameChangeSuccess0104::decode(&success_wire),
        Ok(success)
    );

    let failure = CharacterNameChangeFailure0104 {
        pc_uid: 42,
        slot: 1,
        error_code: 4,
    };
    assert_eq!(
        CharacterNameChangeFailure0104::decode(&failure.encode()),
        Ok(failure)
    );
}

#[test]
fn duplicate_debug_never_discloses_password() {
    let request = DuplicateExitRequest0104 {
        id: FixedUtf16::from_str("account").unwrap(),
        password: FixedUtf16::from_str("secret-password").unwrap(),
    };
    let debug = format!("{request:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("secret-password"));
}
