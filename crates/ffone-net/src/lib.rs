//! Blocking, headless OpenFusion login-to-world transport for protocol 0104.
//!
//! Passwords are accepted only as call arguments, encoded immediately, and never retained in a
//! session or included in an error.

#![forbid(unsafe_code)]

use std::{
    fmt,
    io::{self, Read, Write},
    net::{IpAddr, Shutdown, SocketAddr, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    time::Duration,
};

use ffone_protocol::{
    AllGroupFreeChatRequest0104, AroundDecodeError, BuddyAcceptRequest0104,
    BuddyFindNameAcceptRequest0104, BuddyFindNameRequest0104, BuddyFreeChatRequest0104,
    BuddyLifecyclePacket0104, BuddyMakeRequest0104, BuddyRemoveRequest0104,
    BuddySetBlockRequest0104, BuddyStateRequest0104, BuddyWarpRequest0104,
    CharacterCreateFailure0104, CharacterCreateRequest0104, CharacterCreateSuccess0104,
    CharacterDeleteFailure0104, CharacterDeleteRequest0104, CharacterDeleteSuccess0104,
    CharacterInfo0104, CharacterNameChangeFailure0104, CharacterNameChangeRequest0104,
    CharacterNameChangeSuccess0104, CharacterNameCheckFailure0104, CharacterNameCheckRequest0104,
    CharacterNameCheckSuccess0104, CharacterNameSaveFailure0104, CharacterNameSaveRequest0104,
    CharacterNameSaveSuccess0104, CharacterSelectRequest, CharacterTutorialSaveRequest0104,
    DEFAULT_KEY, DecodedFrame, DuplicateExitRequest0104, EnvironmentDotToggle0104, FixedUtf16,
    FrameError, FreeChatRequest0104, GmSetValueRequest0104, GroupLeaveRequest0104,
    InitialAroundPacket0104, InventoryPacket0104, ItemChestOpenRequest0104, ItemMoveRequest0104,
    ItemUseDecodeError0104, ItemUsePacket0104, ItemUseRequest0104, LegacyClientEncoder,
    LoginFailure, LoginRequest, LoginSuccess, MAX_BODY_SIZE_0104, NanoTunePacket0104,
    NanoTuneRequest0104, NpcCombatDecodeError0104, NpcCombatPacket0104, NpcInteractionRequest0104,
    PayloadError, PcAttackCharsRequest0104, PcAttackNpcsRequest0104, PcBankCloseRequest0104,
    PcBankOpenRequest0104, PcBankReply0104, PcChangeMentorRequest0104, PcCombatStateRequest0104,
    PcDisassembleItemRequest0104, PcEnterFailure, PcEnterRequest, PcEnterSuccess,
    PcExitRequest0104, PcGrenadeStyleFireRequest0104, PcItemDeleteRequest0104, PcJumpRequest0104,
    PcLoadData0104, PcLoadingCompleteRequest, PcLoadingCompleteSuccess, PcMoveRequest0104,
    PcRegenRequest0104, PcRocketStyleFireRequest0104, PcSpecialStateSwitchRequest0104,
    PcStopRequest0104, PcTaskStopRequest0104, PcWarpUseNpcRequest0104,
    PresentNpcTypesDecodeError0104, PresentNpcTypesPacket0104, PresentNpcTypesRequest0104,
    QuickSlotPacket0104, QuickSlotRegisterRequest0104, RegisteredGameplayRequest0104,
    ShardSelectFailure, ShardSelectSuccess, VendorBatteryBuyRequest0104, VendorItemBuyRequest0104,
    VendorItemRestoreBuyRequest0104, VendorItemSellRequest0104, VendorPacket0104,
    VendorStartRequest0104, VendorTableUpdateRequest0104, WirePayload,
    decode_buddy_lifecycle_packet_0104, decode_initial_around_0104, decode_inventory_packet_0104,
    decode_item_use_packet_0104, decode_nano_tune_packet_0104, decode_npc_combat_packet_0104,
    decode_pc_bank_reply_0104, decode_present_npc_types_packet_0104, decode_quick_slot_packet_0104,
    decode_server_frame, decode_vendor_packet_0104, derive_frontend_key, derive_login_e_key,
    derive_shard_e_key, packet,
    wire_0104::{LsCharSelectFailure0104, LsShardSelectRequest0104},
};

#[cfg(test)]
mod tests;

mod transport_types_gameplay_sender;
mod transport_types_gameplay_receiver;
mod transport_constants;
mod transport_validation;
mod transport_codec;
mod transport_projects_login_session;
mod transport_projects_shard_session;
mod transport_commands;
mod transport_operations;
mod transport_state;

pub use transport_types_gameplay_sender::{
    Result, LoginWorldSender, LoginWorldReceiver, LoginWorldConnection, ShardTicket,
    GameplaySender, NpcCombatGameplayFrame0104, QuickSlotGameplayFrame0104,
    ItemUseGameplayFrame0104, BankGameplayFrame0104, NanoTuneGameplayFrame0104,
    VendorGameplayFrame0104, BuddyLifecycleGameplayFrame0104, PresentNpcTypesGameplayFrame0104
};
use transport_types_gameplay_sender::{LoginWorldWriter, GameplayWriter};
pub use transport_types_gameplay_receiver::{
    GameplayReceiver, GameplayConnection, LoadingComplete, WorldBootstrap
};
pub use transport_constants::{
    DEFAULT_CONNECT_TIMEOUT, DEFAULT_IO_TIMEOUT, CLEAN_WEB_PLAYER_SHARD_NUM_0104
};
pub use transport_validation::NetError;
pub use transport_codec::{TcpFrameIo, WorldBootstrapPacket};
use transport_codec::{decode_server_announcement, shard_ticket_from_login_frame, heartbeat_payload};
pub use transport_projects_login_session::{LoginSession, ShardSession};
use transport_commands::clean_shard_select_request_0104;
use transport_operations::character_select_rejected_0104;
pub use transport_state::InventoryGameplayFrame0104;
