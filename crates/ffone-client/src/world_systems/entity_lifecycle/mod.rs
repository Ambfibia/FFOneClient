//! Native OpenFusion 0104 player/NPC lifecycle state.
//!
//! This module deliberately owns only network identity, appearance, lifecycle,
//! and renderer-resolution requests. It never creates placeholder geometry.
//! A native GLB resolver consumes [`PendingPcVisual0104`] and
//! [`PendingNpcVisual0104`] after matching semantic assets.

use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
};

use bevy::prelude::*;
use ffone_protocol::{
    AroundDecodeError, AroundDelNpc0104, AroundDelPc0104, AroundDelShiny0104,
    AroundDelTransportation0104, AttackResult0104, CharTimeBuffTimeout0104,
    CharacterAttackCharacters0104, CountedPayloadError0104, DecodedFrame, InitialAroundPacket0104,
    ItemBase0104, Nano0104, NpcAppearance0104, NpcAttackChars0104, NpcAttackPcs0104, NpcBarker0104,
    NpcEnter0104, NpcExit0104, NpcMove0104, NpcNew0104, NpcSkillSignal0104, NpcSkillSignalKind0104,
    PayloadError, PcAppearance0104, PcAttackChars0104, PcAttackCharsSuccess0104, PcAttackNpcs0104,
    PcAttackNpcsSuccess0104, PcExit0104, PcJump0104, PcMove0104, PcNew0104, PcRegen0104,
    PcRegenPacket0104, PcStop0104, PcStyle0104, PcSuddenDead0104, ShinyAppearance0104,
    ShinyExit0104, TransportationAppearance0104, TransportationExit0104, TransportationMove0104,
    WirePayload, decode_npc_around_0104, decode_npc_skill_signal_0104, decode_pc_around_0104,
    decode_pc_regen_packet_0104, decode_shiny_around_0104, decode_transportation_around_0104,
    packet::{
        P_FE2CL_AROUND_DEL_NPC, P_FE2CL_AROUND_DEL_PC, P_FE2CL_AROUND_DEL_SHINY,
        P_FE2CL_AROUND_DEL_TRANSPORTATION, P_FE2CL_CHARACTER_ATTACK_CHARACTERS,
        P_FE2CL_NANO_SKILL_USE, P_FE2CL_NANO_SKILL_USE_SUCC, P_FE2CL_NPC_AROUND,
        P_FE2CL_NPC_ATTACK_CHARS, P_FE2CL_NPC_ATTACK_PCS, P_FE2CL_NPC_ENTER, P_FE2CL_NPC_EXIT,
        P_FE2CL_NPC_MOVE, P_FE2CL_NPC_NEW, P_FE2CL_NPC_SKILL_CANCEL,
        P_FE2CL_NPC_SKILL_CORRUPTION_HIT, P_FE2CL_NPC_SKILL_CORRUPTION_READY,
        P_FE2CL_NPC_SKILL_FIRE, P_FE2CL_NPC_SKILL_HIT, P_FE2CL_NPC_SKILL_READY, P_FE2CL_PC_AROUND,
        P_FE2CL_PC_ATTACK_CHARS, P_FE2CL_PC_ATTACK_CHARS_SUCC, P_FE2CL_PC_ATTACK_NPCS,
        P_FE2CL_PC_ATTACK_NPCS_SUCC, P_FE2CL_PC_EXIT, P_FE2CL_PC_JUMP, P_FE2CL_PC_MOVE,
        P_FE2CL_PC_NEW, P_FE2CL_PC_REGEN, P_FE2CL_PC_STOP, P_FE2CL_PC_SUDDEN_DEAD,
        P_FE2CL_REP_BARKER, P_FE2CL_SHINY_AROUND, P_FE2CL_SHINY_ENTER, P_FE2CL_SHINY_EXIT,
        P_FE2CL_SHINY_NEW, P_FE2CL_TRANSPORTATION_AROUND, P_FE2CL_TRANSPORTATION_ENTER,
        P_FE2CL_TRANSPORTATION_EXIT, P_FE2CL_TRANSPORTATION_MOVE, P_FE2CL_TRANSPORTATION_NEW,
    },
};

use crate::{
    coordinates::{ProtocolPosition, ProtocolYawDegrees, protocol_distance_to_native},
    movement::LegacyPlayerController,
    remote::{DecodedRemotePacket, RemoteAnimation, RemoteMotion, RemotePlayer},
    world_nano_authority::{
        WorldNanoAuthoritativeProjection0104, WorldNanoEntityKind0104,
        WorldNanoProjectionError0104, decode_world_nano_authority_0104,
    },
    world_npc_skill_authority::{
        WorldNpcSkillAuthoritativeProjection0104, WorldNpcSkillEntityKind0104,
        WorldNpcSkillProjectionError0104, decode_world_npc_skill_authority_0104,
    },
};

#[cfg(test)]
mod escort_facing_tests;

#[cfg(test)]
mod tests;

mod types;
mod projects;
mod commands;
mod systems;
mod animation;
mod codec;
mod operations;
mod entities;
mod input;
mod constants;

pub use types::{
    NetworkEntityLifecycle0104Plugin, NetworkNanoEffectEvents0104,
    NetworkNpcResultEffectEvents0104, NetworkNpcSkillEffect0104,
    NetworkNpcSkillEffectEvents0104, NetworkNpcBarkerEvent0104,
    ActiveNetworkEntitySession0104, NetworkEntityLifecycleIngressEvent0104,
    NetworkEntityLifecycleIngress0104, NetworkRemotePc0104, NetworkPcAppearance0104, RemotePcVisibility0104,
    PendingPcVisual0104, NetworkNpc0104, NetworkNpcAppearance0104, PendingNpcVisual0104,
    NetworkNpcMotion0104, NetworkNpcCombatAnimation0104, NetworkNpcCombatClip0104,
    NetworkTransportation0104, NetworkTransportationAppearance0104,
    NetworkTransportationMotion0104, NetworkShiny0104, NetworkShinyAppearance0104,
    RemotePcRegistry0104, NetworkNpcRegistry0104, NetworkTransportationRegistry0104,
    NetworkShinyRegistry0104, NetworkEntityLifecycleStats0104, MalformedLifecycleFrame0104,
    MalformedLifecycleFrames0104, PassthroughLifecycleFrame0104,
    PassthroughLifecycleFrames0104, LifecycleEntityKind0104, IgnoredLifecycleFrame0104,
    IgnoredLifecycleFrames0104, IgnoredLifecycleBootstrap0104,
    PassthroughLifecycleBootstrap0104, LifecycleBootstrapDiagnostics0104,
    DecodedEntityLifecyclePacket0104
};
pub(crate) use types::NetworkNpcReadyAnimation0104;
use types::{NetworkNpcSkillPhase0104, RemotePcAuthorityUpdate0104, NpcAuthorityUpdate0104};
pub use projects::{NetworkSessionEpoch0104, NetworkSessionEntity0104};
use projects::{begin_session, disconnect_session};
pub use commands::{NetworkNpcAttackEventQueue0104, NetworkNpcBarkerEventQueue0104};
pub use systems::NetworkHealingTickEffects0104;
use systems::{
    apply_player_motion, apply_player_regen, apply_player_sudden_dead, apply_npc_motion,
    apply_npc_attack_results, apply_mixed_attack_results, apply_pc_attack_results,
    apply_world_nano_authority, apply_world_npc_skill_authority, apply_remote_pc_authority,
    apply_npc_authority, apply_transportation_motion
};
pub use animation::NetworkNpcAnimationLayers0104;
use animation::{
    request_network_npc_combat_animation, apply_network_npc_hp_animation,
    apply_network_npc_skill_animation
};
pub use codec::{
    IgnoredLifecycleFrameReason0104, EntityLifecycleDecodeError0104,
    decode_entity_lifecycle_frame_0104
};
use codec::{consume_live_frame, push_ignored_frame};
pub use operations::consume_network_entity_lifecycle_0104;
use operations::{
    upsert_player, upsert_npc, upsert_transportation, upsert_shiny, face_network_npc_toward_npc,
    face_network_npc_toward_pc
};
use entities::{despawn_player, despawn_npc, despawn_transportation};
pub use entities::despawn_shiny;
use input::resolve_remote_player;
use constants::{PC_ATTACK_ENTITY_TYPE, NPC_ATTACK_ENTITY_TYPE};
