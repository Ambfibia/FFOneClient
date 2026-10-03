//! Sequence-owned tutorial/cutscene actors.
//!
//! Runtime values come from [`TutorialMissionContent`], while model, texture,
//! scale, and animation-asset routing come from the same production
//! `NetworkNpcVisualCatalog0104` used by ordinary world NPCs. Tutorial
//! choreography owns only actor lifetime and behavior; it has no parallel
//! visual catalog.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::{
    animation::RepeatAnimation,
    asset::{AssetId, LoadState, RecursiveDependencyLoadState},
    gltf::Gltf,
    prelude::*,
};

use crate::{
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarTargetFeed, LegacyTargetKind, LegacyTargetSample,
    },
    coordinates::{LegacyUnityHeadingDegrees, ProtocolPosition, ProtocolYawDegrees},
    legacy_npc_nano_animation::{
        LegacyAnimationBlend, LegacyAnimationRepeat, LegacyNpcAnimationRole,
        npc_role_for_forced_clip,
    },
    movement::{LegacyOrbitCamera, advance_native_xorshift32},
    native_terrain::NativeHeightmapCollider,
    network_world_runtime::{
        NetworkNpcAppearEffect0104, NetworkNpcVisual0104, NetworkNpcVisualCatalogState0104,
        network_npc_animation_uses_delta_additive_layer_0104, spawn_network_npc_visual_0104,
    },
    tutorial_auxiliary_choreography::{
        TUTORIAL_AUXILIARY_DEFINITIONS, TUTORIAL_INITIALIZATION, TutorialAuxiliaryAction,
    },
    tutorial_choreography::{ChoreographyAction, NpcAction, TUTORIAL_SCENE_CHOREOGRAPHIES},
    tutorial_logic::{
        BUTTERCUP_ID, COLLAPSE_NUMBUH_TWO_ID, DEMO_MONSTER_ID, FUSION_BUTTERCUP_ID,
        FUSION_PORTAL_ID, LAIR_DEXTER_ID, LAIR_EXIT_ID, NUMBUH_TWO_ID, ObservedNpc,
        TECH_SQUARE_ATTENDANT_ID, TutorialNpcObservation, TutorialNpcSpawn,
    },
    tutorial_mission_content::{GameplayNpcUiDefinition, TutorialMissionContent},
    world::{
        AUTHORED_CHARACTER_CONTROLLER_HEIGHT, AuthoredTriMeshCollider, NativeTerrainSpatialLookup,
        NativeTerrainSpatialRegistry, NativeWorldSet, authored_collider_blocks_segment,
        collider_ground_height, NativeWorldSceneRoot, NativeWorldPresentationStatus,
        legacy_dong_squared_distance_native,
    },
};

#[cfg(test)]
mod tests;

mod constants;
mod types;
mod collision;
mod systems;
mod animation_apply_tutorial_actor_animation_p;
mod assets;
mod commands;
mod operations_damage_actor;
mod operations_tutorial_target_sample_with_values;
mod entities;
mod materials;

pub use constants::MISSION_TARGET_ID;
use constants::TUTORIAL_ACTOR_STAND_CLIPS;
#[cfg(test)]
use constants::TUTORIAL_ACTOR_CROSS_FADE_SECONDS;
pub use types::{
    TutorialActor, TutorialActorScene, TutorialActorVisualUnavailable0104,
    TutorialActorCombatTransition, TutorialActorStandRandomStream, TutorialActorMotion,
    TutorialActorIssue, TutorialActorIssueQueue, TutorialTargetingProfile,
    TutorialTargetingFacing, TutorialActorObservationSample, TutorialNpcObservationSnapshot,
    TutorialActorCombatConfig, TutorialActorSet, TutorialActorPlugin
};
pub(crate) use types::TutorialActorPlayerKillDeathPresentation;
use types::TutorialActorCombatCompletion;
pub use collision::TutorialActorGrounding;
use collision::advance_tutorial_actor_grounding;
pub use systems::{
    TutorialActorForceUpdate, apply_tutorial_actor_commands, advance_tutorial_actor_motion,
    refresh_tutorial_npc_observation
};
pub use animation_apply_tutorial_actor_animation_p::{
    TutorialActorAnimationSource, TutorialActorAnimationPlayback,
    TutorialActorAnimationIssueHistory, TutorialActorAnimationAssets, TutorialActorPoseState,
    TutorialActorPose, advance_tutorial_actor_combat_animation,
    apply_tutorial_actor_animation_playback
};
use animation_apply_tutorial_actor_animation_p::{
    tutorial_actor_move_clip, play_actor_pose, play_actor_pose_with_contract,
    play_actor_combat_pose, set_actor_pose_state, tutorial_actor_attack_clip
};
#[cfg(test)]
use animation_apply_tutorial_actor_animation_p::{
    tutorial_actor_pose_effective_once, TutorialActorAnimationControl,
    tutorial_actor_animation_control, prepare_tutorial_actor_animation_graph,
    ensure_tutorial_actor_additive_base_animation, legacy_npc_clip_is_additive,
    report_tutorial_actor_animation_unavailable, restart_tutorial_actor_animation,
    restart_tutorial_actor_additive_animation
};
#[cfg(test)]
use animation_apply_tutorial_actor_animation_p::{
    exact_tutorial_actor_named_animation, exact_tutorial_actor_pose_animation
};
pub use assets::TutorialActorRegistry;
pub use commands::{
    TutorialActorCommand, TutorialActorCommandQueue, TutorialActorEvent,
    TutorialActorEventQueue
};
#[cfg(test)]
pub use operations_damage_actor::{tutorial_spawn_transform, tutorial_target_sample};
use operations_damage_actor::{
    tutorial_spawn_transform_for_class, start_unforced_actor_stand_poses, actor_entity,
    warp_actor_native, translate_actor_native, move_actor_native, stop_actor_motion,
    rotate_actor_native, face_actor_native_position, face_actor, tutorial_actor_idle_stand_for_roll,
    delete_actor, damage_actor, clear_actor_interactions, configure_demo_monster,
    attempt_player_attack, set_actor_interacting
};
pub use operations_damage_actor::{
    tutorial_actor_root_name, tutorial_actor_npc_types, ground_tutorial_actors,
    tutorial_virtual_server_damage, cleanup_tutorial_actors,
    produce_tutorial_avatar_target_feed
};
#[cfg(test)]
use operations_damage_actor::tutorial_target_sample_with_los;
use operations_tutorial_target_sample_with_values::tutorial_target_sample_with_values;
pub use operations_tutorial_target_sample_with_values::build_tutorial_npc_observation;
use entities::spawn_actor;
pub use entities::spawn_tutorial_actor_visuals;
use materials::tutorial_actor_native_combat_blend;
