//! Source-proven gameplay SFX for the native tutorial/player runtime.
//!
//! The cue names and times below come from the primary Retrobution
//! `AnimationClip.m_Events` and native table rows. Runtime lookup remains
//! strictly semantic through [`NativeAudioCatalog`]; legacy bundles are never
//! opened by the client.

use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    animation::RepeatAnimation,
    audio::{SpatialListener, SpatialScale, Volume},
    prelude::*,
};
use ffone_runtime_contracts::PlayerRigGender;

use crate::{
    audio_channel::{GameplayAudioChannel, GameplayChannelMixPlugin},
    avatar_action::{LegacyAvatarActionSet, LegacyAvatarActionState, LegacyLocomotionState},
    localization::{LocalizedVoice, VoiceLanguage},
    movement::LegacyPlayerController,
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    tutorial_actors::{
        TutorialActor, TutorialActorAnimationPlayback, apply_tutorial_actor_animation_playback,
    },
    tutorial_choreography::SpatialAudioTarget,
    tutorial_choreography_runtime::TutorialChoreographyPresentation,
    tutorial_player_presentation::{PlayerWeaponAnimationCatalog, PlayerWeaponAnimationProfile},
    tutorial_player_rig_runtime::TutorialSelectedPlayerRig,
    user_equip_ui::UserEquipUiState,
};

#[path = "nano.rs"]
mod nano;

macro_rules! sound_event {
    ($seconds:expr, $cue:expr) => {
        LegacySoundEvent {
            seconds: $seconds,
            cue: $cue,
        }
    };
}

macro_rules! audio_clip {
    ($clip:literal, $duration:expr, $seconds:expr, $cue:expr) => {
        ActorClipAudio {
            clip: $clip,
            duration_seconds: $duration,
            events: &[sound_event!($seconds, $cue)],
        }
    };
}

#[cfg(test)]
mod tests;

mod constants;
mod audio_gameplay_audio_runtime;
mod audio_drive_gameplay_audio;
mod operations;
mod commands;
mod animation_animation_event_crossings;

pub(crate) use constants::LEGACY_SPATIAL_SCALE;
use constants::{
    SPAWN_CLIPS, CERBERUS_CLIPS, OIL_CLIPS, BAT_CLIPS, BUTTERCUP_CLIPS, NUMBUH_ONE_CLIPS,
    DEXTER_PISTOL_CLIPS, NUMBUH_FIVE_CLIPS, BEN_CLIPS, SAMURAI_JACK_CLIPS, DEXTER_CLIPS,
    DEXTER_SWORD_CLIPS
};
use audio_gameplay_audio_runtime::{
    SoundCue, SelectedSound, LegacySoundEvent, ActorClipAudio, PlayerLocomotionAudioCursor
};
#[cfg(test)]
use audio_gameplay_audio_runtime::{
    actor_clip_audio, QueuedAnimationSound, GameplayAudioRandom, legacy_npc_voice_true_name,
    expand_legacy_random_sound
};
pub use audio_gameplay_audio_runtime::{
    LegacyNpcVoiceCue, legacy_npc_open_voice_cue, GameplayAudioRuntime, GameplaySfxAudio,
    RetrobutionAudioMix, GameplayAudioSet, GameplayAudioPlugin
};
use audio_drive_gameplay_audio::drive_gameplay_audio;
#[cfg(test)]
use audio_drive_gameplay_audio::legacy_spatial_sound_gate;
use operations::{
    actor_clips, player_damage_cue, player_infection_damage_cue, semantic_numeric_family,
    player_locomotion_cue, player_locomotion_loop_duration
};
use commands::player_weapon_attack_event_seconds;
use animation_animation_event_crossings::animation_event_crossings;
