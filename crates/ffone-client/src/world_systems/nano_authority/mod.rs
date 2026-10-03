//! Pure authoritative projection for protocol-0104 normal-world Nano results.
//!
//! OpenFusion mutates combatants before serializing the `eST`-specific tail of
//! `P_FE2CL_NANO_SKILL_USE_SUCC` (caster) or the byte-identical
//! `P_FE2CL_NANO_SKILL_USE` (viewers). This module validates the complete
//! packet and projects only absolute post-state. Bevy/runtime mutation remains
//! outside this module so callers can parse every record before committing any
//! state.

use std::fmt;

use ffone_protocol::{
    NanoSkillResult0104, NanoSkillTarget0104, NanoSkillUseDecodeError0104,
    NanoSkillUseDelivery0104, NanoSkillUsePacket0104, decode_nano_skill_use_packet_0104,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldNanoEntityKind0104 {
    Player,
    Npc,
    Mob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNanoEntity0104 {
    pub kind: WorldNanoEntityKind0104,
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNanoCasterPostState0104 {
    pub pc_id: i32,
    pub nano_id: i16,
    pub skill_id: i16,
    pub nano_stamina: i16,
    pub nano_deactivated: bool,
    /// Present for Bloodsucking/Leech. OpenFusion places the caster's absolute
    /// HP in the nested heal record while copying the damaged target's ID into
    /// that record, so the packet prefix is the only correct owner identity.
    pub absolute_hp: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNanoMovePostState0104 {
    pub map_number: i32,
    pub position: [i32; 3],
}

/// One ordered target update. Optional fields mean "this result family does
/// not author this property"; callers must not synthesize it from effect
/// amounts. `source_result` retains damage/heal/drain values for presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldNanoTargetPostState0104 {
    pub target: WorldNanoEntity0104,
    pub absolute_hp: Option<i32>,
    pub absolute_condition_bit_flag: Option<i32>,
    pub absolute_weapon_battery: Option<i32>,
    pub absolute_nano_battery: Option<i32>,
    pub absolute_nano_stamina: Option<i16>,
    pub nano_deactivated: Option<bool>,
    pub movement: Option<WorldNanoMovePostState0104>,
    pub source_result: NanoSkillResult0104,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldNanoAuthoritativeProjection0104 {
    pub delivery: NanoSkillUseDelivery0104,
    pub skill_type: i32,
    pub caster: WorldNanoCasterPostState0104,
    /// Server result order is preserved. This matters if a malformed request
    /// repeats a target or a multi-target Leech updates caster HP repeatedly.
    pub targets: Vec<WorldNanoTargetPostState0104>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldNanoProjectionError0104 {
    Protocol(NanoSkillUseDecodeError0104),
    InvalidCasterId(i32),
    InvalidNanoId(i16),
    InvalidSkillId(i16),
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
    PlayerOnlyResult {
        skill_type: i32,
        target: WorldNanoEntity0104,
    },
    LeechTargetMismatch {
        heal: NanoSkillTarget0104,
        damage: NanoSkillTarget0104,
    },
}

impl From<NanoSkillUseDecodeError0104> for WorldNanoProjectionError0104 {
    fn from(error: NanoSkillUseDecodeError0104) -> Self {
        Self::Protocol(error)
    }
}

impl fmt::Display for WorldNanoProjectionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => error.fmt(formatter),
            Self::InvalidCasterId(id) => write!(formatter, "invalid Nano caster PC ID {id}"),
            Self::InvalidNanoId(id) => write!(formatter, "invalid active Nano ID {id}"),
            Self::InvalidSkillId(id) => write!(formatter, "invalid active Nano skill ID {id}"),
            Self::NegativeValue { field, value } => {
                write!(
                    formatter,
                    "Nano result field {field} cannot be negative: {value}"
                )
            }
            Self::InvalidBoolean { field, value } => write!(
                formatter,
                "Nano result field {field} must be 0 or 1, got {value}"
            ),
            Self::UnsupportedEntityType(entity_type) => write!(
                formatter,
                "Nano result has unsupported protocol-0104 eCT {entity_type}"
            ),
            Self::InvalidTargetId(id) => write!(formatter, "invalid Nano result target ID {id}"),
            Self::PlayerOnlyResult { skill_type, target } => write!(
                formatter,
                "Nano skill type {skill_type} produced a player-only result for {target:?}"
            ),
            Self::LeechTargetMismatch { heal, damage } => write!(
                formatter,
                "OpenFusion Leech heal/damage target identities diverge: {heal:?} vs {damage:?}"
            ),
        }
    }
}

impl std::error::Error for WorldNanoProjectionError0104 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            _ => None,
        }
    }
}

/// Strictly decodes local success and remote use packets, then validates and
/// projects every record before returning. `Err` and `Ok(None)` carry no
/// partial projection, which lets the runtime commit the returned value as one
/// transaction.
pub fn decode_world_nano_authority_0104(
    packet_type: u32,
    payload: &[u8],
) -> Result<Option<WorldNanoAuthoritativeProjection0104>, WorldNanoProjectionError0104> {
    let Some(packet) = decode_nano_skill_use_packet_0104(packet_type, payload)? else {
        return Ok(None);
    };
    project_world_nano_authority_0104(&packet).map(Some)
}

/// Projects a fully decoded packet into absolute normal-world post-state.
pub fn project_world_nano_authority_0104(
    packet: &NanoSkillUsePacket0104,
) -> Result<WorldNanoAuthoritativeProjection0104, WorldNanoProjectionError0104> {
    let prefix = packet.result.prefix();
    if prefix.pc_id <= 0 {
        return Err(WorldNanoProjectionError0104::InvalidCasterId(prefix.pc_id));
    }
    if prefix.nano_id <= 0 {
        return Err(WorldNanoProjectionError0104::InvalidNanoId(prefix.nano_id));
    }
    if prefix.skill_id <= 0 {
        return Err(WorldNanoProjectionError0104::InvalidSkillId(
            prefix.skill_id,
        ));
    }
    nonnegative_i16("prefix.iNanoStamina", prefix.nano_stamina)?;
    let nano_deactivated = wire_bool("prefix.bNanoDeactive", prefix.nano_deactivated)?;

    let mut caster_absolute_hp = None;
    let mut targets = Vec::with_capacity(packet.result.results().len());
    for source_result in packet.result.results().iter().copied() {
        let target = project_target(source_result.target())?;
        let mut post_state = WorldNanoTargetPostState0104 {
            target,
            absolute_hp: None,
            absolute_condition_bit_flag: None,
            absolute_weapon_battery: None,
            absolute_nano_battery: None,
            absolute_nano_stamina: None,
            nano_deactivated: None,
            movement: None,
            source_result,
        };

        match source_result {
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
                if target.kind == WorldNanoEntityKind0104::Player {
                    post_state.absolute_nano_stamina = Some(result.nano_stamina);
                    post_state.nano_deactivated = Some(deactivated);
                }
            }
            NanoSkillResult0104::Buff(result) => {
                wire_bool("Buff.bProtected", result.protected)?;
                post_state.absolute_condition_bit_flag = Some(result.condition_bit_flag);
            }
            NanoSkillResult0104::BatteryDrain(result) => {
                require_player_target(prefix.skill_type, target)?;
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
                require_player_target(prefix.skill_type, target)?;
                post_state.movement = Some(WorldNanoMovePostState0104 {
                    map_number: result.map_number,
                    position: result.position,
                });
            }
            NanoSkillResult0104::Resurrect(result) => {
                require_player_target(prefix.skill_type, target)?;
                nonnegative("Resurrect.iRegenHP", result.hp)?;
                post_state.absolute_hp = Some(result.hp);
            }
            NanoSkillResult0104::Leech(result) => {
                if result.heal.target != result.damage.target {
                    return Err(WorldNanoProjectionError0104::LeechTargetMismatch {
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

    Ok(WorldNanoAuthoritativeProjection0104 {
        delivery: packet.delivery,
        skill_type: prefix.skill_type,
        caster: WorldNanoCasterPostState0104 {
            pc_id: prefix.pc_id,
            nano_id: prefix.nano_id,
            skill_id: prefix.skill_id,
            nano_stamina: prefix.nano_stamina,
            nano_deactivated,
            absolute_hp: caster_absolute_hp,
        },
        targets,
    })
}

fn project_target(
    target: NanoSkillTarget0104,
) -> Result<WorldNanoEntity0104, WorldNanoProjectionError0104> {
    let kind = match target.entity_type {
        1 => WorldNanoEntityKind0104::Player,
        2 => WorldNanoEntityKind0104::Npc,
        4 => WorldNanoEntityKind0104::Mob,
        unsupported => {
            return Err(WorldNanoProjectionError0104::UnsupportedEntityType(
                unsupported,
            ));
        }
    };
    if target.id <= 0 {
        return Err(WorldNanoProjectionError0104::InvalidTargetId(target.id));
    }
    Ok(WorldNanoEntity0104 {
        kind,
        id: target.id,
    })
}

fn require_player_target(
    skill_type: i32,
    target: WorldNanoEntity0104,
) -> Result<(), WorldNanoProjectionError0104> {
    if target.kind == WorldNanoEntityKind0104::Player {
        Ok(())
    } else {
        Err(WorldNanoProjectionError0104::PlayerOnlyResult { skill_type, target })
    }
}

fn wire_bool(field: &'static str, value: i32) -> Result<bool, WorldNanoProjectionError0104> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(WorldNanoProjectionError0104::InvalidBoolean { field, value }),
    }
}

fn nonnegative(field: &'static str, value: i32) -> Result<(), WorldNanoProjectionError0104> {
    if value >= 0 {
        Ok(())
    } else {
        Err(WorldNanoProjectionError0104::NegativeValue { field, value })
    }
}

fn nonnegative_i16(field: &'static str, value: i16) -> Result<(), WorldNanoProjectionError0104> {
    nonnegative(field, i32::from(value))
}

#[cfg(test)]
mod tests;
