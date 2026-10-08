use super::*;

/// Asset and animation route for an ordinary-world Nano. The protocol loadout
/// remains authoritative; this value owns presentation only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldNanoGameplayPresentation {
    pub model_path: String,
    pub style: u8,
    /// Clean Nano tune-card position, one based (`skill1`..=`skill3`).
    pub skill_slot: u8,
}

impl WorldNanoGameplayPresentation {
    #[must_use]
    pub const fn skill_clip(&self) -> Option<&'static str> {
        match self.skill_slot {
            1 => Some("skill1"),
            2 => Some("skill2"),
            3 => Some("skill3"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialNanoGameplayLoadout {
    pub nano_id: i16,
    pub skill_id: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialNanoGameplayIssue {
    UnsupportedLoadout { nano_id: i16, skill_id: i16 },
    MissingOwner { owner: Entity },
    AssetBlocked(String),
}

#[derive(Debug, Default, Resource)]
pub struct TutorialNanoGameplayIssueQueue {
    pub(super) pending: VecDeque<TutorialNanoGameplayIssue>,
}

impl TutorialNanoGameplayIssueQueue {
    pub(super) fn push(&mut self, issue: TutorialNanoGameplayIssue) {
        self.pending.push_back(issue);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialNanoGameplayIssue> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TutorialGameplayNanoRoot {
    pub owner: Entity,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TutorialNanoOrbitPattern {
    None,
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TutorialNanoPulsePattern {
    None,
    Sine,
    Cosine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TutorialNanoTurnPattern {
    None,
    Clockwise,
    CounterClockwise,
}

/// Exact mutable fields owned by clean `NanoMoveController` after SetParent.
/// The lateral offset deliberately lives in world space: turning the player
/// after a summon does not rotate the Nano around them.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(super) struct TutorialNanoMovementPattern {
    pub(super) pattern_started_seconds: f32,
    pub(super) lateral_offset: Vec3,
    pub(super) orbit: TutorialNanoOrbitPattern,
    pub(super) pulse: TutorialNanoPulsePattern,
    pub(super) turn: TutorialNanoTurnPattern,
    pub(super) orbit_speed_radians: f32,
    pub(super) turn_speed_radians: f32,
    pub(super) pulse_height: f32,
    pub(super) pulse_hz: f32,
}

impl TutorialNanoMovementPattern {
    pub(super) fn new(owner: &Transform, now_seconds: f32, random: &mut LegacyNanoStandRandomStream) -> Self {
        let mut pattern = Self {
            pattern_started_seconds: now_seconds,
            lateral_offset: owner.rotation * Vec3::new(0.7, 0.0, 0.0),
            orbit: TutorialNanoOrbitPattern::None,
            pulse: TutorialNanoPulsePattern::None,
            turn: TutorialNanoTurnPattern::None,
            orbit_speed_radians: 0.0,
            turn_speed_radians: 0.0,
            pulse_height: 0.0,
            pulse_hz: 0.0,
        };
        pattern.randomize(now_seconds, random);
        pattern
    }

    pub(super) fn randomize(&mut self, now_seconds: f32, random: &mut LegacyNanoStandRandomStream) {
        self.pattern_started_seconds = now_seconds;
        self.orbit = match random.next_index(10) {
            0 => TutorialNanoOrbitPattern::Clockwise,
            1 => TutorialNanoOrbitPattern::CounterClockwise,
            _ => TutorialNanoOrbitPattern::None,
        };
        self.pulse = match random.next_index(3) {
            0 => TutorialNanoPulsePattern::Sine,
            1 => TutorialNanoPulsePattern::Cosine,
            _ => TutorialNanoPulsePattern::None,
        };
        self.turn = match random.next_index(10) {
            0 => TutorialNanoTurnPattern::CounterClockwise,
            1 => TutorialNanoTurnPattern::Clockwise,
            _ => TutorialNanoTurnPattern::None,
        };
        // Clean generates degrees with `180/3.14`, while Unity consumes them
        // with its actual degree-to-radian conversion. Preserve that tiny
        // PI/3.14 multiplier instead of rounding it away.
        let legacy_degree_factor = std::f32::consts::PI / 3.14;
        self.orbit_speed_radians = (1.0 + random.next_unit_f32()) * legacy_degree_factor;
        self.turn_speed_radians = (0.5 + random.next_unit_f32() * 1.5) * legacy_degree_factor;
        self.pulse_height = 0.1 + random.next_unit_f32() * 0.2;
        self.pulse_hz = 0.5 + random.next_unit_f32();
    }

    pub(super) fn pulse_offset(self) -> f32 {
        let phase = self.pulse_hz * self.pattern_started_seconds * 3.14;
        match self.pulse {
            TutorialNanoPulsePattern::None => 0.0,
            TutorialNanoPulsePattern::Sine => phase.sin() * self.pulse_height,
            TutorialNanoPulsePattern::Cosine => phase.cos() * self.pulse_height,
        }
    }
}

#[derive(Debug, Component)]
pub struct TutorialGameplayNanoScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TutorialNanoSkillCondition(pub i32);

#[derive(Default, Resource)]
pub(super) struct TutorialNanoGameplayAssets {
    pub(super) gltf: Option<Handle<Gltf>>,
    pub(super) scene: Option<Handle<WorldAsset>>,
    pub(super) graph: Option<Handle<AnimationGraph>>,
    pub(super) nodes: BTreeMap<&'static str, AnimationNodeIndex>,
    pub(super) sound_events: Option<Arc<[NetworkNpcAnimationSoundEvent0104]>>,
    pub(super) corruption_events: Vec<corruption::ParticleEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum TutorialNanoGameplaySet {
    AdvanceCooldown,
    ApplyCommands,
    PrepareAsset,
    BindMaterials,
    PlayAnimation,
    AnimationEvents,
    Follow,
}

pub struct TutorialNanoGameplayPlugin;

impl Plugin for TutorialNanoGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialNanoGameplayCommandQueue>()
            .init_resource::<TutorialNanoGameplayEventQueue>()
            .init_resource::<TutorialNanoGameplayIssueQueue>()
            .init_resource::<TutorialNanoGameplayState>()
            .init_resource::<TutorialNanoGameplayAssets>()
            .init_resource::<LegacyNanoStandRandomStream>()
            .configure_sets(
                Update,
                (
                    TutorialNanoGameplaySet::AdvanceCooldown,
                    TutorialNanoGameplaySet::ApplyCommands,
                    TutorialNanoGameplaySet::PrepareAsset,
                    TutorialNanoGameplaySet::BindMaterials,
                    TutorialNanoGameplaySet::PlayAnimation,
                    TutorialNanoGameplaySet::AnimationEvents,
                    TutorialNanoGameplaySet::Follow,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                advance_tutorial_nano_skill_cooldown
                    .in_set(TutorialNanoGameplaySet::AdvanceCooldown),
            )
            .add_systems(
                Update,
                apply_tutorial_nano_gameplay_commands
                    .in_set(TutorialNanoGameplaySet::ApplyCommands),
            )
            .add_systems(
                Update,
                prepare_tutorial_nano_gameplay_asset.in_set(TutorialNanoGameplaySet::PrepareAsset),
            )
            .add_systems(
                Update,
                bind_tutorial_gameplay_nano_face_texture
                    .in_set(TutorialNanoGameplaySet::BindMaterials),
            )
            .add_systems(
                Update,
                play_tutorial_nano_gameplay_animation
                    .in_set(TutorialNanoGameplaySet::PlayAnimation),
            )
            .add_systems(
                Update,
                emit_tutorial_nano_animation_events
                    .in_set(TutorialNanoGameplaySet::AnimationEvents),
            )
            .add_systems(
                Update,
                emit_world_nano_animation_sounds
                    .after(TutorialNanoGameplaySet::PlayAnimation)
                    .in_set(GameplayAudioSet::Collect),
            )
            .add_systems(
                Update,
                follow_tutorial_gameplay_nano.in_set(TutorialNanoGameplaySet::Follow),
            );
    }
}
