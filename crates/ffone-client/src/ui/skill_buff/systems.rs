use super::*;

pub(super) fn advance_skill_buff_cash_timer(time: Res<Time>, mut model: ResMut<SkillBuffUiModel>) {
    model.advance_cash_timer(time.delta_secs());
}

pub(super) fn sync_skill_buff_nano_gumballs(
    model: Res<SkillBuffUiModel>,
    transient: Option<ResMut<NanoWheelTransientUi>>,
) {
    if !model.is_changed() {
        return;
    }
    let Some(mut transient) = transient else {
        return;
    };
    transient.set_gumball_effect(-1);
    for (slot, flag) in STIM_FLAGS.into_iter().enumerate() {
        if model.local_condition_bit_flag & flag != 0 {
            transient.set_gumball_effect(slot as i32);
        }
    }
}
