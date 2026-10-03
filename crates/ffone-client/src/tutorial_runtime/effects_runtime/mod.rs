//! Exact-data native runtime bridge for the original tutorial's Unity particle
//! prefabs and Oni projectiles.
//!
//! Publication and playback are intentionally separate.  The asset pipeline
//! can prove and publish the exact serialized dependency closures today, but
//! Bevy does not natively implement Unity 4's `ParticleEmitter`,
//! `ParticleAnimator`, `ParticleRenderer`, `EffectEmitterController`, or the
//! legacy effect shaders.  The focused native module below implements the
//! deterministic, audited subset and reports a node-specific blocker for every
//! closure feature that cannot be reproduced without inventing Unity state.

mod tutorial_native_effects;
mod native_skill_projectiles;
pub use tutorial_native_effects::TutorialEffectMaterialAnimation;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::{animation::RepeatAnimation, prelude::*};
use ffone_runtime_contracts::{
    RETROBUTION_CHARACTER_ACTOR_EFFECT_IDS, RETROBUTION_FUSION_ACTOR_EFFECT_IDS,
    RETROBUTION_NPC_GAME_ICON_EFFECT_IDS, RETROBUTION_NPC_WARP_EFFECT_IDS,
    RETROBUTION_PLAYER_STATUS_EFFECT_IDS, RETROBUTION_TUTORIAL_BUILD_ID,
    RETROBUTION_TUTORIAL_BULLET_TYPES, RETROBUTION_TUTORIAL_EFFECT_IDS,
    RETROBUTION_TUTORIAL_PROJECTILE_EFFECT_IDS, RETROBUTION_TUTORIAL_SOURCE_ASSET_PROOFS,
    RETROBUTION_WEAPON_EFFECT_IDS, RETROBUTION_WORLD_EP_EFFECT_IDS, TUTORIAL_BULLET_ROW_SCHEMA,
    TUTORIAL_EFFECT_CATALOG_PATH, TUTORIAL_EFFECT_CATALOG_SCHEMA, TUTORIAL_EFFECT_CLOSURE_SCHEMA,
    TUTORIAL_PROJECTILE_CATALOG_PATH, TUTORIAL_PROJECTILE_CATALOG_SCHEMA, TutorialBulletParameters,
    TutorialBulletRowFile, TutorialEffectCatalog, TutorialEffectCatalogEntry,
    TutorialEffectClosureFile, TutorialProjectileCatalog, TutorialSourceAssetProof,
    TutorialSourceFileProof,
};
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::tutorial_mission_content::TutorialMissionContent;
use crate::{
    network_world_runtime::NetworkNpcVisual0104,
    tutorial_actors::{
        TutorialActor, TutorialActorAnimationPlayback, TutorialActorEvent, TutorialActorEventQueue,
        TutorialActorPlayerKillDeathPresentation, apply_tutorial_actor_animation_playback,
    },
};

#[cfg(test)]
mod tests;

mod state_tutorial_effect_runtime_issue;
mod state_tutorial_effect_runtime_process_one;
mod assets;
mod constants;
mod codec;
mod validation;
mod types;
mod commands;
mod animation_emit_retrobution_actor_animation;
mod operations;
mod audio;
mod containers;
mod input;

use state_tutorial_effect_runtime_issue::{EFFECT_RENDERER_STATUS, PROJECTILE_RENDERER_STATUS};
pub use state_tutorial_effect_runtime_issue::{
    TutorialEffectRuntimeCommand, TutorialEffectRuntimeDisposition,
    TutorialEffectRuntimeRecord, TutorialEffectRuntimeIssue, TutorialEffectRuntime
};
pub use state_tutorial_effect_runtime_process_one::{
    TutorialEffectsRuntimePlugin, TutorialEffectRuntimeSet, process_tutorial_effect_runtime,
    cleanup_tutorial_effect_runtime, TutorialOniProjectileState
};
use assets::{PRIMARY_EFFECTS_ASSET, safe_asset_path};
use constants::{
    EFFECTS_DEPENDENCY_B4, EFFECTS_DEPENDENCY_BD5, RETROBUTION_ACTOR_EFFECT_EVENTS,
    RETROBUTION_ACTOR_DEATH_PRESENTATION_EVENTS
};
pub use constants::RETROBUTION_ONI_MOTION;
use codec::STREAMED_WORLD_EFFECT_COMMANDS_PER_FRAME;
pub use validation::TutorialEffectLibraryError;
use validation::{
    validate_catalog_headers, validate_effect_closure, validate_object_proofs,
    validate_exact_bullet_parameters, reject_symlink
};
pub use types::{
    TutorialEffectLibrary, TutorialEffectPlacement, TutorialProjectileMotion,
    WarheadAuthority, TutorialNativeClosureBlockerReason, TutorialProjectileVisualReadiness,
    TutorialOniMotionContract, TutorialOniMotionPhase
};
use types::{ValidatedEffect, TutorialEffectAttachmentProof, ActiveNativeInstance};
use commands::{RetrobutionActorEffectEvent, RetrobutionActorDeathPresentationEvent};
use animation_emit_retrobution_actor_animation::emit_retrobution_actor_animation_events;
pub use animation_emit_retrobution_actor_animation::animation_event_crossings;
use operations::{
    retrobution_actor_current_effect_name, exact_projectile_inputs,
    exact_projectile_success_scale, exact_projectile_success_script,
    exact_invisible_projectile_carrier, valid_oni_velocity_sample, canonical_json,
    blake3_hash, fail
};
use audio::exact_tutorial_success_sound_path;
pub use containers::tutorial_oni_velocity_samples_from_unity_unit_draws;
use input::{read_json, read_verified};
