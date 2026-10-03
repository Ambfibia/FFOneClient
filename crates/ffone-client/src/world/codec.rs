use super::*;

// Published map geometry has a strict one-node/one-mesh/one-primitive identity
// contract. Loading that primitive directly avoids WorldInstanceSpawner hierarchy
// cloning for every placement, so request/finalization can be broad enough to
// fill a nine-tile neighborhood without exposing a multi-second blank area.
// Both stages remain bounded to avoid moving the old WorldAssetRoot burst into one
// large ECS/material insertion frame.
pub(super) const NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME: usize = 256;

pub(super) const NATIVE_WORLD_VISUAL_FINALIZATIONS_PER_FRAME: usize = 256;

pub(super) const NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME: usize = 64;

pub(super) const NATIVE_WORLD_ENTITY_DESPAWNS_PER_FRAME: usize = 256;

// WorldInstanceReady can publish thousands of visual hierarchies in a dense
// dong. Build object-complete bounds incrementally instead of walking every
// ready hierarchy in one Update.
pub(super) const NATIVE_WORLD_RANGE_ENTITY_VISITS_PER_FRAME: usize = 256;

pub(super) const NATIVE_WORLD_RANGE_GROUP_FINALIZATIONS_PER_FRAME: usize = 64;

// Dense clean tiles contain more than 800 authored collider primitives. GLTF
// meshes often become ready together, so cooking all of them in one Update
// produced a conspicuous streaming hitch. A deterministic per-frame budget
// preserves exact triangle data while distributing that burst over roughly
// half a second at 60 FPS.
pub(super) const AUTHORED_COLLIDER_COOKS_PER_FRAME: usize = 32;

// Reusing already-cooked Arc geometry performs no vertex/index conversion.
// Bound only ECS insertion and transformed-AABB work for those placements.
pub(super) const AUTHORED_CACHED_COLLIDER_MATERIALIZATIONS_PER_FRAME: usize = 512;

// Cardinality, rather than collider count alone, bounds the allocations and
// index rewrites performed by `authored_collider_from_mesh`. Always admitting
// one oversized primitive prevents starvation while keeping an ordinary frame
// below these aggregate payload limits.
pub(super) const AUTHORED_COLLIDER_VERTICES_PER_FRAME: usize = 32_768;

pub(super) const AUTHORED_COLLIDER_INDICES_PER_FRAME: usize = 65_536;

#[derive(Debug)]
pub(super) struct NativeWorldSceneUnloadFrame {
    pub(super) entity: Entity,
    pub(super) children: Vec<Entity>,
    pub(super) next_child: usize,
}

impl NativeWorldSceneUnloadFrame {
    pub(super) fn new(entity: Entity, children: Option<&Children>) -> Self {
        Self {
            entity,
            children: children
                .map(|children| children.iter().collect())
                .unwrap_or_default(),
            next_child: 0,
        }
    }
}
