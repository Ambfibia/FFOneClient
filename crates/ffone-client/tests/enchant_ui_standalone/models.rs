use super::*;

pub(super) fn awaiting_weapon_model() -> EnchantModeModel0104 {
    let mut model = EnchantModeModel0104::default();
    model.open(10_000, true);
    model.clear_intents();
    attach_ready_weapon(&mut model);
    model.clear_intents();
    model.activate_enchant().unwrap();
    model.accept_system_message().unwrap();
    model.advance(ENCHANT_WAIT_SECONDS_0104).unwrap();
    assert!(matches!(model.phase(), EnchantPhase0104::Waiting { .. }));
    model.advance(0.001).unwrap();
    assert!(matches!(
        model.phase(),
        EnchantPhase0104::AwaitingReply { .. }
    ));
    model
}
