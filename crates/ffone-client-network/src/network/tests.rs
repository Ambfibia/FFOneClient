use super::*;
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use ffone_protocol::{
    AllGroupFreeChatSuccess0104, BuddyBaseInfo0104, BuddyFreeChatSuccess0104,
    BuddyLifecyclePacket0104, BuddyListInfo0104, BuddyStateSuccess0104, CharacterInfo0104,
    CharacterSelectRequest, DEFAULT_KEY, EquipChangePacket0104, FixedUtf16,
    FreeChatSuccess0104, InventoryPacket0104, ItemBase0104, ItemMoveSuccessPacket0104,
    ItemUsePacket0104, ItemUseSkillResults0104, ItemUseSuccessPrefix0104, LoginSuccess,
    NANO_TUNE_ITEM_SLOT_COUNT_0104, NanoTuneFailure0104, NanoTuneSuccess0104,
    PcExitSuccess0104, PcGotoSuccess0104, PcLoadData0104, PcLoadingCompleteSuccess,
    PresentNpcTypesPacket0104, PresentNpcTypesReply0104, QuickSlotEntry0104, QuickSlotInfo0104,
    QuickSlotPacket0104, ShardSelectSuccess, SkillResultBuff0104, VendorPacket0104,
    VendorStartSuccess0104, WirePayload, decode_client_frame, derive_login_e_key,
    encode_client_frame, encode_server_frame, packet,
};

mod constants;
mod codec;
mod input;
mod operations;
mod commands;
mod output;
mod state;
mod assets;
mod animation;
mod character_session;

use constants::{TUTORIAL_TEST_UID, TUTORIAL_TEST_SERVER_TIME};
use codec::read_tutorial_wire;
use input::read_tutorial_client;
use operations::send_tutorial_server;
use state::send_tutorial_shard_selection;
