//! Render and motion runtime for server-owned protocol-0104 world entities.
//!
//! Static world dongs are owned by [`crate::world`]. This module closes the
//! other half of an ordinary-world frame: it resolves every live NPC type
//! through the exact XDT mesh row and the published semantic character
//! registry, instantiates the validated GLB, and advances authoritative
//! `NPC_MOVE` destinations. Missing catalog evidence remains visible as a
//! typed issue; placeholder geometry is never fabricated.

#[cfg(test)]
mod animation_tests;
#[cfg(test)]
use crate::assets::TABLE_SET_PATH;
pub mod shiny;
mod transportation;

use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
    sync::Arc,
    time::Duration,
};

use bevy::{
    animation::{AnimationTargetId, RepeatAnimation},
    asset::LoadState,
    camera::visibility::{NoFrustumCulling, RenderLayers, VisibilityRange},
    gltf::{Gltf, GltfAssetLabel},
    mesh::{MeshTag, skinning::SkinnedMesh},
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use ffone_runtime_contracts::{
    RETROBUTION_CHARACTER_ACTOR_EFFECT_IDS, RETROBUTION_FUSION_ACTOR_EFFECT_IDS,
    RETROBUTION_TUTORIAL_EFFECT_IDS,
};
use ffone_skinned_model::NativeSampler;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    assets::{AssetLocator, NPC_TEXTURE_CATALOG_PATH},
    attachment::{
        LegacyPlayerAttachmentSlot, player_attachment_socket_full_path,
        standard_player_attachment_placement,
    },
    character_creation_data::{CharacterCreationData, CharacterCreationDataResource},
    character_scene::{
        LegacyCharacterSceneDeferredReveal, LegacyCharacterSceneStatus, NativeSceneRole,
        native_scene_container_transform, spawn_legacy_character_scene,
    },
    coordinates::{LegacyCharacterRootPolicy, protocol_distance_to_native},
    entity_lifecycle::{
        NetworkNpc0104, NetworkNpcAnimationLayers0104, NetworkNpcAppearance0104,
        NetworkNpcCombatAnimation0104, NetworkNpcCombatClip0104, NetworkNpcMotion0104,
        NetworkNpcReadyAnimation0104, NetworkPcAppearance0104, NetworkRemotePc0104,
        NetworkTransportation0104, NetworkTransportationMotion0104, PendingNpcVisual0104,
        PendingPcVisual0104, consume_network_entity_lifecycle_0104,
    },
    gameplay_audio::{GameplayAudioRuntime, GameplayAudioSet},
    hnpc_runtime::HnpcRuntimeCatalog,
    legacy_material_animation::{
        LegacyMaterialAnimationClip, apply_legacy_material_float,
        parse_legacy_material_animation_clips, sample_legacy_float_curve,
    },
    legacy_model_material::{
        LegacyMaterialApplied, LegacyMaterialMetadataError, LegacyMaterialPassCompanion,
        LegacyModelMaterial, LegacyModelMaterialParams, LegacyModelTextures, LegacyNpcTextureRole,
        LegacyOutlineCompanion, LegacyOutlineMaterial, LegacyOutlineSource, LegacyPassKind,
        LegacyShaderKind, PendingLegacyModelMaterial, legacy_npc_texture_role,
        load_legacy_main_texture_replacement_with_sampler,
    },
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    movement::LegacyPlayerController,
    player_appearance_material::{NativePlayerMaterialBinding, bind_native_player_look_material},
    player_preview::{NativePlayerLook, NativePlayerPartKind},
    player_shared_rig::{
        NativePlayerRigAnimationApplied, NativePlayerRigAnimationIssue,
        NativePlayerRigAnimationRequest, NativePlayerRigAssetCache, NativePlayerRigBones,
        NativePlayerRigCatalog, NativePlayerRigSpawnRequest, NativePlayerRigStand1Playback,
        NativePlayerRigStatus, spawn_native_player_rig,
    },
    remote::{RemoteAnimation, RemoteAnimationState},
    tutorial_actors::TutorialActor,
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
        animation_event_crossings, process_tutorial_effect_runtime,
    },
    tutorial_player_presentation::{
        PlayerWeaponAnimationCatalog, PlayerWeaponAnimationProfile, TutorialPlayerClip,
    },
    world::spawn_pending_authored_model_collider,
};

use crate::scene_hierarchy::is_descendant_of;

#[cfg(test)]
mod tests;

mod assets_parse_character_registry;
mod textures;
mod collision;
mod constants;
mod state;
mod pc_visibility;
mod types;
mod animation_sync_network_hnpc_animation;
mod animation_play_network_npc_animation;
mod audio;
mod models;
mod operations_finalize_network_pc_visuals;
mod operations_animate_network_npc_appear_effect;
mod input;
mod materials;
mod entities;
mod commands;
mod systems;

use assets_parse_character_registry::{
    RegistryModel0104, parse_character_registry, load_network_npc_visual_catalog_0104
};
#[cfg(test)]
use assets_parse_character_registry::CHARACTER_REGISTRY_SCHEMA;
pub use assets_parse_character_registry::{
    NetworkNpcVisualCatalogIssue0104, NetworkNpcVisualCatalogRequest0104,
    NetworkNpcVisualCatalogState0104
};
use textures::{
    parse_npc_texture_catalog, optional_texture_name, resolve_npc_texture_override,
    NpcTextureRoots0104
};
#[cfg(test)]
use textures::{NPC_TEXTURE_CATALOG_SCHEMA, network_npc_texture_uses_fusion_matter_0104};
pub use textures::{
    NetworkNpcTextureOverride0104, NetworkNpcTextureVariantBound0104,
    NpcSceneTextureOverrides0104, bind_network_npc_texture_variants_0104
};
#[cfg(test)]
pub(crate) use textures::NetworkNpcTextureSlot0104;
use collision::{NativeCharacterCollisionContract0104, materialize_network_npc_collision_0104};
#[cfg(test)]
use collision::CHARACTER_COLLISION_SCHEMA_V2;
use constants::{
    CONSOLIDATED_TABLE, ARRIVAL_EPSILON, NETWORK_NPC_LOW_LAYER_CLIPS_0104,
    NETWORK_NPC_UNCOVERED_HIGH_LAYER_GROUP_0104
};
pub(crate) use state::register_network_world_runtime_0104;
pub use types::{
    NetworkNpcVisualDefinition0104, NetworkHnpcVisualDefinition0104,
    NetworkNpcVisualCatalog0104, NetworkPcVisualDataState0104, NetworkPcVisual0104,
    NetworkPcVisualIssue0104, NetworkHnpcVisual0104, NativeHnpcAppearancePlugin,
    NetworkNpcVisual0104, NetworkNpcAppearEffect0104, NetworkNpcAppearOutlineBound0104,
    NetworkNpcGrounding0104, NetworkNpcVisualIssue0104, NetworkNpcMotionSettled0104
};
use types::{
    NativeCharacterCollider0104, NetworkPcVisualGeneration0104,
    NetworkHnpcVisualGeneration0104, NetworkPcRig0104, NetworkPcAppearancePart0104,
    PendingNetworkPcAppearanceAttachment0104, PendingNetworkPcWeaponAttachment0104,
    NetworkPcWeaponAttachment0104, NetworkHnpcRig0104, NetworkHnpcPart0104,
    PendingNetworkHnpcAttachment0104, PendingNetworkNpcCollision0104,
    NetworkNpcHighAnimations0104, NetworkNpcAdditivePair0104
};
use animation_sync_network_hnpc_animation::{
    NetworkNpcAnimationEffectEvent0104, parse_network_npc_animation_document,
    parse_network_npc_animation_effect_events, parse_network_npc_animation_end_events,
    NetworkPcAnimationRevision0104, NetworkHnpcAnimationRevision0104,
    sync_network_pc_animation_0104, block_network_pc_rig, NetworkHnpcAnimationState0104,
    sync_network_hnpc_animation_0104, block_network_hnpc_rig, network_npc_clip_is_additive_0104,
    NetworkNpcAnimationApplied0104, NetworkNpcPreparedAnimationGraph0104,
    NetworkNpcAnimationAssets0104
};
#[cfg(test)]
use animation_sync_network_hnpc_animation::{
    network_pc_animation_clip, hnpc_idle_clip, network_hnpc_animation_clip
};
pub(crate) use animation_sync_network_hnpc_animation::{
    NetworkPcRigAppearanceStatus0104, network_npc_animation_uses_delta_additive_layer_0104
};
pub use animation_sync_network_hnpc_animation::NetworkHnpcRigAppearanceStatus0104;
use animation_play_network_npc_animation::{
    play_network_npc_animation_0104, network_npc_melee_clip_0104,
    emit_network_npc_animation_effects_0104, emit_network_hnpc_animation_sounds_0104,
    emit_network_npc_animation_sounds_0104, network_pc_vehicle_animation_clip
};
#[cfg(test)]
use animation_play_network_npc_animation::{
    network_npc_stand_clip_0104, prepare_network_npc_animation_graph_0104
};
pub use audio::{NetworkNpcAnimationSoundEvent0104, parse_network_npc_animation_sound_events};
use audio::{
    NetworkNpcAnimationSoundCursor0104, NetworkNpcAnimationSoundCursors0104,
    NetworkHnpcAnimationSoundCursor0104, network_npc_sound_ancestor
};
use models::select_registry_model;
use operations_finalize_network_pc_visuals::{
    positive_speed_or_one, network_pc_appearance_attachment_slot,
    bind_network_pc_appearance_attachments_0104, bind_network_pc_weapon_attachment_0104,
    bind_network_pc_appearance_0104, finalize_network_pc_visuals_0104, finish_network_pc_emote_0104,
    bind_network_hnpc_attachments_0104, bind_network_hnpc_materials_0104,
    finalize_network_hnpc_visuals_0104, hnpc_idle_clips, network_npc_fusion_matter_params_0104
};
#[cfg(test)]
use operations_finalize_network_pc_visuals::{
    network_pc_expected_appearance_routes, network_pc_appearance_routes_ready
};
use operations_animate_network_npc_appear_effect::{
    animate_network_npc_appear_effect_0104, network_npc_visual_ancestor,
    network_npc_cross_fade_0104, cross_fade_network_npc_low_layer_0104,
    network_npc_one_shot_completed_0104, network_npc_idle_end_crossed_0104,
    network_npc_idle_cursor_0104, store_network_npc_idle_cursor_0104,
    restart_network_npc_additive_pair_0104, stop_finished_network_npc_additive_pairs_0104,
    network_npc_uncovered_high_layer_targets_0104, surface_has_named_ancestor,
    network_npc_playbacks_0104, network_npc_effect_ancestor, network_npc_ancestor, required_string,
    required_i64, optional_i64, required_i32, required_f32
};
#[cfg(test)]
use operations_animate_network_npc_appear_effect::network_npc_appear_alpha_0104;
use input::{
    load_network_pc_visual_data_0104, resolve_network_pc_visuals_0104,
    resolve_network_hnpc_visuals_0104, resolve_network_npc_visuals_0104
};
use materials::{
    NetworkPcAppearanceMaterialBound0104, NetworkHnpcMaterialBound0104,
    apply_network_npc_material_animation_0104
};
pub use materials::{
    NetworkNpcAppearMaterialBound0104, NetworkNpcVisualMaterialReady0104,
    NetworkNpcMaterialVisibilityBlocked0104, NetworkNpcMaterialSurface0104,
    finalize_network_npc_material_visibility_0104
};
pub use entities::{spawn_network_hnpc_visual_0104, spawn_network_npc_visual_0104};
use commands::{
    NetworkHnpcIdleEventCursor0104, NETWORK_NPC_IDLE_REQUEST_0104,
    NetworkNpcIdleEventCursor0104
};
pub use systems::{advance_network_npc_motion_0104, advance_network_transportation_motion_0104};
use systems::{apply_network_npc_high_layers_0104, sync_network_pc_vehicle_weapon_visibility};
#[cfg(test)]
use systems::advance_server_entity_motion;
