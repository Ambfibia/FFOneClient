use super::*;

/// Exact ordinary-world attack attributes projected from one XDT weapon row.
///
/// The delay is the clean client's `m_iDelayTime * 0.1` cooldown. Keeping it
/// attached to the authoritative hand item prevents a missing tutorial-only
/// profile from emitting one combat packet per rendered frame and tripping
/// OpenFusion's rapid-fire disconnect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerWeaponCombatProfile {
    pub attack_half_angle_degrees: f32,
    pub attack_range: f32,
    pub blast_radius: f32,
    pub attack_cooldown_seconds: f32,
    pub target_capacity: usize,
    pub target_mode: LegacyWeaponTargetMode,
    /// Exact `m_iEffect1` passed to `cnAvatarAttack.MakeBullet` while the
    /// ordinary weapon battery is empty.
    pub effect1_bullet_type: i32,
    /// Exact `m_iEffect2`. The clean client selects it only while
    /// `cnOwnAvatarStatus.iBatteryWpn > 0` and this value is non-zero.
    pub effect2_bullet_type: i32,
    /// Exact lifetime used by `cnWarHead`, in seconds
    /// (`m_iDurationTime * 0.1`). Ordinary `BulletMoveScript` projectiles use
    /// their BulletTable lifetime instead.
    pub warhead_duration_seconds: f32,
    /// Exact grenade launch velocity (`m_iDeliverTime * 0.5`) applied on the
    /// clean client's vertical axis before gravity.
    pub grenade_initial_vertical_speed: f32,
}

impl PlayerWeaponCombatProfile {
    /// Reproduces the clean `cnAvatarAttack.MakeBullet` effect selection.
    #[must_use]
    pub const fn bullet_type(self, weapon_battery: i32) -> i32 {
        if weapon_battery > 0 && self.effect2_bullet_type != 0 {
            self.effect2_bullet_type
        } else {
            self.effect1_bullet_type
        }
    }

    /// Resolves the five dangling clean-XDT weapon references to the nearest
    /// complete primary BulletTable family used by weapons with the same
    /// equip type, target mode, range and attack-sound pair.
    ///
    /// Both the clean primary and the alternate donor stop at BulletTable row
    /// 169 even though eight primary weapon items reference 170..174. Keeping
    /// [`Self::bullet_type`] exact preserves that source defect for audits;
    /// this explicitly named presentation extension prevents those obtainable
    /// weapons from silently attacking without a moving native projectile.
    #[must_use]
    pub const fn native_presentation_bullet_type(self, weapon_battery: i32) -> i32 {
        match self.bullet_type(weapon_battery) {
            // Rifle target-mode 4, range 16 m, sounds 11/36.
            170 => 13,
            171 => 152,
            // Pistol target-mode 2, range 12 m, sounds 9/34. The missing
            // source row is selected for both battery states, so retain one
            // stable charged-family presentation in both states.
            172 => 164,
            // Rifle target-mode 3, range 15 m, sounds 26/51.
            173 => 136,
            174 => 165,
            bullet_type => bullet_type,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlayerWeaponAttackSounds {
    pub(super) normal: Vec<String>,
    pub(super) charged: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialPlayerPresentationResolution {
    AnimationContractResolved(ContractResolvedTutorialPlayerAnimation),
    AnimationDelay(Duration),
    AnimationBlocked {
        request: TutorialPlayerAnimationRequest,
        reason: TutorialPlayerRigCapabilityError,
    },
    /// Forwarded to a future attachment consumer; this does not claim that a
    /// visible item was attached.
    EquipmentForwarded(TutorialPlayerEquipmentRequest),
}

/// FIFO contract gate for a concrete selected-player rig. Renderer code may
/// consume `AnimationContractResolved`; constructing this value alone never
/// claims visual playback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialPlayerPresentationConsumer {
    pub(super) capabilities: TutorialPlayerRigCapabilities,
}

impl TutorialPlayerPresentationConsumer {
    #[must_use]
    pub const fn new(capabilities: TutorialPlayerRigCapabilities) -> Self {
        Self { capabilities }
    }

    #[must_use]
    pub const fn capabilities(&self) -> &TutorialPlayerRigCapabilities {
        &self.capabilities
    }

    pub fn consume_next(
        &self,
        queue: &mut TutorialPlayerPresentationCommandQueue,
    ) -> Option<TutorialPlayerPresentationResolution> {
        match queue.pop_front()? {
            TutorialPlayerPresentationCommand::Animation(request) => {
                Some(match self.capabilities.resolve(request) {
                    Ok(resolved) => {
                        TutorialPlayerPresentationResolution::AnimationContractResolved(resolved)
                    }
                    Err(reason) => {
                        TutorialPlayerPresentationResolution::AnimationBlocked { request, reason }
                    }
                })
            }
            TutorialPlayerPresentationCommand::AnimationDelay(duration) => Some(
                TutorialPlayerPresentationResolution::AnimationDelay(duration),
            ),
            TutorialPlayerPresentationCommand::Equipment(request) => Some(
                TutorialPlayerPresentationResolution::EquipmentForwarded(request),
            ),
        }
    }

    /// Resolves every command currently pending in one renderer frame.
    ///
    /// Retrobution's `AvatarAttack` starts `attack1` and `attack1upper` in the
    /// same Unity call stack. Draining only one FIFO item per frame introduces
    /// a visible one-frame layer desynchronization.
    pub fn consume_available(
        &self,
        queue: &mut TutorialPlayerPresentationCommandQueue,
    ) -> Vec<TutorialPlayerPresentationResolution> {
        std::iter::from_fn(|| self.consume_next(queue)).collect()
    }
}
