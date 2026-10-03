use super::*;

#[derive(Component)]
pub(super) struct NetworkHnpcIdleEventCursor0104 {
    pub(super) revision: u64,
    pub(super) seek: f32,
    pub(super) completions: u32,
}

/// Semantic idle request. `NpcAnimation.SetStandMotion` resolves it to one of
/// the model's stand clips and chooses again at every authored `end` event.
pub(super) const NETWORK_NPC_IDLE_REQUEST_0104: &str = "stand1";

/// Last sampled position of the applied idle clip, so its authored `end`
/// event is consumed once per crossing, including natural loop wraps.
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(super) struct NetworkNpcIdleEventCursor0104 {
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}
