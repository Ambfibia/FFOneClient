use super::*;

#[derive(Debug, Default, PartialEq)]
pub struct LegacyFrameSchedule {
    pub actions: Vec<LegacyAvatarActionIntent>,
    pub visuals: Vec<LegacyVisualCommand>,
    pub force_camera_angle: bool,
}

/// Pure per-frame action resolver. Its branch structure intentionally mirrors
/// the original `else if` chain; Nano remains an independent trailing `if`.
#[must_use]
pub fn schedule_legacy_action_frame(
    input: LegacyAvatarActionInput,
    context: &LegacyAvatarActionContext,
    selection: &LegacyTargetSelection,
    state: &mut LegacyAvatarActionState,
    bindings: &LegacyAvatarClipBindings,
    delta_seconds: f32,
) -> LegacyFrameSchedule {
    let mut schedule = LegacyFrameSchedule::default();
    if !context.ready_for_play || context.dead {
        return schedule;
    }

    advance_attack_timers(delta_seconds, context, state, &mut schedule);
    if context.move_mode != LegacyMoveMode::None
        || !context.input_enabled
        || context.system_popup
        || matches!(context.time_buff_condition, 5 | 6)
    {
        return schedule;
    }

    let primary_pressed = input.primary_held || input.primary_just_pressed;
    let mut primary_branch_claimed = false;
    if input.primary_just_pressed && !context.attack_locked {
        if let Some(trigger) = selection.trigger {
            primary_branch_claimed = true;
            schedule.actions.push(if context.vehicle_mounted {
                LegacyAvatarActionIntent::DismountVehicle
            } else {
                LegacyAvatarActionIntent::UseTrigger(trigger)
            });
        } else if !context.combat_condition
            && (selection.focused_npc.is_some() || selection.focused_player.is_some())
        {
            // This branch is claimed even when the focused hostile cannot be
            // talked to; the held branch starts on the following frame.
            primary_branch_claimed = true;
            if context.vehicle_mounted {
                schedule
                    .actions
                    .push(LegacyAvatarActionIntent::DismountVehicle);
            } else if let Some(npc) = selection.focused_npc {
                if !selection.check_attack_target && npc.talk_enabled {
                    schedule
                        .actions
                        .push(LegacyAvatarActionIntent::TalkNpc(npc.entity));
                }
            } else if let Some(player) = selection.focused_player
                && !context.attack_players_enabled
                && player.talk_enabled
            {
                schedule
                    .actions
                    .push(LegacyAvatarActionIntent::TalkPlayer(player.entity));
            }
        }
    }

    if !primary_branch_claimed
        && primary_pressed
        && !context.attack_locked
        && selection.trigger.is_none()
    {
        primary_branch_claimed = true;
        if context.overheat_allows_attack {
            if context.vehicle_mounted {
                schedule
                    .actions
                    .push(LegacyAvatarActionIntent::DismountVehicle);
            }
            let should_attack = if context.combat_condition {
                true
            } else if context.vehicle_mounted {
                false
            } else if selection.focused_npc.is_none() && selection.focused_player.is_none() {
                true
            } else if let Some(npc) = selection.focused_npc {
                selection.check_attack_target || !npc.talk_enabled
            } else if let Some(player) = selection.focused_player {
                context.attack_players_enabled || !player.talk_enabled
            } else {
                false
            };
            if should_attack {
                schedule_primary_attack(
                    delta_seconds,
                    context,
                    selection,
                    state,
                    bindings,
                    &mut schedule,
                );
            }
        }
    }

    if !primary_branch_claimed
        && input.weapon_cycle_just_pressed
        && !context.weapon_change_locked
        && !context.vehicle_mounted
        && !context.weapon_change_cooldown_blocked
        && !context.weapon_change_in_progress
        && context.weapon_swap_available
    {
        schedule.actions.push(LegacyAvatarActionIntent::WeaponCycle);
    }

    if input.nano_just_pressed && !context.nano_locked && !context.vehicle_mounted && context.nano_skill_usable {
        schedule.actions.push(LegacyAvatarActionIntent::NanoSkill {
            targets: selection.nano_targets.clone(),
        });
    }
    schedule
}
