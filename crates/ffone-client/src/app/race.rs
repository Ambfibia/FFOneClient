//! Race instance frames, NPC sessions, production runtime and outputs.

use super::local_inventory::LocalInventoryRuntime;
use super::option_runtime::{OptionProductionRuntime, option_channel_gain};
use super::runtime_status::{RuntimeStatus, legacy_avatar_max_fusion_matter};
use super::{LocalPlayer, WorldSliceEntity};
use bevy::{
    audio::Volume,
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    coordinates::ProtocolPosition,
    gameplay_ui::NpcServiceKind,
    localization::{LocalizedVoice, VoiceLanguage},
    mission_ui::MissionUiModel,
    network::{NetworkBridge, NetworkCommand},
    race_ui::{
        hud::RaceHudState,
        mode::{
            RACE_CANCEL_FAILURE_PACKET_ID, RACE_CANCEL_SUCCESS_PACKET_ID,
            RACE_END_FAILURE_PACKET_ID, RACE_END_SUCCESS_PACKET_ID, RACE_MODE_GAME_MODE_ID,
            RACE_RANK_GAME_MODE_ID, RACE_START_FAILURE_PACKET_ID, RACE_START_SUCCESS_PACKET_ID,
            RaceEcomOperation, RaceEcomType, RaceModeEffect, RaceModeModel, RaceModeOpenContext,
            RaceModeOutput, RaceModePhase, RaceModePresentationInput, RaceModeReply,
            RaceModeUiCommand, RaceModeUiCommandOutbox, RaceNpcContext, RacePlayerState,
            RaceReplyEnvelope, RaceRewardPresentation, RaceSound, decode_race_reply_0104,
        },
        rank::{
            RaceRankCatalog, RaceRankEffect, RaceRankModel, RaceRankOutput,
            RaceRankPresentationInput, RaceRankUiCommand, RaceRankUiCommandOutbox,
        },
    },
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    system_message_ui::{
        SystemMessageRequest, SystemMessageUiAction, SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::TutorialMissionContent,
    world_audio::RetrobutionInstanceAudioState,
    world_behaviour::WorldGameplayIntentQueue,
};
use ffone_protocol::{DecodedFrame, ItemBase0104, ItemReward0104};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod codec;
mod constants;
mod audio_pending_race_npc_voice;
mod types;
mod input;
mod projects;
mod state;
mod assets;
mod operations;
mod commands;
mod systems;
mod entities;

pub(super) use codec::{
    RACE_INSTANCE_MAP_INFO_PACKET_ID, RACE_GET_RING_SUCCESS_PACKET_ID,
    RACE_GET_RING_FAILURE_PACKET_ID, decode_race_instance_map_info_0104,
    decode_race_ring_success_0104, decode_race_ring_failure_0104, RaceNetworkFrameInbox
};
#[cfg(test)]
pub(super) use codec::race_frame_owned;
pub(super) use constants::{
    RACE_INSTANCE_MAP_INFO_BASE_SIZE, RACE_INSTANCE_MAP_SWITCH_SIZE,
    RACE_SYSTEM_MESSAGE_ID_BASE, RACE_RANK_HTTP_TRANSPORT_AVAILABLE, RACE_RANK_HTTP_GAP
};
pub(super) use audio_pending_race_npc_voice::{
    RACE_START_AUDIO_PATH, RACE_FINISH_AUDIO_PATH, RACE_RING_AUDIO_PATH,
    RACE_BUTTON_SOUND_GAIN, PendingRaceNpcVoice
};
pub(super) use types::{RaceInstanceMapInfo0104, RaceProductionOwners};
pub(super) use input::read_race_i32;
pub(super) use projects::{RaceNpcSession, reset_race_session};
pub(super) use state::{
    RaceProductionRuntime, open_race_mode_from_npc, consume_race_mode_ui_commands,
    abort_race_mode_transport
};
pub(super) use assets::{CleanRaceNpcRoute, clean_race_npc_route, warp_away_xcom_index};
pub(super) use operations::{
    race_service_allowed, consume_race_network_frames, consume_race_rank_ui_commands,
    set_race_cursor_locked, consume_race_production_outputs,
    consume_race_system_message_outbox, reset_race_shell, open_race_fail_from_warp_away
};
pub(super) use commands::apply_correlated_race_reply_0104;
pub(super) use systems::{sync_race_presentation_input, sync_race_world_rings, sync_race_hud, cancel_expired_race};
pub(super) use entities::spawn_race_npc_voice;
