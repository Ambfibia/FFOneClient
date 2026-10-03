//! Pass companions, renderer order and render-queue sort biases.

use super::render_plan::LegacyPassKind;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyMaterialPassCompanion {
    pub source_mesh_entity: Entity,
    pub pass: LegacyPassKind,
}

/// Authoritative renderer position inside one flattened legacy model. Shared
/// player rigs preserve `ActorSkinCombiner` traversal; standalone logical
/// models publish an explicit queue/compositing ordinal recovered offline.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyMaterialRendererOrder {
    pub renderer_index: u16,
}

/// Fail-closed proof that all native material entities for one source renderer
/// received its exact renderer/pass sort order.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyMaterialSortOrderApplied {
    pub renderer_index: u16,
    pub pass_count: u8,
}

// `Material::depth_bias` is a transparent-phase view-depth sort bias in Bevy
// 0.17; it does not move vertices or alter depth comparison. Bevy otherwise
// sorts each primitive by its entity origin. That loses Unity's ShaderLab queue
// when, for example, a Fusion eye renderer is parented to the head while the
// skinned face renderer remains at the actor root (Fusion Mac/Bloo).
//
// Preserve the exact typed ShaderLab queue as the dominant key, then use the
// publisher-flattened renderer/pass order as the tie-breaker inside one queue.
// The same queue scale must apply to characters, static world and particles.
// A single queue step exceeds both the 340-unit world and 1000-unit cinematic
// camera ranges. An actor-sized bias lets the ship cover its foreground vortex
// and swaps the vortex's q3010/q3011 shells when the camera turns.
// One renderer/pass slot receives one view unit, exceeding the audited 0.633
// same-queue requirement (Fusion Professor Utonium is the current outlier).
pub(super) const LEGACY_MATERIAL_RENDER_QUEUE_BASE: i32 = 2_900;
pub(super) const LEGACY_MATERIAL_RENDER_QUEUE_SORT_STEP: f32 = 2_048.0;
pub(super) const LEGACY_MATERIAL_RENDERER_SORT_STEP: f32 = 1.0;
pub(super) const LEGACY_MATERIAL_PASS_STRIDE: u16 = 4;

pub(crate) fn legacy_render_queue_sort_bias(queue: i32) -> f32 {
    queue.saturating_sub(LEGACY_MATERIAL_RENDER_QUEUE_BASE) as f32
        * LEGACY_MATERIAL_RENDER_QUEUE_SORT_STEP
}

pub(super) fn legacy_material_sort_bias(
    source_render_queue: i32,
    renderer_index: u16,
    pass_ordinal: usize,
) -> Option<f32> {
    let pass_ordinal = u16::try_from(pass_ordinal).ok()?;
    if pass_ordinal >= LEGACY_MATERIAL_PASS_STRIDE {
        return None;
    }
    let queue_offset = source_render_queue.checked_sub(LEGACY_MATERIAL_RENDER_QUEUE_BASE)?;
    Some(
        queue_offset as f32 * LEGACY_MATERIAL_RENDER_QUEUE_SORT_STEP
            + f32::from(
                renderer_index
                    .saturating_mul(LEGACY_MATERIAL_PASS_STRIDE)
                    .saturating_add(pass_ordinal),
            ) * LEGACY_MATERIAL_RENDERER_SORT_STEP,
    )
}

pub(super) fn legacy_static_world_sort_bias(source_render_queue: i32, pass_ordinal: usize) -> Option<f32> {
    // Static-world v1 GLBs predate `extras.ffone.rendererIndex` and contain one
    // source renderer per mesh primitive. Preserve the ShaderLab queue with
    // renderer zero, then retain pass order as the local tie-breaker.
    legacy_material_sort_bias(source_render_queue, 0, pass_ordinal)
}
