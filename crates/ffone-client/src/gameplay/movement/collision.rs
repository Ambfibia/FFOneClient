use super::*;

pub(crate) const LEGACY_COLLISION_SIDES: u8 = 1;

pub(crate) const LEGACY_COLLISION_ABOVE: u8 = 2;

pub(crate) const LEGACY_COLLISION_BELOW: u8 = 4;

/// The local player exists, but authored world collision has not completed its
/// first ground sample yet. `grounded == false` during this phase is a loading
/// sentinel, not a real airborne/jump state, so animation state machines must
/// hold their grounded idle pose until this marker is removed.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LegacyWorldColliderPending;

/// Collision ownership for the initial native slice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LegacyCollisionMode {
    /// A future/native collision system is responsible for updating
    /// [`LegacyPlayerController::grounded`].
    External,
    /// Explicit development-only flat ground. This is not a claim of world
    /// collision parity and should be removed per tile once native colliders
    /// are loaded.
    PlaceholderGroundPlane { height: f32 },
}
