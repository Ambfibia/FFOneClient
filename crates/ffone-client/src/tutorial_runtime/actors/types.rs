use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct TutorialActor {
    pub id: i32,
    pub npc_type: i32,
    pub team: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub damaged: bool,
    pub interacting: bool,
    /// Mirrors the tutorial-global `bDontKill` decision on the demo monster.
    pub invulnerable: bool,
}

impl TutorialActor {
    #[must_use]
    pub const fn is_alive(self) -> bool {
        self.hp > 0
    }
}

#[derive(Debug, Component)]
pub struct TutorialActorScene;

#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub enum TutorialActorVisualUnavailable0104 {
    HiddenLocation,
    MissingRoute(String),
}

/// Runtime-only continuation for the low/high animation states recovered from
/// `NpcAnimation`. Authored one-shot cutscene poses deliberately do not receive
/// one of these continuations and retain their original clamp behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TutorialActorCombatTransition {
    pub(super) request_serial: u64,
    pub(super) completion: TutorialActorCombatCompletion,
}

/// Arms the visual payload owned by Retrobution's `DeadMotion`, as opposed to
/// a cutscene which merely forces the semantic `death` clip. The original
/// component only generates its ES372/Oni payload when the killing transform
/// is tagged Player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) struct TutorialActorPlayerKillDeathPresentation {
    pub request_serial: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TutorialActorCombatCompletion {
    Ready,
    Despawn,
}

/// App-lifetime compatibility stream for `NpcAnimation.ForceStandMotion`.
///
/// Retrobution consumes Unity's app-global `Random.Range(0, 100)` stream, but
/// neither that global seed nor its internal state is serialized. We preserve
/// the proven one draw made by `NpcAnimation.SetModel -> StandMotion` for a
/// newly spawned actor that has no pending forced pose, plus one draw per
/// accepted explicit `idle` command. The exact half-open integer range stays
/// app-global across tutorial transitions; asset retries, state changes, and
/// `ForceUpdate` consume no additional draw.
#[derive(Debug, Resource)]
pub struct TutorialActorStandRandomStream {
    pub(super) state: u32,
    pub(super) draw_count: u64,
}

impl Default for TutorialActorStandRandomStream {
    fn default() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let folded = nanos ^ (nanos >> 32) ^ (nanos >> 64) ^ (nanos >> 96);
        Self::with_seed(folded as u32)
    }
}

impl TutorialActorStandRandomStream {
    pub(super) const FALLBACK_SEED: u32 = 0x57a9_d1e5;

    pub(super) fn with_seed(seed: u32) -> Self {
        Self {
            state: if seed == 0 { Self::FALLBACK_SEED } else { seed },
            draw_count: 0,
        }
    }

    pub(super) fn draw_percent(&mut self) -> u32 {
        self.draw_count = self.draw_count.wrapping_add(1);
        advance_native_xorshift32(&mut self.state) % 100
    }

    /// Number of accepted semantic `idle` commands consumed by this stream.
    #[must_use]
    pub const fn draw_count(&self) -> u64 {
        self.draw_count
    }
}

/// Continuous native-space movement requested by recovered `MoveNpc` calls.
/// The command executor never teleports a moving actor to its destination.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct TutorialActorMotion {
    pub target: Vec3,
    pub speed_units_per_second: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialActorIssue {
    UnknownNpcType {
        id: i32,
        npc_type: i32,
    },
    MissingActor {
        id: i32,
    },
    InvalidDamage {
        id: i32,
        amount: i32,
    },
    InvalidMotionSpeed {
        id: i32,
        speed_units_per_second: f32,
    },
    RigAnimationUnavailable {
        id: i32,
        clip: &'static str,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialActorIssueQueue {
    pub(super) pending: VecDeque<TutorialActorIssue>,
}

impl TutorialActorIssueQueue {
    pub(super) fn push(&mut self, issue: TutorialActorIssue) {
        self.pending.push_back(issue);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<TutorialActorIssue> {
        self.pending.pop_front()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialActorIssue> {
        std::mem::take(&mut self.pending)
    }
}

/// Original level-1 cone parameters:
/// `m_pAvatarData[1] = view 15/2500, attack 90/200`. Distances are converted
/// from protocol centiunits. `cnAvatarAttack` forces talk range to `6` while
/// `cntutorialscript.bTutorial` is active.
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct TutorialTargetingProfile {
    pub player_height: f32,
    pub view_half_angle_degrees: f32,
    pub view_distance: f32,
    pub attack_half_angle_degrees: f32,
    pub attack_distance: f32,
    pub talk_distance: f32,
}

impl Default for TutorialTargetingProfile {
    fn default() -> Self {
        Self {
            player_height: AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
            view_half_angle_degrees: 15.0,
            view_distance: 25.0,
            attack_half_angle_degrees: 90.0,
            attack_distance: 2.0,
            talk_distance: 6.0,
        }
    }
}

/// Camera-derived horizontal direction. The original cone list uses the
/// gameplay camera yaw, not the avatar root yaw. If absent, the producer uses
/// the root's conventional Bevy `-Z` forward.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct TutorialTargetingFacing(pub Vec3);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialActorObservationSample {
    pub id: i32,
    pub position: Vec3,
    pub damaged: bool,
    pub interacting: bool,
}

#[derive(Debug, Clone, PartialEq, Resource, Default)]
pub struct TutorialNpcObservationSnapshot(pub TutorialNpcObservation);

impl TutorialNpcObservationSnapshot {
    #[must_use]
    pub const fn get(&self) -> &TutorialNpcObservation {
        &self.0
    }
}

#[derive(Debug, Default, Resource)]
pub struct TutorialActorCombatConfig {
    pub demo_dont_kill: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum TutorialActorSet {
    ApplyCommands,
    AdvanceMotion,
    GroundActors,
    AdvanceCombatAnimation,
    ProduceTargets,
    Observe,
}

pub struct TutorialActorPlugin;

impl Plugin for TutorialActorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialActorRegistry>()
            .init_resource::<TutorialActorCommandQueue>()
            .init_resource::<TutorialActorEventQueue>()
            .init_resource::<TutorialActorIssueQueue>()
            .init_resource::<TutorialTargetingProfile>()
            .init_resource::<TutorialNpcObservationSnapshot>()
            .init_resource::<TutorialActorCombatConfig>()
            .init_resource::<TutorialActorAnimationAssets>()
            .init_resource::<TutorialActorStandRandomStream>()
            .configure_sets(
                Update,
                (
                    TutorialActorSet::ApplyCommands,
                    TutorialActorSet::AdvanceMotion,
                    TutorialActorSet::ProduceTargets,
                    TutorialActorSet::Observe,
                )
                    .chain(),
            )
            .configure_sets(
                Update,
                TutorialActorSet::GroundActors
                    .after(TutorialActorSet::AdvanceMotion)
                    .after(NativeWorldSet::ResolveCollision),
            )
            .configure_sets(
                Update,
                TutorialActorSet::AdvanceCombatAnimation.after(TutorialActorSet::GroundActors),
            )
            .add_systems(
                Update,
                apply_tutorial_actor_commands.in_set(TutorialActorSet::ApplyCommands),
            )
            .add_systems(
                Update,
                advance_tutorial_actor_motion.in_set(TutorialActorSet::AdvanceMotion),
            )
            .add_systems(
                Update,
                ground_tutorial_actors.in_set(TutorialActorSet::GroundActors),
            )
            .add_systems(
                Update,
                advance_tutorial_actor_combat_animation
                    .in_set(TutorialActorSet::AdvanceCombatAnimation),
            )
            .add_systems(
                Update,
                produce_tutorial_avatar_target_feed.in_set(TutorialActorSet::ProduceTargets),
            )
            .add_systems(
                Update,
                refresh_tutorial_npc_observation.in_set(TutorialActorSet::Observe),
            );
    }
}
