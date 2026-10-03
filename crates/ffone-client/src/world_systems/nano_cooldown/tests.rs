use crate::world_nano_cooldown::*;

#[test]
fn accepted_request_starts_one_slot_from_xdt_tenths() {
    let mut runtime = WorldNanoCooldownRuntime::default();
    assert_eq!(runtime.start(1, 7, 22, 80), Ok(8.0));
    assert!(!runtime.is_active(0, 7));
    assert!(runtime.is_active(1, 7));
    assert_eq!(runtime.remaining_fraction(1, 7), Some(1.0));
    assert_eq!(runtime.slots()[1].cool_type, 22);

    runtime.advance(2.0);
    assert_eq!(runtime.remaining_fraction(1, 7), Some(0.75));
    runtime.advance(6.0);
    assert!(!runtime.is_active(1, 7));
    assert_eq!(runtime.remaining_fraction(1, 7), None);
}

#[test]
fn switching_active_nano_preserves_channels_but_retuning_clears_only_changed_slot() {
    let mut runtime = WorldNanoCooldownRuntime::default();
    runtime.start(0, 1, 22, 80).unwrap();
    runtime.start(2, 9, 7, 30).unwrap();

    runtime.retain_equipped([Some(1), Some(5), Some(9)]);
    assert!(runtime.is_active(0, 1));
    assert!(runtime.is_active(2, 9));

    runtime.retain_equipped([Some(2), Some(5), Some(9)]);
    assert!(!runtime.is_active(0, 1));
    assert!(runtime.is_active(2, 9));
}

#[test]
fn malformed_and_zero_duration_channels_fail_closed() {
    let mut runtime = WorldNanoCooldownRuntime::default();
    assert!(runtime.start(3, 1, 0, 10).is_err());
    assert!(runtime.start(0, 0, 0, 10).is_err());
    assert!(runtime.start(0, 1, 0, -1).is_err());
    assert_eq!(runtime.start(0, 1, 0, 0), Ok(0.0));
    assert_eq!(runtime.remaining_fraction(0, 1), None);
}
