//! Server-owned movement modifiers shared by active and passive Nano buffs.
use crate::app::*;
use ffone_protocol::PcBuffUpdate0104;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MovementBuffs {
    speed: i32,
    slow: i32,
    jump: i32,
    nano_power_tiers: [i32; 3],
}

impl MovementBuffs {
    pub(super) fn apply(&mut self, update: PcBuffUpdate0104) {
        let value = match update.update_kind {
            1 | 3 => update.time_buff.value,
            2 => 0,
            _ => return,
        };
        match update.buff_id {
            1 => self.speed = value,
            3 => self.jump = value,
            8 => self.slow = value,
            21..=23 => self.nano_power_tiers[(update.buff_id - 21) as usize] = value,
            _ => {}
        }
    }

    pub(super) fn rocket_jump_bonus(
        &self,
        slot: usize,
        skill: &GameplaySkillUiDefinition,
    ) -> Option<i32> {
        if !matches!(skill.skill_type, 29 | 40) || !skill.active || skill.effect_type != 1 || skill.target != 2 {
            return None;
        }
        let tier = usize::try_from(*self.nano_power_tiers.get(slot)?).ok()?;
        skill
            .values_a
            .get(tier)
            .copied()
            .filter(|value| *value >= 0)
    }

    fn attributes(self, base_run: i32, base_jump: i32, mounted: bool) -> (i32, i32) {
        let run = (i64::from(base_run) + i64::from(self.speed) - i64::from(self.slow))
            .clamp(0, i64::from(i32::MAX)) as i32;
        let jump = (i64::from(base_jump) + if mounted { 0 } else { i64::from(self.jump) })
            .clamp(0, i64::from(i32::MAX)) as i32;
        (run, jump)
    }
}

#[derive(Component)]
pub(super) struct MovementBuffBase {
    run: i32,
    jump: i32,
}

pub(super) fn launch_nano_rocket(
    controller: &mut LegacyPlayerController,
    base: Option<&MovementBuffBase>,
    buffs: &MovementBuffs,
    slot: usize,
    skill: &GameplaySkillUiDefinition,
) -> bool {
    let Some(bonus) = buffs.rocket_jump_bonus(slot, skill) else {
        return false;
    };
    // Rocket uses the unmodified jump attribute, not the passive Jump bonus.
    let base_jump = base.map_or(controller.jump_height_server_units, |base| base.jump);
    let airborne = !controller.grounded;
    let launched = controller.launch_nano_rocket((base_jump as f32 + bonus as f32) * 0.01);
    if launched && skill.skill_type == 40 {
        controller.launch_nano_dash_from(airborne);
    }
    launched
}

/// Server GM changes replace the unmodified run attribute, so subsequent buff
/// projection and removal retain the authoritative speed.
pub(super) fn apply_gm_speed(
    controller: &mut LegacyPlayerController,
    base: Option<&mut MovementBuffBase>,
    speed: i32,
) {
    if let Some(base) = base {
        base.run = speed;
    }
    // Before the first projection there is no base yet; it will be captured
    // from this controller by sync_movement_buffs.
    controller.run_speed_server_units = speed;
}

/// Retain the server GM jump attribute when Nano buffs are projected or removed.
pub(super) fn apply_gm_jump(
    controller: &mut LegacyPlayerController,
    base: Option<&mut MovementBuffBase>,
    jump: i32,
) {
    if let Some(base) = base {
        base.jump = jump;
    }
    controller.jump_height_server_units = jump;
}

/// Capture unmodified controller attributes once per player, then project
/// absolute buff values each frame. A refresh must never compound a bonus.
pub(super) fn sync_movement_buffs(
    mut commands: Commands,
    buffs: Res<MovementBuffs>,
    vehicle: Res<LocalVehiclePresentationRuntime>,
    conditions: Option<Res<SkillBuffUiModel>>,
    inventory: Option<Res<LocalInventoryRuntime>>,
    content: Option<Res<TutorialMissionContent>>,
    mut players: Query<
        (
            Entity,
            &mut LegacyPlayerController,
            Option<&MovementBuffBase>,
        ),
        With<LocalPlayer>,
    >,
) {
    for (entity, mut controller, base) in &mut players {
        controller.incapacitated = conditions.as_ref().is_some_and(|buffs|
            buffs.local_control_condition() != 0);
        let (base_run, base_jump) = base.map_or_else(
            || {
                let run = controller.run_speed_server_units;
                let jump = controller.jump_height_server_units;
                commands
                    .entity(entity)
                    .insert(MovementBuffBase { run, jump });
                (run, jump)
            },
            |base| (base.run, base.jump),
        );
        let mounted = vehicle.family != LegacyVehiclePresentationFamily::None;
        let vehicle_speed = mounted
            .then(|| {
                let item = inventory.as_ref()?.snapshot()?.equipment()
                    [ffone_protocol::CharacterEquipSlot0104::Vehicle as usize];
                content.as_ref()?.gameplay_vehicle_speed(item.item_id)
            })
            .flatten();
        controller.set_vehicle_speed(vehicle_speed);
        let (mut run, jump) = buffs.attributes(
            base_run,
            base_jump,
            vehicle.family != LegacyVehiclePresentationFamily::None,
        );
        if let Some(speed) = vehicle_speed {
            run = speed.saturating_sub(buffs.slow).max(0);
        }
        // Skill HIT and reconnect post-state can precede PC_BUFF_UPDATE.
        // Apply Snare immediately from the authoritative condition mask;
        // once its value arrives, the normal absolute modifier takes over.
        if buffs.slow == 0 && conditions.as_ref().is_some_and(|conditions|
            conditions.local_condition_bit_flag & 0x80 != 0) {
            run /= 2;
        }
        if controller.run_speed_server_units != run {
            controller.run_speed_server_units = run;
        }
        if controller.jump_height_server_units != jump {
            controller.jump_height_server_units = jump;
        }
    }
}

#[cfg(test)]
mod tests;
