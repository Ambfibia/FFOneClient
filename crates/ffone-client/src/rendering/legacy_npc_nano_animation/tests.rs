use crate::legacy_npc_nano_animation::*;

#[test]
fn nano_idle_voice_gate_and_low_stamina_first_pass_match_state_transitions() {
    let mut random = LegacyNanoStandRandomStream::with_seed(9);
    let mut machine = LegacyNanoAnimationMachine::default();
    machine.request_stand(&mut random);
    let accepted = (0..400)
        .filter(|_| machine.allows_sound("Nano_NanHello01", &mut random))
        .count();
    assert!(accepted > 0 && accepted < 50);
    assert!(machine.allows_sound("Nano_SFX_Dance01", &mut random));
    machine.set_low_stamina(true);
    machine.request_stand(&mut random);
    assert_eq!(machine.clip(), Some("discharge"));
    assert!(machine.force_stand_sound());
    let draws = random.draw_count();
    assert!(machine.allows_sound("Nano_NanOOE01", &mut random));
    assert_eq!(draws, random.draw_count());
    machine.complete(&mut random);
    assert!(!machine.force_stand_sound());
    assert_eq!(machine.clip(), Some("discharge"));
    machine.set_low_stamina(false);
    machine.complete(&mut random);
    assert!(matches!(
        machine.clip(),
        Some("stand1" | "stand2" | "stand3")
    ));
}

#[test]
fn forced_npc_once_restarts_but_death_clamps() {
    assert!(LegacyNpcAnimationRole::Forced.restarts_after_completion("skill0", true));
    assert!(!LegacyNpcAnimationRole::Forced.restarts_after_completion("skill0", false));
    assert!(!LegacyNpcAnimationRole::Death.restarts_after_completion("death", true));
    assert_eq!(
        LegacyNpcAnimationRole::Melee.repeat(false),
        LegacyAnimationRepeat::Once
    );
    assert!(LegacyNpcAnimationRole::Melee.is_additive());
}

#[test]
fn nano_call_skill_and_stand_completion_are_owned_by_one_machine() {
    let mut random = LegacyNanoStandRandomStream::with_seed(7);
    let mut machine = LegacyNanoAnimationMachine::default();
    machine.set_passive_skill(Some(LegacyNanoAnimationMode::Skill2));
    machine.request(
        LegacyNanoAnimationMode::Call,
        "call",
        LegacyAnimationBlend::CrossFade100Ms,
    );
    machine.complete(&mut random);
    assert_eq!(machine.mode(), LegacyNanoAnimationMode::Skill2);
    assert_eq!(machine.clip(), Some("skill2"));
    machine.complete(&mut random);
    assert_eq!(machine.mode(), LegacyNanoAnimationMode::Stand);
    assert!(matches!(
        machine.clip(),
        Some("stand1" | "stand2" | "stand3")
    ));
}

#[test]
fn dance_restarts_without_falling_into_stand() {
    let mut random = LegacyNanoStandRandomStream::with_seed(11);
    let mut machine = LegacyNanoAnimationMachine::default();
    machine.request(
        LegacyNanoAnimationMode::Emote,
        "dance5",
        LegacyAnimationBlend::CrossFade300Ms,
    );
    let serial = machine.request_serial();
    machine.complete(&mut random);
    assert_eq!(machine.clip(), Some("dance5"));
    assert_eq!(machine.blend(), LegacyAnimationBlend::Immediate);
    assert!(machine.request_serial() > serial);
    assert_eq!(random.draw_count(), 0);
}
