//! Engine-neutral normal-world Nano target planning for protocol 0104.
//!
//! The branch structure follows clean Retrobution `GameFrame.NanoSkillUse`:
//! targeted skills require `m_iEffectType == 1`; `m_iEffectTarget` selects a
//! focus, self, cone, caster-area, or focused-area route; and the selected
//! actor IDs are written to `sP_CL2FE_REQ_NANO_SKILL_USE`. Runtime entity and
//! transform discovery stays outside this module.

use std::collections::BTreeSet;

use ffone_protocol::NanoSkillUseRequest0104;

use crate::{
    tutorial_mission_content::GameplaySkillUiDefinition,
    world::AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
};

const SERVER_DISTANCE_TO_NATIVE: f32 = 0.01;
const LEGACY_PI: f32 = 3.14;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldNanoTargetKind {
    Npc { team: i32 },
    Player,
}

/// One already-live, targetable normal-world actor.
///
/// Positions and extents use native client units. `position` is the actor's
/// range-reference point supplied by the world producer. Clean NPC and player
/// list searches extend the XDT radius by `max(radius, height / 2)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldNanoTarget {
    pub actor_id: i32,
    pub kind: WorldNanoTargetKind,
    pub position: [f32; 3],
    pub radius: f32,
    pub height: f32,
}

/// Non-Bevy inputs needed to plan one normal-world Nano use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldNanoPlanInput<'a> {
    pub self_id: i32,
    pub caster_position: [f32; 3],
    pub caster_forward: [f32; 3],
    pub focused_target_id: Option<i32>,
    pub targets: &'a [WorldNanoTarget],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldNanoPlanError {
    InactiveSkill,
    UnsupportedEffectType(i32),
    UnsupportedEffectTarget(i32),
    UnsupportedTargetType { target: i32, target_type: i32 },
    InvalidTargetNumber(i32),
    InvalidDistance { field: &'static str, value: i32 },
    InvalidAngle(i32),
    InvalidSelfId(i32),
    InvalidCasterGeometry,
    InvalidTargetGeometry(i32),
    DuplicateTargetId(i32),
    MissingFocus,
    UnknownFocus(i32),
    FocusTargetTypeMismatch { focus_id: i32, target_type: i32 },
    FocusOutsideSelection(i32),
    NoTargets,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RankedTarget {
    actor_id: i32,
    distance: f32,
}

/// Builds the exact protocol-0104 Nano-use request after pure target planning.
///
/// XDT range and area values are converted with clean
/// `CoordUtil.Distance_SToC` (`value * 0.01`). Target order is deterministic:
/// the required focus/self is promoted first, then remaining actors are
/// ordered by distance and network actor ID before `m_iTargetNumber` is
/// applied.
pub fn plan_world_nano_skill_0104(
    skill: &GameplaySkillUiDefinition,
    input: WorldNanoPlanInput<'_>,
) -> Result<NanoSkillUseRequest0104, WorldNanoPlanError> {
    if !skill.active {
        return Err(WorldNanoPlanError::InactiveSkill);
    }
    if skill.effect_type != 1 {
        return Err(WorldNanoPlanError::UnsupportedEffectType(skill.effect_type));
    }
    if input.self_id <= 0 {
        return Err(WorldNanoPlanError::InvalidSelfId(input.self_id));
    }
    let target_capacity = usize::try_from(skill.target_number)
        .ok()
        .filter(|capacity| *capacity > 0)
        .ok_or(WorldNanoPlanError::InvalidTargetNumber(skill.target_number))?
        .min(NanoSkillUseRequest0104::MAX_TARGETS);

    // Self skills do not consume the surrounding actor list. NPC and player
    // IDs occupy separate protocol namespaces and may have the same number.
    if skill.target != 2 {
        validate_target_set(input.self_id, input.targets)?;
    }

    let (arg1, mut ranked) = match skill.target {
        1 => plan_focus(skill, input)?,
        2 => (
            input.self_id,
            vec![RankedTarget {
                actor_id: input.self_id,
                distance: 0.0,
            }],
        ),
        3 => plan_cone(skill, input)?,
        4 => return Err(WorldNanoPlanError::UnsupportedEffectTarget(4)),
        5 => plan_caster_area(skill, input)?,
        6 => plan_focused_area(skill, input)?,
        unsupported => {
            return Err(WorldNanoPlanError::UnsupportedEffectTarget(unsupported));
        }
    };

    if ranked.is_empty() {
        return Err(WorldNanoPlanError::NoTargets);
    }
    ranked.truncate(target_capacity);
    if ranked.is_empty() {
        return Err(WorldNanoPlanError::NoTargets);
    }

    Ok(NanoSkillUseRequest0104 {
        bullet_id: -1,
        arg1,
        arg2: 0,
        arg3: 0,
        target_ids: ranked.into_iter().map(|target| target.actor_id).collect(),
    })
}

fn plan_focus(
    skill: &GameplaySkillUiDefinition,
    input: WorldNanoPlanInput<'_>,
) -> Result<(i32, Vec<RankedTarget>), WorldNanoPlanError> {
    require_target_type(skill, &[1, 2])?;
    let focus = focus_for_target_type(skill, input)?;
    require_caster_geometry(input.caster_position)?;
    let range = native_distance("m_iEffectRange", skill.range)?;
    let caster = caster_reference(skill.target_type, input.caster_position);
    let distance = distance(caster, target_reference(skill.target_type, focus));
    if !within_extent(distance, focus, range) {
        return Err(WorldNanoPlanError::FocusOutsideSelection(focus.actor_id));
    }
    Ok((
        focus.actor_id,
        vec![RankedTarget {
            actor_id: focus.actor_id,
            distance,
        }],
    ))
}

fn plan_cone(
    skill: &GameplaySkillUiDefinition,
    input: WorldNanoPlanInput<'_>,
) -> Result<(i32, Vec<RankedTarget>), WorldNanoPlanError> {
    require_target_type(skill, &[1, 2])?;
    let focus = focus_for_target_type(skill, input)?;
    require_caster_geometry(input.caster_position)?;
    let caster = caster_reference(skill.target_type, input.caster_position);
    let forward = normalized_horizontal_forward(input.caster_forward)?;
    let range = native_distance("m_iEffectRange", skill.range)?;
    if !(0..=180).contains(&skill.angle) {
        return Err(WorldNanoPlanError::InvalidAngle(skill.angle));
    }
    let half_angle_radians = skill.angle as f32 / 180.0 * LEGACY_PI;

    let mut ranked = input
        .targets
        .iter()
        .filter(|candidate| target_type_matches(skill.target_type, candidate.kind))
        .filter_map(|candidate| {
            let offset = subtract(target_reference(skill.target_type, candidate), caster);
            let distance = magnitude(offset);
            let direction = normalize(offset).unwrap_or(forward);
            let angle = direction
                .iter()
                .zip(forward)
                .map(|(left, right)| left * right)
                .sum::<f32>()
                .clamp(-1.0, 1.0)
                .acos();
            (angle <= half_angle_radians && within_extent(distance, candidate, range)).then_some(
                RankedTarget {
                    actor_id: candidate.actor_id,
                    distance,
                },
            )
        })
        .collect::<Vec<_>>();
    sort_ranked(&mut ranked);
    promote_required(&mut ranked, focus.actor_id)?;
    Ok((focus.actor_id, ranked))
}

fn plan_caster_area(
    skill: &GameplaySkillUiDefinition,
    input: WorldNanoPlanInput<'_>,
) -> Result<(i32, Vec<RankedTarget>), WorldNanoPlanError> {
    require_target_type(skill, &[1, 2, 3])?;
    require_caster_geometry(input.caster_position)?;
    let area = native_distance("m_iEffectArea", skill.area)?;
    let mut ranked = if skill.target_type == 3 {
        Vec::new()
    } else {
        ranked_area_targets(
            skill.target_type,
            caster_reference(skill.target_type, input.caster_position),
            area,
            input.targets,
        )
    };
    if matches!(skill.target_type, 2 | 3) {
        ranked.push(RankedTarget {
            actor_id: input.self_id,
            distance: 0.0,
        });
    }
    sort_ranked(&mut ranked);
    if matches!(skill.target_type, 2 | 3) {
        promote_required(&mut ranked, input.self_id)?;
    }
    Ok((input.self_id, ranked))
}

fn plan_focused_area(
    skill: &GameplaySkillUiDefinition,
    input: WorldNanoPlanInput<'_>,
) -> Result<(i32, Vec<RankedTarget>), WorldNanoPlanError> {
    require_target_type(skill, &[1, 2])?;
    let focus = focus_for_target_type(skill, input)?;
    require_caster_geometry(input.caster_position)?;
    let range = native_distance("m_iEffectRange", skill.range)?;
    if !within_extent(
        distance(
            caster_reference(skill.target_type, input.caster_position),
            target_reference(skill.target_type, focus),
        ),
        focus,
        range,
    ) {
        return Err(WorldNanoPlanError::FocusOutsideSelection(focus.actor_id));
    }
    let area = native_distance("m_iEffectArea", skill.area)?;
    let mut ranked = ranked_area_targets(
        skill.target_type,
        target_reference(skill.target_type, focus),
        area,
        input.targets,
    );
    if !ranked
        .iter()
        .any(|candidate| candidate.actor_id == focus.actor_id)
    {
        ranked.push(RankedTarget {
            actor_id: focus.actor_id,
            distance: 0.0,
        });
    }
    sort_ranked(&mut ranked);
    promote_required(&mut ranked, focus.actor_id)?;
    Ok((focus.actor_id, ranked))
}

fn ranked_area_targets(
    target_type: i32,
    origin: [f32; 3],
    area: f32,
    targets: &[WorldNanoTarget],
) -> Vec<RankedTarget> {
    let mut ranked = targets
        .iter()
        .filter(|candidate| target_type_matches(target_type, candidate.kind))
        .filter_map(|candidate| {
            let distance = distance(origin, target_reference(target_type, candidate));
            within_extent(distance, candidate, area).then_some(RankedTarget {
                actor_id: candidate.actor_id,
                distance,
            })
        })
        .collect::<Vec<_>>();
    sort_ranked(&mut ranked);
    ranked
}

fn validate_target_set(
    self_id: i32,
    targets: &[WorldNanoTarget],
) -> Result<(), WorldNanoPlanError> {
    let mut ids = BTreeSet::new();
    for target in targets {
        if target.actor_id <= 0
            || target.actor_id == self_id
            || !target
                .position
                .iter()
                .all(|coordinate| coordinate.is_finite())
            || !target.radius.is_finite()
            || target.radius < 0.0
            || !target.height.is_finite()
            || target.height < 0.0
        {
            return Err(WorldNanoPlanError::InvalidTargetGeometry(target.actor_id));
        }
        if !ids.insert(target.actor_id) {
            return Err(WorldNanoPlanError::DuplicateTargetId(target.actor_id));
        }
    }
    Ok(())
}

fn focus_for_target_type<'a>(
    skill: &GameplaySkillUiDefinition,
    input: WorldNanoPlanInput<'a>,
) -> Result<&'a WorldNanoTarget, WorldNanoPlanError> {
    let focus_id = input
        .focused_target_id
        .ok_or(WorldNanoPlanError::MissingFocus)?;
    let focus = input
        .targets
        .iter()
        .find(|candidate| candidate.actor_id == focus_id)
        .ok_or(WorldNanoPlanError::UnknownFocus(focus_id))?;
    if !target_type_matches(skill.target_type, focus.kind) {
        return Err(WorldNanoPlanError::FocusTargetTypeMismatch {
            focus_id,
            target_type: skill.target_type,
        });
    }
    Ok(focus)
}

fn require_target_type(
    skill: &GameplaySkillUiDefinition,
    supported: &[i32],
) -> Result<(), WorldNanoPlanError> {
    if supported.contains(&skill.target_type) {
        Ok(())
    } else {
        Err(WorldNanoPlanError::UnsupportedTargetType {
            target: skill.target,
            target_type: skill.target_type,
        })
    }
}

fn target_type_matches(target_type: i32, kind: WorldNanoTargetKind) -> bool {
    match (target_type, kind) {
        (1, WorldNanoTargetKind::Npc { team }) => team > 1,
        (2, WorldNanoTargetKind::Player) => true,
        _ => false,
    }
}

fn caster_reference(target_type: i32, mut position: [f32; 3]) -> [f32; 3] {
    if target_type == 2 {
        position[1] += AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5;
    }
    position
}

fn target_reference(target_type: i32, target: &WorldNanoTarget) -> [f32; 3] {
    let mut position = target.position;
    if target_type == 2 {
        position[1] += target.height * 0.5;
    }
    position
}

fn native_distance(field: &'static str, raw: i32) -> Result<f32, WorldNanoPlanError> {
    if raw < 0 {
        return Err(WorldNanoPlanError::InvalidDistance { field, value: raw });
    }
    Ok(raw as f32 * SERVER_DISTANCE_TO_NATIVE)
}

fn require_caster_geometry(position: [f32; 3]) -> Result<(), WorldNanoPlanError> {
    if position.iter().all(|coordinate| coordinate.is_finite()) {
        Ok(())
    } else {
        Err(WorldNanoPlanError::InvalidCasterGeometry)
    }
}

fn normalized_horizontal_forward(forward: [f32; 3]) -> Result<[f32; 3], WorldNanoPlanError> {
    if !forward.iter().all(|coordinate| coordinate.is_finite()) {
        return Err(WorldNanoPlanError::InvalidCasterGeometry);
    }
    normalize([forward[0], 0.0, forward[2]]).ok_or(WorldNanoPlanError::InvalidCasterGeometry)
}

fn within_extent(distance: f32, target: &WorldNanoTarget, limit: f32) -> bool {
    let extent = target.radius.max(target.height * 0.5);
    distance < extent + limit
}

fn promote_required(
    ranked: &mut Vec<RankedTarget>,
    required_id: i32,
) -> Result<(), WorldNanoPlanError> {
    let index = ranked
        .iter()
        .position(|candidate| candidate.actor_id == required_id)
        .ok_or(WorldNanoPlanError::FocusOutsideSelection(required_id))?;
    let required = ranked.remove(index);
    ranked.insert(0, required);
    Ok(())
}

fn sort_ranked(ranked: &mut [RankedTarget]) {
    ranked.sort_by(|left, right| {
        left.distance
            .total_cmp(&right.distance)
            .then_with(|| left.actor_id.cmp(&right.actor_id))
    });
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    magnitude(subtract(left, right))
}

fn magnitude(vector: [f32; 3]) -> f32 {
    vector
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt()
}

fn normalize(vector: [f32; 3]) -> Option<[f32; 3]> {
    let magnitude = magnitude(vector);
    if magnitude <= f32::EPSILON || !magnitude.is_finite() {
        return None;
    }
    Some([
        vector[0] / magnitude,
        vector[1] / magnitude,
        vector[2] / magnitude,
    ])
}

#[cfg(test)]
mod tests;
