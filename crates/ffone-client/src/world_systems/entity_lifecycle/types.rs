use super::*;

#[derive(Debug, Default)]
pub struct NetworkEntityLifecycle0104Plugin;

impl Plugin for NetworkEntityLifecycle0104Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkEntityLifecycleIngress0104>()
            .init_resource::<NetworkNpcAttackEventQueue0104>()
            .init_resource::<NetworkNanoEffectEvents0104>()
            .init_resource::<NetworkNpcResultEffectEvents0104>()
            .init_resource::<NetworkHealingTickEffects0104>()
            .init_resource::<NetworkNpcSkillEffectEvents0104>()
            .init_resource::<NetworkNpcBarkerEventQueue0104>()
            .init_resource::<ActiveNetworkEntitySession0104>()
            .init_resource::<RemotePcRegistry0104>()
            .init_resource::<NetworkNpcRegistry0104>()
            .init_resource::<NetworkTransportationRegistry0104>()
            .init_resource::<NetworkShinyRegistry0104>()
            .init_resource::<NetworkEntityLifecycleStats0104>()
            .init_resource::<MalformedLifecycleFrames0104>()
            .init_resource::<PassthroughLifecycleFrames0104>()
            .init_resource::<IgnoredLifecycleFrames0104>()
            .init_resource::<LifecycleBootstrapDiagnostics0104>()
            .add_systems(Update, consume_network_entity_lifecycle_0104);
        crate::network_world_runtime::register_network_world_runtime_0104(app);
    }
}

/// Validated, session-owned results for one-shot Nano presentation.
#[derive(Debug, Default, Resource)]
pub struct NetworkNanoEffectEvents0104(pub VecDeque<WorldNanoAuthoritativeProjection0104>);

/// Recipient results survive independently of the caster entity (including eggs).
#[derive(Debug, Default, Resource)]
pub struct NetworkNpcResultEffectEvents0104(pub VecDeque<WorldNpcSkillAuthoritativeProjection0104>);

/// Validated NPC cast signals retain ground targets and corruption style.
#[derive(Debug, Clone)]
pub struct NetworkNpcSkillEffect0104 {
    pub signal: NpcSkillSignal0104,
    pub position: Option<[i32; 3]>,
    pub style: Option<i16>,
    pub mega: bool,
}

#[derive(Debug, Default, Resource)]
pub struct NetworkNpcSkillEffectEvents0104(pub VecDeque<NetworkNpcSkillEffect0104>);

/// Every clean source that appends a server-driven line to an NPC's Barker
/// FIFO. Rendering/content lookup stays with the gameplay UI owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkNpcBarkerEvent0104 {
    Mission(NpcBarker0104),
    SkillReady(NpcSkillSignal0104),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub struct ActiveNetworkEntitySession0104 {
    pub epoch: Option<NetworkSessionEpoch0104>,
    pub local_player_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum NetworkEntityLifecycleIngressEvent0104 {
    BeginSession {
        epoch: NetworkSessionEpoch0104,
        local_player_id: i32,
    },
    Bootstrap {
        epoch: NetworkSessionEpoch0104,
        entities: InitialAroundPacket0104,
    },
    Frame {
        epoch: NetworkSessionEpoch0104,
        frame: DecodedFrame,
    },
    Disconnect {
        epoch: NetworkSessionEpoch0104,
    },
}

#[derive(Debug, Default, Resource)]
pub struct NetworkEntityLifecycleIngress0104 {
    pub(super) events: VecDeque<NetworkEntityLifecycleIngressEvent0104>,
}

impl NetworkEntityLifecycleIngress0104 {
    pub fn begin_session(&mut self, epoch: NetworkSessionEpoch0104, local_player_id: i32) {
        self.events
            .push_back(NetworkEntityLifecycleIngressEvent0104::BeginSession {
                epoch,
                local_player_id,
            });
    }

    pub fn push_bootstrap(
        &mut self,
        epoch: NetworkSessionEpoch0104,
        entities: InitialAroundPacket0104,
    ) {
        self.events
            .push_back(NetworkEntityLifecycleIngressEvent0104::Bootstrap { epoch, entities });
    }

    pub fn push_frame(&mut self, epoch: NetworkSessionEpoch0104, frame: DecodedFrame) {
        self.events
            .push_back(NetworkEntityLifecycleIngressEvent0104::Frame { epoch, frame });
    }

    pub fn disconnect(&mut self, epoch: NetworkSessionEpoch0104) {
        self.events
            .push_back(NetworkEntityLifecycleIngressEvent0104::Disconnect { epoch });
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkRemotePc0104 {
    pub pc_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub struct NetworkPcAppearance0104(pub PcAppearance0104);

/// Typed hand-off from network appearance to the native modular-avatar resolver.
#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub struct PendingPcVisual0104 {
    pub pc_id: i32,
    pub style: PcStyle0104,
    pub equipment: [ItemBase0104; 9],
    pub nano: Nano0104,
    pub render_type: i32,
}

impl From<&PcAppearance0104> for PendingPcVisual0104 {
    fn from(appearance: &PcAppearance0104) -> Self {
        Self {
            pc_id: appearance.id,
            style: appearance.style.clone(),
            equipment: appearance.equipment,
            nano: appearance.nano,
            render_type: appearance.render_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkNpc0104 {
    pub npc_id: i32,
    pub npc_type: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkNpcAppearance0104(pub NpcAppearance0104);

/// Typed hand-off from NPC table identity to the semantic native GLB resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct PendingNpcVisual0104 {
    pub npc_id: i32,
    pub npc_type: i32,
    pub barker_type: i32,
}

impl From<NpcAppearance0104> for PendingNpcVisual0104 {
    fn from(appearance: NpcAppearance0104) -> Self {
        Self {
            npc_id: appearance.npc_id,
            npc_type: appearance.npc_type,
            barker_type: appearance.barker_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct NetworkNpcMotion0104 {
    pub destination: Vec3,
    pub speed: f32,
    pub move_style: i16,
}

/// One non-looping animation request proven by a server NPC combat packet.
/// The revision distinguishes repeated attacks that request the same clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkNpcCombatAnimation0104 {
    pub revision: u64,
    pub clip: NetworkNpcCombatClip0104,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkNpcCombatClip0104 {
    Melee,
    Wound,
    Skill,
    Skill1,
    Skill2,
    Skill3,
    SkillReady,
    MegaReady,
    Mega,
    CorruptionReady,
    Corruption,
}

impl NetworkNpcCombatClip0104 {
    pub(crate) const fn high_layer(self) -> Option<usize> {
        match self {
            Self::Melee => Some(0),
            Self::Wound => Some(1),
            _ => None,
        }
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Melee => "melee1",
            Self::Wound => "wound",
            Self::Skill => "skill0",
            Self::Skill1 => "skill1",
            Self::Skill2 => "skill2",
            Self::Skill3 => "skill3",
            Self::SkillReady => "readyspell",
            Self::MegaReady => "megatak_readyspell",
            Self::Mega => "skill0",
            Self::CorruptionReady => "corruptak_readyspell",
            Self::Corruption => "corruptak",
        }
    }

    /// Spell-preparation states are persistent low-layer poses in the
    /// clean `NpcAnimation` state machine. Every other combat clip is an
    /// event-bounded state. High-layer completion returns to ready; low-layer
    /// skill and corruption completion selects stand.
    #[must_use]
    pub const fn repeats(self) -> bool {
        matches!(
            self,
            Self::MegaReady | Self::CorruptionReady | Self::SkillReady
        )
    }

    /// Clean NPC models which do not expose a dedicated spell-ready clip can
    /// still retain the ordinary combat-ready pose. Attack and hit clips do
    /// not silently substitute an unrelated melee animation.
    #[must_use]
    pub const fn fallback_name(self) -> Option<&'static str> {
        match self {
            Self::MegaReady | Self::CorruptionReady | Self::SkillReady => Some("ready"),
            _ => None,
        }
    }
}

/// Low-layer `AttackReady` continuation recovered from the clean Unity
/// `NpcAnimation.EndAnimation` implementation. Melee and wound one-shots
/// enter this state; authoritative movement or a new combat request leaves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) struct NetworkNpcReadyAnimation0104;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(super) enum NetworkNpcSkillPhase0104 {
    MegaReady,
    CorruptionReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkTransportation0104 {
    pub transportation_kind: i32,
    pub id: i32,
    pub transportation_type: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkTransportationAppearance0104(pub TransportationAppearance0104);

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct NetworkTransportationMotion0104 {
    pub destination: Vec3,
    pub speed: f32,
    pub move_style: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkShiny0104 {
    pub shiny_id: i32,
    pub shiny_type: i32,
    pub map_number: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkShinyAppearance0104(pub ShinyAppearance0104);

#[derive(Debug, Default, Resource)]
pub struct RemotePcRegistry0104 {
    pub(super) entities: BTreeMap<i32, Entity>,
}

impl RemotePcRegistry0104 {
    pub fn get(&self, pc_id: i32) -> Option<Entity> {
        self.entities.get(&pc_id).copied()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, Entity)> + '_ {
        self.entities.iter().map(|(id, entity)| (*id, *entity))
    }

    pub(super) fn insert(&mut self, pc_id: i32, entity: Entity) {
        self.entities.insert(pc_id, entity);
    }

    pub(super) fn remove(&mut self, pc_id: i32) -> Option<Entity> {
        self.entities.remove(&pc_id)
    }

    pub(super) fn clear(&mut self) {
        self.entities.clear();
    }
}

#[derive(Debug, Default, Resource)]
pub struct NetworkNpcRegistry0104 {
    pub(super) entities: BTreeMap<i32, Entity>,
}

impl NetworkNpcRegistry0104 {
    pub fn get(&self, npc_id: i32) -> Option<Entity> {
        self.entities.get(&npc_id).copied()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, Entity)> + '_ {
        self.entities.iter().map(|(id, entity)| (*id, *entity))
    }

    pub(super) fn insert(&mut self, npc_id: i32, entity: Entity) {
        self.entities.insert(npc_id, entity);
    }

    pub(super) fn remove(&mut self, npc_id: i32) -> Option<Entity> {
        self.entities.remove(&npc_id)
    }

    pub(super) fn clear(&mut self) {
        self.entities.clear();
    }
}

#[derive(Debug, Default, Resource)]
pub struct NetworkTransportationRegistry0104 {
    pub(super) entities: BTreeMap<(i32, i32), Entity>,
}

impl NetworkTransportationRegistry0104 {
    pub fn get(&self, transportation_kind: i32, id: i32) -> Option<Entity> {
        self.entities.get(&(transportation_kind, id)).copied()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = ((i32, i32), Entity)> + '_ {
        self.entities.iter().map(|(key, entity)| (*key, *entity))
    }

    pub(super) fn insert(&mut self, transportation_kind: i32, id: i32, entity: Entity) {
        self.entities.insert((transportation_kind, id), entity);
    }

    pub(super) fn remove(&mut self, transportation_kind: i32, id: i32) -> Option<Entity> {
        self.entities.remove(&(transportation_kind, id))
    }

    pub(super) fn clear(&mut self) {
        self.entities.clear();
    }
}

#[derive(Debug, Default, Resource)]
pub struct NetworkShinyRegistry0104 {
    pub(super) entities: BTreeMap<i32, Entity>,
}

impl NetworkShinyRegistry0104 {
    pub fn get(&self, shiny_id: i32) -> Option<Entity> {
        self.entities.get(&shiny_id).copied()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, Entity)> + '_ {
        self.entities.iter().map(|(id, entity)| (*id, *entity))
    }

    pub(super) fn insert(&mut self, shiny_id: i32, entity: Entity) {
        self.entities.insert(shiny_id, entity);
    }

    pub(super) fn remove(&mut self, shiny_id: i32) -> Option<Entity> {
        self.entities.remove(&shiny_id)
    }

    pub(super) fn clear(&mut self) {
        self.entities.clear();
    }
}

#[derive(Debug, Default, Resource)]
pub struct NetworkEntityLifecycleStats0104 {
    pub sessions_started: u64,
    pub sessions_cleared: u64,
    pub bootstrap_packets: u64,
    pub live_frames: u64,
    pub player_upserts: u64,
    pub npc_upserts: u64,
    pub player_despawns: u64,
    pub npc_despawns: u64,
    pub npc_damage_updates: u64,
    pub transportation_upserts: u64,
    pub shiny_upserts: u64,
    pub transportation_despawns: u64,
    pub shiny_despawns: u64,
    pub local_player_appearances_ignored: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedLifecycleFrame0104 {
    pub epoch: NetworkSessionEpoch0104,
    pub frame: DecodedFrame,
    pub error: EntityLifecycleDecodeError0104,
}

#[derive(Debug, Default, Resource)]
pub struct MalformedLifecycleFrames0104 {
    pub frames: Vec<MalformedLifecycleFrame0104>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassthroughLifecycleFrame0104 {
    pub epoch: NetworkSessionEpoch0104,
    pub frame: DecodedFrame,
}

#[derive(Debug, Default, Resource)]
pub struct PassthroughLifecycleFrames0104 {
    pub frames: Vec<PassthroughLifecycleFrame0104>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEntityKind0104 {
    Player,
    Npc,
    Transportation,
    Shiny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoredLifecycleFrame0104 {
    pub epoch: NetworkSessionEpoch0104,
    pub frame: DecodedFrame,
    pub reason: IgnoredLifecycleFrameReason0104,
}

#[derive(Debug, Default, Resource)]
pub struct IgnoredLifecycleFrames0104 {
    pub frames: Vec<IgnoredLifecycleFrame0104>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoredLifecycleBootstrap0104 {
    pub epoch: NetworkSessionEpoch0104,
    pub active_epoch: Option<NetworkSessionEpoch0104>,
    pub entities: InitialAroundPacket0104,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassthroughLifecycleBootstrap0104 {
    pub epoch: NetworkSessionEpoch0104,
    pub entities: InitialAroundPacket0104,
}

#[derive(Debug, Default, Resource)]
pub struct LifecycleBootstrapDiagnostics0104 {
    pub stale: Vec<IgnoredLifecycleBootstrap0104>,
    pub passthrough: Vec<PassthroughLifecycleBootstrap0104>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedEntityLifecyclePacket0104 {
    PcAround(Vec<PcAppearance0104>),
    PcNew(PcNew0104),
    PcExit(PcExit0104),
    AroundDelPc(AroundDelPc0104),
    PcMotion(DecodedRemotePacket),
    PcRegen(PcRegen0104),
    PcStateChange {
        pc_id: i32,
        state: i8,
    },
    PcStyleChange { pc_id: i32, style: PcStyle0104 },
    PcEquipmentChange(ffone_protocol::EquipChangePacket0104),
    PcSuddenDead(PcSuddenDead0104),
    NpcAround(Vec<NpcAppearance0104>),
    NpcEnter(NpcEnter0104),
    NpcExit(NpcExit0104),
    NpcMove(NpcMove0104),
    NpcNew(NpcNew0104),
    AroundDelNpc(AroundDelNpc0104),
    NpcAttackResults(Vec<AttackResult0104>),
    NpcAttackPcs(NpcAttackPcs0104),
    /// `PC_ATTACK_CHARs`/`PC_ATTACK_CHARs_SUCC`: PvP-mode hitscan results the
    /// clean client routes by `eCT` (`4` NPC, otherwise player).
    MixedAttackResults(Vec<AttackResult0104>),
    RemotePcAttack { pc_id: i32, results: Vec<AttackResult0104>, npc_only: bool },
    NpcAttackChars(NpcAttackChars0104),
    CharacterAttackCharacters(CharacterAttackCharacters0104),
    NpcBarker(NpcBarker0104),
    NanoSkillAuthority(WorldNanoAuthoritativeProjection0104),
    BuffTimeout(CharTimeBuffTimeout0104),
    HealingTick(ffone_protocol::TimeBuffHealTick0104),
    NpcSkill {
        signal: NpcSkillSignal0104,
        authority: Option<WorldNpcSkillAuthoritativeProjection0104>,
    },
    TransportationAround(Vec<TransportationAppearance0104>),
    TransportationUpsert(TransportationAppearance0104),
    TransportationExit(TransportationExit0104),
    TransportationMove(TransportationMove0104),
    AroundDelTransportation(AroundDelTransportation0104),
    ShinyAround(Vec<ShinyAppearance0104>),
    ShinyUpsert(ShinyAppearance0104),
    ShinyExit(ShinyExit0104),
    AroundDelShiny(AroundDelShiny0104),
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct RemotePcAuthorityUpdate0104 {
    // `PcAppearance0104` has no weapon/Nano battery, active-slot or
    // eCSTB-deletion fields. Those lossless values remain available on the
    // decoded projection; lifecycle mirrors every field its remote appearance
    // contract can actually own.
    pub(super) absolute_hp: Option<i32>,
    pub(super) absolute_condition_bit_flag: Option<i32>,
    pub(super) absolute_nano_id: Option<i16>,
    pub(super) absolute_nano_skill_id: Option<i16>,
    pub(super) absolute_nano_stamina: Option<i16>,
    pub(super) nano_deactivated: Option<bool>,
    pub(super) movement: Option<(i32, [i32; 3])>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct NpcAuthorityUpdate0104 {
    pub(super) absolute_hp: Option<i32>,
    pub(super) absolute_condition_bit_flag: Option<i32>,
    pub(super) movement: Option<[i32; 3]>,
}

/// Instance-local visibility transition. Retiring roots no longer own a server ID.
#[derive(Debug, Clone, Copy, Component)]
pub struct RemotePcVisibility0104 {
    pub alpha: f32,
    pub retiring: bool,
}
impl Default for RemotePcVisibility0104 {
    fn default() -> Self { Self { alpha: 0.0, retiring: false } }
}
