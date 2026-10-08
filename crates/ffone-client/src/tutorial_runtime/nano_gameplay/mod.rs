//! Asset-backed gameplay Nano used by the Retrobution tutorial.
//!
//! The event-scene Nano in `tutorial_nano_presentation` intentionally remains
//! a separate singleton. This module owns the real active Nano: an exact GLB
//! instance which follows the local player and is the authority for skill
//! stamina and tutorial skill events.

use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    sync::Arc,
};

#[cfg(test)]
use std::time::Duration;

use bevy::{
    animation::RepeatAnimation,
    asset::{LoadState, RecursiveDependencyLoadState},
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
};

use crate::{
    assets::AssetLocator,
    avatar_action::{LegacyAttackTarget, LegacyTargetKind},
    character_scene::{NativeSceneRole, native_scene_container_transform},
    gameplay_audio::{GameplayAudioRuntime, GameplayAudioSet},
    legacy_model_material::{
        LegacyModelMaterial, PendingLegacyModelMaterial, load_legacy_main_texture_replacement,
    },
    legacy_npc_nano_animation::{
        LegacyAnimationBlend, LegacyNanoAnimationMachine, LegacyNanoAnimationMode,
        LegacyNanoCompletion, LegacyNanoStandRandomStream,
    },
    network_world_runtime::{
        NetworkNpcAnimationSoundEvent0104, parse_network_npc_animation_sound_events,
    },
    tutorial_actors::TutorialActor,
    tutorial_effects_runtime::animation_event_crossings,
    tutorial_logic::DEMO_MONSTER_ID,
    tutorial_nano_presentation::{
        TUTORIAL_NANO_FACE_MATERIAL_NAME, TUTORIAL_NANO_FACE_TEXTURE_PATH, TUTORIAL_NANO_MODEL_PATH,
    },
};

use crate::scene_hierarchy::is_descendant_of;

#[cfg(test)]
mod tests;

mod constants;
mod commands;
mod audio;
mod animation_play_tutorial_nano_gameplay_anim;
mod models;
mod types;
mod state;
mod systems;
mod textures;
mod operations;
mod corruption;
mod assets_prepare_tutorial_nano_gameplay_a;

pub use constants::{
    TUTORIAL_BUTTERCUP_NANO_ID, TUTORIAL_BUTTERCUP_SKILL_ID,
    TUTORIAL_BUTTERCUP_INITIAL_STAMINA, TUTORIAL_BUTTERCUP_SKILL_DRAIN,
    TUTORIAL_BUTTERCUP_SKILL_STAMINA_FLOOR, TUTORIAL_BUTTERCUP_SKILL_COOLDOWN_SECONDS,
    TUTORIAL_BUTTERCUP_SKILL_CONDITION, TUTORIAL_BUTTERCUP_STUN_EFFECT_ID,
    TUTORIAL_BUTTERCUP_STUN_EFFECT_NODE, TUTORIAL_BUTTERCUP_SKILL_RANGE,
    TUTORIAL_BUTTERCUP_SKILL_HALF_ANGLE_DEGREES, TUTORIAL_BUTTERCUP_SKILL_TARGET_CAPACITY,
    TUTORIAL_BUTTERCUP_SUMMON_EFFECT_ID, TUTORIAL_NANO_DISMISS_EFFECT_ID,
    TUTORIAL_NANO_CALL_CROSS_FADE_SECONDS, TUTORIAL_NANO_FOLLOW_RATE,
    TUTORIAL_NANO_PATTERN_SECONDS, TUTORIAL_BUTTERCUP_SKILL_EFFECT_ID,
    TUTORIAL_BUTTERCUP_SKILL_EFFECT_NODE
};
use constants::CHEESE_NANO_ID;
#[cfg(test)]
use constants::GAMEPLAY_REQUIRED_CLIPS;
pub use commands::{
    TUTORIAL_BUTTERCUP_SUMMON_EVENT_SECONDS, TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS,
    TutorialNanoGameplayCommand, TutorialNanoGameplayCommandQueue, TutorialNanoGameplayEvent,
    TutorialNanoGameplayEventQueue
};
pub use audio::{
    TUTORIAL_BUTTERCUP_SUMMON_SFX_TRUE_NAME, TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES,
    TUTORIAL_BUTTERCUP_SKILL_SFX_TRUE_NAME, TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES,
    TutorialNanoGameplayAudioCategory
};
use audio::WorldNanoAnimationSoundCursor;
use animation_play_tutorial_nano_gameplay_anim::{
    CALL_CLIP, SKILL_CLIP, gameplay_nano_asset_clip_name, TutorialNanoSkillAnimationEvents,
    emit_world_nano_animation_sounds, play_tutorial_nano_gameplay_animation,
    emit_tutorial_nano_animation_events
};
#[cfg(test)]
use animation_play_tutorial_nano_gameplay_anim::{
    CHEESE_SKILL1_ASSET_CLIP, TutorialGameplayNanoAnimationPlayback
};
use models::CHEESE_NANO_MODEL_PATH;
pub use types::{
    WorldNanoGameplayPresentation, TutorialNanoGameplayLoadout, TutorialNanoGameplayIssue,
    TutorialNanoGameplayIssueQueue, TutorialGameplayNanoRoot, TutorialGameplayNanoScene,
    TutorialNanoSkillCondition, TutorialNanoGameplaySet, TutorialNanoGameplayPlugin
};
use types::{
    TutorialNanoOrbitPattern, TutorialNanoTurnPattern, TutorialNanoMovementPattern,
    TutorialNanoGameplayAssets
};
#[cfg(test)]
use types::TutorialNanoPulsePattern;
pub use state::{TutorialNanoGameplayStatus, TutorialNanoGameplayState};
use systems::{advance_tutorial_nano_orbit, advance_tutorial_nano_skill_cooldown};
pub use systems::{advance_tutorial_nano_follow, apply_tutorial_nano_gameplay_commands};
use textures::bind_tutorial_gameplay_nano_face_texture;
pub use operations::{tutorial_nano_follow_target, cleanup_tutorial_nano_gameplay};
use operations::{
    equip_gameplay_nano, summon_gameplay_nano, withdraw_gameplay_nano, dismiss_gameplay_nano,
    use_gameplay_nano_skill, play_world_nano_skill, follow_tutorial_gameplay_nano
};
use assets_prepare_tutorial_nano_gameplay_a::prepare_tutorial_nano_gameplay_asset;
