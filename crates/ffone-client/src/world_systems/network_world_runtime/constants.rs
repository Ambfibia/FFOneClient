
pub(super) const CONSOLIDATED_TABLE: &str = "npc_imports_consolidated";

pub(super) const ARRIVAL_EPSILON: f32 = 0.001;

/// Low-layer states an additive melee/wound can be composed over.
pub(super) const NETWORK_NPC_LOW_LAYER_CLIPS_0104: [&str; 7] = [
    "ready", "stand1", "stand2", "stand3", "stand4", "walk", "run",
];

/// Mask group for targets which a high-layer clip animates but a low-layer
/// clip does not; Bevy would otherwise write the bare difference to them.
pub(super) const NETWORK_NPC_UNCOVERED_HIGH_LAYER_GROUP_0104: u32 = 0;
