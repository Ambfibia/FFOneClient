//! Selected-player shared-rig runtime for the tutorial world.
//!
//! The renderer consumes only exact clip contracts and manifest-verified
//! character routes. It never substitutes an NPC model when a player asset is
//! missing, and it records applied animation only after mutating Bevy's real
//! `AnimationPlayer`.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::Duration,
};

use bevy::{
    animation::{AnimatedBy, AnimationTargetId, RepeatAnimation},
    asset::{LoadState, RecursiveDependencyLoadState},
    camera::visibility::RenderLayers,
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
};
use ffone_protocol::CharacterEquipSlot0104;
use ffone_runtime_contracts::PlayerRigGender;

use crate::{
    attachment::{
        LegacyPlayerAttachmentSlot, player_attachment_socket_full_path,
        standard_player_attachment_placement,
    },
    avatar_action::{
        LegacyAnimationLayer, LegacyAvatarActionSet, LegacyAvatarActionState,
        LegacyAvatarPresentationContext, LegacyAvatarTraversalPresentation, LegacyLocomotionState,
        LegacyVisualClip, LegacyVisualCommand, LegacyVisualCompletion, LegacyVisualCompletionQueue,
        LegacyVisualRequestQueue,
    },
    character_creation_data::{CharacterCreationData, CharacterCreationDataResource},
    character_scene::{NativeSceneRole, native_scene_container_transform},
    legacy_environment::LegacyEnvironmentSet,
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyMaterialPassCompanion, LegacyModelMaterial,
        PendingLegacyModelMaterial,
    },
    movement::{LegacyMovementSet, LegacyPlayerController, LegacyWorldColliderPending},
    network::CharacterSummary,
    player_appearance_material::{
        ActorSkinTextureRole, NativePlayerMaterialBinding, bind_native_player_look_material,
        uses_global_skin_secondary,
    },
    player_emote::{PlayerEmoteAdvance, PlayerEmoteContinuation, PlayerEmoteCursor, emote_events},
    player_preview::{NativePlayerLook, NativePlayerPartKind, NativePlayerPartLook},
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigBones, NativePlayerRigCatalog,
        NativePlayerRigSpawnRequest, NativePlayerRigStand1Playback, NativePlayerRigStatus,
        spawn_native_player_rig,
    },
    tutorial_player_presentation::{
        ContractResolvedTutorialPlayerAnimation, PlayerWeaponAnimationCatalog,
        PlayerWeaponAnimationProfile, TutorialPlayerAnimationDispatch,
        TutorialPlayerAnimationRequest, TutorialPlayerClip, TutorialPlayerClipPlayback,
        TutorialPlayerPresentationCommandQueue, TutorialPlayerPresentationConsumer,
        TutorialPlayerPresentationResolution, TutorialPlayerRigCapabilities,
    },
};
pub(crate) mod personal_vehicle;
pub use personal_vehicle::PersonalVehiclePresentation;

use crate::scene_hierarchy::is_descendant_of;

#[cfg(test)]
mod tests;

mod constants;
mod animation_prepare_tutorial_player_animation_adapter;
mod animation_finalize_tutorial_player_rig_readiness;
mod animation_advance_tutorial_player_animation_state;
mod systems;
mod models;
mod types;
mod materials;
mod operations_bridge_legacy_avatar_visual_requests;
mod operations_consume_tutorial_player_presentation;
mod assets;
mod entities;
mod input;
mod commands;
mod damage;

use constants::{TUTORIAL_WEAPON_IDS, UPPER_BODY_MASK, UNARMED_ATTACK_LOWER_BODY_MASK};
use animation_prepare_tutorial_player_animation_adapter::{
    TutorialPlayerAnimationAdapter, TutorialAnimationDelayPlayback,
    switch_tutorial_composed_pose_mask, tutorial_player_clip_end_event_seconds
};
pub use animation_prepare_tutorial_player_animation_adapter::{
    TutorialSelectedPlayerRig, TutorialSelectedPlayerRigActive,
    TutorialSelectedPlayerRigCandidate, TutorialSelectedPlayerRigStatus,
    TutorialPlayerAnimationApplied, TutorialPlayerRigIssue, TutorialPlayerRigIssueQueue,
    SpawnedTutorialSelectedPlayerRig, TutorialPlayerRigRuntimePlugin
};
use animation_finalize_tutorial_player_rig_readiness::{
    finalize_tutorial_player_rig_readiness, legacy_visual_tutorial_clip,
    CoalescedLegacyLayerAnimation, coalesce_legacy_layer_animation,
    queue_coalesced_legacy_layer_animation
};
use animation_advance_tutorial_player_animation_state::{
    advance_tutorial_player_animation_state, tutorial_upper_pose_source, add_tutorial_upper_clip,
    apply_resolved_tutorial_animation, release_paused_tutorial_main_animation
};
#[cfg(test)]
use animation_advance_tutorial_player_animation_state::{
    tutorial_animation_transition_duration, restart_missing_tutorial_base_animation
};
use systems::{
    sync_tutorial_skyway_attachment, sync_tutorial_zipline_attachment,
    update_tutorial_directional_turn, advance_tutorial_upper_layer_playback,
    sync_tutorial_player_weapon_visibility
};
use models::{SKYWAY_MODEL_PATH, ZIPLINE_MODEL_PATH};
pub use types::{
    TutorialSkywayPresentation, TutorialPlayerAppearanceReady, TutorialPlayerFallbackVisual,
    TutorialPlayerFallbackReplaced, TutorialPlayerWeaponAttachment,
    TutorialPlayerEquipmentAttachmentSpawned
};
use types::{
    TutorialSkywayAttachment, TutorialZiplineAttachment, PendingLegacyVisualCompletion,
    TutorialUpperLayerPlayback, TutorialBodyShapePlayback, TutorialDirectionalTurnPlayback,
    TutorialPlayerAppearancePart, TutorialPlayerWeaponLook,
    PendingTutorialPlayerAppearanceAttachment
};
pub use materials::TutorialPlayerAppearanceMaterialBound;
use materials::TutorialPlayerWeaponMaterialBound;
use operations_bridge_legacy_avatar_visual_requests::{
    tutorial_player_appearance_attachment_slot, skyway_attachment_transform,
    owning_skyway_attachment, legacy_body_shape_normalized_times, switch_tutorial_base_layer_mask,
    switch_tutorial_body_shape_mask, legacy_directional_turn_target, move_towards,
    normalized_upper_layer_weight, tutorial_upper_layer_alpha,
    keep_tutorial_gameplay_playback_alive, tutorial_clamp_finished_or_removed,
    bind_tutorial_player_appearance_attachments, bind_tutorial_player_weapon_materials,
    bind_tutorial_player_appearance, swap_tutorial_player_fallback_on_ready,
    block_tutorial_player_appearance, ancestor_tutorial_player_appearance_part,
    pending_legacy_visual_completion, bridge_legacy_avatar_visual_requests,
    tutorial_unarmed_attack_masks_target, tutorial_emote_holds_locomotion,
    tutorial_startup_emote_loading_barrier, tutorial_emote_interrupt_requested
};
#[cfg(test)]
use operations_bridge_legacy_avatar_visual_requests::attack_interrupts_player_emote;
use operations_consume_tutorial_player_presentation::{
    consume_tutorial_player_presentation, resume_tutorial_delayed_playback
};
use assets::skyway_socket_full_path;
pub use assets::tutorial_player_weapon_socket_full_path;
pub use entities::spawn_tutorial_selected_player_rig;
use input::resolve_required_tutorial_player_clips;
use commands::upper_event_releases_immediately;
