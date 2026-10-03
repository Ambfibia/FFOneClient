use super::*;

pub(super) fn advance_tutorial_nano_orbit(
    lateral_offset: Vec3,
    orbit: TutorialNanoOrbitPattern,
    orbit_speed_radians: f32,
    delta_seconds: f32,
) -> Vec3 {
    let orbit_delta = orbit_speed_radians * delta_seconds;
    match orbit {
        // Unity/native X reflection reverses every positive Unity yaw.
        TutorialNanoOrbitPattern::Clockwise => Quat::from_rotation_y(-orbit_delta) * lateral_offset,
        TutorialNanoOrbitPattern::CounterClockwise => {
            Quat::from_rotation_y(orbit_delta) * lateral_offset
        }
        TutorialNanoOrbitPattern::None => lateral_offset,
    }
}

pub(super) fn advance_tutorial_nano_skill_cooldown(
    time: Res<Time>,
    mut state: ResMut<TutorialNanoGameplayState>,
) {
    state.advance_skill_cooldown(time.delta_secs());
}

/// Exact `NanoMoveController` update, intentionally without clamping the
/// `deltaTime * 3` blend factor.
#[must_use]
pub fn advance_tutorial_nano_follow(current: Vec3, target: Vec3, delta_seconds: f32) -> Vec3 {
    current * (1.0 - delta_seconds * TUTORIAL_NANO_FOLLOW_RATE)
        + target * (delta_seconds * TUTORIAL_NANO_FOLLOW_RATE)
}

pub fn apply_tutorial_nano_gameplay_commands(world: &mut World) {
    let mut pending = world
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .take_all();
    while let Some(command) = pending.pop_front() {
        match command {
            TutorialNanoGameplayCommand::Equip {
                nano_id,
                skill_id,
                stamina,
            } => equip_gameplay_nano(world, nano_id, skill_id, stamina, None),
            TutorialNanoGameplayCommand::EquipWorld {
                nano_id,
                skill_id,
                stamina,
                presentation,
            } => equip_gameplay_nano(world, nano_id, skill_id, stamina, Some(presentation)),
            TutorialNanoGameplayCommand::Summon { owner } => summon_gameplay_nano(world, owner),
            TutorialNanoGameplayCommand::Dismiss => dismiss_gameplay_nano(world, true, true),
            TutorialNanoGameplayCommand::Withdraw => withdraw_gameplay_nano(world),
            TutorialNanoGameplayCommand::Hide => dismiss_gameplay_nano(world, true, false),
            TutorialNanoGameplayCommand::UseSkill { owner, targets } => {
                use_gameplay_nano_skill(world, owner, targets)
            }
            TutorialNanoGameplayCommand::PlayWorldSkill { owner } => {
                play_world_nano_skill(world, owner)
            }
        }
    }
}
