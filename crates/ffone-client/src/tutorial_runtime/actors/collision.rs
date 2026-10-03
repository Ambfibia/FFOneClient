use super::*;

/// Vertical state retained by the original `NpcMoveController`.
///
/// Retrobution probes from `root + height` down to `root - 10` every frame.
/// A root below the hit is snapped up immediately; a root above it accelerates
/// down at 10 units/s² and is clamped to 10 units/s until it reaches the hit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Component)]
pub struct TutorialActorGrounding {
    pub(super) vertical_velocity: f32,
}

#[must_use]
pub(super) fn advance_tutorial_actor_grounding(
    current_y: f32,
    ground_y: Option<f32>,
    vertical_velocity: f32,
    delta_seconds: f32,
) -> (f32, f32) {
    const GRAVITY: f32 = 10.0;
    const TERMINAL_FALL_SPEED: f32 = -10.0;

    if !current_y.is_finite()
        || !vertical_velocity.is_finite()
        || !delta_seconds.is_finite()
        || delta_seconds <= 0.0
    {
        return (current_y, vertical_velocity);
    }
    if let Some(ground_y) = ground_y.filter(|height| height.is_finite()) {
        if ground_y >= current_y {
            return (ground_y, 0.0);
        }
        let next_velocity = (vertical_velocity - GRAVITY * delta_seconds).max(TERMINAL_FALL_SPEED);
        let next_y = current_y + next_velocity * delta_seconds;
        if next_y <= ground_y {
            return (ground_y, 0.0);
        }
        return (next_y, next_velocity);
    }

    let next_velocity = (vertical_velocity - GRAVITY * delta_seconds).max(TERMINAL_FALL_SPEED);
    (current_y + next_velocity * delta_seconds, next_velocity)
}
