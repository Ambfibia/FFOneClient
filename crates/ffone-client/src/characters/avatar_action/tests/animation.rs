use super::*;

#[test]
fn repeated_same_clip_swings_have_distinct_effect_generations() {
    let mut state = LegacyAvatarActionState::default();
    let context = LegacyAvatarActionContext::default();
    let selection = LegacyTargetSelection::default();
    let bindings = LegacyAvatarClipBindings::default();
    for generation in 1..=3 {
        state.attack_accumulated = 0.0;
        state.seconds_since_attack = f32::INFINITY;
        let mut frame = LegacyFrameSchedule::default();
        schedule_primary_attack(0.01, &context, &selection, &mut state, &bindings, &mut frame);
        assert_eq!(state.attack_generation(), generation);
        assert_eq!(state.attack_sequence(), 1);
        assert!(matches!(state.upper_action, Some(LegacyVisualClip::AttackUpper(1))));
    }
    let mut blocked = context;
    blocked.attack_cooldown_blocked = true;
    schedule_primary_attack(0.01, &blocked, &selection, &mut state, &bindings, &mut LegacyFrameSchedule::default());
    assert_eq!(state.attack_generation(), 3);
}
