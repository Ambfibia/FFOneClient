use super::*;

pub(super) const RETROBUTION_SWORD_TRAIL_LENGTH: usize = 30;

pub(super) const RETROBUTION_SWORD_TRAIL_MAX_SECONDS: f32 = 0.75;

pub(super) const RETROBUTION_SWORD_TRAIL_TINT: Vec4 = Vec4::new(1.0, 0.726_802_47, 0.953_917_74, 1.0);

// Retrobution draws character materials in Unity queue 2900 and the tutorial
// mesh effects in queues 3000/3011. Bevy otherwise reorders every AlphaMode::Blend
// mesh only by view depth. A queue step must dominate the complete tutorial
// camera depth range so a depth-writing additive slash cannot run before Jack
// and prevent his character surfaces from passing the later depth test.
// Keep the existing ambient-emission distance contract. This intentionally
// remains independent from the camera direction: turning the camera must not
// freeze a live Unity particle controller.
pub(super) const STREAMED_WORLD_EFFECT_FAR_MARGIN: f32 = 32.0;

pub(super) const MAX_STREAMED_WORLD_PARTICLES: usize = 2_048;
