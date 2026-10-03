use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LegacyMoveMode {
    #[default]
    None,
    Skill,
    Other,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LegacyWeaponTargetMode {
    #[default]
    Normal,
    Rocket,
    Grenade,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LegacyTargetSelection {
    pub source_connected: bool,
    pub focused_npc: Option<LegacyFocusedTarget>,
    pub focused_player: Option<LegacyFocusedTarget>,
    pub trigger: Option<Entity>,
    pub check_attack_target: bool,
    pub attack_targets: Vec<LegacyAttackTarget>,
    pub nano_targets: Vec<LegacyAttackTarget>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LegacyLocomotionState {
    Stun,
    Dash,
    DashAir,
    #[default]
    Stand,
    Ready,
    Run,
    RunBack,
    JumpStart,
    Jump,
    Landing,
    LandingRun,
    Swim,
    SwimBack,
    SwimIdle,
    SwimLeft,
    SwimRight,
    Slide,
    RopeDown,
    RopeDrop,
    RopeLeft,
    RopeRight,
    RopeStand1,
    RopeStand2,
    RopeTurn,
    RopeUp,
    BroomStick,
    Inventory,
    BoardInventory,
    ScooterInventory,
}

impl LegacyLocomotionState {
    #[must_use]
    pub fn clip(self) -> LegacyVisualClip {
        match self {
            Self::Stun => LegacyVisualClip::Stun,
            Self::Dash => LegacyVisualClip::Dash,
            Self::DashAir => LegacyVisualClip::DashAir,
            Self::Stand => LegacyVisualClip::Stand1,
            Self::Ready => LegacyVisualClip::Ready,
            Self::Run => LegacyVisualClip::Run,
            Self::RunBack => LegacyVisualClip::RunBack,
            Self::JumpStart => LegacyVisualClip::JumpStart,
            Self::Jump => LegacyVisualClip::Jump,
            Self::Landing => LegacyVisualClip::JumpEnd,
            Self::LandingRun => LegacyVisualClip::JumpLandRun,
            Self::Swim => LegacyVisualClip::Swim,
            Self::SwimBack => LegacyVisualClip::SwimBack,
            Self::SwimIdle => LegacyVisualClip::SwimIdle,
            Self::SwimLeft => LegacyVisualClip::SwimLeft,
            Self::SwimRight => LegacyVisualClip::SwimRight,
            Self::Slide => LegacyVisualClip::Slide,
            Self::RopeDown => LegacyVisualClip::RopeDown,
            Self::RopeDrop => LegacyVisualClip::RopeDrop,
            Self::RopeLeft => LegacyVisualClip::RopeLeft,
            Self::RopeRight => LegacyVisualClip::RopeRight,
            Self::RopeStand1 => LegacyVisualClip::RopeStand1,
            Self::RopeStand2 => LegacyVisualClip::RopeStand2,
            Self::RopeTurn => LegacyVisualClip::RopeTurn,
            Self::RopeUp => LegacyVisualClip::RopeUp,
            Self::BroomStick => LegacyVisualClip::Mount1,
            Self::Inventory => LegacyVisualClip::Inventory,
            Self::BoardInventory => LegacyVisualClip::BoardInventory,
            Self::ScooterInventory => LegacyVisualClip::ScooterInventory,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct LegacyAvatarActionState {
    pub locomotion: LegacyLocomotionState,
    pub upper_action: Option<LegacyVisualClip>,
    pub target_selection: LegacyTargetSelection,
    /// Clamp animation currently owning the stationary full-body/base layer.
    /// Directional locomotion interrupts this base immediately while the
    /// independent upper attack is allowed to finish.
    pub(super) base_action: Option<LegacyVisualClip>,
    pub(super) death_phase: LegacyAvatarDeathPhase,
    pub(super) visual_initialized: bool,
    /// A confirmed Hand change owns Ready until its first rendered cycle ends.
    pub(super) weapon_change_visual_active: bool,
    pub(super) was_grounded: bool,
    pub(super) was_in_combat: bool,
    pub(super) attack_sequence: u32,
    pub(super) attack_generation: u64,
    pub(super) attack_accumulated: f32,
    pub(super) seconds_since_attack: f32,
    pub(super) attack_delay_remaining: Option<f32>,
}

impl Default for LegacyAvatarActionState {
    fn default() -> Self {
        Self {
            locomotion: LegacyLocomotionState::Stand,
            upper_action: None,
            target_selection: LegacyTargetSelection::default(),
            base_action: None,
            death_phase: LegacyAvatarDeathPhase::Alive,
            visual_initialized: false,
            weapon_change_visual_active: false,
            was_grounded: true,
            was_in_combat: false,
            attack_sequence: 1,
            attack_generation: 0,
            attack_accumulated: 0.0,
            seconds_since_attack: f32::INFINITY,
            attack_delay_remaining: None,
        }
    }
}

impl LegacyAvatarActionState {
    pub fn interrupt_for_damage(&mut self) {
        self.weapon_change_visual_active = false;
        self.upper_action = None;
        self.base_action = None;
        self.attack_delay_remaining = None;
        self.attack_generation = self.attack_generation.wrapping_add(1);
        self.visual_initialized = false;
    }
    pub fn begin_weapon_change_visual(&mut self) {
        self.weapon_change_visual_active = true;
        self.base_action = None;
        self.upper_action = None;
        self.attack_delay_remaining = None;
        self.attack_generation = self.attack_generation.wrapping_add(1);
        self.visual_initialized = false;
    }

    pub fn weapon_change_visual_active(&self) -> bool {
        self.weapon_change_visual_active
    }
    /// Forces the next locomotion resolve to replay the current visual onto a
    /// newly promoted presentation rig. Authoritative apparel replacement can
    /// swap the visual root while the controller keeps the same locomotion
    /// state, so state equality alone cannot initialize that root.
    pub fn invalidate_visual(&mut self) {
        self.visual_initialized = false;
    }

    #[must_use]
    pub fn attack_generation(&self) -> u64 {
        self.attack_generation
    }

    #[must_use]
    pub fn attack_sequence(&self) -> u32 {
        self.attack_sequence
    }

    #[must_use]
    pub fn attack_accumulated(&self) -> f32 {
        self.attack_accumulated
    }

    #[must_use]
    pub const fn base_action(&self) -> Option<LegacyVisualClip> {
        self.base_action
    }

    /// One full-body semantic is authoritative at every renderer frame. The
    /// death sequence sits above stationary attacks and locomotion exactly as
    /// clean `AvatarDead` sits above `AvatarMove`/`DamageMotion`.
    #[must_use]
    pub fn authoritative_visual_clip(&self) -> LegacyVisualClip {
        match self.death_phase {
            LegacyAvatarDeathPhase::Dying => LegacyVisualClip::Die,
            LegacyAvatarDeathPhase::Dead => LegacyVisualClip::Death,
            LegacyAvatarDeathPhase::Alive => match self.base_action {
                Some(clip) => clip,
                None => self.locomotion.clip(),
            },
        }
    }
}
