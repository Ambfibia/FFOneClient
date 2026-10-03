
pub const NATIVE_WORLD_SCENE_SCHEMA: &str = "ffone.native-world-scene.v2";

pub const LEGACY_NATIVE_WORLD_SCENE_SCHEMA: &str = "ffone.native-world-scene.v1";

pub const MAP_TILE_SCHEMA: &str = "ffone.map-tile.v1";

pub const FIRST_NATIVE_WORLD_SCENE: &str = "map/tiles/map_08_06/scene.json";

/// Regression fixtures only. Runtime discovery is registry-driven with a legacy catalog fallback.
pub const NATIVE_WORLD_SCENE_PATHS: &[&str] =
    &[FIRST_NATIVE_WORLD_SCENE, "map/tiles/map_12_03/scene.json"];

pub const WORLD_TILE_SIZE_NATIVE: f32 = 512.0;

pub const LEGACY_DONG_UNLOAD_DISTANCE_NATIVE: f32 = 340.0;

/// Exact Retrobution gameplay-camera far clip, retained as parity evidence and
/// as the ambient-effect simulation ceiling.
pub const LEGACY_WORLD_CAMERA_FAR_NATIVE: f32 = 300.0;

/// Deliberately modest user-facing horizon extension. Object-complete adaptive
/// ranges retain visible silhouettes while Retrobution's exact distance fog
/// masks the final approach to this clip plane.
pub const EXTENDED_WORLD_CAMERA_FAR_NATIVE: f32 = 340.0;

pub const EXTENDED_DONG_UNLOAD_DISTANCE_NATIVE: f32 = 420.0;

/// The legacy nine-tile neighborhood remains the absolute resident-set bound.
pub const NATIVE_WORLD_MAX_RESIDENT_TILES: usize = 9;

pub(super) const NATIVE_WORLD_MAX_IN_FLIGHT_VISUAL_ASSETS: usize = 1_024;

pub(super) const NATIVE_WORLD_MAX_PENDING_COLLIDERS: usize = 256;

pub(super) const NATIVE_WORLD_RANGE_GROUP_FINALIZATIONS_PER_ROOT: usize = 16;

// Object size, rather than the often-wrong Unity renderer layer, owns the
// native far-world budget. Across five dense/representative published tiles,
// radius*35 keeps the sum-of-range-squared proxy at ~1.02x the former layer
// policy while retaining genuinely visible structure silhouettes to 340.
pub(super) const NATIVE_WORLD_RANGE_PER_RADIUS: f32 = 35.0;

/// Clean Retrobution's Balanced graphics profile presents SmallStuff through
/// 130 Unity units. Only individually audited oversized renderer families use
/// that primary limit here; this is deliberately not a layer-wide or general
/// world-range policy.
pub(super) const PRIMARY_AUDITED_SMALLSTUFF_PRESENTATION_END_NATIVE: f32 = 130.0;

pub(super) const NATIVE_WORLD_RANGE_CAMERA_STEP: f32 = 4.0;

pub(super) const WORLD_BASIS: &str = "H=diag(-1,1,1)";

pub(super) const WORLD_UNIT_SCALE: &str = "1-unity-unit-equals-1-bevy-unit";

pub(super) const WORLD_ORIGIN_POLICY: &str = "source-trs-unchanged-no-auto-centering";

pub(super) const UNIT_QUATERNION_TOLERANCE: f32 = 0.000_1;

pub(super) const GROUND_EPSILON: f32 = 0.05;

pub(super) const GROUNDED_STEP_UP: f32 = 0.3;

// A grounded CharacterController follows a walkable triangle as its capsule
// moves horizontally. Bound that downward continuation by both the exact
// 50-degree slopeLimit and the serialized 0.3 stepOffset: this follows curved
// terrain without restoring the old one-metre magnetic ground snap.
pub(super) const AUTHORED_WALKABLE_MAX_TANGENT: f32 = 1.191_753_6;

// tan(50 degrees)
pub(super) const AUTHORED_GROUND_SWEEP_TOLERANCE: f32 = 0.000_01;

// OpenFusion persists the last packet-authored PC elevation, while the clean
// client lets every visible remote CharacterController settle onto the local
// TerrainCollider. Ordinary-world spawn points can differ from the recovered
// heightfield by several units, so remote grounding needs a bounded initial
// step-down larger than the already-grounded local controller window.
pub(super) const REMOTE_GROUND_STEP_DOWN: f32 = 16.0;

pub(super) const REMOTE_GROUND_STEP_UP: f32 = 2.0;

// Exact OwnUser CharacterController values from the Retrobution player
// prefab: height 1.6, radius 0.3, skinWidth 0.03, slopeLimit 50, stepOffset
// 0.3. The feet/root convention already accounts for its Y=0.8 center.
pub const AUTHORED_CHARACTER_CONTROLLER_HEIGHT: f32 = 1.6;

pub const AUTHORED_CHARACTER_CONTROLLER_RADIUS: f32 = 0.3;

pub(super) const AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH: f32 = 0.03;

pub(super) const AUTHORED_WALKABLE_MIN_UP_DOT: f32 = 0.642_787_64;

pub(super) const AUTHORED_MAX_CONTACT_PLANES: usize = 16;

pub(super) const NATIVE_WORLD_RANGE_X_BITS: u32 = 9;

pub(super) const NATIVE_WORLD_RANGE_Z_BITS: u32 = 9;

pub(super) const NATIVE_WORLD_RANGE_Y_BITS: u32 = 10;

pub(super) const NATIVE_WORLD_RANGE_END_BITS: u32 = 4;

pub(super) const NATIVE_WORLD_RANGE_END_LEVELS: f32 = (1_u32 << NATIVE_WORLD_RANGE_END_BITS) as f32 - 1.0;
