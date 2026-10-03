//! Normal-world primary-attack projection into protocol-0104 requests.
//!
//! Target acquisition remains owned by `world_targeting`; this module owns
//! only the clean client's three outbound weapon-mode ABIs.

use bevy::prelude::*;
use ffone_protocol::{
    PcAttackNpcsRequest0104, PcGrenadeStyleFireRequest0104, PcRocketStyleFireRequest0104,
};

use crate::{
    avatar_action::{LegacyAttackTarget, LegacyTargetKind, LegacyWeaponTargetMode},
    coordinates::ProtocolPosition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldPrimaryAttack0104 {
    Hitscan(PcAttackNpcsRequest0104),
    Rocket(PcRocketStyleFireRequest0104),
    Grenade(PcGrenadeStyleFireRequest0104),
}

/// Builds the exact clean-client request for the selected weapon target mode.
///
/// Rocket and grenade packets use skill ID zero for weapon fire. Their target
/// position is the avatar's horizontal forward vector extended by the current
/// attack range, matching `cnAvatarAttack.MakeAttack`.
#[must_use]
pub fn build_world_primary_attack_0104(
    mode: LegacyWeaponTargetMode,
    actor_transform: &Transform,
    attack_range: f32,
    targets: &[LegacyAttackTarget],
    mut network_npc_id: impl FnMut(Entity) -> Option<i32>,
) -> Option<WorldPrimaryAttack0104> {
    match mode {
        LegacyWeaponTargetMode::Normal => {
            let npc_ids = targets
                .iter()
                .filter(|target| matches!(target.kind, LegacyTargetKind::Npc { .. }))
                .filter_map(|target| network_npc_id(target.entity))
                .take(PcAttackNpcsRequest0104::MAX_TARGETS)
                .collect();
            Some(WorldPrimaryAttack0104::Hitscan(PcAttackNpcsRequest0104 {
                npc_ids,
            }))
        }
        LegacyWeaponTargetMode::Rocket | LegacyWeaponTargetMode::Grenade => {
            if !actor_transform.translation.is_finite()
                || !actor_transform.rotation.is_finite()
                || !attack_range.is_finite()
                || attack_range < 0.0
            {
                return None;
            }
            let mut forward = actor_transform.rotation * Vec3::NEG_Z;
            forward.y = 0.0;
            let forward = forward.try_normalize().unwrap_or(Vec3::NEG_Z);
            let destination = actor_transform.translation + forward * attack_range;
            let destination = ProtocolPosition::from_native(destination).raw();
            match mode {
                LegacyWeaponTargetMode::Rocket => Some(WorldPrimaryAttack0104::Rocket(
                    PcRocketStyleFireRequest0104 {
                        skill_id: 0,
                        position: ProtocolPosition::from_native(actor_transform.translation).raw(),
                        destination,
                    },
                )),
                LegacyWeaponTargetMode::Grenade => Some(WorldPrimaryAttack0104::Grenade(
                    PcGrenadeStyleFireRequest0104 {
                        skill_id: 0,
                        destination,
                    },
                )),
                LegacyWeaponTargetMode::Normal => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests;
