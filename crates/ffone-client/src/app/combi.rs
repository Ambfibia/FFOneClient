//! Combi (item combination) production catalog, shell, network frames and outputs.

use super::WorldSliceEntity;
use super::email_catalog::EMAIL_ITEM_TABLES_0104;
use super::local_inventory::LocalInventoryRuntime;
use super::option_runtime::{OptionProductionRuntime, option_channel_gain};
use super::runtime_status::RuntimeStatus;
use super::world_intents::queue_service_farewell;
use bevy::{
    audio::Volume,
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    assets::AssetLocator,
    combi_runtime::{
        CombiAuthoritativeCommit0104, CombiModeLease0104, CombiNpcAnimation0104,
        CombiPlayerAuthority0104, CombiProductionError0104, CombiProductionOutput0104,
        CombiProductionRuntime0104, CombiShellEffect0104, CombiSystemMessage0104,
    },
    combi_ui::{
        COMBI_FAILURE_PACKET_ID_0104, COMBI_RECIPE_TABLE_PATH, COMBI_SUCCESS_PACKET_ID_0104,
        CombiAuthoritativeSnapshot0104, CombiExternalModalState0104, CombiItemCatalog0104,
        CombiItemMetadata0104, CombiModalChoice0104, CombiModeProjection0104, CombiPhase0104,
        CombiRecipeTable0104, CombiSelectionChange0104, CombiSystemModal0104, CombiUiOutbox0104,
        CombiUiState0104, clean_type_label, combined_appearance_item_id,
        localized_combi_equipped_item_message, localized_combi_wire_error,
    },
    guide_runtime::GuideRuntime,
    inventory_runtime::{InventoryLocation0104, InventoryRuntime0104},
    mission_ui::MissionUiModel,
    network::{NetworkBridge, NetworkCommand},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::TutorialMissionContent,
    user_equip_ui::{
        UserEquipCatalogQuery, UserEquipItemCatalog, UserEquipModalState, UserEquipUiState,
    },
};
use ffone_protocol::{DecodedFrame, ItemBase0104, ItemMoveSuccessPacket0104};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod constants;
mod audio_combi_ui_mode_sound_true_name;
mod animation;
mod containers;
mod types;
mod operations;
mod state;
mod codec;
mod projects;
mod entities;
mod systems;

pub(super) use constants::{
    COMBI_SYSTEM_MESSAGE_ID_BASE_0104, COMBI_HELP_OWNER_GAP_0104, COMBI_ITEM_TABLES_0104
};
pub(super) use audio_combi_ui_mode_sound_true_name::{
    COMBI_UI_MODE_SOUND_TRUE_NAME_0104, COMBI_UI_MODE_SOUND_GAIN_0104
};
pub(super) use animation::COMBI_ANIMATION_OWNER_GAP_0104;
pub(super) use containers::{
    COMBI_SERIALIZED_RARITY_LABEL_0104, COMBI_SERIALIZED_TRADE_LABEL_0104
};
pub(super) use types::{
    CombiProductionCatalog0104, PendingCombiSystemAction0104, PendingCombiSystemMessage0104,
    CombiProductionShell0104, CombiProductionOwners0104
};
pub(super) use operations::{
    combi_table_i32_0104, combi_table_i32_default_zero_0104, combi_table_string_0104,
    combi_player_authority_0104, reset_combi_shell_0104, consume_combi_network_frames_0104,
    drive_combi_production_0104, consume_combi_system_message_outbox_0104,
    restore_combi_cursor_0104, consume_combi_production_outputs_0104
};
pub(super) use state::{RuntimeCombiItemCatalog0104, combi_inventory_after_commit_0104};
pub(super) use codec::CombiNetworkFrameInbox0104;
pub(super) use projects::reset_combi_session_0104;
pub(super) use entities::spawn_combi_sfx_0104;
pub(super) use systems::sync_combi_presentation_0104;
