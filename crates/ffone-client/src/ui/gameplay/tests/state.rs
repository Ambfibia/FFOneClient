use super::*;

#[test]
fn nano_effect_state_fails_closed_and_matches_gumball_slot_semantics() {
    for value in [None, Some(0.0), Some(-0.1), Some(1.1), Some(f32::NAN)] {
        assert_eq!(nano_cooldown_layers(true, value), None);
    }
    assert_eq!(nano_cooldown_layers(false, Some(1.0)), None);

    let mut state = NanoWheelTransientUi::default();
    assert!(state.set_gumball_effect(0));
    assert!(state.set_gumball_effect(2));
    assert_eq!(
        state.slots.map(|slot| slot.gumball_enabled),
        [true, false, true]
    );
    assert!(!state.set_gumball_effect(3));
    assert_eq!(
        state.slots.map(|slot| slot.gumball_enabled),
        [true, false, true]
    );
    assert!(state.set_gumball_effect(-1));
    assert_eq!(state.slots.map(|slot| slot.gumball_enabled), [false; 3]);
}
