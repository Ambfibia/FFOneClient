use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TutorialNanoGameplayStatus {
    #[default]
    Absent,
    Loading,
    Ready,
    Blocked(String),
}

#[derive(Debug, Clone, Resource)]
pub struct TutorialNanoGameplayState {
    pub(super) loadout: Option<TutorialNanoGameplayLoadout>,
    pub(super) world_presentation: Option<WorldNanoGameplayPresentation>,
    pub(super) entity: Option<Entity>,
    pub(super) owner: Option<Entity>,
    pub(super) stamina: i32,
    pub(super) skill_cooldown_remaining_seconds: f32,
    pub(super) status: TutorialNanoGameplayStatus,
    pub(super) generation: u64,
    pub(super) activated_generation: Option<u64>,
    pub(super) asset_contract_ready: bool,
    pub(super) face_texture_bound: bool,
    pub(super) animation: LegacyNanoAnimationMachine,
    pub(super) applied_animation_request_serial: Option<u64>,
}

impl Default for TutorialNanoGameplayState {
    fn default() -> Self {
        Self {
            loadout: None,
            world_presentation: None,
            entity: None,
            owner: None,
            stamina: 0,
            skill_cooldown_remaining_seconds: 0.0,
            status: TutorialNanoGameplayStatus::Absent,
            generation: 0,
            activated_generation: None,
            asset_contract_ready: false,
            face_texture_bound: false,
            animation: LegacyNanoAnimationMachine::default(),
            applied_animation_request_serial: None,
        }
    }
}

impl TutorialNanoGameplayState {
    #[must_use]
    pub const fn loadout(&self) -> Option<TutorialNanoGameplayLoadout> {
        self.loadout
    }

    #[must_use]
    pub fn world_presentation(&self) -> Option<&WorldNanoGameplayPresentation> {
        self.world_presentation.as_ref()
    }

    #[must_use]
    pub const fn entity(&self) -> Option<Entity> {
        self.entity
    }

    #[must_use]
    pub const fn owner(&self) -> Option<Entity> {
        self.owner
    }

    #[must_use]
    pub const fn stamina(&self) -> i32 {
        self.stamina
    }

    #[must_use]
    pub const fn skill_cooldown_remaining_seconds(&self) -> f32 {
        self.skill_cooldown_remaining_seconds
    }

    /// Normalized `GameCondition` time remaining for `cnNanoWheel`: `1 -> 0`.
    /// Completed and malformed state fails closed to no HUD overlay.
    #[must_use]
    pub fn skill_cooldown_remaining_fraction(&self) -> Option<f32> {
        let remaining = self.skill_cooldown_remaining_seconds;
        if !remaining.is_finite() || remaining <= 0.0 {
            return None;
        }
        Some((remaining / TUTORIAL_BUTTERCUP_SKILL_COOLDOWN_SECONDS).clamp(0.0, 1.0))
    }

    #[must_use]
    pub const fn status(&self) -> &TutorialNanoGameplayStatus {
        &self.status
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.entity.is_some()
            && self.owner.is_some()
            && matches!(self.status, TutorialNanoGameplayStatus::Ready)
    }

    #[must_use]
    pub fn can_use_skill(&self) -> bool {
        self.is_active()
            && self.stamina > 0
            && self.skill_cooldown_remaining_seconds <= 0.0
            && self.loadout
                == Some(TutorialNanoGameplayLoadout {
                    nano_id: TUTORIAL_BUTTERCUP_NANO_ID,
                    skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
                })
    }

    pub(super) fn requires_exact_face_texture(&self) -> bool {
        self.loadout
            .is_some_and(|loadout| loadout.nano_id == TUTORIAL_BUTTERCUP_NANO_ID)
    }

    pub fn advance_skill_cooldown(&mut self, delta_seconds: f32) {
        if !delta_seconds.is_finite()
            || delta_seconds <= 0.0
            || self.skill_cooldown_remaining_seconds <= 0.0
        {
            return;
        }
        self.skill_cooldown_remaining_seconds =
            (self.skill_cooldown_remaining_seconds - delta_seconds).max(0.0);
    }

    pub(super) fn request_clip(&mut self, clip: &str) {
        let mode = match clip {
            CALL_CLIP => LegacyNanoAnimationMode::Call,
            "skill1" => LegacyNanoAnimationMode::Skill1,
            "skill2" => LegacyNanoAnimationMode::Skill2,
            "skill3" => LegacyNanoAnimationMode::Skill3,
            "win" => LegacyNanoAnimationMode::Win,
            "lose" => LegacyNanoAnimationMode::Lose,
            "tie" => LegacyNanoAnimationMode::Tie,
            "stand1" | "stand2" | "stand3" => LegacyNanoAnimationMode::Stand,
            _ => LegacyNanoAnimationMode::Emote,
        };
        self.animation
            .request(mode, clip, LegacyAnimationBlend::CrossFade100Ms);
        self.applied_animation_request_serial = None;
    }

    pub(super) fn mark_absent(&mut self) {
        self.entity = None;
        self.owner = None;
        self.status = TutorialNanoGameplayStatus::Absent;
        self.activated_generation = None;
        self.asset_contract_ready = false;
        self.face_texture_bound = false;
        self.animation.clear();
        self.applied_animation_request_serial = None;
    }
}
