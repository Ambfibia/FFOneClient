//! Client-side network worker isolated from presentation and world rendering.

use std::{
    fmt,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};

use bevy::prelude::Resource;
use ffone_net::{
    BankGameplayFrame0104, BuddyLifecycleGameplayFrame0104, GameplayReceiver, GameplaySender,
    InventoryGameplayFrame0104, ItemUseGameplayFrame0104, LoginSession,
    NanoTuneGameplayFrame0104, NetError, PresentNpcTypesGameplayFrame0104,
    QuickSlotGameplayFrame0104, ShardSession, ShardTicket, VendorGameplayFrame0104, WorldBootstrap,
};
use ffone_protocol::{
    AllGroupFreeChatRequest0104, BuddyAcceptRequest0104, BuddyFindNameAcceptRequest0104,
    BuddyFindNameRequest0104, BuddyFreeChatRequest0104, BuddyMakeRequest0104,
    BuddyRemoveRequest0104, BuddySetBlockRequest0104, BuddyStateRequest0104, BuddyWarpRequest0104,
    CHARACTER_EQUIP_SLOT_COUNT_0104, CharacterCreateRequest0104, CharacterCreateSuccess0104,
    CharacterDeleteSuccess0104, CharacterInfo0104, CharacterNameChangeRequest0104,
    CharacterNameChangeSuccess0104, CharacterNameCheckRequest0104, CharacterNameCheckSuccess0104,
    CharacterNameSaveRequest0104, CharacterNameSaveSuccess0104, CharacterStyle0104,
    CharacterTutorialSaveRequest0104, DecodedFrame, EquippedItem0104, FreeChatRequest0104,
    GmSetValueRequest0104, GroupLeaveRequest0104, ItemChestOpenRequest0104, ItemMoveRequest0104,
    ItemUseRequest0104, NanoActiveRequest0104, NanoEquipRequest0104, NanoSkillUseRequest0104,
    NanoTunePacket0104, NanoTuneRequest0104, NanoUnequipRequest0104, NpcInteractionRequest0104,
    PayloadError, PcBankCloseRequest0104, PcBankOpenRequest0104, PcChangeMentorRequest0104,
    PcDisassembleItemRequest0104, PcGrenadeStyleFireRequest0104, PcItemDeleteRequest0104,
    PcJumpRequest0104, PcLoadData0104, PcMoveRequest0104, PcRegenRequest0104,
    PcRocketStyleFireRequest0104, PcSpecialStateSwitchRequest0104, PcStopRequest0104,
    PcTaskStopRequest0104, PcVehicleOffRequest0104, PcWarpUseNpcRequest0104,
    PresentNpcTypesRequest0104, QuickSlotRegisterRequest0104, RegisteredGameplayRequest0104,
    VendorBatteryBuyRequest0104, VendorItemBuyRequest0104, VendorItemRestoreBuyRequest0104,
    VendorItemSellRequest0104, VendorStartRequest0104, VendorTableUpdateRequest0104, WirePayload,
    fixed_payload_size, packet,
};

#[cfg(test)]
mod tests;

mod constants;
mod types;
mod commands;
#[cfg(test)]
mod state;
mod operations_network_worker;
mod operations_connect_character_shard;
mod assets;
mod animation;
mod output;
mod entities;

pub use constants::{
    TUTORIAL_COMPLETION_SECTOR_V_POSITION_0104, TUTORIAL_COMPLETION_SECTOR_V_ANGLE_0104
};
use constants::TUTORIAL_GAMEPLAY_EXIT_TIMEOUT;
pub use types::{
    CharacterEntryLocation0104, CharacterSummary, WorldReady, CharacterOperationStage,
    NanoTunePending0104, NanoTuneCorrelationFault0104, NanoTuneCorrelationError0104,
    CorrelatedNanoTuneReply0104, NetworkStream0104, DisconnectReason0104, NetworkBridge
};
use types::{GameplayHandle, GameplayReaderFinished};
pub use commands::{NetworkCommand, NetworkEvent};
use commands::{
    scripted_entry_move_request,
    registered_fixed_world_action_0104, registered_nano_skill_use_request_0104,
    send_registered_world_action_0104
};
#[cfg(test)]
use state::{RetainedLoginSelection, RETAINED_LOGIN_SELECTION_TIMEOUT};
use operations_network_worker::network_worker;
#[cfg(test)]
use operations_network_worker::character_entry_location;
use operations_connect_character_shard::{
    connect_character_shard, character_summaries,
    send_login_required, send_gameplay, disconnect_gameplay
};
use assets::CharacterEntryRoute;
use animation::character_entry_pose;
use output::{publish_character_entry_failure, publish_world_then_spawn_gameplay_reader};
use entities::spawn_gameplay_reader;
#[cfg(test)]
use entities::spawn_login_world_reader;
#[cfg(test)]
use ffone_net::LoginWorldSender;
#[cfg(test)]
use types::LoginWorldHandle;
#[cfg(test)]
use commands::request_retained_shard_ticket;
#[cfg(test)]
use std::sync::mpsc::Sender;
