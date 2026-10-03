use super::*;

pub(super) const EFFECT_RENDERER_STATUS: &str =
    "exact-unity-serialized-closure-published; native-unity-particle-renderer-required";

pub(super) const PROJECTILE_RENDERER_STATUS: &str =
    "exact-bullettable-row-published; native-unity-effect-and-oni-projectile-renderer-required";

#[derive(Clone, Debug, PartialEq)]
pub enum TutorialEffectRuntimeCommand {
    Preload {
        effect_id: i32,
        source_line: u32,
    },
    Add {
        effect_id: i32,
        placement: TutorialEffectPlacement,
        scale: f32,
        tracked: bool,
        name: Option<String>,
        destroy_after_seconds: Option<f32>,
        source_line: u32,
    },
    ClearTracked {
        source_line: u32,
    },
    DestroyNamed {
        name: String,
        source_line: u32,
    },
    /// One player-weapon projectile driven by its exact `BulletTable` row.
    Projectile {
        bullet_type: i32,
        source: Vec3,
        target: Vec3,
        /// `BulletMoveScript.Crash` returns before loading the success effect
        /// when `m_TargetObj` is null (except the unused damage-type-2 path).
        target_exists: bool,
        /// Exact Nano/NPC style values passed to `AvatarUtil.MakeBullet`.
        source_style: i32,
        target_style: i32,
        motion: TutorialProjectileMotion,
        source_line: u32,
    },
    ProjectilePair {
        types: [i32; 2],
        source: Vec3,
        target: Vec3,
        oni: bool,
        priority: i32,
        source_line: u32,
    },
    /// Operational Oni path. `sampled_initial_velocity` is captured at the
    /// call site because substituting a Rust RNG would not reproduce Unity's
    /// global `Random` stream.
    ProjectilePairSampled {
        types: [i32; 2],
        source: Vec3,
        target: Vec3,
        oni: bool,
        priority: i32,
        reverse: bool,
        sampled_initial_velocity: [Vec3; 2],
        source_line: u32,
    },
}

impl TutorialEffectRuntimeCommand {
    pub(super) fn source_line(&self) -> u32 {
        match self {
            Self::Preload { source_line, .. }
            | Self::Add { source_line, .. }
            | Self::ClearTracked { source_line }
            | Self::DestroyNamed { source_line, .. }
            | Self::Projectile { source_line, .. }
            | Self::ProjectilePair { source_line, .. }
            | Self::ProjectilePairSampled { source_line, .. } => *source_line,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TutorialEffectRuntimeDisposition {
    SerializedClosurePreloaded,
    NativeEffectQueued {
        instance_id: u64,
        rendered_nodes: usize,
        blocked_nodes: usize,
    },
    NativeProjectilePairQueued {
        instance_ids: [u64; 2],
        rendered_nodes: usize,
        blocked_nodes: usize,
    },
    NativeProjectileQueued {
        instance_id: u64,
        rendered_nodes: usize,
        blocked_nodes: usize,
    },
    ClearedTrackedInstances {
        count: usize,
    },
    DestroyedNamedInstance {
        instance_id: u64,
    },
    RejectedFailClosed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TutorialEffectRuntimeRecord {
    pub command: TutorialEffectRuntimeCommand,
    pub disposition: TutorialEffectRuntimeDisposition,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TutorialEffectRuntimeIssue {
    ExactCatalogUnavailable {
        source_line: u32,
    },
    EffectOutsideExactCatalog {
        effect_id: i32,
        source_line: u32,
    },
    InvalidEffectTransform {
        effect_id: i32,
        source_line: u32,
    },
    NativeParticleRendererUnavailable {
        effect_id: i32,
        container_route: String,
        closure_path: String,
        closure_blake3: String,
        closure_object_count: u64,
        component_types: Vec<String>,
        exact_bone: Option<(i32, String)>,
        source_line: u32,
    },
    NamedEffectWasNotInstantiated {
        name: String,
        source_line: u32,
    },
    ProjectilePairOutsideExactContract {
        types: [i32; 2],
        source_line: u32,
    },
    ProjectileOutsideExactContract {
        bullet_type: i32,
        source_line: u32,
    },
    InvalidProjectileEndpoints {
        source_line: u32,
    },
    NativeProjectileRendererUnavailable {
        types: [i32; 2],
        row_blake3: [String; 2],
        effect_scripts: [[i32; 3]; 2],
        particle_closure_blake3: [String; 2],
        particle_component_types: [Vec<String>; 2],
        source_line: u32,
    },
    UnsupportedSerializedClosureNode {
        effect_id: i32,
        asset: String,
        path_id: i64,
        object_type: String,
        reason: TutorialNativeClosureBlockerReason,
        source_line: u32,
    },
    ProjectileRuntimeSamplesRequired {
        types: [i32; 2],
        reverse: Option<bool>,
        required_draw_count_normal: usize,
        required_draw_count_reverse: usize,
        source_line: u32,
    },
    InvalidProjectileVelocitySample {
        projectile_index: usize,
        sample: Vec3,
        reverse: bool,
        source_line: u32,
    },
    ExactBoneAttachmentUnavailable {
        actor_id: i32,
        node_name: String,
        match_count: usize,
        source_line: u32,
    },
    ExactEntityBoneAttachmentUnavailable {
        root_entity: Entity,
        node_name: String,
        match_count: usize,
        source_line: u32,
    },
    NativePreloadFailed {
        effect_id: i32,
        asset_path: String,
        detail: String,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialEffectRuntime {
    pub(super) native_catalog: BTreeMap<i32, tutorial_native_effects::NativeEffectPlan>,
    pub(super) native_skill_projectiles: BTreeMap<i32, native_skill_projectiles::SkillProjectile>,
    pub(super) library: Option<Arc<TutorialEffectLibrary>>,
    pub(super) pending: VecDeque<TutorialEffectRuntimeCommand>,
    pub(super) streamed_world_pending: VecDeque<TutorialEffectRuntimeCommand>,
    pub(super) issues: VecDeque<TutorialEffectRuntimeIssue>,
    pub(super) records: VecDeque<TutorialEffectRuntimeRecord>,
    pub(super) preloaded: BTreeSet<i32>,
    pub(super) native_preload_complete: BTreeSet<i32>,
    pub(super) native_preload_failures_reported: BTreeSet<i32>,
    pub(super) next_instance_id: u64,
    pub(super) active: BTreeMap<u64, ActiveNativeInstance>,
    pub(super) tracked: BTreeSet<u64>,
    pub(super) named: BTreeMap<String, u64>,
    pub(super) native_preloads: VecDeque<tutorial_native_effects::NativePreloadRequest>,
    pub(super) native_spawns: VecDeque<tutorial_native_effects::NativeSpawnRequest>,
    pub(super) native_despawns: VecDeque<u64>,
}
