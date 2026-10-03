//! Source-compatible water and terrain-attribute gameplay contracts.
//!
//! Retrobution combines `MapAttributeTable` bits with the avatar's water
//! contact state. The water material only supplies that contact; poison and
//! healing authority remain the clean terrain bits. This module keeps that
//! classification and the authored water-surface sampling independent from
//! the application/network layer.

use bevy::{
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
};

pub const LEGACY_TERRAIN_ATTRIBUTE_POISON: u8 = 0x08;
pub const LEGACY_TERRAIN_ATTRIBUTE_HEAL: u8 = 0x10;
pub const LEGACY_TERRAIN_ATTRIBUTE_SCRIPTED_MINIMUM: u8 = 64;
pub const LEGACY_TERRAIN_CONTACT_HEIGHT: f32 = 0.5;
pub const LEGACY_WATER_CONTACT_HEIGHT: f32 = 0.5;
pub const LEGACY_WATER_FALLBACK_DEPTH: f32 = 2.5;
// `cnOwnAvatarStatus` constructor: `m_fMinimumCheckTimer = 1f`.
pub const LEGACY_ENVIRONMENT_TRANSITION_SECONDS: f32 = 1.0;
pub const LEGACY_LOCAL_INFECTION_TICK_SECONDS: f32 = 2.0;
pub const LEGACY_LOCAL_REGEN_TICK_SECONDS: f32 = 4.0;
// `GameFrame.fMaxCombatTime`: every local incoming/outgoing combat result
// refreshes this lease before cnVirtualServer may emit another heal tick.
pub const LEGACY_COMBAT_TIMEOUT_SECONDS: f32 = 5.0;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyEnvironmentSet {
    Update,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyWaterSurface {
    pub infected: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LegacyAvatarEnvironmentState {
    pub in_water: bool,
    pub infected_water: bool,
    pub poisoned: bool,
    pub healing: bool,
    pub terrain_attribute: Option<u8>,
    pub poison_transition_elapsed_seconds: f32,
    pub heal_transition_elapsed_seconds: f32,
    pub local_infection_elapsed_seconds: f32,
    pub local_regen_elapsed_seconds: f32,
    pub local_combat_timeout_remaining_seconds: f32,
    pub last_observed_hp: Option<i32>,
}

impl LegacyAvatarEnvironmentState {
    pub fn observe_local_combat(&mut self) {
        self.local_combat_timeout_remaining_seconds = LEGACY_COMBAT_TIMEOUT_SECONDS;
    }
}

impl Default for LegacyAvatarEnvironmentState {
    fn default() -> Self {
        Self {
            in_water: false,
            infected_water: false,
            poisoned: false,
            healing: false,
            terrain_attribute: None,
            poison_transition_elapsed_seconds: LEGACY_ENVIRONMENT_TRANSITION_SECONDS,
            heal_transition_elapsed_seconds: LEGACY_ENVIRONMENT_TRANSITION_SECONDS,
            local_infection_elapsed_seconds: 0.0,
            local_regen_elapsed_seconds: 0.0,
            local_combat_timeout_remaining_seconds: 0.0,
            last_observed_hp: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LegacyTerrainEnvironmentFlags {
    pub poisoned: bool,
    pub healing: bool,
}

#[must_use]
pub const fn legacy_terrain_environment_flags(
    attribute: Option<u8>,
    terrain_or_water_contact: bool,
) -> LegacyTerrainEnvironmentFlags {
    let Some(attribute) = attribute else {
        return LegacyTerrainEnvironmentFlags {
            poisoned: false,
            healing: false,
        };
    };
    if attribute >= LEGACY_TERRAIN_ATTRIBUTE_SCRIPTED_MINIMUM || !terrain_or_water_contact {
        return LegacyTerrainEnvironmentFlags {
            poisoned: false,
            healing: false,
        };
    }
    LegacyTerrainEnvironmentFlags {
        poisoned: attribute & LEGACY_TERRAIN_ATTRIBUTE_POISON != 0,
        healing: attribute & LEGACY_TERRAIN_ATTRIBUTE_HEAL != 0,
    }
}

/// Apply the clean vehicle gate to the terrain-attribute result.
///
/// `cnOwnAvatarStatus.Update` derives poison and heal only from bits `0x08`
/// and `0x10`. `cnAvatarAnimation.bWater` participates in the shared contact
/// gate, but the `ffPoison` material is not an independent damage flag.
#[must_use]
pub const fn legacy_environment_flags(
    attribute: Option<u8>,
    terrain_or_water_contact: bool,
    vehicle_mounted: bool,
) -> LegacyTerrainEnvironmentFlags {
    // `cnOwnAvatarStatus.Update` clears both candidate flags while
    // `cnAvatarAnimation.iVehicle != 0`.
    if vehicle_mounted {
        return LegacyTerrainEnvironmentFlags {
            poisoned: false,
            healing: false,
        };
    }
    legacy_terrain_environment_flags(attribute, terrain_or_water_contact)
}

/// Native volume approximation for Unity's `cnAvatarAnimation.bWater`.
///
/// A player may first be observed already below the authored surface after a
/// spawn, warp or streaming edge. Entry therefore cannot depend on the prior
/// `in_water` value. The bounded depth keeps a horizontal surface from
/// claiming unrelated world geometry far below it.
#[must_use]
pub fn legacy_water_surface_contact(surface_height: f32, avatar_y: f32) -> bool {
    if !surface_height.is_finite() || !avatar_y.is_finite() {
        return false;
    }
    let distance = avatar_y - surface_height;
    distance < LEGACY_WATER_CONTACT_HEIGHT && distance > -LEGACY_WATER_FALLBACK_DEPTH
}

/// Highest upward-facing triangle at the requested world X/Z.
#[must_use]
pub fn legacy_water_surface_height(
    mesh: &Mesh,
    global: &GlobalTransform,
    world_x: f32,
    world_z: f32,
) -> Option<f32> {
    if !world_x.is_finite() || !world_z.is_finite() {
        return None;
    }
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return None;
    };
    let world_from_local = global.to_matrix();
    let mut highest: Option<f32> = None;
    let mut visit = |a_index: usize, b_index: usize, c_index: usize| {
        let (Some(a), Some(b), Some(c)) = (
            positions.get(a_index),
            positions.get(b_index),
            positions.get(c_index),
        ) else {
            return;
        };
        let a = world_from_local.transform_point3(Vec3::from_array(*a));
        let b = world_from_local.transform_point3(Vec3::from_array(*b));
        let c = world_from_local.transform_point3(Vec3::from_array(*c));
        let normal = (b - a).cross(c - a).normalize_or_zero();
        if !normal.is_finite() || normal.y.abs() < 0.5 {
            return;
        }
        let Some(height) = triangle_height_at_xz(a, b, c, world_x, world_z) else {
            return;
        };
        highest = Some(highest.map_or(height, |current| current.max(height)));
    };

    match mesh.indices() {
        Some(Indices::U16(indices)) => {
            for triangle in indices.chunks_exact(3) {
                visit(
                    usize::from(triangle[0]),
                    usize::from(triangle[1]),
                    usize::from(triangle[2]),
                );
            }
        }
        Some(Indices::U32(indices)) => {
            for triangle in indices.chunks_exact(3) {
                visit(
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                );
            }
        }
        None => {
            for triangle in (0..positions.len()).collect::<Vec<_>>().chunks_exact(3) {
                visit(triangle[0], triangle[1], triangle[2]);
            }
        }
    }
    highest
}

fn triangle_height_at_xz(a: Vec3, b: Vec3, c: Vec3, x: f32, z: f32) -> Option<f32> {
    let denominator = (b.z - c.z) * (a.x - c.x) + (c.x - b.x) * (a.z - c.z);
    if !denominator.is_finite() || denominator.abs() <= f32::EPSILON {
        return None;
    }
    let weight_a = ((b.z - c.z) * (x - c.x) + (c.x - b.x) * (z - c.z)) / denominator;
    let weight_b = ((c.z - a.z) * (x - c.x) + (a.x - c.x) * (z - c.z)) / denominator;
    let weight_c = 1.0 - weight_a - weight_b;
    if weight_a < -0.000_01 || weight_b < -0.000_01 || weight_c < -0.000_01 {
        return None;
    }
    let height = weight_a * a.y + weight_b * b.y + weight_c * c.y;
    height.is_finite().then_some(height)
}

#[cfg(test)]
mod tests;
