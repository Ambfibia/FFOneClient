use super::*;

#[test]
fn background_animation_clock_starts_at_zero_each_time_screen_opens() {
    let mut clock = CharacterSelectionBackgroundClock::default();

    assert_eq!(clock.update(false, 10.0), 0.0);
    assert_eq!(clock.update(true, 10.0), 0.0);
    assert_eq!(clock.update(true, 0.25), 0.25);
    assert_eq!(clock.update(true, 0.50), 0.75);
    assert_eq!(clock.update(false, 20.0), 0.0);
    assert_eq!(clock.update(true, 20.0), 0.0);
}
