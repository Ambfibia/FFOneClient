use super::*;

pub(in super::super) fn project_nano_skill_cooldown(
    client_state: ClientState,
    runtime_slots: &[RuntimeNanoSlot; 3],
    loadout: Option<TutorialNanoGameplayLoadout>,
    tutorial_remaining_fraction: Option<f32>,
    world_remaining_fractions: [Option<f32>; 3],
    nano_wheel: &mut NanoWheelTransientUi,
) {
    let mut projected = [None; 3];
    match client_state {
        ClientState::Tutorial => {
            let remaining_fraction = tutorial_remaining_fraction
                .filter(|remaining| remaining.is_finite() && *remaining > 0.0 && *remaining <= 1.0);
            if let (Some(loadout), Some(remaining_fraction)) = (loadout, remaining_fraction)
                && let Some(index) = runtime_slots.iter().position(|slot| {
                    slot.nano_id == Some(loadout.nano_id) && slot.skill_id == loadout.skill_id
                })
            {
                projected[index] = Some(remaining_fraction);
            }
        }
        ClientState::World => {
            for (projected, remaining) in projected.iter_mut().zip(world_remaining_fractions) {
                *projected = remaining.filter(|remaining| {
                    remaining.is_finite() && *remaining > 0.0 && *remaining <= 1.0
                });
            }
        }
        _ => {}
    }
    for (slot, remaining) in nano_wheel.slots.iter_mut().zip(projected) {
        // Gumball state has a separate packet/source owner and must survive
        // this projection untouched.
        slot.active_skill_cooldown_remaining = remaining;
    }
}
