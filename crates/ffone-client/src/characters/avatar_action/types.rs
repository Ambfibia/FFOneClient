use super::*;

/// Scripted movement owner currently driving the local avatar presentation.
///
/// The world traversal systems remain authoritative for position. This fact is
/// deliberately renderer-neutral so those systems can project their ownership
/// into the one avatar animation state machine without issuing animation
/// commands themselves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LegacyAvatarTraversalPresentation {
    #[default]
    None,
    Slope,
    Zipline,
    RopeDrop,
    RopeLeft,
    RopeRight,
    RopeStand1,
    RopeStand2,
    RopeTurn,
    RopeUp,
    BroomStick,
}

/// Presentation family of the vehicle the avatar is actually riding.
///
/// An equipped vehicle item is not sufficient to populate this value. The
/// runtime owner must set `Board` or `Scooter` only from confirmed mounted
/// state, keeping inventory equipment and active transport separate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LegacyVehiclePresentationFamily {
    #[default]
    None,
    Board,
    Scooter,
}

impl LegacyVehiclePresentationFamily {
    /// Clean `cnAvatarAnimation.StrModifyForVehicle` family table.
    #[must_use]
    pub const fn from_legacy_equip_type(equip_type: i32) -> Option<Self> {
        match equip_type {
            0 => Some(Self::None),
            1 => Some(Self::Board),
            2 | 3 => Some(Self::Scooter),
            _ => None,
        }
    }
}

/// Cross-system facts that select full-body local-avatar presentation.
///
/// This is an optional component at the FSM boundary: existing player spawns
/// retain normal locomotion until the world/UI owner attaches it. Its resolver
/// encodes the clean priority explicitly: scripted traversal, then inventory,
/// then the ordinary grounded/jump/swim locomotion table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Component)]
pub struct LegacyAvatarPresentationContext {
    pub traversal: LegacyAvatarTraversalPresentation,
    pub inventory_open: bool,
    pub mounted_vehicle: LegacyVehiclePresentationFamily,
}

impl LegacyAvatarPresentationContext {
    #[must_use]
    pub const fn authoritative_locomotion_override(self) -> Option<LegacyLocomotionState> {
        match self.traversal {
            LegacyAvatarTraversalPresentation::Slope => Some(LegacyLocomotionState::Slide),
            LegacyAvatarTraversalPresentation::Zipline => Some(LegacyLocomotionState::RopeDown),
            LegacyAvatarTraversalPresentation::RopeDrop => Some(LegacyLocomotionState::RopeDrop),
            LegacyAvatarTraversalPresentation::RopeLeft => Some(LegacyLocomotionState::RopeLeft),
            LegacyAvatarTraversalPresentation::RopeRight => Some(LegacyLocomotionState::RopeRight),
            LegacyAvatarTraversalPresentation::RopeStand1 => {
                Some(LegacyLocomotionState::RopeStand1)
            }
            LegacyAvatarTraversalPresentation::RopeStand2 => {
                Some(LegacyLocomotionState::RopeStand2)
            }
            LegacyAvatarTraversalPresentation::RopeTurn => Some(LegacyLocomotionState::RopeTurn),
            LegacyAvatarTraversalPresentation::RopeUp => Some(LegacyLocomotionState::RopeUp),
            LegacyAvatarTraversalPresentation::BroomStick => {
                Some(LegacyLocomotionState::BroomStick)
            }
            LegacyAvatarTraversalPresentation::None if self.inventory_open => {
                Some(match self.mounted_vehicle {
                    LegacyVehiclePresentationFamily::None => LegacyLocomotionState::Inventory,
                    LegacyVehiclePresentationFamily::Board => LegacyLocomotionState::BoardInventory,
                    LegacyVehiclePresentationFamily::Scooter => {
                        LegacyLocomotionState::ScooterInventory
                    }
                })
            }
            LegacyAvatarTraversalPresentation::None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyNanoTargetPolicy {
    /// XDT `m_iEffectTarget`: 1 focus, 3 cone, 5 caster area, 6 focus area.
    pub effect_target: i32,
    /// XDT `m_iTargetType`: 1 hostile NPC, 2 player/group, 3 self.
    pub target_type: i32,
    pub range: f32,
    pub area: f32,
    pub half_angle_degrees: f32,
    pub capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyTargetKind {
    Npc { team: i32 },
    Player,
}

impl LegacyTargetKind {
    pub(super) fn is_friendly_npc(self) -> bool {
        matches!(self, Self::Npc { team: 1 })
    }
}

/// Result of the native cone/range/LOS acquisition layer. The geometric
/// acquisition itself stays visibly disconnected until native world colliders
/// exist; this scheduler owns the exact legacy sorting and action semantics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyTargetSample {
    pub entity: Entity,
    pub kind: LegacyTargetKind,
    pub distance: f32,
    pub in_view: bool,
    /// True when the target passes the source-proven attack-angle test before
    /// any weapon-specific range is applied. Nano skills reuse the angle but
    /// supply their own range from `SkillTableElement`.
    pub in_attack_arc: bool,
    /// The active Nano skill owns its angle independently of the weapon.
    pub in_nano_arc: bool,
    pub in_attack_cone: bool,
    pub talk_enabled: bool,
    /// Native actor reference position and extents used by focused-area Nano
    /// overlays. They do not participate in ordinary weapon selection.
    pub position: [f32; 3],
    pub radius: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyTriggerSample {
    pub entity: Entity,
    pub radius: f32,
}

#[derive(Debug, Clone, Default, Component)]
pub struct LegacyAvatarTargetFeed {
    /// False means no native cone/raycast producer has populated this feed.
    pub source_connected: bool,
    pub samples: Vec<LegacyTargetSample>,
    pub trigger: Option<LegacyTriggerSample>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyFocusedTarget {
    pub entity: Entity,
    pub kind: LegacyTargetKind,
    pub distance: f32,
    pub talk_enabled: bool,
}

impl From<LegacyTargetSample> for LegacyFocusedTarget {
    fn from(sample: LegacyTargetSample) -> Self {
        Self {
            entity: sample.entity,
            kind: sample.kind,
            distance: sample.distance,
            talk_enabled: sample.talk_enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyAttackTarget {
    pub entity: Entity,
    pub kind: LegacyTargetKind,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyVisualCompletion {
    pub actor: Entity,
    /// The requested semantic clip, even when the driver used attack1 fallback.
    pub clip: LegacyVisualClip,
}

#[derive(Debug, Default, Resource)]
pub struct LegacyVisualCompletionQueue {
    pub(super) pending: VecDeque<LegacyVisualCompletion>,
}

impl LegacyVisualCompletionQueue {
    pub fn push(&mut self, completion: LegacyVisualCompletion) {
        self.pending.push_back(completion);
    }

    pub fn take_all(&mut self) -> VecDeque<LegacyVisualCompletion> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum LegacyAvatarDeathPhase {
    #[default]
    Alive,
    Dying,
    Dead,
}
