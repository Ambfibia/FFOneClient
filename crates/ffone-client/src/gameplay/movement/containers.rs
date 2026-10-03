use super::*;

pub(super) fn bevy_mouse_delta_to_unity(delta: Vec2) -> Vec2 {
    Vec2::new(delta.x, -delta.y)
}
