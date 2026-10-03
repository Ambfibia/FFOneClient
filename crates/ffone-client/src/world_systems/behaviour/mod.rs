//! Native world behaviour: the non-geometry half of every streamed world tile.
//!
//! The static-world installer publishes what a tile looks like. This module
//! consumes the sibling `ffone.native-world-behaviours.v1` document, which
//! carries the scripted Infected-Zone elements, effect emitters, billboards,
//! visibility switches, animation players, trigger volumes and rigid bodies of
//! the same nodes.
//!
//! Every record already carries its exact native world matrix, so behaviour is
//! placed without any dependency on conversion metadata. Records that name
//! published models are additionally linked to the spawned visual entities of
//! their own tile. This lets a legacy `BillboardNode` rotate the exact authored
//! renderer subtree while `VisibleSwitch` controls that billboard behaviour.

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    path::Path,
    sync::{Arc, OnceLock},
};

use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures_lite::future},
};
use ffone_protocol::{
    PcJumppadRequest0104, PcLauncherRequest0104, PcMovePlatformRequest0104, PcSlopeRequest0104,
    PcZiplineRequest0104, RegisteredGameplayRequest0104, WirePayload, packet,
};
use ffone_runtime_contracts::{
    TUTORIAL_EFFECT_CLOSURE_SCHEMA, TutorialEffectClosureFile, TutorialUnityObjectProof,
};
use serde::Deserialize;

use crate::tutorial_effects_runtime::{
    TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
};
use crate::{
    avatar_action::{LegacyAvatarTargetFeed, LegacyTriggerSample},
    coordinates::{
        LegacyUnityHeadingDegrees, ProtocolMoveVelocity, ProtocolPosition, ProtocolScaledVelocity,
        unity_to_native_vector,
    },
    gameplay_audio::GameplayAudioRuntime,
    launcher_ui::{
        LauncherTriggerSpec, LauncherUiEffect, LauncherUiExternalState, LauncherUiModel,
        LauncherUiOutbox, LauncherUiSet, launcher_forward,
    },
    movement::{LegacyInputState, LegacyOrbitCamera},
    player_shared_rig::NativePlayerRigBones,
    tutorial_player_rig_runtime::TutorialSelectedPlayerRig,
    world::{
        NativeWorldBehaviourStatus, NativeWorldScope, PendingNativeWorldSceneSpawn,
        PendingNativeWorldSceneUnload, SpawnedNativeWorldCollider, SpawnedNativeWorldVisual,
    },
};

#[cfg(test)]
use crate::world::RuntimeManagedNativeWorldVisual;

mod streaming;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "material_sharing_tests.rs"]
mod material_sharing_tests;

mod constants;
mod containers;
mod animation_blackhole_nif_animation_player;
mod animation_update_world_animation_materials;
mod codec;
mod materials;
mod state;
mod types;
mod operations_process_world_trigger_uses;
mod operations_consume_world_launcher_outbox;
mod commands;
mod validation;
mod input;
mod assets;
mod models;
mod systems_update_world_trigger_volumes;
mod systems_apply_pending_world_behaviours;
mod textures;
mod localization;
mod output;

pub use constants::NATIVE_WORLD_BEHAVIOUR_SCHEMA;
use containers::{
    WORLD_EFFECT_PREFAB_CLOSURE_SCHEMA, NativeWorldObjectSourceRoutes,
    load_native_world_object_source_routes, parse_native_world_object_source_routes,
    unity_vec3, has_reproducible_world_particle_prefab, native_rotation_to_unity_euler_degrees
};
pub use containers::WorldEffectPrefabClosure;
use animation_blackhole_nif_animation_player::{
    PendingWorldAnimationRecord, WorldAnimationCompiledTracks, WorldAnimationTargetSample,
    WorldAnimationVisibilityBindings, blackhole_nif_animation_player,
    validate_world_effect_nif_animation_bindings, compile_world_animation_tracks,
    world_animation_dynamic_model_entities, world_animation_clock_is_paused,
    world_animation_transform_repeats, world_animation_sample_time,
    sample_world_animation_vector_fixed, WorldGpuUvAnimation
};
#[cfg(test)]
use animation_blackhole_nif_animation_player::{
    WorldAnimationCompiledTrack, blackhole_nif_animation_contract
};
pub use animation_blackhole_nif_animation_player::{
    WorldAnimationPlayer, WorldAnimationTargetBinding, WorldAnimationTarget, AnimationRecord,
    WorldAnimationTargetRecord, WorldAnimationTrs, WorldAnimationClip, WorldAnimationChannel,
    WorldAnimationFloatCurve
};
pub(crate) use animation_blackhole_nif_animation_player::{
    WorldAnimationSamples, apply_world_animation_samples
};
use animation_update_world_animation_materials::{
    compile_world_gpu_uv_animation, update_world_animation_materials,
    take_world_animation_clips
};
use codec::{
    WORLD_SCRIPTED_EFFECTS_PER_FRAME, WORLD_BEHAVIOUR_WORK_PER_FRAME,
    WORLD_BEHAVIOUR_WORK_PER_ROOT_PER_FRAME, WORLD_ANIMATION_BINDINGS_PER_FRAME,
    WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_FRAME
};
pub use codec::WorldSurfacePacketClock;
use materials::{
    WorldAnimationMaterialOwner, compile_affine_world_material_curve,
    prepare_world_animation_material_bindings, sample_world_player_material_curve,
    world_material_curve_is_gpu_driven, apply_world_material_float_curve
};
#[cfg(test)]
use materials::{
    WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_PLAYER, WorldAnimationMaterialBinding,
    sample_world_clip_material_curve, sample_world_material_curve
};
pub(crate) use materials::WorldAnimationMaterialBindings;
pub use state::{BillboardMode, WorldSwitchState};
pub use types::{
    WorldBillboard, WorldVisibilitySwitch, WorldEffectEmitter, WorldFloatingIconSpin,
    WorldTriggerKind, WorldTrigger,
    WorldPlatformMotion, WorldTriggerVolumeShape, WorldTriggerVolume, WorldRigidBody,
    WorldWaypoint, WorldTriggerUseQueue, WorldZiplineTraversal, WorldRopeTraversal,
    WorldSlopeTraversal, WorldJumppadArmed, ActiveWorldLauncher, WorldBehavioursApplied,
    NativeWorldBehaviourDocument, BillboardRecord, VisibilitySwitchRecord,
    EffectEmitterRecord, TriggerRecord, WaypointRecord, TriggerVolumeRecord, RigidBodyRecord,
    PendingWorldScriptedEffects, PendingWorldBehaviourSpawn, WorldTriggerVolumeOccupied,
    NativeWorldBehaviourRoot, WorldBehaviourPlugin
};
use types::{
    NativeWorldObjectsDocument, WorldBehaviourSpawnPhase, WorldGpuUvCurve,
    LoadedNativeWorldBehaviour
};
use operations_process_world_trigger_uses::{
    clean_initial_trigger_server_id, default_capsule_direction, canonical_map_tile_id,
    mat4_from_world_matrix, vec3, placement, json_vector3, json_quaternion,
    enqueue_world_ep_effects, world_scripted_effect_count, organized_world_source_node,
    charge_world_behaviour_work, visibility_for, world_channel_sample_time,
    world_uv_curve_has_seamless_loop, sample_cubic_scalar, legacy_platform_eased_progress,
    legacy_platform_source_position, trigger_volume_contains_local_point, distance_to_segment,
    sample_polyline, nearest_polyline_distance, lerp_degrees
};
#[cfg(test)]
use operations_process_world_trigger_uses::{
    world_scripted_effect_closure, world_scripted_effect_placement,
    reproducible_world_scripted_effect, polyline_length
};
pub use operations_process_world_trigger_uses::{
    transform_from_world_matrix, enqueue_pending_world_scripted_effects,
    activate_world_jumppads, process_world_trigger_uses
};
#[cfg(test)]
use operations_process_world_trigger_uses::sample_cubic_vector;
pub use operations_consume_world_launcher_outbox::{
    finish_unsupported_world_slopes, consume_world_launcher_outbox,
    retire_unloading_world_behaviour_documents, materialize_pending_world_behaviours,
    add_world_preview_behaviour_systems
};
use operations_consume_world_launcher_outbox::retire_world_behaviour_document;
pub use commands::WorldGameplayIntentQueue;
use commands::{world_ep_effect_command, make_jumppad_request};
pub use validation::WorldBehaviourError;
pub use input::{load_native_world_behaviours, PendingWorldBehaviourLoad};
use input::{parse_native_world_behaviours, retire_world_behaviour_load_task};
pub use assets::behaviour_document_path;
use assets::{NativeWorldObjectSourceRoute, read_hashed_world_behaviour_asset};
use models::{legacy_static_model_id, reparent_authored_world_model, entity_or_ancestor_is_model};
#[cfg(test)]
use models::authored_model_local_transform;
use systems_update_world_trigger_volumes::update_world_animations;
pub use systems_update_world_trigger_volumes::{
    update_world_billboards, update_world_visibility_switches, update_world_platforms,
    update_world_floating_icons,
    update_world_trigger_volumes, update_world_zipline_traversals,
    update_world_rope_traversals, update_world_slope_traversals, update_world_belts,
    sync_world_launcher_input, apply_world_launcher_camera
};
pub use systems_apply_pending_world_behaviours::apply_pending_world_behaviours;
use textures::native_world_texture_offset_y;
use localization::legacy_platform_world_translation;
pub use output::publish_world_platform_packets;
