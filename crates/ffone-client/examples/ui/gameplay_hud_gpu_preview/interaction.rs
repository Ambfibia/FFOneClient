use super::*;

#[derive(Default)]
pub(super) struct ScrollPointerProbe {
    pub(super) phase: u8,
    pub(super) destination: Vec2,
    pub(super) before: f32,
}
