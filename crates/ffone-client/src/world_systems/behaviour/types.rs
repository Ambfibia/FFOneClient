use super::*;

#[derive(Component, Clone, Debug)]
pub struct WorldBillboard {
    pub mode: BillboardMode,
    /// `VisibleSwitch` enables this when the owning renderer becomes visible.
    /// FFOne keeps it enabled after streaming because rotating an off-screen
    /// pivot has no observable output and avoids hiding the renderer itself.
    pub runtime_enabled: bool,
}

/// Exact `VisibleSwitch.switchable` association. The clean script uses
/// renderer visibility only to enable/disable the pointed `BillboardNode`; it
/// never hides or swaps the renderer itself.
#[derive(Component, Clone, Debug)]
pub struct WorldVisibilitySwitch {
    pub controlled_billboards: Vec<Entity>,
    /// Renderer entities whose Unity visibility callbacks drive the linked
    /// BillboardNode. This must be evaluated from Bevy's actual view
    /// visibility instead of eagerly enabling every imported billboard.
    pub renderer_entities: Vec<Entity>,
}

#[derive(Component, Clone, Debug)]
pub struct WorldEffectEmitter {
    pub controller: String,
    pub effect_name: Option<String>,
    pub priority: i64,
    pub max_timer: f32,
    pub longest_life_time: f32,
    pub trail_times: f32,
    pub conform_to_scale: bool,
    pub disable_update: bool,
    pub particle_element_count: usize,
    pub model_entities: Vec<Entity>,
    pub nif_object: Option<serde_json::Value>,
    pub particles: Vec<serde_json::Value>,
    pub particle_elements: Vec<serde_json::Value>,
}

/// Visual-only rotation for the authored needed-jump wing marker. Its effect
/// emitter has no particle or AnimationClip motion to drive the published mesh.
#[derive(Component, Clone, Debug, Default)]
pub struct WorldFloatingIconSpin {
    pub initial_rotation: Option<Quat>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldTriggerKind {
    Ring,
    Jumppad,
    Platform,
    Launcher,
    Zipline,
    Switch,
    Rope,
    Belt,
    Slope,
    Synchronizer,
    Other,
}

impl WorldTriggerKind {
    pub(super) fn from_document(kind: &str) -> Self {
        match kind {
            "ring" => Self::Ring,
            "jumppad" => Self::Jumppad,
            "platform" => Self::Platform,
            "launcher" => Self::Launcher,
            "zipline" => Self::Zipline,
            "switch" => Self::Switch,
            "rope" => Self::Rope,
            "belt" => Self::Belt,
            "slope" => Self::Slope,
            "synchronizer" => Self::Synchronizer,
            _ => Self::Other,
        }
    }
}

/// One scripted Infected-Zone element.
///
/// `server_id` and `object_id` are the identities the original client sends to
/// the server, so they are preserved exactly and never renumbered.
#[derive(Component, Clone, Debug)]
pub struct WorldTrigger {
    pub kind: WorldTriggerKind,
    pub server_id: i64,
    pub object_id: i64,
    pub cne_id: i64,
    pub trigger_type: i64,
    pub radius: f32,
    pub velocity: f32,
    pub speed: f32,
    pub add_power: f32,
    pub min_power: f32,
    pub max_power: f32,
    pub move_type: i64,
    pub waypoint_count: i64,
    pub start_position: Vec3,
    pub from: Vec3,
    pub to: Vec3,
    pub spin_velocity: f32,
    pub initial_rotate: Vec3,
    pub max_rotate: Vec3,
    pub head: Option<String>,
    pub tail: Option<String>,
    pub target_element_id: i64,
    pub target_element_trigger: i64,
    pub model_entities: Vec<Entity>,
    /// Ordered local-space points reached from the clean `head` pointer.
    pub path_points: Vec<Vec3>,
}

/// Source-parity driver for legacy `EpPlatformTrigger`. All published world
/// rigid bodies are kinematic and are moved by this script, never by gravity.
#[derive(Component, Clone, Debug)]
pub struct WorldPlatformMotion {
    pub initial_translation: Vec3,
    pub initial_rotation: Quat,
    pub initial_scale: Vec3,
    pub from: Vec3,
    pub to: Vec3,
    pub velocity: f32,
    pub spin_velocity: f32,
    pub move_type: i64,
    pub path_points: Vec<Vec3>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldTriggerVolumeShape {
    Sphere,
    Box,
    Capsule,
}

#[derive(Component, Clone, Debug)]
pub struct WorldTriggerVolume {
    pub shape: WorldTriggerVolumeShape,
    pub center: Vec3,
    pub radius: f32,
    pub size: Vec3,
    pub height: f32,
    /// Unity CapsuleCollider axis: 0=X, 1=Y, 2=Z.
    pub direction: i64,
    pub is_trigger: bool,
    /// Matching `EpTrigger` on the same clean Unity node.
    pub trigger: Option<Entity>,
}

/// Authored rigid-body state. The clean maps publish only kinematic bodies;
/// their transform is simulated by [`WorldPlatformMotion`].
#[derive(Component, Clone, Debug)]
pub struct WorldRigidBody {
    pub mass: f32,
    pub drag: f32,
    pub angular_drag: f32,
    pub use_gravity: bool,
    pub is_kinematic: bool,
}

#[derive(Component, Clone, Debug)]
pub struct WorldWaypoint {
    pub point: Vec3,
    pub next: Option<Entity>,
    pub previous: Option<Entity>,
}

/// User-triggered world interaction forwarded by the avatar action scheduler.
#[derive(Resource, Debug, Default)]
pub struct WorldTriggerUseQueue {
    pub(super) pending: VecDeque<(Entity, Entity)>,
}

#[derive(Component, Clone, Debug)]
pub struct WorldZiplineTraversal {
    pub start: Vec3,
    pub end: Vec3,
    pub speed: f32,
    pub travelled: f32,
    pub packet_elapsed: f32,
    pub hang_height: f32,
}

#[derive(Component, Clone, Debug)]
pub struct WorldLauncherTraversal {
    pub horizontal_velocity: Vec3,
    pub packet_elapsed: f32,
    pub upward_pose: bool,
}

#[derive(Component, Clone, Debug)]
pub struct WorldRopeTraversal {
    pub points: Vec<Vec3>,
    pub speed: f32,
    pub distance: f32,
    pub total_length: f32,
    pub move_type: i64,
}

#[derive(Component, Clone, Debug)]
pub struct WorldSlopeTraversal {
    pub points: Vec<Vec3>,
    pub speed: f32,
    pub segment: usize,
    pub drift: Vec3,
    pub packet_elapsed: f32,
    pub slope_id: i32,
}

/// Clean `InsideJumppadRange`: entering a trigger only arms its power.  The
/// launch is consumed by a controller collision with that pad within 3 s.
#[derive(Component, Clone, Copy, Debug)]
pub struct WorldJumppadArmed {
    pub trigger: Entity,
    pub power: f32,
    pub remaining_seconds: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ActiveWorldLauncher {
    pub(super) trigger: Option<Entity>,
    pub(super) camera_position: Option<Vec3>,
    pub(super) custom_camera: bool,
}

impl WorldTriggerUseQueue {
    pub fn push(&mut self, actor: Entity, trigger: Entity) {
        self.pending.push_back((actor, trigger));
    }

    pub fn take_all(&mut self) -> VecDeque<(Entity, Entity)> {
        std::mem::take(&mut self.pending)
    }
}

/// Marks a tile root whose behaviour document has already been applied.
#[derive(Component, Clone, Debug)]
pub struct WorldBehavioursApplied {
    pub document: String,
    pub billboards: usize,
    pub visibility_switches: usize,
    pub effect_emitters: usize,
    pub animations: usize,
    pub triggers: usize,
    pub waypoints: usize,
    pub trigger_volumes: usize,
    pub rigid_bodies: usize,
    pub blockers: usize,
}

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeWorldBehaviourDocument {
    pub schema: String,
    pub id: String,
    pub scope: String,
    pub tile: [i32; 2],
    #[serde(default)]
    pub billboards: Vec<BillboardRecord>,
    #[serde(default)]
    pub visibility_switches: Vec<VisibilitySwitchRecord>,
    #[serde(default)]
    pub effect_emitters: Vec<EffectEmitterRecord>,
    #[serde(default)]
    pub animations: Vec<AnimationRecord>,
    #[serde(default)]
    pub animation_clips: Vec<WorldAnimationClip>,
    #[serde(default)]
    pub effect_prefab_closures: Vec<WorldEffectPrefabClosure>,
    #[serde(default)]
    pub triggers: Vec<TriggerRecord>,
    #[serde(default)]
    pub waypoints: Vec<WaypointRecord>,
    #[serde(default)]
    pub trigger_volumes: Vec<TriggerVolumeRecord>,
    #[serde(default)]
    pub rigid_bodies: Vec<RigidBodyRecord>,
    #[serde(default)]
    pub blockers: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillboardRecord {
    pub node: String,
    pub enabled: bool,
    #[serde(default)]
    pub mode: i64,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilitySwitchRecord {
    pub node: String,
    pub enabled: bool,
    pub switches: String,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectEmitterRecord {
    pub node: String,
    pub enabled: bool,
    pub controller: String,
    #[serde(default)]
    pub effect_name: Option<String>,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub max_timer: f64,
    #[serde(default)]
    pub longest_life_time: f64,
    #[serde(default)]
    pub trail_times: f64,
    #[serde(default)]
    pub conform_to_scale: bool,
    #[serde(default)]
    pub disable_update: bool,
    #[serde(default)]
    pub particle_element_count: usize,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub nif_object: Option<serde_json::Value>,
    #[serde(default)]
    pub particles: Vec<serde_json::Value>,
    #[serde(default)]
    pub resolved_particle_prefabs: Vec<Option<String>>,
    #[serde(default)]
    pub particle_elements: Vec<serde_json::Value>,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerRecord {
    pub node: String,
    pub enabled: bool,
    pub kind: String,
    #[serde(default)]
    pub server_id: i64,
    #[serde(default)]
    pub object_id: i64,
    #[serde(default)]
    pub cne_id: i64,
    #[serde(default)]
    pub trigger_type: i64,
    #[serde(default)]
    pub radius: f64,
    #[serde(default)]
    pub velocity: f64,
    #[serde(default)]
    pub speed: f64,
    #[serde(default)]
    pub add_power: f64,
    #[serde(default)]
    pub min_power: f64,
    #[serde(default)]
    pub max_power: f64,
    #[serde(default)]
    pub move_type: i64,
    #[serde(default)]
    pub waypoint_count: i64,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub start_position: [f64; 3],
    #[serde(default)]
    pub from: [f64; 3],
    #[serde(default)]
    pub to: [f64; 3],
    #[serde(default)]
    pub spin_velocity: f64,
    #[serde(default)]
    pub initial_rotation: [f64; 3],
    #[serde(default)]
    pub max_rotation: [f64; 3],
    #[serde(default)]
    pub head: Option<String>,
    #[serde(default)]
    pub tail: Option<String>,
    #[serde(default)]
    pub target_element_id: i64,
    #[serde(default)]
    pub target_element_trigger: i64,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaypointRecord {
    pub node: String,
    pub enabled: bool,
    #[serde(default)]
    pub point: [f64; 3],
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub previous: Option<String>,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerVolumeRecord {
    pub node: String,
    pub enabled: bool,
    pub kind: String,
    #[serde(default)]
    pub center: [f64; 3],
    #[serde(default)]
    pub radius: f64,
    #[serde(default)]
    pub size: Option<[f64; 3]>,
    #[serde(default)]
    pub height: f64,
    #[serde(default = "default_capsule_direction")]
    pub direction: i64,
    #[serde(default)]
    pub is_trigger: bool,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RigidBodyRecord {
    pub node: String,
    pub enabled: bool,
    #[serde(default)]
    pub mass: f64,
    #[serde(default)]
    pub drag: f64,
    #[serde(default)]
    pub angular_drag: f64,
    #[serde(default)]
    pub use_gravity: bool,
    #[serde(default)]
    pub is_kinematic: bool,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NativeWorldObjectsDocument {
    pub(super) objects: Vec<NativeWorldObjectSourceRoute>,
}

#[derive(Component)]
pub struct PendingWorldScriptedEffects {
    pub(super) document: Option<Arc<NativeWorldBehaviourDocument>>,
    pub(super) next_record: usize,
}

impl PendingWorldScriptedEffects {
    pub(super) fn document(&self) -> &Arc<NativeWorldBehaviourDocument> {
        self.document
            .as_ref()
            .expect("pending scripted effects retain their tile document")
    }

    pub(super) fn take_document(&mut self) -> Arc<NativeWorldBehaviourDocument> {
        self.document
            .take()
            .expect("completed scripted effects retain their tile document")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum WorldBehaviourSpawnPhase {
    #[default]
    Billboards,
    VisibilitySwitches,
    EffectEmitters,
    Animations,
    Triggers,
    WaypointEntities,
    WaypointLinks,
    TriggerVolumes,
    RigidBodies,
    Complete,
}

/// Incremental materialization state for one decoded world-behaviour graph.
///
/// The document stays immutable and shared with the later scripted-effect
/// admission pass. Only compact entity indices and cross-record bindings are
/// accumulated here, so unloading the tile still drops the complete graph as
/// one recursively-owned subtree.
#[derive(Component)]
pub struct PendingWorldBehaviourSpawn {
    pub(super) document: Option<Arc<NativeWorldBehaviourDocument>>,
    pub(super) animation_clips: Vec<Arc<WorldAnimationClip>>,
    pub(super) entities_by_model: HashMap<String, Vec<Entity>>,
    pub(super) authored_model_worlds: HashMap<Entity, Mat4>,
    pub(super) phase: WorldBehaviourSpawnPhase,
    pub(super) next_record: usize,
    pub(super) nodes: HashMap<String, Vec<Entity>>,
    pub(super) animation_clip_indices: HashMap<String, usize>,
    pub(super) animation_record: Option<PendingWorldAnimationRecord>,
    pub(super) waypoint_record_indices: HashMap<String, usize>,
    pub(super) trigger_entities: HashMap<String, Entity>,
    pub(super) waypoint_entities: HashMap<String, Entity>,
    pub(super) next_ring_server_id: i64,
    pub(super) native_particle_effects: usize,
    pub(super) native_scripted_effects: usize,
}

impl PendingWorldBehaviourSpawn {
    pub(super) fn new(
        document: Arc<NativeWorldBehaviourDocument>,
        animation_clips: Vec<Arc<WorldAnimationClip>>,
        entities_by_model: HashMap<String, Vec<Entity>>,
        authored_model_worlds: HashMap<Entity, Mat4>,
        native_particle_effects: usize,
        native_scripted_effects: usize,
    ) -> Self {
        let animation_clip_indices = animation_clips
            .iter()
            .enumerate()
            .map(|(index, clip)| (clip.id.clone(), index))
            .collect();
        let waypoint_record_indices = document
            .waypoints
            .iter()
            .enumerate()
            .map(|(index, waypoint)| (waypoint.node.clone(), index))
            .collect();
        Self {
            document: Some(document),
            animation_clips,
            entities_by_model,
            authored_model_worlds,
            phase: WorldBehaviourSpawnPhase::Billboards,
            next_record: 0,
            nodes: HashMap::new(),
            animation_clip_indices,
            animation_record: None,
            waypoint_record_indices,
            trigger_entities: HashMap::new(),
            waypoint_entities: HashMap::new(),
            next_ring_server_id: 1,
            native_particle_effects,
            native_scripted_effects,
        }
    }

    pub(super) fn document(&self) -> &Arc<NativeWorldBehaviourDocument> {
        self.document
            .as_ref()
            .expect("pending world behaviour spawn retains its tile document")
    }

    pub(super) fn take_document(&mut self) -> Arc<NativeWorldBehaviourDocument> {
        self.document
            .take()
            .expect("completed world behaviour spawn retains its tile document")
    }

    pub(super) fn resolve(&self, models: &[String]) -> Vec<Entity> {
        let mut entities = models
            .iter()
            .filter_map(|model| self.entities_by_model.get(model))
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        entities.sort_unstable();
        entities.dedup();
        entities
    }

    pub(super) fn finish_phase(&mut self, next: WorldBehaviourSpawnPhase) {
        self.phase = next;
        self.next_record = 0;
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WorldGpuUvCurve {
    pub(super) value_at_bind: f32,
    pub(super) velocity: f32,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WorldTriggerVolumeOccupied(pub bool);

/// The native asset root the behaviour documents are read from.
///
/// Behaviour is loaded per streamed tile rather than eagerly: the complete set
/// is tens of megabytes of JSON and only the loaded tiles are ever needed.
#[derive(Resource, Clone, Debug)]
pub struct NativeWorldBehaviourRoot(pub std::path::PathBuf);

pub(super) struct LoadedNativeWorldBehaviour {
    pub(super) document: NativeWorldBehaviourDocument,
    pub(super) animation_clips: Vec<Arc<WorldAnimationClip>>,
    pub(super) object_source_routes: NativeWorldObjectSourceRoutes,
}

pub struct WorldBehaviourPlugin;

impl Plugin for WorldBehaviourPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGameplayIntentQueue>()
            .init_resource::<WorldTriggerUseQueue>()
            .init_resource::<ActiveWorldLauncher>()
            .init_resource::<WorldSurfacePacketClock>();
        app.add_systems(
            Update,
            retire_unloading_world_behaviour_documents
                .after(apply_pending_world_behaviours)
                .before(crate::world::NativeWorldSet::Unload),
        );
        app.add_systems(
            Update,
            (
                apply_pending_world_behaviours,
                materialize_pending_world_behaviours.after(apply_pending_world_behaviours),
                enqueue_pending_world_scripted_effects.after(materialize_pending_world_behaviours),
                update_world_animations,
                apply_world_animation_samples.after(update_world_animations),
                prepare_world_animation_material_bindings
                    .after(update_world_animations)
                    .before(crate::world::NativeWorldSet::RevealPresentation),
                update_world_animation_materials
                    .after(update_world_animations)
                    .after(prepare_world_animation_material_bindings)
                    .before(crate::world::NativeWorldSet::RevealPresentation),
                update_world_billboards,
                update_world_visibility_switches,
                update_world_platforms.before(crate::world::NativeWorldSet::ResolveCollision),
                update_world_floating_icons.before(crate::world::NativeWorldSet::ResolveCollision),
            ),
        );
        // Bevy implements tuple system configs only up to a fixed arity. Keep
        // this interaction/traversal half in a second registration; all
        // cross-half ordering remains explicit through the same `.after` and
        // `.before` constraints below.
        app.add_systems(
            Update,
            (
                update_world_trigger_volumes
                    .after(crate::world_targeting::produce_world_avatar_target_feed)
                    .before(crate::avatar_action::LegacyAvatarActionSet::Resolve)
                    .before(crate::world::NativeWorldSet::ResolveCollision),
                sync_world_launcher_input
                    .after(crate::movement::LegacyMovementSet::ReadInput)
                    .before(LauncherUiSet::Interaction),
                process_world_trigger_uses.before(LauncherUiSet::Interaction),
                consume_world_launcher_outbox
                    .after(process_world_trigger_uses)
                    .after(LauncherUiSet::Interaction),
                update_world_zipline_traversals
                    .after(process_world_trigger_uses)
                    .after(crate::movement::LegacyMovementSet::Simulate)
                    .before(crate::world::NativeWorldSet::ResolveCollision),
                finish_world_zipline_steps
                    .after(update_world_zipline_traversals)
                    .after(crate::world::NativeWorldSet::ResolveCollision)
                    .before(crate::movement::LegacyMovementSet::CameraPose)
                    .before(crate::avatar_action::LegacyAvatarActionSet::VisualFeedback),
                update_world_rope_traversals
                    .after(process_world_trigger_uses)
                    .after(crate::movement::LegacyMovementSet::Simulate)
                    .before(crate::world::NativeWorldSet::ResolveCollision),
                update_world_slope_traversals
                    .after(update_world_trigger_volumes)
                    .after(crate::movement::LegacyMovementSet::Simulate)
                    .before(crate::world::NativeWorldSet::ResolveCollision),
                finish_unsupported_world_slopes
                    .after(crate::world::NativeWorldSet::ResolveCollision),
                update_world_launcher_traversals
                    .after(consume_world_launcher_outbox)
                    .after(crate::world::NativeWorldSet::ResolveCollision),
                activate_world_jumppads.after(crate::world::NativeWorldSet::ResolveCollision),
                update_world_belts.after(crate::world::NativeWorldSet::ResolveCollision),
                publish_world_platform_packets
                    .after(crate::world::NativeWorldSet::ResolveCollision),
                apply_world_launcher_camera
                    .after(consume_world_launcher_outbox)
                    .after(crate::movement::LegacyMovementSet::CameraPose),
            ),
        );
    }
}
