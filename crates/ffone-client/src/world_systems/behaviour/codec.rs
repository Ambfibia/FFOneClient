use super::*;

pub(super) const WORLD_SCRIPTED_EFFECTS_PER_FRAME: usize = 2;

// World behaviour documents can describe several thousand controller records
// and more than seven thousand animation targets for one tile. JSON decoding
// already happens off-thread, but materializing that whole graph through
// `Commands` in one Update used to turn completion of the async task into a
// main-thread frame spike. Count entity binding and authored controller work
// explicitly and share one deterministic budget across all streamed roots.
pub(super) const WORLD_BEHAVIOUR_WORK_PER_FRAME: usize = 96;

pub(super) const WORLD_BEHAVIOUR_WORK_PER_ROOT_PER_FRAME: usize = 32;

pub(super) const WORLD_ANIMATION_BINDINGS_PER_FRAME: usize = 8;

pub(super) const WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_FRAME: usize = 64;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WorldSurfacePacketClock(pub f32);
