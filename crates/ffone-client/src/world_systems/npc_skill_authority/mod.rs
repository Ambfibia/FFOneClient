//! Pure authoritative projection for protocol-0104 NPC skill-hit packets.
//!
//! Both variable packet bodies are decoded and semantically validated before
//! any projection is returned. Runtime/Bevy mutation stays with the caller so
//! malformed records cannot partially update local or remote combat state.

use std::fmt;

use ffone_protocol::{
    NanoSkillResult0104, NanoSkillTarget0104, NpcSkillAuthorityDecodeError0104,
    NpcSkillAuthorityPacket0104, NpcSkillCorruptionResult0104, decode_npc_skill_authority_0104,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldNpcSkillEntityKind0104 {
    Player,
    Npc,
    Mob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNpcSkillEntity0104 {
    pub kind: WorldNpcSkillEntityKind0104,
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldNpcSkillCastKind0104 {
    Skill { skill_type: i32 },
    Corruption { style: i16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNpcSkillCasterPostState0104 {
    pub npc_id: i32,
    pub skill_id: i16,
    pub position: [i32; 3],
    /// Present for an ordinary Bloodsucking/Leech result. OpenFusion stores
    /// source HP in the nested heal record while copying the damaged target's
    /// identity into that record, so the fixed prefix owns this HP.
    pub absolute_hp: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNpcSkillMovePostState0104 {
    pub map_number: i32,
    pub position: [i32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldNpcSkillSourceResult0104 {
    Skill(NanoSkillResult0104),
    Corruption(NpcSkillCorruptionResult0104),
}

/// One server-ordered target projection. `None` means the packet family does
/// not author that field; callers must never derive absolute state from the
/// effect amounts retained in `source_result`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNpcSkillTargetPostState0104 {
    pub target: WorldNpcSkillEntity0104,
    pub absolute_hp: Option<i32>,
    pub absolute_condition_bit_flag: Option<i32>,
    pub absolute_weapon_battery: Option<i32>,
    pub absolute_nano_battery: Option<i32>,
    pub absolute_active_nano_slot: Option<i16>,
    pub absolute_nano_id: Option<i16>,
    pub absolute_nano_stamina: Option<i16>,
    pub nano_deactivated: Option<bool>,
    pub condition_status_deleted: Option<i32>,
    pub movement: Option<WorldNpcSkillMovePostState0104>,
    pub source_result: WorldNpcSkillSourceResult0104,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldNpcSkillAuthoritativeProjection0104 {
    pub kind: WorldNpcSkillCastKind0104,
    pub caster: WorldNpcSkillCasterPostState0104,
    pub targets: Vec<WorldNpcSkillTargetPostState0104>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldNpcSkillProjectionError0104 {
    Protocol(NpcSkillAuthorityDecodeError0104),
    InvalidNpcId(i32),
    InvalidSkillId(i16),
    InvalidCorruptionStyle(i16),
    NegativeValue {
        field: &'static str,
        value: i32,
    },
    InvalidBoolean {
        field: &'static str,
        value: i32,
    },
    UnsupportedEntityType(i32),
    InvalidTargetId(i32),
    PlayerTargetRequired {
        context: &'static str,
        target: WorldNpcSkillEntity0104,
    },
    LeechTargetMismatch {
        heal: NanoSkillTarget0104,
        damage: NanoSkillTarget0104,
    },
    InvalidCorruptionHitFlag(i8),
    InvalidActiveNanoSlot(i16),
}

impl From<NpcSkillAuthorityDecodeError0104> for WorldNpcSkillProjectionError0104 {
    fn from(error: NpcSkillAuthorityDecodeError0104) -> Self {
        Self::Protocol(error)
    }
}

impl fmt::Display for WorldNpcSkillProjectionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => error.fmt(formatter),
            Self::InvalidNpcId(id) => write!(formatter, "invalid NPC skill caster ID {id}"),
            Self::InvalidSkillId(id) => write!(formatter, "invalid NPC skill ID {id}"),
            Self::InvalidCorruptionStyle(style) => {
                write!(
                    formatter,
                    "invalid corruption style {style}; expected 0..=2"
                )
            }
            Self::NegativeValue { field, value } => write!(
                formatter,
                "NPC skill result field {field} cannot be negative: {value}"
            ),
            Self::InvalidBoolean { field, value } => write!(
                formatter,
                "NPC skill result field {field} must be 0 or 1, got {value}"
            ),
            Self::UnsupportedEntityType(entity_type) => write!(
                formatter,
                "NPC skill result has unsupported protocol-0104 eCT {entity_type}"
            ),
            Self::InvalidTargetId(id) => write!(formatter, "invalid NPC skill target ID {id}"),
            Self::PlayerTargetRequired { context, target } => {
                write!(
                    formatter,
                    "{context} requires a player target, got {target:?}"
                )
            }
            Self::LeechTargetMismatch { heal, damage } => write!(
                formatter,
                "OpenFusion NPC Leech heal/damage target identities diverge: {heal:?} vs {damage:?}"
            ),
            Self::InvalidCorruptionHitFlag(hit_flag) => write!(
                formatter,
                "invalid corruption hit flag {hit_flag}; expected style win/tie/lose"
            ),
            Self::InvalidActiveNanoSlot(slot) => write!(
                formatter,
                "invalid corruption target active Nano slot {slot}; expected -1..=2"
            ),
        }
    }
}

impl std::error::Error for WorldNpcSkillProjectionError0104 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            _ => None,
        }
    }
}

/// Strictly decodes and validates `NPC_SKILL_HIT` or
/// `NPC_SKILL_CORRUPTION_HIT`. Unknown packet IDs remain passthrough.
pub fn decode_world_npc_skill_authority_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<WorldNpcSkillAuthoritativeProjection0104>, WorldNpcSkillProjectionError0104> {
    let Some(packet) = decode_npc_skill_authority_0104(packet_type, payload)? else {
        return Ok(None);
    };
    project_world_npc_skill_authority_0104(&packet).map(Some)
}

/// Projects a fully decoded NPC skill packet. All records are validated into
/// an owned value before this function returns, providing an atomic commit
/// boundary to the runtime.
pub fn project_world_npc_skill_authority_0104(
    packet: &NpcSkillAuthorityPacket0104,
) -> Result<WorldNpcSkillAuthoritativeProjection0104, WorldNpcSkillProjectionError0104> {
    match packet {
        NpcSkillAuthorityPacket0104::SkillHit(hit) => {
            let prefix = hit.prefix();
            validate_caster(prefix.npc_id, prefix.skill_id)?;
            let mut caster_absolute_hp = None;
            let mut targets = Vec::with_capacity(hit.results().len());
            for result in hit.results().iter().copied() {
                let target = project_target(result.target())?;
                let mut post_state =
                    empty_target_projection(target, WorldNpcSkillSourceResult0104::Skill(result));
                match result {
                    NanoSkillResult0104::Damage(result) => {
                        wire_bool("Damage.bProtected", result.protected)?;
                        nonnegative("Damage.iDamage", result.damage)?;
                        nonnegative("Damage.iHP", result.hp)?;
                        post_state.absolute_hp = Some(result.hp);
                    }
                    NanoSkillResult0104::HealHp(result) => {
                        nonnegative("Heal_HP.iHealHP", result.healed_hp)?;
                        nonnegative("Heal_HP.iHP", result.hp)?;
                        post_state.absolute_hp = Some(result.hp);
                    }
                    NanoSkillResult0104::DamageDebuff(result) => {
                        wire_bool("Damage_N_Debuff.bProtected", result.protected)?;
                        nonnegative("Damage_N_Debuff.iDamage", result.damage)?;
                        nonnegative("Damage_N_Debuff.iHP", result.hp)?;
                        nonnegative_i16("Damage_N_Debuff.iStamina", result.nano_stamina)?;
                        let deactivated =
                            wire_bool("Damage_N_Debuff.bNanoDeactive", result.nano_deactivated)?;
                        post_state.absolute_hp = Some(result.hp);
                        post_state.absolute_condition_bit_flag = Some(result.condition_bit_flag);
                        if target.kind == WorldNpcSkillEntityKind0104::Player {
                            post_state.absolute_nano_stamina = Some(result.nano_stamina);
                            post_state.nano_deactivated = Some(deactivated);
                        }
                    }
                    NanoSkillResult0104::Buff(result) => {
                        wire_bool("Buff.bProtected", result.protected)?;
                        post_state.absolute_condition_bit_flag = Some(result.condition_bit_flag);
                    }
                    NanoSkillResult0104::BatteryDrain(result) => {
                        require_player_target("BatteryDrain", target)?;
                        wire_bool("BatteryDrain.bProtected", result.protected)?;
                        nonnegative("BatteryDrain.iDrainW", result.drained_weapon_battery)?;
                        nonnegative("BatteryDrain.iBatteryW", result.weapon_battery)?;
                        nonnegative("BatteryDrain.iDrainN", result.drained_nano_battery)?;
                        nonnegative("BatteryDrain.iBatteryN", result.nano_battery)?;
                        nonnegative_i16("BatteryDrain.iStamina", result.nano_stamina)?;
                        post_state.absolute_weapon_battery = Some(result.weapon_battery);
                        post_state.absolute_nano_battery = Some(result.nano_battery);
                        post_state.absolute_nano_stamina = Some(result.nano_stamina);
                        post_state.nano_deactivated = Some(wire_bool(
                            "BatteryDrain.bNanoDeactive",
                            result.nano_deactivated,
                        )?);
                        post_state.absolute_condition_bit_flag = Some(result.condition_bit_flag);
                    }
                    NanoSkillResult0104::Move(result) => {
                        post_state.movement = Some(WorldNpcSkillMovePostState0104 {
                            map_number: result.map_number,
                            position: result.position,
                        });
                    }
                    NanoSkillResult0104::Resurrect(result) => {
                        nonnegative("Resurrect.iRegenHP", result.hp)?;
                        post_state.absolute_hp = Some(result.hp);
                    }
                    NanoSkillResult0104::Leech(result) => {
                        if result.heal.target != result.damage.target {
                            return Err(WorldNpcSkillProjectionError0104::LeechTargetMismatch {
                                heal: result.heal.target,
                                damage: result.damage.target,
                            });
                        }
                        wire_bool("Leech.Damage.bProtected", result.damage.protected)?;
                        nonnegative("Leech.Heal.iHealHP", result.heal.healed_hp)?;
                        nonnegative("Leech.Heal.iHP", result.heal.hp)?;
                        nonnegative("Leech.Damage.iDamage", result.damage.damage)?;
                        nonnegative("Leech.Damage.iHP", result.damage.hp)?;
                        caster_absolute_hp = Some(result.heal.hp);
                        post_state.absolute_hp = Some(result.damage.hp);
                    }
                }
                targets.push(post_state);
            }
            Ok(WorldNpcSkillAuthoritativeProjection0104 {
                kind: WorldNpcSkillCastKind0104::Skill {
                    skill_type: prefix.skill_type,
                },
                caster: WorldNpcSkillCasterPostState0104 {
                    npc_id: prefix.npc_id,
                    skill_id: prefix.skill_id,
                    position: prefix.position,
                    absolute_hp: caster_absolute_hp,
                },
                targets,
            })
        }
        NpcSkillAuthorityPacket0104::CorruptionHit(hit) => {
            let prefix = hit.prefix();
            validate_caster(prefix.npc_id, prefix.skill_id)?;
            if !(0..=2).contains(&prefix.style) {
                return Err(WorldNpcSkillProjectionError0104::InvalidCorruptionStyle(
                    prefix.style,
                ));
            }
            let mut targets = Vec::with_capacity(hit.results().len());
            for result in hit.results().iter().copied() {
                let target = project_target(result.target)?;
                require_player_target("NPC corruption hit", target)?;
                wire_bool("CAttackResult.bProtected", result.protected)?;
                nonnegative("CAttackResult.iDamage", result.damage)?;
                if !matches!(result.hit_flag, 4 | 8 | 16) {
                    return Err(WorldNpcSkillProjectionError0104::InvalidCorruptionHitFlag(
                        result.hit_flag,
                    ));
                }
                if !(-1..=2).contains(&result.active_nano_slot) {
                    return Err(WorldNpcSkillProjectionError0104::InvalidActiveNanoSlot(
                        result.active_nano_slot,
                    ));
                }
                let deactivated =
                    wire_bool("CAttackResult.bNanoDeactive", result.nano_deactivated)?;
                nonnegative_i16("CAttackResult.iNanoID", result.nano_id)?;
                nonnegative_i16("CAttackResult.iNanoStamina", result.nano_stamina)?;

                let mut post_state = empty_target_projection(
                    target,
                    WorldNpcSkillSourceResult0104::Corruption(result),
                );
                // Unlike ordinary skill handlers, OpenFusion subtracts
                // corruption damage directly and may serialize negative HP.
                post_state.absolute_hp = Some(result.hp);
                post_state.absolute_condition_bit_flag = Some(result.condition_bit_flag);
                post_state.absolute_active_nano_slot = Some(result.active_nano_slot);
                post_state.absolute_nano_id = Some(result.nano_id);
                post_state.absolute_nano_stamina = Some(result.nano_stamina);
                post_state.nano_deactivated = Some(deactivated);
                post_state.condition_status_deleted = Some(result.condition_status_deleted);
                targets.push(post_state);
            }
            Ok(WorldNpcSkillAuthoritativeProjection0104 {
                kind: WorldNpcSkillCastKind0104::Corruption {
                    style: prefix.style,
                },
                caster: WorldNpcSkillCasterPostState0104 {
                    npc_id: prefix.npc_id,
                    skill_id: prefix.skill_id,
                    position: prefix.position,
                    absolute_hp: None,
                },
                targets,
            })
        }
    }
}

fn validate_caster(npc_id: i32, skill_id: i16) -> Result<(), WorldNpcSkillProjectionError0104> {
    if npc_id <= 0 {
        return Err(WorldNpcSkillProjectionError0104::InvalidNpcId(npc_id));
    }
    if skill_id <= 0 {
        return Err(WorldNpcSkillProjectionError0104::InvalidSkillId(skill_id));
    }
    Ok(())
}

fn project_target(
    target: NanoSkillTarget0104,
) -> Result<WorldNpcSkillEntity0104, WorldNpcSkillProjectionError0104> {
    let kind = match target.entity_type {
        1 => WorldNpcSkillEntityKind0104::Player,
        2 => WorldNpcSkillEntityKind0104::Npc,
        4 => WorldNpcSkillEntityKind0104::Mob,
        unsupported => {
            return Err(WorldNpcSkillProjectionError0104::UnsupportedEntityType(
                unsupported,
            ));
        }
    };
    if target.id <= 0 {
        return Err(WorldNpcSkillProjectionError0104::InvalidTargetId(target.id));
    }
    Ok(WorldNpcSkillEntity0104 {
        kind,
        id: target.id,
    })
}

fn empty_target_projection(
    target: WorldNpcSkillEntity0104,
    source_result: WorldNpcSkillSourceResult0104,
) -> WorldNpcSkillTargetPostState0104 {
    WorldNpcSkillTargetPostState0104 {
        target,
        absolute_hp: None,
        absolute_condition_bit_flag: None,
        absolute_weapon_battery: None,
        absolute_nano_battery: None,
        absolute_active_nano_slot: None,
        absolute_nano_id: None,
        absolute_nano_stamina: None,
        nano_deactivated: None,
        condition_status_deleted: None,
        movement: None,
        source_result,
    }
}

fn require_player_target(
    context: &'static str,
    target: WorldNpcSkillEntity0104,
) -> Result<(), WorldNpcSkillProjectionError0104> {
    if target.kind == WorldNpcSkillEntityKind0104::Player {
        Ok(())
    } else {
        Err(WorldNpcSkillProjectionError0104::PlayerTargetRequired { context, target })
    }
}

fn wire_bool(field: &'static str, value: i32) -> Result<bool, WorldNpcSkillProjectionError0104> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(WorldNpcSkillProjectionError0104::InvalidBoolean { field, value }),
    }
}

fn nonnegative(field: &'static str, value: i32) -> Result<(), WorldNpcSkillProjectionError0104> {
    if value >= 0 {
        Ok(())
    } else {
        Err(WorldNpcSkillProjectionError0104::NegativeValue { field, value })
    }
}

fn nonnegative_i16(
    field: &'static str,
    value: i16,
) -> Result<(), WorldNpcSkillProjectionError0104> {
    nonnegative(field, i32::from(value))
}

#[cfg(test)]
mod tests;
