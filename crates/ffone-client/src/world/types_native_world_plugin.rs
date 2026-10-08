use super::*;

impl PendingNativeWorldSceneSpawn {
    pub(super) fn new(terrain: Option<Arc<NativeTerrain>>) -> Self {
        Self {
            next_visual: 0,
            next_collider: 0,
            terrain,
            terrain_spawned: false,
        }
    }

    pub(super) fn next_batch(
        &mut self,
        visual_count: usize,
        collider_count: usize,
        visual_budget: usize,
        collider_budget: usize,
    ) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
        let visual_start = self.next_visual.min(visual_count);
        let visual_end = visual_start
            .saturating_add(visual_budget.min(NATIVE_WORLD_VISUAL_SPAWNS_PER_FRAME))
            .min(visual_count);
        let collider_start = self.next_collider.min(collider_count);
        let collider_end = collider_start
            .saturating_add(collider_budget.min(NATIVE_WORLD_COLLIDER_SPAWNS_PER_FRAME))
            .min(collider_count);
        self.next_visual = visual_end;
        self.next_collider = collider_end;
        (visual_start..visual_end, collider_start..collider_end)
    }

    pub(super) fn is_complete(&self, visual_count: usize, collider_count: usize) -> bool {
        self.next_visual >= visual_count
            && self.next_collider >= collider_count
            && self.terrain_spawned
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldRequestedScope(pub(super) NativeWorldScope);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SpawnedNativeWorldVisual {
    pub model_path: String,
    pub source_model_path: String,
    pub scene: usize,
}

/// Exact primary presentation exception for audited oversized SmallStuff
/// renderers. The identity is path-scoped so no other layer-12 visual is
/// affected and the authored mesh remains available whenever the camera is
/// inside the clean client's 130-unit Balanced-profile cap.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldPrimaryFarPresentationGuard;

#[derive(Debug)]
pub(super) struct NativeWorldPreparedRangeMember {
    pub(super) visual_root: Entity,
    pub(super) local_minimum: Vec3,
    pub(super) local_maximum: Vec3,
    pub(super) meshes: Vec<Entity>,
}

#[derive(Debug)]
pub(super) struct NativeWorldPendingRangeGroup {
    pub(super) expected_members: usize,
    pub(super) members: Vec<NativeWorldPreparedRangeMember>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeWorldVisualSceneReady;

#[derive(Debug, Clone)]
pub(super) struct PendingAuthoredPerimeterFootprint {
    pub(super) source_mesh: Handle<Mesh>,
    pub(super) local_scale: Vec2,
    pub(super) expected_vertex_count: usize,
    pub(super) expected_index_count: usize,
}

/// Moving authored triangle surface currently supporting a local avatar.
/// The previous matrix lets collision reproduce Unity platform carry without
/// introducing a non-source free-body physics solver.
#[derive(Component, Debug, Clone, Copy)]
pub struct NativeWorldGroundSupport {
    pub collider: Entity,
    pub last_world_from_local: Mat4,
    /// Up-facing geometric normal of the exact supporting triangle. Scripted
    /// EP slopes consume this as clean `ControllerColliderHit.normal` instead
    /// of inferring surface tilt from their waypoint direction.
    pub surface_normal: Vec3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnedNativeWorldScene {
    pub root: Entity,
    pub visuals: Vec<Entity>,
    pub colliders: Vec<Entity>,
}

pub struct NativeWorldPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeWorldSet {
    Unload,
    ResolveCollision,
    /// Normal gameplay-camera obstruction correction. Direct camera owners
    /// such as NPC sub-targets must run after this set so the player-targeted
    /// ray cannot overwrite their final pose.
    ResolveCameraOcclusion,
    RevealPresentation,
}

impl Plugin for NativeWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, square_settings::update_shader);
        install_native_terrain(app);
        app.init_resource::<NativeTerrainSpatialRegistry>()
            .init_resource::<NativeWorldStreamingStatus>()
            .init_resource::<NativeWorldLocationPresentation>()
            .init_resource::<NativeWorldObjectRangeCameraCache>()
            .init_resource::<NativeWorldMeshAabbCache>()
            .init_resource::<AuthoredColliderGeometryCache>()
            .init_resource::<AuthoredColliderSpatialIndex>()
            // One global typed observer replaces one observer entity per
            // world visual. Dense tiles contain thousands of GLB instances;
            // the event target query below still keeps the ownership exact.
            .add_observer(mark_native_world_visual_scene_ready)
            .add_systems(
                Update,
                (
                    unload_native_world_scenes_incrementally
                        .in_set(NativeWorldSet::Unload)
                        .run_if(any_with_component::<PendingNativeWorldSceneUnload>),
                    materialize_pending_native_world_scene_spawns,
                    materialize_loaded_native_world_visual_assets
                        .after(materialize_pending_native_world_scene_spawns),
                    reveal_neighboring_water_surfaces
                        .in_set(NativeWorldSet::RevealPresentation)
                        .after(stream_neighboring_water_surfaces),
                    prepare_native_world_object_ranges
                        .after(materialize_loaded_native_world_visual_assets),
                    propagate_native_world_object_ranges_to_late_meshes
                        .after(prepare_native_world_object_ranges),
                    make_unsafe_native_world_range_groups_unbounded
                        .after(propagate_native_world_object_ranges_to_late_meshes)
                        .after(crate::world_behaviour::materialize_pending_world_behaviours),
                    update_native_world_object_residency
                        .after(make_unsafe_native_world_range_groups_unbounded),
                    materialize_authored_tri_mesh_colliders
                        .after(materialize_pending_native_world_scene_spawns),
                    sync_authored_collider_world_bounds
                        .after(materialize_authored_tri_mesh_colliders),
                    sync_runtime_attached_authored_collider_transforms
                        .after(materialize_authored_tri_mesh_colliders)
                        .after(sync_authored_collider_world_bounds)
                        .after(crate::world_behaviour::materialize_pending_world_behaviours)
                        .after(crate::world_behaviour::update_world_platforms)
                        .after(crate::world_behaviour::apply_world_animation_samples)
                        .after(crate::world_behaviour::update_world_billboards)
                        .after(crate::network_world_runtime::advance_network_npc_motion_0104)
                        .after(
                            crate::network_world_runtime::advance_network_transportation_motion_0104,
                        ),
                    sync_authored_collider_spatial_index
                        .after(sync_authored_collider_world_bounds)
                        .after(sync_runtime_attached_authored_collider_transforms),
                    sync_native_terrain_spatial_registry,
                ),
            )
            .add_systems(
                Update,
                resolve_authored_world_ground
                    .in_set(NativeWorldSet::ResolveCollision)
                    .after(sync_native_terrain_spatial_registry)
                    .after(sync_authored_collider_world_bounds)
                    .after(sync_runtime_attached_authored_collider_transforms)
                    .after(sync_authored_collider_spatial_index)
                    .after(LegacyMovementSet::Simulate)
                    .after(interpolate_remote_players)
                    .after(crate::network_world_runtime::advance_network_npc_motion_0104)
                    .after(
                        crate::network_world_runtime::advance_network_transportation_motion_0104,
                    )
                    .before(LegacyMovementSet::CameraPose),
            )
            .add_systems(
                Update,
                player_contact_shadow::update_player_contact_shadow
                    .after(NativeWorldSet::ResolveCollision)
                    .after(sync_authored_collider_spatial_index)
                    .run_if(|| {
                        !cfg!(feature = "diagnostics")
                            || std::env::var_os("FFONE_PERF_CONTACT_SHADOW_BASELINE").is_none()
                    }),
            )
            .add_systems(
                Update,
                resolve_authored_camera_occlusion
                    .in_set(NativeWorldSet::ResolveCameraOcclusion)
                    .after(LegacyMovementSet::CameraPose)
                    .after(sync_authored_collider_spatial_index),
            )
            .add_systems(
                Update,
                stream_native_world_dongs
                    .after(LegacyMovementSet::Simulate)
                    .after(unload_native_world_scenes_incrementally)
                    .after(materialize_pending_native_world_scene_spawns),
            )
            .add_systems(
                Update,
                stream_neighboring_water_surfaces
                    .after(stream_native_world_dongs)
                    .after(reveal_native_world_scenes)
                    .after(unload_native_world_scenes_incrementally),
            )
            .add_systems(
                Update,
                reveal_native_world_scenes
                    .in_set(NativeWorldSet::RevealPresentation)
                    .after(prepare_native_world_object_ranges)
                    .after(propagate_native_world_object_ranges_to_late_meshes)
                    .after(make_unsafe_native_world_range_groups_unbounded)
                    .after(update_native_world_object_residency)
                    .after(materialize_authored_tri_mesh_colliders)
                    .after(crate::world_behaviour::apply_pending_world_behaviours)
                    .after(crate::world_behaviour::enqueue_pending_world_scripted_effects),
            );
    }
}

/// Tests actual collision geometry between a water sheet and the avatar's feet.
/// The proximity band alone also includes dry floors just above the sheet.
#[derive(bevy::ecs::system::SystemParam)]
pub struct NativeWaterOcclusion<'w, 's> {
    pub(super) index: Option<Res<'w, AuthoredColliderSpatialIndex>>,
    pub(super) terrains: Query<'w, 's, (&'static GlobalTransform, &'static NativeHeightmapCollider)>,
    pub(super) colliders: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static AuthoredTriMeshCollider,
            &'static AuthoredColliderWorldBounds,
        ),
    >,
    pub(super) candidates: Local<'s, Vec<Entity>>,
}

impl NativeWaterOcclusion<'_, '_> {
    pub fn blocks(&mut self, surface_height: f32, position: Vec3) -> bool {
        if position.y <= surface_height {
            return false;
        }
        // Exclude the water sheet itself while retaining the solid surface at
        // the feet. A barrier may be steeper than the walking slope limit.
        let start = position + Vec3::Y * 0.001;
        let end = Vec3::new(position.x, surface_height + 0.001, position.z);
        if self
            .terrains
            .iter()
            .any(|(global, terrain)| terrain.segment_hit(global, start, end).is_some())
        {
            return true;
        }
        let blocks = |(global, collider, bounds): (
            &GlobalTransform,
            &AuthoredTriMeshCollider,
            &AuthoredColliderWorldBounds,
        )| {
            authored_collider_blocks_segment_with_bounds(collider, global, bounds, start, end)
        };
        if let Some(index) = &self.index {
            index.candidates(
                Vec3::new(position.x, surface_height, position.z),
                position,
                &mut self.candidates,
            );
            self.candidates
                .iter()
                .any(|entity| self.colliders.get(*entity).is_ok_and(blocks))
        } else {
            self.colliders.iter().any(blocks)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AuthoredSegmentHit {
    pub fraction: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AuthoredWallMotionResult {
    pub(super) position: Vec3,
    pub(super) contact_normal: Option<Vec3>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AuthoredCapsuleResponse {
    /// UP/SIDE and ordinary overlap recovery reject steep faces in XZ so a
    /// wall cannot manufacture vertical climbing motion.
    Wall,
    /// PhysX CCT's DOWN pass sweeps the complete capsule against the complete
    /// contact normal and stops after its first hit (`maxIterDown == 1`).
    DownwardSweep,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AuthoredControllerMotionResult {
    pub(super) position: Vec3,
    pub(super) contact_normal: Option<Vec3>,
    pub(super) collision_flags: u8,
}

// Exact world-space preparation for repeated NPC/remote-player vertical probes.
// Cells are rejected by outward-rounded intervals of the ORIGINAL barycentric
// formula, not a tighter AABB that would discard its accepted edge tolerance.
#[derive(Clone, Copy)]
pub(super) struct GroundInterval {
    pub(super) low: f32,
    pub(super) high: f32,
}

impl GroundInterval {
    pub(super) fn rounded(low: f32, high: f32) -> Self {
        if low.is_nan() || high.is_nan() {
            Self {
                low: f32::NEG_INFINITY,
                high: f32::INFINITY,
            }
        } else {
            Self {
                low: low.next_down(),
                high: high.next_up(),
            }
        }
    }

    pub(super) fn constant(value: f32) -> Self {
        Self {
            low: value,
            high: value,
        }
    }
    pub(super) fn add(self, rhs: Self) -> Self {
        Self::rounded(self.low + rhs.low, self.high + rhs.high)
    }
    pub(super) fn sub(self, rhs: Self) -> Self {
        Self::rounded(self.low - rhs.high, self.high - rhs.low)
    }
    pub(super) fn mul(self, value: f32) -> Self {
        let (a, b) = (self.low * value, self.high * value);
        if a.is_nan() || b.is_nan() {
            return Self::rounded(f32::NAN, f32::NAN);
        }
        Self::rounded(a.min(b), a.max(b))
    }
    pub(super) fn div(self, value: f32) -> Self {
        let (a, b) = (self.low / value, self.high / value);
        if a.is_nan() || b.is_nan() {
            return Self::rounded(f32::NAN, f32::NAN);
        }
        Self::rounded(a.min(b), a.max(b))
    }
}

// Work is shared across every actor/query in this update. Incomplete indices
// always use the original query; preparing a large collider never gates loading.
pub(super) struct GroundPreparationBudget {
    pub(super) triangles: usize,
    pub(super) cell_tests: usize,
}

impl Default for GroundPreparationBudget {
    fn default() -> Self {
        Self {
            triangles: 512,
            cell_tests: 4096,
        }
    }
}
