use super::*;

pub(super) fn legacy_platform_world_translation(
    motion: &WorldPlatformMotion,
    timer: f32,
    returning: bool,
) -> Vec3 {
    if matches!(motion.move_type, 0..=3 | 6..=10) {
        let source_position = legacy_platform_source_position(motion, timer, returning);
        // Line and rotate-move modes use Start's unscaled `worldTo`.
        // Curves and waypoint paths explicitly multiply by localScale.x in
        // the clean script before applying the initial rotation.
        let scale = if matches!(motion.move_type, 1..=3 | 8..=10) {
            motion.initial_scale.x
        } else {
            1.0
        };
        motion.initial_translation + motion.initial_rotation * (source_position * scale)
    } else {
        motion.initial_translation
    }
}
