//! Exact, renderer-independent presentation commands for the tutorial player.
//!
//! Retrobution does not route the three tutorial poses through normal
//! locomotion. `cntutorialscript` calls `cnAvatarAnimation.AvatarEmote(string)`
//! directly for `staying`, `standup`, and `run`. That overload performs an
//! immediate `Animation.Play`, marks the avatar as emoting, and hides hand
//! attachment `AttachName(3)`. `AvatarStandForce`, by contrast, resolves
//! `stand1`, resets the current movement direction, and calls
//! `Animation.CrossFade(..., 0.15f)`.
//!
//! All primary tutorial poses keep that original dispatch. Only FFR-authored
//! additive emotes enter through their short native cross-fade.
//! This module preserves that distinction and the exact primary,
//! gender-specific Unity
//! AnimationClip PathIDs. It deliberately does not alias a missing clip to a
//! different animation. The gameplay renderer assembles the selected account
//! character and resolves every request against the clips actually published
//! on that player rig before reporting playback.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fmt,
    time::Duration,
};

use bevy::prelude::Resource;
use ffone_protocol::{CharacterEquipSlot0104, EquippedItem0104};
use ffone_runtime_contracts::{PlayerRigClipContract, PlayerRigGender};
use serde_json::Value;

use crate::{
    assets::AssetLocator,
    avatar_action::{
        LEGACY_ANIMATION_BLEND_SECONDS, LEGACY_END_ANIMATION_BLEND_SECONDS, LegacyWeaponTargetMode,
    },
};

#[cfg(test)]
mod tests;

mod constants;
mod animation_player_weapon_animation_catalog;
mod animation_tutorial_player_clip_is_vehicle;
mod animation_tutorial_player_clip_from_exact_name;
mod animation_tutorial_player_rig_capabilities;
mod state;
mod operations;
mod types;
mod audio;
mod commands;
mod validation;

pub use constants::{
    FFR_CUSTOM_EMOTE_CROSS_FADE_SECONDS, AVATAR_EMOTE_CODE_CROSS_FADE_SECONDS,
    TUTORIAL_STAND_FORCE_CROSS_FADE_SECONDS, TUTORIAL_DIRECT_AVATAR_EMOTE_NAMES,
    FFR_CUSTOM_AVATAR_EMOTE_NAMES, TUTORIAL_WEAPON_SLOT, TUTORIAL_WEAPON_ITEM_TYPE
};
pub use animation_player_weapon_animation_catalog::{
    TUTORIAL_END_ANIMATION_CROSS_FADE_SECONDS, TutorialPlayerClip, TutorialPlayerClipPlayback,
    PlayerWeaponAnimationProfile, PlayerWeaponAnimationCatalog
};
pub use animation_tutorial_player_clip_from_exact_name::{
    TutorialPlayerAnimationDispatch, TutorialPlayerAnimationRequest,
    TutorialPlayerClipAvailability, ResolvedTutorialPlayerAnimation,
    MissingTutorialPlayerClip, TutorialPlayerRigCapabilityError, TutorialPlayerRigCapabilities
};
pub use animation_tutorial_player_rig_capabilities::ContractResolvedTutorialPlayerAnimation;
pub use state::TUTORIAL_PLAYER_RUNTIME_READY_STATUS;
pub use operations::{tutorial_stand_force_cross_fade, tutorial_player_gender_from_protocol};
pub use types::{
    PlayerWeaponCombatProfile, TutorialPlayerPresentationResolution,
    TutorialPlayerPresentationConsumer
};
use types::PlayerWeaponAttackSounds;
use audio::weapon_sound_variants;
pub use commands::{
    TutorialPlayerEquipmentRequest, tutorial_weapon_request,
    TutorialPlayerPresentationCommand, TutorialPlayerPresentationCommandQueue
};
pub use validation::TutorialPlayerPresentationError;
